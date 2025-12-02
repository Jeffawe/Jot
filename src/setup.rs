// src/setup.rs
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

const SETUP_HOOK_SCRIPT: &str = include_str!("scripts/setup_hook.sh");
const INSTALL_LLM_SCRIPT: &str = include_str!("scripts/install_llm.sh");

// ============================================================================
// INSTALL (make install)
// ============================================================================
pub fn install() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Installing jotx...");

    // Since we're already a binary, this just means:
    // 1. Copy self to a location in PATH
    // 2. Make executable

    let current_exe = std::env::current_exe()?;
    let install_dir = PathBuf::from(std::env::var("HOME")?).join(".local/bin");

    fs::create_dir_all(&install_dir)?;

    let target = install_dir.join("jotx");

    if current_exe.canonicalize()? == target.canonicalize().unwrap_or_default() {
        println!("✅ Already installed at: {}", target.display());
        return Ok(());
    }

    // Copy binary to install location
    fs::copy(&current_exe, &target)?;

    // Make executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target, perms)?;
    }

    println!("✅ Installed to: {}", target.display());

    // Check if in PATH
    let path = std::env::var("PATH")?;
    if !path.contains(".local/bin") {
        println!("\n⚠️  ~/.local/bin is not in your PATH");
        println!("Add this to your ~/.bashrc or ~/.zshrc:");
        println!("  export PATH=\"$HOME/.local/bin:$PATH\"");
    }

    Ok(())
}

// ============================================================================
// SETUP HOOKS (make hooks)
// ============================================================================
pub fn setup_hooks() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Setting up shell hooks...");

    // Write the embedded script to a temp file
    let temp_script = "/tmp/jotx_setup_hook.sh";
    fs::write(temp_script, SETUP_HOOK_SCRIPT)?;

    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(temp_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(temp_script, perms)?;
    }

    // Run the script
    let status = Command::new("bash").arg(temp_script).status()?;

    // Clean up temp file
    let _ = fs::remove_file(temp_script);

    if status.success() {
        println!("✅ Hooks installed");
        println!(
            "Please run: source ~/.zshrc  (or ~/.bashrc) for all terminal sessions or restart your terminal"
        );
        Ok(())
    } else {
        Err("Failed to setup hooks".into())
    }
}

// ============================================================================
// INSTALL LLM (make install-llm)
// ============================================================================
pub fn install_llm(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Jotx requires Ollama to work. We will install it for you!");
    println!();

    if !force {
        // Try to read from /dev/tty (actual terminal) instead of stdin
        #[cfg(unix)]
        {
            use std::fs::File;
            use std::io::Read;

            print!("Continue with installation? (Y/n) ");
            io::stdout().flush()?;

            // Try to open /dev/tty for interactive input
            if let Ok(mut tty) = File::open("/dev/tty") {
                let mut input = String::new();
                let mut buf = [0u8; 1024];

                if let Ok(n) = tty.read(&mut buf) {
                    input = String::from_utf8_lossy(&buf[..n]).to_string();
                }

                // Default to Yes, only cancel if explicitly 'n' or 'N'
                if input.trim().eq_ignore_ascii_case("n") {
                    println!("❌ Cancelled");
                    println!("   You can install later with: jotx handle-llm");
                    return Ok(());
                }
            } else {
                // If /dev/tty is not available (truly non-interactive), auto-proceed
                println!("🤖 Running in non-interactive mode, proceeding with installation...");
            }
        }

        #[cfg(not(unix))]
        {
            // Fallback for non-Unix systems
            print!("Continue with installation? (Y/n) ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().eq_ignore_ascii_case("n") {
                println!("❌ Cancelled");
                println!("   You can install later with: jotx handle-llm");
                return Ok(());
            }
        }
    }

    // Write embedded script to temp file
    let temp_script = "/tmp/jotx_install_llm.sh";
    fs::write(temp_script, INSTALL_LLM_SCRIPT)?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(temp_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(temp_script, perms)?;
    }

    // Run script
    let status = Command::new("bash").arg(temp_script).status()?;

    // Clean up
    let _ = fs::remove_file(temp_script);

    if status.success() {
        println!();
        println!("✅ LLM setup complete! You can now use: jotx ask <query>");
        Ok(())
    } else {
        println!();
        println!("❌ Installation failed. Try running:");
        println!("   jotx handle-llm");
        Err("LLM installation failed".into())
    }
}

// ============================================================================
// SETUP (make setup) - Full installation
// ============================================================================
pub fn full_setup(force: bool, gui: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !gui {
        // 1. Install binary (if not already)
        install()?;
        println!();
    }

    // 2. Setup hooks
    setup_hooks()?;
    println!();

    // 3. Install LLM
    install_llm(force)?;
    println!();

    // 4. Create jotx directory and save path
    let jotx_dir = PathBuf::from(std::env::var("HOME")?).join(".jotx");
    fs::create_dir_all(&jotx_dir)?;

    let current_dir = std::env::current_dir()?;
    fs::write(
        jotx_dir.join("path"),
        current_dir.to_string_lossy().as_bytes(),
    )?;
    println!("✅ Full setup complete!");

    Ok(())
}

// ============================================================================
// CLEAN (make clean)
// ============================================================================
pub fn clean() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 Cleaning build artifacts...");

    // Remove temp files
    let _ = fs::remove_file("/tmp/jotx.pid");
    let _ = fs::remove_file("/tmp/jotx.log");
    let _ = fs::remove_file("/tmp/jotx.err");

    println!("✅ Clean complete");
    Ok(())
}

// ============================================================================
// CLEAN DATA (make clean-data)
// ============================================================================
pub fn clean_data(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !force {
        println!("⚠️  This will delete all stored data!");
        print!("Are you sure? (y/N) ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("❌ Cancelled");
            return Ok(());
        }
    }

    let jotx_dir = PathBuf::from(std::env::var("HOME")?).join(".jotx");

    if jotx_dir.exists() {
        fs::remove_dir_all(jotx_dir)?;
        println!("✅ Data deleted");
    } else {
        println!("ℹ️  No data directory found");
    }

    Ok(())
}

// ============================================================================
// REMOVE HOOKS (for uninstall)
// ============================================================================
pub fn remove_hooks() -> Result<(), Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;

    // Remove from .zshrc
    let zshrc = PathBuf::from(&home).join(".zshrc");
    if zshrc.exists() {
        remove_hooks_from_file(&zshrc)?;
    }

    // Remove from .bashrc
    let bashrc = PathBuf::from(&home).join(".bashrc");
    if bashrc.exists() {
        remove_hooks_from_file(&bashrc)?;
    }

    Ok(())
}

fn remove_hooks_from_file(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;

    if !content.contains("# JOTX_START") {
        return Ok(());
    }

    // Create backup
    let backup_name = format!(
        "{}.backup.{}",
        path.display(),
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    fs::copy(path, &backup_name)?;

    // Remove hooks
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip = false;

    for line in lines {
        if line.contains("# JOTX_START") {
            skip = true;
        } else if line.contains("# JOTX_END") {
            skip = false;
            continue;
        }

        if !skip {
            new_lines.push(line);
        }
    }

    fs::write(path, new_lines.join("\n"))?;
    println!("✅ Removed hooks from {:?}", path);

    Ok(())
}

// ============================================================================
// UNINSTALL (make uninstall)
// ============================================================================
pub fn uninstall(force: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !force {
        println!("⚠️  This will completely uninstall jotx!");
        print!("Are you sure? (y/N) ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("❌ Cancelled");
            return Ok(());
        }
    }

    println!("🗑️  Uninstalling jotx...");

    // Stop daemon
    Command::new("jotx")
        .arg("exit")
        .status()
        .expect("failed to stop daemon");

    // Clean build artifacts
    clean()?;

    // Clean data
    clean_data(true)?;

    // Remove hooks
    remove_hooks()?;

    // Remove binary
    let install_path = PathBuf::from(std::env::var("HOME")?).join(".local/bin/jotx");

    if install_path.exists() {
        fs::remove_file(&install_path)?;
        println!("✅ Removed binary from {}", install_path.display());
    }

    println!();
    println!("✅ Uninstall complete");
    println!("   Run 'source ~/.zshrc' (or ~/.bashrc) to reload your shell");

    Ok(())
}

pub fn update() -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    println!("📦 Downloading latest version...");

    let status = Command::new("bash")
        .arg("-c")
        .arg("curl -fsSL https://raw.githubusercontent.com/Jeffawe/Jot/main/install.sh | bash")
        .status()?;

    if status.success() {
        println!("✅ Update complete!");
        println!("Restart jotx with: jotx run");
    } else {
        return Err("Update failed".into());
    }

    Ok(())
}
