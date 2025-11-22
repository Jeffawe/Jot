use serde::{Deserialize, Serialize};
use async_trait::async_trait;
mod default;
mod manager;
mod handle_llm;

pub use manager::GLOBAL_LLM;
pub use handle_llm::handle_llm;

/// Query parameters that the LLM extracts from natural language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMQueryParams {
    pub keywords: Vec<String>,
    pub entry_types: Option<Vec<String>>,
    pub time_range: Option<TimeRange>,
    pub filters: Option<QueryFilters>,
    pub use_semantic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeRange {
    Today,
    Yesterday,
    LastWeek,
    LastMonth,
    Custom { start: i64, end: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilters {
    pub working_dir: Option<String>,
    pub git_repo: Option<String>,
    pub git_branch: Option<String>,
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