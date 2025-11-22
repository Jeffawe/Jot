use reqwest::Client;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

use crate::plugin::{GLOBAL_PLUGIN_MANAGER, LlmContext};
use crate::commands::get_working_directory;

use super::{LlmModel, LLMQueryParams};

pub struct OllamaModel {
    client: Client,
    api_base: String,
    model: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaModel {
    pub fn new(api_base: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_base,
            model,
        }
    }
    
    async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}/api/generate", self.api_base);

        if let Ok(plugins) = GLOBAL_PLUGIN_MANAGER.lock() {
            let context = LlmContext {
                model: self.model.clone(),
                provider: "ollama".to_string(),
                working_dir: get_working_directory(),
            };
            plugins.trigger_llm_before(prompt, &context);
        }
        
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature,
                num_predict: max_tokens,
            },
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Ollama API error {}: {}", status, error_text).into());
        }
        
        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response)
    }
    
    fn build_interpret_prompt(&self, query: &str, directory: &str) -> String {
        format!(
            r#"You are a query interpreter for a digital history search tool.

Database schema:
- entry_type: command, browser, app_activity
- timestamp: unix timestamp
- working_dir, git_repo, git_branch
- content: the actual command/activity

User query: "{}"
Directory the command was run in: "{}"

Analyze the query and output ONLY a valid JSON object (no markdown, no explanation):
{{
  "keywords": ["word1", "word2"],
  "entry_types": ["command"],
  "time_range": "yesterday" | "last_week" | "today" | null,
  "filters": {{
    "working_dir": null,
    "git_repo": null,
    "git_branch": null
  }},
  "use_semantic": true
}}"#,
            query, directory
        )
    }
    
    fn build_answer_prompt(&self, query: &str) -> String {
        format!(
            r#"You are a helpful command-line assistant. Answer this question concisely in 1-2 sentences.

Question: {}

Answer:"#,
            query
        )
    }
}

#[async_trait]
impl LlmModel for OllamaModel {
    async fn interpret_query(
        &self,
        query: &str,
        directory: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<LLMQueryParams, Box<dyn std::error::Error>> {
        let prompt = self.build_interpret_prompt(query, directory);
        let response = self.generate(&prompt, max_tokens, temperature).await?;
        
        // Clean up response (remove markdown code blocks if present)
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        
        // Parse JSON response
        let params: LLMQueryParams = serde_json::from_str(cleaned)
            .map_err(|e| {
                format!("Failed to parse LLM response as JSON: {}\n\nResponse was:\n{}", e, cleaned)
            })?;
        
        Ok(params)
    }
    
    async fn answer_question(
        &self,
        query: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = self.build_answer_prompt(query);
        self.generate(&prompt, max_tokens, temperature).await
    }
    
    fn model_name(&self) -> &str {
        &self.model
    }
    
    async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}