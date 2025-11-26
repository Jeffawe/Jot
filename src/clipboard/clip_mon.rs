use copypasta::{ClipboardContext, ClipboardProvider};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::db::CLIPBOARD_DB;

use crate::context::get_context;
use crate::types::{ClipboardEntry, SimplifiedWindowInfo};
use crate::embeds::{generate_embedding};

pub struct ClipMon {
    ctx: ClipboardContext,
    last_clip: String,
    last_context: Option<SimplifiedWindowInfo>,
}

impl ClipMon {
    pub fn new() -> Self {
        Self {
            ctx: ClipboardContext::new().unwrap(),
            last_clip: String::new(),
            last_context: None,
        }
    }

    pub fn check(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let clip = self.ctx.get_contents().unwrap_or_default();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let current_context = match get_context() {
            Ok(info) => info,
            Err(e) => {
                eprintln!("Failed to get context: {}", e);
                return Ok(());
            }
        };

        // New clipboard?
        if !clip.is_empty() && clip != self.last_clip {
            let entry = ClipboardEntry {
                timestamp,
                context: current_context.clone(),
                content: clip.clone(),
            };

            println!("New clipboard entry: {:?}", entry);

            // Write directly to DB
            if let Err(e) = self.add_to_db(&entry) {
                eprintln!("Failed to save clipboard to DB: {}", e);
            }

            self.last_clip = clip;
            self.last_context = Some(current_context.clone());
        }
        // Context changed but same clipboard?
        else if let Some(ref prev) = self.last_context {
            if prev != &current_context {
                println!(
                    "Context → {} – {}",
                    current_context.info.name, current_context.title
                );
                self.last_context = Some(current_context);
            }
        } else {
            self.last_context = Some(current_context);
        }

        Ok(())
    }

    pub fn add_to_db(&self, entry: &ClipboardEntry) -> Result<(), Box<dyn std::error::Error>> {
        let db = CLIPBOARD_DB
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;

        let embeds = generate_embedding(&entry.content)?;

        db.insert_clipboard(
            &entry.content,
            entry.timestamp,
            &entry.context.info.name,
            &entry.context.title,
            Some(embeds),
        )?;
        Ok(())
    }
}

pub static GLOBAL_CLIP_MON: Lazy<Mutex<ClipMon>> = Lazy::new(|| Mutex::new(ClipMon::new()));
