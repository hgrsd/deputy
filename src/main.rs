use std::io::{self, Write};

use crate::{
    model::Message,
    provider::anthropic::session_builder::AnthropicSessionBuilder,
    tool::{ExecCommandTool, ListFilesTool, ReadFilesTool, WriteFileTool},
};

mod model;
mod provider;
mod session;
mod tool;

fn on_message(message: &Message) {
    match message {
        Message::User(text) => println!("> {}", text),
        Message::Model(text) => println!("deputy: {}", text),
        Message::ToolCall {
            id: _,
            tool_name,
            arguments: _,
        } => println!("tool use: {}", tool_name),
        Message::ToolResult {
            id,
            output: _,
            is_error,
        } => println!(
            "tool result for {}, error: {:?}",
            id.clone().expect("id must be present"),
            is_error
        ),
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
