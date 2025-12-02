// cache.rs
use crate::ask::fingerprint::QueryFingerprint;
use crate::llm::LLMQueryParams;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct FingerprintCache {
    db: Arc<Mutex<Connection>>,
    hot_cache: Vec<CacheEntry>, // Changed to Vec for easier iteration
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
                query TEXT UNIQUE NOT NULL,
                keywords TEXT NOT NULL,
                temporal TEXT,
                embedding BLOB NOT NULL,
                params_json TEXT NOT NULL,
                hit_count INTEGER DEFAULT 1,
                last_used INTEGER NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_last_used ON fingerprint_cache(last_used)",
            [],
        )?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            hot_cache: Vec::new(),
            max_hot_cache_size: 100,
        })
    }

    /// Try to find a matching cached query
    pub fn find_match(
        &mut self,
        fingerprint: &QueryFingerprint,
        threshold: f32,
    ) -> Option<LLMQueryParams> {
        
        // Find best match in hot cache
        let best_match = self
            .hot_cache
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                let similarity = fingerprint.similarity(&entry.fingerprint);
                if similarity >= threshold {
                    Some((
                        idx,
                        similarity,
                        entry.fingerprint.query.clone(),
                        entry.params.clone(),
                    ))
                } else {
                    None
                }
            })
            .max_by(|(_, sim_a, _, _), (_, sim_b, _, _)| {
                sim_a
                    .partial_cmp(sim_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some((idx, score, query, params)) = best_match {
            // Record hit
            self.hot_cache[idx].hit_count += 1;
            self.hot_cache[idx].last_used = now();

            // Update DB asynchronously (non-blocking)
            let _ = self.update_hit_count(&query);

            println!(
                "✓ Cache hit: '{}' → '{}' (similarity: {:.3})",
                fingerprint.query, query, score
            );

            Some(params)
        } else {
            println!("✗ Cache miss: '{}'", fingerprint.query);
            None
        }
    }

    /// Insert a new query into cache
    pub fn insert(
        &mut self,
        fingerprint: QueryFingerprint,
        params: LLMQueryParams,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = now();

        // Insert into database
        {
            let db = self.db.lock().unwrap();
            let keywords_json = serde_json::to_string(&fingerprint.keywords)?;
            let temporal_json = serde_json::to_string(&fingerprint.temporal)?;
            let embedding_blob = vec_to_blob(&fingerprint.embedding);
            let params_json = serde_json::to_string(&params)?;

            db.execute(
                "INSERT OR REPLACE INTO fingerprint_cache 
                 (query, keywords, temporal, embedding, params_json, last_used)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    &fingerprint.query,
                    keywords_json,
                    temporal_json,
                    embedding_blob,
                    params_json,
                    timestamp
                ],
            )?;
        }

        // Add to hot cache
        self.hot_cache.push(CacheEntry {
            fingerprint,
            params,
            hit_count: 1,
            last_used: timestamp,
        });

        // Evict if too large
        if self.hot_cache.len() > self.max_hot_cache_size {
            self.evict_least_used();
        }

        Ok(())
    }

    pub fn warm_up_cache(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Warm up cache but check if it has been warmed up of recently
        if self.hot_cache.is_empty() {
            self.warm_up()
        } else {
            Ok(())
        }
    }

    /// Load hot cache from DB on startup
    pub fn warm_up(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare(
            "SELECT query, embedding, params_json, hit_count, last_used 
             FROM fingerprint_cache 
             ORDER BY last_used DESC 
             LIMIT ?1",
        )?;

        let entries = stmt.query_map([self.max_hot_cache_size], |row| {
            let embedding_blob: Vec<u8> = row.get(1)?;
            let params_json: String = row.get(2)?;

            let embedding = blob_to_vec(&embedding_blob);
            let params: LLMQueryParams = serde_json::from_str(&params_json).unwrap();

            Ok(CacheEntry {
                fingerprint: QueryFingerprint::new(&row.get::<_, String>(0)?, embedding),
                params,
                hit_count: row.get(3)?,
                last_used: row.get(4)?,
            })
        })?;

        for entry in entries {
            self.hot_cache.push(entry?);
        }

        println!("Warmed up cache with {} entries", self.hot_cache.len());
        Ok(())
    }

    fn evict_least_used(&mut self) {
        if let Some(idx) = self
            .hot_cache
            .iter()
            .enumerate()
            .min_by_key(|(_, entry)| (entry.hit_count, entry.last_used))
            .map(|(idx, _)| idx)
        {
            self.hot_cache.remove(idx);
        }
    }

    pub fn update_hit_count(&mut self, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(db) = self.db.lock() {
            db.execute(
                "UPDATE fingerprint_cache 
                    SET hit_count = hit_count + 1, last_used = ?1 
                    WHERE query = ?2",
                params![now(), query],
            )?;
            Ok(())
        } else {
            Err("Failed to update hit count".into())
        }
    }
}

// Helper functions
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn vec_to_blob(vec: &[f32]) -> Vec<u8> {
    vec.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn blob_to_vec(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
