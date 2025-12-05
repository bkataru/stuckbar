//! # stuckbar
//!
//! A CLI tool for restarting Windows Explorer when the taskbar gets stuck.
//!
//! This crate provides functionality to kill, start, and restart the Windows Explorer
//! process, which is useful when the Windows taskbar becomes unresponsive.
//!
//! ## Features
//!
//! - `mcp` - Enable Model Context Protocol (MCP) server support for AI agent integration
//!
//! ## Platform Support
//!
//! This tool is Windows-only. Running on other platforms will result in an error.

use colored::Colorize;
use std::process::Command;

/// Delay in milliseconds before starting explorer.exe after termination
pub const RESTART_DELAY_MS: u64 = 500;

/// Result of a process operation
#[derive(Debug, PartialEq, Clone)]
pub struct ProcessResult {
    pub success: bool,
    pub message: String,
}

impl ProcessResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
        }
    }
}

/// Trait for abstracting process operations (enables testing)
pub trait ProcessRunner {
    fn kill_process(&self, process_name: &str) -> ProcessResult;
    fn start_process(&self, process_name: &str) -> ProcessResult;
    fn sleep_ms(&self, ms: u64);
}

/// Real implementation that interacts with the system
pub struct SystemProcessRunner;

impl ProcessRunner for SystemProcessRunner {
    fn kill_process(&self, process_name: &str) -> ProcessResult {
        let result = Command::new("taskkill")
            .args(["/F", "/IM", process_name])
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    ProcessResult::success(format!("Successfully terminated {}", process_name))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    ProcessResult::failure(format!(
                        "Failed to terminate {}: {}",
                        process_name, stderr
                    ))
                }
            }
            Err(e) => ProcessResult::failure(format!("Error executing taskkill: {}", e)),
        }
    }

    fn start_process(&self, process_name: &str) -> ProcessResult {
        let result = Command::new(process_name).spawn();

        match result {
            Ok(_) => ProcessResult::success(format!("Successfully started {}", process_name)),
            Err(e) => ProcessResult::failure(format!("Error starting {}: {}", process_name, e)),
        }
    }

    fn sleep_ms(&self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}

/// Explorer manager that handles explorer.exe operations
pub struct ExplorerManager<R: ProcessRunner> {
    pub runner: R,
    pub restart_delay_ms: u64,
}

impl<R: ProcessRunner> ExplorerManager<R> {
    pub fn new(runner: R) -> Self {
        Self {
            runner,
            restart_delay_ms: RESTART_DELAY_MS,
        }
    }

    pub fn with_restart_delay(mut self, delay_ms: u64) -> Self {
        self.restart_delay_ms = delay_ms;
        self
    }

    /// Kill explorer.exe process
    pub fn kill(&self) -> bool {
        println!("{}", "Terminating explorer.exe...".yellow());
        let result = self.runner.kill_process("explorer.exe");

        if result.success {
            println!("{}", result.message.green());
        } else {
            eprintln!("{}", result.message.red());
        }

        result.success
    }

    /// Start explorer.exe process
    pub fn start(&self) -> bool {
        println!("{}", "Starting explorer.exe...".yellow());
        let result = self.runner.start_process("explorer.exe");

        if result.success {
            println!("{}", result.message.green());
        } else {
            eprintln!("{}", result.message.red());
        }

        result.success
    }

    /// Restart explorer.exe (kill then start)
    pub fn restart(&self) -> bool {
        println!("{}", "Restarting explorer.exe...".cyan().bold());

        if !self.kill() {
            return false;
        }

        // Small delay to ensure explorer is fully terminated
        self.runner.sleep_ms(self.restart_delay_ms);

        if !self.start() {
            return false;
        }

        println!("{}", "Explorer.exe restarted successfully!".green().bold());
        true
    }

    /// Kill explorer.exe without printing (for MCP/programmatic use)
    pub fn kill_silent(&self) -> ProcessResult {
        self.runner.kill_process("explorer.exe")
    }

    /// Start explorer.exe without printing (for MCP/programmatic use)
    pub fn start_silent(&self) -> ProcessResult {
        self.runner.start_process("explorer.exe")
    }

    /// Restart explorer.exe without printing (for MCP/programmatic use)
    pub fn restart_silent(&self) -> ProcessResult {
        let kill_result = self.runner.kill_process("explorer.exe");
        if !kill_result.success {
            return kill_result;
        }

        self.runner.sleep_ms(self.restart_delay_ms);

        let start_result = self.runner.start_process("explorer.exe");
        if !start_result.success {
            return start_result;
        }

        ProcessResult::success("Explorer.exe restarted successfully")
    }
}

/// Check if the current platform is Windows
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// Check platform and return an error message if not Windows
pub fn check_platform() -> Result<(), String> {
    if !is_windows() {
        Err(format!(
            "stuckbar is a Windows-only tool.\n\
            Current platform '{}' is not supported.\n\
            This tool restarts explorer.exe which only exists on Windows.",
            std::env::consts::OS
        ))
    } else {
        Ok(())
    }
}

#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Mock process runner for testing
    pub struct MockProcessRunner {
        kill_results: RefCell<Vec<ProcessResult>>,
        start_results: RefCell<Vec<ProcessResult>>,
        sleep_calls: RefCell<Vec<u64>>,
    }

    impl MockProcessRunner {
        pub fn new() -> Self {
            Self {
                kill_results: RefCell::new(Vec::new()),
                start_results: RefCell::new(Vec::new()),
                sleep_calls: RefCell::new(Vec::new()),
            }
        }

        pub fn with_kill_result(self, result: ProcessResult) -> Self {
            self.kill_results.borrow_mut().push(result);
            self
        }

        pub fn with_start_result(self, result: ProcessResult) -> Self {
            self.start_results.borrow_mut().push(result);
            self
        }

        pub fn get_sleep_calls(&self) -> Vec<u64> {
            self.sleep_calls.borrow().clone()
        }
    }

    impl Default for MockProcessRunner {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ProcessRunner for MockProcessRunner {
        fn kill_process(&self, _process_name: &str) -> ProcessResult {
            self.kill_results
                .borrow_mut()
                .pop()
                .unwrap_or_else(|| ProcessResult::failure("No mock result configured"))
        }

        fn start_process(&self, _process_name: &str) -> ProcessResult {
            self.start_results
                .borrow_mut()
                .pop()
                .unwrap_or_else(|| ProcessResult::failure("No mock result configured"))
        }

        fn sleep_ms(&self, ms: u64) {
            self.sleep_calls.borrow_mut().push(ms);
        }
    }

    // ProcessResult tests
    #[test]
    fn test_process_result_success() {
        let result = ProcessResult::success("test message");
        assert!(result.success);
        assert_eq!(result.message, "test message");
    }

    #[test]
    fn test_process_result_failure() {
        let result = ProcessResult::failure("error message");
        assert!(!result.success);
        assert_eq!(result.message, "error message");
    }

    #[test]
    fn test_process_result_clone() {
        let result = ProcessResult::success("test");
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    // ExplorerManager::kill tests
    #[test]
    fn test_kill_success() {
        let runner = MockProcessRunner::new().with_kill_result(ProcessResult::success("Killed"));
        let manager = ExplorerManager::new(runner);

        assert!(manager.kill());
    }

    #[test]
    fn test_kill_failure() {
        let runner =
            MockProcessRunner::new().with_kill_result(ProcessResult::failure("Failed to kill"));
        let manager = ExplorerManager::new(runner);

        assert!(!manager.kill());
    }

    // ExplorerManager::start tests
    #[test]
    fn test_start_success() {
        let runner = MockProcessRunner::new().with_start_result(ProcessResult::success("Started"));
        let manager = ExplorerManager::new(runner);

        assert!(manager.start());
    }

    #[test]
    fn test_start_failure() {
        let runner =
            MockProcessRunner::new().with_start_result(ProcessResult::failure("Failed to start"));
        let manager = ExplorerManager::new(runner);

        assert!(!manager.start());
    }

    // ExplorerManager::restart tests
    #[test]
    fn test_restart_success() {
        let runner = MockProcessRunner::new()
            .with_kill_result(ProcessResult::success("Killed"))
            .with_start_result(ProcessResult::success("Started"));
        let manager = ExplorerManager::new(runner).with_restart_delay(100);

        assert!(manager.restart());
    }

    #[test]
    fn test_restart_kill_fails() {
        let runner =
            MockProcessRunner::new().with_kill_result(ProcessResult::failure("Failed to kill"));
        let manager = ExplorerManager::new(runner);

        assert!(!manager.restart());
    }

    #[test]
    fn test_restart_start_fails() {
        let runner = MockProcessRunner::new()
            .with_kill_result(ProcessResult::success("Killed"))
            .with_start_result(ProcessResult::failure("Failed to start"));
        let manager = ExplorerManager::new(runner);

        assert!(!manager.restart());
    }

    #[test]
    fn test_restart_sleeps_between_operations() {
        let runner = MockProcessRunner::new()
            .with_kill_result(ProcessResult::success("Killed"))
            .with_start_result(ProcessResult::success("Started"));
        let manager = ExplorerManager::new(runner).with_restart_delay(250);

        manager.restart();

        let sleep_calls = &manager.runner.get_sleep_calls();
        assert_eq!(sleep_calls.len(), 1);
        assert_eq!(sleep_calls[0], 250);
    }

    #[test]
    fn test_restart_uses_default_delay() {
        let runner = MockProcessRunner::new()
            .with_kill_result(ProcessResult::success("Killed"))
            .with_start_result(ProcessResult::success("Started"));
        let manager = ExplorerManager::new(runner);

        assert_eq!(manager.restart_delay_ms, RESTART_DELAY_MS);
    }

    // ExplorerManager builder pattern test
    #[test]
    fn test_explorer_manager_with_restart_delay() {
        let runner = MockProcessRunner::new();
        let manager = ExplorerManager::new(runner).with_restart_delay(1000);
        assert_eq!(manager.restart_delay_ms, 1000);
    }

    // Silent method tests
    #[test]
    fn test_kill_silent_success() {
        let runner = MockProcessRunner::new().with_kill_result(ProcessResult::success("Killed"));
        let manager = ExplorerManager::new(runner);

        let result = manager.kill_silent();
        assert!(result.success);
    }

    #[test]
    fn test_kill_silent_failure() {
        let runner = MockProcessRunner::new().with_kill_result(ProcessResult::failure("Error"));
        let manager = ExplorerManager::new(runner);

        let result = manager.kill_silent();
        assert!(!result.success);
    }

    #[test]
    fn test_start_silent_success() {
        let runner = MockProcessRunner::new().with_start_result(ProcessResult::success("Started"));
        let manager = ExplorerManager::new(runner);

        let result = manager.start_silent();
        assert!(result.success);
    }

    #[test]
    fn test_start_silent_failure() {
        let runner = MockProcessRunner::new().with_start_result(ProcessResult::failure("Error"));
        let manager = ExplorerManager::new(runner);

        let result = manager.start_silent();
        assert!(!result.success);
    }

    #[test]
    fn test_restart_silent_success() {
        let runner = MockProcessRunner::new()
            .with_kill_result(ProcessResult::success("Killed"))
            .with_start_result(ProcessResult::success("Started"));
        let manager = ExplorerManager::new(runner);

        let result = manager.restart_silent();
        assert!(result.success);
        assert_eq!(result.message, "Explorer.exe restarted successfully");
    }

    #[test]
    fn test_restart_silent_kill_fails() {
        let runner =
            MockProcessRunner::new().with_kill_result(ProcessResult::failure("Kill failed"));
        let manager = ExplorerManager::new(runner);

        let result = manager.restart_silent();
        assert!(!result.success);
        assert_eq!(result.message, "Kill failed");
    }

    #[test]
    fn test_restart_silent_start_fails() {
        let runner = MockProcessRunner::new()
            .with_kill_result(ProcessResult::success("Killed"))
            .with_start_result(ProcessResult::failure("Start failed"));
        let manager = ExplorerManager::new(runner);

        let result = manager.restart_silent();
        assert!(!result.success);
        assert_eq!(result.message, "Start failed");
    }

    // Platform check tests
    #[test]
    fn test_is_windows() {
        // This test verifies the function works, actual result depends on platform
        let result = is_windows();
        #[cfg(target_os = "windows")]
        assert!(result);
        #[cfg(not(target_os = "windows"))]
        assert!(!result);
    }

    #[test]
    fn test_check_platform() {
        let result = check_platform();
        #[cfg(target_os = "windows")]
        assert!(result.is_ok());
        #[cfg(not(target_os = "windows"))]
        {
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.contains("Windows-only"));
            assert!(err.contains(std::env::consts::OS));
        }
    }
}
