#[cfg(feature = "mcp")]
use clap::Args;
use clap::{Parser, Subcommand};
use colored::Colorize;
use stuckbar::{check_platform, ExplorerManager, SystemProcessRunner};

#[derive(Parser)]
#[command(
    name = "stuckbar",
    about = "A CLI tool for restarting Windows Explorer when the taskbar gets stuck",
    long_about = "A CLI tool for restarting Windows Explorer when the taskbar gets stuck.\n\n\
                  This tool is Windows-only and provides commands to kill, start, or restart \
                  explorer.exe. It also supports running as an MCP (Model Context Protocol) \
                  server for AI agent integration.",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Terminate explorer.exe process
    Kill,
    /// Start explorer.exe process
    Start,
    /// Restart explorer.exe (kill then start)
    Restart,
    /// Start an MCP server for AI agent integration
    #[cfg(feature = "mcp")]
    Serve(ServeArgs),
}

/// Arguments for the serve command
#[cfg(feature = "mcp")]
#[derive(Args, Debug, Clone, PartialEq)]
pub struct ServeArgs {
    /// Use STDIO transport (for direct process communication)
    #[arg(long, group = "transport")]
    pub stdio: bool,

    /// Use HTTP transport (for network-based communication)
    #[cfg(feature = "mcp-http")]
    #[arg(long, group = "transport")]
    pub http: bool,

    /// Host address to bind to (only used with --http)
    #[cfg(feature = "mcp-http")]
    #[arg(long, default_value = "127.0.0.1", requires = "http")]
    pub host: String,

    /// Port number to listen on (only used with --http)
    #[cfg(feature = "mcp-http")]
    #[arg(long, default_value = "8080", requires = "http")]
    pub port: u16,
}

/// Execute the CLI command
fn run_command(command: Option<Commands>) -> bool {
    let manager = ExplorerManager::new(SystemProcessRunner);

    match command {
        Some(Commands::Kill) => manager.kill(),
        Some(Commands::Start) => manager.start(),
        Some(Commands::Restart) => manager.restart(),
        #[cfg(feature = "mcp")]
        Some(Commands::Serve(args)) => {
            run_mcp_server(args);
            true
        }
        None => manager.restart(),
    }
}

/// Run the MCP server with the specified transport
#[cfg(feature = "mcp")]
#[allow(unused_variables)]
fn run_mcp_server(args: ServeArgs) {
    use tokio::runtime::Runtime;

    let rt = Runtime::new().expect("Failed to create Tokio runtime");

    rt.block_on(async {
        #[cfg(feature = "mcp-http")]
        if args.http {
            if let Err(e) = stuckbar::mcp::run_http_server(&args.host, args.port).await {
                eprintln!("{} {}", "MCP HTTP server error:".red(), e);
                std::process::exit(1);
            }
            return;
        }

        // Default to STDIO if no transport specified or --stdio flag used
        if let Err(e) = stuckbar::mcp::run_stdio_server().await {
            eprintln!("{} {}", "MCP STDIO server error:".red(), e);
            std::process::exit(1);
        }
    });
}

fn main() {
    // Check platform before doing anything
    if let Err(e) = check_platform() {
        eprintln!("{}", e.red().bold());
        std::process::exit(1);
    }

    let cli = Cli::parse();

    let success = run_command(cli.command);

    if !success {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // CLI parsing tests
    #[test]
    fn test_cli_parse_no_args() {
        let cli = Cli::parse_from(["stuckbar"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_parse_kill() {
        let cli = Cli::parse_from(["stuckbar", "kill"]);
        assert_eq!(cli.command, Some(Commands::Kill));
    }

    #[test]
    fn test_cli_parse_start() {
        let cli = Cli::parse_from(["stuckbar", "start"]);
        assert_eq!(cli.command, Some(Commands::Start));
    }

    #[test]
    fn test_cli_parse_restart() {
        let cli = Cli::parse_from(["stuckbar", "restart"]);
        assert_eq!(cli.command, Some(Commands::Restart));
    }

    #[test]
    fn test_cli_version_flag() {
        let result = Cli::try_parse_from(["stuckbar", "--version"]);
        assert!(result.is_err()); // clap returns Err for --version
    }

    #[test]
    fn test_cli_help_flag() {
        let result = Cli::try_parse_from(["stuckbar", "--help"]);
        assert!(result.is_err()); // clap returns Err for --help
    }

    #[test]
    fn test_cli_invalid_command() {
        let result = Cli::try_parse_from(["stuckbar", "invalid"]);
        assert!(result.is_err());
    }

    // Commands enum tests
    #[test]
    fn test_commands_equality() {
        assert_eq!(Commands::Kill, Commands::Kill);
        assert_eq!(Commands::Start, Commands::Start);
        assert_eq!(Commands::Restart, Commands::Restart);
        assert_ne!(Commands::Kill, Commands::Start);
    }

    #[test]
    fn test_commands_clone() {
        let cmd = Commands::Restart;
        let cloned = cmd.clone();
        assert_eq!(cmd, cloned);
    }

    // MCP serve command tests
    #[cfg(feature = "mcp")]
    #[test]
    fn test_cli_parse_serve_stdio() {
        let cli = Cli::parse_from(["stuckbar", "serve", "--stdio"]);
        match cli.command {
            Some(Commands::Serve(args)) => {
                assert!(args.stdio);
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[cfg(all(feature = "mcp", feature = "mcp-http"))]
    #[test]
    fn test_cli_parse_serve_http() {
        let cli = Cli::parse_from(["stuckbar", "serve", "--http"]);
        match cli.command {
            Some(Commands::Serve(args)) => {
                assert!(args.http);
                assert_eq!(args.host, "127.0.0.1");
                assert_eq!(args.port, 8080);
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[cfg(all(feature = "mcp", feature = "mcp-http"))]
    #[test]
    fn test_cli_parse_serve_http_custom_port() {
        let cli = Cli::parse_from(["stuckbar", "serve", "--http", "--port", "3000"]);
        match cli.command {
            Some(Commands::Serve(args)) => {
                assert!(args.http);
                assert_eq!(args.port, 3000);
            }
            _ => panic!("Expected Serve command"),
        }
    }

    #[cfg(all(feature = "mcp", feature = "mcp-http"))]
    #[test]
    fn test_cli_parse_serve_http_custom_host_port() {
        let cli = Cli::parse_from([
            "stuckbar", "serve", "--http", "--host", "0.0.0.0", "--port", "9000",
        ]);
        match cli.command {
            Some(Commands::Serve(args)) => {
                assert!(args.http);
                assert_eq!(args.host, "0.0.0.0");
                assert_eq!(args.port, 9000);
            }
            _ => panic!("Expected Serve command"),
        }
    }
}
