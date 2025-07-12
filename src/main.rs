use std::{
    collections::HashMap,
    io::{self, Write},
};

use crate::{
    model::{Message, Model},
    provider::anthropic::{anthropic_model::AnthropicModel, types::Tool as AnthropicTool},
    session::Session,
    tool::{ListFilesTool, Tool},
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
    let list_files_tool = ListFilesTool {};
    let list_files = AnthropicTool {
        name: list_files_tool.name(),
        description: list_files_tool.description(),
        input_schema: list_files_tool.input_schema(),
    };
    let client = AnthropicModel::new(
        anthropic_key, "claude-sonnet-4-20250514".to_owned(),
        10_000,
        Some("You are an agentic code assistant called deputy. You will refer to yourself as the user's deputy. Use the tools available and your reasoning power to assist the user as best as you can.".to_owned()),
        Some(vec![list_files])
    );
    let tools = HashMap::from([(list_files_tool.name(), list_files_tool)]);
    let mut session = Session::new(client, tools, on_message);

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
