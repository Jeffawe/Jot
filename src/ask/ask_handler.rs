use crate::db::GLOBAL_DB;
use crate::llm::{GLOBAL_LLM, LLMQueryParams};
use crate::commands::get_working_directory;

use super::fingerprint::QueryFingerprint;
use super::intent::{Intent, classify_intent};
use super::search_handler::{display_results_interactive, search_with_llm_params};
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
    // Step 1: Classify intent
    let intent = classify_intent(query);

    let mut db = match GLOBAL_DB.lock() {
        Ok(db) => db,
        Err(e) => return Err(format!("DB lock failed: {}", e).into()),
    };

    let mut llm_daemon = GLOBAL_LLM
        .lock()
        .map_err(|e| format!("LLM lock failed: {}", e))?;

    match llm_daemon.get_llm().await {
        Ok(_) => {}
        Err(e) => {
            return Err(format!("LLM initialization failed: {}. Use jotx handlellm to fix", e).into());
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

            // Tier 2: Fingerprint cache
            let fingerprint = QueryFingerprint::from_query(query);

            if let Some(params) = db.cache.find_match(&fingerprint, 0.80) {
                // RECORD THE HIT!
                db.cache.record_hit(query)?;
                let results = execute_search(&params, query, print_only)?;
                return Ok(AskResponse::SearchResults(results));
            }

            // Tier 3: LLM fallback
            let params = llm_daemon.interpret_query(query, directory).await?;

            // Cache for next time
            db.cache.insert(query, fingerprint, params.clone())?;

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
    let context = get_working_directory();
    let results = if params.use_semantic {
        let query_text = params.keywords.join(" ");
        let result = semantic_search(&query_text);
        result?
    } else {
        match search_with_llm_params(params, &context, print_only) {
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
