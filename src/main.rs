use copypasta::{ClipboardContext, ClipboardProvider};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

mod context;
mod types;

use context::get_context;
use types::ClipboardEntry;

const MAX_HISTORY: usize = 100;
const SLEEP_DURATION_SECS: u64 = 3;

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();
    let mut history: Vec<ClipboardEntry> = Vec::new();

    loop {
        let clip = ctx.get_contents().unwrap_or_default();
        let start = SystemTime::now();
        let timestamp = start.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let context = match get_context() {
            Ok(info) => info,
            Err(e) => {
                println!("error occurred while getting the active window: {}", e);
                continue;
            }
        };

        if clip.is_empty() {
            println!("Clipboard is empty");
            thread::sleep(Duration::from_secs(SLEEP_DURATION_SECS));
            continue;
        }

        if history.len() >= MAX_HISTORY {
            history.remove(0);
        }

        if history.is_empty() {
            let entry = ClipboardEntry {
                timestamp,
                context: context,
                content: clip,
            };
            println!("New clipboard entry: {:?}", entry);
            history.push(entry);
            thread::sleep(Duration::from_secs(SLEEP_DURATION_SECS));
            continue;
        }

        let should_add =
            history.last().unwrap().content != clip || history.last().unwrap().context != context;

        if should_add {
            let entry = ClipboardEntry {
                timestamp,
                context,
                content: clip,
            };

            println!("New clipboard entry: {:?}", entry);
            history.push(entry);
        }

        thread::sleep(Duration::from_secs(SLEEP_DURATION_SECS));
    }
}
