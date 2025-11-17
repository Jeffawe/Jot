use clap::{Parser, Subcommand};
use rusqlite::Result;
use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use std::fmt;
use std::str::FromStr;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ClipboardEntry {
    pub timestamp: u64,
    pub context: SimplifiedWindowInfo,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct RelatedCommand {
    pub id: i64,
    pub content: String,
    pub strength: i64,       // How many times seen together
    pub sequence_order: i64, // How far apart (1 = immediate next, 2 = two steps, etc)
    pub last_seen: i64,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ShellEntry {
    pub timestamp: u64,
    pub context: Option<SimplifiedWindowInfo>, // Window context (terminal app)
    pub content: String,                       // The command
    pub times_run: u32,
    pub working_dir: Option<String>, // Where it was run
    pub git_repo: Option<String>,    // Git repo if available
    pub user: Option<String>,        // Username
    pub host: Option<String>,        // Hostname
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
    /// Search your history
    Search {
        query: String,

        #[arg(long)]
        print_only: bool,
    },
    /// Show service status
    Status,
    /// Show settings
    Settings,
    /// Cleanup old entries
    Cleanup,
    /// Gracefully stop the running service
    Exit,

    #[command(hide = true)] // Hide from help menu
    InternalDaemon,

    Capture {
        #[arg(long)]
        cmd: String,

        #[arg(long)]
        pwd: Option<String>,

        #[arg(long)]
        user: Option<String>,

        #[arg(long)]
        host: Option<String>,
    },
}

#[derive(Default)]
pub struct QueryParams {
    pub entry_type: Option<EntryType>,
    pub content_search: Option<String>,
    pub working_dir: Option<String>,
    pub app_name: Option<String>,
    pub user: Option<String>,
    pub host: Option<String>,
    pub limit: Option<usize>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Entry {
    pub id: i64,
    pub entry_type: EntryType,
    pub content: String,
    pub timestamp: i64,
    pub times_run: i64,
    pub working_dir: Option<String>,
    pub git_repo: Option<String>,
    pub git_branch: Option<String>,
    pub user: Option<String>,
    pub host: Option<String>,
    pub app_name: Option<String>,
    pub window_title: Option<String>,
    pub embedding: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum EntryType {
    Clipboard,
    Shell,
}

impl EntryType {
    pub fn as_str(&self) -> &str {
        match self {
            EntryType::Clipboard => "clipboard",
            EntryType::Shell => "shell",
        }
    }
}

// Convert to string
impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntryType::Clipboard => write!(f, "clipboard"),
            EntryType::Shell => write!(f, "shell"),
        }
    }
}

// Convert from string
impl FromStr for EntryType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "clipboard" => Ok(EntryType::Clipboard),
            "shell" => Ok(EntryType::Shell),
            _ => Err(format!("Unknown entry type: {}", s)),
        }
    }
}

// For writing to SQLite
impl ToSql for EntryType {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

// For reading from SQLite
impl FromSql for EntryType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| {
            EntryType::from_str(s).map_err(|e| {
                FromSqlError::Other(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e,
                )))
            })
        })
    }
}
