use std::collections::HashMap;

use crate::{
    context::Context,
    core::Tool,
    error::{Result, SessionError},
    io::IO,
    provider::openai::{openai_model::OpenAIModel, types::Tool as OpenAITool, types::Function},
    session::Session,
};

pub struct OpenAISessionBuilder<'a> {
    context: Option<&'a Context>,
    tools: HashMap<String, Box<dyn Tool>>,
    io: Option<&'a mut Box<dyn IO>>,
}

impl<'a> OpenAISessionBuilder<'a> {
    pub fn new() -> Self {
        Self {
            context: None,
            tools: HashMap::new(),
            io: None,
        }
    }

    pub fn io(mut self, io: &'a mut Box<dyn IO>) -> Self {
        self.io = Some(io);
        self
    }

    pub fn context(mut self, context: &'a Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn tool(mut self, tool: Box<dyn Tool>) -> Self {
        let name = tool.name();
        self.tools.insert(name, tool);
        self
    }

    pub fn build(self) -> Result<Session<'a, OpenAIModel>> {
        let context = self
            .context
            .ok_or_else(|| SessionError::Processing { reason: "Context is required".to_string() })?;
        let io = self.io.ok_or_else(|| SessionError::Processing { reason: "IO is required".to_string() })?;

        // Get API key from environment
        let api_key = std::env::var("OPENAI_API_KEY")?;

        let openai_tools = if self.tools.is_empty() {
            None
        } else {
            Some(
                self.tools
                    .values()
                    .map(|tool| OpenAITool {
                        tool_type: "function".to_string(),
                        function: Function {
                            name: tool.name(),
                            description: tool.description(),
                            parameters: tool.input_schema(),
                        },
                    })
                    .collect(),
            )
        };

        let openai_model = OpenAIModel::new(
            api_key,
            context.model_config.model_name.clone(),
            Some(context.model_config.max_tokens),
            openai_tools,
            context.model_config.base_url_override.clone()
        );

        Ok(Session::new(openai_model, self.tools, io, context))
    }
}
