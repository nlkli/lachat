#[derive(Clone, Debug, Default)]
pub struct Args {
    pub model: Option<String>,
    pub prompt: Vec<String>,
    pub temperature: Option<String>,
    pub max_tokens: Option<String>,
    pub chat: Option<String>,
    pub system: Option<String>,
    pub session: Option<String>,
    pub interactive: bool,
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
          Chat ID or chat name.
          Enables persistent chat history stored in the session.
  -S, --session <PATH>
          Path to the session directory.
          Defaults to $LACHAT_SESSION or the built-in default path /tmp/lachat
  -t, --temperature <VALUE>
          Sampling temperature (float).
  -x, --max-tokens <VALUE>
          Sets maximum tokens to generate.
  -i, --interactive
          Start an interactive chat session.
  -k, --kill
          Kill the currently running llama-server.
  -h, --help
          Print this help message and exit.
  -V, --version
          Print version information and exit.
PASSTHROUGH ARGUMENTS:
  -- <ARGS>...
          All arguments after '--' are passed directly to llama-server.
PROMPT SOURCES (in order):
  1. stdin (if not empty)
  2. --prompt arguments
EXAMPLES:
  lachat hello!
  cat main.rs | lachat -m qwen -p "code refactor" -c mychat -- --port 5050"#;

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
                    "kill" => args.kill = true,
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
                        'i' => args.interactive = true,
                        'k' => args.kill = true,
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

    pub fn extract_llama_addr<'a>(&'a self) -> (Option<&'a str>, Option<u16>) {
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
}
