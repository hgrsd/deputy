use ignore::WalkBuilder;
use std::path::Path;
use std::{path::PathBuf, str::FromStr};

pub struct Context {
    agent_instructions: Option<String>,
    cwd: String,
    initial_file_tree: Option<String>,
}

impl Context {
    fn generate_file_tree(cwd: &Path) -> Option<String> {
        let mut tree_lines = Vec::new();

        let walker = WalkBuilder::new(cwd)
            .max_depth(Some(2))
            .hidden(false)
            .git_ignore(true)
            .git_exclude(true)
            .git_global(true)
            .build();

        let mut entries: Vec<_> = walker.collect();
        entries.sort_by(|a, b| match (a.as_ref(), b.as_ref()) {
            (Ok(a_entry), Ok(b_entry)) => a_entry.path().cmp(b_entry.path()),
            _ => std::cmp::Ordering::Equal,
        });

        for result in entries {
            match result {
                Ok(entry) => {
                    if let Ok(relative_path) = entry.path().strip_prefix(cwd) {
                        if relative_path.as_os_str().is_empty() {
                            continue;
                        }

                        let depth = relative_path.components().count() - 1;
                        let indent = "  ".repeat(depth);
                        let file_name = relative_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy();

                        let marker = if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                            format!("{}/ (directory)", file_name)
                        } else {
                            file_name.to_string()
                        };

                        tree_lines.push(format!("{}{}", indent, marker));
                    }
                }
                Err(_) => continue,
            }
        }

        if tree_lines.is_empty() {
            None
        } else {
            Some(tree_lines.join("\n"))
        }
    }

    pub fn from_env() -> Self {
        let cwd = std::env::current_dir().expect("Unable to detect current working directory");

        let initial_file_tree = Self::generate_file_tree(&cwd);

        let instruction_paths_priority_order = [cwd.join("DEPUTY.md"),
            PathBuf::from_str("~/.deputy/DEPUTY.md").unwrap(),
            cwd.join("AGENTS.md"),
            cwd.join("CLAUDE.md"),
            PathBuf::from_str("~/.claude/CLAUDE.md").unwrap()];
        let instructions = instruction_paths_priority_order
            .iter()
            .find(|path| path.exists())
            .and_then(|path| std::fs::read_to_string(path).ok());

        Self {
            agent_instructions: instructions,
            cwd: cwd.to_string_lossy().into_owned(),
            initial_file_tree,
        }
    }

    pub fn system_prompt(&self) -> String {
        let mut prompt = String::new();
        prompt.push_str(&format!("
# Deputy

You are an agentic code assistant called deputy.
You will refer to yourself as the user's deputy.

# Tool calling

Use the tools available and your reasoning power to assist the user as best as you can.

- When making tool calls, I'd like you to explain in some detail what you are doing and why, so that the user understands the process. Be clear about the reasons for each tool call (unless it's blindingly obvious).
- Try to make as few as possible that will allow you to achieve your goals. Many tools might have batch functionality (like reading multiple files in one go); try using those where relevant.
- You can ask for more than one tool call in a single turn. If you want to read files, list some others, and make an edit to yet another, and maybe run a command too, you can just ask for all of these tool calls to be
performed in a single turn. No need to do them one-by-one. The user will decide which ones to allow.

# Collaboration

Whenever the user asks something that is ambiguous, or when your tools give you multiple reasonable options,you need to work through the ambiguity together with the user.
The best way of doing this is by dialogue; ask the user questions to help figure out what they need, offer options and architectural approaches, and distil, in collaboration, a good plan of action.
This is especially important when figuring out product or UX issues, or working through engineering trade-offs.

If the user asks you to do something that is not possible, you will refuse and explain why.

# Style

You are succint and to the point. You're never jovial and you try to avoid cliches. You think through issues carefully and provide reasoned responses. Assume your user is intelligent, open-minded, and curious.

# Planning

When you are about to start working on a piece of code, you should always plan out your approach before starting. It is best to start off with a todo list and present that to the user for their feedback and approval.
Then, you work through the todo list step by step. Stop after each step and summarise what you have done, then tick off that item from the todo list. For each step, you should also consider the potential risks and benefits of the approach you are taking, and discuss these with the user. This will help ensure that you are making the best possible decisions for the project.

You should never just start editing files without a plan and without user approval.

# Context

You are currently operating from the following working directory: {}.
",
self.cwd
));

        if let Some(file_tree) = &self.initial_file_tree {
            prompt.push_str(&format!("

## Initial Project Structure

Here is the file tree of the current working directory at startup (this may change as we work):

{}

Note: This is a snapshot from when Deputy started. The actual file structure may have changed during our conversation as files are created, modified, or deleted.
", file_tree));
        }

        if let Some(instructions) = &self.agent_instructions {
            prompt.push_str("\n
            # User instructions
            The user has provided the following instructions, which you should follow as best as possible, wherever relevant, unless the user has specifically given you
            permission to deviate from them. If you think you might benefit from deviating from them, then you should always ask the user for permission to do so.
        \n\n");
            prompt.push_str(&format!(
                "<instructions>\n{}\n</instructions>\n",
                instructions
            ));
        }
        prompt
    }
}
