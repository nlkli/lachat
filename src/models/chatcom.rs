use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ChatCompletionsRequest {
    /// Model identifier.
    /// Can be a GGUF model name, preset name, or router model ID.
    pub model: String,

    /// Conversation history in ChatML format.
    /// Order matters: earlier messages provide context for later ones.
    pub messages: Vec<ChatMessage>,

    /// Enable streaming response using Server-Sent Events (SSE).
    /// When true, the server sends partial tokens incrementally.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Sampling temperature.
    /// Higher values make output more random, lower values more deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Nucleus sampling probability (top-p).
    /// Limits token selection to the smallest set whose cumulative probability ≥ top_p.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Top-k sampling.
    /// Limits token selection to the k most likely tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    /// Maximum number of tokens to generate.
    /// In llama.cpp this maps to `n_predict`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    /// Stop sequences.
    /// Generation stops when any of these sequences is encountered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// Response formatting instructions (plain JSON or schema-constrained JSON).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Additional parameters passed to the chat template (Jinja).
    /// Used to control model-specific template behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_template_kwargs: Option<serde_json::Value>,

    /// Reasoning output format.
    /// If set to `none`, raw text is returned without reasoning parsing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_format: Option<String>,

    /// Force reasoning output to always be included.
    /// Only effective for models that support explicit reasoning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_forced_open: Option<bool>,

    /// Whether to parse tool calls from the generated output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_tool_calls: Option<bool>,

    /// Allow multiple or parallel tool calls in a single response.
    /// Supported only by some models and templates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
}


#[derive(Debug, Serialize)]
pub struct ChatMessage {
    /// Role of the message author.
    /// Determines how the message is interpreted by the chat template.
    pub role: ChatRole,

    /// Message content.
    /// For multimodal models this may be structured,
    /// but a plain string is the most common and stable form.
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    /// System-level instructions that guide model behavior.
    System,

    /// User input message.
    User,

    /// Assistant response message.
    Assistant,

    /// Tool or function call result.
    Tool,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ResponseFormat {
    /// Request a JSON object as output.
    /// The model is instructed to return valid JSON.
    #[serde(rename = "json_object")]
    JsonObject {
        /// Optional JSON schema to constrain the output structure.
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<serde_json::Value>,
    },

    /// Request output that strictly follows a JSON Schema.
    #[serde(rename = "json_schema")]
    JsonSchema {
        /// JSON Schema definition describing the expected output.
        schema: serde_json::Value,
    },
}

pub struct ChatCompletionsBuilder {
    model: String,
    messages: Vec<ChatMessage>,

    stream: Option<bool>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    top_k: Option<u32>,
    max_tokens: Option<i32>,
    stop: Option<Vec<String>>,
    response_format: Option<ResponseFormat>,
    chat_template_kwargs: Option<Value>,
    reasoning_format: Option<String>,
    thinking_forced_open: Option<bool>,
    parse_tool_calls: Option<bool>,
    parallel_tool_calls: Option<bool>,
}

impl ChatCompletionsBuilder {
    /// Create a new chat completion request builder.
    ///
    /// `model` and `messages` are required.
    pub fn new(
        model: impl Into<String>,
        messages: Vec<ChatMessage>,
    ) -> Self {
        Self {
            model: model.into(),
            messages,

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
        }
    }

    /// Build the final ChatCompletionsRequest.
    pub fn build(self) -> ChatCompletionsRequest {
        ChatCompletionsRequest {
            model: self.model,
            messages: self.messages,
            stream: self.stream,
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
            max_tokens: self.max_tokens,
            stop: self.stop,
            response_format: self.response_format,
            chat_template_kwargs: self.chat_template_kwargs,
            reasoning_format: self.reasoning_format,
            thinking_forced_open: self.thinking_forced_open,
            parse_tool_calls: self.parse_tool_calls,
            parallel_tool_calls: self.parallel_tool_calls,
        }
    }

    /// Enable or disable streaming (SSE).
    pub fn stream(mut self, value: bool) -> Self {
        self.stream = Some(value);
        self
    }

    /// Set sampling temperature.
    pub fn temperature(mut self, value: f32) -> Self {
        self.temperature = Some(value);
        self
    }

    /// Set nucleus sampling probability (top-p).
    pub fn top_p(mut self, value: f32) -> Self {
        self.top_p = Some(value);
        self
    }

    /// Set top-k sampling.
    pub fn top_k(mut self, value: u32) -> Self {
        self.top_k = Some(value);
        self
    }

    /// Set maximum number of generated tokens.
    pub fn max_tokens(mut self, value: i32) -> Self {
        self.max_tokens = Some(value);
        self
    }

    /// Add a single stop sequence.
    pub fn stop(mut self, sequence: impl Into<String>) -> Self {
        self.stop.get_or_insert_with(Vec::new).push(sequence.into());
        self
    }

    /// Set multiple stop sequences.
    pub fn stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.stop = Some(sequences);
        self
    }

    /// Set response format (JSON / JSON schema).
    pub fn response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }

    /// Pass additional parameters to the chat template.
    pub fn chat_template_kwargs(mut self, value: Value) -> Self {
        self.chat_template_kwargs = Some(value);
        self
    }

    /// Set reasoning format.
    pub fn reasoning_format(mut self, value: impl Into<String>) -> Self {
        self.reasoning_format = Some(value.into());
        self
    }

    /// Force reasoning output to always be included.
    pub fn thinking_forced_open(mut self, value: bool) -> Self {
        self.thinking_forced_open = Some(value);
        self
    }

    /// Enable parsing of tool calls from model output.
    pub fn parse_tool_calls(mut self, value: bool) -> Self {
        self.parse_tool_calls = Some(value);
        self
    }

    /// Allow parallel or multiple tool calls.
    pub fn parallel_tool_calls(mut self, value: bool) -> Self {
        self.parallel_tool_calls = Some(value);
        self
    }
}


