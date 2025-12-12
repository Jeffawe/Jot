use std::thread;
use std::time::Duration;
use crossbeam_channel::{bounded, Sender, Receiver};
use once_cell::sync::Lazy;
use crate::db::Database;
use crate::embeds::generate_embedding;

// Global DB writer instance
pub static DB_WRITER: Lazy<DbWriter> = Lazy::new(|| {
    DbWriter::new().expect("Failed to initialize DB writer")
});

#[derive(Debug, Clone)]
pub enum DbEntry {
    Shell {
        content: String,
        timestamp: u64,
        working_dir: Option<String>,
        user: Option<String>,
        host: Option<String>,
        app_name: String,
        window_title: String,
    },
    Clipboard {
        content: String,
        timestamp: u64,
        app_name: String,
        window_title: String,
    },
}

pub struct DbWriter {
    pub is_running: bool,
    sender: Sender<DbEntry>,
}

impl DbWriter {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (sender, receiver) = bounded(1000); // Queue size: 1000 entries
        
        // Spawn background worker thread
        thread::spawn(move || {
            worker_thread(receiver);
        });
        
        Ok(Self { is_running: false, sender })
    }

    pub fn update_is_running(&mut self, is_running: bool) {
        self.is_running = is_running;
    }
    
    /// Queue a shell entry for insertion
    pub fn insert_shell(
        &self,
        content: String,
        timestamp: u64,
        working_dir: Option<String>,
        user: Option<String>,
        host: Option<String>,
        app_name: String,
        window_title: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = DbEntry::Shell {
            content,
            timestamp,
            working_dir,
            user,
            host,
            app_name,
            window_title,
        };
        
        self.sender.send(entry)
            .map_err(|e| format!("Failed to queue shell entry: {}", e).into())
    }
    
    /// Queue a clipboard entry for insertion
    pub fn insert_clipboard(
        &self,
        content: String,
        timestamp: u64,
        app_name: String,
        window_title: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entry = DbEntry::Clipboard {
            content,
            timestamp,
            app_name,
            window_title,
        };
        
        self.sender.send(entry)
            .map_err(|e| format!("Failed to queue clipboard entry: {}", e).into())
    }
    
    /// Get queue size (for monitoring)
    pub fn queue_len(&self) -> usize {
        self.sender.len()
    }
}

/// Background worker thread that processes the queue
fn worker_thread(receiver: Receiver<DbEntry>) {
    // Create own DB instance for this thread
    let mut db = match Database::new() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("DB writer thread failed to initialize database: {}", e);
            return;
        }
    };
    
    let mut batch: Vec<DbEntry> = Vec::new();
    let batch_size = 10; // Process in batches
    let batch_timeout = Duration::from_millis(500); // Or flush after 500ms
    
    loop {
        // Try to receive with timeout
        match receiver.recv_timeout(batch_timeout) {
            Ok(entry) => {
                batch.push(entry);
                
                // Collect more entries if available (non-blocking)
                while batch.len() < batch_size {
                    match receiver.try_recv() {
                        Ok(entry) => batch.push(entry),
                        Err(_) => break, // No more entries available
                    }
                }
                
                // Process batch
                process_batch(&mut db, &mut batch);
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // Timeout - flush any pending entries
                if !batch.is_empty() {
                    process_batch(&mut db, &mut batch);
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                // Channel closed - flush and exit
                if !batch.is_empty() {
                    process_batch(&mut db, &mut batch);
                }
                break;
            }
        }
    }
}

/// Process a batch of entries
fn process_batch(db: &mut Database, batch: &mut Vec<DbEntry>) {
    for entry in batch.drain(..) {
        match entry {
            DbEntry::Shell {
                content,
                timestamp,
                working_dir,
                user,
                host,
                app_name,
                window_title,
            } => {
                if let Err(e) = process_shell_entry(
                    db,
                    &content,
                    timestamp,
                    working_dir.as_deref(),
                    user.as_deref(),
                    host.as_deref(),
                    &app_name,
                    &window_title,
                ) {
                    eprintln!("Failed to insert shell entry: {}", e);
                }
            }
            DbEntry::Clipboard {
                content,
                timestamp,
                app_name,
                window_title,
            } => {
                if let Err(e) = process_clipboard_entry(
                    db,
                    &content,
                    timestamp,
                    &app_name,
                    &window_title,
                ) {
                    eprintln!("Failed to insert clipboard entry: {}", e);
                }
            }
        }
    }
}

/// Process a single shell entry with retry logic
fn process_shell_entry(
    db: &mut Database,
    content: &str,
    timestamp: u64,
    working_dir: Option<&str>,
    user: Option<&str>,
    host: Option<&str>,
    app_name: &str,
    window_title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate embedding (can fail gracefully)
    let embedding = match generate_embedding(content) {
        Ok(emb) => Some(emb),
        Err(e) => {
            eprintln!("Failed to generate embedding: {}", e);
            None
        }
    };
    
    // Retry logic for DB lock
    let max_retries = 3;
    let mut attempt = 0;
    
    loop {
        match db.insert_shell(
            content,
            timestamp,
            working_dir,
            user,
            host,
            app_name,
            window_title,
            embedding.clone(),
        ) {
            Ok(_) => return Ok(()),
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(format!("Failed after {} retries: {}", max_retries, e).into());
                }
                
                // Wait before retry (exponential backoff)
                let wait_time = Duration::from_millis(100 * (2_u64.pow(attempt - 1)));
                thread::sleep(wait_time);
            }
        }
    }
}

/// Process a single clipboard entry with retry logic
fn process_clipboard_entry(
    db: &mut Database,
    content: &str,
    timestamp: u64,
    app_name: &str,
    window_title: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate embedding
    let embedding = match generate_embedding(content) {
        Ok(emb) => Some(emb),
        Err(e) => {
            eprintln!("Failed to generate embedding: {}", e);
            None
        }
    };
    
    // Retry logic
    let max_retries = 3;
    let mut attempt = 0;
    
    loop {
        match db.insert_clipboard(
            content,
            timestamp,
            app_name,
            window_title,
            embedding.clone(),
        ) {
            Ok(_) => return Ok(()),
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(format!("Failed after {} retries: {}", max_retries, e).into());
                }
                
                let wait_time = Duration::from_millis(100 * (2_u64.pow(attempt - 1)));
                thread::sleep(wait_time);
            }
        }
    }
}