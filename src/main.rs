use crate::{
    context::Context,
    io::{IO, TerminalIO},
    provider::{Provider, session_factory::SessionFactory},
    tools::ToolRegistry,
};
use clap::Parser;

mod context;
mod core;
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Create context with all configuration
    let context = Context::new(args.provider.clone(), args.model, args.yolo)?;

    let tools = ToolRegistry::with_default_tools().into_tools();
    let mut io: Box<dyn IO> = Box::new(TerminalIO::new()?);

    if context.yolo_mode {
        io.show_message(
            "⚠️  YOLO MODE ENABLED ⚠️",
            "All tool calls will execute automatically without permission prompts.\nThis can be dangerous - use with caution!",
        );
    }

    io.show_message(
        &format!(
            "Deputy ready! Using provider: {}, model: {}{}",
            context.provider, 
            context.model_name,
            if context.yolo_mode { " (YOLO MODE)" } else { "" }
        ),
        "Type your commands below. Type 'exit' to exit (or use Ctrl-C).",
    );

    let mut session = SessionFactory::build_session(tools, &mut io, &context)?;
    session.run().await?;

    Ok(())
}