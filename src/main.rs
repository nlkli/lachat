mod chat;
mod cli;
mod laserv;
mod models;
mod process;
mod session;
mod sse;
mod utils;
use std::io;

use models::openai;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "9909";
const DEFAULT_SE_PATH: &str = "/tmp/lachat";

fn launch_llama_server(host: &str, port: &str) -> Result<u32> {
    let pid = process::spawn_detached(
        "llama-server",
        &[
            "--host",
            host,
            "--port",
            port,
            "--sleep-idle-seconds",
            "3600",
        ],
    )?;

    Ok(pid)
}

fn main() -> Result<()> {
    let stdin = utils::read_stdin()?;
    let args = cli::Args::parse();
    println!("{:#?}", args);

    let host = args.get_or("h", DEFAULT_HOST);
    let port = args.get_or("P", DEFAULT_PORT);

    let se = session::Session::new(args.get_or("S", DEFAULT_SE_PATH))?;
    let state = if let Some(st) = se.read_state()? {
        st
    } else {
        let pid = launch_llama_server(host, port)?;
        session::State { llamacpp_pid: pid, llamacpp_port: port.parse()? }
    };

    let client = laserv::Client::new(&format!("{host}:{port}"));
    if !client.health() {
        if let Ok(state) = se.read_state() {}

        // TODO
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
