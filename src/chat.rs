use crate::Result;
use crate::laserv::Client;
use crate::models::{
    laserv::Completion,
    openai::{CompletionRequest, Message},
};
use std::io::{BufRead, BufReader, Read, Write};

pub struct Chat<'a> {
    client: &'a Client,
    cr: CompletionRequest,
}

impl<'a> Chat<'a> {
    pub fn new(client: &'a Client, cr: CompletionRequest) -> Self {
        Self { client, cr }
    }

    pub fn add_message(&mut self, msg: Message) {
        self.cr.messages.push(msg);
    }

    pub fn clear(&mut self) {
        let system = self.cr.messages.iter().find(|m| m.is_system()).cloned();
        self.cr.messages.clear();
        if let Some(system) = system {
            self.cr.messages.push(system);
        }
    }

    pub fn send(&mut self, msg: Message) -> Result<Box<dyn Iterator<Item = Completion>>> {
        self.add_message(msg);
        self.client.chat_completions(&self.cr)
    }

    pub fn messages(&self) -> &[Message] {
        self.cr.messages.as_slice()
    }
}

pub fn interactive_chat<W: Write, R: Read>(chat: &mut Chat, mut w: W, r: R) -> Result<()> {
    let mut reader = BufReader::new(r);

    const SHORT_HELP: &str =
        "Type message, end with '.' on a new line. /help for commands, /exit to quit.";
    const HELP: &str = r#"Commands:
  /clear  Clear context
  /exit   Exit the chat
  /quit   Exit the chat

Multi-line input is supported. Finish your message with '.' on its own line"#;

    writeln!(w, "{SHORT_HELP}")?;

    let input_prompt = |w: &mut W| -> Result<()> {
        write!(w, "\n> ")?;
        w.flush()?;
        Ok(())
    };

    loop {
        input_prompt(&mut w)?;

        let mut input = String::new();

        loop {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                return Ok(());
            }

            let trimmed = line.trim();

            match trimmed {
                "/exit" | "/quit" => return Ok(()),
                "/help" if input.is_empty() => {
                    writeln!(w, "{HELP}")?;
                    input_prompt(&mut w)?;
                    continue;
                }
                "/clear" if input.is_empty() => {
                    chat.clear();
                    writeln!(w, "context cleared")?;
                    input_prompt(&mut w)?;
                    continue;
                }
                "." => break,
                _ => input.push_str(&line),
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let completions = chat.send(Message::user(input))?;

        write!(w, "\n< ")?;
        w.flush()?;

        let mut full_response = String::new();

        for completion in completions {
            if let Some(content) = completion.first_content() {
                write!(w, "{content}")?;
                w.flush()?;
                full_response.push_str(content);
            }
        }

        if !full_response.is_empty() {
            chat.add_message(Message::assistant(&full_response));
        }

        writeln!(w)?;
    }
}
