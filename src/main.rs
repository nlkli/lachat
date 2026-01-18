mod chat;
mod cli;
mod laserv;
mod models;
mod process;
mod session;
mod sse;
mod utils;
use models::openai;
use std::io::{self, Write};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: u16 = 8080;

const DEFAULT_SE_PATH: &str = "/tmp/lachat";

fn launch_llama_server(args: &[String]) -> Result<u32> {
    let mut args = args.to_vec();
    if !args.contains(&"--sleep-idle-seconds".into()) {
        args.push("--sleep-idle-seconds".into());
        args.push("3600".into());
    }
    process::spawn_detached("llama-server", args)
}

fn main() -> Result<()> {
    let stdin = utils::read_stdin()?;
    let args = cli::Args::parse();

    let se = session::Session::new(args.get_or("S", DEFAULT_SE_PATH))?;
    let mut state = if let Some(mut st) = se.read_state()? {
        if args.kill && st.pid != 0 {
            process::kill_pid(st.pid)?;
            st.pid = 0;
            se.write_state(&st)?;
            return Ok(());
        }
        st
    } else {
        let (host, port) = args.extract_llama_addr();
        let pid = launch_llama_server(&args.llama_server_args)?;
        session::State {
            pid,
            host: host.into(),
            port,
        }
    };

    if !args.llama_server_args.is_empty() {
        
    }

    let base_url = format!("http://{}:{}", state.host, state.port);
    let client = laserv::Client::new(base_url);
    if !client.health() {
        let mut llama_server_args = args.llama_server_args.clone();
        let host_pos = llama_server_args.iter().position(|a| a == "--host");
        if let Some(pos) = host_pos {
            state.host = llama_server_args[pos + 1].clone();
        } else {
            llama_server_args.push("--host".into());
            llama_server_args.push(state.host.clone());
        }
        let port_pos = llama_server_args.iter().position(|a| a == "--port");
        if let Some(pos) = port_pos {
            state.port = llama_server_args[pos + 1].parse().unwrap_or(DEFAULT_PORT);
        } else {
            llama_server_args.push("--port".into());
            llama_server_args.push(state.port.to_string());
        }
        state.pid = launch_llama_server(&llama_server_args)?;
    }

    se.write_state(&state)?;
    let base_url = format!("http://{}:{}", state.host, state.port);
    let client = laserv::Client::new(base_url).wait(15000)?;

    let available_models = client.available_models()?;
    if available_models.is_empty() {
        return Err("empty available models".into());
    }
    let mut model = available_models.first_model_name().unwrap();
    if let Some(ref m) = args.model {
        model = utils::fuzzy_search(&available_models.name_list(), m).unwrap_or(model);
    }

    let mut crb = openai::CompletionRequest::builder(model);
    let mut chat = if let Some(ref chat_id) = args.chat {
        if let Ok(Some(chat)) = se.read_chat(chat_id) {
            chat
        } else {
            session::Chat::new()
        }
    } else {
        session::Chat::new()
    };

    if let Some(t) = args
        .temperature
        .as_ref()
        .and_then(|t| t.parse::<f32>().ok())
        .map(|f| f.abs().max(1.).min(0.))
    {
        crb = crb.temperature(t);
    }

    if let Some(ref system) = args.system {
        if utils::is_existing_file(system) {
            let content = std::fs::read_to_string(system)?;
            chat.push(openai::Message::system(content));
        } else {
            chat.push(openai::Message::system(system.clone()));
        }
    }

    if !stdin.is_empty() {
        chat.push(openai::Message::user(stdin));
    }

    for p in args.prompt.iter() {
        if utils::is_existing_file(p) {
            let content = std::fs::read_to_string(p)?;
            chat.push(openai::Message::user(content));
        } else {
            chat.push(openai::Message::user(p.clone()));
        }
    }

    if chat.is_empty() {
        return Ok(());
    };

    let cr = crb.messages(chat).stream(true).build();
    if args.interactive {
        let mut ch = chat::Chat::new(client.clone(), cr);
        chat::interactive_chat(&mut ch, std::io::stdout(), std::io::stdin())?;
        if let Some(ref chat_id) = args.chat {
            se.write_chat(chat_id, ch.messages())?;
        }
        return Ok(());
    }

    let mut buff: Vec<u8> = Vec::new();
    let w = utils::DualWriter {
        w1: io::stdout(),
        w2: &mut buff,
    };
    client.write_chat_completions(&cr, w)?;
    println!();
    if let Some(ref chat_id) = args.chat {
        let mut chat = Vec::with_capacity(cr.messages.len() + 1);
        chat.extend_from_slice(&cr.messages);
        chat.push(openai::Message::assistant(String::from_utf8(buff)?));
        se.write_chat(chat_id, &chat)?;
    }
    // TODO background

    Ok(())
}
