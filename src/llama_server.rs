use crate::Result;
use crate::models::openai;
use crate::sse::SseReader;
use std::{
    ffi::OsStr,
    io::{BufReader, Cursor},
    process::{Command, Stdio},
};

pub fn spawn<I, S>(args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let _ = Command::new("llama-server")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn process");
}

#[derive(Clone, Debug)]
pub enum Completion {
    Response(openai::CompletionResponse),
    Chunk(openai::CompletionChunk)
}

impl Completion {
    pub fn is_chunk(&self) -> bool {
        matches!(self, Self::Chunk(_))
    }

    pub fn first_content(&self) -> Option<&str> {
        match self {
            Self::Response(r) => r.first_content(),
            Self::Chunk(c) => c.first_content(),
        }
    }
    
    pub fn into_first_content(self) -> Option<String> {
        match self {
            Self::Response(r) => r.into_first_content(),
            Self::Chunk(c) => c.into_first_content(),
        }
    }

    pub fn usage(&self) -> Option<&openai::Usage> {
        match self {
            Self::Response(r) => Some(&r.usage),
            Self::Chunk(_) => None,
        }
    }

    pub fn timings(&self) -> Option<&openai::Timings> {
        match self {
            Self::Response(r) => r.timings.as_ref(),
            Self::Chunk(c) => c.timings.as_ref(),
        }
    }
}

pub struct Client {
    base_url: String,
}

impl Client {
    pub fn new(host: &str) -> Self {
        Self {
            base_url: format!("http://{host}"),
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

    pub fn chat_completions(
        &self,
        cr: &openai::CompletionRequest,
    ) -> Result<Box<dyn Iterator<Item = Completion>>> {
        let req = minreq::post(self.url_endpoint("/chat/completions"))
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::to_vec(cr)?);

        if cr.stream == Some(true) {
            let resp = req.send_lazy()?;
            let reader = std::io::BufReader::new(resp);

            let iter = SseReader::new(reader)
                .filter_map(|res| {
                    let data = res.ok()?;
                    let chunk = serde_json::from_str::<openai::CompletionChunk>(&data).ok()?;
                    Some(Completion::Chunk(chunk))
                });

            Ok(Box::new(iter))
        } else {
            let resp = req.send()?;
            let response =
                serde_json::from_slice::<openai::CompletionResponse>(&resp.into_bytes())?;

            let iter = std::iter::once(Completion::Response(response));
            Ok(Box::new(iter))
        }
    }
}
