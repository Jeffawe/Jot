use std::time::SystemTime;

use crate::commands::get_working_directory;
use crate::db::USER_DB;
use crate::embeds::EMBEDDING_MODEL;
use crate::llm::{GLOBAL_LLM, LLMQueryParams};
use crate::types::GUISearchResult;

use super::fingerprint::QueryFingerprint;
use super::intent::{Intent, classify_intent};
use super::search_handler::{
    display_results_interactive, keyword_search_with_params, search, search_gui,
};
use super::semantic::semantic_search;

#[derive(Debug)]
pub enum AskResponse {
    Knowledge(String),
    SearchResults(Option<String>),
}

pub async fn ask(
    query: &str,
    directory: &str,
    print_only: bool,
    test: bool,
) -> Result<AskResponse, Box<dyn std::error::Error>> {
    if query.trim().is_empty() {
        return Err("Query cannot be empty".into());
    }

    let intent = classify_intent(query);

    // Initialize LLM early - we'll need it regardless
    let mut llm_daemon = GLOBAL_LLM.lock().await;
    llm_daemon.get_llm().await.map_err(|e| {
        format!(
            "LLM initialization failed: {}. Use jotx handle-llm to fix",
            e
        )
    })?;

    match intent {
        Intent::Knowledge => {
            // Direct LLM answer (no search)
            let answer = llm_daemon.answer_question(query).await?;
            Ok(AskResponse::Knowledge(answer))
        }

        Intent::Retrieval => {
            // Tier 1: Single word -> direct search (no LLM needed)
            let word_count = query.split_whitespace().count();
            if word_count <= 1 {
                let result = search(query, directory, print_only);
                return Ok(AskResponse::SearchResults(result));
            }

            // Tier 2: Try fingerprint cache
            let cached_params = try_cache_lookup(query).unwrap_or(None);

            if let Some(params) = cached_params {
                if !print_only {
                    println!("✓ Cache hit");
                }
                let results = execute_search(&params, query, print_only)?;
                return Ok(AskResponse::SearchResults(results));
            }

            // Tier 3: LLM fallback (cache miss)
            if !print_only {
                println!("✗ Cache miss - querying LLM...");
            }

            let params = llm_daemon.interpret_query(query, directory).await?;

            if test || !print_only {
                println!("LLM Query Params: {:?}", params);
            }

            // Cache the result for next time
            if let Err(e) = cache_query_params(query, &params) {
                if test {
                    println!("Failed to cache query params: {}", e);
                }
            }

            let results = execute_search(&params, query, print_only)?;
            Ok(AskResponse::SearchResults(results))
        }
    }
}

/// Try to find cached params for this query
fn try_cache_lookup(query: &str) -> Result<Option<LLMQueryParams>, Box<dyn std::error::Error>> {
    // Try to get embedding (non-blocking)
    let embed_lock = EMBEDDING_MODEL.try_lock();
    if embed_lock.is_err() {
        // Embedding service busy, skip cache
        return Ok(None);
    }

    let mut embed = embed_lock.unwrap();
    let query_embedding = match embed.embed(query) {
        Ok(embedding) => embedding,
        Err(_) => {
            // Embedding failed, skip cache
            return Ok(None);
        }
    };

    // Create fingerprint
    let fingerprint = QueryFingerprint::new(query, query_embedding);

    // Search cache
    let mut db = USER_DB
        .lock()
        .map_err(|e| format!("DB lock failed: {}", e))?;

    db.cache.warm_up_cache()?;

    if let Some(params) = db.cache.find_match(&fingerprint, 0.90) {
        // Record hit (this updates hit_count and last_used)
        db.cache.update_hit_count(query)?;
        Ok(Some(params))
    } else {
        Ok(None)
    }
}

/// Cache query and its LLM-generated params
fn cache_query_params(
    query: &str,
    params: &LLMQueryParams,
) -> Result<(), Box<dyn std::error::Error>> {
    // Try to get embedding (non-blocking)
    let embed_lock = EMBEDDING_MODEL.try_lock();
    if embed_lock.is_err() {
        // Embedding service busy, skip caching
        return Ok(());
    }

    let mut embed = embed_lock.unwrap();
    let query_embedding = match embed.embed(query) {
        Ok(embedding) => embedding,
        Err(_) => {
            // Embedding failed, skip caching
            return Ok(());
        }
    };

    // Create fingerprint (you might want to extract keywords here too)
    let fingerprint = QueryFingerprint::new(query, query_embedding);

    // Insert into cache
    let mut db = USER_DB
        .try_lock()
        .map_err(|e| format!("DB lock failed: {}", e))?;

    db.cache.insert(fingerprint, params.clone())?;

    Ok(())
}

fn execute_search(
    params: &LLMQueryParams,
    query: &str,
    print_only: bool,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let directory = get_working_directory();
    let results = if params.use_semantic {
        let query_text = params.keywords.join(" ");
        let result = semantic_search(&query_text);
        result?
    } else {
        match keyword_search_with_params(params, &directory) {
            Ok(res) => res,
            Err(e) => {
                return Err(format!("Search failed: {}", e).into());
            }
        }
    };

    let results =
        display_results_interactive(query, &results, "Keyword Search Results", print_only)
            .map(|r| r.content.clone());

    return Ok(results);
}

#[allow(dead_code)]
pub async fn ask_gui(
    query: &str,
    directory: &str,
) -> Result<Vec<GUISearchResult>, Box<dyn std::error::Error>> {
    if query.trim().is_empty() {
        return Err("Query cannot be empty".into());
    }

    let intent = classify_intent(query);

    let mut llm_daemon = GLOBAL_LLM.lock().await;

    match llm_daemon.get_llm().await {
        Ok(_) => {}
        Err(e) => {
            return Err(format!(
                "LLM initialization failed: {}. Use jotx handle-llm to fix",
                e
            )
            .into());
        }
    };

    match intent {
        Intent::Knowledge => {
            // Direct LLM answer (no search)
            let answer = llm_daemon.answer_question(query).await?;
            Ok(vec![GUISearchResult {
                title: "LLM Answer".to_string(),
                content: answer,
                source: "LLM".to_string(),
                score: 1.0,
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs() as i64,
            }])
        }

        Intent::Retrieval => {
            // Three-tier cache system
            let word_count = query.split_whitespace().count();

            if word_count <= 1 {
                let result = search_gui(query, directory)?;

                return Ok(result);
            }

            let cached_params = try_cache_lookup(query).unwrap_or(None);

            // 2. Handle Cache Hit
            if let Some(params) = cached_params {
                let results = execute_search_gui(&params)?;
                return Ok(results);
            }

            // Tier 3: LLM fallback
            let params = llm_daemon.interpret_query(query, directory).await?;

            // Cache the result for next time
            let _ = cache_query_params(query, &params);

            let results = execute_search_gui(&params)?;
            Ok(results)
        }
    }
}

pub fn execute_search_gui(
    params: &LLMQueryParams,
) -> Result<Vec<GUISearchResult>, Box<dyn std::error::Error>> {
    let directory = get_working_directory();
    let results = if params.use_semantic {
        let query_text = params.keywords.join(" ");
        let result = semantic_search(&query_text);
        result?
    } else {
        match keyword_search_with_params(params, &directory) {
            Ok(res) => res,
            Err(e) => {
                return Err(format!("Search failed: {}", e).into());
            }
        }
    };

    let results = results
        .into_iter()
        .map(|r| GUISearchResult {
            title: "Result".to_string(),
            content: r.content,
            source: r.entry_type,
            timestamp: r.timestamp,
            score: r.similarity,
        })
        .collect();

    return Ok(results);
}
