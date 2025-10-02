//! Tool abstractions for FerricLink Core
//!
//! This module provides the core abstractions for tools that can be used
//! by language models and other components.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::Result;
use crate::runnables::{Runnable, RunnableConfig};
use crate::impl_serializable;

/// A tool call made by a language model
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,
    /// Name of the tool being called
    pub name: String,
    /// Arguments passed to the tool
    pub args: HashMap<String, serde_json::Value>,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            args: HashMap::new(),
        }
    }

    /// Create a new tool call with arguments
    pub fn new_with_args(
        id: impl Into<String>,
        name: impl Into<String>,
        args: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            args,
        }
    }

    /// Add an argument to the tool call
    pub fn add_arg(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.args.insert(key.into(), value);
    }

    /// Get an argument value
    pub fn get_arg(&self, key: &str) -> Option<&serde_json::Value> {
        self.args.get(key)
    }
}

impl_serializable!(ToolCall, ["ferriclink", "tools", "tool_call"]);

/// A tool result returned by a tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    /// The tool call ID this result corresponds to
    pub tool_call_id: String,
    /// The result content
    pub content: String,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ToolResult {
    /// Create a new tool result
    pub fn new(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new tool result with metadata
    pub fn new_with_metadata(
        tool_call_id: impl Into<String>,
        content: impl Into<String>,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            metadata,
        }
    }

    /// Add metadata to the result
    pub fn add_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metadata.insert(key.into(), value);
    }
}

impl_serializable!(ToolResult, ["ferriclink", "tools", "tool_result"]);

/// Schema for a tool's input parameters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolSchema {
    /// Name of the tool
    pub name: String,
    /// Description of the tool
    pub description: String,
    /// JSON schema for the input parameters
    pub input_schema: serde_json::Value,
}

impl ToolSchema {
    /// Create a new tool schema
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    /// Create a new tool schema with input schema
    pub fn new_with_schema(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

impl_serializable!(ToolSchema, ["ferriclink", "tools", "tool_schema"]);

/// Base trait for all tools
#[async_trait]
pub trait BaseTool: Send + Sync + 'static {
    /// Get the name of the tool
    fn name(&self) -> &str;

    /// Get the description of the tool
    fn description(&self) -> &str;

    /// Get the schema for this tool
    fn schema(&self) -> ToolSchema;

    /// Check if the tool is available
    fn is_available(&self) -> bool {
        true
    }

    /// Get the input schema for this tool
    fn input_schema(&self) -> Option<serde_json::Value> {
        Some(self.schema().input_schema.clone())
    }

    /// Get the output schema for this tool
    fn output_schema(&self) -> Option<serde_json::Value> {
        None
    }
}

/// Trait for tools that can be invoked with a single input
#[async_trait]
pub trait Tool: BaseTool {
    /// Invoke the tool with the given input
    async fn invoke(
        &self,
        input: HashMap<String, serde_json::Value>,
        config: Option<RunnableConfig>,
    ) -> Result<ToolResult>;
}

/// A simple tool that wraps a function
pub struct FunctionTool<F> {
    name: String,
    description: String,
    schema: ToolSchema,
    func: F,
}

impl<F> FunctionTool<F>
where
    F: Fn(HashMap<String, serde_json::Value>) -> Result<String> + Send + Sync + 'static,
{
    /// Create a new function tool
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        func: F,
    ) -> Self {
        let name = name.into();
        let description = description.into();
        let schema = ToolSchema::new(&name, &description);
        
        Self {
            name,
            description,
            schema,
            func,
        }
    }

    /// Create a new function tool with custom schema
    pub fn new_with_schema(
        name: impl Into<String>,
        description: impl Into<String>,
        schema: ToolSchema,
        func: F,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            schema,
            func,
        }
    }
}

#[async_trait]
impl<F> BaseTool for FunctionTool<F>
where
    F: Fn(HashMap<String, serde_json::Value>) -> Result<String> + Send + Sync + 'static,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn schema(&self) -> ToolSchema {
        self.schema.clone()
    }
}

#[async_trait]
impl<F> Tool for FunctionTool<F>
where
    F: Fn(HashMap<String, serde_json::Value>) -> Result<String> + Send + Sync + 'static,
{
    async fn invoke(
        &self,
        input: HashMap<String, serde_json::Value>,
        _config: Option<RunnableConfig>,
    ) -> Result<ToolResult> {
        let content = (self.func)(input)?;
        Ok(ToolResult::new("", content))
    }
}

/// A tool that can be used as a runnable
pub struct RunnableTool<T> {
    tool: T,
    tool_call_id: String,
}

impl<T> RunnableTool<T>
where
    T: Tool,
{
    /// Create a new runnable tool
    pub fn new(tool: T, tool_call_id: impl Into<String>) -> Self {
        Self {
            tool,
            tool_call_id: tool_call_id.into(),
        }
    }
}

#[async_trait]
impl<T> Runnable<HashMap<String, serde_json::Value>, ToolResult> for RunnableTool<T>
where
    T: Tool,
{
    async fn invoke(
        &self,
        input: HashMap<String, serde_json::Value>,
        config: Option<RunnableConfig>,
    ) -> Result<ToolResult> {
        let mut result = self.tool.invoke(input, config).await?;
        result.tool_call_id = self.tool_call_id.clone();
        Ok(result)
    }
}

/// A collection of tools
pub struct ToolCollection {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolCollection {
    /// Create a new empty tool collection
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Add a tool to the collection
    pub fn add_tool<T>(&mut self, tool: T)
    where
        T: Tool + 'static,
    {
        let name = tool.name().to_string();
        self.tools.insert(name, Box::new(tool));
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Get all tool names
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get all tools
    pub fn tools(&self) -> &HashMap<String, Box<dyn Tool>> {
        &self.tools
    }

    /// Get the number of tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Invoke a tool by name
    pub async fn invoke_tool(
        &self,
        name: &str,
        input: HashMap<String, serde_json::Value>,
        config: Option<RunnableConfig>,
    ) -> Result<ToolResult> {
        let tool = self.get_tool(name)
            .ok_or_else(|| crate::errors::FerricLinkError::generic(format!("Tool '{}' not found", name)))?;
        
        tool.invoke(input, config).await
    }
}

impl Default for ToolCollection {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a simple function tool
pub fn function_tool<F>(
    name: impl Into<String>,
    description: impl Into<String>,
    func: F,
) -> FunctionTool<F>
where
    F: Fn(HashMap<String, serde_json::Value>) -> Result<String> + Send + Sync + 'static,
{
    FunctionTool::new(name, description, func)
}

/// Helper function to create a tool with custom schema
pub fn function_tool_with_schema<F>(
    name: impl Into<String>,
    description: impl Into<String>,
    schema: ToolSchema,
    func: F,
) -> FunctionTool<F>
where
    F: Fn(HashMap<String, serde_json::Value>) -> Result<String> + Send + Sync + 'static,
{
    FunctionTool::new_with_schema(name, description, schema, func)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializable::Serializable;

    #[test]
    fn test_tool_call() {
        let mut call = ToolCall::new("call_123", "test_tool");
        call.add_arg("param1", serde_json::Value::String("value1".to_string()));
        
        assert_eq!(call.id, "call_123");
        assert_eq!(call.name, "test_tool");
        assert_eq!(call.get_arg("param1"), Some(&serde_json::Value::String("value1".to_string())));
    }

    #[test]
    fn test_tool_result() {
        let mut result = ToolResult::new("call_123", "Tool executed successfully");
        result.add_metadata("execution_time", serde_json::Value::Number(serde_json::Number::from(100)));
        
        assert_eq!(result.tool_call_id, "call_123");
        assert_eq!(result.content, "Tool executed successfully");
        assert_eq!(result.metadata.get("execution_time"), Some(&serde_json::Value::Number(serde_json::Number::from(100))));
    }

    #[test]
    fn test_tool_schema() {
        let schema = ToolSchema::new("test_tool", "A test tool");
        
        assert_eq!(schema.name, "test_tool");
        assert_eq!(schema.description, "A test tool");
        assert!(schema.input_schema.is_object());
    }

    #[tokio::test]
    async fn test_function_tool() {
        let tool = function_tool(
            "add",
            "Add two numbers",
            |args| {
                let a = args.get("a")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| crate::errors::FerricLinkError::validation("Missing or invalid 'a' parameter"))?;
                let b = args.get("b")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| crate::errors::FerricLinkError::validation("Missing or invalid 'b' parameter"))?;
                Ok((a + b).to_string())
            },
        );
        
        assert_eq!(tool.name(), "add");
        assert_eq!(tool.description(), "Add two numbers");
        
        let mut args = HashMap::new();
        args.insert("a".to_string(), serde_json::Value::Number(serde_json::Number::from(5)));
        args.insert("b".to_string(), serde_json::Value::Number(serde_json::Number::from(3)));
        
        let result = tool.invoke(args, None).await.unwrap();
        assert_eq!(result.content, "8");
    }

    #[tokio::test]
    async fn test_tool_collection() {
        let mut collection = ToolCollection::new();
        
        let add_tool = function_tool("add", "Add two numbers", |args| {
            let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Ok((a + b).to_string())
        });
        
        let multiply_tool = function_tool("multiply", "Multiply two numbers", |args| {
            let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(1.0);
            let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(1.0);
            Ok((a * b).to_string())
        });
        
        collection.add_tool(add_tool);
        collection.add_tool(multiply_tool);
        
        assert_eq!(collection.len(), 2);
        assert!(!collection.is_empty());
        assert!(collection.tool_names().contains(&"add"));
        assert!(collection.tool_names().contains(&"multiply"));
        
        let mut args = HashMap::new();
        args.insert("a".to_string(), serde_json::Value::Number(serde_json::Number::from(4)));
        args.insert("b".to_string(), serde_json::Value::Number(serde_json::Number::from(5)));
        
        let result = collection.invoke_tool("add", args, None).await.unwrap();
        assert_eq!(result.content, "9");
    }

    #[tokio::test]
    async fn test_runnable_tool() {
        let tool = function_tool("test", "Test tool", |_| Ok("test result".to_string()));
        let runnable_tool = RunnableTool::new(tool, "call_123");
        
        let args = HashMap::new();
        let result = runnable_tool.invoke(args, None).await.unwrap();
        
        assert_eq!(result.tool_call_id, "call_123");
        assert_eq!(result.content, "test result");
    }

    #[test]
    fn test_serialization() {
        let call = ToolCall::new("call_123", "test_tool");
        let json = call.to_json().unwrap();
        let deserialized: ToolCall = ToolCall::from_json(&json).unwrap();
        assert_eq!(call, deserialized);
    }
}
