# Deputy

A command-line AI assistant that acts as your coding deputy. Deputy integrates with Anthropic's Claude API to provide an intelligent assistant with file system access, capable of reading, writing, and executing commands in your project directory.

## Features

- Interactive chat interface
- File system operations (read, write, list files)
- Command execution capabilities
- Contextual awareness of your project structure
- Configurable Claude model selection

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/hgrsd/deputy
   cd deputy
   ```

2. Install using Cargo:
   ```bash
   cargo install --path .
   ```

3. Set up your Anthropic API key:
   ```bash
   export ANTHROPIC_API_KEY=your_api_key_here
   ```
   
   Alternatively, you can set it when running the binary:
   ```bash
   ANTHROPIC_API_KEY=your_api_key_here deputy
   ```

## Usage

Simply run the binary and start chatting with your deputy:

```bash
deputy
```

You can also specify which Claude model to use:

```bash
# Use a specific model
deputy --model claude-opus-4-20250514

# Or use the short flag
deputy -m claude-opus-4-20250514

# See all available options
deputy --help
```

The default model is `claude-sonnet-4-20250514` if none is specified.

Type your commands or questions, and Deputy will assist you with code analysis, file operations, and project management tasks. Type `exit` to quit (or use Ctrl-C).

## License

MIT
