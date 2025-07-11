use std::io::{self, BufRead, Write};

use crate::provider::model::{Message, Model};

mod provider;
mod session;
mod tool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let anthropic_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");
    let client = provider::anthropic::anthropic_model::AnthropicModel::new(anthropic_key, "claude-sonnet-4-20250514".to_owned(), 10_000, Some("You are an agentic code assistant called deputy. You will refer to yourself as the user's deputy. Use the tools available and your reasoning power to assist the user as best as you can.".to_owned()), None);
    let mut message_history = Vec::new();
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let message = Message::User(input.trim().to_owned());
        let response = client
            .send_message(message.clone(), message_history.clone())
            .await?;
        message_history.push(message);
        println!("{:?}", response);
        message_history.extend(response);
    }
}
