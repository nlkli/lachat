mod models;
mod sse;
mod chat;
mod llama_server;
use models::openai;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 9909;

fn main() -> Result<()> {

    let cr = openai::CompletionRequest::builder("Qwen/Qwen2.5-Coder-3B-Instruct-GGUF")
        .message(openai::Message::user("Hello! clang simple sort i32 arr algoritm."))
        .stream(true)
        .build();
    let client = llama_server::Client::new("localhost:9909");
    let ch = chat::Chat::new(client, cr);
    chat::interactive_chat(ch, std::io::stdout(), std::io::stdin())?;

    Ok(())
}
