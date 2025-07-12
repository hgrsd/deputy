use std::collections::HashMap;

use crate::{
    core::{Message, Model, Tool},
    ui::Spinner,
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
        let mut current_message = message.clone();
        let mut spinner: Option<Spinner> = None;

        loop {
            let mut turn_finished = true;

            if spinner.is_none() {
                spinner = Some(Spinner::new("Thinking..."));
            }

            let response = self
                .model
                .send_message(current_message.clone(), self.message_history.clone())
                .await?;

            if let Some(s) = spinner.take() {
                s.finish();
            }

            self.message_history.push(current_message.clone());

            for m in response {
                self.message_history.push(m.clone());
                (self.on_message)(&m);
                if let Message::ToolCall {
                    id,
                    tool_name,
                    arguments,
                } = m
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
                    spinner = Some(Spinner::new(&format!("Executing {}...", tool_name)));
                    current_message = result;
                }
            }

            if turn_finished {
                break;
            }
        }

        if let Some(s) = spinner.take() {
            s.finish();
        }

        Ok(())
    }
}
