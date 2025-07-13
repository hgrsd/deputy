use std::collections::HashMap;

use crate::{
    core::{Message, Tool},
    provider::anthropic::{anthropic_model::AnthropicModel, types::Tool as AnthropicTool},
    session::Session,
};

pub struct AnthropicSessionBuilder<F: Fn(&Message)> {
    api_key: Option<String>,
    model_name: Option<String>,
    max_tokens: Option<u32>,
    system_prompt: Option<String>,
    tools: HashMap<String, Box<dyn Tool>>,
    on_message: Option<F>,
}

impl<F: Fn(&Message)> AnthropicSessionBuilder<F> {
    pub fn new() -> Self {
        Self {
            api_key: None,
            model_name: None,
            max_tokens: None,
            system_prompt: None,
            tools: HashMap::new(),
            on_message: None,
        }
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

    pub fn system_prompt(mut self, system_prompt: &str) -> Self {
        self.system_prompt = Some(system_prompt.to_owned());
        self
    }

    pub fn tool(mut self, tool: Box<dyn Tool>) -> Self {
        let name = tool.name();
        self.tools.insert(name, tool);
        self
    }

    pub fn on_message(mut self, callback: F) -> Self {
        self.on_message = Some(callback);
        self
    }

    pub fn build(self) -> anyhow::Result<Session<AnthropicModel, F>> {
        let api_key = self
            .api_key
            .ok_or_else(|| anyhow::anyhow!("API key is required"))?;
        let model_name = self
            .model_name
            .ok_or_else(|| anyhow::anyhow!("Model name is required"))?;
        let max_tokens = self
            .max_tokens
            .ok_or_else(|| anyhow::anyhow!("Max tokens is required"))?;
        let on_message = self
            .on_message
            .ok_or_else(|| anyhow::anyhow!("Message callback is required"))?;

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
            self.system_prompt,
            anthropic_tools,
        );

        Ok(Session::new(anthropic_model, self.tools, on_message))
    }
}
