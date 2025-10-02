//! Runnable trait and related abstractions for FerricLink Core
//!
//! This module provides the core Runnable trait that powers the FerricLink ecosystem,
//! similar to LangChain's Runnable interface.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use crate::errors::Result;
use crate::impl_serializable;
use crate::utils::{colors, print_colored_text};

/// Configuration for running a Runnable
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct RunnableConfig {
    /// Tags for this run
    #[serde(default)]
    pub tags: Vec<String>,
    /// Metadata for this run
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Whether to run in debug mode
    #[serde(default)]
    pub debug: bool,
    /// Whether to run in verbose mode
    #[serde(default)]
    pub verbose: bool,
    /// Callback handlers for this run
    #[serde(skip)]
    pub callbacks: Vec<Arc<dyn CallbackHandler>>,
}


impl RunnableConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tag to the configuration
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add metadata to the configuration
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Enable verbose mode
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Add a callback handler
    pub fn with_callback(mut self, callback: Arc<dyn CallbackHandler>) -> Self {
        self.callbacks.push(callback);
        self
    }
}

impl PartialEq for RunnableConfig {
    fn eq(&self, other: &Self) -> bool {
        self.tags == other.tags
            && self.metadata == other.metadata
            && self.debug == other.debug
            && self.verbose == other.verbose
        // Skip callbacks comparison
    }
}

impl_serializable!(RunnableConfig, ["ferriclink", "runnables", "config"]);

/// Trait for callback handlers that can be used during runnable execution
#[async_trait]
pub trait CallbackHandler: Send + Sync {
    /// Called when a runnable starts
    async fn on_start(&self, run_id: &str, input: &serde_json::Value) -> Result<()> {
        let _ = (run_id, input);
        Ok(())
    }

    /// Called when a runnable completes successfully
    async fn on_success(&self, run_id: &str, output: &serde_json::Value) -> Result<()> {
        let _ = (run_id, output);
        Ok(())
    }

    /// Called when a runnable fails
    async fn on_error(&self, run_id: &str, error: &crate::errors::FerricLinkError) -> Result<()> {
        let _ = (run_id, error);
        Ok(())
    }

    /// Called when a runnable produces streaming output
    async fn on_stream(&self, run_id: &str, chunk: &serde_json::Value) -> Result<()> {
        let _ = (run_id, chunk);
        Ok(())
    }
}

/// A simple console callback handler for debugging
pub struct ConsoleCallbackHandler {
    /// The color to use for text output (matching LangChain's color scheme)
    pub color: Option<String>,
}

impl ConsoleCallbackHandler {
    /// Create a new console callback handler
    pub fn new() -> Self {
        Self { color: None }
    }

    /// Create a new console callback handler with color
    pub fn new_with_color(color: impl Into<String>) -> Self {
        Self {
            color: Some(color.into()),
        }
    }
}

impl Default for ConsoleCallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CallbackHandler for ConsoleCallbackHandler {
    async fn on_start(&self, run_id: &str, input: &serde_json::Value) -> Result<()> {
        let message = format!("Starting run {run_id} with input: {input}");
        print_colored_text(&message, self.color.as_deref());
        Ok(())
    }

    async fn on_success(&self, run_id: &str, output: &serde_json::Value) -> Result<()> {
        let message = format!("Run {run_id} completed with output: {output}");
        print_colored_text(&message, self.color.as_deref());
        Ok(())
    }

    async fn on_error(&self, run_id: &str, error: &crate::errors::FerricLinkError) -> Result<()> {
        let message = format!("Run {run_id} failed with error: {error}");
        print_colored_text(&message, Some(colors::RED));
        Ok(())
    }

    async fn on_stream(&self, run_id: &str, chunk: &serde_json::Value) -> Result<()> {
        let message = format!("Run {run_id} streamed: {chunk}");
        print_colored_text(&message, self.color.as_deref());
        Ok(())
    }
}

/// The core Runnable trait that all FerricLink components implement
#[async_trait]
pub trait Runnable<Input, Output>: Send + Sync + 'static
where
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    /// Invoke the runnable with a single input
    async fn invoke(&self, input: Input, config: Option<RunnableConfig>) -> Result<Output>;

    /// Invoke the runnable with a single input (convenience method with default config)
    async fn invoke_simple(&self, input: Input) -> Result<Output> {
        self.invoke(input, None).await
    }

    /// Batch invoke the runnable with multiple inputs
    async fn batch(
        &self,
        inputs: Vec<Input>,
        config: Option<RunnableConfig>,
    ) -> Result<Vec<Output>> {
        let mut results = Vec::new();
        for input in inputs {
            let result = self.invoke(input, config.clone()).await?;
            results.push(result);
        }
        Ok(results)
    }

    /// Stream the output of the runnable
    async fn stream(
        &self,
        input: Input,
        config: Option<RunnableConfig>,
    ) -> Result<Pin<Box<dyn futures::Stream<Item = Result<Output>> + Send>>> {
        // Default implementation just yields the single result
        let result = self.invoke(input, config).await;
        let stream = futures::stream::once(async { result });
        Ok(Box::pin(stream))
    }

    /// Get the input schema for this runnable
    fn input_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Get the output schema for this runnable
    fn output_schema(&self) -> Option<serde_json::Value> {
        None
    }

    /// Get the configuration schema for this runnable
    fn config_schema(&self) -> Option<serde_json::Value> {
        None
    }
}

/// A runnable that wraps a simple function
pub struct RunnableLambda<F, Input, Output>
where
    F: Fn(Input) -> Result<Output> + Send + Sync + 'static,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    func: F,
    _phantom: std::marker::PhantomData<(Input, Output)>,
}

impl<F, Input, Output> RunnableLambda<F, Input, Output>
where
    F: Fn(Input) -> Result<Output> + Send + Sync + 'static,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    /// Create a new runnable lambda
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<F, Input, Output> Runnable<Input, Output> for RunnableLambda<F, Input, Output>
where
    F: Fn(Input) -> Result<Output> + Send + Sync + 'static,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    async fn invoke(&self, input: Input, _config: Option<RunnableConfig>) -> Result<Output> {
        (self.func)(input)
    }
}

/// A runnable that wraps an async function
pub struct RunnableAsync<F, Input, Output, Fut>
where
    F: Fn(Input) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Output>> + Send + 'static,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    func: F,
    _phantom: std::marker::PhantomData<(Input, Output)>,
}

impl<F, Input, Output, Fut> RunnableAsync<F, Input, Output, Fut>
where
    F: Fn(Input) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Output>> + Send + 'static,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    /// Create a new async runnable
    pub fn new(func: F) -> Self {
        Self {
            func,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<F, Input, Output, Fut> Runnable<Input, Output> for RunnableAsync<F, Input, Output, Fut>
where
    F: Fn(Input) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Output>> + Send + 'static,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    async fn invoke(&self, input: Input, _config: Option<RunnableConfig>) -> Result<Output> {
        (self.func)(input).await
    }
}

/// A runnable sequence that chains multiple runnables together
pub struct RunnableSequence<Input, Intermediate, Output> {
    first: Arc<dyn Runnable<Input, Intermediate>>,
    second: Arc<dyn Runnable<Intermediate, Output>>,
}

impl<Input, Intermediate, Output> RunnableSequence<Input, Intermediate, Output>
where
    Input: Send + Sync + 'static,
    Intermediate: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    /// Create a new runnable sequence
    pub fn new(
        first: Arc<dyn Runnable<Input, Intermediate>>,
        second: Arc<dyn Runnable<Intermediate, Output>>,
    ) -> Self {
        Self { first, second }
    }
}

#[async_trait]
impl<Input, Intermediate, Output> Runnable<Input, Output>
    for RunnableSequence<Input, Intermediate, Output>
where
    Input: Send + Sync + 'static,
    Intermediate: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    async fn invoke(&self, input: Input, config: Option<RunnableConfig>) -> Result<Output> {
        let intermediate = self.first.invoke(input, config.clone()).await?;
        self.second.invoke(intermediate, config).await
    }
}

/// A runnable that runs multiple runnables in parallel
pub struct RunnableParallel<Input, Output> {
    runnables: Vec<Arc<dyn Runnable<Input, Output>>>,
}

impl<Input, Output> RunnableParallel<Input, Output>
where
    Input: Send + Sync + 'static + Clone,
    Output: Send + Sync + 'static,
{
    /// Create a new runnable parallel
    pub fn new(runnables: Vec<Arc<dyn Runnable<Input, Output>>>) -> Self {
        Self { runnables }
    }

    /// Add a runnable to the parallel execution
    pub fn add_runnable(&mut self, runnable: Arc<dyn Runnable<Input, Output>>) {
        self.runnables.push(runnable);
    }
}

#[async_trait]
impl<Input, Output> Runnable<Input, Vec<Output>> for RunnableParallel<Input, Output>
where
    Input: Send + Sync + 'static + Clone,
    Output: Send + Sync + 'static,
{
    async fn invoke(&self, input: Input, config: Option<RunnableConfig>) -> Result<Vec<Output>> {
        let mut handles = Vec::new();

        for runnable in &self.runnables {
            let runnable = runnable.clone();
            let input = input.clone();
            let config = config.clone();

            let handle = tokio::spawn(async move { runnable.invoke(input, config).await });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.map_err(|e| {
                crate::errors::FerricLinkError::runtime(format!("Task failed: {e}"))
            })?;
            results.push(result?);
        }

        Ok(results)
    }
}

/// Helper function to create a runnable from a simple function
pub fn runnable<F, Input, Output>(func: F) -> Arc<dyn Runnable<Input, Output>>
where
    F: Fn(Input) -> Result<Output> + Send + Sync + 'static,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    Arc::new(RunnableLambda::new(func))
}

/// Helper function to create an async runnable from a function
pub fn runnable_async<F, Input, Output, Fut>(func: F) -> Arc<dyn Runnable<Input, Output>>
where
    F: Fn(Input) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Output>> + Send + 'static,
    Input: Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    Arc::new(RunnableAsync::new(func))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runnable_lambda() {
        let runnable = RunnableLambda::new(|x: i32| Ok(x * 2));
        let result = runnable.invoke_simple(5).await.unwrap();
        assert_eq!(result, 10);
    }

    #[tokio::test]
    async fn test_runnable_async() {
        let runnable = RunnableAsync::new(|x: i32| async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            Ok(x * 3)
        });
        let result = runnable.invoke_simple(4).await.unwrap();
        assert_eq!(result, 12);
    }

    #[tokio::test]
    async fn test_runnable_sequence() {
        let first = Arc::new(RunnableLambda::new(|x: i32| Ok(x + 1)));
        let second = Arc::new(RunnableLambda::new(|x: i32| Ok(x * 2)));
        let sequence = RunnableSequence::new(first, second);

        let result = sequence.invoke_simple(5).await.unwrap();
        assert_eq!(result, 12); // (5 + 1) * 2
    }

    #[tokio::test]
    async fn test_runnable_parallel() {
        let runnable1 = Arc::new(RunnableLambda::new(|x: i32| Ok(x * 2)));
        let runnable2 = Arc::new(RunnableLambda::new(|x: i32| Ok(x * 3)));
        let parallel = RunnableParallel::new(vec![runnable1, runnable2]);

        let results = parallel.invoke_simple(5).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&10)); // 5 * 2
        assert!(results.contains(&15)); // 5 * 3
    }

    #[tokio::test]
    async fn test_runnable_batch() {
        let runnable = RunnableLambda::new(|x: i32| Ok(x * 2));
        let results = runnable.batch(vec![1, 2, 3], None).await.unwrap();
        assert_eq!(results, vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn test_runnable_config() {
        let config = RunnableConfig::new()
            .with_tag("test")
            .with_metadata("key", serde_json::Value::String("value".to_string()))
            .with_debug(true);

        assert!(config.tags.contains(&"test".to_string()));
        assert_eq!(
            config.metadata.get("key"),
            Some(&serde_json::Value::String("value".to_string()))
        );
        assert!(config.debug);
    }

    #[tokio::test]
    async fn test_console_callback_handler() {
        let handler = ConsoleCallbackHandler::new();
        let run_id = "test-run";
        let input = serde_json::Value::String("test input".to_string());
        let output = serde_json::Value::String("test output".to_string());
        let error = crate::errors::FerricLinkError::generic("test error");

        // These should not panic
        handler.on_start(run_id, &input).await.unwrap();
        handler.on_success(run_id, &output).await.unwrap();
        handler.on_error(run_id, &error).await.unwrap();
        handler.on_stream(run_id, &output).await.unwrap();
    }

    #[tokio::test]
    async fn test_helper_functions() {
        let sync_runnable = runnable(|x: i32| Ok(x + 1));
        let result1 = sync_runnable.invoke_simple(5).await.unwrap();
        assert_eq!(result1, 6);

        let async_runnable = runnable_async(|x: i32| async move { Ok(x * 2) });
        let result2 = async_runnable.invoke_simple(3).await.unwrap();
        assert_eq!(result2, 6);
    }
}
