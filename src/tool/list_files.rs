use serde::Deserialize;

use crate::tool::tool::Tool;

pub struct ListFilesTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    path: String,
}

#[derive(Debug, thiserror::Error)]
#[error("An error occurred while listing files")]
pub struct ListFilesError;

impl Tool for ListFilesTool {
    const NAME: &'static str = "list_files";
    type Error = ListFilesError;

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

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<String> {
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
    }
}
