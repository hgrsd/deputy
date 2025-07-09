use serde::Deserialize;
use std::io::{self, Write};
use std::process::Command;

use crate::tool::tool::Tool;

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

impl Tool for ExecCommandTool {
    const NAME: &'static str = "exec_command";
    type Error = ExecCommandError;

    fn description(&self) -> String {
        "Execute a bash command in the current working directory.".to_owned()
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The bash command to execute."
                }
            },
            "required": ["command"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<String> {
        let input: Input = serde_json::from_value(args)?;
        
        println!("tool call (exec_command) - {:?}", input);

        print!(
            "Are you sure you want to execute this command: '{}' [y/N]? ",
            input.command
        );
        io::stdout().flush().unwrap();

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input).unwrap();

        let response = user_input.trim().to_lowercase();
        if response != "y" && response != "yes" {
            return Err(ExecCommandError(ExecCommandErrorKind::CancelledByUser).into());
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(&input.command)
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
