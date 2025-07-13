use serde::Deserialize;

use crate::core::Tool;

pub struct WriteFileTool;

#[derive(Deserialize, Debug)]
struct Range {
    start: usize,
    end: usize,
}

#[derive(Deserialize, Debug)]
struct Input {
    path: String,
    content: String,
    range: Option<Range>,
}

impl Tool for WriteFileTool {
    fn name(&self) -> String {
        "write_file".to_owned()
    }

    fn description(&self) -> String {
        "Writes or edits a file.".to_owned()
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
                },
                "range": {
                    "type": "object",
                    "description": "A range of lines to be edited. This is useful if you want to edit a specific part of a file. The entire specified range will be replaced with the new content provided. The new content can be shorter or longer than the original range.",
                    "properties": {
                        "start": {
                            "type": "integer",
                            "description": "The first line of the range; inclusive."
                        },
                        "end": {
                            "type": "integer",
                            "description": "The last line of the range; inclusive."
                        }
                    },
                    "required": ["start", "end"]
                }
            },
            "required": ["path", "content"]
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
            let path = cwd.join(&input.path);

            if let Some(range) = input.range {
                let file = std::fs::read_to_string(path)?;
                let iter = file.lines();
                let prefix = iter.clone().take(range.start - 1);
                let postfix = iter.clone().skip(range.end);
                let joined_string: Vec<&str> = prefix
                    .chain(std::iter::once(input.content.as_str()))
                    .chain(postfix)
                    .collect();
                let content = joined_string.join("\n");
                std::fs::write(cwd.join(&input.path), content)
                    .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
            } else {
                std::fs::write(cwd.join(&input.path), input.content)
                    .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
            }
            Ok("File written successfully".to_owned())
        })
    }
}
