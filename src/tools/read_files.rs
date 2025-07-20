use std::path::PathBuf;

use serde::Deserialize;

use crate::{core::Tool, io::IO};

pub struct ReadFilesTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    paths: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

fn get_paths(input: &Input) -> Vec<PathBuf> {
    let cwd = std::env::current_dir().expect("Failed to get current working directory");
    input.paths.iter().map(|p| cwd.join(p)).collect()
}

impl Tool for ReadFilesTool {
    fn name(&self) -> String {
        "read_files".to_owned()
    }

    fn description(&self) -> String {
        "Read files. The paths must be relative to the the current working directory.\
         Each file will be read and returned as a string. Optionally, you can provide a limit and offset for the lines to be read.\
         This is generally a good idea when you want to get a quick sense of what a file contains while preserving some space in your context.\n\
         Never read a file without having first validated that the path exist; especially if the user has given you a filename in their message.\n\n\
         Always prefer reading multiple files at once, rather than calling this tool multiple times, provided that you know which files you want to read. Doing so is more efficient.
         ".to_owned()
    }

    fn ask_permission(&self, args: serde_json::Value, io: &mut Box<dyn IO>) {
        let input: Input = serde_json::from_value(args).expect("unable to parse input");
        let display_paths: Vec<String> = get_paths(&input)
            .iter()
            .map(|p| p.to_string_lossy())
            .map(|s| s.to_string())
            .collect();

        io.show_message(
            "Permission request",
            &format!(
                "deputy wants to read the following files: [{}]",
                display_paths.join(", ")
            ),
        );
    }

    fn permission_id(&self, _args: serde_json::Value) -> String {
        String::from("read_files")
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

    fn call<'a>(
        &'a self,
        args: serde_json::Value,
        _io: &'a mut Box<dyn IO>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>
    {
        Box::pin(async move {
            let input: Input = serde_json::from_value(args)?;

            let paths = get_paths(&input);
            let mut output = String::new();
            for path in &paths {
                match std::fs::read_to_string(path) {
                    Ok(data) => {
                        let lines = data.lines().collect::<Vec<_>>();
                        let limit = input.limit.unwrap_or(lines.len());
                        let offset = input.offset.unwrap_or(0);
                        let sampled_lines =
                            &lines[offset..offset + limit.min(lines.len() - offset)];
                        output.push_str(&format!(
                            "<path>\n{}\n</path>\n<data>\n{}\n</data>\n",
                            path.display(),
                            sampled_lines.join("\n")
                        ));
                    }
                    Err(error) => output.push_str(&format!(
                        "<path>\n{}\n</path>\n<error>\n{}\n</error>\n",
                        path.display(),
                        error,
                    )),
                };
            }
            Ok(output)
        })
    }
}
