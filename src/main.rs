use clap::Parser;
use ctrlc;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

mod clipboard;
mod commands;
mod context;
mod managers;
mod pid_controller;
mod shell;
mod types;
mod ask;

use clipboard::clip_mon::GLOBAL_CLIP_MON;
use shell::shell_mon::GLOBAL_SHELL_MON;
use commands::print_help;
use ask::ask_handler::ask;
use managers::shutdown_manager::{on_shutdown, shutdown};
use pid_controller::{is_running, remove_pid, save_pid};
use types::{Cli, Commands};

const SLEEP_DURATION_SECS: u64 = 1;
const APP_LOOP_SECS: u64 = 10;

// static RUNNING: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));

fn main() {
    let cli = Cli::parse();

    // Set up Ctrl+C handler (uses global RUNNING)
    ctrlc::set_handler(move || {
        println!("\nCtrl+C received. Shutting down...");
        shutdown();
        stop_service();
    })
    .expect("Error setting Ctrl+C handler");

    on_shutdown(|| {
        println!("  🌐 Closing network connections...");
    });

    match cli.command {
        Commands::Run => start_service(),
        Commands::Info => print_help(),
        Commands::Ask { query } => ask(&query),
        Commands::Status => {
            if is_running() {
                println!("jotx is RUNNING");
            } else {
                println!("jotx is STOPPED");
            }
        }
        Commands::Exit => stop_service(),
        Commands::InternalDaemon => {
            save_pid();
            run_service();
        }
    }
}

// Start service in background
fn start_service() {
    if is_running() {
        println!("Service already running!");
        return;
    }

    println!("🚀 Starting application...\n");

    let exe = std::env::current_exe().expect("Failed to get exe path");

    // Spawn detached background process
    let stdout = std::fs::File::create("/tmp/jotx.log")
        .map(Stdio::from)
        .unwrap_or_else(|_| Stdio::null());

    let stderr = std::fs::File::create("/tmp/jotx.err")
        .map(Stdio::from)
        .unwrap_or_else(|_| Stdio::null());

    Command::new(exe)
        .arg("internal-daemon")
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .expect("Failed to spawn daemon");

    thread::sleep(Duration::from_millis(200));
    println!("Service started. Use 'jotx exit' to stop.\n");
}

// Stop service
fn stop_service() {
    if !is_running() {
        println!("Service not running.");
        return;
    }

    println!("Stopping service...");
    if let Ok(pid_str) = std::fs::read_to_string(pid_controller::PID_FILE) {
        if let Ok(pid) = pid_str.trim().parse::<u32>() {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
        }
    }
    remove_pid();

    println!("Service stopped.");
}

// The actual long-running service
pub fn run_service() {
    println!("Running service...\n");
    println!("run_service started, PID: {}", std::process::id());

    // Clipboard thread
    thread::spawn(move || {
        while is_running() {
            // Lock the mutex to get mutable access
            if let Ok(mut monitor) = GLOBAL_CLIP_MON.lock() {
                if let Err(e) = monitor.check() {
                    eprintln!("Clipboard error: {}", e);
                }
            }
            thread::sleep(Duration::from_secs(SLEEP_DURATION_SECS));
        }
    });

    // Shell thread
    thread::spawn(move || {
        while is_running() {
            // Lock the mutex to get mutable access
            if let Ok(mut monitor) = GLOBAL_SHELL_MON.lock() {
                if let Err(e) = monitor.check() {
                    eprintln!("Shell error: {}", e);
                }
            }
            thread::sleep(Duration::from_secs(SLEEP_DURATION_SECS));
        }
    });

    // Main service loop — checks global flag
    while is_running() {
        thread::sleep(Duration::from_secs(APP_LOOP_SECS));
    }

    shutdown();
    remove_pid();
    println!("\nGoodbye!");
}
