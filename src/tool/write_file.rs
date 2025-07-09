use serde::Deserialize;

use crate::tool::tool::Tool;

pub struct WriteFileTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    path: String,
    content: String,
}

#[derive(Debug, thiserror::Error)]
#[error("An error occurred while writing the file")]
pub struct WriteFileError;

impl Tool for WriteFileTool {
    const NAME: &'static str = "write_file";
    type Error = WriteFileError;

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
        std::fs::write(cwd.join(&input.path), input.content).map_err(|_| WriteFileError)?;
        Ok("File written successfully".to_owned())
    }
}
