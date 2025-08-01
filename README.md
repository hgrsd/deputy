# Deputy

Deputy is an experimental terminal-based AI coding assistant designed to explore the practical application of agentic LLM systems within development workflows. Rather than offering another chat interface, Deputy integrates directly with your filesystem and shell environment, providing an assistant capable of reading code, manipulating files, executing commands, and navigating complex project structures.

![](assets/e1.png)
![](assets/e2.png)
![](assets/e3.png)

This project represents active research into how agentic systems can meaningfully augment developer productivity. Whilst functional and genuinely useful, Deputy remains experimental and continues evolving towards a more robust, production-ready assistant. The current implementation deliberately prioritises exploration over polish, and contributions or issues are welcomed for collaborative development.

## Architecture

Deputy employs a modular architecture built around three core concepts: providers, tools, and sessions. Providers abstract different AI services (currently Anthropic's Claude and OpenAI), tools define the operations Deputy can perform within your environment, and sessions manage the conversation state and permission model.

The system initialises by scanning your project directory, generating a contextual file tree, and optionally loading agent instructions from configuration files. Each interaction flows through a permission system that can prompt for approval, remember decisions, or operate autonomously in YOLO mode.

## Core Capabilities

Deputy's functionality centres on four fundamental tools that enable comprehensive project interaction. The file listing tool provides directory traversal with git-aware filtering and recursive exploration. File reading supports selective content extraction with line limiting and offset capabilities, allowing targeted examination of large codebases without overwhelming context windows.

File writing operations include both complete file creation and selective range editing, with built-in diff generation for change visibility. Command execution provides full shell access, enabling build processes, test execution, git operations, and arbitrary system commands within your project context.

The permission system governs all tool usage through three modes: explicit approval for each operation, persistent approval for repeated similar operations, and autonomous execution in YOLO mode. This approach balances safety with efficiency, allowing users to establish trust boundaries appropriate to their workflow.

## Configuration

Deputy searches for agent instruction files in a predetermined hierarchy, checking first for `DEPUTY.md` in your current directory, then `~/.deputy/DEPUTY.md`, followed by `AGENTS.md`, `CLAUDE.md`, and `~/.claude/CLAUDE.md`. These files allow customisation of Deputy's behaviour and integration of project-specific guidance.

The system prompt incorporates comprehensive guidance for creative shell command usage, encouraging efficient information gathering through targeted searches, command composition, and selective file examination rather than wholesale content ingestion.

## Installation

Deputy requires Rust and an API key for your chosen provider (Anthropic or OpenAI). Installation proceeds through Cargo:

```bash
git clone https://github.com/hgrsd/deputy
cd deputy
cargo install --path .
```

Alternatively, install directly from crates.io:

```bash
cargo install deputy
```

Configure your API key through environment variables:

```bash
# For Anthropic
export ANTHROPIC_API_KEY=your_anthropic_api_key_here

# For OpenAI
export OPENAI_API_KEY=your_openai_api_key_here
```

## Usage

Navigate to any project directory and invoke Deputy:

```bash
cd your-project
deputy
```

The default configuration employs Anthropic's `claude-sonnet-4-20250514` model. Alternative configurations include provider and model specification:

```bash
# Using Anthropic
deputy --provider anthropic --model claude-opus-4-20250514
deputy -p anthropic -m claude-opus-4-20250514

# Using OpenAI
deputy --provider openai --model gpt-4o
deputy -p openai -m gpt-4o-mini
```

### Custom API Endpoints

Deputy supports overriding the default API base URLs, enabling integration with local or third-party OpenAI/Anthropic compatible APIs:

```bash
# Using a local OpenAI-compatible API
deputy --provider openai --base-url http://localhost:8080/v1

# Using a third-party Anthropic-compatible service
deputy --provider anthropic --base-url https://custom-api.example.com/v1

# Short form
deputy -p openai -b http://localhost:8080/v1
```

This feature is particularly useful for:
- Local development with self-hosted models
- Integration with custom API gateways
- Testing against alternative service providers
- Corporate environments with proxied API access

### YOLO Mode

YOLO mode eliminates permission prompts, executing all tool calls automatically:

```bash
deputy --yolo
```

This mode significantly accelerates interaction but requires careful consideration of the security implications.

### Debug Mode

Debug mode provides detailed logging of tool calls and results:

```bash
DEPUTY_DEBUG=true deputy
```

## Session Management

Deputy maintains conversation history and permission state throughout each session. The permission system learns from your approval patterns, offering to remember decisions for similar operations. This creates an adaptive workflow where frequently-used operations become frictionless whilst maintaining oversight for novel or potentially destructive actions.

Sessions terminate through the `exit` command or interrupt signal. Each session operates independently, with no persistence between invocations beyond the static configuration files.

## Contributing

Yes please! Please feel free to open issues, fork this repo, or open PRs. All contributions and collaborations are welcome.

## License

MIT