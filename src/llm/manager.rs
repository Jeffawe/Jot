use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::process::Command;
use reqwest::Client;
use std::sync::Arc;

use super::{LlmModel, default::OllamaModel};
use crate::config::{GLOBAL_CONFIG, LlmConfig};

pub struct LlmManager {
    model: Option<Arc<Box<dyn LlmModel>>>,
    config: LlmConfig,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum LlmError {
    OllamaNotInstalled,
    OllamaNotRunning,
    ModelNotFound(String),
    Other(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::OllamaNotInstalled => write!(f, "Ollama is not installed"),
            LlmError::OllamaNotRunning => write!(f, "Ollama service is not running"),
            LlmError::ModelNotFound(model) => write!(f, "Model '{}' not found", model),
            LlmError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

impl LlmManager {
    pub fn new() -> Self {
        let config = match GLOBAL_CONFIG.lock() {
            Ok(cfg) => cfg.llm.clone(),
            Err(_) => {
                LlmConfig {
                    provider: "ollama".to_string(),
                    api_key: None,
                    max_history_results: 5,
                    model: "llama2".to_string(),
                    api_base: None,
                    max_tokens: 512,
                    temperature: 0.7,
                }
            }
        };

        Self {
            model: None,
            config,
        }
    }
    
    /// Check if Ollama is installed
    pub fn is_ollama_installed(&self) -> bool {
        Command::new("which")
            .arg("ollama")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// Check if Ollama service is running
    pub async fn is_ollama_running(&self) -> bool {
        let api_base = self.config.api_base.clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string());
        
        Client::new()
            .get(&api_base)
            .timeout(std::time::Duration::from_secs(2))
            .send()
             .await 
            .is_ok()
    }
    
    /// Check if the configured model exists locally
    pub fn is_model_available(&self) -> bool {
        Command::new("ollama")
            .args(&["list"])
            .output()
            .map(|output| {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains(&self.config.model)
            })
            .unwrap_or(false)
    }
    
    /// Get or initialize the LLM model
    pub async fn get_llm(&mut self) -> Result<Arc<Box<dyn LlmModel>>, LlmError> {
        // Return existing model if already initialized
        if let Some(ref model) = self.model {
            return Ok(Arc::clone(model));
        }
        
        // Check if Ollama is installed
        if !self.is_ollama_installed() {
            return Err(LlmError::OllamaNotInstalled);
        }
        
        // Check if Ollama is running
        if !self.is_ollama_running().await {
            // Try to start Ollama
            let _ = Command::new("ollama")
                .arg("serve")
                .spawn();
            
            // Wait a bit for it to start
            std::thread::sleep(std::time::Duration::from_secs(2));
            
            if !self.is_ollama_running().await {
                return Err(LlmError::OllamaNotRunning);
            }
        }
        
        // Check if model is available
        if !self.is_model_available() {
            return Err(LlmError::ModelNotFound(self.config.model.clone()));
        }
        
        // Initialize the model
        let api_base = self.config.api_base.clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string());
        
        let model: Box<dyn LlmModel> = Box::new(OllamaModel::new(
            api_base,
            self.config.model.clone(),
        ));
        
        self.model = Some(Arc::new(model));
        Ok(Arc::clone(self.model.as_ref().unwrap()))
    }
    
    /// Interpret a natural language query into search parameters
    pub async fn interpret_query(
        &mut self,
        query: &str,
        directory: &str,
    ) -> Result<super::LLMQueryParams, Box<dyn std::error::Error>> {
        let model = self.get_llm().await?;
        model.interpret_query(
            query,
            directory,
            self.config.max_tokens,
            self.config.temperature,
        ).await
    }
    
    /// Answer a knowledge question directly
    pub async fn answer_question(
        &mut self,
        query: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let model = self.get_llm().await?;
        
        model.answer_question(
            query,
            self.config.max_tokens,
            self.config.temperature,
        ).await
    }
    
    /// Get the current model name
    #[allow(dead_code)]
    pub fn model_name(&self) -> &str {
        &self.config.model
    }
}

pub static GLOBAL_LLM: Lazy<Mutex<LlmManager>> =
    Lazy::new(|| Mutex::new(LlmManager::new()));