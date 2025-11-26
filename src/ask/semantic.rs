use std::collections::HashSet;

use crate::config::GLOBAL_CONFIG;
use crate::db::USER_DB;
use crate::embeds::{cosine_similarity, generate_embedding};
use crate::types::SearchResult;

/// Perform semantic search using embeddings
pub fn semantic_search(query: &str) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    // Generate embedding for query
    let query_embedding = generate_embedding(query)?;

    let mut similarity_threshold = 0.5;

    if let Ok(config) = GLOBAL_CONFIG.read() {
        similarity_threshold = config.search.similarity_threshold;
    }

    let db = USER_DB
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;

    // Get all entries with embeddings
    let mut stmt = db.conn.prepare(
        "SELECT id, entry_type, content, timestamp, times_run, 
                working_dir, host, app_name, window_title, embedding
         FROM entries
         WHERE embedding IS NOT NULL
         ORDER BY timestamp DESC
         LIMIT 1000", // Only search recent entries for performance
    )?;

    let mut results: Vec<SearchResult> = stmt
        .query_map([], |row| {
            let embedding_blob: Option<Vec<u8>> = row.get(8)?;

            let similarity = if let Some(blob) = embedding_blob {
                // Convert blob back to Vec<f32>
                let embedding: Vec<f32> = blob
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();

                cosine_similarity(&query_embedding, &embedding)
            } else {
                0.0
            };

            Ok(SearchResult {
                id: row.get(0)?,
                entry_type: row.get(1)?,
                content: row.get(2)?,
                timestamp: row.get(3)?,
                times_run: row.get(4)?,
                working_dir: row.get(5)?,
                host: row.get(6)?,
                app_name: row.get(7)?,
                window_title: row.get(8)?,
                similarity,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Sort by similarity (highest first)
    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    let mut seen = HashSet::new();
    results.retain(|item| seen.insert(item.content.clone()));

    // Only return results with similarity > 0.5 (threshold)
    let filtered: Vec<SearchResult> = results
        .into_iter()
        .filter(|r| r.similarity > similarity_threshold)
        .take(20)
        .collect();

    Ok(filtered)
}
