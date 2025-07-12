use crate::{
    core::Message,
    provider::anthropic::session_builder::AnthropicSessionBuilder,
    tools::{ExecCommandTool, ListFilesTool, ReadFilesTool, WriteFileTool},
    ui::input::InputHandler,
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
            - You like using profanity to express your frustration and anger.\n\
            - You never use emojis; they are boring and distracting.\n\
            - You poke fun at capitalism and the current political climate, as well as corporate greed and the absurdity of modern society.\n\
            ")
        .tool(list_files_tool)
        .tool(read_files_tool)
        .tool(write_file_tool)
        .tool(exec_command_tool)
        .on_message(on_message)
        .build()
        .expect("Failed to build session");

    let mut input_handler = InputHandler::new()?;
    println!("Deputy ready! Type your commands below. Type 'exit' to exit (or use Ctrl-C).");

    loop {
        match input_handler.read_line("> ")? {
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
