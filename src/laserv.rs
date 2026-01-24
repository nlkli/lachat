use crate::Result;
use crate::iter::chat_completions_code;
use crate::models::{laserv, openai};
use crate::sse::SseReader;
use std::io::Write;

#[derive(Clone, Debug)]
pub struct Client {
    base_url: String,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    pub fn wait(self, timeout_ms: u64) -> Result<Self> {
        let mut attempts = 0;
        let delay_ms = 333;

        loop {
            match minreq::get(self.endpoint("/health")).send() {
                Ok(_) => return Ok(self),
                Err(e) => {
                    let msg = e.to_string();
                    if msg.starts_with("Connection refused") {
                        if delay_ms * attempts >= timeout_ms {
                            return Err(format!(
                                "timeout while waiting for llama-server to become available: {}",
                                e
                            )
                            .into());
                        }
                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                        attempts += 1;
                        continue;
                    }
                    return Err(format!("health check failed: {}", e).into());
                }
            }
        }
    }

    pub fn health(&self) -> bool {
        minreq::get(self.endpoint("/health"))
            .send()
            .ok()
            .and_then(|r| r.as_str().ok().map(str::to_owned))
            .map(|s| s.contains("\"ok\""))
            .unwrap_or(false)
    }

    pub fn available_models(&self) -> Result<laserv::ModelsResponse> {
        let resp = minreq::get(self.endpoint("/models"))
            .send()
            .map_err(|e| format!("failed to fetch models list: {}", e))?;

        serde_json::from_str(resp.as_str()?)
            .map_err(|e| format!("failed to parse models response: {}", e).into())
    }

    pub fn chat_completions(
        &self,
        request: &openai::CompletionRequest,
    ) -> Result<Box<dyn Iterator<Item = laserv::Completion>>> {
        let req = minreq::post(self.endpoint("/chat/completions"))
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_vec(request)?);

        if request.stream == Some(true) {
            let resp = req
                .send_lazy()
                .map_err(|e| format!("failed to start streaming response: {}", e))?;
            let reader = std::io::BufReader::new(resp);

            let iter = SseReader::new(reader).filter_map(|event| {
                let data = event.ok()?;
                let chunk = serde_json::from_str::<openai::CompletionChunk>(&data).ok()?;
                Some(laserv::Completion::Chunk(chunk))
            });

            Ok(Box::new(iter))
        } else {
            let resp = req
                .send()
                .map_err(|e| format!("chat completion request failed: {}", e))?;
            let parsed = serde_json::from_slice::<openai::CompletionResponse>(&resp.into_bytes())
                .map_err(|e| format!("failed to parse completion response: {}", e))?;

            Ok(Box::new(std::iter::once(laserv::Completion::Response(
                parsed,
            ))))
        }
    }

    pub fn write_chat_completions<W: Write>(
        &self,
        request: &openai::CompletionRequest,
        mut writer: W,
    ) -> Result<()> {
        for completion in self.chat_completions(request)? {
            let content = completion.first_content().unwrap_or("");
            write!(writer, "{}", content)?;
        }
        Ok(())
    }

    pub fn write_chat_completions_code_only<W: Write>(
        &self,
        request: &openai::CompletionRequest,
        mut writer: W,
    ) -> Result<()> {
        let iter = self.chat_completions(request)?;
        for (chunk, _) in chat_completions_code(iter) {
            write!(writer, "{}", chunk)?;
        }
        Ok(())
    }

    pub fn write_chat_completions_first_code<W: Write>(
        &self,
        request: &openai::CompletionRequest,
        mut writer: W,
    ) -> Result<()> {
        let iter = self.chat_completions(request)?;
        for (chunk, n) in chat_completions_code(iter) {
            if n > 0 {
                break;
            }
            write!(writer, "{}", chunk)?;
        }
        Ok(())
    }
}
