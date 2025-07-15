use crate::{
    core::Message,
    provider::anthropic::session_builder::AnthropicSessionBuilder,
    tools::{ExecCommandTool, ListFilesTool, ReadFilesTool, WriteFileTool},
    ui::input::InputHandler,
};
use clap::Parser;

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

fn get_terminal_width() -> usize {
    match crossterm::terminal::size() {
        Ok((width, _)) => width as usize,
        Err(_) => 80,
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut wrapped_lines = Vec::new();

    for line in text.lines() {
        if line.len() <= width {
            wrapped_lines.push(line.to_string());
        } else {
            let mut current_line = String::new();
            let words: Vec<&str> = line.split_whitespace().collect();

            for word in words {
                if word.len() > width {
                    if !current_line.is_empty() {
                        wrapped_lines.push(current_line.clone());
                        current_line.clear();
                    }
                    for chunk in word.chars().collect::<Vec<_>>().chunks(width) {
                        wrapped_lines.push(chunk.iter().collect());
                    }
                } else if current_line.len() + word.len() + 1 <= width {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(word);
                } else {
                    if !current_line.is_empty() {
                        wrapped_lines.push(current_line.clone());
                    }
                    current_line = word.to_string();
                }
            }

            if !current_line.is_empty() {
                wrapped_lines.push(current_line);
            }
        }
    }

    wrapped_lines
}

fn print_message_box(title: &str, text: &str) {
    let terminal_width = get_terminal_width();
    let content_width = terminal_width.saturating_sub(4);

    println!("\n┌─ {}", title);

    let wrapped_lines = wrap_text(text, content_width);
    for line in wrapped_lines {
        println!("│ {}", line);
    }

    println!("└─");
}

fn on_message(message: &Message) {
    match message {
        Message::User(text) => {
            print_message_box("You", text);
        }
        Message::Model(text) => {
            print_message_box("Deputy", text);
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

    let list_files_tool = Box::new(ListFilesTool {});
    let read_files_tool = Box::new(ReadFilesTool {});
    let write_file_tool = Box::new(WriteFileTool {});
    let exec_command_tool = Box::new(ExecCommandTool {});

    let mut session = AnthropicSessionBuilder::new()
        .api_key(&anthropic_key)
        .max_tokens(3_000)
        .model_name(&args.model)
        .system_prompt("You are an agentic code assistant called deputy.\n\
            You will refer to yourself as the user's deputy.\n\
            Use the tools available and your reasoning power to assist the user as best as you can.\n\
            Whenever the user asks something that is ambiguous, or when your tools give you multiple reasonable options,\
            you need to work through the ambiguity together with the user.\n\
            The best way of doing this is by socratic dialogue; ask the user questions to help figure out what they need,\
            offer options and architectural approaches, and distil, in collaboration, a good plan of action.\n\
            If the user asks you to do something that is not possible, you will refuse and explain why.\n\n\
            # Language and style\n\n\
            - You speak like a friendly, good-hearted, jaded and slightly sarcastic very senior engineer.\n\
            - You like using humour to lighten the mood and make the conversation more enjoyable. In particular, you like being witty, and using coarse language where it fits.\n\
            - You never use emojis; they are boring and distracting.\n\
            ")
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
                session.send_message(message).await?;
            }
            None => break,
        }
    }

    Ok(())
}