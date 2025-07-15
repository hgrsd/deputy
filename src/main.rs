use crate::{
    config::SYSTEM_PROMPT,
    core::Message,
    provider::anthropic::session_builder::AnthropicSessionBuilder,
    tools::{ExecCommandTool, ListFilesTool, ReadFilesTool, WriteFileTool},
    ui::{input::InputHandler, DisplayManager},
};
use clap::Parser;

mod config;
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
    /// Claude model to use
    #[arg(short, long, default_value = "claude-sonnet-4-20250514")]
    model: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

    let display_manager = DisplayManager::new();
    
    // Create a closure that captures the display manager
    let on_message = |message: &Message| {
        display_manager.handle_message(message);
    };

    let list_files_tool = Box::new(ListFilesTool {});
    let read_files_tool = Box::new(ReadFilesTool {});
    let write_file_tool = Box::new(WriteFileTool {});
    let exec_command_tool = Box::new(ExecCommandTool {});

    let mut session = AnthropicSessionBuilder::new()
        .api_key(&anthropic_key)
        .max_tokens(3_000)
        .model_name(&args.model)
        .system_prompt(SYSTEM_PROMPT)
        .tool(list_files_tool)
        .tool(read_files_tool)
        .tool(write_file_tool)
        .tool(exec_command_tool)
        .on_message(on_message)
        .build()
        .expect("Failed to build session");

    let mut input_handler = InputHandler::new()?;
    println!("┌─ Deputy ready! Using model: {}", args.model);
    println!("│ Type your commands below. Type 'exit' to exit (or use Ctrl-C).");
    println!("└─");

    loop {
        match input_handler.read_line("\n> ")? {
            Some(input) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if trimmed == "exit" {
                    break;
                }

                let message = Message::User(trimmed.to_owned());
                
                // Display the user message first
                display_manager.handle_message(&message);
                
                // Then send it to the session for processing
                session.send_message(message).await?;
            }
            None => break,
        }
    }

    Ok(())
}