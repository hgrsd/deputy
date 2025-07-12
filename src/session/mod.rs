use std::collections::HashMap;

use crate::{
    model::{Message, Model},
    tool::Tool,
};

pub struct Session<M: Model, F: Fn(&Message)> {
    model: M,
    message_history: Vec<Message>,
    tools: HashMap<String, Box<dyn Tool>>,
    on_message: F,
}

impl<M: Model, F: Fn(&Message)> Session<M, F> {
    pub fn new(model: M, tools: HashMap<String, Box<dyn Tool>>, on_message: F) -> Self {
        Self {
            model,
            message_history: Vec::new(),
            tools,
            on_message,
        }
    }

    pub async fn send_message(&mut self, message: Message) -> anyhow::Result<()> {
        loop {
            let mut turn_finished = true;

            self.message_history.push(message.clone());

            let response = self
                .model
                .send_message(message.clone(), self.message_history.clone())
                .await?;

            for message in response {
                self.message_history.push(message.clone());
                (self.on_message)(&message);
                if let Message::ToolCall {
                    id,
                    tool_name,
                    arguments,
                } = message
                {
                    turn_finished = false;
                    let tool = self
                        .tools
                        .get(&tool_name)
                        .ok_or(anyhow::anyhow!("Tool not found: {}", tool_name))?;
                    let result = match tool.call(arguments).await {
                        Ok(output) => Message::ToolResult {
                            id,
                            output,
                            is_error: false,
                        },
                        Err(error) => Message::ToolResult {
                            id,
                            output: error.to_string(),
                            is_error: true,
                        },
                    };
                    self.message_history.push(result);
                }
            }

            if turn_finished {
                break;
            }
        }

        Ok(())
    }
}
