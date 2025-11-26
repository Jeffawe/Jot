use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::SHELL_DB;
use crate::embeds::generate_embedding;
use crate::types::ShellEntry;

pub struct ShellMon {}

impl ShellMon {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_all_histories(
        &mut self,
        case_sensitive: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        // Process bash history
        if let Ok(bash_commands) = self.read_bash_history() {
            for cmd in bash_commands {
                let cmd = if case_sensitive {
                    cmd
                } else {
                    cmd.to_lowercase()
                };

                if let Err(e) = self.add_or_increment(cmd, timestamp) {
                    eprintln!("Error adding bash command: {}", e);
                }
            }
        }

        // Process zsh history
        if let Ok(zsh_commands) = self.read_zsh_history() {
            for cmd in zsh_commands {
                let cmd = if case_sensitive {
                    cmd
                } else {
                    cmd.to_lowercase()
                };
                
                if let Err(e) = self.add_or_increment(cmd, timestamp) {
                    eprintln!("Error adding zsh command: {}", e);
                }
            }
        }

        // Process fish history
        if let Ok(fish_commands) = self.read_fish_history() {
            for cmd in fish_commands {
                if let Err(e) = self.add_or_increment(cmd, timestamp) {
                    eprintln!("Error adding fish command: {}", e);
                }
            }
        }

        Ok(())
    }

    pub fn add_or_increment(
        &mut self,
        cmd: String,
        timestamp: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let db = SHELL_DB
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;

        // Check if command exists
        if let Some(id) = db.get_shell_command_id(&cmd)? {
            // Increment existing
            db.increment_shell_command(id)?;
        } else {
            let new_entry = ShellEntry {
                timestamp,
                content: cmd,
                times_run: 1,
                user: None,
                host: None,
                context: None,
                working_dir: None,
                git_repo: None,
            };

            // Insert new
            self.add_to_db(&new_entry)?;
        }

        Ok(())
    }

    pub fn add_command(
        &mut self,
        cmd: String,
        timestamp: u64,
        pwd: Option<String>,
        user: Option<String>,
        host: Option<String>,
    ) {
        let new_entry = ShellEntry {
            timestamp,
            content: cmd,
            times_run: 1,
            user,
            host,
            context: None,
            working_dir: pwd,
            git_repo: None,
        };

        match self.add_to_db(&new_entry) {
            Ok(_) => (),
            Err(e) => println!("Error adding command to DB: {}", e),
        }
    }

    pub fn add_to_db(&self, entry: &ShellEntry) -> Result<(), Box<dyn std::error::Error>> {
        let db = SHELL_DB
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;

        if let Ok(embeds) = generate_embedding(&entry.content) {
            db.insert_shell(
                &entry.content,
                entry.timestamp,
                entry.working_dir.as_deref(),
                entry.user.as_deref(),
                entry.host.as_deref(),
                "Terminal",
                "unknown",
                Some(embeds),
            )?;
        }

        Ok(())
    }

    fn read_bash_history(&self) -> Result<Vec<String>, std::io::Error> {
        // Get the home directory
        let home = std::env::var("HOME").expect("HOME not set");

        // Build path to .bash_history
        let history_path = PathBuf::from(home).join(".bash_history");

        // Read the file
        let contents = fs::read_to_string(history_path)?;

        // Split into lines and collect
        let commands: Vec<String> = contents.lines().map(|line| line.to_string()).collect();

        Ok(commands)
    }

    fn read_zsh_history(&self) -> Result<Vec<String>, std::io::Error> {
        let home = std::env::var("HOME").expect("HOME not set");
        let history_path = PathBuf::from(home).join(".zsh_history");
        let contents = fs::read_to_string(history_path)?;

        let commands: Vec<String> = contents
            .lines()
            .filter_map(|line| {
                // Zsh format: : 1234567890:0;command here
                // We want just the command part after the semicolon
                if let Some(pos) = line.find(';') {
                    Some(line[pos + 1..].to_string())
                } else {
                    // Some lines might not have timestamp
                    Some(line.to_string())
                }
            })
            .collect();

        Ok(commands)
    }

    fn read_fish_history(&self) -> Result<Vec<String>, std::io::Error> {
        let home = std::env::var("HOME").expect("HOME not set");
        let history_path = PathBuf::from(home).join(".local/share/fish/fish_history");
        let contents = fs::read_to_string(history_path)?;

        let commands: Vec<String> = contents
            .lines()
            .filter_map(|line| {
                // Fish format: - cmd: command here
                if line.trim().starts_with("- cmd: ") {
                    Some(line.trim()[7..].to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(commands)
    }
}

pub static GLOBAL_SHELL_MON: Lazy<Mutex<ShellMon>> = Lazy::new(|| Mutex::new(ShellMon::new()));
