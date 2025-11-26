use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFingerprint {
    pub keywords: HashSet<String>,
    pub temporal: Option<Temporal>,
    pub action_verbs: Vec<String>,
    pub modifiers: Vec<String>,
    pub negations: bool
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Temporal {
    Today,
    Yesterday,
    LastWeek,
    LastMonth,
}

impl QueryFingerprint {
    pub fn from_query(query: &str) -> Self {
        let query_lower = query.to_lowercase();
        
        Self {
            keywords: extract_keywords(&query_lower),
            temporal: extract_temporal(&query_lower),
            action_verbs: extract_action_verbs(&query_lower),
            modifiers: extract_modifiers(&query_lower),
            negations: query_lower.contains("not") || query_lower.contains("without"),
        }
    }
    
    /// Calculate similarity score between two fingerprints (0.0 to 1.0)
    pub fn similarity(&self, other: &QueryFingerprint) -> f32 {
        let mut score = 0.0;
        
        // Keywords overlap (most important - 50%)
        let intersection: HashSet<_> = self.keywords.intersection(&other.keywords).collect();
        let union: HashSet<_> = self.keywords.union(&other.keywords).collect();
        
        if !union.is_empty() {
            score += (intersection.len() as f32 / union.len() as f32) * 0.5;
        }
        
        // Temporal match (30%)
        if self.temporal == other.temporal {
            score += 0.3;
        }
        
        // Negation match (5%)
        if self.negations == other.negations {
            score += 0.05;
        }
        
        score
    }
}

fn extract_keywords(query: &str) -> HashSet<String> {
    let stop_words: HashSet<&str> = ["the", "a", "an", "i", "me", "my", "from", "in", "on", "at", 
                                      "show", "find", "get", "list", "give", "used", "ran", "did"]
        .iter().cloned().collect();
    
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

fn extract_action_verbs(query: &str) -> Vec<String> {
    let verbs = ["show", "find", "get", "list", "search", "used", "ran", "executed"];
    
    verbs.iter()
        .filter(|v| query.contains(*v))
        .map(|s| s.to_string())
        .collect()
}

fn extract_modifiers(query: &str) -> Vec<String> {
    let modifiers = ["failed", "successful", "recent", "old", "long"];
    
    modifiers.iter()
        .filter(|m| query.contains(*m))
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fingerprint_similarity() {
        let fp1 = QueryFingerprint::from_query("ssh yesterday");
        let fp2 = QueryFingerprint::from_query("show me ssh from yesterday");
        
        let similarity = fp1.similarity(&fp2);
        assert!(similarity > 0.8); // Should be very similar
    }
}