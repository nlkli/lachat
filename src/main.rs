mod chat;
mod cli;
mod laserv;
mod models;
mod process;
mod session;
mod sse;
mod utils;
use models::openai;
use std::io;

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

    let se = session::Session::new(args.get_or(
        "S",
        &std::env::var("LACHAT_SESSION").unwrap_or(DEFAULT_SE_PATH.into()),
    ))?;
    let mut state = if let Some(st) = se.read_state()? {
        st
    } else {
        let (host, port) = args.extract_llama_addr();
        let host = host.unwrap_or(DEFAULT_HOST).to_string();
        let port = port.unwrap_or(DEFAULT_PORT);
        let pid = launch_llama_server(&args.llama_server_args)?;
        laserv::Client::new(format!("http://{}:{}", host, port)).wait(15000)?;
        let st = session::State { pid, host, port };
        se.write_state(&st)?;
        st
    };

    if !args.llama_server_args.is_empty() {
        let (host, port) = args.extract_llama_addr();
        let host = host.unwrap_or(&state.host).to_string();
        let port = port.unwrap_or(state.port);
        if host != state.host || port != state.port {
            if state.pid != 0 {
                process::kill_pid(state.pid)?;
                state.pid = 0;
                se.write_state(&state)?;
            }
            let pid = launch_llama_server(&utils::extend_args(
                &args.llama_server_args,
                &["--host", host.as_str(), "--port", port.to_string().as_str()],
            ))?;
            laserv::Client::new(format!("http://{}:{}", host, port)).wait(15000)?;
            state.pid = pid;
            state.host = host;
            state.port = port;
            se.write_state(&state)?;
        }
    }

    if state.pid == 0 {
        let pid = launch_llama_server(&utils::extend_args(
            &args.llama_server_args,
            &[
                "--host",
                state.host.as_str(),
                "--port",
                state.port.to_string().as_str(),
            ],
        ))?;
        laserv::Client::new(format!("http://{}:{}", state.host, state.port)).wait(15000)?;
        state.pid = pid;
        se.write_state(&state)?;
    }

    let client = laserv::Client::new(format!("http://{}:{}", state.host, state.port));
    if !client.health() {
        panic!("llama-server is not responding");
    }

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
        se.write_state(&state)?;
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

    if let Some(t) = args
        .max_tokens
        .as_ref()
        .and_then(|t| t.parse::<u32>().ok())
    {
        crb = crb.max_tokens(t);
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

    let have_messages = !chat.is_empty();
    let cr = crb.messages(chat).stream(true).build();

    if have_messages {
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
    }

    if args.interactive {
        let mut ch = chat::Chat::new(&client, cr);
        chat::interactive_chat(&mut ch, std::io::stdout(), std::io::stdin())?;
        if let Some(ref chat_id) = args.chat {
            se.write_chat(chat_id, ch.messages())?;
        }
    }

    if args.kill && state.pid != 0 {
        process::kill_pid(state.pid)?;
        state.pid = 0;
        se.write_state(&state)?;
    }

    Ok(())
}
