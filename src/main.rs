use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::Command;

/// Delay in milliseconds before starting explorer.exe after termination
const RESTART_DELAY_MS: u64 = 500;

#[derive(Parser)]
#[command(
    name = "stuckbar",
    about = "A CLI tool for restarting Windows Explorer when the taskbar gets stuck",
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
}

/// Result of a process operation
#[derive(Debug, PartialEq)]
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
    runner: R,
    restart_delay_ms: u64,
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
}

/// Execute the CLI command with a given process runner
pub fn run_with_runner<R: ProcessRunner>(command: Option<Commands>, runner: R) -> bool {
    let manager = ExplorerManager::new(runner);

    match command {
        Some(Commands::Kill) => manager.kill(),
        Some(Commands::Start) => manager.start(),
        Some(Commands::Restart) => manager.restart(),
        None => manager.restart(),
    }
}

fn main() {
    let cli = Cli::parse();
    let runner = SystemProcessRunner;

    let success = run_with_runner(cli.command, runner);

    if !success {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Mock process runner for testing
    struct MockProcessRunner {
        kill_results: RefCell<Vec<ProcessResult>>,
        start_results: RefCell<Vec<ProcessResult>>,
        sleep_calls: RefCell<Vec<u64>>,
    }

    impl MockProcessRunner {
        fn new() -> Self {
            Self {
                kill_results: RefCell::new(Vec::new()),
                start_results: RefCell::new(Vec::new()),
                sleep_calls: RefCell::new(Vec::new()),
            }
        }

        fn with_kill_result(self, result: ProcessResult) -> Self {
            self.kill_results.borrow_mut().push(result);
            self
        }

        fn with_start_result(self, result: ProcessResult) -> Self {
            self.start_results.borrow_mut().push(result);
            self
        }

        fn get_sleep_calls(&self) -> Vec<u64> {
            self.sleep_calls.borrow().clone()
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

        // Get a reference before moving runner into manager
        let runner_ref = &manager.runner;

        manager.restart();

        let sleep_calls = runner_ref.get_sleep_calls();
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

    // run_with_runner tests
    #[test]
    fn test_run_with_runner_kill_command() {
        let runner = MockProcessRunner::new().with_kill_result(ProcessResult::success("Killed"));

        assert!(run_with_runner(Some(Commands::Kill), runner));
    }

    #[test]
    fn test_run_with_runner_start_command() {
        let runner = MockProcessRunner::new().with_start_result(ProcessResult::success("Started"));

        assert!(run_with_runner(Some(Commands::Start), runner));
    }

    #[test]
    fn test_run_with_runner_restart_command() {
        let runner = MockProcessRunner::new()
            .with_kill_result(ProcessResult::success("Killed"))
            .with_start_result(ProcessResult::success("Started"));

        assert!(run_with_runner(Some(Commands::Restart), runner));
    }

    #[test]
    fn test_run_with_runner_no_command_defaults_to_restart() {
        let runner = MockProcessRunner::new()
            .with_kill_result(ProcessResult::success("Killed"))
            .with_start_result(ProcessResult::success("Started"));

        // None should behave like restart
        assert!(run_with_runner(None, runner));
    }

    #[test]
    fn test_run_with_runner_kill_failure_returns_false() {
        let runner = MockProcessRunner::new().with_kill_result(ProcessResult::failure("Error"));

        assert!(!run_with_runner(Some(Commands::Kill), runner));
    }

    #[test]
    fn test_run_with_runner_start_failure_returns_false() {
        let runner = MockProcessRunner::new().with_start_result(ProcessResult::failure("Error"));

        assert!(!run_with_runner(Some(Commands::Start), runner));
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
        // This tests that --version is recognized (it will cause early exit in real usage)
        let result = Cli::try_parse_from(["stuckbar", "--version"]);
        assert!(result.is_err()); // clap returns Err for --version
    }

    #[test]
    fn test_cli_help_flag() {
        // This tests that --help is recognized (it will cause early exit in real usage)
        let result = Cli::try_parse_from(["stuckbar", "--help"]);
        assert!(result.is_err()); // clap returns Err for --help
    }

    #[test]
    fn test_cli_invalid_command() {
        let result = Cli::try_parse_from(["stuckbar", "invalid"]);
        assert!(result.is_err());
    }

    // ExplorerManager builder pattern test
    #[test]
    fn test_explorer_manager_with_restart_delay() {
        let runner = MockProcessRunner::new();
        let manager = ExplorerManager::new(runner).with_restart_delay(1000);
        assert_eq!(manager.restart_delay_ms, 1000);
    }
}
