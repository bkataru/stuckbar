# stuckbar

A straightforward CLI tool for getting that annoying Windows taskbar unstuck.

When the Windows taskbar gets stuck (especially when configured to auto-hide), this utility provides a quick way to restart `explorer.exe` from the command line, which snaps the taskbar back to normal.

> **⚠️ Windows Only**: This tool is designed specifically for Windows and will not work on macOS or Linux.

## Installation

### Using cargo (recommended)

```bash
# Basic installation (CLI only)
cargo install stuckbar

# With MCP server support (STDIO transport)
cargo install stuckbar --features mcp

# With full MCP support (STDIO + HTTP transports)
cargo install stuckbar --features mcp-full
```

### From source

```bash
git clone https://github.com/bkataru/stuckbar.git
cd stuckbar
cargo install --path .

# Or with MCP features
cargo install --path . --features mcp-full
```

## Usage

### Basic CLI Usage

```bash
# Restart explorer.exe (default action)
stuckbar

# Or explicitly use the restart command
stuckbar restart

# Just kill explorer.exe
stuckbar kill

# Just start explorer.exe
stuckbar start

# Show help
stuckbar --help

# Show version
stuckbar --version
```

### Commands

| Command   | Description                              |
|-----------|------------------------------------------|
| `restart` | Kill and restart explorer.exe (default)  |
| `kill`    | Terminate explorer.exe process           |
| `start`   | Start explorer.exe process               |
| `serve`   | Start MCP server (requires `mcp` feature)|

## MCP Server (AI Agent Integration)

Stuckbar can run as a [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server, allowing AI agents to programmatically control Windows Explorer operations.

### Features

When running as an MCP server, stuckbar exposes three tools:

| Tool               | Description                                           |
|--------------------|-------------------------------------------------------|
| `kill_explorer`    | Terminate the explorer.exe process                    |
| `start_explorer`   | Start the explorer.exe process                        |
| `restart_explorer` | Restart explorer.exe (recommended for stuck taskbar)  |

### Running the MCP Server

#### STDIO Transport (for direct process communication)

```bash
# Requires: cargo install stuckbar --features mcp
stuckbar serve --stdio
```

This is the recommended transport for most MCP clients like Claude Desktop.

#### HTTP Transport (SSE-based, for network communication)

```bash
# Requires: cargo install stuckbar --features mcp-full
stuckbar serve --http

# With custom host and port
stuckbar serve --http --host 0.0.0.0 --port 3000
```

The HTTP server uses Server-Sent Events (SSE) and exposes:
- SSE endpoint: `http://<host>:<port>/sse`
- Message endpoint: `http://<host>:<port>/message`

### Configuration Examples

#### Claude Desktop

Add to your `claude_desktop_config.json`:

**Windows:**
```json
{
  "mcpServers": {
    "stuckbar": {
      "command": "stuckbar",
      "args": ["serve", "--stdio"]
    }
  }
}
```

Or with full path:
```json
{
  "mcpServers": {
    "stuckbar": {
      "command": "C:\\Users\\<username>\\.cargo\\bin\\stuckbar.exe",
      "args": ["serve", "--stdio"]
    }
  }
}
```

#### Other MCP Clients

For clients that support SSE transport:
```bash
stuckbar serve --http --port 8080
# Connect to: http://localhost:8080/sse
```

## Feature Flags

| Feature    | Description                                          |
|------------|------------------------------------------------------|
| (default)  | Basic CLI functionality                              |
| `mcp`      | MCP server with STDIO transport                      |
| `mcp-http` | MCP server with SSE HTTP transport (includes `mcp`)  |
| `mcp-full` | All MCP features (alias for `mcp-http`)              |

## Building from Source

```bash
# Clone the repository
git clone https://github.com/bkataru/stuckbar.git
cd stuckbar

# Build with default features
cargo build --release

# Build with MCP support
cargo build --release --features mcp-full

# Run tests
cargo test

# Run tests with all features
cargo test --all-features
```

## How It Works

When the Windows taskbar becomes unresponsive or stuck (often happens with auto-hide enabled), the typical fix is to restart Windows Explorer. This tool automates that process by:

1. **Kill**: Forcefully terminating `explorer.exe` using `taskkill /F /IM explorer.exe`
2. **Wait**: Pausing briefly (500ms) to ensure the process is fully terminated
3. **Start**: Launching a new instance of `explorer.exe`

This restores the taskbar, desktop icons, and file explorer functionality.

## Platform Support

This tool is **Windows-only**. Running on macOS or Linux will display an error message:

```
stuckbar is a Windows-only tool.
Current platform 'linux' is not supported.
This tool restarts explorer.exe which only exists on Windows.
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request :)
