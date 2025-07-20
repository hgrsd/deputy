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
    let mut old_line_num = 1;
    let mut new_line_num = 1;
    
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                result.push_str(&format!("\x1b[31m{:4}:- {}\x1b[0m", old_line_num, change.value()));
                old_line_num += 1;
            }
            ChangeTag::Insert => {
                result.push_str(&format!("\x1b[32m{:4}:+ {}\x1b[0m", new_line_num, change.value()));
                new_line_num += 1;
            }
            ChangeTag::Equal => {
                result.push_str(&format!("{:4}:  {}", old_line_num, change.value()));
                old_line_num += 1;
                new_line_num += 1;
            }
        }
    }
    result
}

fn diff_summary(old_content: &str, new_content: &str, max_lines: usize) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut found_first_change = false;
    let mut result = String::new();
    let mut equal_lines_before_change = Vec::new();
    let mut old_line_num = 1;
    let mut new_line_num = 1;
    
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                if !found_first_change {
                    // Collect equal lines before first change, keeping only the last 5
                    equal_lines_before_change.push((old_line_num, format!("{:4}:  {}", old_line_num, change.value())));
                    if equal_lines_before_change.len() > 5 {
                        equal_lines_before_change.remove(0);
                    }
                } else {
                    // After first change, include equal lines normally
                    result.push_str(&format!("{:4}:  {}\x1b[0m", old_line_num, change.value()));
                }
                old_line_num += 1;
                new_line_num += 1;
            }
            ChangeTag::Delete => {
                if !found_first_change {
                    // This is the first change - add the collected equal lines before it
                    found_first_change = true;
                    for (_, equal_line) in &equal_lines_before_change {
                        result.push_str(&format!("{}\x1b[0m", equal_line));
                    }
                }
                
                result.push_str(&format!("\x1b[31m{:4}:- {}\x1b[0m", old_line_num, change.value()));
                old_line_num += 1;
            }
            ChangeTag::Insert => {
                if !found_first_change {
                    // This is the first change - add the collected equal lines before it
                    found_first_change = true;
                    for (_, equal_line) in &equal_lines_before_change {
                        result.push_str(&format!("{}\x1b[0m", equal_line));
                    }
                }
                
                result.push_str(&format!("\x1b[32m{:4}:+ {}\x1b[0m", new_line_num, change.value()));
                new_line_num += 1;
            }
        }
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