use crate::Result;
use crate::models::{laserv, openai};
use crate::sse::SseReader;
use std::io::Write;

#[derive(Clone, Debug)]
pub struct Client {
    base_url: String,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url
        }
    }

    fn url_endpoint(&self, endpoint: &str) -> String {
        format!("{}{endpoint}", self.base_url)
    }

    pub fn health(&self) -> bool {
        if let Ok(res) = minreq::get(self.url_endpoint("/health")).send() {
            if res.as_str().unwrap_or("").contains("\"ok\"") {
                return true;
            }
        }
        false
    }

    pub fn available_models(&self) -> Result<laserv::ModelsResponse> {
        let resp = minreq::get(self.url_endpoint("/models")).send()?;
        let models = serde_json::from_str(resp.as_str()?)?;
        Ok(models)
    }

    pub fn chat_completions(
        &self,
        cr: &openai::CompletionRequest,
    ) -> Result<Box<dyn Iterator<Item = laserv::Completion>>> {
        let req = minreq::post(self.url_endpoint("/chat/completions"))
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_vec(cr)?);

        if cr.stream == Some(true) {
            let resp = req.send_lazy()?;
            let reader = std::io::BufReader::new(resp);

            let iter = SseReader::new(reader).filter_map(|res| {
                let data = res.ok()?;
                let chunk = serde_json::from_str::<openai::CompletionChunk>(&data).ok()?;
                Some(laserv::Completion::Chunk(chunk))
            });

            Ok(Box::new(iter))
        } else {
            let resp = req.send()?;
            let response =
                serde_json::from_slice::<openai::CompletionResponse>(&resp.into_bytes())?;

            let iter = std::iter::once(laserv::Completion::Response(response));
            Ok(Box::new(iter))
        }
    }

    pub fn write_chat_completions<W: Write>(
        &self,
        cr: &openai::CompletionRequest,
        mut w: W,
    ) -> Result<()> {
        for c in self.chat_completions(cr)? {
            write!(&mut w, "{}", c.first_content().unwrap_or(""))?;
        }
        Ok(())
    }
}
