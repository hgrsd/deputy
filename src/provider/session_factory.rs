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
        tools: Vec<Box<dyn Tool>>,
        io: &'a mut Box<dyn IO>,
        context: &'a Context,
    ) -> Result<Session<'a, AnthropicModel>> {
        let mut builder = AnthropicSessionBuilder::new()
            .context(context)
            .io(io);

        for tool in tools {
            builder = builder.tool(tool);
        }

        builder.build()
    }

    pub fn build_session<'a>(
        tools: Vec<Box<dyn Tool>>,
        io: &'a mut Box<dyn IO>,
        context: &'a Context,
    ) -> Result<SessionWrapper<'a>> {
        match context.provider {
            Provider::Anthropic => {
                let session = Self::build_anthropic_session(tools, io, context)?;
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