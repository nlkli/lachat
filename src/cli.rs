#[derive(Clone, Debug, Default)]
pub struct Args {
    pub model: Option<String>,
    pub prompt: Vec<String>,
    pub temperature: Option<String>,
    pub max_tokens: Option<String>,
    pub chat: Option<String>,
    pub system: Option<String>,
    pub session: Option<String>,
    pub code_only: bool,
    pub first_code: bool,
    pub interactive: bool,
    pub last: bool,
    pub chat_list: bool,
    pub clear: bool,
    pub open: bool,
    pub kill: bool,
    pub llama_server_args: Vec<String>,
}

const VERSION: &str = "lachat 0.1.0";
const HELP: &str = r#"lachat — minimal CLI client for llama-server
USAGE:
  lachat [OPTIONS] [PROMPT...] -- [LLAMA_SERVER_ARGS...]
OPTIONS:
  -p, --prompt <TEXT|PATH>
        Prompt to send to the model. Can be specified multiple times.
        If a file path is provided, its contents are used.
  -m, --model <MODEL>
        Model name to use. If not specified, the first available model is used.
        Fuzzy matching is applied when resolving the model name.
  -s, --system <TEXT|PATH>
        System prompt (system message).
  -c, --chat <ID>
        Chat identifier. Enables persistent chat history stored in the session.
        If no prompt is provided, the chat history is printed as JSON.
  -S, --session <PATH>
        Path to the session directory. Stores server state and chat history.
        Defaults to $LACHAT_SESSION or /tmp/lachat.
  -t, --temp --temperature <VALUE>
        Sampling temperature (float).
  -x, --max-tokens <VALUE>
        Maximum number of tokens to generate.
  -e, --code-only
        Output only code blocks from the model response.
  -E, --first-code
        Output only the first code block from the model response.
  -l, --last
        Print the last assistant message.
        Uses the specified --chat or the most recent session.
  -i, --interactive
        Start an interactive chat session.
  -L, --list --chat-list
        List stored chats
  -C, --clear
        Clear chats (all or specified with --chat)
  -o, --open
        Open the llama-server web interface in the default browser.
  -k, --kill
        Kill the currently running llama-server.
  -h, --help
        Print this help message and exit.
  -V, --version
        Print version information and exit.
PASSTHROUGH ARGUMENTS:
  -- <ARGS>...
        All arguments after '--' are passed directly to llama-server.
SERVER BEHAVIOR:
  If no server is running for the session, llama-server is started automatically.
  If server arguments change (host/port), the old server is terminated and restarted.
PROMPT SOURCES (in order):
  stdin (if not empty)
  --prompt arguments
EXAMPLES:
  lachat hello
  lachat -m qwen -c mychat -p "main.rs" -p "refactor this code"
  cat main.go | lachat -s system.txt
  lachat -- --host 127.0.0.1 --port 5050"#;

impl Args {
    pub fn parse() -> Self {
        let mut args = Self::default();
        let input = std::env::args();
        let mut last = None;
        let mut passthrough = false;
        for i in input.skip(1) {
            if i == "--" {
                passthrough = !passthrough;
                continue;
            }
            if passthrough {
                args.llama_server_args.push(i);
                continue;
            }
            if i.starts_with("--") {
                let key = i.trim_start_matches("--");
                match key {
                    "model" => {
                        last.replace('m');
                    }
                    "prompt" => {
                        last.replace('p');
                    }
                    "temperature" => {
                        last.replace('t');
                    }
                    "temp" => {
                        last.replace('t');
                    }
                    "max-tokens" => {
                        last.replace('x');
                    }
                    "chat" => {
                        last.replace('c');
                    }
                    "system" => {
                        last.replace('s');
                    }
                    "session" => {
                        last.replace('S');
                    }
                    "interactive" => args.interactive = true,
                    "chat-list" => args.chat_list = true,
                    "list" => args.chat_list = true,
                    "clear" => args.clear = true,
                    "kill" => args.kill = true,
                    "open" => args.open = true,
                    "code-only" => args.code_only = true,
                    "first-only" => args.first_code = true,
                    "last" => args.last = true,
                    "help" => {
                        println!("{}", HELP);
                        std::process::exit(0);
                    }
                    "version" => {
                        println!("{}", VERSION);
                        std::process::exit(0);
                    }
                    _ => (),
                }
            } else if i.starts_with("-") {
                let trimmed = i.trim_start_matches("-");
                match trimmed {
                    "m" => {
                        last.replace('m');
                        continue;
                    }
                    "p" => {
                        last.replace('p');
                        continue;
                    }
                    "t" => {
                        last.replace('t');
                        continue;
                    }
                    "x" => {
                        last.replace('x');
                        continue;
                    }
                    "c" => {
                        last.replace('c');
                        continue;
                    }
                    "s" => {
                        last.replace('s');
                        continue;
                    }
                    "S" => {
                        last.replace('S');
                        continue;
                    }
                    _ => (),
                }
                let chars = trimmed.chars();
                for c in chars {
                    match c {
                        'e' => args.code_only = true,
                        'E' => args.first_code = true,
                        'l' => args.last = true,
                        'L' => args.chat_list = true,
                        'C' => args.clear = true,
                        'i' => args.interactive = true,
                        'k' => args.kill = true,
                        'o' => args.open = true,
                        'h' => {
                            println!("{}", HELP);
                            std::process::exit(0);
                        }
                        'V' => {
                            println!("{}", VERSION);
                            std::process::exit(0);
                        }
                        _ => (),
                    }
                }
            } else {
                if let Some(c) = last {
                    match c {
                        'm' => {
                            args.model.replace(i);
                        }
                        'p' => {
                            args.prompt.push(i);
                        }
                        't' => args.temperature = i.parse::<f32>().ok().map(|v| v.to_string()),
                        'x' => args.max_tokens = i.parse::<u32>().ok().map(|v| v.to_string()),
                        'c' => {
                            args.chat.replace(i);
                        }
                        's' => {
                            args.system.replace(i);
                        }
                        'S' => {
                            args.session.replace(i);
                        }
                        _ => (),
                    }
                    last = None;
                } else {
                    args.prompt.push(i);
                }
            }
        }
        args
    }

    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        match key {
            "m" => self.model.as_ref().map_or(default, |a| a.as_str()),
            "t" => self.temperature.as_ref().map_or(default, |a| a.as_str()),
            "S" => self.session.as_ref().map_or(default, |a| a.as_str()),
            "c" => self.chat.as_ref().map_or(default, |a| a.as_str()),
            "s" => self.system.as_ref().map_or(default, |a| a.as_str()),
            _ => default,
        }
    }

    pub fn extract_serv_addr<'a>(&'a self) -> (Option<&'a str>, Option<u16>) {
        let host = self
            .llama_server_args
            .iter()
            .position(|a| a == "--host")
            .and_then(|i| self.llama_server_args.get(i + 1).map(String::as_str));
        let port = self
            .llama_server_args
            .iter()
            .position(|a| a == "--port")
            .and_then(|i| self.llama_server_args.get(i + 1))
            .and_then(|v| v.parse::<u16>().ok());

        (host, port)
    }

    pub fn extract_serv_addr_or_default<'a>(&'a self, host: &'a str, port: u16) -> (&'a str, u16) {
        let (h, p) = self.extract_serv_addr();
        (h.unwrap_or(host), p.unwrap_or(port))
    }
}
