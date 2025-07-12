use serde::Deserialize;

use crate::core::Tool;

pub struct ListFilesTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    path: String,
}

impl Tool for ListFilesTool {
    fn name(&self) -> String {
        "list_files_tool".to_owned()
    }

    fn description(&self) -> String {
        "List files in a directory. The directory must be a path relative to the the current working directory. If an empty path is provided, the current working directory will be used.".to_owned()
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory, relative to the current working directory. If an empty path is provided, the current working directory will be used."
                }
            },
            "required": ["path"]
        })
    }

    fn call(
        &self,
        args: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + '_>>
    {
        Box::pin(async move {
            let input: Input = serde_json::from_value(args)?;

            let mut output = String::new();
            let cwd = std::env::current_dir().expect("Failed to get current working directory");
            let path = if input.path.is_empty() {
                cwd
            } else {
                cwd.join(&input.path)
            };

            let entries = std::fs::read_dir(path).expect("Failed to read directory");
            for entry in entries {
                let entry = entry.expect("Failed to read directory entry");
                output.push_str(&format!("{}\n", entry.path().display()));
            }
            Ok(output)
        })
    }
}
