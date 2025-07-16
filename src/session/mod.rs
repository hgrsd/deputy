use std::{
    collections::HashMap,
    io::{BufRead, Write},
};

use crate::{
    core::{Message, Model, PermissionMode, Tool},
    ui::Spinner,
};

pub struct Session<M: Model, F: Fn(&Message)> {
    model: M,
    message_history: Vec<Message>,
    tools: HashMap<String, Box<dyn Tool>>,
    tool_permissions: HashMap<String, PermissionMode>,
    on_message: F,
}

impl<M: Model, F: Fn(&Message)> Session<M, F> {
    pub fn new(model: M, tools: HashMap<String, Box<dyn Tool>>, on_message: F) -> Self {
        Self {
            model,
            message_history: Vec::new(),
            tools,
            tool_permissions: HashMap::new(),
            on_message,
        }
    }

    fn prompt_for_permission(&mut self, tool_name: &str, permission_id: &str) -> bool {
        print!(
            "[1: allow, 2: always allow for {}, 3: deny] > ",
            permission_id
        );
        std::io::stdout().flush().unwrap();
        let mut response = String::new();
        let stdin = std::io::stdin();
        stdin.lock().read_line(&mut response).unwrap();

        match response.trim() {
            "1" => true,
            "2" => {
                self.tool_permissions.insert(
                    tool_name.to_string(),
                    PermissionMode::ApprovedForId {
                        command_id: permission_id.to_string(),
                    },
                );
                true
            }
            _ => false,
        }
    }

    pub async fn send_message(&mut self, message: Message) -> anyhow::Result<()> {
        let mut current_message = message.clone();
        let mut spinner: Option<Spinner> = None;
        let debug_mode = std::env::var("DEPUTY_DEBUG").unwrap_or_default() == "true";
        let mut turn_finished = false;

        while !turn_finished {
            turn_finished = true;

            if spinner.is_none() && !debug_mode {
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

                    let permission_id = {
                        let tool = self
                            .tools
                            .get(&tool_name)
                            .ok_or(anyhow::anyhow!("Tool not found: {}", tool_name))?;

                        if debug_mode {
                            eprintln!(
                                "[DEBUG] Tool call: {} with arguments: {}",
                                tool_name, arguments
                            );
                        }

                        tool.permission_id(arguments.clone())
                    };

                    let allow = match self
                        .tool_permissions
                        .get(&tool_name)
                        .unwrap_or(&PermissionMode::Ask)
                    {
                        PermissionMode::Ask => {
                            {
                                let tool = self.tools.get(&tool_name).unwrap();
                                tool.ask_permission(arguments.clone());
                            }
                            self.prompt_for_permission(&tool_name, &permission_id)
                        }
                        PermissionMode::ApprovedForId { command_id } => {
                            if &permission_id == command_id {
                                true
                            } else {
                                {
                                    let tool = self.tools.get(&tool_name).unwrap();
                                    tool.ask_permission(arguments.clone());
                                }
                                self.prompt_for_permission(&tool_name, &permission_id)
                            }
                        }
                    };

                    let result = if !allow {
                        Message::ToolResult {
                            id,
                            output: String::from("The user did not allow this tool to be executed"),
                            is_error: true,
                        }
                    } else {
                        let tool = self
                            .tools
                            .get(&tool_name)
                            .ok_or(anyhow::anyhow!("Tool not found: {}", tool_name))?;

                        match tool.call(arguments).await {
                            Ok(output) => {
                                if debug_mode {
                                    eprintln!("[DEBUG] Tool result (success): {}", output);
                                }
                                Message::ToolResult {
                                    id,
                                    output,
                                    is_error: false,
                                }
                            }
                            Err(error) => {
                                if debug_mode {
                                    eprintln!("[DEBUG] Tool result (error): {}", error);
                                }
                                Message::ToolResult {
                                    id,
                                    output: error.to_string(),
                                    is_error: true,
                                }
                            }
                        }
                    };
                    if !debug_mode {
                        spinner = Some(Spinner::new(&format!("Executing {}...", tool_name)));
                    }
                    current_message = result;
                }
            }
        }

        if let Some(s) = spinner.take() {
            s.finish();
        }

        Ok(())
    }
}
