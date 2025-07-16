use std::time::Duration;

use reqwest::{Response, StatusCode};

use crate::{
    core::{Message, Model, ModelError},
    provider::anthropic::types::{
        ContentBlock, CreateMessageRequest, CreateMessageResponse, ErrorResponse,
        Message as AnthropicMessage, Tool,
    },
};

pub struct AnthropicModel {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    model_name: String,
    max_tokens: u32,
    system_prompt: Option<String>,
    tools: Option<Vec<Tool>>,
}

impl AnthropicModel {
    pub fn new(
        api_key: String,
        model_name: String,
        max_tokens: u32,
        system_prompt: Option<String>,
        tools: Option<Vec<Tool>>,
    ) -> Self {
        let base_url = String::from("https://api.anthropic.com/v1");
        let client = reqwest::Client::new();
        Self {
            api_key,
            base_url,
            client,
            model_name,
            max_tokens,
            system_prompt,
            tools,
        }
    }

    async fn post_with_retry(
        &self,
        api_url: &str,
        request: &CreateMessageRequest,
    ) -> Result<Response, ModelError> {
        const MAX_RETRIES: u32 = 3;
        const BASE_DELAY_SECS: u64 = 6;

        for attempt in 0..=MAX_RETRIES {
            let request = self
                .client
                .post(api_url)
                .json(request)
                .header("Content-Type", "application/json")
                .header("anthropic-version", "2023-06-01")
                .header("x-api-key", self.api_key.clone())
                .build()
                .map_err(|e| ModelError::RequestError(e.to_string()))?;

            let response = self
                .client
                .execute(request)
                .await
                .map_err(|e| ModelError::ApiError(format!("{}", e)))?;

            if response.status() == StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
                let delay_secs = BASE_DELAY_SECS * 2_u64.pow(attempt);
                eprintln!(
                    "Anthropic API rate limit hit; retrying in {}s... (attempt {}/{})",
                    delay_secs,
                    attempt + 1,
                    MAX_RETRIES
                );
                tokio::time::sleep(Duration::from_secs(delay_secs)).await;
            } else {
                return Ok(response);
            }
        }
        unreachable!("Loop should have returned a response")
    }
}

impl From<Message> for AnthropicMessage {
    fn from(message: Message) -> Self {
        match message {
            Message::User(text) => AnthropicMessage {
                content: vec![ContentBlock::Text { text }],
                role: crate::provider::anthropic::types::Role::User,
            },
            Message::Model(text) => AnthropicMessage {
                content: vec![ContentBlock::Text { text }],
                role: crate::provider::anthropic::types::Role::Assistant,
            },
            Message::ToolCall {
                id,
                tool_name,
                arguments,
            } => AnthropicMessage {
                content: vec![ContentBlock::ToolUse {
                    id: id.expect("all tool calls are expected to have an id"),
                    name: tool_name,
                    input: arguments,
                }],
                role: crate::provider::anthropic::types::Role::Assistant,
            },
            Message::ToolResult {
                id,
                output,
                is_error,
            } => AnthropicMessage {
                content: vec![ContentBlock::ToolResult {
                    tool_use_id: id.expect("all tool results are expected to have an id"),
                    content: output,
                    is_error: if is_error { Some(true) } else { Some(false) },
                }],
                role: crate::provider::anthropic::types::Role::User,
            },
        }
    }
}

impl Model for AnthropicModel {
    async fn send_message(
        &self,
        message: Message,
        message_history: Vec<Message>,
    ) -> Result<Vec<Message>, ModelError> {
        let all_messages: Vec<AnthropicMessage> = message_history
            .into_iter()
            .chain(std::iter::once(message))
            .map(|message| message.into())
            .collect();
        let request = CreateMessageRequest {
            model: self.model_name.clone(),
            max_tokens: self.max_tokens,
            messages: all_messages,
            system: self.system_prompt.clone(),
            tools: self.tools.clone(),
            temperature: None,
            top_p: None,
            top_k: None,
            stream: None,
            stop_sequences: None,
            metadata: None,
        };

        let api_url = format!("{}/messages", self.base_url);
        let result = self.post_with_retry(&api_url, &request).await?;
        if !result.status().is_success() {
            let error = result
                .json::<ErrorResponse>()
                .await
                .map_err(|_| ModelError::ApiError("Unknown API error occurred".to_owned()))?;
            return Err(ModelError::ApiError(error.error.message));
        }

        let body = result
            .json::<CreateMessageResponse>()
            .await
            .map_err(|e| ModelError::ApiError(format!("Failed to parse response: {}", e)))?;

        let mut result = vec![];
        for block in body.content {
            match block {
                ContentBlock::Text { text } => {
                    let message = Message::Model(text);
                    result.push(message);
                }
                ContentBlock::ToolUse { id, name, input } => {
                    let message = Message::ToolCall {
                        id: Some(id),
                        tool_name: name,
                        arguments: input,
                    };
                    result.push(message);
                }
                _ => {
                    return Err(ModelError::ApiError(
                        "Only Text and ToolUse blocks are supported".to_owned(),
                    ));
                }
            }
        }
        Ok(result)
    }
}
