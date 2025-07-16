use crate::{
    core::Message,
    provider::{Provider, factory::ProviderFactory},
    tools::{ExecCommandTool, ListFilesTool, ReadFilesTool, WriteFileTool},
    ui::{DisplayManager, input::InputHandler},
};
use clap::Parser;

mod context;
mod core;
mod provider;
mod session;
mod tools;
mod ui;

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

    let display_manager = DisplayManager::new();

    let on_message = |message: &Message| {
        display_manager.handle_message(message);
    };

    let tools: Vec<Box<dyn crate::core::Tool>> = vec![
        Box::new(ListFilesTool {}),
        Box::new(ReadFilesTool {}),
        Box::new(WriteFileTool {}),
        Box::new(ExecCommandTool {}),
    ];

    let model = args.model.unwrap();
    let mut session =
        ProviderFactory::build_session(args.provider.clone(), &model, tools, on_message)?;

    let mut input_handler = InputHandler::new()?;

    println!(
        "┌─ Deputy ready! Using provider: {}, model: {}",
        args.provider, &model
    );
    println!("│ Type your commands below. Type 'exit' to exit (or use Ctrl-C).");
    println!("└─");

    while let Some(input) = input_handler.read_line("\n> ")? {
        if input.is_empty() {
            continue;
        }
        if input == "exit" {
            break;
        }

        let message = Message::User(input);
        display_manager.handle_message(&message);
        session.send_message(message).await?;
    }

    Ok(())
}
