//! Language model abstractions for FerricLink Core
//!
//! This module provides the core abstractions for language models, including
//! base traits for LLMs and chat models.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;

use crate::errors::Result;
use crate::impl_serializable;
use crate::messages::AnyMessage;
use crate::runnables::RunnableConfig;

/// Configuration for language model generation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationConfig {
    /// Temperature for generation (0.0 to 1.0)
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Maximum number of tokens to generate
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Stop sequences
    #[serde(default)]
    pub stop: Vec<String>,
    /// Top-p sampling parameter
    #[serde(default)]
    pub top_p: Option<f32>,
    /// Top-k sampling parameter
    #[serde(default)]
    pub top_k: Option<u32>,
    /// Presence penalty
    #[serde(default)]
    pub presence_penalty: Option<f32>,
    /// Frequency penalty
    #[serde(default)]
    pub frequency_penalty: Option<f32>,
    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,
    /// Additional parameters
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            max_tokens: None,
            stop: Vec::new(),
            top_p: None,
            top_k: None,
            presence_penalty: None,
            frequency_penalty: None,
            stream: false,
            extra: HashMap::new(),
        }
    }
}

impl GenerationConfig {
    /// Create a new generation config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Add a stop sequence
    pub fn with_stop(mut self, stop: impl Into<String>) -> Self {
        self.stop.push(stop.into());
        self
    }

    /// Enable streaming
    pub fn with_streaming(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Add an extra parameter
    pub fn with_extra(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }
}

impl_serializable!(
    GenerationConfig,
    ["ferriclink", "language_models", "generation_config"]
);

/// A generation result from a language model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Generation {
    /// The generated text
    pub text: String,
    /// Generation metadata
    #[serde(default)]
    pub generation_info: HashMap<String, serde_json::Value>,
}

impl Generation {
    /// Create a new generation
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            generation_info: HashMap::new(),
        }
    }

    /// Create a new generation with metadata
    pub fn new_with_info(
        text: impl Into<String>,
        generation_info: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            text: text.into(),
            generation_info,
        }
    }
}

impl_serializable!(Generation, ["ferriclink", "language_models", "generation"]);

/// A result containing multiple generations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LLMResult {
    /// The generations
    pub generations: Vec<Vec<Generation>>,
    /// Result metadata
    #[serde(default)]
    pub llm_output: HashMap<String, serde_json::Value>,
}

impl LLMResult {
    /// Create a new LLM result
    pub fn new(generations: Vec<Vec<Generation>>) -> Self {
        Self {
            generations,
            llm_output: HashMap::new(),
        }
    }

    /// Create a new LLM result with metadata
    pub fn new_with_output(
        generations: Vec<Vec<Generation>>,
        llm_output: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            generations,
            llm_output,
        }
    }

    /// Get the first generation text
    pub fn first_text(&self) -> Option<&str> {
        self.generations.first()?.first().map(|g| g.text.as_str())
    }

    /// Get all generation texts
    pub fn all_texts(&self) -> Vec<&str> {
        self.generations
            .iter()
            .flat_map(|gens| gens.iter().map(|g| g.text.as_str()))
            .collect()
    }
}

impl_serializable!(LLMResult, ["ferriclink", "language_models", "llm_result"]);

/// Base trait for all language models
#[async_trait]
pub trait BaseLanguageModel: Send + Sync + 'static {
    /// Get the model name
    fn model_name(&self) -> &str;

    /// Get the model type
    fn model_type(&self) -> &str;

    /// Check if the model supports streaming
    fn supports_streaming(&self) -> bool {
        false
    }

    /// Get the input schema for this model
    fn input_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Get the output schema for this model
    fn output_schema(&self) -> Option<serde_json::Value> {
        None
    }
}

/// Trait for language models that generate text from text input
#[async_trait]
pub trait BaseLLM: BaseLanguageModel {
    /// Generate text from a prompt
    async fn generate(
        &self,
        prompt: &str,
        config: Option<GenerationConfig>,
        runnable_config: Option<RunnableConfig>,
    ) -> Result<LLMResult>;

    /// Generate text from multiple prompts
    async fn generate_batch(
        &self,
        prompts: Vec<String>,
        config: Option<GenerationConfig>,
        runnable_config: Option<RunnableConfig>,
    ) -> Result<Vec<LLMResult>> {
        let mut results = Vec::new();
        for prompt in prompts {
            let result = self
                .generate(&prompt, config.clone(), runnable_config.clone())
                .await?;
            results.push(result);
        }
        Ok(results)
    }

    /// Stream text generation
    async fn stream_generate(
        &self,
        prompt: &str,
        config: Option<GenerationConfig>,
        runnable_config: Option<RunnableConfig>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<Generation>> + Send>>> {
        // Default implementation just yields the single result
        let result = self.generate(prompt, config, runnable_config).await?;
        let generation = result
            .generations
            .into_iter()
            .next()
            .and_then(|gens| gens.into_iter().next())
            .unwrap_or_else(|| Generation::new(""));
        let stream = futures::stream::once(async { Ok(generation) });
        Ok(Box::pin(stream))
    }
}

/// Trait for language models that work with chat messages
#[async_trait]
pub trait BaseChatModel: BaseLanguageModel {
    /// Generate a response from chat messages
    async fn generate_chat(
        &self,
        messages: Vec<AnyMessage>,
        config: Option<GenerationConfig>,
        runnable_config: Option<RunnableConfig>,
    ) -> Result<AnyMessage>;

    /// Generate responses from multiple chat conversations
    async fn generate_chat_batch(
        &self,
        conversations: Vec<Vec<AnyMessage>>,
        config: Option<GenerationConfig>,
        runnable_config: Option<RunnableConfig>,
    ) -> Result<Vec<AnyMessage>> {
        let mut results = Vec::new();
        for messages in conversations {
            let result = self
                .generate_chat(messages, config.clone(), runnable_config.clone())
                .await?;
            results.push(result);
        }
        Ok(results)
    }

    /// Stream chat generation
    async fn stream_chat(
        &self,
        messages: Vec<AnyMessage>,
        config: Option<GenerationConfig>,
        runnable_config: Option<RunnableConfig>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<AnyMessage>> + Send>>> {
        // Default implementation just yields the single result
        let result = self
            .generate_chat(messages, config, runnable_config)
            .await?;
        let stream = futures::stream::once(async { Ok(result) });
        Ok(Box::pin(stream))
    }
}

/// A simple mock LLM for testing
pub struct MockLLM {
    model_name: String,
    responses: Vec<String>,
    current_index: std::sync::atomic::AtomicUsize,
}

impl MockLLM {
    /// Create a new mock LLM
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
            responses: Vec::new(),
            current_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Add a response to the mock
    pub fn add_response(mut self, response: impl Into<String>) -> Self {
        self.responses.push(response.into());
        self
    }

    /// Get the next response
    fn get_next_response(&self) -> String {
        if self.responses.is_empty() {
            "Mock response".to_string()
        } else {
            let index = self
                .current_index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.responses[index % self.responses.len()].clone()
        }
    }
}

#[async_trait]
impl BaseLanguageModel for MockLLM {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn model_type(&self) -> &str {
        "mock_llm"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[async_trait]
impl BaseLLM for MockLLM {
    async fn generate(
        &self,
        _prompt: &str,
        _config: Option<GenerationConfig>,
        _runnable_config: Option<RunnableConfig>,
    ) -> Result<LLMResult> {
        let response = self.get_next_response();
        let generation = Generation::new(response);
        Ok(LLMResult::new(vec![vec![generation]]))
    }
}

/// A simple mock chat model for testing
pub struct MockChatModel {
    model_name: String,
    responses: Vec<String>,
    current_index: std::sync::atomic::AtomicUsize,
}

impl MockChatModel {
    /// Create a new mock chat model
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
            responses: Vec::new(),
            current_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Add a response to the mock
    pub fn add_response(mut self, response: impl Into<String>) -> Self {
        self.responses.push(response.into());
        self
    }

    /// Get the next response
    fn get_next_response(&self) -> String {
        if self.responses.is_empty() {
            "Mock chat response".to_string()
        } else {
            let index = self
                .current_index
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.responses[index % self.responses.len()].clone()
        }
    }
}

#[async_trait]
impl BaseLanguageModel for MockChatModel {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn model_type(&self) -> &str {
        "mock_chat_model"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[async_trait]
impl BaseChatModel for MockChatModel {
    async fn generate_chat(
        &self,
        _messages: Vec<AnyMessage>,
        _config: Option<GenerationConfig>,
        _runnable_config: Option<RunnableConfig>,
    ) -> Result<AnyMessage> {
        let response = self.get_next_response();
        Ok(AnyMessage::ai(response))
    }
}

/// Helper function to create a mock LLM
pub fn mock_llm(model_name: impl Into<String>) -> MockLLM {
    MockLLM::new(model_name)
}

/// Helper function to create a mock chat model
pub fn mock_chat_model(model_name: impl Into<String>) -> MockChatModel {
    MockChatModel::new(model_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::BaseMessage;
    use crate::serializable::Serializable;

    #[test]
    fn test_generation_config() {
        let config = GenerationConfig::new()
            .with_temperature(0.8)
            .with_max_tokens(100)
            .with_stop("STOP")
            .with_streaming(true);

        assert_eq!(config.temperature, Some(0.8));
        assert_eq!(config.max_tokens, Some(100));
        assert!(config.stop.contains(&"STOP".to_string()));
        assert!(config.stream);
    }

    #[test]
    fn test_generation() {
        let generation = Generation::new("Hello, world!");
        assert_eq!(generation.text, "Hello, world!");
        assert!(generation.generation_info.is_empty());
    }

    #[test]
    fn test_llm_result() {
        let generations = vec![
            vec![Generation::new("Hello")],
            vec![Generation::new("World")],
        ];
        let result = LLMResult::new(generations);

        assert_eq!(result.generations.len(), 2);
        assert_eq!(result.first_text(), Some("Hello"));
        assert_eq!(result.all_texts(), vec!["Hello", "World"]);
    }

    #[tokio::test]
    async fn test_mock_llm() {
        let llm = MockLLM::new("test-model")
            .add_response("Response 1")
            .add_response("Response 2");

        assert_eq!(llm.model_name(), "test-model");
        assert_eq!(llm.model_type(), "mock_llm");
        assert!(llm.supports_streaming());

        let result = llm.generate("test prompt", None, None).await.unwrap();
        assert_eq!(result.first_text(), Some("Response 1"));

        let result2 = llm.generate("test prompt", None, None).await.unwrap();
        assert_eq!(result2.first_text(), Some("Response 2"));
    }

    #[tokio::test]
    async fn test_mock_chat_model() {
        let chat_model = MockChatModel::new("test-chat-model")
            .add_response("Chat response 1")
            .add_response("Chat response 2");

        assert_eq!(chat_model.model_name(), "test-chat-model");
        assert_eq!(chat_model.model_type(), "mock_chat_model");
        assert!(chat_model.supports_streaming());

        let messages = vec![AnyMessage::human("Hello")];
        let result = chat_model
            .generate_chat(messages, None, None)
            .await
            .unwrap();
        assert!(result.is_ai());
        assert_eq!(result.text(), "Chat response 1");
    }

    #[tokio::test]
    async fn test_llm_batch_generation() {
        let llm = MockLLM::new("test-model")
            .add_response("Response 1")
            .add_response("Response 2");

        let prompts = vec!["prompt 1".to_string(), "prompt 2".to_string()];
        let results = llm.generate_batch(prompts, None, None).await.unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].first_text(), Some("Response 1"));
        assert_eq!(results[1].first_text(), Some("Response 2"));
    }

    #[tokio::test]
    async fn test_chat_batch_generation() {
        let chat_model = MockChatModel::new("test-chat-model")
            .add_response("Chat 1")
            .add_response("Chat 2");

        let conversations = vec![
            vec![AnyMessage::human("Hello 1")],
            vec![AnyMessage::human("Hello 2")],
        ];
        let results = chat_model
            .generate_chat_batch(conversations, None, None)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].text(), "Chat 1");
        assert_eq!(results[1].text(), "Chat 2");
    }

    #[test]
    fn test_serialization() {
        let config = GenerationConfig::new().with_temperature(0.8);
        let json = config.to_json().unwrap();
        let deserialized: GenerationConfig = GenerationConfig::from_json(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
