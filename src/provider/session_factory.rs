use anyhow::Result;

use crate::{
    context::Context,
    core::Tool,
    io::IO,
    provider::{
        Provider,
        anthropic::{anthropic_model::AnthropicModel, session_builder::AnthropicSessionBuilder},
        openai::{openai_model::OpenAIModel, session_builder::OpenAISessionBuilder},
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

    fn build_openai_session<'a>(
        tools: Vec<Box<dyn Tool>>,
        io: &'a mut Box<dyn IO>,
        context: &'a Context,
    ) -> Result<Session<'a, OpenAIModel>> {
        let mut builder = OpenAISessionBuilder::new()
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
            Provider::OpenAI => {
                let session = Self::build_openai_session(tools, io, context)?;
                Ok(SessionWrapper::OpenAI(session))
            }
        }
    }
}

pub enum SessionWrapper<'a> {
    Anthropic(Session<'a, AnthropicModel>),
    OpenAI(Session<'a, OpenAIModel>),
}

impl<'a> SessionWrapper<'a> {
    pub async fn run(&mut self) -> Result<()> {
        match self {
            SessionWrapper::Anthropic(session) => session.run().await,
            SessionWrapper::OpenAI(session) => session.run().await,
        }
    }
}