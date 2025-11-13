use clap::{Parser, Subcommand};

#[allow(dead_code)]
#[derive(Debug)]
pub struct ClipboardEntry {
    pub timestamp: u64,
    pub context: SimplifiedWindowInfo,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ShellEntry {
    pub timestamp: u64,
    pub context: SimplifiedWindowInfo,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimpleProcessInfo {
    pub process_id: u32,
    pub path: String,
    pub name: String,
    pub exec_name: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimplifiedWindowInfo {
    pub id: u32, // Assuming u32 based on the input
    pub os: String,
    pub title: String,
    pub info: SimpleProcessInfo,
}

#[derive(Parser)]
#[command(name = "jotx", version, about = "Your digital memory agent")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the clipboard/shell monitor (interactive mode)
    Run,
    /// Show help info
    Info,
    /// Ask a question
    Ask { query: String },
    /// Show service status
    Status,
    /// Gracefully stop the running service
    Exit,
    
    #[command(hide = true)]  // Hide from help menu
    InternalDaemon,
}
