use std::path::PathBuf;

use serde::Deserialize;
use similar::{ChangeTag, TextDiff};

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

fn diff(old_content: &str, new_content: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut result = String::new();
    for change in diff.iter_all_changes() {
        let prefix = match change.tag() {
            ChangeTag::Delete => "\x1b[31m- ",
            ChangeTag::Insert => "\x1b[32m+ ",
            ChangeTag::Equal => " ",
        };
        result.push_str(&format!("{}{}\x1b[0m", prefix, change.value()));
    }
    result
}

fn diff_summary(old_content: &str, new_content: &str, max_lines: usize) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut found_first_change = false;
    let mut result = String::new();
    for change in diff.iter_all_changes() {
        if let ChangeTag::Equal = change.tag() {
            if !found_first_change {
                continue;
            }
        }
        found_first_change = true;
        let prefix = match change.tag() {
            ChangeTag::Delete => "\x1b[31m- ",
            ChangeTag::Insert => "\x1b[32m+ ",
            ChangeTag::Equal => " ",
        };
        result.push_str(&format!("{}{}", prefix, change.value()));
    }

    let mut curtailed = result
        .lines()
        .take(max_lines)
        .collect::<Vec<&str>>()
        .join("\n");
    if result.lines().count() > max_lines {
        curtailed.push_str(&format!(
            "\x1b[0m\n(...{} more lines)",
            result.lines().count() - max_lines
        ));
    }

    curtailed
}

fn replace_range(full_text: &str, range: &Range, new_content: &str) -> String {
    let mut lines = full_text.lines().collect::<Vec<_>>();
    lines.splice(
        range.start - 1..range.end,
        new_content.lines().collect::<Vec<_>>(),
    );
    lines.join("\n")
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
        &'a self,
        args: serde_json::Value,
        io: &'a mut Box<dyn IO>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>
    {
        Box::pin(async move {
            let input: Input = serde_json::from_value(args)?;

            let path = get_path(&input);
            let current_file = std::fs::read_to_string(&path).unwrap_or_default();

            let new_content = if let Some(range) = input.range {
                replace_range(&current_file, &range, &input.content)
            } else {
                input.content
            };

            let short_diff = diff_summary(&current_file, &new_content, 15);

            io.show_message(&format!("deputy edited {}", path.display()), &short_diff);

            std::fs::write(&path, &new_content)
                .map_err(|e| anyhow::anyhow!("Failed to write file: {}", e))?;
            Ok("File written successfully".to_owned())
        })
    }

    fn ask_permission(&self, args: serde_json::Value, io: &mut Box<dyn IO>) {
        let input: Input = serde_json::from_value(args).expect("unable to parse input");
        let path = get_path(&input);
        let current_file = std::fs::read_to_string(&path).unwrap_or_default();
        let new_file = if let Some(range) = input.range {
            replace_range(&current_file, &range, &input.content)
        } else {
            input.content
        };
        let diff = diff(&current_file, &new_file);
        io.show_message(
            &format!("deputy wants to edit the file at {}", input.path),
            &diff,
        );
    }

    fn permission_id(&self, _args: serde_json::Value) -> String {
        String::from("write_file")
    }
}
