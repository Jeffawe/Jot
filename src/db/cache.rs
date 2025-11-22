use crate::ask::fingerprint::QueryFingerprint;
use crate::llm::LLMQueryParams;
use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct FingerprintCache {
    db: Arc<Mutex<Connection>>,
    hot_cache: HashMap<String, CacheEntry>,
    max_hot_cache_size: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    fingerprint: QueryFingerprint,
    params: LLMQueryParams,
    hit_count: u32,
    last_used: i64,
}

impl FingerprintCache {
    pub fn new(db_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Connection::open(db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS fingerprint_cache (
                id INTEGER PRIMARY KEY,
                original_query TEXT UNIQUE,
                fingerprint_json TEXT,
                params_json TEXT,
                hit_count INTEGER DEFAULT 1,
                last_used INTEGER,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;
        
        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            hot_cache: HashMap::new(),
            max_hot_cache_size: 100,
        })
    }
    
    /// Try to find a matching cached query
    pub fn find_match(
        &mut self,
        fingerprint: &QueryFingerprint,
        threshold: f32,
    ) -> Option<LLMQueryParams> {
        // First check hot cache
        let mut best_match: Option<(f32, LLMQueryParams)> = None;
        
        for entry in self.hot_cache.values() {
            let similarity = fingerprint.similarity(&entry.fingerprint);
            if similarity >= threshold {
                if best_match.is_none() || similarity > best_match.as_ref().unwrap().0 {
                    best_match = Some((similarity, entry.params.clone()));
                }
            }
        }
        
        best_match.map(|(_, params)| params)
    }
    
    /// Insert a new query into cache
    pub fn insert(
        &mut self,
        query: &str,
        fingerprint: QueryFingerprint,
        params: LLMQueryParams,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        
        // Insert into database
        {
            let db = self.db.lock().unwrap();
            let fingerprint_json = serde_json::to_string(&fingerprint)?;
            let params_json = serde_json::to_string(&params)?;
            
            db.execute(
                "INSERT OR REPLACE INTO fingerprint_cache 
                 (original_query, fingerprint_json, params_json, hit_count, last_used)
                 VALUES (?1, ?2, ?3, 1, ?4)",
                params![query, fingerprint_json, params_json, now],
            )?;
        } // db lock dropped here
        
        // Add to hot cache
        self.hot_cache.insert(query.to_string(), CacheEntry {
            fingerprint,
            params,
            hit_count: 1,
            last_used: now,
        });
        
        // Evict if too large
        if self.hot_cache.len() > self.max_hot_cache_size {
            self.evict_least_used();
        }
        
        Ok(())
    }
    
    /// Record a cache hit
    pub fn record_hit(&mut self, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(entry) = self.hot_cache.get_mut(query) {
            entry.hit_count += 1;
            entry.last_used = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs() as i64;
        }
        
        {
            let db = self.db.lock().unwrap();
            db.execute(
                "UPDATE fingerprint_cache 
                 SET hit_count = hit_count + 1, last_used = ?1 
                 WHERE original_query = ?2",
                params![
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs() as i64,
                    query
                ],
            )?;
        } // db lock dropped here
        
        Ok(())
    }
    
    fn evict_least_used(&mut self) {
        if let Some(key) = self.hot_cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(k, _)| k.clone())
        {
            self.hot_cache.remove(&key);
        }
    }
}