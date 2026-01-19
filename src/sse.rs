use std::io::{self, BufRead};

pub struct SseReader<R> {
    reader: R,
    buffer: String,
}

impl<R: BufRead> SseReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: String::with_capacity(256),
        }
    }
}

impl<R: BufRead> Iterator for SseReader<R> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buffer.clear();

            match self.reader.read_line(&mut self.buffer) {
                Ok(0) => return None, // EOF
                Err(e) => return Some(Err(e)),
                Ok(_) => {
                    let line = self.buffer.trim();

                    if line.is_empty() || line.starts_with(':') {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data:") {
                        let trimmed = data.trim();
                        if trimmed == "[DONE]" {
                            return None;
                        }
                        return Some(Ok(trimmed.to_string()));
                    }
                }
            }
        }
    }
}
