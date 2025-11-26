use std::time::SystemTime;

use crate::commands::get_working_directory;
use crate::db::USER_DB;
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
) -> Result<AskResponse, Box<dyn std::error::Error>> {
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
            Ok(AskResponse::Knowledge(answer))
        }

        Intent::Retrieval => {
            // Three-tier cache system
            let word_count = query.split_whitespace().count();

            if word_count <= 1 {
                let result = search(query, directory, print_only);

                return Ok(AskResponse::SearchResults(result));
            }

            // Tier 2: Fingerprint cache
            let fingerprint = QueryFingerprint::from_query(query);

            let cached_params = {
                let mut db = match USER_DB.lock() {
                    Ok(db) => db,
                    Err(e) => return Err(format!("DB lock failed: {}", e).into()),
                };

                if let Some(params) = db.cache.find_match(&fingerprint, 0.80) {
                    db.cache.record_hit(query)?;
                    Some(params)
                } else {
                    None
                }
            };

            // 2. Handle Cache Hit
            if let Some(params) = cached_params {
                let results = execute_search(&params, query, print_only)?;
                return Ok(AskResponse::SearchResults(results));
            }

            if !print_only {
                println!("Querying LLM for search parameters...");
            }

            // Tier 3: LLM fallback
            let params = llm_daemon.interpret_query(query, directory).await?;

            println!("LLM interpreted query params: {:?}", params);

            {
                let mut db = match USER_DB.lock() {
                    Ok(db) => db,
                    Err(e) => return Err(format!("DB lock failed: {}", e).into()),
                };

                // Cache for next time
                db.cache.insert(query, fingerprint, params.clone())?;
            }

            let results = execute_search(&params, query, print_only)?;
            Ok(AskResponse::SearchResults(results))
        }
    }
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

            // Tier 2: Fingerprint cache
            let fingerprint = QueryFingerprint::from_query(query);

            let cached_params = {
                let mut db = match USER_DB.lock() {
                    Ok(db) => db,
                    Err(e) => return Err(format!("DB lock failed: {}", e).into()),
                };

                if let Some(params) = db.cache.find_match(&fingerprint, 0.80) {
                    db.cache.record_hit(query)?;
                    Some(params)
                } else {
                    None
                }
            };

            // 2. Handle Cache Hit
            if let Some(params) = cached_params {
                let results = execute_search_gui(&params)?;
                return Ok(results);
            }

            // Tier 3: LLM fallback
            let params = llm_daemon.interpret_query(query, directory).await?;

            {
                let mut db = match USER_DB.lock() {
                    Ok(db) => db,
                    Err(e) => return Err(format!("DB lock failed: {}", e).into()),
                };

                // Cache for next time
                db.cache.insert(query, fingerprint, params.clone())?;
            }

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

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use chrono::Local;

    use super::*;

    #[tokio::test]
    async fn test_ask() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        println!("Current timestamp: {}", now);
        println!("Current date: {}", Local::now());
        let result = ask(
            "make used today",
            "/Users/somua/Documents/Projects/ClipboardAI/jot-cli",
            false,
        )
        .await
        .unwrap();
        assert!(matches!(result, AskResponse::SearchResults(_)));
    }
}
