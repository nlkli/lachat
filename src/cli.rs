#[derive(Clone, Debug, Default)]
pub struct Args {
    pub host: Option<String>,
    pub port: Option<String>,
    pub model: Option<String>,
    pub prompt: Vec<String>,
    pub temperature: Option<String>,
    pub session: Option<String>,
    pub ctx_size: Option<String>,
    pub chat: Option<String>,
    pub system: Option<String>,
    pub interactive: bool,
    pub background: bool,
    pub read_bg: bool,
    pub llama_server_args: Vec<String>,
}

const VERSION: &str = "lachat 0.1.0";
const HELP: &str = r#"
lachat — minimal CLI client for llama-server

EXAMPLES:
  lachat -m llama3 -p "Hello"
  lachat --host 127.0.0.1 --port 8080 --interactive
  lachat -- -ngl 35 -ctx-size 4096

USAGE:
  lachat [OPTIONS] [-- <LLAMA_SERVER_ARGS>...]

OPTIONS:

  --host <HOST>, -h <HOST>
          Server host address (IPv4)

  --port <PORT>, -P <PORT>
          Server port

  --model <MODEL>, -m <MODEL>
          Model name to use (fuzzy matching)

  --prompt <TEXT>, -p <TEXT>
          Prompt to send to the model

  --system <TEXT>, -s <TEXT>
          System prompt (system message)

  --chat <ID>, -c <ID>
          Chat ID or chat name

  --session <ID>, -S <ID>
          Session identifier

  --temperature <VALUE>, -t <VALUE>
          Sampling temperature (float)

  --interactive, -i
          Run in interactive mode

  --background, -b
          Run in background mode

  --read-bg, -r
          Read output from background session

  -h, --help
          Print this help message and exit

  -V, --version
          Print version information and exit

PASSTHROUGH ARGUMENTS:
  -- <ARGS>...
          All arguments after '--' are passed directly to the llama server

"#;

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
                    "host" => {
                        last.replace('H');
                    }
                    "port" => {
                        last.replace('P');
                    }
                    "model" => {
                        last.replace('m');
                    }
                    "prompt" => {
                        last.replace('p');
                    }
                    "temperature" => {
                        last.replace('t');
                    }
                    "chat" => {
                        last.replace('c');
                    }
                    "ctx-size" => {
                        last.replace('C');
                    }
                    "system" => {
                        last.replace('s');
                    }
                    "session" => {
                        last.replace('S');
                    }
                    "interactive" => args.interactive = true,
                    "background" => args.background = true,
                    "read-bg" => args.read_bg = true,
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
                    "h" => {
                        last.replace('H');
                        continue;
                    }
                    "P" => {
                        last.replace('P');
                        continue;
                    }
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
                    "c" => {
                        last.replace('c');
                        continue;
                    }
                    "C" => {
                        last.replace('C');
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
                        'b' => args.background = true,
                        'r' => args.read_bg = true,
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
                        'h' => {
                            args.host = i
                                .as_str()
                                .parse::<std::net::SocketAddrV4>()
                                .ok()
                                .map(|h| h.to_string());
                        }
                        'P' => args.port = i.parse::<u16>().ok().map(|v| v.to_string()),
                        'm' => {
                            args.model.replace(i);
                        }
                        'p' => {
                            args.prompt.push(i);
                        }
                        't' => args.temperature = i.parse::<f32>().ok().map(|v| v.to_string()),
                        'c' => {
                            args.chat.replace(i);
                        }
                        'C' => args.ctx_size = i.parse::<usize>().ok().map(|v| v.to_string()),

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
                    // args.theme.replace(i);
                }
            }
        }
        args
    }

    pub fn get_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        match key {
            "h" => self.host.as_ref().map_or(default, |a| a.as_str()),
            "P" => self.port.as_ref().map_or(default, |a| a.as_str()),
            "m" => self.model.as_ref().map_or(default, |a| a.as_str()),
            "t" => self.temperature.as_ref().map_or(default, |a| a.as_str()),
            "S" => self.session.as_ref().map_or(default, |a| a.as_str()),
            "c" => self.chat.as_ref().map_or(default, |a| a.as_str()),
            "s" => self.system.as_ref().map_or(default, |a| a.as_str()),
            _ => default,
        }
    }
}
