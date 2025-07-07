use anyhow::Result;
use serde::Deserialize;

pub struct ReadFilesTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    paths: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
#[error("An error occurred while reading the files")]
pub struct ReadFileError;

impl rig::tool::Tool for ReadFilesTool {
    const NAME: &'static str = "read_files";

    type Error = ReadFileError;

    type Args = Input;

    type Output = String;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_owned(),
            description:
                "Read files. The paths must be relative to the the current working directory. Each file will be read and returned as a string. Optionally, you can provide a limit and offset for the lines to be read. This is generally a good idea when you want to get a quick sense of what a file contains while preserving some space in your context."
                    .to_owned(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Paths to the files to be read, relative to the current working directory.",
                    },
                    "limit": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "Number of lines to read from each file. If not provided, the entire file will be read."
                    },
                    "offset": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "Number of lines to skip from the beginning of each file. If not provided, reading will start at the beginning."
                    }
                },
                "required": ["paths"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let cwd = std::env::current_dir().expect("Failed to get current working directory");
        let mut output = String::new();
        for path in args.paths {
            let joined_path = cwd.join(&path);
            let data = std::fs::read_to_string(&joined_path).expect("Failed to read file");
            let lines = data.lines().collect::<Vec<_>>();
            let limit = args.limit.unwrap_or(lines.len());
            let offset = args.offset.unwrap_or(0);
            let sampled_lines = &lines[offset..offset + limit.min(lines.len() - offset)];
            output.push_str(&format!(
                "path: {}\ndata: \n{}",
                joined_path.display(),
                sampled_lines.join("\n")
            ));
        }
        Ok(output)
    }
}
