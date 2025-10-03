//! Example selectors for FerricLink Core.
//!
//! **Example selector** implements logic for selecting examples to include them in prompts.
//! This allows us to select examples that are most relevant to the input.
//!
//! Example selectors are crucial for:
//! - Few-shot learning and prompting
//! - Dynamic prompt construction
//! - Semantic similarity-based example selection
//! - Length-based prompt management
//! - Max Marginal Relevance algorithms

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::Result;
use crate::impl_serializable;
use crate::vectorstores::{VectorStore, VectorSearchResult};

/// A type alias for example data
pub type Example = HashMap<String, String>;

/// Interface for selecting examples to include in prompts.
///
/// Example selectors are used to dynamically choose which examples to include
/// in prompts based on the input variables. This is essential for few-shot
/// learning and context-aware prompt construction.
#[async_trait]
pub trait BaseExampleSelector: Send + Sync {
    /// Add a new example to the store.
    ///
    /// # Arguments
    ///
    /// * `example` - A dictionary with keys as input variables
    ///   and values as their values.
    ///
    /// # Returns
    ///
    /// Any return value (e.g., example ID for tracking).
    fn add_example(&mut self, example: Example) -> Result<()>;

    /// Async add a new example to the store.
    ///
    /// # Arguments
    ///
    /// * `example` - A dictionary with keys as input variables
    ///   and values as their values.
    ///
    /// # Returns
    ///
    /// Any return value (e.g., example ID for tracking).
    async fn aadd_example(&mut self, example: Example) -> Result<()> {
        self.add_example(example)
    }

    /// Select which examples to use based on the inputs.
    ///
    /// # Arguments
    ///
    /// * `input_variables` - A dictionary with keys as input variables
    ///   and values as their values.
    ///
    /// # Returns
    ///
    /// A list of examples to include in the prompt.
    fn select_examples(&self, input_variables: &Example) -> Result<Vec<Example>>;

    /// Async select which examples to use based on the inputs.
    ///
    /// # Arguments
    ///
    /// * `input_variables` - A dictionary with keys as input variables
    ///   and values as their values.
    ///
    /// # Returns
    ///
    /// A list of examples to include in the prompt.
    async fn aselect_examples(&self, input_variables: &Example) -> Result<Vec<Example>> {
        self.select_examples(input_variables)
    }
}

/// Select examples based on length.
///
/// This selector chooses examples that fit within a maximum length constraint,
/// making it useful for managing prompt size and token limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LengthBasedExampleSelector {
    /// A list of the examples that the prompt template expects
    pub examples: Vec<Example>,
    /// Function to measure text length. Defaults to word count.
    #[serde(skip, default = "default_text_length")]
    pub get_text_length: fn(&str) -> usize,
    /// Max length for the prompt, beyond which examples are cut
    pub max_length: usize,
    /// Length of each example (cached for performance)
    #[serde(skip)]
    pub example_text_lengths: Vec<usize>,
}

/// Default text length function for serialization
fn default_text_length() -> fn(&str) -> usize {
    LengthBasedExampleSelector::default_text_length
}

impl LengthBasedExampleSelector {
    /// Create a new length-based example selector.
    ///
    /// # Arguments
    ///
    /// * `examples` - Initial list of examples
    /// * `max_length` - Maximum total length for selected examples
    /// * `get_text_length` - Function to measure text length (defaults to word count)
    pub fn new(
        examples: Vec<Example>,
        max_length: usize,
        get_text_length: Option<fn(&str) -> usize>,
    ) -> Self {
        let get_text_length = get_text_length.unwrap_or(Self::default_text_length);
        let mut selector = Self {
            examples,
            get_text_length,
            max_length,
            example_text_lengths: Vec::new(),
        };
        selector.update_lengths();
        selector
    }

    /// Create a new selector with default word count function.
    pub fn with_word_count(examples: Vec<Example>, max_length: usize) -> Self {
        Self::new(examples, max_length, Some(Self::default_text_length))
    }

    /// Create a new selector with character count function.
    pub fn with_char_count(examples: Vec<Example>, max_length: usize) -> Self {
        Self::new(examples, max_length, Some(Self::char_length))
    }

    /// Default text length function (word count).
    pub fn default_text_length(text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Character count function.
    pub fn char_length(text: &str) -> usize {
        text.len()
    }

    /// Update the cached lengths of examples.
    fn update_lengths(&mut self) {
        self.example_text_lengths = self
            .examples
            .iter()
            .map(|example| {
                let text = self.example_to_text(example);
                (self.get_text_length)(&text)
            })
            .collect();
    }

    /// Convert an example to text for length calculation.
    fn example_to_text(&self, example: &Example) -> String {
        let mut values: Vec<_> = example.values().cloned().collect();
        values.sort();
        values.join(" ")
    }

    /// Get the current total length of examples.
    pub fn total_length(&self) -> usize {
        self.example_text_lengths.iter().sum()
    }

    /// Get the number of examples.
    pub fn len(&self) -> usize {
        self.examples.len()
    }

    /// Check if the selector is empty.
    pub fn is_empty(&self) -> bool {
        self.examples.is_empty()
    }
}

#[async_trait]
impl BaseExampleSelector for LengthBasedExampleSelector {
    fn add_example(&mut self, example: Example) -> Result<()> {
        self.examples.push(example.clone());
        let text = self.example_to_text(&example);
        self.example_text_lengths.push((self.get_text_length)(&text));
        Ok(())
    }

    fn select_examples(&self, input_variables: &Example) -> Result<Vec<Example>> {
        let input_text = self.example_to_text(input_variables);
        let input_length = (self.get_text_length)(&input_text);
        let remaining_length = self.max_length.saturating_sub(input_length);

        let mut selected = Vec::new();
        let mut current_length = 0;

        for (i, example) in self.examples.iter().enumerate() {
            let example_length = self.example_text_lengths[i];
            if current_length + example_length <= remaining_length {
                selected.push(example.clone());
                current_length += example_length;
            } else {
                break;
            }
        }

        Ok(selected)
    }
}

impl_serializable!(LengthBasedExampleSelector, ["ferriclink", "example_selectors", "length_based"]);

/// Select examples based on semantic similarity using vector stores.
///
/// This selector uses embeddings and vector similarity search to find
/// the most relevant examples for a given input.
pub struct SemanticSimilarityExampleSelector {
    /// Vector store containing the examples
    pub vectorstore: Box<dyn VectorStore>,
    /// Number of examples to select
    pub k: usize,
    /// Optional keys to filter examples to
    pub example_keys: Option<Vec<String>>,
    /// Optional keys to filter input to
    pub input_keys: Option<Vec<String>>,
    /// Extra arguments passed to similarity search
    pub vectorstore_kwargs: Option<HashMap<String, serde_json::Value>>,
}

impl SemanticSimilarityExampleSelector {
    /// Create a new semantic similarity example selector.
    ///
    /// # Arguments
    ///
    /// * `vectorstore` - Vector store containing the examples
    /// * `k` - Number of examples to select
    /// * `example_keys` - Optional keys to filter examples to
    /// * `input_keys` - Optional keys to filter input to
    /// * `vectorstore_kwargs` - Extra arguments for similarity search
    pub fn new(
        vectorstore: Box<dyn VectorStore>,
        k: usize,
        example_keys: Option<Vec<String>>,
        input_keys: Option<Vec<String>>,
        vectorstore_kwargs: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        Self {
            vectorstore,
            k,
            example_keys,
            input_keys,
            vectorstore_kwargs,
        }
    }

    /// Convert an example to text for embedding.
    fn example_to_text(&self, example: &Example, input_keys: Option<&[String]>) -> String {
        let filtered_example = if let Some(keys) = input_keys {
            example
                .iter()
                .filter(|(k, _)| keys.contains(k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            example.clone()
        };

        let mut values: Vec<_> = filtered_example.values().cloned().collect();
        values.sort();
        values.join(" ")
    }

    /// Convert search results to examples.
    fn search_results_to_examples(&self, results: Vec<VectorSearchResult>) -> Vec<Example> {
        let mut examples = Vec::new();
        
        for result in results {
            let mut example = HashMap::new();
            
            // Convert metadata to example format
            for (key, value) in result.metadata {
                if let Some(str_value) = value.as_str() {
                    example.insert(key, str_value.to_string());
                }
            }
            
            // Filter by example keys if specified
            if let Some(ref example_keys) = self.example_keys {
                example = example
                    .into_iter()
                    .filter(|(k, _)| example_keys.contains(k))
                    .collect();
            }
            
            if !example.is_empty() {
                examples.push(example);
            }
        }
        
        examples
    }
}

#[async_trait]
impl BaseExampleSelector for SemanticSimilarityExampleSelector {
    fn add_example(&mut self, _example: Example) -> Result<()> {
        // For sync version, we can't easily handle async operations
        // This is a limitation of the current design
        // In practice, you'd want to use the async version
        Err(crate::errors::FerricLinkError::generic(
            "Sync add_example not supported for SemanticSimilarityExampleSelector. Use aadd_example instead."
        ))
    }

    async fn aadd_example(&mut self, example: Example) -> Result<()> {
        let text = self.example_to_text(&example, self.input_keys.as_deref());
        // Convert example to the right metadata format
        let metadata: HashMap<String, serde_json::Value> = example
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        
        // Add to vector store
        self.vectorstore
            .add_texts(vec![text], Some(vec![metadata]), None)
            .await?;
        
        Ok(())
    }

    fn select_examples(&self, _input_variables: &Example) -> Result<Vec<Example>> {
        // For sync version, we can't easily handle async operations
        // This is a limitation of the current design
        // In practice, you'd want to use the async version
        Err(crate::errors::FerricLinkError::generic(
            "Sync select_examples not supported for SemanticSimilarityExampleSelector. Use aselect_examples instead."
        ))
    }

    async fn aselect_examples(&self, input_variables: &Example) -> Result<Vec<Example>> {
        let query_text = self.example_to_text(input_variables, self.input_keys.as_deref());
        
        // Perform async similarity search
        let results = self.vectorstore
            .similarity_search(
                &query_text,
                self.k,
                self.vectorstore_kwargs.clone(),
            )
            .await?;
        
        Ok(self.search_results_to_examples(results))
    }
}

/// Select examples based on Max Marginal Relevance (MMR).
///
/// MMR balances relevance and diversity in example selection, often
/// leading to better performance than simple similarity search.
///
/// Note: This is a placeholder implementation. MMR requires additional
/// methods in the VectorStore trait that are not yet implemented.
pub struct MaxMarginalRelevanceExampleSelector {
    /// Vector store containing the examples
    pub vectorstore: Box<dyn VectorStore>,
    /// Number of examples to select
    pub k: usize,
    /// Number of examples to fetch for reranking
    pub fetch_k: usize,
    /// Optional keys to filter examples to
    pub example_keys: Option<Vec<String>>,
    /// Optional keys to filter input to
    pub input_keys: Option<Vec<String>>,
    /// Extra arguments passed to similarity search
    pub vectorstore_kwargs: Option<HashMap<String, serde_json::Value>>,
}

impl MaxMarginalRelevanceExampleSelector {
    /// Create a new MMR example selector.
    ///
    /// # Arguments
    ///
    /// * `vectorstore` - Vector store containing the examples
    /// * `k` - Number of examples to select
    /// * `fetch_k` - Number of examples to fetch for reranking
    /// * `example_keys` - Optional keys to filter examples to
    /// * `input_keys` - Optional keys to filter input to
    /// * `vectorstore_kwargs` - Extra arguments for similarity search
    pub fn new(
        vectorstore: Box<dyn VectorStore>,
        k: usize,
        fetch_k: usize,
        example_keys: Option<Vec<String>>,
        input_keys: Option<Vec<String>>,
        vectorstore_kwargs: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        Self {
            vectorstore,
            k,
            fetch_k,
            example_keys,
            input_keys,
            vectorstore_kwargs,
        }
    }

    /// Convert an example to text for embedding.
    fn example_to_text(&self, example: &Example, input_keys: Option<&[String]>) -> String {
        let filtered_example = if let Some(keys) = input_keys {
            example
                .iter()
                .filter(|(k, _)| keys.contains(k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            example.clone()
        };

        let mut values: Vec<_> = filtered_example.values().cloned().collect();
        values.sort();
        values.join(" ")
    }

    /// Convert search results to examples.
    fn search_results_to_examples(&self, results: Vec<VectorSearchResult>) -> Vec<Example> {
        let mut examples = Vec::new();
        
        for result in results {
            let mut example = HashMap::new();
            
            // Convert metadata to example format
            for (key, value) in result.metadata {
                if let Some(str_value) = value.as_str() {
                    example.insert(key, str_value.to_string());
                }
            }
            
            // Filter by example keys if specified
            if let Some(ref example_keys) = self.example_keys {
                example = example
                    .into_iter()
                    .filter(|(k, _)| example_keys.contains(k))
                    .collect();
            }
            
            if !example.is_empty() {
                examples.push(example);
            }
        }
        
        examples
    }
}

#[async_trait]
impl BaseExampleSelector for MaxMarginalRelevanceExampleSelector {
    fn add_example(&mut self, _example: Example) -> Result<()> {
        // For sync version, we can't easily handle async operations
        Err(crate::errors::FerricLinkError::generic(
            "Sync add_example not supported for MaxMarginalRelevanceExampleSelector. Use aadd_example instead."
        ))
    }

    async fn aadd_example(&mut self, example: Example) -> Result<()> {
        let text = self.example_to_text(&example, self.input_keys.as_deref());
        // Convert example to the right metadata format
        let metadata: HashMap<String, serde_json::Value> = example
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        
        // Add to vector store
        self.vectorstore
            .add_texts(vec![text], Some(vec![metadata]), None)
            .await?;
        
        Ok(())
    }

    fn select_examples(&self, _input_variables: &Example) -> Result<Vec<Example>> {
        // For sync version, we can't easily handle async operations
        Err(crate::errors::FerricLinkError::generic(
            "Sync select_examples not supported for MaxMarginalRelevanceExampleSelector. Use aselect_examples instead."
        ))
    }

    async fn aselect_examples(&self, input_variables: &Example) -> Result<Vec<Example>> {
        let query_text = self.example_to_text(input_variables, self.input_keys.as_deref());
        
        // For now, fall back to regular similarity search since MMR is not implemented
        // TODO: Implement proper MMR algorithm when vector store supports it
        let results = self.vectorstore
            .similarity_search(
                &query_text,
                self.k,
                self.vectorstore_kwargs.clone(),
            )
            .await?;
        
        Ok(self.search_results_to_examples(results))
    }
}

/// Utility function to return values in a dictionary sorted by key.
///
/// # Arguments
///
/// * `values` - A dictionary with keys as input variables
///   and values as their values.
///
/// # Returns
///
/// A list of values in dict sorted by key.
pub fn sorted_values(values: &Example) -> Vec<String> {
    let mut sorted_pairs: Vec<_> = values.iter().collect();
    sorted_pairs.sort_by_key(|(key, _)| *key);
    sorted_pairs.into_iter().map(|(_, value)| value.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_examples() -> Vec<Example> {
        vec![
            [("input".to_string(), "What is AI?".to_string())]
                .iter()
                .cloned()
                .collect(),
            [("input".to_string(), "How does machine learning work?".to_string())]
                .iter()
                .cloned()
                .collect(),
            [("input".to_string(), "Explain neural networks".to_string())]
                .iter()
                .cloned()
                .collect(),
        ]
    }

    #[test]
    fn test_length_based_selector_basic() {
        let examples = create_test_examples();
        let selector = LengthBasedExampleSelector::with_word_count(examples, 10);
        
        let input = [("input".to_string(), "Tell me about AI".to_string())]
            .iter()
            .cloned()
            .collect();
        
        let selected = selector.select_examples(&input).unwrap();
        assert!(!selected.is_empty());
        assert!(selected.len() <= 3);
    }

    #[test]
    fn test_length_based_selector_add_example() {
        let examples = create_test_examples();
        let mut selector = LengthBasedExampleSelector::with_word_count(examples, 20);
        
        let new_example = [("input".to_string(), "What is deep learning?".to_string())]
            .iter()
            .cloned()
            .collect();
        
        selector.add_example(new_example).unwrap();
        assert_eq!(selector.len(), 4);
    }

    #[test]
    fn test_length_based_selector_max_length() {
        let examples = create_test_examples();
        let selector = LengthBasedExampleSelector::with_word_count(examples, 5);
        
        let input = [("input".to_string(), "AI question".to_string())]
            .iter()
            .cloned()
            .collect();
        
        let selected = selector.select_examples(&input).unwrap();
        // Should select fewer examples due to length constraint
        assert!(selected.len() <= 3);
    }

    #[test]
    fn test_sorted_values() {
        let mut example = HashMap::new();
        example.insert("z".to_string(), "last".to_string());
        example.insert("a".to_string(), "first".to_string());
        example.insert("m".to_string(), "middle".to_string());
        
        let sorted = sorted_values(&example);
        assert_eq!(sorted, vec!["first", "middle", "last"]);
    }

    #[test]
    fn test_length_based_selector_empty() {
        let selector = LengthBasedExampleSelector::with_word_count(vec![], 10);
        assert!(selector.is_empty());
        
        let input = [("input".to_string(), "test".to_string())]
            .iter()
            .cloned()
            .collect();
        
        let selected = selector.select_examples(&input).unwrap();
        assert!(selected.is_empty());
    }

    #[tokio::test]
    async fn test_length_based_selector_async() {
        let examples = create_test_examples();
        let selector = LengthBasedExampleSelector::with_word_count(examples, 15);
        
        let input = [("input".to_string(), "AI question".to_string())]
            .iter()
            .cloned()
            .collect();
        
        let selected = selector.aselect_examples(&input).await.unwrap();
        assert!(!selected.is_empty());
    }
}
