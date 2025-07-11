use std::io::{self, Write};

use crate::{
    core::Message,
    provider::anthropic::session_builder::AnthropicSessionBuilder,
    tools::{ExecCommandTool, ListFilesTool, ReadFilesTool, WriteFileTool},
};

mod core;
mod provider;
mod session;
mod tools;
mod ui;

fn on_message(message: &Message) {
    match message {
        Message::User(text) => println!("you > {}", text),
        Message::Model(text) => {
            let lines: Vec<&str> = text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if i == 0 {
                    println!("deputy > {}", line);
                } else {
                    println!("         {}", line);
                }
            }
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");

    let list_files_tool = Box::new(ListFilesTool {});
    let read_files_tool = Box::new(ReadFilesTool {});
    let write_file_tool = Box::new(WriteFileTool {});
    let exec_command_tool = Box::new(ExecCommandTool {});

    let mut session = AnthropicSessionBuilder::new()
        .api_key(&anthropic_key)
        .max_tokens(3_000)
        .model_name("claude-sonnet-4-20250514")
        .system_prompt("You are an agentic code assistant called deputy. You will refer to yourself as the user's deputy. Use the tools available and your reasoning power to assist the user as best as you can.")
        .tool(list_files_tool)
        .tool(read_files_tool)
        .tool(write_file_tool)
        .tool(exec_command_tool)
        .on_message(on_message)
        .build()
        .expect("Failed to build session");

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let message = Message::User(input.trim().to_owned());
        session.send_message(message).await?;
    }
}
