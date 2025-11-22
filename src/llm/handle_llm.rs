use std::process::Command;
use colored::*;
use reqwest::Client;

pub async fn handle_llm() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║        JotX LLM Management             ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();
    
    // Check current status
    println!("{}", "Current Status:".yellow());
    
    let ollama_installed = Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    if ollama_installed {
        println!("  {} Ollama installed", "✓".green());
        
        // Check if running
        let running = Client::new()
            .get("http://localhost:11434")
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .is_ok();
        
        if running {
            println!("  {} Ollama service running", "✓".green());
        } else {
            println!("  {} Ollama service not running", "✗".red());
        }
        
        // List installed models
        if let Ok(output) = Command::new("ollama").arg("list").output() {
            let models = String::from_utf8_lossy(&output.stdout);
            println!("\n{}", "Installed Models:".yellow());
            for line in models.lines().skip(1) {
                if !line.trim().is_empty() {
                    println!("  • {}", line.split_whitespace().next().unwrap_or(""));
                }
            }
        }
    } else {
        println!("  {} Ollama not installed", "✗".red());
    }
    
    println!();
    println!("{}", "Available Actions:".yellow());
    println!("  1) Install/Setup Ollama");
    println!("  2) List available models");
    println!("  3) Download a model");
    println!("  4) Remove a model");
    println!("  5) Change default model");
    println!("  6) Start Ollama service");
    println!("  0) Exit");
    println!();
    
    print!("Select an option: ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim() {
        "1" => install_ollama()?,
        "2" => list_available_models()?,
        "3" => download_model()?,
        "4" => remove_model()?,
        "5" => change_default_model()?,
        "6" => start_ollama_service()?,
        "0" => println!("Goodbye!"),
        _ => println!("Invalid option"),
    }
    
    Ok(())
}

fn install_ollama() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "Running installation script...".cyan());
    
    let script_path = concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/install-ollama.sh");
    
    let status = Command::new("bash")
        .arg(script_path)
        .status()?;
    
    if status.success() {
        println!("\n{}", "✓ Installation complete!".green());
    } else {
        println!("\n{}", "✗ Installation failed".red());
    }
    
    Ok(())
}

fn list_available_models() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "Popular Small Models (Fast & Efficient):".yellow());
    println!("  • qwen2:0.5b        ~300MB  (Fastest, recommended)");
    println!("  • tinyllama         ~600MB  (Good balance)");
    println!("  • llama3.2:1b       ~1GB    (Better quality)");
    println!("  • llama3.2:3b       ~2GB    (High quality)");
    println!("\nTo download a model, select option 3");
    
    Ok(())
}

fn download_model() -> Result<(), Box<dyn std::error::Error>> {
    print!("\nEnter model name (e.g., qwen2:0.5b): ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut model = String::new();
    io::stdin().read_line(&mut model)?;
    let model = model.trim();
    
    println!("\n{} {}", "Downloading".cyan(), model);
    
    let status = Command::new("ollama")
        .arg("pull")
        .arg(model)
        .status()?;
    
    if status.success() {
        println!("\n{} Model downloaded!", "✓".green());
    } else {
        println!("\n{} Download failed", "✗".red());
    }
    
    Ok(())
}

fn remove_model() -> Result<(), Box<dyn std::error::Error>> {
    // List current models first
    Command::new("ollama").arg("list").status()?;
    
    print!("\nEnter model name to remove: ");
    use std::io::{self, Write};
    io::stdout().flush()?;
    
    let mut model = String::new();
    io::stdin().read_line(&mut model)?;
    let model = model.trim();
    
    let status = Command::new("ollama")
        .arg("rm")
        .arg(model)
        .status()?;
    
    if status.success() {
        println!("\n{} Model removed!", "✓".green());
    } else {
        println!("\n{} Removal failed", "✗".red());
    }
    
    Ok(())
}

fn change_default_model() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "This would update your config file".yellow());
    println!("TODO: Implement config file editing");
    // TODO: Load config, change model field, save config
    
    Ok(())
}

fn start_ollama_service() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}", "Starting Ollama service...".cyan());
    
    Command::new("ollama")
        .arg("serve")
        .spawn()?;
    
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    println!("{}", "✓ Ollama service started".green());
    
    Ok(())
}