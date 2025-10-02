//! Retriever abstractions for FerricLink Core
//!
//! This module provides the core abstractions for retrievers that can
//! fetch relevant documents based on queries.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::documents::Document;
use crate::errors::Result;
use crate::runnables::{Runnable, RunnableConfig};
use crate::impl_serializable;
use crate::vectorstores::VectorStore;

/// A retriever result containing documents and metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrieverResult {
    /// The retrieved documents
    pub documents: Vec<Document>,
    /// Additional metadata about the retrieval
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RetrieverResult {
    /// Create a new retriever result
    pub fn new(documents: Vec<Document>) -> Self {
        Self {
            documents,
            metadata: HashMap::new(),
        }
    }

    /// Create a new retriever result with metadata
    pub fn new_with_metadata(
        documents: Vec<Document>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            documents,
            metadata,
        }
    }

    /// Add metadata to the result
    pub fn add_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
    }

    /// Get the number of documents
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Check if the result is empty
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }
}

impl_serializable!(RetrieverResult, ["ferriclink", "retrievers", "retriever_result"]);

/// Base trait for all retrievers
#[async_trait]
pub trait BaseRetriever: Send + Sync + 'static {
    /// Retrieve documents based on a query
    async fn get_relevant_documents(
        &self,
        query: &str,
        config: Option<RunnableConfig>,
    ) -> Result<RetrieverResult>;

    /// Retrieve documents for multiple queries
    async fn get_relevant_documents_batch(
        &self,
        queries: Vec<String>,
        config: Option<RunnableConfig>,
    ) -> Result<Vec<RetrieverResult>> {
        let mut results = Vec::new();
        for query in queries {
            let result = self.get_relevant_documents(&query, config.clone()).await?;
            results.push(result);
        }
        Ok(results)
    }

    /// Get the input schema for this retriever
    fn input_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Get the output schema for this retriever
    fn output_schema(&self) -> Option<serde_json::Value> {
        None
    }
}

/// A retriever that wraps a vector store
pub struct VectorStoreRetriever {
    vector_store: Box<dyn VectorStore>,
    search_kwargs: HashMap<String, serde_json::Value>,
}

impl VectorStoreRetriever {
    /// Create a new vector store retriever
    pub fn new(vector_store: Box<dyn VectorStore>) -> Self {
        Self {
            vector_store,
            search_kwargs: HashMap::new(),
        }
    }

    /// Create a new vector store retriever with search parameters
    pub fn new_with_kwargs(
        vector_store: Box<dyn VectorStore>,
        search_kwargs: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            vector_store,
            search_kwargs,
        }
    }

    /// Add a search parameter
    pub fn add_search_kwarg(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.search_kwargs.insert(key.into(), value);
    }

    /// Get the number of documents to retrieve
    fn get_k(&self) -> usize {
        self.search_kwargs
            .get("k")
            .and_then(|v| v.as_u64())
            .map(|k| k as usize)
            .unwrap_or(4)
    }

    /// Get the filter for the search
    fn get_filter(&self) -> Option<HashMap<String, serde_json::Value>> {
        self.search_kwargs
            .get("filter")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
    }
}

#[async_trait]
impl BaseRetriever for VectorStoreRetriever {
    async fn get_relevant_documents(
        &self,
        query: &str,
        _config: Option<RunnableConfig>,
    ) -> Result<RetrieverResult> {
        let k = self.get_k();
        let filter = self.get_filter();

        let search_results = self
            .vector_store
            .similarity_search(query, k, filter)
            .await?;

        let documents: Vec<Document> = search_results
            .into_iter()
            .map(|result| result.document)
            .collect();

        let mut retriever_result = RetrieverResult::new(documents);
        retriever_result.add_metadata(
            "search_type",
            serde_json::Value::String("similarity".to_string()),
        );
        retriever_result.add_metadata("k", serde_json::Value::Number(serde_json::Number::from(k)));

        Ok(retriever_result)
    }
}

/// A retriever that can be used as a runnable
pub struct RunnableRetriever<R> {
    retriever: R,
}

impl<R> RunnableRetriever<R>
where
    R: BaseRetriever,
{
    /// Create a new runnable retriever
    pub fn new(retriever: R) -> Self {
        Self { retriever }
    }
}

#[async_trait]
impl<R> Runnable<String, RetrieverResult> for RunnableRetriever<R>
where
    R: BaseRetriever,
{
    async fn invoke(
        &self,
        input: String,
        config: Option<RunnableConfig>,
    ) -> Result<RetrieverResult> {
        self.retriever.get_relevant_documents(&input, config).await
    }
}

/// A retriever that combines multiple retrievers
pub struct MultiRetriever {
    retrievers: Vec<Box<dyn BaseRetriever>>,
    combine_method: CombineMethod,
}

/// Method for combining results from multiple retrievers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CombineMethod {
    /// Take the union of all results
    Union,
    /// Take the intersection of all results
    Intersection,
    /// Take the first retriever's results
    First,
    /// Take the last retriever's results
    Last,
}

impl Default for CombineMethod {
    fn default() -> Self {
        Self::Union
    }
}

impl MultiRetriever {
    /// Create a new multi-retriever
    pub fn new(retrievers: Vec<Box<dyn BaseRetriever>>) -> Self {
        Self {
            retrievers,
            combine_method: CombineMethod::Union,
        }
    }

    /// Create a new multi-retriever with a specific combine method
    pub fn new_with_method(
        retrievers: Vec<Box<dyn BaseRetriever>>,
        combine_method: CombineMethod,
    ) -> Self {
        Self {
            retrievers,
            combine_method,
        }
    }

    /// Add a retriever to the multi-retriever
    pub fn add_retriever(&mut self, retriever: Box<dyn BaseRetriever>) {
        self.retrievers.push(retriever);
    }

    /// Set the combine method
    pub fn set_combine_method(&mut self, method: CombineMethod) {
        self.combine_method = method;
    }

    /// Combine results from multiple retrievers
    fn combine_results(&self, results: Vec<RetrieverResult>) -> RetrieverResult {
        match self.combine_method {
            CombineMethod::Union => {
                let mut all_documents = Vec::new();
                let mut combined_metadata = HashMap::new();
                
                for result in results {
                    all_documents.extend(result.documents);
                    combined_metadata.extend(result.metadata);
                }
                
                RetrieverResult::new_with_metadata(all_documents, combined_metadata)
            }
            CombineMethod::Intersection => {
                if results.is_empty() {
                    return RetrieverResult::new(Vec::new());
                }
                
                let mut intersection = results[0].documents.clone();
                for result in results.iter().skip(1) {
                    intersection.retain(|doc| {
                        result.documents.iter().any(|other_doc| {
                            doc.page_content == other_doc.page_content
                        })
                    });
                }
                
                RetrieverResult::new(intersection)
            }
            CombineMethod::First => {
                results.into_iter().next().unwrap_or_else(|| RetrieverResult::new(Vec::new()))
            }
            CombineMethod::Last => {
                results.into_iter().last().unwrap_or_else(|| RetrieverResult::new(Vec::new()))
            }
        }
    }
}

#[async_trait]
impl BaseRetriever for MultiRetriever {
    async fn get_relevant_documents(
        &self,
        query: &str,
        config: Option<RunnableConfig>,
    ) -> Result<RetrieverResult> {
        let mut results = Vec::new();
        
        for retriever in &self.retrievers {
            let result = retriever.get_relevant_documents(query, config.clone()).await?;
            results.push(result);
        }
        
        Ok(self.combine_results(results))
    }
}

/// Helper function to create a vector store retriever
pub fn vector_store_retriever(vector_store: Box<dyn VectorStore>) -> VectorStoreRetriever {
    VectorStoreRetriever::new(vector_store)
}

/// Helper function to create a runnable retriever
pub fn runnable_retriever<R>(retriever: R) -> RunnableRetriever<R>
where
    R: BaseRetriever,
{
    RunnableRetriever::new(retriever)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializable::Serializable;
    use crate::vectorstores::InMemoryVectorStore;

    #[test]
    fn test_retriever_result() {
        let docs = vec![
            Document::new("Document 1"),
            Document::new("Document 2"),
        ];
        let result = RetrieverResult::new(docs.clone());
        
        assert_eq!(result.documents, docs);
        assert_eq!(result.len(), 2);
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_vector_store_retriever() {
        let vector_store = Box::new(InMemoryVectorStore::new());
        
        // Add some documents to the vector store
        let docs = vec![
            Document::new("Hello world"),
            Document::new("Rust is awesome"),
            Document::new("Python is great"),
        ];
        vector_store.add_documents(docs, None).await.unwrap();
        
        // Create retriever
        let retriever = VectorStoreRetriever::new(vector_store);
        
        // Retrieve documents
        let result = retriever.get_relevant_documents("Hello", None).await.unwrap();
        assert!(!result.is_empty());
        assert_eq!(result.len(), 3); // Only 3 documents in store
    }

    #[tokio::test]
    async fn test_vector_store_retriever_default_k() {
        let vector_store = Box::new(InMemoryVectorStore::new());
        
        // Add some documents to the vector store
        let docs = vec![
            Document::new("Hello world"),
            Document::new("Rust is awesome"),
            Document::new("Python is great"),
            Document::new("Check out FerricLink!"),
            Document::new("FerricLink is a Rust library for building AI applications, inspired by LangChain."),
        ];
        vector_store.add_documents(docs, None).await.unwrap();
        
        // Create retriever
        let retriever = VectorStoreRetriever::new(vector_store);
        
        // Retrieve documents
        let result = retriever.get_relevant_documents("Hello", None).await.unwrap();
        assert!(!result.is_empty());
        assert_eq!(result.len(), 4); // Only 4 documents retrieved by default with k=4
    }

    #[tokio::test]
    async fn test_vector_store_retriever_with_kwargs() {
        let vector_store = Box::new(InMemoryVectorStore::new());
        
        let docs = vec![
            Document::new("Document 1"),
            Document::new("Document 2"),
        ];
        vector_store.add_documents(docs, None).await.unwrap();
        
        let mut search_kwargs = HashMap::new();
        search_kwargs.insert("k".to_string(), serde_json::Value::Number(serde_json::Number::from(1)));
        
        let retriever = VectorStoreRetriever::new_with_kwargs(vector_store, search_kwargs);
        
        let result = retriever.get_relevant_documents("Document", None).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_runnable_retriever() {
        let vector_store = Box::new(InMemoryVectorStore::new());
        let retriever = VectorStoreRetriever::new(vector_store);
        let runnable_retriever = RunnableRetriever::new(retriever);
        
        let result = runnable_retriever.invoke("test query".to_string(), None).await.unwrap();
        assert!(result.is_empty()); // Empty vector store
    }

    #[tokio::test]
    async fn test_multi_retriever_union() {
        let vector_store1 = Box::new(InMemoryVectorStore::new());
        let vector_store2 = Box::new(InMemoryVectorStore::new());
        
        // Add different documents to each store
        vector_store1.add_documents(vec![Document::new("Store 1 doc")], None).await.unwrap();
        vector_store2.add_documents(vec![Document::new("Store 2 doc")], None).await.unwrap();
        
        let retriever1 = VectorStoreRetriever::new(vector_store1);
        let retriever2 = VectorStoreRetriever::new(vector_store2);
        
        let multi_retriever = MultiRetriever::new(vec![
            Box::new(retriever1),
            Box::new(retriever2),
        ]);
        
        let result = multi_retriever.get_relevant_documents("doc", None).await.unwrap();
        assert_eq!(result.len(), 2); // Union of both results
    }

    #[tokio::test]
    async fn test_multi_retriever_first() {
        let vector_store1 = Box::new(InMemoryVectorStore::new());
        let vector_store2 = Box::new(InMemoryVectorStore::new());
        
        vector_store1.add_documents(vec![Document::new("First doc")], None).await.unwrap();
        vector_store2.add_documents(vec![Document::new("Second doc")], None).await.unwrap();
        
        let retriever1 = VectorStoreRetriever::new(vector_store1);
        let retriever2 = VectorStoreRetriever::new(vector_store2);
        
        let multi_retriever = MultiRetriever::new_with_method(
            vec![Box::new(retriever1), Box::new(retriever2)],
            CombineMethod::First,
        );
        
        let result = multi_retriever.get_relevant_documents("doc", None).await.unwrap();
        assert_eq!(result.len(), 1); // Only first retriever's results
    }

    #[test]
    fn test_combine_methods() {
        assert_eq!(CombineMethod::default(), CombineMethod::Union);
    }

    #[test]
    fn test_serialization() {
        let docs = vec![Document::new("Test document")];
        let result = RetrieverResult::new(docs);
        let json = result.to_json().unwrap();
        let deserialized: RetrieverResult = RetrieverResult::from_json(&json).unwrap();
        assert_eq!(result, deserialized);
    }
}
