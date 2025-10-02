//! Callback system for FerricLink Core
//!
//! This module provides a comprehensive callback system for monitoring
//! and tracing the execution of FerricLink components.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::errors::Result;
use crate::impl_serializable;
use crate::utils::{colors, print_bold_text, print_colored_text};

/// A run ID for tracking execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunId {
    /// The unique identifier
    pub id: String,
    /// The timestamp when the run was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl RunId {
    /// Create a new run ID
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create a new run ID with a custom ID
    pub fn new_with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            created_at: chrono::Utc::now(),
        }
    }
}

impl Default for RunId {
    fn default() -> Self {
        Self::new()
    }
}

impl_serializable!(RunId, ["ferriclink", "callbacks", "run_id"]);

/// Information about a run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunInfo {
    /// The run ID
    pub run_id: RunId,
    /// The name of the component being run
    pub name: String,
    /// The type of the component
    pub component_type: String,
    /// Input to the component
    pub input: serde_json::Value,
    /// Output from the component (if completed)
    pub output: Option<serde_json::Value>,
    /// Error that occurred (if any)
    pub error: Option<String>,
    /// Start time of the run
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End time of the run (if completed)
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Duration of the run (if completed)
    pub duration: Option<Duration>,
    /// Tags associated with the run
    #[serde(default)]
    pub tags: Vec<String>,
    /// Metadata associated with the run
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Parent run ID (if this is a sub-run)
    pub parent_run_id: Option<RunId>,
    /// Child run IDs
    #[serde(default)]
    pub child_run_ids: Vec<RunId>,
}

impl RunInfo {
    /// Create a new run info
    pub fn new(
        run_id: RunId,
        name: impl Into<String>,
        component_type: impl Into<String>,
        input: serde_json::Value,
    ) -> Self {
        Self {
            run_id,
            name: name.into(),
            component_type: component_type.into(),
            input,
            output: None,
            error: None,
            start_time: chrono::Utc::now(),
            end_time: None,
            duration: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
            parent_run_id: None,
            child_run_ids: Vec::new(),
        }
    }

    /// Mark the run as completed with output
    pub fn complete_with_output(mut self, output: serde_json::Value) -> Self {
        self.output = Some(output);
        self.end_time = Some(chrono::Utc::now());
        self.duration = Some(
            (self.end_time.unwrap() - self.start_time)
                .to_std()
                .unwrap_or_default(),
        );
        self
    }

    /// Mark the run as failed with error
    pub fn complete_with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self.end_time = Some(chrono::Utc::now());
        self.duration = Some(
            (self.end_time.unwrap() - self.start_time)
                .to_std()
                .unwrap_or_default(),
        );
        self
    }

    /// Add a tag to the run
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add metadata to the run
    pub fn add_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Set the parent run ID
    pub fn with_parent(mut self, parent_run_id: RunId) -> Self {
        self.parent_run_id = Some(parent_run_id);
        self
    }

    /// Add a child run ID
    pub fn add_child(mut self, child_run_id: RunId) -> Self {
        self.child_run_ids.push(child_run_id);
        self
    }

    /// Check if the run is completed
    pub fn is_completed(&self) -> bool {
        self.end_time.is_some()
    }

    /// Check if the run failed
    pub fn is_failed(&self) -> bool {
        self.error.is_some()
    }

    /// Check if the run succeeded
    pub fn is_successful(&self) -> bool {
        self.is_completed() && !self.is_failed()
    }
}

impl_serializable!(RunInfo, ["ferriclink", "callbacks", "run_info"]);

/// Base trait for all callback handlers
#[async_trait]
pub trait CallbackHandler: Send + Sync + 'static {
    /// Called when a run starts
    async fn on_run_start(&self, run_info: &RunInfo) -> Result<()> {
        let _ = run_info;
        Ok(())
    }

    /// Called when a run completes successfully
    async fn on_run_success(&self, run_info: &RunInfo) -> Result<()> {
        let _ = run_info;
        Ok(())
    }

    /// Called when a run fails
    async fn on_run_error(&self, run_info: &RunInfo) -> Result<()> {
        let _ = run_info;
        Ok(())
    }

    /// Called when a run produces streaming output
    async fn on_run_stream(&self, run_info: &RunInfo, chunk: &serde_json::Value) -> Result<()> {
        let _ = (run_info, chunk);
        Ok(())
    }

    /// Called when a run is cancelled
    async fn on_run_cancel(&self, run_info: &RunInfo) -> Result<()> {
        let _ = run_info;
        Ok(())
    }
}

/// A console callback handler that prints run information to stdout
pub struct ConsoleCallbackHandler {
    /// Whether to print detailed information
    pub verbose: bool,
    /// The color to use for text output (matching LangChain's color scheme)
    pub color: Option<String>,
}

impl ConsoleCallbackHandler {
    /// Create a new console callback handler
    pub fn new() -> Self {
        Self {
            verbose: false,
            color: None,
        }
    }

    /// Create a new console callback handler with verbosity setting
    pub fn new_with_verbose(verbose: bool) -> Self {
        Self {
            verbose,
            color: None,
        }
    }

    /// Create a new console callback handler with color
    pub fn new_with_color(color: impl Into<String>) -> Self {
        Self {
            verbose: false,
            color: Some(color.into()),
        }
    }

    /// Create a new console callback handler with verbosity and color
    pub fn new_with_verbose_and_color(verbose: bool, color: impl Into<String>) -> Self {
        Self {
            verbose,
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
    async fn on_run_start(&self, run_info: &RunInfo) -> Result<()> {
        // Match LangChain's format: "> Entering new {name} chain..."
        let message = format!("\n\n> Entering new {} chain...", run_info.name);
        print_bold_text(&message);

        if self.verbose {
            println!("   Input: {}", run_info.input);
            if !run_info.tags.is_empty() {
                println!("   Tags: {:?}", run_info.tags);
            }
        }

        Ok(())
    }

    async fn on_run_success(&self, run_info: &RunInfo) -> Result<()> {
        // Match LangChain's format: "> Finished chain."
        print_bold_text("\n> Finished chain.");

        if self.verbose {
            if let Some(output) = &run_info.output {
                let color = self.color.as_deref();
                print_colored_text(&format!("   Output: {}", output), color);
            }
        }

        Ok(())
    }

    async fn on_run_error(&self, run_info: &RunInfo) -> Result<()> {
        let error_msg = run_info.error.as_deref().unwrap_or("Unknown error");
        print_colored_text(&format!("\n> Error: {}", error_msg), Some(colors::RED));

        Ok(())
    }

    async fn on_run_stream(&self, _run_info: &RunInfo, chunk: &serde_json::Value) -> Result<()> {
        let color = self.color.as_deref();
        print_colored_text(&format!("{}", chunk), color);
        Ok(())
    }

    async fn on_run_cancel(&self, run_info: &RunInfo) -> Result<()> {
        print_colored_text(
            &format!("\n> Run {} was cancelled", run_info.run_id.id),
            Some(colors::YELLOW),
        );
        Ok(())
    }
}

/// A callback handler that collects run information in memory
pub struct MemoryCallbackHandler {
    runs: Arc<tokio::sync::RwLock<Vec<RunInfo>>>,
}

impl MemoryCallbackHandler {
    /// Create a new memory callback handler
    pub fn new() -> Self {
        Self {
            runs: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Get all runs
    pub async fn get_runs(&self) -> Vec<RunInfo> {
        self.runs.read().await.clone()
    }

    /// Get runs by name
    pub async fn get_runs_by_name(&self, name: &str) -> Vec<RunInfo> {
        self.runs
            .read()
            .await
            .iter()
            .filter(|run| run.name == name)
            .cloned()
            .collect()
    }

    /// Get runs by component type
    pub async fn get_runs_by_type(&self, component_type: &str) -> Vec<RunInfo> {
        self.runs
            .read()
            .await
            .iter()
            .filter(|run| run.component_type == component_type)
            .cloned()
            .collect()
    }

    /// Get successful runs
    pub async fn get_successful_runs(&self) -> Vec<RunInfo> {
        self.runs
            .read()
            .await
            .iter()
            .filter(|run| run.is_successful())
            .cloned()
            .collect()
    }

    /// Get failed runs
    pub async fn get_failed_runs(&self) -> Vec<RunInfo> {
        self.runs
            .read()
            .await
            .iter()
            .filter(|run| run.is_failed())
            .cloned()
            .collect()
    }

    /// Clear all runs
    pub async fn clear(&self) {
        self.runs.write().await.clear();
    }

    /// Get the number of runs
    pub async fn len(&self) -> usize {
        self.runs.read().await.len()
    }
}

impl Default for MemoryCallbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CallbackHandler for MemoryCallbackHandler {
    async fn on_run_start(&self, run_info: &RunInfo) -> Result<()> {
        self.runs.write().await.push(run_info.clone());
        Ok(())
    }

    async fn on_run_success(&self, run_info: &RunInfo) -> Result<()> {
        // Update the existing run info
        if let Some(existing_run) = self
            .runs
            .write()
            .await
            .iter_mut()
            .find(|run| run.run_id.id == run_info.run_id.id)
        {
            *existing_run = run_info.clone();
        }
        Ok(())
    }

    async fn on_run_error(&self, run_info: &RunInfo) -> Result<()> {
        // Update the existing run info
        if let Some(existing_run) = self
            .runs
            .write()
            .await
            .iter_mut()
            .find(|run| run.run_id.id == run_info.run_id.id)
        {
            *existing_run = run_info.clone();
        }
        Ok(())
    }
}

/// A callback manager that manages multiple callback handlers
pub struct CallbackManager {
    handlers: Vec<Arc<dyn CallbackHandler>>,
}

impl CallbackManager {
    /// Create a new callback manager
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Add a callback handler
    pub fn add_handler(&mut self, handler: Arc<dyn CallbackHandler>) {
        self.handlers.push(handler);
    }

    /// Remove all handlers
    pub fn clear(&mut self) {
        self.handlers.clear();
    }

    /// Get the number of handlers
    pub fn len(&self) -> usize {
        self.handlers.len()
    }

    /// Check if there are any handlers
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }

    /// Call all handlers for run start
    pub async fn on_run_start(&self, run_info: &RunInfo) -> Result<()> {
        for handler in &self.handlers {
            handler.on_run_start(run_info).await?;
        }
        Ok(())
    }

    /// Call all handlers for run success
    pub async fn on_run_success(&self, run_info: &RunInfo) -> Result<()> {
        for handler in &self.handlers {
            handler.on_run_success(run_info).await?;
        }
        Ok(())
    }

    /// Call all handlers for run error
    pub async fn on_run_error(&self, run_info: &RunInfo) -> Result<()> {
        for handler in &self.handlers {
            handler.on_run_error(run_info).await?;
        }
        Ok(())
    }

    /// Call all handlers for run stream
    pub async fn on_run_stream(&self, run_info: &RunInfo, chunk: &serde_json::Value) -> Result<()> {
        for handler in &self.handlers {
            handler.on_run_stream(run_info, chunk).await?;
        }
        Ok(())
    }

    /// Call all handlers for run cancel
    pub async fn on_run_cancel(&self, run_info: &RunInfo) -> Result<()> {
        for handler in &self.handlers {
            handler.on_run_cancel(run_info).await?;
        }
        Ok(())
    }
}

impl Default for CallbackManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a console callback handler
pub fn console_callback_handler() -> Arc<ConsoleCallbackHandler> {
    Arc::new(ConsoleCallbackHandler::new())
}

/// Helper function to create a verbose console callback handler
pub fn verbose_console_callback_handler() -> Arc<ConsoleCallbackHandler> {
    Arc::new(ConsoleCallbackHandler::new_with_verbose(true))
}

/// Helper function to create a colored console callback handler
pub fn colored_console_callback_handler(color: impl Into<String>) -> Arc<ConsoleCallbackHandler> {
    Arc::new(ConsoleCallbackHandler::new_with_color(color))
}

/// Helper function to create a verbose colored console callback handler
pub fn verbose_colored_console_callback_handler(
    verbose: bool,
    color: impl Into<String>,
) -> Arc<ConsoleCallbackHandler> {
    Arc::new(ConsoleCallbackHandler::new_with_verbose_and_color(
        verbose, color,
    ))
}

/// Helper function to create a memory callback handler
pub fn memory_callback_handler() -> Arc<MemoryCallbackHandler> {
    Arc::new(MemoryCallbackHandler::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializable::Serializable;

    #[test]
    fn test_run_id() {
        let run_id = RunId::new();
        assert!(!run_id.id.is_empty());
        assert!(run_id.created_at <= chrono::Utc::now());
    }

    #[test]
    fn test_run_info() {
        let run_id = RunId::new();
        let run_info = RunInfo::new(
            run_id.clone(),
            "test_component",
            "test_type",
            serde_json::json!({"input": "test"}),
        );

        assert_eq!(run_info.run_id, run_id);
        assert_eq!(run_info.name, "test_component");
        assert_eq!(run_info.component_type, "test_type");
        assert!(!run_info.is_completed());
        assert!(!run_info.is_failed());
        assert!(!run_info.is_successful());
    }

    #[test]
    fn test_run_info_completion() {
        let run_id = RunId::new();
        let mut run_info = RunInfo::new(
            run_id,
            "test_component",
            "test_type",
            serde_json::json!({"input": "test"}),
        );

        run_info = run_info.complete_with_output(serde_json::json!({"output": "result"}));
        assert!(run_info.is_completed());
        assert!(!run_info.is_failed());
        assert!(run_info.is_successful());
        assert!(run_info.output.is_some());
        assert!(run_info.end_time.is_some());
        assert!(run_info.duration.is_some());
    }

    #[test]
    fn test_run_info_error() {
        let run_id = RunId::new();
        let mut run_info = RunInfo::new(
            run_id,
            "test_component",
            "test_type",
            serde_json::json!({"input": "test"}),
        );

        run_info = run_info.complete_with_error("Test error");
        assert!(run_info.is_completed());
        assert!(run_info.is_failed());
        assert!(!run_info.is_successful());
        assert!(run_info.error.is_some());
    }

    #[tokio::test]
    async fn test_console_callback_handler() {
        let handler = ConsoleCallbackHandler::new();
        let run_info = RunInfo::new(
            RunId::new(),
            "test",
            "test_type",
            serde_json::json!({"input": "test"}),
        );

        // These should not panic
        handler.on_run_start(&run_info).await.unwrap();
        handler.on_run_success(&run_info).await.unwrap();
        handler.on_run_error(&run_info).await.unwrap();
        handler
            .on_run_stream(&run_info, &serde_json::json!("chunk"))
            .await
            .unwrap();
        handler.on_run_cancel(&run_info).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_callback_handler() {
        let handler = MemoryCallbackHandler::new();
        let run_info = RunInfo::new(
            RunId::new(),
            "test",
            "test_type",
            serde_json::json!({"input": "test"}),
        );

        handler.on_run_start(&run_info).await.unwrap();
        assert_eq!(handler.len().await, 1);

        let runs = handler.get_runs().await;
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].name, "test");
    }

    #[tokio::test]
    async fn test_callback_manager() {
        let mut manager = CallbackManager::new();
        let console_handler = Arc::new(ConsoleCallbackHandler::new());
        let memory_handler = Arc::new(MemoryCallbackHandler::new());

        manager.add_handler(console_handler);
        manager.add_handler(memory_handler);

        assert_eq!(manager.len(), 2);
        assert!(!manager.is_empty());

        let run_info = RunInfo::new(
            RunId::new(),
            "test",
            "test_type",
            serde_json::json!({"input": "test"}),
        );

        manager.on_run_start(&run_info).await.unwrap();
        manager.on_run_success(&run_info).await.unwrap();
    }

    #[test]
    fn test_serialization() {
        let run_id = RunId::new();
        let run_info = RunInfo::new(
            run_id,
            "test",
            "test_type",
            serde_json::json!({"input": "test"}),
        );

        let json = run_info.to_json().unwrap();
        let deserialized: RunInfo = RunInfo::from_json(&json).unwrap();
        assert_eq!(run_info.name, deserialized.name);
        assert_eq!(run_info.component_type, deserialized.component_type);
    }
}
