use serde::Deserialize;

use crate::core::Tool;

pub struct ReadFilesTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    paths: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

impl Tool for ReadFilesTool {
    fn name(&self) -> String {
        "read_files".to_owned()
    }

    fn description(&self) -> String {
        "Read files. The paths must be relative to the the current working directory.\
         Each file will be read and returned as a string. Optionally, you can provide a limit and offset for the lines to be read.\
         This is generally a good idea when you want to get a quick sense of what a file contains while preserving some space in your context.\n\
         Never read a file without having first validated that the path exist; especially if the user has given you a filename in their message.\n\
         ".to_owned()
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
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
        })
    }

    fn call(
        &self,
        args: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + '_>>
    {
        Box::pin(async move {
            let input: Input = serde_json::from_value(args)?;

            let cwd = std::env::current_dir().expect("Failed to get current working directory");
            let mut output = String::new();
            for path in input.paths {
                let joined_path = cwd.join(&path);
                let data = std::fs::read_to_string(&joined_path).expect("Failed to read file");
                let lines = data.lines().collect::<Vec<_>>();
                let limit = input.limit.unwrap_or(lines.len());
                let offset = input.offset.unwrap_or(0);
                let sampled_lines = &lines[offset..offset + limit.min(lines.len() - offset)];
                output.push_str(&format!(
                    "path: {}\ndata: \n{}",
                    joined_path.display(),
                    sampled_lines.join("\n")
                ));
            }
            Ok(output)
        })
    }
}
