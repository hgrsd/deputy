use anyhow::Result;
use serde::Deserialize;

pub struct WriteFileTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    path: String,
    content: String,
}

#[derive(Debug, thiserror::Error)]
#[error("An error occurred while writing the file")]
pub struct WriteFileError;

impl rig::tool::Tool for WriteFileTool {
    const NAME: &'static str = "write_file";

    type Error = WriteFileError;

    type Args = Input;

    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_owned(),
            description:
                "Writes a file. The paths must be relative to the the current working directory. The file will be written with the provided content. This can be used to edit files by reading the file content first, and writing it back with the updated content."
                    .to_owned(),
            parameters: serde_json::json!({
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
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("tool call ({}) - {:?}", Self::NAME, args);
        let cwd = std::env::current_dir().expect("Failed to get current working directory");
        std::fs::write(cwd.join(&args.path), args.content).map_err(|_| WriteFileError)?;
        Ok("File written successfully".to_owned())
    }
}
