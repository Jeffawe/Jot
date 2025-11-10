use copypasta::{ClipboardContext, ClipboardProvider};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::context::get_context;
use crate::types::{ClipboardEntry, SimplifiedWindowInfo};

const MAX_HISTORY: usize = 100;

pub struct ClipMon {
    ctx: ClipboardContext,
    pub history: VecDeque<ClipboardEntry>,
    last_clip: String,
    last_context: Option<SimplifiedWindowInfo>,
}

impl ClipMon {
    pub fn new() -> Self {
        Self {
            ctx: ClipboardContext::new().unwrap(),
            history: VecDeque::with_capacity(MAX_HISTORY),
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

        // 1. New clipboard?
        if !clip.is_empty() && clip != self.last_clip {
            let entry = ClipboardEntry {
                timestamp,
                context: current_context.clone(),
                content: clip.clone(),
            };

            if self.history.len() >= MAX_HISTORY {
                self.history.pop_front();
            }

            println!("New clipboard entry: {:?}", entry);
            self.history.push_back(entry);

            self.last_clip = clip;
            self.last_context = Some(current_context.clone());
        }
        // Context changed but same clipboard?
        else if let Some(ref prev) = self.last_context {
            if prev != &current_context {
                println!(
                    "Context → {} – {}",
                    current_context.info.name,
                    current_context.title
                );
                self.last_context = Some(current_context);
            }
        } else {
            self.last_context = Some(current_context);
        }

        Ok(())
    }
}