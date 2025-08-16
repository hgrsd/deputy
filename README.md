# Deputy

A terminal-based AI coding assistant that actually works with your files and shell.

![](assets/e1.png)
![](assets/e2.png)
![](assets/e3.png)

## What it does

Deputy gives you an AI assistant that can:
- Read and write files in your project
- Run shell commands 
- Navigate your codebase intelligently
- Remember what you've approved it to do

No copying and pasting code snippets. No switching between terminal and browser. Just tell it what you want and it gets on with it.

## Installation

```bash
cargo install deputy
```

Set your API key:
```bash
export ANTHROPIC_API_KEY=your_key_here
# or
export OPENAI_API_KEY=your_key_here
```

## Usage

```bash
cd your-project
deputy
```

That's it. Deputy will scan your project and you can start chatting.

### Options

```bash
deputy --provider open-ai --model gpt-4o    # Use OpenAI instead
deputy --yolo                              # Skip permission prompts
deputy --base-url http://localhost:8080/v1 # Custom API endpoint
deputy --config ./my-config.md             # Use custom configuration file
# ollama, you need to set OPENAI_API_KEY to some fake value (not an empty string)
deputy --provider open-ai --base-url http://localhost:11434/v1 --model gpt-oss:20b  
```

## Permissions

Deputy asks before doing potentially destructive things. You can:
- Approve once
- Remember your choice for similar operations
- Use `--yolo` mode to skip prompts entirely

## Configuration

You can specify a custom configuration file using the `--config` option:

```bash
deputy --config ./path/to/my-config.md
```

When using `--config`, Deputy will read ONLY that file and ignore the default search locations.

If no custom config is specified, Deputy loads configuration files in priority order (first found wins):

1. `DEPUTY.md` in your project root
2. `~/.deputy/DEPUTY.md` for global config
3. `AGENTS.md` in your project root
4. `CLAUDE.md` in your project root  
5. `~/.claude/CLAUDE.md` for global config

These files contain instructions that Deputy will follow during your session.

## Contributing

Issues and PRs welcome.

## License

MIT