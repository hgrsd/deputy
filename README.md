# Deputy

An experimental AI coding assistant that works directly in your terminal. Deputy is a research project exploring how agentic LLM systems can integrate with development workflows, providing an assistant that can read your code, write files, execute commands, and help you navigate complex projects.

Deputy is currently experimental and built to understand how agentic LLM systems work in practice. While functional and useful, it's actively evolving toward a more robust, production-quality assistant.

## The Experiment

This project explores how AI assistants can move beyond simple chat interfaces to become true development partners. Instead of copying and pasting code snippets, Deputy works directly in your project directory and can:

- **Understand your entire codebase** by reading files and exploring project structure
- **Make actual changes** by writing and editing files for you
- **Execute commands** to test, build, or analyze your project
- **Maintain context** across your entire conversation, remembering what you've discussed

The aim is to understand what makes an AI assistant truly helpful for developers and how to build systems that can reason about code, understand project context, and take meaningful actions.

## Current Capabilities

### Core Features
- **Interactive chat interface** - Natural conversation about your code
- **File system operations** - Read, write, and manage files across your project
- **Command execution** - Run tests, build scripts, git commands, and more
- **Project awareness** - Understands your project structure and dependencies

### AI Integration
- **Multiple provider support** - Designed to work with different AI providers (currently Anthropic)
- **Model selection** - Choose the right model for your task
- **Configurable behavior** - Experiment with different approaches

## Installation

1. **Clone and install:**
   ```bash
   git clone https://github.com/hgrsd/deputy
   cd deputy
   cargo install --path .
   ```

2. **Set up your API key:**
   ```bash
   export ANTHROPIC_API_KEY=your_api_key_here
   ```
   *(Currently supports Anthropic - more providers planned)*

## Getting Started

Navigate to any project directory and start Deputy:

```bash
cd your-project
deputy
```

Try asking Deputy to:
- "Can you explain what this main.rs file does?"
- "Help me add error handling to this function"
- "Write a test for this module"
- "Refactor this code to be more readable"

Type `exit` or press Ctrl-C to quit.

## Usage Options

```bash
# Basic usage
deputy

# Specify provider (currently: anthropic)
deputy --provider anthropic

# Specify model (default: claude-sonnet-4-20250514)
deputy --model claude-opus-4-20250514

# Combine options
deputy --provider anthropic --model claude-opus-4-20250514

# Use short flags
deputy -p anthropic -m claude-opus-4-20250514

# See all options
deputy --help
```

## Current Status

As an experimental project, Deputy is actively being developed. Current areas of focus include:
- Adding support for more AI providers
- Better error handling and recovery
- Enhanced configuration options

## Contributing

Contributions are welcome! Whether you're interested in adding new AI providers, improving the conversation flow, better error handling, performance optimizations, or documentation improvements, feel free to open issues or submit pull requests.

## License

MIT
