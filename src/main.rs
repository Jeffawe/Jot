use std::thread;
use std::time::Duration;

mod clipboard;
mod context;
mod shell;
mod types;

use clipboard::clip_mon::ClipMon;
use shell::shell_mon::ShellMon;

const SLEEP_DURATION_SECS: u64 = 1;

fn main() {
    println!("Starting clipboard monitoring...");

    let mut monitor = ClipMon::new();

    let shell_monitor = ShellMon::new();

    thread::spawn(move || {
        loop {
            match monitor.check() {
                Ok(_) => {}
                Err(e) => eprintln!("Error checking clipboard: {}", e),
            }
            
            thread::sleep(Duration::from_secs(SLEEP_DURATION_SECS));
        }
    });

    loop {
        thread::sleep(Duration::from_secs(SLEEP_DURATION_SECS));
    }
}
