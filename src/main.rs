use std::io::{self, BufRead, Write};

use rig::{
    client::{CompletionClient, ProviderClient},
    providers::anthropic,
};

use crate::session::Session;

mod session;
mod tool;

fn on_message(message: &str) {
    println!("{}", message);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = anthropic::Client::from_env();
    let agent = client.agent("claude-sonnet-4-20250514")
        .preamble("You are an agentic code assistant called deputy. You will refer to yourself as the user's deputy. Use the tools available and your reasoning power to assist the user as best as you can.")
        .tool(tool::ListFilesTool)
        .tool(tool::ReadFilesTool)
        .tool(tool::WriteFileTool)
        .tool(tool::ExecCommandTool)
        .max_tokens(10_000)
        .build();

    let mut session = Session::new(agent, on_message);

    let stdin = std::io::stdin();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let line = stdin.lock().lines().next().unwrap()?;
        if line.trim().to_lowercase() == "exit" {
            break;
        }
        session.message(line).await?;
    }

    Ok(())
}
