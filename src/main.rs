mod chat;
mod cli;
mod iter;
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
const DEFAULT_SESSION_PATH: &str = "/tmp/lachat";

fn launch_llama_server(args: &[String]) -> Result<u32> {
    let mut final_args = args.to_vec();
    if !final_args.iter().any(|a| a == "--sleep-idle-seconds") {
        final_args.extend(["--sleep-idle-seconds".into(), "5400".into()]);
    }
    process::spawn_detached("llama-server", final_args)
        .map_err(|e| format!("failed to launch llama-server: {}", e).into())
}

#[inline(always)]
fn wait_for_server(host: &str, port: u16) -> Result<()> {
    laserv::Client::new(format!("http://{}:{}", host, port))
        .wait(15_000)
        .map(|_| ())
        .map_err(|e| format!("llama-server did not become ready: {}", e).into())
}

fn main() -> Result<()> {
    let stdin_input = utils::read_stdin().map_err(|e| format!("failed to read stdin: {}", e))?;
    let args = cli::Args::parse();

    let session = session::Session::new(args.get_or(
        "S",
        &std::env::var("LACHAT_SESSION").unwrap_or(DEFAULT_SESSION_PATH.into()),
    ))
    .map_err(|e| format!("failed to initialize session: {}", e))?;

    let mut state = match session.read_state()? {
        Some(state) => state,
        None => {
            let (host, port) = args.extract_serv_addr_or_default(DEFAULT_HOST, DEFAULT_PORT);

            let pid = launch_llama_server(&args.llama_server_args)?;
            wait_for_server(&host, port)?;

            let state = session::State::new(pid, host.into(), port);
            session.write_state(&state)?;
            state
        }
    };

    if !args.llama_server_args.is_empty() {
        let (host, port) = args.extract_serv_addr_or_default(&state.host, state.port);

        if host != state.host || port != state.port {
            if state.pid != 0 {
                process::kill_pid(state.pid)
                    .map_err(|e| format!("failed to stop previous llama-server: {}", e))?;
                state.pid = 0;
                session.write_state(&state)?;
            }

            let extended_args = utils::extend_args(
                &args.llama_server_args,
                &["--host", &host, "--port", &port.to_string()],
            );
            let pid = launch_llama_server(&extended_args)?;
            wait_for_server(&host, port)?;

            state = session::State::new(pid, host.into(), port);
            session.write_state(&state)?;
        }
    }

    if state.pid == 0 {
        let extended_args = utils::extend_args(
            &args.llama_server_args,
            &["--host", &state.host, "--port", &state.port.to_string()],
        );
        let pid = launch_llama_server(&extended_args)?;
        wait_for_server(&state.host, state.port)?;

        state.pid = pid;
        session.write_state(&state)?;
    }

    let client = laserv::Client::new(format!("http://{}:{}", state.host, state.port));
    if !client.health() {
        return Err("llama-server is not responding".into());
    }

    let available_models = client
        .available_models()
        .map_err(|e| format!("failed to fetch available models: {}", e))?;
    let default_model = available_models
        .first_model_name()
        .ok_or("no models available on llama-server")?;

    let model_name = args
        .model
        .as_ref()
        .and_then(|m| utils::fuzzy_search(&available_models.name_list(), m))
        .unwrap_or(default_model);

    let mut request_builder = openai::CompletionRequest::builder(model_name);

    let mut messages = match args.chat.as_ref() {
        Some(chat_id) => session
            .read_chat(chat_id)?
            .unwrap_or_else(session::Chat::new),
        None => session::Chat::new(),
    };

    if let Some(temp) = args
        .temperature
        .as_ref()
        .and_then(|t| t.parse::<f32>().ok())
        .map(|t| t.abs().clamp(0.0, 1.0))
    {
        request_builder = request_builder.temperature(temp);
    }

    if let Some(max_tokens) = args.max_tokens.as_ref().and_then(|t| t.parse::<u32>().ok()) {
        request_builder = request_builder.max_tokens(max_tokens);
    }

    if let Some(system_prompt) = args.system.as_ref() {
        let content = if utils::is_existing_file(system_prompt) {
            std::fs::read_to_string(system_prompt)
                .map_err(|e| format!("failed to read system file: {}", e))?
        } else {
            system_prompt.clone()
        };
        messages.push(openai::Message::system(content));
    }

    let has_user_input = !stdin_input.is_empty() || !args.prompt.is_empty();

    if !stdin_input.is_empty() {
        messages.push(openai::Message::user(stdin_input));
    }

    for prompt in &args.prompt {
        let content = if utils::is_existing_file(prompt) {
            std::fs::read_to_string(prompt)
                .map_err(|e| format!("failed to read prompt file: {}", e))?
        } else {
            prompt.clone()
        };
        messages.push(openai::Message::user(content));
    }

    let completion_request = request_builder.messages(messages).stream(true).build();

    if has_user_input {
        let mut response_buffer = Vec::new();
        let writer = utils::DualWriter {
            w1: io::stdout(),
            w2: &mut response_buffer,
        };

        if args.first_code {
            client.write_chat_completions_first_code(&completion_request, writer)?;
        } else if args.code_only {
            client.write_chat_completions_code_only(&completion_request, writer)?;
        } else {
            client.write_chat_completions(&completion_request, writer)?;
        }

        println!();

        let response_text = String::from_utf8(response_buffer)?;
        session.write_chat(
            "",
            &[
                completion_request.messages.last().unwrap().clone(),
                openai::Message::assistant(&response_text),
            ],
        )?;

        if let Some(chat_id) = args.chat.as_ref() {
            let mut updated_chat = Vec::with_capacity(completion_request.messages.len() + 1);
            updated_chat.extend_from_slice(&completion_request.messages);
            updated_chat.push(openai::Message::assistant(response_text));
            session.write_chat(chat_id, &updated_chat)?;
        }
    } else if args.last {
        if let Some(chat_id) = args.chat.as_ref() {
            if let Some(chat) = session.read_chat(chat_id)? {
                if let Some(m) = chat.last() {
                    println!("{}", m.content);
                }
            }
        } else {
            let chat = session.read_chat("")?;
            if let Some(c) = chat {
                if let Some(m) = c.last() {
                    println!("{}", m.content);
                }
            }
        }
    } else if let Some(chat_id) = args.chat.as_ref() {
        if let Some(chat) = session.read_chat(chat_id)? {
            println!("{}", serde_json::to_string_pretty(&chat)?);
        }
    }

    if args.interactive {
        let mut interactive_chat = chat::Chat::new(&client, completion_request);
        chat::interactive_chat(&mut interactive_chat, io::stdout(), io::stdin())?;

        if let Some(chat_id) = args.chat.as_ref() {
            session.write_chat(chat_id, interactive_chat.messages())?;
        }
    }

    if args.clear {
        if let Some(chat_id) = args.chat.as_ref() {
            session.clear_chat(chat_id)?;
        } else {
            session.clear_all_chat()?;
        }
    }

    if args.chat_list {
        for c in session.chat_list()? {
            println!("{}", c);
        }
    }

    if args.kill && state.pid != 0 {
        process::kill_pid(state.pid).map_err(|e| format!("failed to stop llama-server: {}", e))?;
        state.pid = 0;
        session.write_state(&state)?;
    } else {
        if args.open {
            utils::open_url(&format!("http://{}:{}", state.host, state.port))?
        }
    }

    Ok(())
}
