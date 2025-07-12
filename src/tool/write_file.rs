use serde::Deserialize;

use crate::tool::tool::Tool;

pub struct WriteFileTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    path: String,
    content: String,
}

impl Tool for WriteFileTool {
    fn name(&self) -> String {
        "write_file".to_owned()
    }

    fn description(&self) -> String {
        "Writes a file. The paths must be relative to the the current working directory. The file will be written with the provided content. This can be used to edit files by reading the file content first, and writing it back with the updated content.".to_owned()
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to be written, relative to the current working directory.",
                },
                "content": {
                    "type": "string",
                    "description": "Content to be written to the file."
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<String> {
        let input: Input = serde_json::from_value(args)?;

        let cwd = std::env::current_dir().expect("Failed to get current working directory");
        std::fs::write(cwd.join(&input.path), input.content)
            .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
        Ok("File written successfully".to_owned())
    }
}
