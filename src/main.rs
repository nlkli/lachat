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

fn main() -> Result<()> {
    let args = cli::Args::parse();
    println!("{:#?}", args);

    let _se = session::Session::new(args.get_or("session", DEFAULT_SE_PATH))?;

    let host = args.get_or("host", DEFAULT_HOST);
    let port = args.get_or("port", DEFAULT_PORT);

    let client = laserv::Client::new(&format!("{host}:{port}"));
    if !client.health() {
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

    let mut messages = Vec::new();

    if let Some(ref system) = args.system {
        messages.push(openai::Message::system(system.clone()));
    }

    let stdin = utils::read_stdin()?;
    if !stdin.is_empty() {
        messages.push(openai::Message::user(stdin));
    }

    if let Some(ref prompt) = args.prompt {
        messages.push(openai::Message::user(prompt.clone()));
    }

    if !messages.is_empty() {
        let cr = &openai::CompletionRequest::builder(model)
            .messages(messages)
            .stream(true)
            .build();
        client.write_chat_completions(cr, io::stdout())?;
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
