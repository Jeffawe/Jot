// prompt_builder.rs
use crate::db::Sample;

pub struct AdaptivePromptBuilder {
    model_params: ModelSize,
    few_shot_cache: FewShotCache,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModelSize {
    Tiny,      // <1B params (0.5B, 1B)
    Small,     // 1-3B params (1.5B, 3B)
    Medium,    // 3-8B params (7B, 8B)
    Large,     // 8B+ params (13B, 70B, etc.)
}

struct FewShotCache {
    examples: Vec<FewShotExample>,
    max_size: usize,
}

#[derive(Debug, Clone)]
struct FewShotExample {
    query: String,
    keywords: Vec<String>,
    entry_type: Option<String>,
    time_range: Option<String>,
    use_semantic: bool,
    success_rate: f32,
    usage_count: u32,
}

impl AdaptivePromptBuilder {
    pub fn new(model_name: String) -> Self {
        let model_params = Self::detect_model_size(&model_name);
        let max_cache = match model_params {
            ModelSize::Tiny => 3,
            ModelSize::Small => 5,
            ModelSize::Medium => 10,
            ModelSize::Large => 20,
        };
        
        Self {
            model_params,
            few_shot_cache: FewShotCache {
                examples: Vec::new(),
                max_size: max_cache,
            },
        }
    }
    
    /// Detect model size from name
    fn detect_model_size(model_name: &str) -> ModelSize {
        let name_lower = model_name.to_lowercase();
        
        // Extract parameter count from common patterns
        let param_patterns = [
            (r"0\.5b", ModelSize::Tiny),
            (r"1\.5b", ModelSize::Small),
            (r"3b", ModelSize::Small),
            (r"7b", ModelSize::Medium),
            (r"8b", ModelSize::Medium),
            (r"13b", ModelSize::Large),
            (r"70b", ModelSize::Large),
        ];
        
        for (pattern, size) in param_patterns {
            if name_lower.contains(pattern) {
                return size;
            }
        }
        
        // Default to Small for unknown models
        ModelSize::Small
    }
    
    /// Build adaptive prompt with samples
    pub fn build_prompt(
        &self,
        query: &str,
        directory: &str,
        samples: &[Sample],
    ) -> String {
        match self.model_params {
            ModelSize::Tiny => self.build_tiny_prompt(query, samples),
            ModelSize::Small => self.build_small_prompt(query, directory, samples),
            ModelSize::Medium => self.build_medium_prompt(query, directory, samples),
            ModelSize::Large => self.build_large_prompt(query, directory, samples),
        }
    }
    
    /// Minimal prompt for tiny models (<1B) - focus on essentials only
    fn build_tiny_prompt(&self, query: &str, samples: &[Sample]) -> String {
        let samples_text = if samples.len() > 3 {
            self.format_samples_compact(&samples[..3])
        } else {
            self.format_samples_compact(samples)
        };
        
        format!(
r#"Output JSON only. No text before or after.

Format: {{"keywords":[],"time_range":null}}

Examples in history:
{}

Query: "{}"

JSON:"#,
            samples_text,
            query
        )
    }
    
    /// Compact prompt for small models (1-3B)
    fn build_small_prompt(&self, query: &str, directory: &str, samples: &[Sample]) -> String {
        let samples_text = self.format_samples_compact(&samples[..samples.len().min(5)]);
        
        format!(
r#"Convert query to JSON. Output ONLY valid JSON, no other text.

Format:
{{"keywords":[],"time_range":null,"filters":{{"working_dir":null}}}}

Rules:
- keywords: Array of search terms (expand abbreviations, e.g., "push code" → ["git", "push"])
- time_range: "today", "yesterday", "last_week", "last_month" or leave null if not applies
- filters.working_dir: Directory context (use if query mentions location)

Similar commands in history:
{}

Query: "{}"
Current directory: "{}"
JSON:"#,
            samples_text,
            query,
            directory
        )
    }
    
    /// Balanced prompt for medium models (3-8B)
    fn build_medium_prompt(&self, query: &str, directory: &str, samples: &[Sample]) -> String {
        let samples_text = self.format_samples_detailed(&samples[..samples.len().min(8)]);
        
        let few_shot = self.get_best_few_shot_examples(5);
        let few_shot_text = if !few_shot.is_empty() {
            format!("Learned patterns from past searches:\n{}\n", self.format_few_shot(&few_shot))
        } else {
            String::new()
        };
        
        format!(
r#"Convert the natural language query into structured search parameters. Return ONLY valid JSON.

Output format:
{{"keywords":[],"time_range":null,"custom_start":null,"custom_end":null,"filters":{{"working_dir":null,"app_name":null}},"use_semantic":false}}

Field definitions:
- keywords: Array of search terms (expand abbreviations, e.g., "push code" → ["git", "push"])
- time_range: "today", "yesterday", "last_week", "last_month", or null
- use_semantic: true for vague queries (should only be true if entry type is clipboard)

{}Commands in user's history (for context):
{}

Current directory: {}
User query: "{}"

JSON output:"#,
            few_shot_text,
            samples_text,
            directory,
            query
        )
    }
    
    /// Comprehensive prompt for large models (8B+)
    fn build_large_prompt(&self, query: &str, directory: &str, samples: &[Sample]) -> String {
        let samples_text = self.format_samples_detailed(&samples[..samples.len().min(15)]);
        
        let few_shot = self.get_best_few_shot_examples(10);
        let few_shot_text = if !few_shot.is_empty() {
            format!("Successfully learned query patterns:\n{}\n", self.format_few_shot_detailed(&few_shot))
        } else {
            String::new()
        };
        
        format!(
r#"You are a terminal history search assistant. Convert natural language queries into structured search parameters.

Output format (JSON only, no additional text):
{{"keywords":[],"time_range":null,"custom_start":null,"custom_end":null,"filters":{{"working_dir":null,"app_name":null}},"use_semantic":false}}

Parameter specifications:
- keywords: Extract search terms. Expand common abbreviations (e.g., "push code" → ["git", "push", "origin"])
- time_range: Temporal filter - "today", "yesterday", "last_week", "last_month", or null
- custom_start/custom_end: Unix timestamps for custom date ranges (usually null)
- filters.working_dir: Directory context (use if query mentions location)
- filters.app_name: Application filter (use if query mentions specific app)
- use_semantic: Set to true for vague/abstract queries (should only be true if it's a vague clipboard entry type)

{}User's command history context (top similar commands):
{}

Current working directory: {}
User query: "{}"

Analysis: Consider the query intent and historical patterns to generate optimal search parameters.

JSON output:"#,
            few_shot_text,
            samples_text,
            directory,
            query
        )
    }
    
    /// Format samples in compact form (for tiny/small models)
    fn format_samples_compact(&self, samples: &[Sample]) -> String {
        samples
            .iter()
            .take(5)
            .map(|s| format!("- {}", s.command))
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Format samples with detail (for medium/large models)
    fn format_samples_detailed(&self, samples: &[Sample]) -> String {
        samples
            .iter()
            .enumerate()
            .map(|(i, s)| {
                format!(
                    "{}. {} (used {} times, {:.0}% match)",
                    i + 1,
                    s.command,
                    s.quality_score as i32,
                    s.similarity * 100.0
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Format few-shot examples
    fn format_few_shot(&self, examples: &[FewShotExample]) -> String {
        examples
            .iter()
            .map(|ex| {
                format!(
                    "\"{}\" → {{\"keywords\":{:?},\"entry_types\":{:?},\"time_range\":{:?},\"use_semantic\":{}}}",
                    ex.query,
                    ex.keywords,
                    ex.entry_type,
                    ex.time_range,
                    ex.use_semantic
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Format few-shot with success rates (for large models)
    fn format_few_shot_detailed(&self, examples: &[FewShotExample]) -> String {
        examples
            .iter()
            .map(|ex| {
                format!(
                    "Query: \"{}\"\nOutput: {{\"keywords\":{:?},\"entry_types\":{:?},\"time_range\":{:?},\"use_semantic\":{}}}\nSuccess rate: {:.0}% ({} uses)\n",
                    ex.query,
                    ex.keywords,
                    ex.entry_type,
                    ex.time_range,
                    ex.use_semantic,
                    ex.success_rate * 100.0,
                    ex.usage_count
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Get best performing few-shot examples
    fn get_best_few_shot_examples(&self, n: usize) -> Vec<FewShotExample> {
        let mut examples = self.few_shot_cache.examples.clone();
        
        // Sort by success_rate * log(usage_count) to balance quality and popularity
        examples.sort_by(|a, b| {
            let score_a = a.success_rate * (a.usage_count as f32).ln().max(1.0);
            let score_b = b.success_rate * (b.usage_count as f32).ln().max(1.0);
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        examples.into_iter().take(n).collect()
    }
    
    #[allow(dead_code)]
    /// Add a new few-shot example from successful search
    pub fn add_example(
        &mut self,
        query: String,
        keywords: Vec<String>,
        entry_type: Option<String>,
        time_range: Option<String>,
        use_semantic: bool,
        success: bool,
    ) {
        // Check if example already exists
        if let Some(existing) = self.few_shot_cache.examples
            .iter_mut()
            .find(|ex| ex.query == query)
        {
            // Update existing example
            existing.usage_count += 1;
            let alpha = 0.3; // Exponential moving average
            let new_success = if success { 1.0 } else { 0.0 };
            existing.success_rate = alpha * new_success + (1.0 - alpha) * existing.success_rate;
        } else {
            // Add new example
            self.few_shot_cache.examples.push(FewShotExample {
                query,
                keywords,
                entry_type,
                time_range,
                use_semantic,
                success_rate: if success { 1.0 } else { 0.5 },
                usage_count: 1,
            });
        }
        
        // Sort and trim to max size
        self.few_shot_cache.examples.sort_by(|a, b| {
            let score_a = a.success_rate * (a.usage_count as f32).ln().max(1.0);
            let score_b = b.success_rate * (b.usage_count as f32).ln().max(1.0);
            score_b.partial_cmp(&score_a).unwrap()
        });
        
        self.few_shot_cache.examples.truncate(self.few_shot_cache.max_size);
    }
    
    /// Get recommended sample count based on model size
    pub fn get_recommended_sample_count(&self) -> usize {
        match self.model_params {
            ModelSize::Tiny => 3,
            ModelSize::Small => 5,
            ModelSize::Medium => 8,
            ModelSize::Large => 15,
        }
    }
    
    #[allow(dead_code)]
    /// Persist few-shot cache to database
    pub fn save_to_db(&self, conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {        
        for example in &self.few_shot_cache.examples {
            let keywords_json = serde_json::to_string(&example.keywords)?;
            conn.execute(
                "INSERT OR REPLACE INTO prompt_examples 
                 (query, keywords, entry_type, time_range, use_semantic, success_rate, usage_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    &example.query,
                    keywords_json,
                    &example.entry_type,
                    &example.time_range,
                    example.use_semantic,
                    example.success_rate,
                    example.usage_count,
                ],
            )?;
        }
        
        Ok(())
    }
    
    #[allow(dead_code)]
    /// Load few-shot cache from database
    pub fn load_from_db(&mut self, conn: &rusqlite::Connection) -> Result<(), Box<dyn std::error::Error>> {
        let mut stmt = conn.prepare(
            "SELECT query, keywords, entry_type, time_range, use_semantic, success_rate, usage_count
             FROM prompt_examples
             ORDER BY success_rate * (usage_count + 1) DESC
             LIMIT ?1"
        )?;
        
        self.few_shot_cache.examples = stmt
            .query_map([self.few_shot_cache.max_size], |row| {
                let keywords_json: String = row.get(1)?;
                let keywords: Vec<String> = serde_json::from_str(&keywords_json)
                    .unwrap_or_default();
                
                Ok(FewShotExample {
                    query: row.get(0)?,
                    keywords,
                    entry_type: row.get(2)?,
                    time_range: row.get(3)?,
                    use_semantic: row.get(4)?,
                    success_rate: row.get(5)?,
                    usage_count: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(())
    }
}