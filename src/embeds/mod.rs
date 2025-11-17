use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static EMBEDDING_MODEL: Lazy<Mutex<SentenceEmbeddingsModel>> = Lazy::new(|| {
    Mutex::new(SentenceEmbeddingsModel::new())
});

pub struct SentenceEmbeddingsModel {
    model: TextEmbedding,
}

impl SentenceEmbeddingsModel {
    pub fn new() -> Self {        
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true)
        )
        .expect("Failed to create embedding model");
        
        Self { model }
    }

    pub fn embed(&mut self, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let embeddings = self.model.embed(vec![text], None)?;
        
        if embeddings.is_empty() {
            return Err("Failed to generate embedding".into());
        }
        
        Ok(embeddings[0].clone())
    }

    // pub fn embed_batch(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    //     let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    //     let embeddings = self.model.embed(text_refs, None)?;
    //     Ok(embeddings)
    // }
}

// Helper function for easy access
pub fn generate_embedding(text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut model = EMBEDDING_MODEL.lock()
        .map_err(|e| format!("Failed to lock embedding model: {}", e))?;
    model.embed(text)
}

// Calculate cosine similarity between two embeddings
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }
    
    dot_product / (magnitude_a * magnitude_b)
}