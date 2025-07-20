use std::collections::HashMap;

use crate::{
    context::Context,
    core::{Message, Model, PermissionMode, Tool},
    io::IO,
};

pub struct Session<'a, M: Model> {
    model: M,
    message_history: Vec<Message>,
    tools: HashMap<String, Box<dyn Tool>>,
    tool_permissions: HashMap<String, PermissionMode>,
    io: &'a mut Box<dyn IO>,
}

impl<'a, M: Model> Session<'a, M> {
    pub fn new(
        model: M,
        tools: HashMap<String, Box<dyn Tool>>,
        io: &'a mut Box<dyn IO>,
        _context: &Context,
    ) -> Self {
        Self {
            model,
            message_history: Vec::new(),
            tools,
            tool_permissions: HashMap::new(),
            io,
        }
    }

    fn prompt_for_permission(&mut self, tool_name: &str, permission_id: &str) -> bool {
        let response = self
            .io
            .get_user_input(&format!(
                "[1: allow, 2: always allow for {}, 3: deny and tell me what to do differently] > ",
                permission_id,
            ))
            .expect("Failed to read user input");

        if let Some(s) = response {
            match s.as_str() {
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
        } else {
            false
        }
    }

    fn display_message(&self, message: &Message) {
        match message {
            Message::User(text) => self.io.show_message("You", text),
            Message::Model(text) => self.io.show_message("Deputy", text),
            _ => {}
        }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        while let Some(input) = self.io.get_user_input("> ")? {
            if input.is_empty() {
                continue;
            }
            if input == "exit" {
                break;
            }
            self.io.show_message("You", &input);
            let message = Message::User(input.clone());
            self.send_message(message).await?;
        }

        Ok(())
    }

    pub async fn send_message(&mut self, message: Message) -> anyhow::Result<()> {
        let mut current_message = message.clone();
        let debug_mode = std::env::var("DEPUTY_DEBUG").unwrap_or_default() == "true";
        let mut turn_finished = false;

        while !turn_finished {
            turn_finished = true;

            let response = self
                .model
                .send_message(current_message.clone(), self.message_history.clone())
                .await?;

            self.message_history.push(current_message.clone());

            for m in response {
                self.message_history.push(m.clone());
                self.display_message(&m);
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
                                tool.ask_permission(arguments.clone(), self.io);
                            }
                            self.prompt_for_permission(&tool_name, &permission_id)
                        }
                        PermissionMode::ApprovedForId { command_id } => {
                            if &permission_id == command_id {
                                true
                            } else {
                                {
                                    let tool = self.tools.get(&tool_name).unwrap();
                                    tool.ask_permission(arguments.clone(), self.io);
                                }
                                self.prompt_for_permission(&tool_name, &permission_id)
                            }
                        }
                    };

                    if !allow {
                        self.message_history.push(Message::ToolResult {
                            id,
                            output: String::from("The user did not allow this tool to be executed"),
                            is_error: true,
                        });
                        turn_finished = true;
                    } else {
                        let tool = self
                            .tools
                            .get(&tool_name)
                            .ok_or(anyhow::anyhow!("Tool not found: {}", tool_name))?;

                        let result = match tool.call(arguments, self.io).await {
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
                        };
                        current_message = result;
                    };
                }
            }
        }
        Ok(())
    }
}
