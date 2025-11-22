use crate::config::GLOBAL_CONFIG;
use crate::db::GLOBAL_DB;
use crate::llm::{LLMQueryParams, TimeRange};
use crate::plugin::GLOBAL_PLUGIN_MANAGER;

use crate::types::SearchResult;
use console::Term;
use dialoguer::Select;

const MAX_RESULTS: usize = 10;

pub fn search(query: &str, context: &str, print_only: bool) -> Option<String> {
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
    match keyword_search(query, context) {
        Ok(results) if !results.is_empty() => {
            return display_results_interactive(
                query,
                &results,
                "Keyword Search Results",
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

// Keyword search using SQLite FTS5
fn keyword_search(
    query: &str,
    context: &str,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let db = GLOBAL_DB
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;

    // STRATEGY SWITCH:
    // If query is very short (1-2 chars), FTS often fails (especially with trigrams).
    // Use standard SQL LIKE for short queries, FTS for long ones.
    let use_fts = query.len() >= 3;

    let mut stmt;
    
    let mut results: Vec<SearchResult>;

    if use_fts {
        // --- EXISTING FTS LOGIC ---
        let fts_query = format!("{}*", query);
        
        stmt = db.conn.prepare(
            "SELECT e.id, e.entry_type, e.content, e.timestamp, e.times_run, 
                    e.working_dir, e.host, e.app_name, e.window_title,
                    CASE 
                        WHEN e.host = ?2 AND ?2 != '' THEN 15.0
                        ELSE 0.0
                    END as pwd_boost
             FROM entries_fts 
             JOIN entries e ON entries_fts.rowid = e.id
             WHERE entries_fts MATCH ?1
             ORDER BY pwd_boost DESC, e.times_run DESC, e.timestamp DESC
             LIMIT 50",
        )?;

        results = stmt.query_map(rusqlite::params![&fts_query, context], |row| {
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
                similarity: row.get::<_, f32>(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    } else {
        // --- FALLBACK LIKE LOGIC (For 1-2 char queries) ---
        let like_query = format!("%{}%", query); 
        
        stmt = db.conn.prepare(
            "SELECT e.id, e.entry_type, e.content, e.timestamp, e.times_run, 
                    e.working_dir, e.host, e.app_name, e.window_title,
                    CASE 
                        WHEN e.host = ?2 AND ?2 != '' THEN 15.0
                        ELSE 0.0
                    END as pwd_boost
             FROM entries e
             WHERE e.content LIKE ?1
             ORDER BY pwd_boost DESC, e.times_run DESC, e.timestamp DESC
             LIMIT 50",
        )?;

        results = stmt.query_map(rusqlite::params![&like_query, context], |row| {
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
                similarity: row.get::<_, f32>(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    }

    let query_lower = query.to_lowercase();

    // Calculate detailed relevance scores for top 50 results only
    for result in &mut results {
        let host = result.host.as_deref().unwrap_or("");
        let base_score = calculate_relevance_score(&result.content, &query_lower, host, context);

        // Add frequency bonus (times_run)
        let frequency_bonus = (result.times_run as f32).min(10.0) * 2.0; // Max +20 points

        result.similarity = base_score + frequency_bonus;
    }

    // Final sort by calculated score
    results.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Return top 20 for display
    results.truncate(20);

    Ok(results)
}

fn calculate_relevance_score(
    content: &str,
    query: &str,
    result_pwd: &str,
    context_pwd: &str,
) -> f32 {
    let content_lower = content.to_lowercase();
    let mut score;

    // 1. Exact match = highest score
    if content_lower == query {
        score = 100.0;
    }
    // 2. Content starts with query = very high score
    else if content_lower.starts_with(query) {
        score = 90.0;
    }
    // 3. Content contains query
    else if let Some(pos) = content_lower.find(query) {
        // Safe word boundary checking
        let is_word_start = pos == 0 || {
            content_lower[..pos]
                .chars()
                .last()
                .map(|c| !c.is_alphanumeric())
                .unwrap_or(true)
        };

        let end_pos = pos + query.len();
        let is_word_end = end_pos >= content_lower.len() || {
            content_lower[end_pos..]
                .chars()
                .next()
                .map(|c| !c.is_alphanumeric())
                .unwrap_or(true)
        };

        if is_word_start && is_word_end {
            // Word match - score based on position
            let position_penalty = (pos as f32 / content_lower.len().max(1) as f32) * 20.0;
            score = 80.0 - position_penalty;
        } else {
            // Substring match
            let position_penalty = (pos as f32 / content_lower.len().max(1) as f32) * 30.0;
            score = 60.0 - position_penalty;
        }
    }
    // 4. Partial character match (fallback)
    else {
        let match_ratio = query.chars().filter(|&c| content_lower.contains(c)).count() as f32
            / query.len().max(1) as f32;
        score = match_ratio * 40.0;
    }

    // PWD-based boosting (already done in SQL, but add extra granular boost)
    if !context_pwd.is_empty() && !result_pwd.is_empty() {
        if result_pwd == context_pwd {
            score += 15.0;
        } else if result_pwd.starts_with(context_pwd) || context_pwd.starts_with(result_pwd) {
            score += 8.0;
        }
    }

    score
}

pub fn display_results_interactive<'a>(
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

/// Search using LLM-interpreted parameters
pub fn search_with_llm_params(
    params: &LLMQueryParams,
    context: &str,
    print_only: bool,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    // Only show UI messages if NOT print_only mode
    if !print_only {
        println!("🔍 Searching with AI parameters...\n");
    }

    match keyword_search_with_params(params, context) {
        Ok(results) if !results.is_empty() => Ok(results),
        _ => {
            if !print_only {
                println!("❌ No results found");
            }
            return Ok(vec![]);
        }
    }
}

/// Keyword search using LLM-extracted parameters
fn keyword_search_with_params(
    params: &LLMQueryParams,
    context: &str,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let db = GLOBAL_DB
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;

    // Build FTS5 query from keywords
    let fts_query = if params.keywords.is_empty() {
        "*".to_string()
    } else {
        // Join keywords with OR for broader matching
        params.keywords
            .iter()
            .map(|k| format!("{}*", k))
            .collect::<Vec<_>>()
            .join(" OR ")
    };

    // Build WHERE clauses for filters
    let mut where_clauses = vec!["entries_fts MATCH ?1".to_string()];
    let mut bind_params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(fts_query.clone())];
    let mut param_index = 2;

    // Entry type filter
    if let Some(ref types) = params.entry_types {
        if !types.is_empty() {
            let placeholders = types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            where_clauses.push(format!("e.entry_type IN ({})", placeholders));
            for t in types {
                bind_params.push(Box::new(t.clone()));
            }
        }
    }

    // Time range filter
    if let Some(ref time_range) = params.time_range {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let timestamp_start = match time_range {
            TimeRange::Today => now - (24 * 60 * 60),
            TimeRange::Yesterday => {
                let today_start = now - (now % (24 * 60 * 60));
                today_start - (24 * 60 * 60)
            }
            TimeRange::LastWeek => now - (7 * 24 * 60 * 60),
            TimeRange::LastMonth => now - (30 * 24 * 60 * 60),
            TimeRange::Custom { start, end: _ } => *start,
        };
        
        where_clauses.push(format!("e.timestamp >= ?{}", param_index));
        bind_params.push(Box::new(timestamp_start));
        param_index += 1;
        
        // Handle custom end time
        if let TimeRange::Custom { start: _, end } = time_range {
            where_clauses.push(format!("e.timestamp <= ?{}", param_index));
            bind_params.push(Box::new(*end));
            param_index += 1;
        }
    }

    // Working directory filter
    if let Some(ref filters) = params.filters {
        if let Some(ref working_dir) = filters.working_dir {
            where_clauses.push(format!("e.host LIKE ?{}", param_index));
            bind_params.push(Box::new(format!("%{}%", working_dir)));
            param_index += 1;
        }
    }

    // Build final SQL query
    let where_clause = where_clauses.join(" AND ");
    let sql = format!(
        "SELECT e.id, e.entry_type, e.content, e.timestamp, e.times_run, 
                e.working_dir, e.host, e.app_name, e.window_title,
                CASE 
                    WHEN e.host = ?{} AND ?{} != '' THEN 15.0
                    WHEN (e.host LIKE ?{} || '%' OR ?{} LIKE e.host || '%') AND ?{} != '' AND e.host != '' THEN 8.0
                    ELSE 0.0
                END as pwd_boost
         FROM entries_fts 
         JOIN entries e ON entries_fts.rowid = e.id
         WHERE {}
         ORDER BY pwd_boost DESC, e.times_run DESC, e.timestamp DESC
         LIMIT 50",
        param_index, param_index, param_index, param_index, param_index,
        where_clause
    );

    // Add context as final parameter
    bind_params.push(Box::new(context.to_string()));

    // Prepare statement
    let mut stmt = db.conn.prepare(&sql)?;

    // Execute query with dynamic parameters
    let params_refs: Vec<&dyn rusqlite::ToSql> = bind_params.iter().map(|b| b.as_ref()).collect();
    
    let mut results: Vec<SearchResult> = stmt
        .query_map(params_refs.as_slice(), |row| {
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
                similarity: row.get::<_, f32>(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    // Calculate relevance scores
    let query_str = params.keywords.join(" ").to_lowercase();
    
    for result in &mut results {
        let host = result.host.as_deref().unwrap_or("");
        let base_score = calculate_relevance_score(&result.content, &query_str, host, context);
        let frequency_bonus = (result.times_run as f32).min(10.0) * 2.0;
        result.similarity = base_score + frequency_bonus;
    }

    // Final sort by score
    results.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    results.truncate(20);

    Ok(results)
}
