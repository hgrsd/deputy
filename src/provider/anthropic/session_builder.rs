use std::collections::HashMap;

use crate::{
    context::Context,
    core::Tool,
    io::IO,
    provider::anthropic::{anthropic_model::AnthropicModel, types::Tool as AnthropicTool},
    session::Session,
};

pub struct AnthropicSessionBuilder<'a> {
    api_key: Option<String>,
    model_name: Option<String>,
    max_tokens: Option<u32>,
    context: Option<&'a Context>,
    tools: HashMap<String, Box<dyn Tool>>,
    io: Option<&'a mut Box<dyn IO>>,
}

impl<'a> AnthropicSessionBuilder<'a> {
    pub fn new() -> Self {
        Self {
            api_key: None,
            model_name: None,
            max_tokens: None,
            context: None,
            tools: HashMap::new(),
            io: None,
        }
    }

    pub fn io(mut self, io: &'a mut Box<dyn IO>) -> Self {
        self.io = Some(io);
        self
    }

    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_owned());
        self
    }

    pub fn model_name(mut self, model_name: &str) -> Self {
        self.model_name = Some(model_name.to_owned());
        self
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
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
        let api_key = self
            .api_key
            .ok_or_else(|| anyhow::anyhow!("API key is required"))?;
        let model_name = self
            .model_name
            .ok_or_else(|| anyhow::anyhow!("Model name is required"))?;
        let max_tokens = self
            .max_tokens
            .ok_or_else(|| anyhow::anyhow!("Max tokens is required"))?;
        let context = self
            .context
            .ok_or_else(|| anyhow::anyhow!("Context is required"))?;
        let io = self.io.ok_or_else(|| anyhow::anyhow!("IO is required"))?;

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
            model_name,
            max_tokens,
            Some(context.system_prompt()),
            anthropic_tools,
        );

        Ok(Session::new(anthropic_model, self.tools, io, context))
    }
}