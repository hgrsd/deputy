use ignore::gitignore::GitignoreBuilder;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::{core::Tool, io::IO};

pub struct ListFilesTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    path: String,
    #[serde(default)]
    recursive: bool,
    #[serde(default)]
    include_hidden: bool,
}

fn build_path(input: &Input) -> PathBuf {
    let cwd = std::env::current_dir().expect("Failed to get current working directory");
    if input.path.is_empty() {
        cwd
    } else {
        cwd.join(&input.path)
    }
}

impl Tool for ListFilesTool {
    fn name(&self) -> String {
        "list_files_tool".to_owned()
    }

    fn description(&self) -> String {
        "List files in a directory. The directory must be a path relative to the the current working directory. If an empty path is provided, the current working directory will be used. When recursive is true, recursively lists all files and directories in a tree format. Hidden files (starting with '.') are excluded by default unless include_hidden is true. IMPORTANT: Only use include_hidden=true when you have a strong reason to examine hidden files, such as debugging configuration issues or when explicitly asked by the user.".to_owned()
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory, relative to the current working directory. If an empty path is provided, the current working directory will be used."
                },
                "recursive": {
                    "type": "boolean",
                    "description": "When true, recursively lists all files and directories in a tree format. Defaults to false."
                },
                "include_hidden": {
                    "type": "boolean",
                    "description": "When true, includes hidden files and directories (starting with '.'). Defaults to false. IMPORTANT: Only use this when you have a strong reason to examine hidden files, such as debugging configuration issues or when explicitly asked by the user."
                }
            },
            "required": ["path"]
        })
    }

    fn ask_permission(&self, args: serde_json::Value, io: &mut Box<dyn IO>) {
        let input: Input = serde_json::from_value(args).expect("unable to parse input");
        let path = build_path(&input);
        io.show_message(
            "Permission request",
            &format!(
                "list_files wants to read the files and directories in {}",
                path.display()
            ),
        );
    }

    fn permission_id(&self, _args: serde_json::Value) -> String {
        String::from("list_files")
    }

    fn call<'a>(
        &'a self,
        args: serde_json::Value,
        io: &'a mut Box<dyn IO>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<String>> + Send + 'a>>
    {
        Box::pin(async move {
            let input: Input = serde_json::from_value(args)?;

            let path = build_path(&input);
            let gitignore = build_gitignore(&path);

            let output = if input.recursive {
                list_files_recursive(&path, 0, &gitignore, input.include_hidden)
            } else {
                let mut output = String::new();
                let entries = std::fs::read_dir(&path).expect("Failed to read directory");
                for entry in entries {
                    let entry = entry.expect("Failed to read directory entry");
                    let entry_path = entry.path();

                    if should_include_path(&entry_path, &gitignore, input.include_hidden) {
                        if path.is_dir() {
                            output.push_str(&format!("{} (directory)\n", entry_path.display()));
                        } else {
                            output.push_str(&format!("{}\n", entry_path.display()));
                        }
                    }
                }
                output
            };
            io.show_snippet(&format!("deputy is listing files in {}", path.display()), &output);
            Ok(output)
        })
    }
}

fn list_files_recursive(
    path: &Path,
    depth: usize,
    gitignore: &ignore::gitignore::Gitignore,
    include_hidden: bool,
) -> String {
    let mut output = String::new();
    let indent = "  ".repeat(depth);

    if let Ok(entries) = std::fs::read_dir(path) {
        let mut entries: Vec<_> = entries.collect();
        entries.sort_by(|a, b| {
            let a_path = a.as_ref().unwrap().path();
            let b_path = b.as_ref().unwrap().path();

            match (a_path.is_dir(), b_path.is_dir()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a_path.file_name().cmp(&b_path.file_name()),
            }
        });

        for entry in entries.into_iter().flatten() {
            let entry_path = entry.path();

            if should_include_path(&entry_path, gitignore, include_hidden) {
                if entry_path.is_dir() {
                    output.push_str(&format!(
                        "{}{}/ (directory)\n",
                        indent,
                        entry_path.display()
                    ));
                    output.push_str(&list_files_recursive(
                        &entry_path,
                        depth + 1,
                        gitignore,
                        include_hidden,
                    ));
                } else {
                    output.push_str(&format!("{}{}\n", indent, entry_path.display()));
                }
            }
        }
    }

    output
}

fn build_gitignore(path: &Path) -> ignore::gitignore::Gitignore {
    let mut builder = GitignoreBuilder::new(path);

    let mut current_path = path.to_path_buf();
    loop {
        let gitignore_path = current_path.join(".gitignore");
        if gitignore_path.exists() {
            if let Some(e) = builder.add(&gitignore_path) {
                eprintln!(
                    "Warning: Failed to parse .gitignore at {}: {}",
                    gitignore_path.display(),
                    e
                );
            }
        }

        if !current_path.pop() {
            break;
        }
    }

    builder.build().unwrap_or_else(|e| {
        eprintln!("Warning: Failed to build gitignore matcher: {}", e);
        GitignoreBuilder::new(path).build().unwrap()
    })
}

fn should_include_path(
    path: &Path,
    gitignore: &ignore::gitignore::Gitignore,
    include_hidden: bool,
) -> bool {
    if !include_hidden {
        if let Some(name) = path.file_name() {
            if let Some(name_str) = name.to_str() {
                if name_str.starts_with('.') {
                    return false;
                }
            }
        }
    }

    match gitignore.matched(path, path.is_dir()) {
        ignore::Match::None | ignore::Match::Whitelist(_) => true,
        ignore::Match::Ignore(_) => false,
    }
}
