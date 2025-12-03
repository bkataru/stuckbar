# stuckbar

A straightforward CLI tool for getting that annoying Windows taskbar unstuck.

When the Windows taskbar gets stuck (especially when configured to auto-hide), this utility provides a quick way to restart `explorer.exe` from the command line.

## Installation

### From source

```bash
cargo install --path .
```

### Using cargo

```bash
cargo install stuckbar
```

## Usage

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

## Commands

| Command   | Description                              |
|-----------|------------------------------------------|
| `restart` | Kill and restart explorer.exe (default)  |
| `kill`    | Terminate explorer.exe process           |
| `start`   | Start explorer.exe process               |

## License

MIT License - see [LICENSE](LICENSE) for details.
