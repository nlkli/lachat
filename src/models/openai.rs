use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Message role in a chat conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

impl Role {
    /// Returns the role as a string slice.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    /// Creates a system message.
    #[inline]
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }

    /// Creates a user message.
    #[inline]
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    /// Creates an assistant message.
    #[inline]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }

    /// Creates a tool message.
    #[inline]
    pub fn tool(content: impl Into<String>) -> Self {
        Self::new(Role::Tool, content)
    }

    /// Creates a message with the specified role and content.
    #[inline]
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// Checks if this is a user message.
    #[inline]
    pub const fn is_user(&self) -> bool {
        matches!(self.role, Role::User)
    }

    /// Checks if this is an assistant message.
    #[inline]
    pub const fn is_assistant(&self) -> bool {
        matches!(self.role, Role::Assistant)
    }

    /// Checks if this is a system message.
    #[inline]
    pub const fn is_system(&self) -> bool {
        matches!(self.role, Role::System)
    }

    /// Checks if this is a tool message.
    #[inline]
    pub const fn is_tool(&self) -> bool {
        matches!(self.role, Role::Tool)
    }
}

/// Structured output format for model responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Free-form JSON object with optional schema validation.
    JsonObject {
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<Value>,
    },
    /// Strict JSON schema enforcement.
    JsonSchema { schema: Value },
}

/// OpenAI-compatible chat completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    // Sampling parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    // Advanced features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_template_kwargs: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_forced_open: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
}

/// Builder for constructing completion requests.
#[derive(Debug, Clone)]
pub struct CompletionRequestBuilder {
    request: CompletionRequest,
}

impl CompletionRequestBuilder {
    /// Creates a new builder for the specified model.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            request: CompletionRequest {
                model: model.into(),
                messages: Vec::new(),
                stream: None,
                temperature: None,
                top_p: None,
                top_k: None,
                max_tokens: None,
                stop: None,
                response_format: None,
                chat_template_kwargs: None,
                reasoning_format: None,
                thinking_forced_open: None,
                parse_tool_calls: None,
                parallel_tool_calls: None,
            },
        }
    }

    /// Adds a message to the conversation.
    pub fn message(mut self, message: Message) -> Self {
        self.request.messages.push(message);
        self
    }

    /// Adds a message with the specified role and content.
    pub fn add(mut self, role: Role, content: impl Into<String>) -> Self {
        self.request.messages.push(Message::new(role, content));
        self
    }

    /// Adds multiple messages at once.
    pub fn messages(mut self, messages: impl IntoIterator<Item = Message>) -> Self {
        self.request.messages.extend(messages);
        self
    }

    /// Enables or disables streaming.
    pub fn stream(mut self, enabled: bool) -> Self {
        self.request.stream = Some(enabled);
        self
    }

    /// Sets the sampling temperature (0.0 to 2.0).
    pub fn temperature(mut self, value: f32) -> Self {
        self.request.temperature = Some(value);
        self
    }

    /// Sets nucleus sampling probability (0.0 to 1.0).
    pub fn top_p(mut self, value: f32) -> Self {
        self.request.top_p = Some(value);
        self
    }

    /// Sets top-k sampling parameter.
    pub fn top_k(mut self, value: u32) -> Self {
        self.request.top_k = Some(value);
        self
    }

    /// Sets maximum tokens to generate.
    pub fn max_tokens(mut self, value: u32) -> Self {
        self.request.max_tokens = Some(value);
        self
    }

    /// Adds a stop sequence.
    pub fn stop(mut self, sequence: impl Into<String>) -> Self {
        self.request
            .stop
            .get_or_insert_with(Vec::new)
            .push(sequence.into());
        self
    }

    /// Sets multiple stop sequences.
    pub fn stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.request.stop = Some(sequences);
        self
    }

    /// Sets the response format for structured outputs.
    pub fn response_format(mut self, format: ResponseFormat) -> Self {
        self.request.response_format = Some(format);
        self
    }

    /// Sets chat template parameters (Jinja).
    pub fn chat_template_kwargs(mut self, kwargs: Value) -> Self {
        self.request.chat_template_kwargs = Some(kwargs);
        self
    }

    /// Sets the reasoning format.
    pub fn reasoning_format(mut self, format: impl Into<String>) -> Self {
        self.request.reasoning_format = Some(format.into());
        self
    }

    /// Forces thinking tags to be opened.
    pub fn thinking_forced_open(mut self, enabled: bool) -> Self {
        self.request.thinking_forced_open = Some(enabled);
        self
    }

    /// Enables tool call parsing.
    pub fn parse_tool_calls(mut self, enabled: bool) -> Self {
        self.request.parse_tool_calls = Some(enabled);
        self
    }

    /// Enables parallel tool calls.
    pub fn parallel_tool_calls(mut self, enabled: bool) -> Self {
        self.request.parallel_tool_calls = Some(enabled);
        self
    }

    /// Builds the completion request.
    pub fn build(self) -> CompletionRequest {
        self.request
    }
}

/// Token usage statistics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Performance timing information.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

impl Timings {
    /// Returns total processing time in milliseconds.
    #[inline]
    pub const fn total_ms(&self) -> f64 {
        self.prompt_ms + self.predicted_ms
    }
}

/// A single choice in a completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub index: usize,
    pub message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Complete non-streaming chat completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timings: Option<Timings>,
}

impl CompletionResponse {
    /// Returns the content of the first message.
    #[inline]
    pub fn first_content(&self) -> Option<&str> {
        self.choices.first().map(|c| c.message.content.as_str())
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.choices
            .get(0)
            .map(|c| c.finish_reason.is_some())
            .unwrap_or(false)
    }
}

/// Delta content in a streaming chunk.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Role>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// A single choice in a streaming chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChoice {
    pub index: usize,
    pub delta: Delta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Streaming chat completion chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    pub choices: Vec<StreamChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timings: Option<Timings>,
}

impl CompletionChunk {
    /// Returns the content delta from the first choice.
    #[inline]
    pub fn first_content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|c| c.delta.content.as_deref())
    }

    #[inline]
    pub fn is_done(&self) -> bool {
        self.choices
            .get(0)
            .map(|c| c.finish_reason.is_some())
            .unwrap_or(false)
    }
}

impl CompletionRequest {
    /// Creates a new builder for the specified model.
    #[inline]
    pub fn builder(model: impl Into<String>) -> CompletionRequestBuilder {
        CompletionRequestBuilder::new(model)
    }
}
