use std::collections::HashMap;

use crate::{
    context::Context,
    core::{Message, Model, PermissionMode, Tool},
    error::{SessionError, ToolError, Result},
    io::IO,
};

pub struct Session<'a, M: Model> {
    model: M,
    message_history: Vec<Message>,
    tools: HashMap<String, Box<dyn Tool>>,
    tool_permissions: HashMap<String, PermissionMode>,
    io: &'a mut Box<dyn IO>,
    context: &'a Context,
}

impl<'a, M: Model> Session<'a, M> {
    pub fn new(
        model: M,
        tools: HashMap<String, Box<dyn Tool>>,
        io: &'a mut Box<dyn IO>,
        context: &'a Context,
    ) -> Self {
        Self {
            model,
            message_history: Vec::new(),
            tools,
            tool_permissions: HashMap::new(),
            io,
            context,
        }
    }

    fn prompt_for_permission(&mut self, tool_name: &str, permission_id: &str) -> Result<bool> {
        let response = self
            .io
            .get_user_input(&format!(
                "[1: allow, 2: always allow for {}, 3: deny and tell me what to do differently] > ",
                permission_id,
            ))
            .map_err(|e| SessionError::UserInput { 
                reason: format!("Failed to read user input: {}", e) 
            })?;

        if let Some(s) = response {
            match s.as_str() {
                "1" => Ok(true),
                "2" => {
                    self.tool_permissions.insert(
                        tool_name.to_string(),
                        PermissionMode::ApprovedForId {
                            command_id: permission_id.to_string(),
                        },
                    );
                    Ok(true)
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    fn display_message(&self, message: &Message) {
        match message {
            Message::User(text) => self.io.show_message("You", text),
            Message::Model(text) => self.io.show_message("Deputy", text),
            _ => {}
        }
    }

    pub async fn run(&mut self) -> Result<()> {
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

    pub async fn send_message(&mut self, message: Message) -> Result<()> {
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
                let on_rejected = || turn_finished = true;
                let mut tool_results = self
                    .process_tool_calls(tool_calls, debug_mode, on_rejected)
                    .await?;

                // extract the last tool call & result pair; safe because tool_calls isn't empty
                let (last_call, last_result) = tool_results.pop().unwrap(); 
                // add all other calls & results to the messages history
                for (call, result) in tool_results {
                    self.message_history.push(call);
                    self.message_history.push(result.clone());
                }
                self.message_history.push(last_call);
                // if one of the tools was rejected, turn_finished will now be false, meaning we
                // are going to pass back control to the user after we have processed all results
                // this means that we just need to add the last call & result to the message
                // history
                if turn_finished {
                    self.message_history.push(last_result);
                } else {
                    // if the turn isn't finished yet, then we need to add the last call to the
                    // history, but the last result must be set to the current message to continue
                    // the turn, passing back control to the model
                    current_message = last_result;
                }
            }
        }
        Ok(())
    }

    async fn process_tool_calls(
        &mut self,
        tool_calls: Vec<Message>,
        debug_mode: bool,
        mut on_rejected: impl FnMut(),
    ) -> Result<Vec<(Message, Message)>> {
        let mut results = Vec::new();
        let mut batch_cancelled = false;

        for tool_call in tool_calls {
            let result = if batch_cancelled {
                self.create_cancellation_message(&tool_call)
            } else {
                match self.handle_tool_call(tool_call.clone(), debug_mode).await? {
                    Some(result) => result,
                    None => {
                        batch_cancelled = true;
                        on_rejected();
                        self.create_denial_message(&self.extract_tool_id(&tool_call))
                    }
                }
            };
            
            results.push((tool_call, result));
        }

        Ok(results)
    }

    async fn handle_tool_call(
        &mut self,
        tool_call: Message,
        debug_mode: bool,
    ) -> Result<Option<Message>> {
        let Message::ToolCall { id, tool_name, arguments } = tool_call else {
            return Err(SessionError::Processing { reason: "Expected ToolCall".to_string() }.into());
        };
        
        self.log_debug(debug_mode, &format!("Tool call: {} with arguments: {}", tool_name, arguments));

        let permission_id = {
            let tool = self.tools.get(&tool_name)
                .ok_or_else(|| ToolError::NotFound { reason: format!("tool: {}", tool_name) })?;
            tool.permission_id(arguments.clone())?
        };
        
        if !self.authorize_tool_execution(&tool_name, &permission_id, &arguments, debug_mode)? {
            return Ok(None);
        }

        let result = self.execute_tool_call(id.clone().unwrap_or_default(), &tool_name, arguments.clone(), debug_mode).await?;
        Ok(Some(result))
    }

    fn authorize_tool_execution(
        &mut self,
        tool_name: &str,
        permission_id: &str,
        arguments: &serde_json::Value,
        debug_mode: bool,
    ) -> Result<bool> {
        if self.context.model_config.yolo_mode {
            self.log_debug(debug_mode, &format!("YOLO MODE: Auto-allowing tool {} with permission_id {}", tool_name, permission_id));
            return Ok(true);
        }

        let permission_mode = self.tool_permissions.get(tool_name).unwrap_or(&PermissionMode::Ask);
        
        let requires_user_prompt = match permission_mode {
            PermissionMode::Ask => true,
            PermissionMode::ApprovedForId { command_id } => permission_id != command_id,
        };

        if requires_user_prompt {
            {
                let tool = self.tools.get(tool_name)
                    .ok_or_else(|| ToolError::NotFound { reason: format!("tool: {}", tool_name) })?;
                tool.ask_permission(arguments.clone(), self.io);
            }
            self.prompt_for_permission(tool_name, permission_id)
        } else {
            Ok(true)
        }
    }

    async fn execute_tool_call(
        &mut self,
        id: String,
        tool_name: &str,
        arguments: serde_json::Value,
        debug_mode: bool,
    ) -> Result<Message> {
        let result = {
            let tool = self.tools.get(tool_name)
                .ok_or_else(|| ToolError::NotFound { reason: format!("tool: {}", tool_name) })?;
            tool.call(arguments, self.io).await
        };
        let result = match result {
            Ok(output) => {
                self.log_debug(debug_mode, &format!("Tool result (success): {}", output));
                Message::ToolResult { id: Some(id), output, is_error: false }
            }
            Err(error) => {
                self.log_debug(debug_mode, &format!("Tool result (error): {}", error));
                Message::ToolResult { id: Some(id), output: error.to_string(), is_error: true }
            }
        };
        Ok(result)
    }


    fn create_cancellation_message(&self, tool_call: &Message) -> Message {
        let Message::ToolCall { id, .. } = tool_call else {
            panic!("Expected ToolCall message");
        };
        
        Message::ToolResult {
            id: id.clone(),
            output: "Tool execution cancelled because the user denied a previous tool call in this batch. Control has been returned to the user to provide guidance on how to proceed.".to_string(),
            is_error: true,
        }
    }

    fn create_denial_message(&self, id: &str) -> Message {
        Message::ToolResult {
            id: Some(id.to_string()),
            output: "The user denied this tool call. Control has been returned to the user to provide guidance on how to proceed differently.".to_string(),
            is_error: true,
        }
    }

    fn extract_tool_id(&self, tool_call: &Message) -> String {
        match tool_call {
            Message::ToolCall { id, .. } => id.clone().unwrap_or_default(),
            _ => panic!("Expected ToolCall message"),
        }
    }

    fn log_debug(&self, debug_mode: bool, message: &str) {
        if debug_mode {
            eprintln!("[DEBUG] {}", message);
        }
    }
}
