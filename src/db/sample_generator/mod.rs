use rusqlite::params;

use crate::{db::USER_DB, embeds::EMBEDDING_MODEL};

#[derive(Debug, Clone, Copy)]
pub enum SampleStrategy {
    Similarity, // Pure semantic similarity
    Balanced,   // Balance similarity + quality
    Diverse,    // Maximize diversity
    Adaptive,   // Adapt weights as DB grows
}

pub struct SampleSelector {}

#[derive(Debug, Clone)]
pub struct Sample {
    pub command: String,
    pub context: String,    // What was searched for
    pub quality_score: f32, // How useful this sample has been
    pub similarity: f32,    // Similarity to current query
}

impl SampleSelector {
    pub fn new() -> Self {
        Self {}
    }
    /// Get top-K most relevant samples for a query
    pub fn get_samples(
        &mut self,
        query: &str,
        k: usize,
        strategy: SampleStrategy,
    ) -> Result<Vec<Sample>, Box<dyn std::error::Error>> {
        let query_embedding = match EMBEDDING_MODEL.lock() {
            Ok(mut embed) => embed.embed(query)?,
            Err(_) => return Err("Failed to lock embedding model".into()),
        };

        let db = USER_DB
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;

        // Try vector search first (if available)
        match self.get_samples_vector(&db.conn, &query_embedding, k, strategy) {
            Ok(samples) => {
                drop(db);
                return Ok(samples);
            }
            Err(e) => {
                eprintln!("Vector search failed: {}, using fallback", e);
            }
        }

        // Fallback: Load candidates and compute similarity in Rust
        self.get_samples_fallback(&db.conn, &query_embedding, k, strategy)
    }

    /// Fast vector search using sqlite-vec
    fn get_samples_vector(
        &self,
        conn: &rusqlite::Connection,
        query_embedding: &[f32],
        k: usize,
        strategy: SampleStrategy,
    ) -> Result<Vec<Sample>, Box<dyn std::error::Error>> {
        let embedding_blob = vec_to_blob(query_embedding);

        // Get more candidates than needed for strategy filtering
        let candidate_limit = match strategy {
            SampleStrategy::Diverse => k * 3,
            _ => k * 2,
        };

        let mut stmt = conn.prepare(
            "SELECT e.id, e.content, e.times_run, v.distance
         FROM vec_entries v
         JOIN entries e ON e.id = v.entry_id
         WHERE v.embedding MATCH ?1
           AND e.entry_type = 'command'
         ORDER BY distance ASC
         LIMIT ?2",
        )?;

        let all_samples: Vec<(i64, String, f32, f32, i32)> = stmt
            .query_map(params![embedding_blob, candidate_limit], |row| {
                let id: i64 = row.get(0)?;
                let command: String = row.get(1)?;
                let times_run: i32 = row.get(2)?;
                let distance: f32 = row.get(3)?;

                let similarity = 1.0 - distance; // Convert distance to similarity
                let quality_score = (times_run as f32).ln().max(1.0);

                Ok((id, command, similarity, quality_score, times_run))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Apply strategy filtering
        let selected = self.select_by_strategy(all_samples, k, strategy);

        Ok(selected)
    }

    /// Fallback: Two-stage filtering when sqlite-vec is not available
    fn get_samples_fallback(
        &self,
        conn: &rusqlite::Connection,
        query_embedding: &[f32],
        k: usize,
        strategy: SampleStrategy,
    ) -> Result<Vec<Sample>, Box<dyn std::error::Error>> {
        // Stage 1: Get top candidates by simple heuristics (FAST)
        let candidate_limit = (k * 10).min(1000); // Cap at 1000 to avoid loading too much

        let mut stmt = conn.prepare(
            "SELECT id, content, embedding, times_run 
         FROM entries
         WHERE entry_type = 'command' 
           AND embedding IS NOT NULL
         ORDER BY times_run DESC, timestamp DESC
         LIMIT ?1",
        )?;

        // Stage 2: Compute similarity only on candidates
        let all_samples: Vec<(i64, String, f32, f32, i32)> = stmt
            .query_map([candidate_limit], |row| {
                let id: i64 = row.get(0)?;
                let command: String = row.get(1)?;
                let embedding_blob: Vec<u8> = row.get(2)?;
                let times_run: i32 = row.get(3)?;

                let embedding = blob_to_vec(&embedding_blob);
                let similarity = cosine_similarity(query_embedding, &embedding);
                let quality_score = (times_run as f32).ln().max(1.0);

                Ok((id, command, similarity, quality_score, times_run))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Stage 3: Apply strategy filtering
        let selected = self.select_by_strategy(all_samples, k, strategy);

        Ok(selected)
    }

    fn select_by_strategy(
        &self,
        mut samples: Vec<(i64, String, f32, f32, i32)>,
        k: usize,
        strategy: SampleStrategy,
    ) -> Vec<Sample> {
        match strategy {
            SampleStrategy::Similarity => {
                // Pure similarity-based
                samples.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
            }

            SampleStrategy::Balanced => {
                // Balance similarity and quality
                samples.sort_by(|a, b| {
                    let score_a = a.2 * 0.6 + a.3 * 0.4; // 60% similarity, 40% quality
                    let score_b = b.2 * 0.6 + b.3 * 0.4;
                    score_b.partial_cmp(&score_a).unwrap()
                });
            }

            SampleStrategy::Diverse => {
                // Get diverse samples (avoid too similar commands)
                return self.select_diverse(samples, k);
            }

            SampleStrategy::Adaptive => {
                // Adaptive: more weight to quality as DB grows
                let total_samples = samples.len();
                let quality_weight = (total_samples as f32 / 1000.0).min(0.5);
                let similarity_weight = 1.0 - quality_weight;

                samples.sort_by(|a, b| {
                    let score_a = a.2 * similarity_weight + a.3 * quality_weight;
                    let score_b = b.2 * similarity_weight + b.3 * quality_weight;
                    score_b.partial_cmp(&score_a).unwrap()
                });
            }
        }

        samples
            .into_iter()
            .take(k)
            .map(|(_id, command, similarity, quality_score, _)| Sample {
                command,
                context: String::new(),
                quality_score,
                similarity,
            })
            .collect()
    }

    fn select_diverse(
        &self,
        mut samples: Vec<(i64, String, f32, f32, i32)>,
        k: usize,
    ) -> Vec<Sample> {
        let mut selected = Vec::new();
        samples.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        for (_id, command, similarity, quality_score, _) in samples {
            if selected.len() >= k {
                break;
            }

            // Only add if sufficiently different from already selected
            let is_diverse = selected.iter().all(|s: &Sample| {
                let word_overlap = jaccard_similarity_str(&command, &s.command);
                word_overlap < 0.7 // Less than 70% word overlap
            });

            if is_diverse {
                selected.push(Sample {
                    command,
                    context: String::new(),
                    quality_score,
                    similarity,
                });
            }
        }

        selected
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

fn jaccard_similarity_str(a: &str, b: &str) -> f32 {
    let words_a: std::collections::HashSet<_> = a.split_whitespace().collect();
    let words_b: std::collections::HashSet<_> = b.split_whitespace().collect();

    let intersection = words_a.intersection(&words_b).count();
    let union = words_a.union(&words_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f32 / union as f32
    }
}

fn vec_to_blob(vec: &[f32]) -> Vec<u8> {
    vec.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn blob_to_vec(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sample_gen() {
        let query = "version command used";
        let mut sample_gen = SampleSelector::new();
        let samples = sample_gen
            .get_samples(query, 3, SampleStrategy::Diverse)
            .unwrap();
        println!("Samples: {:#?}", samples);
        assert!(samples.len() > 0);
    }
}
