#[derive(Clone, Debug, Default)]
pub struct Args {
    pub host: Option<String>,
    pub port: Option<String>,
    pub model: Option<String>,
    pub prompt: Option<String>,
    pub session: Option<String>,
    pub chat: Option<String>,
    pub system: Option<String>,
    pub interactive: bool,
    pub background: bool,
    pub read_bg: bool,
    pub llama_server_args: Vec<String>,
}

const VERSION: &str = "lachat 0.1.0";
const HELP: &str = r#"
  -h, --help    Print help
  -V, --version Print version"#;

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
                    "chat" => {
                        last.replace('c');
                    }
                    "session" => {
                        last.replace('s');
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
                    "c" => {
                        last.replace('c');
                        continue;
                    }
                    "s" => {
                        last.replace('s');
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
                        'P' => {
                            args.port = i.as_str().parse::<u16>().ok().map(|v| v.to_string());
                        }
                        'm' => {
                            args.model.replace(i);
                        }
                        'p' => {
                            args.prompt.replace(i);
                        }
                        'c' => {
                            args.chat.replace(i);
                        }
                        's' => {
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
            "host" => self.host.as_ref().map_or(default, |a| a.as_str()),
            "port" => self.port.as_ref().map_or(default, |a| a.as_str()),
            "model" => self.model.as_ref().map_or(default, |a| a.as_str()),
            "prompt" => self.prompt.as_ref().map_or(default, |a| a.as_str()),
            "session" => self.session.as_ref().map_or(default, |a| a.as_str()),
            "chat" => self.chat.as_ref().map_or(default, |a| a.as_str()),
            "system" => self.system.as_ref().map_or(default, |a| a.as_str()),
            _ => default,
        }
    }
}
