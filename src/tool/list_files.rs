use anyhow::Result;
use serde::Deserialize;

pub struct ListFilesTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    path: String,
}

#[derive(Debug, thiserror::Error)]
#[error("An error occurred while listing files")]
pub struct ListFilesError;

impl rig::tool::Tool for ListFilesTool {
    const NAME: &'static str = "list_files";

    type Error = ListFilesError;

    type Args = Input;

    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_owned(),
            description: "List files in a directory. The directory must be a path relative to the the current working directory. If an empty path is provided, the current working directory will be used.".to_owned(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the directory, relative to the current working directory. If an empty path is provided, the current working directory will be used."
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut output = String::new();
        let cwd = std::env::current_dir().expect("Failed to get current working directory");
        let path = if args.path.is_empty() {
            cwd
        } else {
            cwd.join(&args.path)
        };

        let entries = std::fs::read_dir(path).expect("Failed to read directory");
        for entry in entries {
            let entry = entry.expect("Failed to read directory entry");
            output.push_str(&format!("{}\n", entry.path().display()));
        }
        Ok(output)
    }
}
