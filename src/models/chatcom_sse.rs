use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,

    #[serde(default)]
    pub system_fingerprint: Option<String>,

    pub choices: Vec<Choice>,

    #[serde(default)]
    pub timings: Option<Timings>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: usize,
    pub delta: Delta,

    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Delta {
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Timings {
    pub cache_n: u32,
    pub prompt_n: u32,
    pub prompt_ms: f64,
    pub prompt_per_token_ms: f64,
    pub prompt_per_second: f64,
    pub predicted_n: u32,
    pub predicted_ms: f64,
    pub predicted_per_token_ms: f64,
    pub predicted_per_second: f64,
}

impl ChatCompletionChunk {
    pub fn token(&self) -> Option<&str> {
        self.choices.get(0).and_then(|c| c.delta.content.as_deref())
    }

    pub fn is_finished(&self) -> bool {
        self.choices.get(0).and_then(|c| c.finish_reason.as_deref()) == Some("stop")
    }
}
