#[derive(Clone, Debug)]
pub enum Message {
    User(String),
    Model(String),
    ToolCall {
        id: std::option::Option<String>,
        tool_name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        id: std::option::Option<String>,
        output: String,
        is_error: bool,
    },
}

#[derive(Debug)]
pub enum ModelError {
    ApiError(String),
    RequestError(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelError::ApiError(msg) => write!(f, "API error: {}", msg),
            ModelError::RequestError(msg) => write!(f, "Access error: {}", msg),
        }
    }
}

impl std::error::Error for ModelError {}

pub trait Model {
    fn send_message(
        &self,
        message: Message,
        message_history: Vec<Message>,
    ) -> impl Future<Output = Result<Vec<Message>, ModelError>>;
}
