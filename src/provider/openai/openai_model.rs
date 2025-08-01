use std::time::Duration;

use reqwest::{Response, StatusCode};

use crate::{
    core::{Message, Model, ModelError},
    provider::openai::types::{
        ChatCompletionRequest, ChatCompletionResponse, ErrorResponse, Message as OpenAIMessage,
        Tool, ToolCall, FunctionCall, Role,
    },
};

pub struct OpenAIModel {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
    model_name: String,
    max_tokens: Option<u32>,
    tools: Option<Vec<Tool>>,
}

impl OpenAIModel {
    pub fn new(
        api_key: String,
        model_name: String,
        max_tokens: Option<u32>,
        tools: Option<Vec<Tool>>,
    ) -> Self {
        let base_url = String::from("https://api.openai.com/v1");
        let client = reqwest::Client::new();
        Self {
            api_key,
            base_url,
            client,
            model_name,
            max_tokens,
            tools,
        }
    }

    async fn post_with_retry(
        &self,
        api_url: &str,
        request: &ChatCompletionRequest,
    ) -> Result<Response, ModelError> {
        const MAX_RETRIES: u32 = 3;
        const BASE_DELAY_SECS: u64 = 6;

        for attempt in 0..=MAX_RETRIES {
            let http_request = self
                .client
                .post(api_url)
                .json(request)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .build()
                .map_err(|e| ModelError::RequestError(e.to_string()))?;

            let response = self
                .client
                .execute(http_request)
                .await
                .map_err(|e| ModelError::ApiError(format!("{}", e)))?;

            if response.status() == StatusCode::TOO_MANY_REQUESTS && attempt < MAX_RETRIES {
                let delay_secs = BASE_DELAY_SECS * 2_u64.pow(attempt);
                eprintln!(
                    "OpenAI API rate limit hit; retrying in {}s... (attempt {}/{})",
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

impl From<Message> for OpenAIMessage {
    fn from(message: Message) -> Self {
        match message {
            Message::User(text) => OpenAIMessage {
                role: Role::User,
                content: Some(text),
                tool_calls: None,
                tool_call_id: None,
            },
            Message::Model(text) => OpenAIMessage {
                role: Role::Assistant,
                content: Some(text),
                tool_calls: None,
                tool_call_id: None,
            },
            Message::ToolCall {
                id,
                tool_name,
                arguments,
            } => OpenAIMessage {
                role: Role::Assistant,
                content: None,
                tool_calls: Some(vec![ToolCall {
                    id: id.expect("all tool calls are expected to have an id"),
                    call_type: "function".to_string(),
                    function: FunctionCall {
                        name: tool_name,
                        arguments: arguments.to_string(),
                    },
                }]),
                tool_call_id: None,
            },
            Message::ToolResult { id, output, .. } => OpenAIMessage {
                role: Role::Tool,
                content: Some(output),
                tool_calls: None,
                tool_call_id: Some(id.expect("all tool results are expected to have an id")),
            },
        }
    }
}

impl Model for OpenAIModel {
    async fn send_message(
        &self,
        message: Message,
        message_history: Vec<Message>,
    ) -> Result<Vec<Message>, ModelError> {
        let all_messages: Vec<OpenAIMessage> = message_history
            .into_iter()
            .chain(std::iter::once(message))
            .map(|message| message.into())
            .collect();

        let request = ChatCompletionRequest {
            model: self.model_name.clone(),
            messages: all_messages,
            tools: self.tools.clone(),
            tool_choice: None,
            temperature: None,
            top_p: None,
            max_tokens: self.max_tokens,
            stop: None,
            stream: None,
        };

        let api_url = format!("{}/chat/completions", self.base_url);
        let result = self.post_with_retry(&api_url, &request).await?;
        
        if !result.status().is_success() {
            let error = result
                .json::<ErrorResponse>()
                .await
                .map_err(|_| ModelError::ApiError("Unknown API error occurred".to_owned()))?;
            return Err(ModelError::ApiError(error.error.message));
        }

        let body = result
            .json::<ChatCompletionResponse>()
            .await
            .map_err(|e| ModelError::ApiError(format!("Failed to parse response: {}", e)))?;

        let mut result = vec![];
        
        for choice in body.choices {
            let message = &choice.message;
            
            if let Some(content) = &message.content {
                result.push(Message::Model(content.clone()));
            }
            
            if let Some(tool_calls) = &message.tool_calls {
                for tool_call in tool_calls {
                    let arguments: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)
                        .map_err(|e| ModelError::ApiError(format!("Failed to parse tool arguments: {}", e)))?;
                    
                    result.push(Message::ToolCall {
                        id: Some(tool_call.id.clone()),
                        tool_name: tool_call.function.name.clone(),
                        arguments,
                    });
                }
            }
        }
        
        Ok(result)
    }
}