use ignore::WalkBuilder;
use std::path::Path;
use std::path::PathBuf;
use crate::provider::Provider;
use crate::error::{ConfigError, Result};

pub struct ModelConfig {
    pub provider: Provider,
    pub model_name: String,
    pub base_url_override: Option<String>,
    pub yolo_mode: bool,
    pub max_tokens: u32,
}

pub struct SessionConfig {
    agent_instructions: Option<String>,
    cwd: String,
    initial_file_tree: Option<String>,
}

pub struct Context {
    pub model_config: ModelConfig,
    pub session_config: SessionConfig,
}

impl ModelConfig {
    /// Creates a new ModelConfig with the provided settings.
    /// 
    /// Validates the provider configuration before creating the config.
    /// Sets max_tokens to a default value of 5,000.
    pub fn new(provider: Provider, model_name: String, yolo_mode: bool, base_url_override: Option<String>) -> Result<Self> {
        provider.validate_configuration()?;

        Ok(Self {
            provider,
            model_name,
            base_url_override,
            yolo_mode,
            max_tokens: 5_000,
        })
    }
}

impl SessionConfig {
    /// Creates a SessionConfig from the current environment.
    /// 
    /// Detects the current working directory, generates an initial file tree,
    /// and searches for agent instruction files. If a custom config path is provided,
    /// only that file will be read. Otherwise, searches in this priority order:
    /// 1. `DEPUTY.md` in current directory
    /// 2. `~/.deputy/DEPUTY.md` 
    /// 3. `AGENTS.md` in current directory
    /// 4. `CLAUDE.md` in current directory
    /// 5. `~/.claude/CLAUDE.md`
    pub fn from_env(custom_config_path: Option<PathBuf>) -> Result<Self> {
        let cwd = std::env::current_dir()
            .map_err(|e| ConfigError::Invalid { 
                reason: format!("configuration file: current directory: {}", e)
            })?;

        let initial_file_tree = Self::generate_file_tree(&cwd);

        let instructions = if let Some(custom_path) = custom_config_path {
            // Use custom config path if provided
            if custom_path.exists() {
                std::fs::read_to_string(&custom_path)
                    .map_err(|e| ConfigError::ReadFailed {
                        reason: format!("configuration file {}: {}", custom_path.display(), e)
                    })?
                    .into()
            } else {
                return Err(ConfigError::Invalid {
                    reason: format!("configuration file: {}", custom_path.display())
                }.into());
            }
        } else {
            // Use default priority order
            let home_deputy_path = dirs::home_dir()
                .ok_or_else(|| ConfigError::Invalid {
                    reason: "configuration file: ~/.deputy/DEPUTY.md".to_string()
                })?
                .join(".deputy/DEPUTY.md");
            
            let home_claude_path = dirs::home_dir()
                .ok_or_else(|| ConfigError::Invalid {
                    reason: "configuration file: ~/.claude/CLAUDE.md".to_string()
                })?
                .join(".claude/CLAUDE.md");
            
            let instruction_paths_priority_order = [
                cwd.join("DEPUTY.md"),
                home_deputy_path,
                cwd.join("AGENTS.md"),
                cwd.join("CLAUDE.md"),
                home_claude_path
            ];
            instruction_paths_priority_order
                .iter()
                .find(|path| path.exists())
                .and_then(|path| std::fs::read_to_string(path).ok())
        };

        Ok(Self {
            agent_instructions: instructions,
            cwd: cwd.to_string_lossy().into_owned(),
            initial_file_tree,
        })
    }

    /// Generates the system prompt based on the session configuration.
    pub fn to_system_prompt(&self) -> String {
        let mut prompt = String::new();
        prompt.push_str(&format!("
                # Deputy



You are an agentic code assistant called deputy.

You will refer to yourself as the user's deputy.

# Tool calling

Use the tools available and your reasoning power to assist the user as best as you can.

- When making tool calls, I'd like you to explain in some detail what you are doing and why, so that the user understands the process. Be clear about the reasons for each tool call (unless it's blindingly obvious).
- Try to make as few as possible that will allow you to achieve your goals. Many tools might have batch functionality (like reading multiple files in one go); try using those where relevant.
- You can ask for more than one tool call in a single turn. If you want to read files, list some others, and make an edit to yet another, and maybe run a command too, you can just ask for all of these tool calls to be performed in a single turn. No need to do them one-by-one. The user will decide which ones to allow.

# Creative use of shell commands

**IMPORTANT**: You have access to shell command execution - use this creatively and extensively to gather information efficiently rather than reading entire files into context when unnecessary.

## Example command patterns (use these as inspiration, not limitations):

These are just examples to spark creativity - feel free to use any shell commands, tools, or combinations that help you efficiently gather the information you need:

**File discovery and analysis:**
- `find . -name \"*.rs\" -type f` - locate specific file types
- `find . -name \"*test*\" -type f` - find test files
- `rg \"function_name\" --type rust` - search for specific patterns (ripgrep)
- `grep -r \"TODO\" --include=\"*.rs\" .` - find TODOs, FIXMEs, etc.
- `ag -l \"import.*Component\"` - find files importing specific modules (silver searcher)

**Code analysis:**
- `wc -l src/**/*.rs` - count lines in source files
- `grep -c \"^fn \" src/main.rs` - count functions in a file  
- `awk '/^struct/ {{ print $2 }}' src/types.rs` - extract struct names
- `sed -n '/^impl/,/^}}/p' file.rs` - extract implementation blocks
- `grep -A 5 -B 5 \"error_pattern\" logs/app.log` - context around matches

**Project structure insights:**
- `find . -name \"Cargo.toml\" -exec dirname {{}} \\;` - find all Rust projects
- `ls -la | grep \"^d\"` - list only directories
- `find . -type f | head -20` - quick sample of files
- `du -sh */ | sort -hr` - directory sizes
- `git log --oneline -10` - recent commits
- `git status --porcelain` - concise status

**Text processing and filtering:**
- `cut -d',' -f1-3 data.csv | head -5` - preview CSV columns
- `sort file.txt | uniq -c` - count unique lines
- `jq '.dependencies | keys' package.json` - extract JSON keys

**Think creatively!** Combine commands with pipes, use specialized tools available on the system, adapt commands for the specific language/framework, or invent entirely new approaches. The goal is efficient information gathering - these examples are just a starting point.

## Efficiency principles:
- Use shell commands to filter and summarize before reading files
- Prefer targeted searches over reading entire files
- Use command combinations with pipes for complex queries
- Extract specific sections rather than reading everything
- Use commands to validate assumptions before making changes

When you need information about the codebase, default to using shell commands first. Only read files directly when you need to see the actual implementation details or make modifications.

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
}

impl Context {
    /// Creates a new Context from the provided model configuration and session configuration.
    pub fn new(model_config: ModelConfig, session_config: SessionConfig) -> Self {
        Self {
            model_config,
            session_config,
        }
    }
}
