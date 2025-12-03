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

#[derive(Subcommand)]
enum Commands {
    /// Terminate explorer.exe process
    Kill,
    /// Start explorer.exe process
    Start,
    /// Restart explorer.exe (kill then start)
    Restart,
}

fn kill_explorer() -> bool {
    println!("{}", "Terminating explorer.exe...".yellow());

    let result = Command::new("taskkill")
        .args(["/F", "/IM", "explorer.exe"])
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                println!("{}", "Successfully terminated explorer.exe".green());
                true
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("{} {}", "Failed to terminate explorer.exe:".red(), stderr);
                false
            }
        }
        Err(e) => {
            eprintln!("{} {}", "Error executing taskkill:".red(), e);
            false
        }
    }
}

fn start_explorer() -> bool {
    println!("{}", "Starting explorer.exe...".yellow());

    let result = Command::new("explorer.exe").spawn();

    match result {
        Ok(_) => {
            println!("{}", "Successfully started explorer.exe".green());
            true
        }
        Err(e) => {
            eprintln!("{} {}", "Error starting explorer.exe:".red(), e);
            false
        }
    }
}

fn restart_explorer() -> bool {
    println!("{}", "Restarting explorer.exe...".cyan().bold());

    if !kill_explorer() {
        return false;
    }

    // Small delay to ensure explorer is fully terminated
    std::thread::sleep(std::time::Duration::from_millis(RESTART_DELAY_MS));

    if !start_explorer() {
        return false;
    }

    println!("{}", "Explorer.exe restarted successfully!".green().bold());
    true
}

fn main() {
    let cli = Cli::parse();

    let success = match cli.command {
        Some(Commands::Kill) => kill_explorer(),
        Some(Commands::Start) => start_explorer(),
        Some(Commands::Restart) => restart_explorer(),
        None => {
            // Default action: restart explorer
            restart_explorer()
        }
    };

    if !success {
        std::process::exit(1);
    }
}
