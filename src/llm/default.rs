use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::commands::get_working_directory;
use crate::db::{SampleSelector, SampleStrategy};
use crate::llm::prompt::AdaptivePromptBuilder;
use crate::plugin::{GLOBAL_PLUGIN_MANAGER, LlmContext};

use super::{LLMQueryParams, LlmModel};

pub struct OllamaModel {
    client: Client,
    api_base: String,
    model: String,
    prompt_builder: AdaptivePromptBuilder,
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
            prompt_builder: AdaptivePromptBuilder::new(model.clone()),
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

        let response = self
            .client
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

    // fn build_interpret_prompt(&self, query: &str, directory: &str) -> String {
    //     format!(
    //         r#"Convert query to JSON. Output ONLY valid JSON.
    //         Format:
    //         {{"keywords":[],"entry_types":null,"time_range":null,"custom_start":null,"custom_end":null,"filters":{{"working_dir":null,"app_name":null}},"use_semantic":false}}
    //         Rules:
    //         - entry_types: "shell", "clipboard", or null
    //         - time_range: "today", "yesterday", "last_week", "last_month", or null
    //         - use_semantic: true if vague, false if specific
    //         Examples:
    //         "git commit yesterday" → {{"keywords":["git","commit"],"entry_types":"shell","time_range":"yesterday","custom_start":null,"custom_end":null,"filters":{{"working_dir":null,"app_name":null}},"use_semantic":false}}
    //         "connect server" → {{"keywords":["ssh"],"entry_types":"shell","time_range":null,"custom_start":null,"custom_end":null,"filters":{{"working_dir":null,"app_name":null}},"use_semantic":true}}
    //         "copied rust code" → {{"keywords":["rust"],"entry_types":"clipboard","time_range":null,"custom_start":null,"custom_end":null,"filters":{{"working_dir":null,"app_name":null}},"use_semantic":true}}
    //         Query: "{query}"
    //         Directory: "{directory}"
    //         JSON:"#,
    //                     query = query,
    //                     directory = directory
    //                 )
    // }

    fn build_interpret_prompt(&self, query: &str, directory: &str) -> String {
        let sample_count = self.prompt_builder.get_recommended_sample_count();
        let mut sample_selector = SampleSelector {};
        let samples = sample_selector
            .get_samples(query, sample_count, SampleStrategy::Adaptive)
            .unwrap_or_default();
        let prompt = self.prompt_builder.build_prompt(query, directory, &samples);
        prompt
    }

    fn build_answer_prompt(&self, query: &str) -> String {
        format!(
            r#"You are a helpful command-line assistant. Answer this question concisely in 1-2 sentences. If the question requires a simple command answer. Give the command only.

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

        println!("Prompt: {}", prompt);
        let response = self.generate(&prompt, max_tokens, temperature).await?;

        // More aggressive cleaning
        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            // Remove any text before the first {
            .split_once('{')
            .map(|(_, after)| format!("{{{}", after))
            .unwrap_or(response.to_string())
            // Remove any text after the last }
            .rsplit_once('}')
            .map(|(before, _)| format!("{}}}", before))
            .unwrap_or(response.to_string());

        // Parse JSON response
        let params: LLMQueryParams = serde_json::from_str(&cleaned).map_err(|e| {
        format!(
            "Failed to parse LLM response as JSON: {}\n\nCleaned response:\n{}\n\nOriginal response:\n{}",
            e, cleaned, response
        )
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

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::*;

    #[tokio::test]
    async fn test_ollama_generate() {
        let config = Config::default();
        let model = OllamaModel::new(
            config.llm.api_base.unwrap().to_string(),
            config.llm.model.clone(),
        );

        let result = model.generate("Say hello in one word", 50, 0.7).await;

        match result {
            Ok(response) => println!("Response: {}", response),
            Err(e) => println!("Error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_interpret_query() {
        let config = Config::default();
        let model = OllamaModel::new(
            config.llm.api_base.unwrap().to_string(),
            config.llm.model.clone(),
        );

        let result = model
            .interpret_query(
                "find all git commands from yesterday",
                "/home/user/project",
                500,
                0.3,
            )
            .await;

        match result {
            Ok(params) => {
                println!("Keywords: {:?}", params.keywords);
                println!("Time range: {:?}", params.time_range);
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_answer_question() {
        let config = Config::default();
        let model = OllamaModel::new(
            config.llm.api_base.unwrap().to_string(),
            config.llm.model.clone(),
        );

        let result = model
            .answer_question("How do I list files in a directory?", 100, 0.7)
            .await;

        match result {
            Ok(answer) => println!("Answer: {}", answer),
            Err(e) => println!("Error: {}", e),
        }
    }
}
