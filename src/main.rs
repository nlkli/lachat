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

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 8080;
const DEFAULT_SE_PATH: &str = "/tmp/lachat";

fn extract_addr_params<'a>(args: &'a [String]) -> (&'a str, u16) {
    let host = args
        .iter()
        .position(|a| a == "--host")
        .and_then(|i| args.get(i + 1).map(String::as_str))
        .unwrap_or(DEFAULT_HOST);
    let port = args
        .iter()
        .position(|a| a == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    (host, port)
}

fn launch_llama_server(args: &[String]) -> Result<u32> {
    let mut args = args.to_vec();
    if !args.contains(&"--sleep-idle-seconds".into()) {
        args.push("--sleep-idle-seconds".into());
        args.push("3600".into());
    }
    process::spawn_detached("llama-server", args)
}

fn main() -> Result<()> {
    let res = minreq::get("http://127.0.0.1:8081/health").send();
    if let Err(e) = res {
        println!("{}", e.to_string());
    }
    return Ok(());

    let stdin = utils::read_stdin()?;
    let args = cli::Args::parse();

    let se = session::Session::new(args.get_or("S", DEFAULT_SE_PATH))?;
    let mut state = if let Some(mut st) = se.read_state()? {
        if args.kill && st.llamacpp_pid != 0 {
            process::kill_pid(st.llamacpp_pid)?;
            st.llamacpp_pid = 0;
            se.write_state(&st)?;
            return Ok(());
        }
        st
    } else {
        let (host, port) = extract_addr_params(&args.llama_server_args);
        let pid = launch_llama_server(&args.llama_server_args)?;
        session::State {
            llamacpp_pid: pid,
            llamacpp_host: host.into(),
            llamacpp_port: port,
        }
    };

    let base_url = format!("http://{}:{}", state.llamacpp_host, state.llamacpp_port);
    let client = laserv::Client::new(base_url);
    if !client.health() {
        let mut llama_server_args = args.llama_server_args.clone();
        let host_pos = llama_server_args.iter().position(|a| a == "--host");
        if let Some(pos) = host_pos {
            state.llamacpp_host = llama_server_args[pos+1].clone();
        } else {
            llama_server_args.push("--host".into());
            llama_server_args.push(state.llamacpp_host.clone());
        }
        let port_pos = llama_server_args.iter().position(|a| a == "--port");
        if let Some(pos) = port_pos {
            state.llamacpp_port = llama_server_args[pos+1].parse().unwrap_or(DEFAULT_PORT);
        } else {
            llama_server_args.push("--port".into());
            llama_server_args.push(state.llamacpp_port.to_string());
        }
        state.llamacpp_pid = launch_llama_server(&llama_server_args)?;
    }

    se.write_state(&state)?;

    let available_models = client.available_models()?;
    if available_models.is_empty() {
        return Err("empty available models".into());
    }
    let mut model = available_models.first_model_name().unwrap();
    if let Some(ref m) = args.model {
        model = utils::fuzzy_search(&available_models.name_list(), m).unwrap_or(model);
    }

    let mut crb = openai::CompletionRequest::builder(model);
    let mut messages = Vec::new();

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
            messages.push(openai::Message::system(content));
        } else {
            messages.push(openai::Message::system(system.clone()));
        }
    }

    if !stdin.is_empty() {
        messages.push(openai::Message::user(stdin));
    }

    for p in args.prompt.iter() {
        if utils::is_existing_file(p) {
            let content = std::fs::read_to_string(p)?;
            messages.push(openai::Message::user(content));
        } else {
            messages.push(openai::Message::user(p.clone()));
        }
    }

    if !messages.is_empty() {
        let cr = crb.stream(true).build();
        client.write_chat_completions(&cr, io::stdout())?;
        println!();
        // TODO background
    }

    if args.interactive {
        let cr = openai::CompletionRequest::builder(model)
            .stream(true)
            .build();
        let ch = chat::Chat::new(client.clone(), cr);
        chat::interactive_chat(ch, std::io::stdout(), std::io::stdin())?;
    }

    Ok(())
}
