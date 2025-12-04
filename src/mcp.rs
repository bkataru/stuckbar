//! # MCP Server Module
//!
//! This module provides Model Context Protocol (MCP) server support for stuckbar,
//! allowing AI agents to control Windows Explorer operations.
//!
//! ## Features
//!
//! The MCP server exposes three tools:
//! - `kill_explorer` - Terminate the explorer.exe process
//! - `start_explorer` - Start the explorer.exe process
//! - `restart_explorer` - Restart explorer.exe (kill then start)
//!
//! ## Transport Options
//!
//! Two transport modes are supported:
//! - **STDIO** - Standard input/output transport for direct process communication
//! - **HTTP** - SSE (Server-Sent Events) HTTP transport for network-based communication
//!
//! ## Usage
//!
//! ```bash
//! # Start MCP server with STDIO transport
//! stuckbar serve --stdio
//!
//! # Start MCP server with HTTP transport on default port (8080)
//! stuckbar serve --http
//!
//! # Start MCP server with HTTP transport on custom port
//! stuckbar serve --http --port 3000
//!
//! # Start MCP server with HTTP on custom host and port
//! stuckbar serve --http --host 0.0.0.0 --port 8080
//! ```

use crate::{ExplorerManager, SystemProcessRunner, check_platform};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt, handler::server::router::tool::ToolRouter,
    model::*, tool, tool_handler, tool_router, transport::stdio,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// MCP Server for stuckbar operations
///
/// This server exposes Windows Explorer management tools to MCP clients,
/// enabling AI agents to fix stuck taskbars programmatically.
#[derive(Clone)]
pub struct StuckbarMcpServer {
    /// Thread-safe reference to the explorer manager
    manager: Arc<Mutex<ExplorerManager<SystemProcessRunner>>>,
    /// Tool router for handling MCP tool calls
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl StuckbarMcpServer {
    /// Create a new MCP server instance
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(ExplorerManager::new(SystemProcessRunner))),
            tool_router: Self::tool_router(),
        }
    }

    /// Kill the Windows Explorer process
    ///
    /// Forcefully terminates explorer.exe, which will cause the taskbar,
    /// desktop icons, and file explorer windows to disappear temporarily.
    #[tool(
        description = "Terminate the Windows Explorer (explorer.exe) process. This will cause the taskbar and desktop to temporarily disappear. Use this when you need to forcefully stop explorer."
    )]
    async fn kill_explorer(&self) -> Result<CallToolResult, McpError> {
        // Check platform first
        if let Err(e) = check_platform() {
            return Ok(CallToolResult::error(vec![Content::text(e)]));
        }

        let manager = self.manager.lock().await;
        let result = manager.kill_silent();

        if result.success {
            Ok(CallToolResult::success(vec![Content::text(result.message)]))
        } else {
            Ok(CallToolResult::error(vec![Content::text(result.message)]))
        }
    }

    /// Start the Windows Explorer process
    ///
    /// Launches explorer.exe, which will restore the taskbar, desktop icons,
    /// and enable file explorer functionality.
    #[tool(
        description = "Start the Windows Explorer (explorer.exe) process. This will restore the taskbar and desktop. Use this after killing explorer or if explorer is not running."
    )]
    async fn start_explorer(&self) -> Result<CallToolResult, McpError> {
        // Check platform first
        if let Err(e) = check_platform() {
            return Ok(CallToolResult::error(vec![Content::text(e)]));
        }

        let manager = self.manager.lock().await;
        let result = manager.start_silent();

        if result.success {
            Ok(CallToolResult::success(vec![Content::text(result.message)]))
        } else {
            Ok(CallToolResult::error(vec![Content::text(result.message)]))
        }
    }

    /// Restart the Windows Explorer process
    ///
    /// Kills and then restarts explorer.exe with a small delay between operations.
    /// This is the recommended action for fixing a stuck taskbar.
    #[tool(
        description = "Restart Windows Explorer (explorer.exe) by killing and restarting it. This is the recommended fix for a stuck or unresponsive Windows taskbar. The operation includes a brief delay between kill and start."
    )]
    async fn restart_explorer(&self) -> Result<CallToolResult, McpError> {
        // Check platform first
        if let Err(e) = check_platform() {
            return Ok(CallToolResult::error(vec![Content::text(e)]));
        }

        let manager = self.manager.lock().await;
        let result = manager.restart_silent();

        if result.success {
            Ok(CallToolResult::success(vec![Content::text(result.message)]))
        } else {
            Ok(CallToolResult::error(vec![Content::text(result.message)]))
        }
    }
}

impl Default for StuckbarMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[tool_handler]
impl ServerHandler for StuckbarMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "stuckbar".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                website_url: None,
                icons: None,
            },
            instructions: Some(
                "Stuckbar MCP Server - A tool for managing Windows Explorer.\n\n\
                Available tools:\n\
                - kill_explorer: Terminate explorer.exe\n\
                - start_explorer: Start explorer.exe\n\
                - restart_explorer: Restart explorer.exe (recommended for stuck taskbar)\n\n\
                Use 'restart_explorer' to fix a stuck or unresponsive Windows taskbar."
                    .to_string(),
            ),
        }
    }
}

/// Run the MCP server with STDIO transport
///
/// This function starts the MCP server using standard input/output for communication.
/// It blocks until the server is shut down.
///
/// # Errors
///
/// Returns an error if the server fails to start or encounters a runtime error.
pub async fn run_stdio_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = StuckbarMcpServer::new();
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// Run the MCP server with SSE (Server-Sent Events) HTTP transport
///
/// This function starts the MCP server using SSE over HTTP for communication.
/// It binds to the specified host and port.
///
/// # Arguments
///
/// * `host` - The host address to bind to (e.g., "127.0.0.1" or "0.0.0.0")
/// * `port` - The port number to listen on
///
/// # Errors
///
/// Returns an error if the server fails to start or encounters a runtime error.
#[cfg(feature = "mcp-http")]
pub async fn run_http_server(
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use rmcp::transport::sse_server::{SseServer, SseServerConfig};

    let bind_addr = format!("{}:{}", host, port);
    let config = SseServerConfig {
        bind: bind_addr.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: tokio_util::sync::CancellationToken::new(),
        sse_keep_alive: None,
    };

    eprintln!("Starting stuckbar MCP server on http://{}/sse", bind_addr);
    eprintln!("Press Ctrl+C to stop the server");

    let (sse_server, router) = SseServer::new(config);
    let listener = tokio::net::TcpListener::bind(sse_server.config.bind).await?;
    let ct = sse_server.config.ct.child_token();

    let axum_server = axum::serve(listener, router).with_graceful_shutdown(async move {
        ct.cancelled().await;
    });

    tokio::spawn(async move {
        if let Err(e) = axum_server.await {
            eprintln!("HTTP server error: {}", e);
        }
    });

    let ct = sse_server.with_service(StuckbarMcpServer::new);

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    eprintln!("\nShutting down...");
    ct.cancel();

    Ok(())
}

/// Configuration for the MCP SSE HTTP server
#[cfg(feature = "mcp-http")]
#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    /// Host address to bind to
    pub host: String,
    /// Port number to listen on
    pub port: u16,
    /// Path for SSE endpoint (default: "/sse")
    pub sse_path: String,
    /// Path for message POST endpoint (default: "/message")
    pub post_path: String,
}

#[cfg(feature = "mcp-http")]
impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            sse_path: "/sse".to_string(),
            post_path: "/message".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let server = StuckbarMcpServer::new();
        let info = server.get_info();

        assert_eq!(info.server_info.name, "stuckbar");
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("restart_explorer"));
    }

    #[test]
    fn test_server_default() {
        let server = StuckbarMcpServer::default();
        let info = server.get_info();

        assert_eq!(info.server_info.name, "stuckbar");
    }

    #[test]
    fn test_server_info_version() {
        let server = StuckbarMcpServer::new();
        let info = server.get_info();

        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_server_capabilities() {
        let server = StuckbarMcpServer::new();
        let info = server.get_info();

        // Server should have tools capability enabled
        assert!(info.capabilities.tools.is_some());
    }

    #[cfg(feature = "mcp-http")]
    #[test]
    fn test_http_server_config_default() {
        let config = HttpServerConfig::default();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
    }
}
