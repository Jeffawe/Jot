use clap::Parser;
use ctrlc;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

mod ask;
mod clipboard;
mod commands;
mod config;
mod context;
mod db;
mod embeds;
mod llm;
mod managers;
mod pid_controller;
mod plugin;
mod settings;
mod shell;
mod types;

use types::{Cli, Commands};

use ask::ask_handler::{AskResponse, ask};
use ask::search_handler::search;
use commands::{get_working_directory, run_make, show_settings};
use config::reload_config;
use llm::handle_llm;

use clipboard::clip_mon::GLOBAL_CLIP_MON;
use config::GLOBAL_CONFIG;
use db::GLOBAL_DB;
use plugin::{
    CommandContext, DaemonContext, GLOBAL_PLUGIN_MANAGER, SensitiveCommandFilter,
    check_plugin_functions, create_new_plugin_script,
};
use settings::GLOBAL_SETTINGS;
use shell::shell_mon::GLOBAL_SHELL_MON;

use managers::shutdown_manager::{on_shutdown, shutdown};
use pid_controller::{is_running, remove_pid, save_pid};

use crate::commands::get_plugin_dir;

const CLIP_SLEEP_DURATION_SECS: u64 = 1;
const SHELL_SLEEP_DURATION_SECS: u64 = 3600;
const APP_LOOP_SECS: u64 = 10;

const SERVICE_NAME: &str = "jotx";
const SERVICE_NAME_SHORT: &str = "js";

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    on_shutdown(|| {
        println!("  🌐 Closing network connections...");
    });

    match cli.command {
        Commands::Run => start_service(),
        Commands::Ask { query, print_only } => {
            let pwd = get_working_directory();

            let ask_result = ask(&query, &pwd, print_only);
            if let Some(result) = ask_to_string(ask_result.await) {
                if print_only {
                    print!("{}", result);
                }
            } else if print_only {
                std::process::exit(1);
            }
        }
        Commands::Cleanup => force_maintain(),
        Commands::Search { query, print_only } => {
            let pwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| String::from(""));

            if let Some(result) = search(&query, &pwd, print_only) {
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
        Commands::HandleLlm => match handle_llm().await {
            Ok(_) => println!("✅ LLM setup completed successfully."),
            Err(e) => eprintln!("❌ LLM setup failed: {}", e),
        },
        Commands::Plugin(args) => {
            if args.create {
                // Logic for jotx plugin --create <NAME>
                if let Some(name) = args.name {
                    let plugin_dir = get_plugin_dir();
                    let result = create_new_plugin_script(&plugin_dir, &name);
                    match result {
                        Ok(path) => println!("✅ Plugin created at: {}", path),
                        Err(e) => eprintln!("❌ Error creating plugin: {}", e),
                    }
                } else {
                    eprintln!("Error: --create requires a plugin name.");
                }
            } else if let Some(target) = args.check {
                // Logic for jotx plugin --check <NAME> or --check all
                let result;
                if target == "all" || target.is_empty() {
                    result = check_plugin_functions(&get_plugin_dir(), None);
                } else {
                    result = check_plugin_functions(&get_plugin_dir(), Some(&target));
                }
                match result {
                    Ok(_) => println!("✅ Plugin check completed successfully."),
                    Err(e) => eprintln!("❌ Plugin check failed: {}", e),
                }
            } else {
                println!("Plugin command requires --create or --check.");
            }
        }
        Commands::Reload => reload(),
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
        Commands::CleanData => run_make("clean-data"),
        Commands::Uninstall => run_make("uninstall"),
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

    initialize_plugins();

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

    println!("Initial data load from terminal histories...");
    if let Ok(mut monitor) = GLOBAL_SHELL_MON.lock() {
        if let Err(e) = monitor.read_all_histories() {
            eprintln!("Shell error: {}", e);
        }
    }

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

    let mut daemon_context = DaemonContext {
        iteration: 0,
        uptime_secs: 0,
    };

    while is_running() {
        if let Ok(config) = GLOBAL_CONFIG.lock() {
            if last_maintenance.elapsed().as_secs()
                >= config.storage.maintenance_interval_days * 86400
            {
                maintain();
                last_maintenance = Instant::now();
            }
        }

        daemon_context.iteration += 1;
        daemon_context.uptime_secs = get_uptime();

        if let Ok(plugins) = GLOBAL_PLUGIN_MANAGER.lock() {
            plugins.trigger_daemon_tick(&daemon_context);
        }

        thread::sleep(Duration::from_secs(APP_LOOP_SECS));
    }

    shutdown();
    remove_pid();
    println!("\nGoodbye!");
}

pub fn initialize_plugins() {
    let mut pm = GLOBAL_PLUGIN_MANAGER.lock().unwrap();

    // Register built-in plugins
    pm.register(Box::new(SensitiveCommandFilter));

    println!("✅ Loaded {} plugins", pm.list().len());
}

pub fn get_uptime() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_secs()
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

        if let Ok(plugins) = GLOBAL_PLUGIN_MANAGER.lock() {
            let command_capture = CommandContext {
                command: cmd.to_string(),
                working_dir: pwd.clone().unwrap_or_default(),
                user: user.clone().unwrap_or_default(),
                host: host.clone().unwrap_or_default(),
                timestamp,
            };

            if plugins.trigger_command_captured(&command_capture) {
                monitor.add_command(cmd.to_string(), timestamp, pwd, user, host);
            }
        }
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

pub fn reload() {
    if let Err(e) = reload_config() {
        eprintln!("Failed to reload settings: {}", e);
    }
}

fn ask_to_string(resp: Result<AskResponse, Box<dyn std::error::Error>>) -> Option<String> {
    match resp.ok()? {
        AskResponse::Knowledge(s) => Some(s),
        AskResponse::SearchResults(opt) => opt,
    }
}
