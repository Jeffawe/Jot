use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

// The main config struct - mirrors your TOML file structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub llm: LlmConfig,
    pub search: SearchConfig,
    pub storage: StorageConfig,
    pub privacy: PrivacyConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LlmConfig {
    pub provider: String, // "ollama", "openai", "anthropic"
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub max_history_results: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PrivacyConfig {
    pub excludes_contains_string: Vec<String>,
    pub excludes_starts_with_string: Vec<String>,
    pub excludes_ends_with_string: Vec<String>,
    pub excludes_regex: Vec<String>,
    pub exclude_folders: Vec<String>,
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
        let contains_string = vec![
            "./target/debug/jotx".to_string(),
            "jotx.exe".to_string(),
            "password".to_string(),
        ];
        let starts_with_string = vec![
            "jotx.exe".to_string(),
            "jotx".to_string(),
            "ja".to_string(),
            "js".to_string(),
        ];
        let folder_excludes = vec![".git".to_string(), "node_modules".to_string()];
        Config {
            llm: LlmConfig {
                provider: "ollama".to_string(),
                api_key: None,
                api_base: Some("http://localhost:11434".to_string()),
                model: "qwen2.5:3b".to_string(),
                max_tokens: 500,
                temperature: 0.3,
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
            privacy: PrivacyConfig {
                excludes_contains_string: contains_string,
                excludes_starts_with_string: starts_with_string,
                excludes_ends_with_string: vec![],
                excludes_regex: vec![],
                exclude_folders: folder_excludes,
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
        let config: Config =
            toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

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

    pub fn update_privacy_settings(
        &mut self,
        privacy: PrivacyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.privacy = privacy;
        self.save()?;
        Ok(())
    }

    pub fn update_llm_model(&mut self, model: String) -> Result<(), Box<dyn std::error::Error>> {
        self.llm.model = model;
        self.save()?;
        Ok(())
    }
}

// Global config singleton
pub static GLOBAL_CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| {
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("⚠️  Failed to load config: {}, using defaults", e);
        Config::default()
    });
    RwLock::new(config)
});

pub fn reload_config() -> Result<(), Box<dyn std::error::Error>> {
    GLOBAL_CONFIG.write().unwrap().reload()
}

pub fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".jotx").join("config.toml")
}
