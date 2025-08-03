use serde::Deserialize;
use std::process::Command;

use crate::{core::Tool, error::{ToolError, Result}, io::IO};

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

    fn ask_permission(&self, args: serde_json::Value, io: &mut Box<dyn IO>) {
        let input: Input = serde_json::from_value(args).unwrap_or(Input { command: "<invalid command>".to_string() });
        io.show_message(
            "deputy wants to execute the following command",
            &input.command,
        );
    }

    fn permission_id(&self, args: serde_json::Value) -> Result<String> {
        let input: Input = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArguments {
                reason: format!("exec_command: {}", e)
            })?;
        Ok(input.command.split_whitespace().take(1).collect())
    }

    fn call<'a>(
        &'a self,
        args: serde_json::Value,
        io: &'a mut Box<dyn IO>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send + 'a>>
    {
        Box::pin(async move {
            let input: Input = serde_json::from_value(args)
                .map_err(|e| ToolError::InvalidArguments {
                    reason: format!("exec_command: {}", e)
                })?;

            let output = Command::new("sh")
                .arg("-c")
                .arg(&input.command)
                .output()
                .map_err(|e| ToolError::ExecutionFailed {
                    reason: format!("exec_command: {}", e)
                })?;

            if !output.status.success() {
                return Err(ToolError::ExecutionFailed {
                    reason: format!("command '{}' failed with exit code {}: {}", input.command, output.status.code().unwrap_or(-1), String::from_utf8_lossy(&output.stderr))
                }.into());
            }

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

            let output = if !stderr.is_empty() {
                stderr
                    .lines()
                    .map(|line| {
                        let mut s = String::new();
                        s.push_str("\x1b[31m");
                        s.push_str(line);
                        s.push_str("\x1b[0m");
                        s
                    })
                    .collect::<Vec<String>>()
                    .join("\n")
            } else {
                stdout.lines().take(10).collect::<Vec<&str>>().join("\n")
            };

            io.show_snippet(&format!("deputy is running {}", &input.command), &output);

            Ok(result)
        })
    }
}
