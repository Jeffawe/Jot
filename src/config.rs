use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use once_cell::sync::Lazy;
use std::sync::{Mutex};

// The main config struct - mirrors your TOML file structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub llm: LlmConfig,
    pub search: SearchConfig,
    pub storage: StorageConfig
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmConfig {
    pub provider: String,           // "ollama", "openai", "anthropic"
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub max_history_results: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchConfig {
    pub similarity_threshold: f32,
    pub max_results: usize,
    pub fuzzy_matching: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageConfig {
    pub maintenance_interval_days: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            llm: LlmConfig {
                provider: "ollama".to_string(),
                api_key: None,
                api_base: Some("http://localhost:11434".to_string()),
                model: "llama2".to_string(),
                max_tokens: 500,
                temperature: 0.7,
                max_history_results: 10,
            },
            search: SearchConfig {
                similarity_threshold: 0.5,
                max_results: 10,
                fuzzy_matching: true,
            },
            storage: StorageConfig {
                maintenance_interval_days: 7,
            },
        }
    }
}

impl Config {
    /// Load config from file, create default if doesn't exist
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        
        // If config doesn't exist, create default
        if !config_path.exists() {
            let default = Config::default();
            default.save()?;
            return Ok(default);
        }
        
        // Read and parse TOML
        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        
        Ok(config)
    }
    
    /// Save config to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Serialize to pretty TOML
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        
        Ok(())
    }
    
    /// Get the config file path
    fn get_config_path() -> PathBuf {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home).join(".jotx").join("config.toml")
    }
    
    /// Reload config from disk (useful for hot-reloading)
    pub fn reload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        *self = Config::load()?;
        Ok(())
    }
}

// Global config singleton
pub static GLOBAL_CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("⚠️  Failed to load config: {}, using defaults", e);
        Config::default()
    });
    Mutex::new(config)
});

// pub fn get_config() -> MutexGuard<'static, Config> {
//     GLOBAL_CONFIG.lock().unwrap()
// }

pub fn reload_config() -> Result<(), Box<dyn std::error::Error>> {
    GLOBAL_CONFIG.lock().unwrap().reload()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.llm.provider, "ollama");
    }
    
    #[test]
    fn test_save_and_load() {
        let config = Config::default();
        config.save().unwrap();
        
        let loaded = Config::load().unwrap();
        assert_eq!(loaded.llm.model, config.llm.model);
    }
}