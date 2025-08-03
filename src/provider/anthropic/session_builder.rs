use std::collections::HashMap;

use crate::{
    context::Context,
    core::Tool,
    io::IO,
    provider::anthropic::{anthropic_model::AnthropicModel, types::Tool as AnthropicTool},
    session::Session,
};

pub struct AnthropicSessionBuilder<'a> {
    context: Option<&'a Context>,
    tools: HashMap<String, Box<dyn Tool>>,
    io: Option<&'a mut Box<dyn IO>>,
}

impl<'a> AnthropicSessionBuilder<'a> {
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

    pub fn build(self) -> anyhow::Result<Session<'a, AnthropicModel>> {
        let context = self
            .context
            .ok_or_else(|| anyhow::anyhow!("Context is required"))?;
        let io = self.io.ok_or_else(|| anyhow::anyhow!("IO is required"))?;

        // Get API key from environment
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY must be set"))?;

        let anthropic_tools = if self.tools.is_empty() {
            None
        } else {
            Some(
                self.tools
                    .values()
                    .map(|tool| AnthropicTool {
                        name: tool.name(),
                        description: tool.description(),
                        input_schema: tool.input_schema(),
                    })
                    .collect(),
            )
        };

        let anthropic_model = AnthropicModel::new(
            api_key,
            context.model_config.model_name.clone(),
            context.model_config.max_tokens,
            Some(context.session_config.to_system_prompt()),
            anthropic_tools,
            context.model_config.base_url_override.clone()
        );

        Ok(Session::new(anthropic_model, self.tools, io, context))
    }
}
