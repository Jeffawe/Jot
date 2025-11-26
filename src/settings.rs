use once_cell::sync::Lazy;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub capture_clipboard: bool,
    pub capture_shell: bool,
    pub capture_shell_history_with_files: bool,
    pub shell_case_sensitive: bool,
    pub clipboard_case_sensitive: bool,
    pub clipboard_limit: usize,
    pub shell_limit: usize,
}

impl Settings {
    fn default() -> Self {
        Self {
            capture_clipboard: true,
            capture_shell: true,
            capture_shell_history_with_files: false,
            shell_case_sensitive: false,
            clipboard_case_sensitive: false,
            clipboard_limit: 10_000,
            shell_limit: 5_000,
        }
    }

    // Load settings from database
    pub fn load() -> Self {
        match Self::load_from_db() {
            Ok(settings) => settings,
            Err(e) => {
                eprintln!("Failed to load settings: {}, using defaults", e);
                Self::default()
            }
        }
    }

    fn load_from_db() -> Result<Self> {
        let conn = Self::get_connection()?;

        // Initialize settings table if it doesn't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        let mut settings = Self::default();

        // Helper to get a setting
        let get_setting = |key: &str| -> Option<String> {
            conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .ok()
        };

        // Load each setting
        if let Some(val) = get_setting("capture_clipboard") {
            settings.capture_clipboard = val.parse().unwrap_or(true);
        }
        if let Some(val) = get_setting("capture_shell") {
            settings.capture_shell = val.parse().unwrap_or(true);
        }
        if let Some(val) = get_setting("capture_shell_history_with_files") {
            settings.capture_shell_history_with_files = val.parse().unwrap_or(false);
        }
        if let Some(val) = get_setting("shell_case_sensitive") {
            settings.shell_case_sensitive = val.parse().unwrap_or(false);
        }
        if let Some(val) = get_setting("clipboard_case_sensitive") {
            settings.clipboard_case_sensitive = val.parse().unwrap_or(false);
        }
        if let Some(val) = get_setting("clipboard_limit") {
            settings.clipboard_limit = val.parse().unwrap_or(10_000);
        }
        if let Some(val) = get_setting("shell_limit") {
            settings.shell_limit = val.parse().unwrap_or(5_000);
        }

        Ok(settings)
    }

    // Save settings to database
    pub fn save(&self) -> Result<()> {
        let conn = Self::get_connection()?;

        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            ["capture_clipboard", &self.capture_clipboard.to_string()],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            ["capture_shell", &self.capture_shell.to_string()],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [
                "capture_shell_history_with_files",
                &self.capture_shell_history_with_files.to_string(),
            ],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [
                "shell_case_sensitive",
                &self.shell_case_sensitive.to_string(),
            ],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [
                "clipboard_case_sensitive",
                &self.clipboard_case_sensitive.to_string(),
            ],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            ["clipboard_limit", &self.clipboard_limit.to_string()],
        )?;
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            ["shell_limit", &self.shell_limit.to_string()],
        )?;

        Ok(())
    }

    fn get_connection() -> Result<Connection> {
        let db_path = Self::get_db_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        Connection::open(db_path)
    }

    fn get_db_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".jotx")
            .join("jotx.db")
    }

    // Toggle methods
    pub fn toggle_clipboard(&mut self) {
        self.capture_clipboard = !self.capture_clipboard;
        self.save().ok(); // Auto-save on change
    }

    pub fn toggle_shell(&mut self) {
        self.capture_shell = !self.capture_shell;
        self.save().ok();
    }

    pub fn toggle_shell_history(&mut self) {
        self.capture_shell_history_with_files = !self.capture_shell_history_with_files;
        self.save().ok();
    }

    pub fn toggle_shell_case_sensitive(&mut self) {
        self.shell_case_sensitive = !self.shell_case_sensitive;
        self.save().ok();
    }

    pub fn toggle_clipboard_case_sensitive(&mut self) {
        self.clipboard_case_sensitive = !self.clipboard_case_sensitive;
        self.save().ok();
    }

    pub fn set_clipboard_limit(&mut self, limit: usize) {
        self.clipboard_limit = limit;
        self.save().ok();
    }

    pub fn set_shell_limit(&mut self, limit: usize) {
        self.shell_limit = limit;
        self.save().ok();
    }
}

// Load settings from DB on first access
pub static GLOBAL_SETTINGS: Lazy<Mutex<Settings>> = Lazy::new(|| Mutex::new(Settings::load()));
