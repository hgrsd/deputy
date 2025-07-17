use std::path::PathBuf;

use serde::Deserialize;

use crate::{core::Tool, io::IO};

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

fn get_path(input: &Input) -> PathBuf {
    let cwd = std::env::current_dir().expect("Failed to get current working directory");
    cwd.join(&input.path)
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

    fn call<'a>(
        &self,
        args: serde_json::Value,
        _io: &'a mut Box<dyn IO>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + '_>>
    {
        Box::pin(async move {
            let input: Input = serde_json::from_value(args)?;

            let path = get_path(&input);

            if let Some(range) = input.range {
                let file = std::fs::read_to_string(&path)?;
                let iter = file.lines();
                let prefix = iter.clone().take(range.start - 1);
                let postfix = iter.clone().skip(range.end);
                let joined_string: Vec<&str> = prefix
                    .chain(std::iter::once(input.content.as_str()))
                    .chain(postfix)
                    .collect();
                let content = joined_string.join("\n");
                std::fs::write(&path, content)
                    .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
            } else {
                std::fs::write(&path, input.content)
                    .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
            }
            Ok("File written successfully".to_owned())
        })
    }

    fn ask_permission(&self, args: serde_json::Value, io: &mut Box<dyn IO>) {
        let input: Input = serde_json::from_value(args).expect("unable to parse input");
        let path = get_path(&input);
        io.show_message(
            "Permission request",
            &format!(
                "deputy wants to write or edit the file at {}",
                path.display()
            ),
        );
    }

    fn permission_id(&self, _args: serde_json::Value) -> String {
        String::from("write_file")
    }
}