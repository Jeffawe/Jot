use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFingerprint {
    pub query: String,
    pub keywords: HashSet<String>,
    pub temporal: Option<Temporal>,
    pub embedding: Vec<f32>, // 384 floats ~1.5KB
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Temporal {
    Today,
    Yesterday,
    LastWeek,
    LastMonth,
    Relative { days_ago: i64 }, // For "3 days ago"
}

impl QueryFingerprint {
    pub fn new(
        query: &str,
        embedding: Vec<f32>,
    ) -> Self {
        let query_lower = query.to_lowercase();
        Self {
            query: query_lower.clone(),
            keywords: extract_keywords(&query_lower),
            temporal: extract_temporal(&query_lower),
            embedding,
        }
    }

    /// Calculate similarity score between two fingerprints (0.0 to 1.0)
    pub fn similarity(&self, other: &QueryFingerprint) -> f32 {
        let mut score = 0.0;
        
        // Embedding similarity (60% weight - most important)
        let embedding_sim = cosine_similarity(&self.embedding, &other.embedding);
        score += embedding_sim * 0.6;
        
        // Keyword overlap (30% weight)
        let keyword_sim = jaccard_similarity(&self.keywords, &other.keywords);
        score += keyword_sim * 0.3;
        
        // Temporal match (10% weight)
        if self.temporal == other.temporal {
            score += 0.1;
        }
        
        score
    }
}

fn extract_keywords(query: &str) -> HashSet<String> {
    let stop_words: HashSet<&str> = [
        "the", "a", "an", "i", "me", "my", "from", "in", "on", "at", "show", "find", "get", "list",
        "give", "used", "ran", "did",
    ]
    .iter()
    .cloned()
    .collect();

    query
        .split_whitespace()
        .filter(|w| !stop_words.contains(w))
        .filter(|w| w.len() > 2)
        .map(|s| s.to_string())
        .collect()
}

fn extract_temporal(query: &str) -> Option<Temporal> {
    if query.contains("yesterday") {
        Some(Temporal::Yesterday)
    } else if query.contains("today") {
        Some(Temporal::Today)
    } else if query.contains("last week") {
        Some(Temporal::LastWeek)
    } else if query.contains("last month") {
        Some(Temporal::LastMonth)
    } else {
        None
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    
    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fingerprint_similarity() {
        // Mock embeddings (in real code, these come from the model)
        let embedding1 = vec![0.1; 384];
        let embedding2 = vec![0.1; 384];
        
        let fp1 = QueryFingerprint::new(
            "ssh yesterday",
            embedding1,
        );
        
        let fp2 = QueryFingerprint::new(
            "show me ssh from yesterday",
            embedding2,
        );
        
        let similarity = fp1.similarity(&fp2);
        assert!(similarity > 0.8); // High similarity expected
    }
}