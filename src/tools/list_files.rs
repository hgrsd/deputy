use ignore::gitignore::GitignoreBuilder;
use serde::Deserialize;
use std::path::Path;

use crate::core::Tool;

pub struct ListFilesTool;

#[derive(Deserialize, Debug)]
pub struct Input {
    path: String,
    #[serde(default)]
    recursive: bool,
}

impl Tool for ListFilesTool {
    fn name(&self) -> String {
        "list_files_tool".to_owned()
    }

    fn description(&self) -> String {
        "List files in a directory. The directory must be a path relative to the the current working directory. If an empty path is provided, the current working directory will be used. When recursive is true, recursively lists all files and directories in a tree format.".to_owned()
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
                }
            },
            "required": ["path"]
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
            let path = if input.path.is_empty() {
                cwd
            } else {
                cwd.join(&input.path)
            };

            let gitignore = build_gitignore(&path);
            
            if input.recursive {
                Ok(list_files_recursive(&path, 0, &gitignore))
            } else {
                let mut output = String::new();
                let entries = std::fs::read_dir(&path).expect("Failed to read directory");
                for entry in entries {
                    let entry = entry.expect("Failed to read directory entry");
                    let entry_path = entry.path();
                    
                    if should_include_path(&entry_path, &gitignore) {
                        output.push_str(&format!("{}\n", entry_path.display()));
                    }
                }
                Ok(output)
            }
        })
    }
}

fn list_files_recursive(path: &Path, depth: usize, gitignore: &ignore::gitignore::Gitignore) -> String {
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
        
        for entry in entries {
            if let Ok(entry) = entry {
                let entry_path = entry.path();
                let name = entry_path.file_name().unwrap().to_string_lossy();
                
                if should_include_path(&entry_path, gitignore) {
                    if entry_path.is_dir() {
                        output.push_str(&format!("{}{}/\n", indent, name));
                        output.push_str(&list_files_recursive(&entry_path, depth + 1, gitignore));
                    } else {
                        output.push_str(&format!("{}{}\n", indent, name));
                    }
                }
            }
        }
    }
    
    output
}

fn build_gitignore(path: &Path) -> ignore::gitignore::Gitignore {
    let mut builder = GitignoreBuilder::new(path);
    
    // Look for .gitignore files starting from the given path and walking up
    let mut current_path = path.to_path_buf();
    loop {
        let gitignore_path = current_path.join(".gitignore");
        if gitignore_path.exists() {
            if let Some(e) = builder.add(&gitignore_path) {
                eprintln!("Warning: Failed to parse .gitignore at {}: {}", gitignore_path.display(), e);
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

fn should_include_path(path: &Path, gitignore: &ignore::gitignore::Gitignore) -> bool {
    match gitignore.matched(path, path.is_dir()) {
        ignore::Match::None | ignore::Match::Whitelist(_) => true,
        ignore::Match::Ignore(_) => false,
    }
}
