use byteorder::{ByteOrder, LittleEndian};
use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::types::{Entry, EntryType, QueryParams};

const ASSOCIATION_DEPTH: i64 = 3;
const CLEAN_SESSIONS_DAYS: i64 = 90;
const CLEAN_OLD_ASSOCIATIONS_DAYS: i64 = 30;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path)?;

        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;

        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn get_db_path() -> PathBuf {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home).join(".jotx").join("jotx.db")
    }

    fn init_schema(&self) -> Result<()> {
        // Main entries table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                entry_type TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                times_run INTEGER DEFAULT 1,
                
                working_dir TEXT,
                git_repo TEXT,
                git_branch TEXT,
                user TEXT,
                host TEXT,
                
                app_name TEXT,
                window_title TEXT,
                
                embedding BLOB,
                
                created_at INTEGER DEFAULT (strftime('%s', 'now')),
                updated_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entry_type ON entries(entry_type)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON entries(timestamp DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_content ON entries(content)",
            [],
        )?;

        // FTS5 table
        self.conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
                content,
                app_name,
                window_title,
                working_dir,
                content='entries',
                content_rowid='id'
            )",
            [],
        )?;

        // Triggers
        self.conn.execute(
            "CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
                INSERT INTO entries_fts(rowid, content, app_name, window_title, working_dir)
                VALUES (new.id, new.content, new.app_name, new.window_title, new.working_dir);
            END",
            [],
        )?;

        self.conn.execute(
            "CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
                DELETE FROM entries_fts WHERE rowid = old.id;
            END",
            [],
        )?;

        self.conn.execute(
            "CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
                UPDATE entries_fts 
                SET content = new.content,
                    app_name = new.app_name,
                    window_title = new.window_title,
                    working_dir = new.working_dir
                WHERE rowid = new.id;
            END",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS command_associations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            command_a_id INTEGER NOT NULL,
            command_b_id INTEGER NOT NULL,
            sequence_order INTEGER NOT NULL, -- 1 means A->B, -1 means just co-occurrence
            strength INTEGER DEFAULT 1,      -- Increments each time seen together
            last_seen INTEGER NOT NULL,       -- Timestamp of last co-occurrence
            FOREIGN KEY (command_a_id) REFERENCES entries(id) ON DELETE CASCADE,
            FOREIGN KEY (command_b_id) REFERENCES entries(id) ON DELETE CASCADE,
            UNIQUE(command_a_id, command_b_id, sequence_order)
        )",
            [],
        )?;

        // Index for fast lookups
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_associations_a 
         ON command_associations(command_a_id, strength DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_associations_strength 
         ON command_associations(strength DESC, last_seen DESC)",
            [],
        )?;

        // NEW: Session tracker - groups commands run close together
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS command_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER NOT NULL,
            session_id TEXT NOT NULL,  -- UUID or timestamp-based
            position INTEGER NOT NULL,  -- Order in session (0, 1, 2, 3...)
            timestamp INTEGER NOT NULL,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
        )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions 
         ON command_sessions(session_id, position)",
            [],
        )?;

        Ok(())
    }

    pub fn insert_clipboard(
        &self,
        content: &str,
        timestamp: u64,
        app_name: &str,
        window_title: &str,
        embedding: Option<Vec<f32>>,
    ) -> Result<()> {
        let embedding_blob: Option<Vec<u8>> = embedding.map(|vec| {
            let mut blob = vec![0u8; vec.len() * 4];
            LittleEndian::write_f32_into(&vec, &mut blob);
            blob
        });

        self.conn.execute(
            "INSERT INTO entries (entry_type, content, timestamp, app_name, window_title, embedding)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                "clipboard",
                content,
                timestamp as i64,
                app_name,
                window_title,
                embedding_blob
            ),
        )?;
        Ok(())
    }

    // Check if shell command exists and return its ID
    pub fn get_shell_command_id(&self, content: &str) -> Result<Option<i64>> {
        let result = self.conn.query_row(
            "SELECT id FROM entries WHERE entry_type = ?1 AND content = ?2",
            (EntryType::Shell, content),
            |row| row.get(0),
        );

        match result {
            Ok(id) => Ok(Some(id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // Increment times_run for existing entry
    pub fn increment_shell_command(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE entries SET times_run = times_run + 1, updated_at = strftime('%s', 'now') 
             WHERE id = ?1",
            [id],
        )?;
        Ok(())
    }

    pub fn insert_shell(
        &self,
        content: &str,
        timestamp: u64,
        working_dir: Option<&str>,
        user: Option<&str>,
        host: Option<&str>,
        app_name: &str,
        window_title: &str,
        embedding: Option<Vec<f32>>,
    ) -> Result<()> {
        let embedding_blob: Option<Vec<u8>> = embedding.map(|vec| {
            let mut blob = vec![0u8; vec.len() * 4];
            LittleEndian::write_f32_into(&vec, &mut blob);
            blob
        });

        // Check if command already exists with the same host
        let existing: Option<i64> = self
            .conn
            .query_row(
                "SELECT id FROM entries 
             WHERE entry_type = 'shell' 
             AND content = ?1 
             AND (host = ?2 OR (host IS NULL AND ?2 IS NULL))",
                rusqlite::params![content, host],
                |row| row.get(0),
            )
            .ok();

        let entry_id = if let Some(id) = existing {
            // Same command + same host: increment times_run
            self.conn.execute(
                "UPDATE entries 
             SET times_run = times_run + 1, 
                 updated_at = strftime('%s', 'now'),
                 timestamp = ?2
             WHERE id = ?1",
                rusqlite::params![id, timestamp as i64],
            )?;
            id // Return existing ID
        } else {
            // Different host or new command: insert new entry
            self.conn.execute(
            "INSERT INTO entries (entry_type, content, timestamp, working_dir, user, host, app_name, window_title, embedding)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                EntryType::Shell.to_string(),
                content,
                timestamp as i64,
                working_dir,
                user,
                host,
                app_name,
                window_title,
                embedding_blob,
            ],
        )?;
            self.conn.last_insert_rowid() // Return new ID
        };

        self.track_associations_only(entry_id)?;

        Ok(())
    }

    pub fn query_entries(&self, params: QueryParams) -> Result<Vec<Entry>> {
        let mut sql = String::from("SELECT * FROM entries WHERE 1=1");
        let mut conditions = Vec::new();

        if let Some(et) = &params.entry_type {
            sql.push_str(" AND entry_type = ?");
            conditions.push(et.as_str());
        }

        if let Some(content) = &params.content_search {
            sql.push_str(" AND content LIKE ?");
            conditions.push(content.as_str());
        }

        if let Some(wd) = &params.working_dir {
            sql.push_str(" AND working_dir = ?");
            conditions.push(wd.as_str());
        }

        if let Some(app) = &params.app_name {
            sql.push_str(" AND app_name = ?");
            conditions.push(app.as_str());
        }

        if let Some(u) = &params.user {
            sql.push_str(" AND user = ?");
            conditions.push(u.as_str());
        }

        if let Some(h) = &params.host {
            sql.push_str(" AND host = ?");
            conditions.push(h.as_str());
        }

        sql.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = params.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        let mut stmt = self.conn.prepare(&sql)?;

        let entries = stmt
            .query_map(rusqlite::params_from_iter(conditions.iter()), |row| {
                Ok(Entry {
                    id: row.get(0)?,
                    entry_type: row.get(1)?,
                    content: row.get(2)?,
                    timestamp: row.get(3)?,
                    times_run: row.get(4)?,
                    working_dir: row.get(5)?,
                    git_repo: row.get(6)?,
                    git_branch: row.get(7)?,
                    user: row.get(8)?,
                    host: row.get(9)?,
                    app_name: row.get(10)?,
                    window_title: row.get(11)?,
                    embedding: row.get(12)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn cleanup_old_entries(&self, clipboard_limit: usize, shell_limit: usize) -> Result<()> {
        // Clean up old clipboard entries
        self.conn.execute(
            "DELETE FROM entries 
             WHERE entry_type = ?1 
             AND id NOT IN (
                 SELECT id FROM entries 
                 WHERE entry_type = ?1 
                 ORDER BY timestamp DESC 
                 LIMIT ?2
             )",
            (EntryType::Clipboard, clipboard_limit),
        )?;

        // Clean up old shell entries
        self.conn.execute(
            "DELETE FROM entries 
             WHERE entry_type = ?1 
             AND id NOT IN (
                 SELECT id FROM entries 
                 WHERE entry_type = ?1 
                 ORDER BY timestamp DESC 
                 LIMIT ?2
             )",
            (EntryType::Shell, shell_limit),
        )?;

        Ok(())
    }

    /// Remove weak associations that haven't been reinforced
    /// Deletes associations with strength < 2 that are older than 30 days
    pub fn cleanup_weak_associations(&self) -> Result<usize> {
        const ONE_MONTH_SECONDS: i64 = CLEAN_OLD_ASSOCIATIONS_DAYS * 24 * 60 * 60;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let cutoff_time = now - ONE_MONTH_SECONDS;

        let deleted = self.conn.execute(
            "DELETE FROM command_associations 
             WHERE strength < 2 
             AND last_seen < ?1",
            params![cutoff_time],
        )?;

        Ok(deleted)
    }

    /// Clean up old sessions (optional - saves more space)
    /// Removes sessions older than 90 days
    pub fn cleanup_old_sessions(&self) -> Result<usize> {
        const THREE_MONTHS_SECONDS: i64 = CLEAN_SESSIONS_DAYS * 24 * 60 * 60;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let cutoff_time = now - THREE_MONTHS_SECONDS;

        let deleted = self.conn.execute(
            "DELETE FROM command_sessions 
             WHERE timestamp < ?1",
            params![cutoff_time],
        )?;

        Ok(deleted)
    }

    /// Combined cleanup - run this periodically
    pub fn run_maintenance(&self) -> Result<()> {
        let associations_deleted = self.cleanup_weak_associations()?;
        let sessions_deleted = self.cleanup_old_sessions()?;

        // Also vacuum to reclaim disk space
        self.conn.execute("VACUUM", [])?;

        println!("🧹 Maintenance complete:");
        println!("  - Removed {} weak associations", associations_deleted);
        println!("  - Removed {} old sessions", sessions_deleted);

        Ok(())
    }
    pub fn should_run_maintenance(&self) -> bool {
        const ONE_WEEK: u64 = 7 * 24 * 60 * 60; // seconds

        match self.get_last_maintenance_time() {
            Ok(last_time) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                now - last_time > ONE_WEEK
            }
            Err(_) => true, // Never run before, so run now
        }
    }

    fn get_last_maintenance_time(&self) -> Result<u64, std::io::Error> {
        let path = self.get_maintenance_file_path();
        let content = std::fs::read_to_string(path)?;
        content
            .trim()
            .parse::<u64>()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))
    }

    pub fn update_last_maintenance(&self) -> Result<(), std::io::Error> {
        let path = self.get_maintenance_file_path();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, now.to_string())
    }

    fn get_maintenance_file_path(&self) -> PathBuf {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home).join(".jotx").join(".last_maintenance")
    }

    pub fn get_or_create_session_id(&self) -> Result<String> {
        const SESSION_TIMEOUT: i64 = 300; // 5 minutes in seconds

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Try to get the most recent session
        let last_session: Option<(String, i64)> = self
            .conn
            .query_row(
                "SELECT session_id, MAX(timestamp) as last_time
             FROM command_sessions
             GROUP BY session_id
             ORDER BY last_time DESC
             LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        // If last command was within timeout, reuse session
        if let Some((session_id, last_time)) = last_session {
            if now - last_time < SESSION_TIMEOUT {
                return Ok(session_id);
            }
        }

        // Otherwise create new session ID
        Ok(format!("session_{}", now))
    }

    fn track_associations_only(&self, entry_id: i64) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        // Get or create session
        let session_id = self.get_or_create_session_id()?;

        // Get position in this session
        let position: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 
         FROM command_sessions WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;

        // Add to current session
        self.conn.execute(
            "INSERT INTO command_sessions (entry_id, session_id, position, timestamp)
         VALUES (?1, ?2, ?3, ?4)",
            params![entry_id, session_id, position, now],
        )?;

        // Update associations with recent commands in this session
        let recent_commands: Vec<i64> = self
            .conn
            .prepare(
                "SELECT entry_id FROM command_sessions 
             WHERE session_id = ?1 AND position < ?2 AND position >= ?3
             ORDER BY position DESC",
            )?
            .query_map(
                params![
                    session_id,
                    position,
                    position.saturating_sub(ASSOCIATION_DEPTH)
                ],
                |row| row.get(0),
            )?
            .collect::<Result<Vec<_>, _>>()?;

        for (idx, prev_command_id) in recent_commands.iter().enumerate() {
            let sequence_order = (recent_commands.len() - idx) as i64;

            self.conn.execute(
                "INSERT INTO command_associations 
             (command_a_id, command_b_id, sequence_order, strength, last_seen)
             VALUES (?1, ?2, ?3, 1, ?4)
             ON CONFLICT(command_a_id, command_b_id, sequence_order) 
             DO UPDATE SET 
                strength = strength + 1,
                last_seen = ?4",
                params![prev_command_id, entry_id, sequence_order, now],
            )?;
        }

        Ok(())
    }

    // // Get related commands for a given command
    // pub fn get_related_commands(&self, entry_id: i64, limit: usize) -> Result<Vec<RelatedCommand>> {
    //     let mut stmt = self.conn.prepare(
    //         "SELECT e.id, e.content, a.strength, a.sequence_order, a.last_seen
    //      FROM command_associations a
    //      JOIN entries e ON e.id = a.command_b_id
    //      WHERE a.command_a_id = ?1
    //      ORDER BY a.strength DESC, a.last_seen DESC
    //      LIMIT ?2",
    //     )?;

    //     let results = stmt
    //         .query_map(params![entry_id, limit], |row| {
    //             Ok(RelatedCommand {
    //                 id: row.get(0)?,
    //                 content: row.get(1)?,
    //                 strength: row.get(2)?,
    //                 sequence_order: row.get(3)?,
    //                 last_seen: row.get(4)?,
    //             })
    //         })?
    //         .collect::<Result<Vec<_>, _>>()?;

    //     Ok(results)
    // }

    // Get count of entries by type
    // pub fn get_entry_count(&self, entry_type: EntryType) -> Result<usize> {
    //     let count: i64 = self.conn.query_row(
    //         "SELECT COUNT(*) FROM entries WHERE entry_type = ?1",
    //         [entry_type],
    //         |row| row.get(0),
    //     )?;
    //     Ok(count as usize)
    // }
}

pub static GLOBAL_DB: Lazy<Mutex<Database>> =
    Lazy::new(|| Mutex::new(Database::new().expect("Failed to initialize database")));
