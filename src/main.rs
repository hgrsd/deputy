use std::io::{self, BufRead, Write};

use rig::{
    client::{CompletionClient, ProviderClient},
    completion::{Chat, Prompt},
    providers::anthropic,
};

mod tool;

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

    let mut history = Vec::new();

    let stdin = std::io::stdin();
    loop {
        print!("> ");
        io::stdout().flush()?;
        let line = stdin.lock().lines().next().unwrap()?;
        if line.trim().to_lowercase() == "exit" {
            break;
        }
        let result = agent.prompt(line.clone()).multi_turn(50).await?;
        history.push(rig::message::Message::user(line));
        history.push(rig::message::Message::assistant(result.clone()));
        println!("{}\n", result);
    }

    Ok(())
}
