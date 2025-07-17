use serde::Deserialize;
use std::process::Command;

use crate::{core::Tool, io::IO};

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
        let input: Input = serde_json::from_value(args).expect("unable to parse argument");
        io.show_message(
            "deputy wants to execute the following command",
            &input.command,
        );
    }

    fn permission_id(&self, args: serde_json::Value) -> String {
        let input: Input = serde_json::from_value(args).expect("unable to parse argument");
        input.command.split_whitespace().take(1).collect()
    }

    fn call<'a>(
        &'a self,
        args: serde_json::Value,
        io: &'a mut Box<dyn IO>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>
    {
        Box::pin(async move {
            let input: Input = serde_json::from_value(args)?;

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

            let output = if !stderr.is_empty() {
                let error_output = stderr.lines().take(10).collect::<Vec<&str>>().join("\n");
                let mut formatted_error = String::new();
                formatted_error.push_str(&format!("\x1b[31m{}", error_output));
                if stderr.lines().count() > 10 {
                    formatted_error.push_str("\n... (truncated)");
                }
                formatted_error.push_str("\x1b[0m");
                formatted_error
            } else {
                let output = stdout.lines().take(10).collect::<Vec<&str>>().join("\n");
                let mut formatted_output = String::new();
                formatted_output.push_str(&format!("\x1b[32m{}", output));
                if stdout.lines().count() > 10 {
                    formatted_output.push_str("\n... (truncated)");
                }
                formatted_output.push_str("\x1b[0m");
                formatted_output
            };

            io.show_message(&input.command, &output);

            Ok(result)
        })
    }
}