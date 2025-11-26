use async_trait::async_trait;
use serde::{Deserialize, Serialize};
mod default;
mod handle_llm;
mod manager;

pub use handle_llm::{
    download_model_with_string, handle_llm, install_ollama, remove_model_with_string,
    start_ollama_service,
};
pub use manager::GLOBAL_LLM;

/// Query parameters that the LLM extracts from natural language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMQueryParams {
    pub keywords: Vec<String>,
    #[serde(default)]
    pub entry_types: Option<String>,
    #[serde(default)]
    pub time_range: Option<SimpleTimeRange>,
    #[serde(default)]
    pub custom_start: Option<i64>,
    #[serde(default)]
    pub custom_end: Option<i64>,
    #[serde(default)]
    pub filters: Option<QueryFilters>,
    #[serde(default)]
    pub use_semantic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")] // FIX: Allows "yesterday" to match "Yesterday"
pub enum SimpleTimeRange {
    Today,
    Yesterday,
    LastWeek,
    LastMonth,
    Custom, // No data here, just a marker
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilters {
    pub working_dir: Option<String>,
    pub app_name: Option<String>,
}

/// Base trait for LLM implementations
#[async_trait]
#[allow(dead_code)]
pub trait LlmModel: Send + Sync {
    /// Initialize the model (load weights, etc.)
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    /// Interpret a natural language query into structured search params
    async fn interpret_query(
        &self,
        query: &str,
        directory: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<LLMQueryParams, Box<dyn std::error::Error>>;

    /// Answer a knowledge question directly
    async fn answer_question(
        &self,
        query: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Get model identifier
    fn model_name(&self) -> &str;
}
