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

            // Collect all tool calls from the response
            let mut tool_calls = Vec::new();
            let mut other_messages = Vec::new();

            for m in response {
                match &m {
                    Message::ToolCall { .. } => tool_calls.push(m),
                    _ => other_messages.push(m),
                }
            }

            for m in other_messages {
                self.message_history.push(m.clone());
                self.display_message(&m);
            }

            if !tool_calls.is_empty() {
                turn_finished = false;
                let mut tool_results = self.process_tool_calls(tool_calls, debug_mode).await?;

                let (last_call, last_result) = tool_results.pop().unwrap();

                for (call, result) in &tool_results {
                    self.message_history.push(call.clone());
                    self.message_history.push(result.clone());
                }

                self.message_history.push(last_call.clone());
                current_message = last_result;
            }
        }
        Ok(())
    }

    async fn process_tool_calls(
        &mut self,
        tool_calls: Vec<Message>,
        debug_mode: bool,
    ) -> anyhow::Result<Vec<(Message, Message)>> {
        let mut tool_results = Vec::new();
        let mut user_denied_tool = false;

        for tool_call in tool_calls {
            if let Message::ToolCall {
                id,
                tool_name,
                arguments,
            } = tool_call.clone()
            {
                if user_denied_tool {
                    tool_results.push((tool_call.clone(), Message::ToolResult {
                        id,
                        output: String::from("Tool execution cancelled because the user denied a previous tool call in this batch. Control has been returned to the user to provide guidance on how to proceed."),
                        is_error: true,
                    }));
                    continue;
                }

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
                    user_denied_tool = true;
                    tool_results.push((tool_call.clone(), Message::ToolResult {
                        id,
                        output: String::from("The user denied this tool call. Control has been returned to the user to provide guidance on how to proceed differently."),
                        is_error: true,
                    }));
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
                    tool_results.push((tool_call.clone(), result));
                }
            }
        }

        Ok(tool_results)
    }
}
