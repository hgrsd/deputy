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
    /// Provider to use
    #[arg(short, long, value_enum, default_value_t = Provider::Anthropic)]
    provider: Provider,

    /// Model to use (provider-specific)
    #[arg(short, long, default_value = "claude-sonnet-4-20250514")]
    model: Option<String>,

    /// Enable yolo mode - run all tool calls without asking for permission (dangerous!)
    #[arg(long)]
    yolo: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Err(error) = args.provider.validate_configuration() {
        eprintln!("Configuration error: {}", error);
        eprintln!("\nFor provider setup help:");
        match args.provider {
            Provider::Anthropic => {
                eprintln!("  Set ANTHROPIC_API_KEY environment variable");
                eprintln!("  Get your API key from: https://console.anthropic.com/");
            }
        }
        std::process::exit(1);
    }

    let tools = ToolRegistry::with_default_tools().into_tools();

    let mut io: Box<dyn IO> = Box::new(TerminalIO::new()?);
    let model = args.model.unwrap();
    let context = Context::from_env();

    if args.yolo {
        io.show_message(
            "⚠️  YOLO MODE ENABLED ⚠️",
            "All tool calls will execute automatically without permission prompts.\nThis can be dangerous - use with caution!",
        );
    }

    io.show_message(
        &format!(
            "Deputy ready! Using provider: {}, model: {}{}",
            args.provider, 
            &model,
            if args.yolo { " (YOLO MODE)" } else { "" }
        ),
        "Type your commands below. Type 'exit' to exit (or use Ctrl-C).",
    );

    let mut session =
        SessionFactory::build_session(args.provider.clone(), &model, tools, &mut io, &context, args.yolo)?;
    session.run().await?;

    Ok(())
}