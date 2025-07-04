use anyhow::Result;
use serde::Deserialize;
use std::io::{self, Write};
use std::process::Command;

pub struct ExecCommandTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    command: String,
}

#[derive(Debug, thiserror::Error)]
enum ExecCommandErrorKind {
    #[error("An error occurred while executing the command")]
    ExecutionError,
    #[error("Command execution was cancelled by the user")]
    CancelledByUser,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ExecCommandError(#[from] ExecCommandErrorKind);

impl rig::tool::Tool for ExecCommandTool {
    const NAME: &'static str = "exec_command";

    type Error = ExecCommandError;

    type Args = Input;

    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_owned(),
            description: "Execute a bash command in the current working directory.".to_owned(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute."
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("tool call ({}) - {:?}", Self::NAME, args);

        print!(
            "Are you sure you want to execute this command: '{}' [y/N]? ",
            args.command
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let response = input.trim().to_lowercase();
        if response != "y" && response != "yes" {
            return Err(ExecCommandError(ExecCommandErrorKind::CancelledByUser));
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .output()
            .map_err(|_| ExecCommandError(ExecCommandErrorKind::ExecutionError))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&format!("STDOUT:\n{}", stdout));
        }
        if !stderr.is_empty() {
            if !result.is_empty() {
                result.push_str("\n");
            }
            result.push_str(&format!("STDERR:\n{}", stderr));
        }

        Ok(result)
    }
}
