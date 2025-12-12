use clap::Parser;
use ctrlc;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};

use jotx::types::{Cli, Commands};

use jotx::ask::{AskResponse, ask, search};
use jotx::clipboard::clip_mon::GLOBAL_CLIP_MON;
use jotx::commands::{get_plugin_dir, get_working_directory, show_privacy_settings, show_settings};
use jotx::config::GLOBAL_CONFIG;
use jotx::config::reload_config;
use jotx::db::{DB_WRITER, USER_DB};
use jotx::llm::handle_llm;
use jotx::plugin::{
    CommandContext, DaemonContext, GLOBAL_PLUGIN_MANAGER, SensitiveCommandFilter,
    check_plugin_functions, create_new_plugin_script,
};
use jotx::settings::GLOBAL_SETTINGS;
use jotx::setup::{clean_data, full_setup, install_llm, setup_hooks, uninstall, update};
use jotx::shell::shell_mon::GLOBAL_SHELL_MON;

use jotx::managers::shutdown_manager::{on_shutdown, shutdown};
use jotx::pid_controller::{PID_FILE, is_running, remove_pid, save_pid};



const CLIP_SLEEP_DURATION_SECS: u64 = 1;
const SHELL_SLEEP_DURATION_SECS: u64 = 60; // This is multiplied by 60 to get 3600 seconds
const APP_LOOP_SECS: u64 = 10;
const DB_LOOP_SECS: u64 = 5; // This is multiplied by 60 to get 300 seconds

const SERVICE_NAME: &str = "jotx";
const SERVICE_NAME_SHORT: &str = "js";
const SERVICE_NAME_SHORT2: &str = "ja";

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

            let ask_result = ask(&query, &pwd, print_only, false).await;
            match ask_result {
                Ok(value) => {
                    if let Some(result) = ask_to_string(value) {
                        if print_only {
                            print!("{}", result);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    if print_only {
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Cleanup => maintain(),
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
                println!("✅ Jotx is running");
                std::process::exit(0);
            } else {
                println!("⏹️ Jotx is stopped");
                std::process::exit(1);
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
        Commands::Privacy => {
            if let Err(e) = show_privacy_settings() {
                eprintln!("Error updating privacy settings: {}", e);
            }
        }
        Commands::Update => {
            if let Err(e) = update() {
                eprintln!("Error updating: {}", e);
            }
        }
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
        Commands::CleanData => {
            if let Err(e) = clean_data(false) {
                eprintln!("Error cleaning data: {}", e);
            }
        }
        Commands::Uninstall => {
            if let Err(e) = uninstall(false) {
                eprintln!("Error uninstalling: {}", e);
            }
        }
        Commands::InstallLLM => {
            if let Err(e) = install_llm(false) {
                eprintln!("Error installing llm: {}", e);
            }
        }
        Commands::Setup => {
            if let Err(e) = full_setup(false, false) {
                eprintln!("Error setting up: {}", e);
            }
        }
        Commands::SetupHooks => {
            if let Err(e) = setup_hooks() {
                eprintln!("Error setting up hooks: {}", e);
            }
        }
    }
}

// Start service in background
fn start_service() {
    if is_running() {
        println!("Service already running!");
        return;
    }

    println!("🚀 Starting Background Services...\n");

    // Set up Ctrl+C handler (uses global RUNNING)
    ctrlc::set_handler(move || {
        println!("\nCtrl+C received. Shutting down...");
        shutdown();
        stop_service();
        // Force terminal cleanup
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stderr(), crossterm::cursor::Show);
        std::process::exit(130);
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
    if let Ok(pid_str) = std::fs::read_to_string(PID_FILE) {
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
    let shell_case_sensitive = {
        if let Ok(settings) = GLOBAL_SETTINGS.lock() {
            settings.shell_case_sensitive
        } else {
            false
        }
    };

    println!("Starting DB writer thread...");

    let _ = &*DB_WRITER;

    if let Ok(mut monitor) = GLOBAL_SHELL_MON.lock() {
        if let Err(e) = monitor.read_all_histories(shell_case_sensitive) {
            eprintln!("Shell error: {}", e);
        }
    }

    // Clipboard thread
    thread::spawn(move || {
        while is_running() {
            let (should_capture, clipboard_case_sensitive) = {
                if let Ok(settings) = GLOBAL_SETTINGS.lock() {
                    (
                        settings.capture_clipboard,
                        settings.clipboard_case_sensitive,
                    )
                } else {
                    (false, false)
                }
            };

            if should_capture {
                // Lock the mutex to get mutable access
                if let Ok(mut monitor) = GLOBAL_CLIP_MON.lock() {
                    if let Err(e) = monitor.check(clipboard_case_sensitive) {
                        eprintln!("Clipboard error: {}", e);
                    }
                }
            }
            thread::sleep(Duration::from_secs(CLIP_SLEEP_DURATION_SECS));
        }
    });

    // Shell thread
    thread::spawn(move || {
        while is_running() {
            let (should_capture, should_capture_files, shell_case_sensitive) = {
                if let Ok(settings) = GLOBAL_SETTINGS.lock() {
                    (
                        settings.capture_shell,
                        settings.capture_shell_history_with_files,
                        settings.shell_case_sensitive,
                    )
                } else {
                    (false, false, false)
                }
            };

            if should_capture && should_capture_files {
                // Lock the mutex to get mutable access
                if let Ok(mut monitor) = GLOBAL_SHELL_MON.lock() {
                    if let Err(e) = monitor.read_all_histories(shell_case_sensitive) {
                        eprintln!("Shell error: {}", e);
                    }
                }
            }

            for _ in 0..60 {
                if !is_running() {
                    break;
                }
                thread::sleep(Duration::from_secs(SHELL_SLEEP_DURATION_SECS));
            }
        }
    });

    // DB writer
    thread::spawn(move || {
        while is_running() {
            let queue_size = DB_WRITER.queue_len();

            // Warn if queue is backing up
            if queue_size > 500 {
                eprintln!("⚠ DB writer queue is large: {} entries pending", queue_size);
            } else if queue_size > 0 {
                // Optional: log normal activity (only in verbose mode)
                #[cfg(debug_assertions)]
                println!("DB writer queue: {} entries", queue_size);
            }

            for _ in 0..60 {
                if !is_running() {
                    break;
                }
                thread::sleep(Duration::from_secs(DB_LOOP_SECS));
            }
        }
    });

    // Main service loop — checks global flag
    let mut last_maintenance = Instant::now();

    let mut daemon_context = DaemonContext {
        iteration: 0,
        uptime_secs: 0,
    };

    while is_running() {
        if let Ok(config) = GLOBAL_CONFIG.read() {
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

    // Avoid capturing our own commands
    if cmd.starts_with(SERVICE_NAME)
        || cmd.starts_with(SERVICE_NAME_SHORT)
        || cmd.starts_with(SERVICE_NAME_SHORT2)
    {
        return;
    }

    // ---- SETTINGS (non-blocking)
    let (should_capture, shell_case_sensitive) = match GLOBAL_SETTINGS.try_lock() {
        Ok(settings) => (settings.capture_shell, settings.shell_case_sensitive),
        Err(_) => return, // lock busy → do nothing
    };

    if !should_capture {
        return;
    }

    // ---- PLUGINS (non-blocking)
    let should_add = match GLOBAL_PLUGIN_MANAGER.try_lock() {
        Ok(plugins) => plugins.trigger_command_captured(&CommandContext {
            command: cmd.to_string(),
            working_dir: pwd.clone().unwrap_or_default(),
            user: user.clone().unwrap_or_default(),
            host: host.clone().unwrap_or_default(),
            timestamp,
        }),
        Err(_) => true, // assume success if busy
    };

    if !should_add {
        return;
    }

    // ---- MONITOR (non-blocking)
    if let Ok(mut monitor) = GLOBAL_SHELL_MON.try_lock() {
        let cmd = if shell_case_sensitive {
            cmd.to_string()
        } else {
            cmd.to_lowercase()
        };

        monitor.add_command(cmd, timestamp, pwd, user, host);
    }
}

fn maintain() {
    let (clipboard_limit, shell_limit) = {
        let settings = GLOBAL_SETTINGS.lock().unwrap();
        (settings.clipboard_limit, settings.shell_limit)
    };

    if let Ok(db) = USER_DB.lock() {
        // Always clean up old entries (this is cheap and frequent)
        if let Err(e) = db.cleanup_old_entries(clipboard_limit, shell_limit) {
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

    print!("Database maintenance completed\n");
}

pub fn reload() {
    if let Err(e) = reload_config() {
        eprintln!("Failed to reload settings: {}", e);
    }
}

fn ask_to_string(resp: AskResponse) -> Option<String> {
    match resp {
        AskResponse::Knowledge(s) => Some(s),
        AskResponse::SearchResults(opt) => opt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_command() {
        initialize_plugins();

        capture_command(
            "ja new stuff",
            Some("/home/user".to_string()),
            Some("user".to_string()),
            Some("host".to_string()),
        );
    }
}
