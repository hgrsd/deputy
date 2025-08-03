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

use crate::error::Result;
use std::future::Future;

pub trait Model {
    fn send_message(
        &self,
        message: Message,
        message_history: Vec<Message>,
    ) -> impl Future<Output = Result<Vec<Message>>>;
}
