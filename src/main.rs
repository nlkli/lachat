// let _ = Command::new("your_command")
//         .args(["arg1", "arg2"])
//         .stdin(Stdio::null())  // Отсоединяем stdin/stdout/stderr
//         .stdout(Stdio::null())
//         .stderr(Stdio::null())
//         .spawn()
//         .expect("Failed to spawn process");

use std::io::Write;
mod models;


type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 9909;

struct Client {
    base_url: String,
}

impl Client {
    fn new(base_url: String) -> Self {
        Self { base_url }
    }

    fn health(&self) -> bool {
        if let Ok(res) = minreq::get(format!("{}/health", self.base_url)).send() {
            if res.as_str().unwrap_or("").contains("\"ok\"") {
                return true;
            }
        }
        false
    }

    fn chat_completions(&self) -> () {

    }
}
    
fn main() -> Result<()> {
    let url = "http://localhost:9909/chat/completions";

    let content = r#"
    Hello!
    "#;
    let body = serde_json::json!({
        "model": "Qwen/Qwen2.5-Coder-3B-Instruct-GGUF",
        "messages": [{"role": "user", "content": content}],
        "stream": true,
        "temperature": 0.7
    });

    let response = minreq::Request::new(minreq::Method::Post, url)
        .with_header("Content-Type", "application/json")
        .with_body(body.to_string())
        .send_lazy()?;

    let mut buffer = String::new();
    let mut stdout = std::io::stdout();

    for result in response {
        let (byte, _remaining) = result?;
        if byte == b'\n' {
            let line = buffer.trim();

            if line.starts_with("data: ") {
                let data = &line[6..];

                if data == "[DONE]" {
                    println!("\n[DONE]");
                    break;
                }

                println!("{data}");

                // if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                //     if let Some(content) = json
                //         .pointer("/choices/0/delta/content")
                //         .and_then(|v| v.as_str())
                //     {
                //         print!("{}", content);
                //         stdout.flush()?;
                //     }
                // }
            }
            buffer.clear();
        } else if byte != b'\r' {
            buffer.push(byte as char);
        }
    }

    Ok(())
}
