//! Vector store abstractions for FerricLink Core
//!
//! This module provides the core abstractions for vector stores that can
//! store and search over embeddings.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::documents::Document;
use crate::embeddings::{Embedding, Embeddings};
use crate::errors::Result;
use crate::impl_serializable;

/// A search result from a vector store
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorSearchResult {
    /// The document that was found
    pub document: Document,
    /// The similarity score
    pub score: f32,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl VectorSearchResult {
    /// Create a new search result
    pub fn new(document: Document, score: f32) -> Self {
        Self {
            document,
            score,
            metadata: HashMap::new(),
        }
    }

    /// Create a new search result with metadata
    pub fn new_with_metadata(
        document: Document,
        score: f32,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            document,
            score,
            metadata,
        }
    }
}

impl_serializable!(
    VectorSearchResult,
    ["ferriclink", "vectorstores", "search_result"]
);

/// Base trait for all vector stores
#[async_trait]
pub trait VectorStore: Send + Sync + 'static {
    /// Add documents to the vector store
    async fn add_documents(
        &self,
        documents: Vec<Document>,
        embeddings: Option<Vec<Embedding>>,
    ) -> Result<Vec<String>>;

    /// Add texts to the vector store
    async fn add_texts(
        &self,
        texts: Vec<String>,
        metadatas: Option<Vec<HashMap<String, serde_json::Value>>>,
        embeddings: Option<Vec<Embedding>>,
    ) -> Result<Vec<String>> {
        let documents: Vec<Document> = texts
            .into_iter()
            .zip(metadatas.unwrap_or_default().into_iter().cycle())
            .map(|(text, metadata)| Document::new_with_metadata(text, metadata))
            .collect();

        self.add_documents(documents, embeddings).await
    }

    /// Search for similar documents
    async fn similarity_search(
        &self,
        query: &str,
        k: usize,
        filter: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<VectorSearchResult>>;

    /// Search for similar documents by embedding
    async fn similarity_search_by_embedding(
        &self,
        query_embedding: &Embedding,
        k: usize,
        filter: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<VectorSearchResult>>;

    /// Delete documents by IDs
    async fn delete(&self, ids: Vec<String>) -> Result<()>;

    /// Get the number of documents in the store
    async fn len(&self) -> Result<usize>;

    /// Check if the store is empty
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }

    /// Clear all documents from the store
    async fn clear(&self) -> Result<()>;

    /// Get the embedding model used by this store
    fn embedding_model(&self) -> Option<&dyn Embeddings> {
        None
    }
}

/// A simple in-memory vector store for testing
pub struct InMemoryVectorStore {
    documents: std::sync::Arc<tokio::sync::RwLock<Vec<Document>>>,
    embeddings: std::sync::Arc<tokio::sync::RwLock<Vec<Embedding>>>,
    ids: std::sync::Arc<tokio::sync::RwLock<Vec<String>>>,
    embedding_model: Option<Box<dyn Embeddings>>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            documents: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            embeddings: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            ids: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            embedding_model: None,
        }
    }

    /// Create a new in-memory vector store with an embedding model
    pub fn new_with_embeddings(embedding_model: Box<dyn Embeddings>) -> Self {
        Self {
            documents: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            embeddings: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            ids: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
            embedding_model: Some(embedding_model),
        }
    }

    /// Generate embeddings for texts using the embedding model
    async fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        if let Some(model) = &self.embedding_model {
            model.embed_documents(texts).await
        } else {
            // Generate mock embeddings if no model is provided
            let mut embeddings = Vec::new();
            for text in texts {
                let mut values = Vec::new();
                for (i, c) in text.chars().enumerate() {
                    values.push((c as u32 as f32 + i as f32) / 100.0);
                }
                // Pad to a fixed dimension if needed
                while values.len() < 128 {
                    values.push(0.0);
                }
                embeddings.push(Embedding::new(values));
            }
            Ok(embeddings)
        }
    }

    /// Calculate cosine similarity between two embeddings
    fn cosine_similarity(&self, a: &Embedding, b: &Embedding) -> f32 {
        a.cosine_similarity(b)
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn add_documents(
        &self,
        documents: Vec<Document>,
        embeddings: Option<Vec<Embedding>>,
    ) -> Result<Vec<String>> {
        let mut doc_store = self.documents.write().await;
        let mut emb_store = self.embeddings.write().await;
        let mut id_store = self.ids.write().await;

        let generated_embeddings = embeddings.unwrap_or_else(|| {
            documents
                .iter()
                .map(|doc| {
                    let mut values = Vec::new();
                    for (i, c) in doc.page_content.chars().enumerate() {
                        values.push((c as u32 as f32 + i as f32) / 100.0);
                    }
                    while values.len() < 128 {
                        values.push(0.0);
                    }
                    Embedding::new(values)
                })
                .collect()
        });

        let mut ids = Vec::new();
        for (i, document) in documents.into_iter().enumerate() {
            let id = uuid::Uuid::new_v4().to_string();
            ids.push(id.clone());
            id_store.push(id);
            doc_store.push(document);
            emb_store.push(generated_embeddings[i].clone());
        }

        Ok(ids)
    }

    async fn similarity_search(
        &self,
        query: &str,
        k: usize,
        _filter: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<VectorSearchResult>> {
        // Generate embedding for the query
        let query_embedding = if let Some(model) = &self.embedding_model {
            model.embed_query(query).await?
        } else {
            // Generate mock embedding
            let mut values = Vec::new();
            for (i, c) in query.chars().enumerate() {
                values.push((c as u32 as f32 + i as f32) / 100.0);
            }
            while values.len() < 128 {
                values.push(0.0);
            }
            Embedding::new(values)
        };

        self.similarity_search_by_embedding(&query_embedding, k, None)
            .await
    }

    async fn similarity_search_by_embedding(
        &self,
        query_embedding: &Embedding,
        k: usize,
        _filter: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Vec<VectorSearchResult>> {
        let documents = self.documents.read().await;
        let embeddings = self.embeddings.read().await;

        if documents.is_empty() {
            return Ok(Vec::new());
        }

        // Calculate similarities
        let mut similarities: Vec<(usize, f32)> = embeddings
            .iter()
            .enumerate()
            .map(|(i, emb)| (i, self.cosine_similarity(query_embedding, emb)))
            .collect();

        // Sort by similarity (descending)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top k results
        let results: Vec<VectorSearchResult> = similarities
            .into_iter()
            .take(k)
            .map(|(idx, score)| VectorSearchResult::new(documents[idx].clone(), score))
            .collect();

        Ok(results)
    }

    async fn delete(&self, ids: Vec<String>) -> Result<()> {
        let mut doc_store = self.documents.write().await;
        let mut emb_store = self.embeddings.write().await;
        let mut id_store = self.ids.write().await;

        // Find indices to remove
        let mut indices_to_remove: Vec<usize> = Vec::new();
        for id in &ids {
            if let Some(pos) = id_store.iter().position(|x| x == id) {
                indices_to_remove.push(pos);
            }
        }

        // Sort indices in descending order to remove from the end first
        indices_to_remove.sort_by(|a, b| b.cmp(a));

        // Remove elements
        for &idx in &indices_to_remove {
            if idx < doc_store.len() {
                doc_store.remove(idx);
                emb_store.remove(idx);
                id_store.remove(idx);
            }
        }

        Ok(())
    }

    async fn len(&self) -> Result<usize> {
        Ok(self.documents.read().await.len())
    }

    async fn clear(&self) -> Result<()> {
        let mut doc_store = self.documents.write().await;
        let mut emb_store = self.embeddings.write().await;
        let mut id_store = self.ids.write().await;

        doc_store.clear();
        emb_store.clear();
        id_store.clear();

        Ok(())
    }

    fn embedding_model(&self) -> Option<&dyn Embeddings> {
        self.embedding_model.as_ref().map(|m| m.as_ref())
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create an in-memory vector store
pub fn in_memory_vector_store() -> InMemoryVectorStore {
    InMemoryVectorStore::new()
}

/// Helper function to create an in-memory vector store with embeddings
pub fn in_memory_vector_store_with_embeddings(
    embedding_model: Box<dyn Embeddings>,
) -> InMemoryVectorStore {
    InMemoryVectorStore::new_with_embeddings(embedding_model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::MockEmbeddings;
    use crate::serializable::Serializable;

    #[tokio::test]
    async fn test_in_memory_vector_store() {
        let store = InMemoryVectorStore::new();

        // Test empty store
        assert!(store.is_empty().await.unwrap());
        assert_eq!(store.len().await.unwrap(), 0);

        // Add documents
        let docs = vec![
            Document::new("Hello world"),
            Document::new("Rust is awesome"),
        ];

        let ids = store.add_documents(docs, None).await.unwrap();
        assert_eq!(ids.len(), 2);
        assert_eq!(store.len().await.unwrap(), 2);
        assert!(!store.is_empty().await.unwrap());
    }

    #[tokio::test]
    async fn test_similarity_search() {
        let store = InMemoryVectorStore::new();

        // Add documents
        let docs = vec![
            Document::new("Hello world"),
            Document::new("Rust programming language"),
            Document::new("Python is great"),
        ];

        store.add_documents(docs, None).await.unwrap();

        // Search for similar documents
        let results = store.similarity_search("Hello", 2, None).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].score >= results[1].score); // Results should be sorted by score
    }

    #[tokio::test]
    async fn test_delete_documents() {
        let store = InMemoryVectorStore::new();

        // Add documents
        let docs = vec![
            Document::new("Doc 1"),
            Document::new("Doc 2"),
            Document::new("Doc 3"),
        ];

        let ids = store.add_documents(docs, None).await.unwrap();
        assert_eq!(store.len().await.unwrap(), 3);

        // Delete one document
        store.delete(vec![ids[0].clone()]).await.unwrap();
        assert_eq!(store.len().await.unwrap(), 2);

        // Clear all documents
        store.clear().await.unwrap();
        assert!(store.is_empty().await.unwrap());
    }

    #[tokio::test]
    async fn test_with_embedding_model() {
        let embedding_model = Box::new(MockEmbeddings::new("test-model", 128));
        let store = InMemoryVectorStore::new_with_embeddings(embedding_model);

        let docs = vec![Document::new("Test document")];
        let ids = store.add_documents(docs, None).await.unwrap();
        assert_eq!(ids.len(), 1);

        let results = store.similarity_search("Test", 1, None).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_vector_search_result() {
        let doc = Document::new("Test document");
        let result = VectorSearchResult::new(doc.clone(), 0.95);

        assert_eq!(result.document, doc);
        assert_eq!(result.score, 0.95);
        assert!(result.metadata.is_empty());
    }

    #[test]
    fn test_serialization() {
        let doc = Document::new("Test document");
        let result = VectorSearchResult::new(doc, 0.95);
        let json = result.to_json().unwrap();
        let deserialized: VectorSearchResult = VectorSearchResult::from_json(&json).unwrap();
        assert_eq!(result, deserialized);
    }
}
