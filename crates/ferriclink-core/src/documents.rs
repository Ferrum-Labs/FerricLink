//! Document types for FerricLink Core
//!
//! This module provides document abstractions for handling text and structured data
//! in the FerricLink ecosystem.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::Result;
use crate::impl_serializable;

/// A document represents a piece of text with associated metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Document {
    /// The text content of the document
    pub page_content: String,
    /// Metadata associated with the document
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Document {
    /// Create a new document with the given content
    pub fn new(page_content: impl Into<String>) -> Self {
        Self {
            page_content: page_content.into(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new document with content and metadata
    pub fn new_with_metadata(
        page_content: impl Into<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            page_content: page_content.into(),
            metadata,
        }
    }

    /// Add metadata to the document
    pub fn add_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Get metadata value as a string
    pub fn get_metadata_string(&self, key: &str) -> Option<String> {
        self.get_metadata(key).and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// Get metadata value as a number
    pub fn get_metadata_number(&self, key: &str) -> Option<f64> {
        self.get_metadata(key).and_then(|v| v.as_f64())
    }

    /// Get metadata value as a boolean
    pub fn get_metadata_bool(&self, key: &str) -> Option<bool> {
        self.get_metadata(key).and_then(|v| v.as_bool())
    }

    /// Check if the document has a specific metadata key
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Get the length of the document content
    pub fn len(&self) -> usize {
        self.page_content.len()
    }

    /// Check if the document is empty
    pub fn is_empty(&self) -> bool {
        self.page_content.is_empty()
    }

    /// Split the document into chunks
    pub fn split(&self, chunk_size: usize, overlap: usize) -> Vec<Document> {
        if self.page_content.len() <= chunk_size {
            return vec![self.clone()];
        }

        let mut chunks = Vec::new();
        let mut start = 0;

        while start < self.page_content.len() {
            let end = (start + chunk_size).min(self.page_content.len());
            let chunk_content = self.page_content[start..end].to_string();
            
            let mut chunk_metadata = self.metadata.clone();
            chunk_metadata.insert("chunk_index".to_string(), serde_json::Value::Number(
                serde_json::Number::from(chunks.len())
            ));
            chunk_metadata.insert("chunk_start".to_string(), serde_json::Value::Number(
                serde_json::Number::from(start)
            ));
            chunk_metadata.insert("chunk_end".to_string(), serde_json::Value::Number(
                serde_json::Number::from(end)
            ));

            chunks.push(Document {
                page_content: chunk_content,
                metadata: chunk_metadata,
            });

            if end >= self.page_content.len() {
                break;
            }

            start = end.saturating_sub(overlap);
        }

        chunks
    }

    /// Join multiple documents into one
    pub fn join(documents: &[Document], separator: &str) -> Document {
        let mut content = String::new();
        let mut metadata = HashMap::new();

        for (i, doc) in documents.iter().enumerate() {
            if i > 0 {
                content.push_str(separator);
            }
            content.push_str(&doc.page_content);

            // Merge metadata, with later documents taking precedence
            for (key, value) in &doc.metadata {
                metadata.insert(key.clone(), value.clone());
            }
        }

        metadata.insert("source_documents".to_string(), serde_json::Value::Number(
            serde_json::Number::from(documents.len())
        ));

        Document {
            page_content: content,
            metadata,
        }
    }
}

impl_serializable!(Document, ["ferriclink", "documents", "document"]);

/// A collection of documents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentCollection {
    /// The documents in the collection
    pub documents: Vec<Document>,
    /// Metadata about the collection
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl DocumentCollection {
    /// Create a new empty document collection
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new document collection with documents
    pub fn new_with_documents(documents: Vec<Document>) -> Self {
        Self {
            documents,
            metadata: HashMap::new(),
        }
    }

    /// Add a document to the collection
    pub fn add_document(&mut self, document: Document) {
        self.documents.push(document);
    }

    /// Add multiple documents to the collection
    pub fn add_documents(&mut self, documents: Vec<Document>) {
        self.documents.extend(documents);
    }

    /// Get the number of documents in the collection
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// Get a document by index
    pub fn get(&self, index: usize) -> Option<&Document> {
        self.documents.get(index)
    }

    /// Get all documents
    pub fn documents(&self) -> &[Document] {
        &self.documents
    }

    /// Get the total length of all documents
    pub fn total_length(&self) -> usize {
        self.documents.iter().map(|doc| doc.len()).sum()
    }

    /// Split all documents into chunks
    pub fn split_all(&self, chunk_size: usize, overlap: usize) -> DocumentCollection {
        let mut chunks = Vec::new();
        
        for doc in &self.documents {
            chunks.extend(doc.split(chunk_size, overlap));
        }

        DocumentCollection {
            documents: chunks,
            metadata: self.metadata.clone(),
        }
    }

    /// Filter documents based on a predicate
    pub fn filter<F>(&self, predicate: F) -> DocumentCollection
    where
        F: Fn(&Document) -> bool,
    {
        DocumentCollection {
            documents: self.documents.iter().filter(|doc| predicate(doc)).cloned().collect(),
            metadata: self.metadata.clone(),
        }
    }

    /// Map documents using a function
    pub fn map<F>(&self, mapper: F) -> DocumentCollection
    where
        F: Fn(&Document) -> Document,
    {
        DocumentCollection {
            documents: self.documents.iter().map(mapper).collect(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Default for DocumentCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl_serializable!(DocumentCollection, ["ferriclink", "documents", "collection"]);

/// Trait for objects that can be converted to documents
pub trait ToDocument {
    /// Convert this object to a document
    fn to_document(&self) -> Document;
}

/// Trait for objects that can be converted from documents
pub trait FromDocument {
    /// Convert a document to this object
    fn from_document(document: &Document) -> Result<Self>
    where
        Self: Sized;
}

impl ToDocument for str {
    fn to_document(&self) -> Document {
        Document::new(self)
    }
}

impl ToDocument for String {
    fn to_document(&self) -> Document {
        Document::new(self)
    }
}

impl ToDocument for Document {
    fn to_document(&self) -> Document {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializable::Serializable;

    #[test]
    fn test_document_creation() {
        let doc = Document::new("Hello, world!");
        assert_eq!(doc.page_content, "Hello, world!");
        assert!(doc.metadata.is_empty());
    }

    #[test]
    fn test_document_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::Value::String("test".to_string()));
        
        let doc = Document::new_with_metadata("Hello, world!", metadata);
        assert_eq!(doc.page_content, "Hello, world!");
        assert_eq!(doc.get_metadata_string("source"), Some("test".to_string()));
    }

    #[test]
    fn test_document_metadata_operations() {
        let mut doc = Document::new("Test content");
        
        doc.add_metadata("key1", serde_json::Value::String("value1".to_string()));
        doc.add_metadata("key2", serde_json::Value::Number(serde_json::Number::from(42)));
        doc.add_metadata("key3", serde_json::Value::Bool(true));
        
        assert_eq!(doc.get_metadata_string("key1"), Some("value1".to_string()));
        assert_eq!(doc.get_metadata_number("key2"), Some(42.0));
        assert_eq!(doc.get_metadata_bool("key3"), Some(true));
        assert!(doc.has_metadata("key1"));
        assert!(!doc.has_metadata("nonexistent"));
    }

    #[test]
    fn test_document_split() {
        let doc = Document::new("This is a test document that should be split into multiple chunks.");
        let chunks = doc.split(20, 5);
        
        assert!(chunks.len() > 1);
        assert_eq!(chunks[0].get_metadata_number("chunk_index"), Some(0.0));
        assert_eq!(chunks[1].get_metadata_number("chunk_index"), Some(1.0));
    }

    #[test]
    fn test_document_join() {
        let doc1 = Document::new("First document");
        let doc2 = Document::new("Second document");
        let joined = Document::join(&[doc1, doc2], " | ");
        
        assert_eq!(joined.page_content, "First document | Second document");
        assert_eq!(joined.get_metadata_number("source_documents"), Some(2.0));
    }

    #[test]
    fn test_document_collection() {
        let mut collection = DocumentCollection::new();
        
        collection.add_document(Document::new("Doc 1"));
        collection.add_document(Document::new("Doc 2"));
        
        assert_eq!(collection.len(), 2);
        assert!(!collection.is_empty());
        assert_eq!(collection.total_length(), 10); // "Doc 1" + "Doc 2"
    }

    #[test]
    fn test_document_collection_operations() {
        let docs = vec![
            Document::new("Short"),
            Document::new("This is a longer document"),
        ];
        let collection = DocumentCollection::new_with_documents(docs);
        
        let filtered = collection.filter(|doc| doc.len() > 10);
        assert_eq!(filtered.len(), 1);
        
        let mapped = collection.map(|doc| Document::new(format!("Processed: {}", doc.page_content)));
        assert_eq!(mapped.len(), 2);
        assert!(mapped.documents[0].page_content.starts_with("Processed:"));
    }

    #[test]
    fn test_to_document_trait() {
        let doc1 = "Hello".to_document();
        let doc2 = "World".to_string().to_document();
        let doc3 = doc1.clone().to_document();
        
        assert_eq!(doc1.page_content, "Hello");
        assert_eq!(doc2.page_content, "World");
        assert_eq!(doc3.page_content, "Hello");
    }

    #[test]
    fn test_serialization() {
        let doc = Document::new("Test content");
        let json = doc.to_json().unwrap();
        let deserialized: Document = Document::from_json(&json).unwrap();
        assert_eq!(doc, deserialized);
    }
}
