use crate::config::GLOBAL_CONFIG;
use crate::db::GLOBAL_DB;
use crate::plugin::GLOBAL_PLUGIN_MANAGER;

use crate::embeds::{cosine_similarity, generate_embedding};
use crate::types::SearchResult;
use console::Term;
use dialoguer::Select;

const MAX_RESULTS: usize = 10;

pub fn search(query: &str, print_only: bool) -> Option<String> {
    if query.is_empty() {
        if !print_only {
            println!("No query provided. Use jotx search <query>");
        }
        return None;
    }

    // Only show UI messages if NOT print_only mode
    if !print_only {
        println!("🔍 Searching for: {}\n", query);
    }

    // Try keyword search first
    match keyword_search(query) {
        Ok(results) if !results.is_empty() => {
            return display_results_interactive(query, &results, "Keyword Search Results", print_only)
                .map(|r| r.content.clone());
        }
        _ => {
            if !print_only {
                println!("No exact matches found. Trying semantic search...\n");
            }

            // Fallback to semantic search
            match semantic_search(query) {
                Ok(results) if !results.is_empty() => {
                    return display_results_interactive(
                        query,
                        &results,
                        "Semantic Search Results",
                        print_only,
                    )
                    .map(|r| r.content.clone());
                }
                _ => {
                    if !print_only {
                        println!("❌ No results found for '{}'", query);
                    }
                    return None;
                }
            }
        }
    }
}

// Keyword search using SQLite FTS5
fn keyword_search(query: &str) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let db = GLOBAL_DB
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;

    // Use FTS5 for full-text search (matches anywhere in the text)
    let mut stmt = db.conn.prepare(
        "SELECT e.id, e.entry_type, e.content, e.timestamp, e.times_run, 
                e.working_dir, e.app_name, e.window_title
         FROM entries_fts 
         JOIN entries e ON entries_fts.rowid = e.id
         WHERE entries_fts MATCH ?1
         ORDER BY e.timestamp DESC
         LIMIT 100",
    )?;

    let mut results: Vec<SearchResult> = stmt
        .query_map([query], |row| {
            Ok(SearchResult {
                id: row.get(0)?,
                entry_type: row.get(1)?,
                content: row.get(2)?,
                timestamp: row.get(3)?,
                times_run: row.get(4)?,
                working_dir: row.get(5)?,
                app_name: row.get(6)?,
                window_title: row.get(7)?,
                similarity: 1.0, // Will be calculated below
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Calculate relevance scores and sort
    let query_lower = query.to_lowercase();

    for result in &mut results {
        result.similarity = calculate_relevance_score(&result.content, &query_lower);
    }

    // Sort by relevance (highest first), then by timestamp
    results.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap()
            .then_with(|| b.timestamp.cmp(&a.timestamp))
    });

    Ok(results)
}

fn calculate_relevance_score(content: &str, query: &str) -> f32 {
    let content_lower = content.to_lowercase();

    // 1. Exact match = highest score
    if content_lower == query {
        return 100.0;
    }

    // 2. Content starts with query = very high score
    if content_lower.starts_with(query) {
        return 90.0;
    }

    // 3. Content contains query as a whole word
    if let Some(pos) = content_lower.find(query) {
        // Check if it's a word boundary
        let is_word_start = pos == 0
            || !content_lower
                .chars()
                .nth(pos - 1)
                .unwrap()
                .is_alphanumeric();
        let end_pos = pos + query.len();
        let is_word_end = end_pos >= content_lower.len()
            || !content_lower
                .chars()
                .nth(end_pos)
                .unwrap()
                .is_alphanumeric();

        if is_word_start && is_word_end {
            // Word match - score based on how early in the string
            let position_penalty = (pos as f32 / content_lower.len() as f32) * 20.0;
            return 80.0 - position_penalty;
        }

        // 4. Contains query but not as whole word (like "echo" in "echomore")
        let position_penalty = (pos as f32 / content_lower.len() as f32) * 30.0;
        return 60.0 - position_penalty;
    }

    // 5. Partial match (shouldn't happen with FTS5, but just in case)
    let match_ratio =
        query.chars().filter(|&c| content_lower.contains(c)).count() as f32 / query.len() as f32;

    match_ratio * 40.0
}

// Semantic search using vector embeddings
fn semantic_search(query: &str) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    // Generate embedding for query
    let query_embedding = generate_embedding(query)?;

    let mut similarity_threshold = 0.5;

    if let Ok(config) = GLOBAL_CONFIG.lock() {
        similarity_threshold = config.search.similarity_threshold;
    }

    let db = GLOBAL_DB
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;

    // Get all entries with embeddings
    let mut stmt = db.conn.prepare(
        "SELECT id, entry_type, content, timestamp, times_run, 
                working_dir, app_name, window_title, embedding
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
                app_name: row.get(6)?,
                window_title: row.get(7)?,
                similarity,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Sort by similarity (highest first)
    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

    // Only return results with similarity > 0.5 (threshold)
    let filtered: Vec<SearchResult> = results
        .into_iter()
        .filter(|r| r.similarity > similarity_threshold)
        .take(20)
        .collect();

    Ok(filtered)
}

fn display_results_interactive<'a>(
    query: &str,
    results: &'a [SearchResult],
    title: &str,
    print_only: bool,
) -> Option<&'a SearchResult> {
    if results.is_empty() {
        return None;
    }

    if !print_only {
        println!("Found {} result(s)\n", results.len());
        println!(
            "🔍 {} - Use ↑↓ arrows, Enter to select, Esc to cancel\n",
            title
        );
    }

    let mut items: Vec<String> = results
        .iter()
        .map(|r| {
            let icon = match r.entry_type.as_str() {
                "clipboard" => "📋",
                "shell" => "💻",
                _ => "📄",
            };
            format!("{} {}", icon, r.content)
        })
        .collect();

    if let Ok(config) = GLOBAL_CONFIG.lock() {
        let max_results = config.search.max_results;
        if max_results > 0 {
            items.truncate(max_results);
        } else {
            items.truncate(MAX_RESULTS);
        }
    } else {
        items.truncate(MAX_RESULTS);
    }

    let selection = Select::new()
        .items(&items)
        .default(0)
        .interact_on_opt(&Term::stderr());

    let selection = selection.ok()??;

    trigger_plugins(query, results);

    Some(&results[selection])
}

fn trigger_plugins(query: &str, results: &[SearchResult]) {
    let mut vec: Vec<SearchResult> = results.to_vec();

    if let Ok(plugins) = GLOBAL_PLUGIN_MANAGER.lock() {
        plugins.trigger_search_after(query, vec.as_mut());
    }
}
