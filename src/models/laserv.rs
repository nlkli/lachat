use serde::Deserialize;
use super::openai;

#[derive(Clone, Debug)]
pub enum Completion {
    Response(openai::CompletionResponse),
    Chunk(openai::CompletionChunk),
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

    pub fn is_done(&self) -> bool {
        match self {
            Self::Response(r) => r.is_done(),
            Self::Chunk(c) => c.is_done(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<ModelEntry>,
}

impl ModelsResponse {
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn first_model(&self) -> Option<&ModelEntry> {
        self.data.first()
    }

    pub fn first_model_name(&self) -> Option<&str> {
        self.first_model().map(|m| m.id.as_str())
    }

    pub fn name_list(&self) -> Vec<&str> {
        self.data.iter().map(|i| i.id.as_str()).collect()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    // pub in_cache: bool,
    // pub path: String,
    pub status: ModelStatus,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ModelStatus {
    pub value: String,

    #[serde(default)]
    pub args: Vec<String>,
}
