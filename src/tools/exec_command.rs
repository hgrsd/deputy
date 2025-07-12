use serde::Deserialize;
use std::io::{self, Write};
use std::process::Command;

use crate::tools::tool::Tool;

pub struct ExecCommandTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    command: String,
}

impl Tool for ExecCommandTool {
    fn name(&self) -> String {
        "exec_command".to_owned()
    }

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

    fn call(
        &self,
        args: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + '_>>
    {
        Box::pin(async move {
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
                anyhow::bail!("Command execution was cancelled by the user");
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(&input.command)
                .output()
                .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let mut result = String::new();
            if !stdout.is_empty() {
                result.push_str(&format!("STDOUT:\n{}", stdout));
            }
            if !stderr.is_empty() {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&format!("STDERR:\n{}", stderr));
            }

            Ok(result)
        })
    }
}
