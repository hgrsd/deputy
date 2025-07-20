use anyhow::Result;

use crate::{
    context::Context,
    core::Tool,
    io::IO,
    provider::{
        Provider,
        anthropic::{anthropic_model::AnthropicModel, session_builder::AnthropicSessionBuilder},
    },
    session::Session,
};

pub struct SessionFactory;

impl SessionFactory {
    fn build_anthropic_session<'a>(
        model: &str,
        tools: Vec<Box<dyn Tool>>,
        io: &'a mut Box<dyn IO>,
        context: &'a Context,
    ) -> Result<Session<'a, AnthropicModel>> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

        let mut builder = AnthropicSessionBuilder::new()
            .api_key(&api_key)
            .max_tokens(5_000)
            .model_name(model)
            .context(context)
            .io(io);

        for tool in tools {
            builder = builder.tool(tool);
        }

        builder.build()
    }

    pub fn build_session<'a>(
        provider: Provider,
        model: &str,
        tools: Vec<Box<dyn Tool>>,
        io: &'a mut Box<dyn IO>,
        context: &'a Context,
    ) -> Result<SessionWrapper<'a>> {
        match provider {
            Provider::Anthropic => {
                let session = Self::build_anthropic_session(model, tools, io, context)?;
                Ok(SessionWrapper::Anthropic(session))
            }
        }
    }
}

pub enum SessionWrapper<'a> {
    Anthropic(Session<'a, AnthropicModel>),
}

impl<'a> SessionWrapper<'a> {
    pub async fn run(&mut self) -> Result<()> {
        match self {
            SessionWrapper::Anthropic(session) => session.run().await,
        }
    }
}