use crate::{
    error::Result,
    context::{Context, ModelConfig, SessionConfig},
    io::{IO, TerminalIO},
    provider::{Provider, session_factory::SessionFactory},
    tools::ToolRegistry,
};
use clap::Parser;
use std::path::PathBuf;

mod context;
mod core;
mod error;
mod io;
mod provider;
mod session;
mod tools;

#[derive(Parser)]
#[command(name = "deputy")]
#[command(about = "An agentic CLI assistant")]
#[command(version)]
struct Args {
    /// Provider to use (anthropic or openai)
    #[arg(short, long, value_enum, default_value_t = Provider::Anthropic)]
    provider: Provider,

    /// Model to use (provider-specific, e.g. claude-sonnet-4-20250514 for Anthropic, gpt-4o for OpenAI)
    #[arg(short, long, default_value = "claude-sonnet-4-20250514")]
    model: String,

    /// Enable yolo mode - run all tool calls without asking for permission (dangerous!)
    #[arg(long)]
    yolo: bool,

    /// Override API base url; this is useful if you want to point deputy at a local or third-party OpenAI or Anthropic compatible API.
    #[arg(short, long)]
    base_url: Option<String>,

    /// Custom configuration file path (when provided, only this file will be read instead of the default priority order)
    #[arg(short, long)]
    config: Option<PathBuf>
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Create context with all configuration
    let model_config = ModelConfig::new(args.provider.clone(), args.model, args.yolo, args.base_url)?;
    let session_config = SessionConfig::from_env(args.config)?;
    let context = Context::new(model_config, session_config);

    let tools = ToolRegistry::with_default_tools().into_tools();
    let mut io: Box<dyn IO> = Box::new(TerminalIO::new()?);

    if context.model_config.yolo_mode {
        io.show_message(
            "⚠️  YOLO MODE ENABLED ⚠️",
            "All tool calls will execute automatically without permission prompts.\nThis can be dangerous - use with caution!",
        );
    }

    io.show_message(
        &format!(
            "Deputy ready! Using provider: {}, model: {}{}{}",
            context.model_config.provider, 
            context.model_config.model_name,
            if context.model_config.yolo_mode { " (YOLO MODE)" } else { "" },
            if let Some(ref url) = context.model_config.base_url_override { format!("base url: {}", url)} else { String::from("") }
        ),
        "Type your commands below. Type 'exit' to exit (or use Ctrl-C).",
    );

    let mut session = SessionFactory::build_session(tools, &mut io, &context)?;
    session.run().await?;

    Ok(())
}
