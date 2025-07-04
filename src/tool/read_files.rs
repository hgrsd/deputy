use anyhow::Result;
use serde::Deserialize;

pub struct ReadFilesTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    paths: Vec<String>,
    sample: Option<usize>,
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
                "Read files. The paths must be relative to the the current working directory. Each file will be read and returned as a string. Optionally, you can decide to sample the first N lines of each file."
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
                    "sample": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "Number of lines to sample from each file. If not provided, the entire file will be read."
                    }
                },
                "required": ["paths"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("tool call ({}) - {:?}", Self::NAME, args);
        let cwd = std::env::current_dir().expect("Failed to get current working directory");
        let mut output = String::new();
        for path in args.paths {
            let joined_path = cwd.join(&path);
            let data = std::fs::read_to_string(&joined_path).expect("Failed to read file");
            let lines = data.lines().collect::<Vec<_>>();
            let sample = args.sample.unwrap_or(lines.len());
            let sampled_lines = &lines[..sample.min(lines.len())];
            output.push_str(&format!(
                "path: {}\ndata: \n{}",
                joined_path.display(),
                sampled_lines.join("\n")
            ));
        }
        Ok(output)
    }
}
