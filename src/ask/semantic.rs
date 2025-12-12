use crate::config::GLOBAL_CONFIG;
use crate::db::USER_DB;
use crate::embeds::{cosine_similarity, generate_embedding};
use crate::types::SearchResult;
use rusqlite::params;
use std::collections::HashSet;

/// Perform semantic search using embeddings
pub fn semantic_search(query: &str) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    // Generate embedding for query
    let query_embedding = generate_embedding(query)?;

    let mut similarity_threshold = 0.8;
    if let Ok(config) = GLOBAL_CONFIG.read() {
        similarity_threshold = config.search.similarity_threshold;
    }

    let db = USER_DB
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;

    // Try vector search first, fallback to manual search
    let results = match semantic_search_vector(&db.conn, &query_embedding, similarity_threshold) {
        Ok(results) => {
            // println!("✓ Using sqlite-vec for semantic search");
            results
        }
        Err(_) => {
            // println!("ℹ Using fallback semantic search");
            semantic_search_fallback(&db.conn, &query_embedding, similarity_threshold)?
        }
    };

    // Deduplicate by content
    let mut seen = HashSet::new();
    let filtered: Vec<SearchResult> = results
        .into_iter()
        .filter(|item| seen.insert(item.content.clone()))
        .take(20)
        .collect();

    Ok(filtered)
}

/// Fast semantic search using sqlite-vec
fn semantic_search_vector(
    conn: &rusqlite::Connection,
    query_embedding: &[f32],
    threshold: f32,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let embedding_blob = vec_to_blob(query_embedding);

    let mut stmt = conn.prepare(
        "SELECT e.id, e.entry_type, e.content, e.timestamp, e.times_run,
            e.working_dir, e.host, e.app_name, e.window_title,
            vec_distance_cosine(v.embedding, ?1) AS distance
            FROM vec_entries v
            JOIN entries e ON e.id = v.entry_id
            ORDER BY distance ASC
            LIMIT 1000",
    )?;

    let mut results: Vec<SearchResult> = stmt
        .query_map(params![embedding_blob], |row| {
            let distance: f32 = row.get(9)?;
            let similarity = 1.0 - distance;

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

    // Filter by threshold and sort
    results.retain(|r| r.similarity > threshold);
    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    Ok(results)
}

/// Fallback semantic search (compute similarity in Rust)
fn semantic_search_fallback(
    conn: &rusqlite::Connection,
    query_embedding: &[f32],
    threshold: f32,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    // Get recent entries with embeddings
    let mut stmt = conn.prepare(
        "SELECT id, entry_type, content, timestamp, times_run, 
                working_dir, host, app_name, window_title, embedding
         FROM entries
         WHERE embedding IS NOT NULL
         ORDER BY timestamp DESC
         LIMIT 1000",
    )?;

    let mut results: Vec<SearchResult> = stmt
        .query_map([], |row| {
            let embedding_blob: Option<Vec<u8>> = row.get(9)?;

            let similarity = if let Some(blob) = embedding_blob {
                let embedding = blob_to_vec(&blob);
                cosine_similarity(query_embedding, &embedding)
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

    // Filter by threshold
    results.retain(|r| r.similarity > threshold);

    Ok(results)
}

fn vec_to_blob(vec: &[f32]) -> Vec<u8> {
    vec.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn blob_to_vec(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
