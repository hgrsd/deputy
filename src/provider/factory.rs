use anyhow::Result;

use crate::{
    core::{Message, Tool},
    provider::anthropic::anthropic_model::AnthropicModel,
    provider::{Provider, anthropic::session_builder::AnthropicSessionBuilder},
    session::Session,
};

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn build_anthropic_session<F: Fn(&Message)>(
        model: &str,
        tools: Vec<Box<dyn Tool>>,
        on_message: F,
    ) -> Result<Session<AnthropicModel, F>> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

        let mut builder = AnthropicSessionBuilder::new()
            .api_key(&api_key)
            .max_tokens(3_000)
            .model_name(model)
            .system_prompt(&crate::context::system_prompt())
            .on_message(on_message);

        for tool in tools {
            builder = builder.tool(tool);
        }

        builder.build()
    }
    pub fn build_session<F: Fn(&Message)>(
        provider: Provider,
        model: &str,
        tools: Vec<Box<dyn Tool>>,
        on_message: F,
    ) -> Result<SessionWrapper<F>> {
        match provider {
            Provider::Anthropic => {
                let session = Self::build_anthropic_session(model, tools, on_message)?;
                Ok(SessionWrapper::Anthropic(session))
            }
        }
    }
}

pub enum SessionWrapper<F: Fn(&Message)> {
    Anthropic(Session<AnthropicModel, F>),
}

impl<F: Fn(&Message)> SessionWrapper<F> {
    pub async fn send_message(&mut self, message: Message) -> Result<()> {
        match self {
            SessionWrapper::Anthropic(session) => session.send_message(message).await,
        }
    }
}
