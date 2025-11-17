use clap::Parser;
use ctrlc;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

mod ask;
mod clipboard;
mod commands;
mod context;
mod db;
mod embeds;
mod managers;
mod pid_controller;
mod settings;
mod shell;
mod types;

use types::{Cli, Commands};

use ask::ask_handler::ask;
use ask::search_handler::search;
use commands::{print_help, show_settings};

use clipboard::clip_mon::GLOBAL_CLIP_MON;
use settings::GLOBAL_SETTINGS;
use shell::shell_mon::GLOBAL_SHELL_MON;

use db::GLOBAL_DB;

use managers::shutdown_manager::{on_shutdown, shutdown};
use pid_controller::{is_running, remove_pid, save_pid};

const CLIP_SLEEP_DURATION_SECS: u64 = 1;
const SHELL_SLEEP_DURATION_SECS: u64 = 10;
const APP_LOOP_SECS: u64 = 10;
const MAINTENANCE_INTERVAL_SECS: u64 = 3600;

const SERVICE_NAME: &str = "jotx";
const SERVICE_NAME_SHORT: &str = "js";

fn main() {
    let cli = Cli::parse();

    on_shutdown(|| {
        println!("  🌐 Closing network connections...");
    });

    match cli.command {
        Commands::Run => start_service(),
        Commands::Info => print_help(),
        Commands::Ask { query } => ask(&query),
        Commands::Cleanup => force_maintain(),
        Commands::Search { query, print_only } => {
            if let Some(result) = search(&query, print_only) {
                if print_only {
                    print!("{}", result);
                }
            } else if print_only {
                std::process::exit(1);
            }
        }
        Commands::Status => {
            if is_running() {
                println!("jotx is RUNNING");
            } else {
                println!("jotx is STOPPED");
            }
        }
        Commands::Settings => show_settings(),
        Commands::Exit => stop_service(),
        Commands::InternalDaemon => {
            save_pid();
            run_service();
        }
        Commands::Capture {
            cmd,
            pwd,
            user,
            host,
        } => {
            capture_command(&cmd, pwd, user, host);
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

    // Set up Ctrl+C handler (uses global RUNNING)
    ctrlc::set_handler(move || {
        println!("\nCtrl+C received. Shutting down...");
        shutdown();
        stop_service();
    })
    .expect("Error setting Ctrl+C handler");

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
            if let Ok(settings) = GLOBAL_SETTINGS.lock() {
                if settings.capture_clipboard {
                    // Lock the mutex to get mutable access
                    if let Ok(mut monitor) = GLOBAL_CLIP_MON.lock() {
                        if let Err(e) = monitor.check() {
                            eprintln!("Clipboard error: {}", e);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(CLIP_SLEEP_DURATION_SECS));
        }
    });

    // Shell thread
    thread::spawn(move || {
        while is_running() {
            if let Ok(settings) = GLOBAL_SETTINGS.lock() {
                if settings.capture_shell {
                    // Lock the mutex to get mutable access
                    if let Ok(mut monitor) = GLOBAL_SHELL_MON.lock() {
                        if let Err(e) = monitor.read_files() {
                            eprintln!("Shell error: {}", e);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_secs(SHELL_SLEEP_DURATION_SECS));
        }
    });

    // Main service loop — checks global flag
    let mut last_maintenance = Instant::now();

    while is_running() {
        if last_maintenance.elapsed().as_secs() >= MAINTENANCE_INTERVAL_SECS {
            maintain();
            last_maintenance = Instant::now();
        }

        thread::sleep(Duration::from_secs(APP_LOOP_SECS));
    }

    shutdown();
    remove_pid();
    println!("\nGoodbye!");
}

fn capture_command(cmd: &str, pwd: Option<String>, user: Option<String>, host: Option<String>) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    if let Ok(mut monitor) = GLOBAL_SHELL_MON.lock() {
        if cmd.starts_with(SERVICE_NAME) || cmd.starts_with(SERVICE_NAME_SHORT) {
            return;
        }

        monitor.add_command(cmd.to_string(), timestamp, pwd, user, host);
    }
}

fn maintain() {
    if let Ok(settings) = GLOBAL_SETTINGS.lock() {
        if let Ok(db) = GLOBAL_DB.lock() {
            // Always clean up old entries (this is cheap and frequent)
            if let Err(e) = db.cleanup_old_entries(settings.clipboard_limit, settings.shell_limit) {
                eprintln!("Cleanup error: {}", e);
            }

            // Only run full maintenance if it's been a while (expensive)
            if db.should_run_maintenance() {
                if let Err(e) = db.run_maintenance() {
                    eprintln!("Maintenance error: {}", e);
                } else {
                    // Update last maintenance timestamp
                    if let Err(e) = db.update_last_maintenance() {
                        eprintln!("Failed to update maintenance timestamp: {}", e);
                    }
                }
            }
        }
    }

    print!("Database maintenance completed\n");
}

fn force_maintain() {
    if let Ok(settings) = GLOBAL_SETTINGS.lock() {
        if let Ok(db) = GLOBAL_DB.lock() {
            // Always clean up old entries (this is cheap and frequent)
            if let Err(e) = db.cleanup_old_entries(settings.clipboard_limit, settings.shell_limit) {
                eprintln!("Cleanup error: {}", e);
            }

            if let Err(e) = db.run_maintenance() {
                eprintln!("Maintenance error: {}", e);
            } else {
                // Update last maintenance timestamp
                if let Err(e) = db.update_last_maintenance() {
                    eprintln!("Failed to update maintenance timestamp: {}", e);
                }
            }
        }
    }

    print!("Database maintenance completed\n");
}
