//! Embedding abstractions for FerricLink Core
//!
//! This module provides the core abstractions for text embeddings.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::errors::Result;
use crate::impl_serializable;

/// A text embedding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Embedding {
    /// The embedding vector
    pub values: Vec<f32>,
    /// Metadata about the embedding
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Embedding {
    /// Create a new embedding
    pub fn new(values: Vec<f32>) -> Self {
        Self {
            values,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a new embedding with metadata
    pub fn new_with_metadata(
        values: Vec<f32>,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        Self { values, metadata }
    }

    /// Get the dimension of the embedding
    pub fn dimension(&self) -> usize {
        self.values.len()
    }

    /// Calculate cosine similarity with another embedding
    pub fn cosine_similarity(&self, other: &Embedding) -> f32 {
        if self.values.len() != other.values.len() {
            return 0.0;
        }

        let dot_product: f32 = self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = self.values.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other.values.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    /// Calculate Euclidean distance to another embedding
    pub fn euclidean_distance(&self, other: &Embedding) -> f32 {
        if self.values.len() != other.values.len() {
            return f32::INFINITY;
        }

        let sum_squared_diffs: f32 = self.values
            .iter()
            .zip(other.values.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();

        sum_squared_diffs.sqrt()
    }
}

impl_serializable!(Embedding, ["ferriclink", "embeddings", "embedding"]);

/// Base trait for all embedding models
#[async_trait]
pub trait Embeddings: Send + Sync + 'static {
    /// Get the dimension of the embeddings produced by this model
    fn dimension(&self) -> usize;

    /// Embed a single text
    async fn embed_query(&self, text: &str) -> Result<Embedding>;

    /// Embed multiple texts
    async fn embed_documents(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let mut embeddings = Vec::new();
        for text in texts {
            let embedding = self.embed_query(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    /// Get the model name
    fn model_name(&self) -> &str;

    /// Get the model type
    fn model_type(&self) -> &str {
        "embeddings"
    }
}

/// A simple mock embedding model for testing
pub struct MockEmbeddings {
    model_name: String,
    dimension: usize,
}

impl MockEmbeddings {
    /// Create a new mock embedding model
    pub fn new(model_name: impl Into<String>, dimension: usize) -> Self {
        Self {
            model_name: model_name.into(),
            dimension,
        }
    }

    /// Generate a mock embedding based on the input text
    fn generate_mock_embedding(&self, text: &str) -> Embedding {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        
        let mut values = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let seed = hash.wrapping_add(i as u64);
            let value = (seed as f32 / u64::MAX as f32) * 2.0 - 1.0; // Normalize to [-1, 1]
            values.push(value);
        }
        
        Embedding::new(values)
    }
}

#[async_trait]
impl Embeddings for MockEmbeddings {
    fn dimension(&self) -> usize {
        self.dimension
    }

    async fn embed_query(&self, text: &str) -> Result<Embedding> {
        Ok(self.generate_mock_embedding(text))
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn model_type(&self) -> &str {
        "mock_embeddings"
    }
}

/// Helper function to create a mock embedding model
pub fn mock_embeddings(model_name: impl Into<String>, dimension: usize) -> MockEmbeddings {
    MockEmbeddings::new(model_name, dimension)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializable::Serializable;

    #[test]
    fn test_embedding_creation() {
        let values = vec![0.1, 0.2, 0.3, 0.4];
        let embedding = Embedding::new(values.clone());
        
        assert_eq!(embedding.values, values);
        assert_eq!(embedding.dimension(), 4);
        assert!(embedding.metadata.is_empty());
    }

    #[test]
    fn test_embedding_cosine_similarity() {
        let embedding1 = Embedding::new(vec![1.0, 0.0, 0.0]);
        let embedding2 = Embedding::new(vec![0.0, 1.0, 0.0]);
        let embedding3 = Embedding::new(vec![1.0, 0.0, 0.0]);
        
        // Orthogonal vectors should have similarity 0
        assert!((embedding1.cosine_similarity(&embedding2) - 0.0).abs() < 1e-6);
        
        // Identical vectors should have similarity 1
        assert!((embedding1.cosine_similarity(&embedding3) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_embedding_euclidean_distance() {
        let embedding1 = Embedding::new(vec![0.0, 0.0]);
        let embedding2 = Embedding::new(vec![3.0, 4.0]);
        
        // Distance should be 5 (3-4-5 triangle)
        assert!((embedding1.euclidean_distance(&embedding2) - 5.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_mock_embeddings() {
        let embeddings = MockEmbeddings::new("test-model", 128);
        
        assert_eq!(embeddings.dimension(), 128);
        assert_eq!(embeddings.model_name(), "test-model");
        assert_eq!(embeddings.model_type(), "mock_embeddings");
        
        let embedding = embeddings.embed_query("test text").await.unwrap();
        assert_eq!(embedding.dimension(), 128);
        
        // Same text should produce same embedding
        let embedding2 = embeddings.embed_query("test text").await.unwrap();
        assert_eq!(embedding.values, embedding2.values);
        
        // Different text should produce different embedding
        let embedding3 = embeddings.embed_query("different text").await.unwrap();
        assert_ne!(embedding.values, embedding3.values);
    }

    #[tokio::test]
    async fn test_embed_documents() {
        let embeddings = MockEmbeddings::new("test-model", 64);
        let texts = vec!["text1".to_string(), "text2".to_string()];
        
        let results = embeddings.embed_documents(&texts).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].dimension(), 64);
        assert_eq!(results[1].dimension(), 64);
    }

    #[test]
    fn test_serialization() {
        let embedding = Embedding::new(vec![0.1, 0.2, 0.3]);
        let json = embedding.to_json().unwrap();
        let deserialized: Embedding = Embedding::from_json(&json).unwrap();
        assert_eq!(embedding, deserialized);
    }
}
