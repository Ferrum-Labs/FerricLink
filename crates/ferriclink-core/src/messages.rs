//! Message types for FerricLink Core
//!
//! This module provides the core message abstractions used throughout the FerricLink
//! ecosystem, similar to LangChain's message system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::impl_serializable;

/// Content of a message, which can be either text or a list of content blocks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    Text(String),
    /// Complex content with multiple blocks
    Blocks(Vec<ContentBlock>),
}

/// A content block within a message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content block
    Text {
        text: String,
    },
    /// Image content block
    Image {
        image_url: String,
        alt_text: Option<String>,
    },
    /// JSON content block
    Json {
        data: serde_json::Value,
    },
    /// Tool call content block
    ToolCall {
        id: String,
        name: String,
        args: HashMap<String, serde_json::Value>,
    },
    /// Tool result content block
    ToolResult {
        tool_call_id: String,
        content: String,
    },
}

/// Base message trait that all message types implement
pub trait BaseMessage: Send + Sync {
    /// Get the content of the message
    fn content(&self) -> &MessageContent;

    /// Get the message type
    fn message_type(&self) -> &str;

    /// Get additional kwargs
    fn additional_kwargs(&self) -> &HashMap<String, serde_json::Value>;

    /// Get response metadata
    fn response_metadata(&self) -> &HashMap<String, serde_json::Value>;

    /// Get the message name
    fn name(&self) -> Option<&str>;

    /// Get the message ID
    fn id(&self) -> Option<&str>;

    /// Convert the message content to text
    fn text(&self) -> String {
        match self.content() {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Blocks(blocks) => {
                blocks
                    .iter()
                    .filter_map(|block| match block {
                        ContentBlock::Text { text } => Some(text.clone()),
                        ContentBlock::ToolResult { content, .. } => Some(content.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("")
            }
        }
    }

    /// Check if this is a human message
    fn is_human(&self) -> bool {
        self.message_type() == "human"
    }

    /// Check if this is an AI message
    fn is_ai(&self) -> bool {
        self.message_type() == "ai"
    }

    /// Check if this is a system message
    fn is_system(&self) -> bool {
        self.message_type() == "system"
    }

    /// Check if this is a tool message
    fn is_tool(&self) -> bool {
        self.message_type() == "tool"
    }
}

/// Human message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HumanMessage {
    /// The content of the message
    pub content: MessageContent,
    /// Additional keyword arguments
    #[serde(default)]
    pub additional_kwargs: HashMap<String, serde_json::Value>,
    /// Response metadata
    #[serde(default)]
    pub response_metadata: HashMap<String, serde_json::Value>,
    /// Optional name for the message
    pub name: Option<String>,
    /// Optional unique identifier
    pub id: Option<String>,
}

impl HumanMessage {
    /// Create a new human message with text content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: MessageContent::Text(content.into()),
            additional_kwargs: HashMap::new(),
            response_metadata: HashMap::new(),
            name: None,
            id: Some(Uuid::new_v4().to_string()),
        }
    }

    /// Create a new human message with content blocks
    pub fn new_with_blocks(content: Vec<ContentBlock>) -> Self {
        Self {
            content: MessageContent::Blocks(content),
            additional_kwargs: HashMap::new(),
            response_metadata: HashMap::new(),
            name: None,
            id: Some(Uuid::new_v4().to_string()),
        }
    }
}

impl BaseMessage for HumanMessage {
    fn content(&self) -> &MessageContent {
        &self.content
    }

    fn message_type(&self) -> &str {
        "human"
    }

    fn additional_kwargs(&self) -> &HashMap<String, serde_json::Value> {
        &self.additional_kwargs
    }

    fn response_metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.response_metadata
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

impl_serializable!(HumanMessage, ["ferriclink", "messages", "human"]);

/// AI message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AIMessage {
    /// The content of the message
    pub content: MessageContent,
    /// Additional keyword arguments
    #[serde(default)]
    pub additional_kwargs: HashMap<String, serde_json::Value>,
    /// Response metadata
    #[serde(default)]
    pub response_metadata: HashMap<String, serde_json::Value>,
    /// Optional name for the message
    pub name: Option<String>,
    /// Optional unique identifier
    pub id: Option<String>,
}

impl AIMessage {
    /// Create a new AI message with text content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: MessageContent::Text(content.into()),
            additional_kwargs: HashMap::new(),
            response_metadata: HashMap::new(),
            name: None,
            id: Some(Uuid::new_v4().to_string()),
        }
    }

    /// Create a new AI message with content blocks
    pub fn new_with_blocks(content: Vec<ContentBlock>) -> Self {
        Self {
            content: MessageContent::Blocks(content),
            additional_kwargs: HashMap::new(),
            response_metadata: HashMap::new(),
            name: None,
            id: Some(Uuid::new_v4().to_string()),
        }
    }
}

impl BaseMessage for AIMessage {
    fn content(&self) -> &MessageContent {
        &self.content
    }

    fn message_type(&self) -> &str {
        "ai"
    }

    fn additional_kwargs(&self) -> &HashMap<String, serde_json::Value> {
        &self.additional_kwargs
    }

    fn response_metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.response_metadata
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

impl_serializable!(AIMessage, ["ferriclink", "messages", "ai"]);

/// System message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemMessage {
    /// The content of the message
    pub content: MessageContent,
    /// Additional keyword arguments
    #[serde(default)]
    pub additional_kwargs: HashMap<String, serde_json::Value>,
    /// Response metadata
    #[serde(default)]
    pub response_metadata: HashMap<String, serde_json::Value>,
    /// Optional name for the message
    pub name: Option<String>,
    /// Optional unique identifier
    pub id: Option<String>,
}

impl SystemMessage {
    /// Create a new system message with text content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: MessageContent::Text(content.into()),
            additional_kwargs: HashMap::new(),
            response_metadata: HashMap::new(),
            name: None,
            id: Some(Uuid::new_v4().to_string()),
        }
    }
}

impl BaseMessage for SystemMessage {
    fn content(&self) -> &MessageContent {
        &self.content
    }

    fn message_type(&self) -> &str {
        "system"
    }

    fn additional_kwargs(&self) -> &HashMap<String, serde_json::Value> {
        &self.additional_kwargs
    }

    fn response_metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.response_metadata
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

impl_serializable!(SystemMessage, ["ferriclink", "messages", "system"]);

/// Tool message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolMessage {
    /// The content of the message
    pub content: MessageContent,
    /// The tool call ID this message is responding to
    pub tool_call_id: String,
    /// Additional keyword arguments
    #[serde(default)]
    pub additional_kwargs: HashMap<String, serde_json::Value>,
    /// Response metadata
    #[serde(default)]
    pub response_metadata: HashMap<String, serde_json::Value>,
    /// Optional name for the message
    pub name: Option<String>,
    /// Optional unique identifier
    pub id: Option<String>,
}

impl ToolMessage {
    /// Create a new tool message
    pub fn new(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            content: MessageContent::Text(content.into()),
            tool_call_id: tool_call_id.into(),
            additional_kwargs: HashMap::new(),
            response_metadata: HashMap::new(),
            name: None,
            id: Some(Uuid::new_v4().to_string()),
        }
    }
}

impl BaseMessage for ToolMessage {
    fn content(&self) -> &MessageContent {
        &self.content
    }

    fn message_type(&self) -> &str {
        "tool"
    }

    fn additional_kwargs(&self) -> &HashMap<String, serde_json::Value> {
        &self.additional_kwargs
    }

    fn response_metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.response_metadata
    }

    fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
}

impl_serializable!(ToolMessage, ["ferriclink", "messages", "tool"]);

/// Union type for all message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content")]
pub enum AnyMessage {
    /// Human message
    Human(HumanMessage),
    /// AI message
    AI(AIMessage),
    /// System message
    System(SystemMessage),
    /// Tool message
    Tool(ToolMessage),
}

impl AnyMessage {
    /// Create a human message
    pub fn human(content: impl Into<String>) -> Self {
        Self::Human(HumanMessage::new(content))
    }

    /// Create an AI message
    pub fn ai(content: impl Into<String>) -> Self {
        Self::AI(AIMessage::new(content))
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::System(SystemMessage::new(content))
    }

    /// Create a tool message
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self::Tool(ToolMessage::new(content, tool_call_id))
    }
}

impl BaseMessage for AnyMessage {
    fn content(&self) -> &MessageContent {
        match self {
            AnyMessage::Human(msg) => msg.content(),
            AnyMessage::AI(msg) => msg.content(),
            AnyMessage::System(msg) => msg.content(),
            AnyMessage::Tool(msg) => msg.content(),
        }
    }

    fn message_type(&self) -> &str {
        match self {
            AnyMessage::Human(msg) => msg.message_type(),
            AnyMessage::AI(msg) => msg.message_type(),
            AnyMessage::System(msg) => msg.message_type(),
            AnyMessage::Tool(msg) => msg.message_type(),
        }
    }

    fn additional_kwargs(&self) -> &HashMap<String, serde_json::Value> {
        match self {
            AnyMessage::Human(msg) => msg.additional_kwargs(),
            AnyMessage::AI(msg) => msg.additional_kwargs(),
            AnyMessage::System(msg) => msg.additional_kwargs(),
            AnyMessage::Tool(msg) => msg.additional_kwargs(),
        }
    }

    fn response_metadata(&self) -> &HashMap<String, serde_json::Value> {
        match self {
            AnyMessage::Human(msg) => msg.response_metadata(),
            AnyMessage::AI(msg) => msg.response_metadata(),
            AnyMessage::System(msg) => msg.response_metadata(),
            AnyMessage::Tool(msg) => msg.response_metadata(),
        }
    }

    fn name(&self) -> Option<&str> {
        match self {
            AnyMessage::Human(msg) => msg.name(),
            AnyMessage::AI(msg) => msg.name(),
            AnyMessage::System(msg) => msg.name(),
            AnyMessage::Tool(msg) => msg.name(),
        }
    }

    fn id(&self) -> Option<&str> {
        match self {
            AnyMessage::Human(msg) => msg.id(),
            AnyMessage::AI(msg) => msg.id(),
            AnyMessage::System(msg) => msg.id(),
            AnyMessage::Tool(msg) => msg.id(),
        }
    }
}

impl_serializable!(AnyMessage, ["ferriclink", "messages", "any"]);

/// Helper function to convert messages to a string representation
pub fn get_buffer_string(messages: &[AnyMessage], human_prefix: &str, ai_prefix: &str) -> String {
    let mut buffer = String::new();
    
    for message in messages {
        match message {
            AnyMessage::Human(msg) => {
                buffer.push_str(human_prefix);
                buffer.push_str(": ");
                buffer.push_str(&msg.text());
                buffer.push('\n');
            }
            AnyMessage::AI(msg) => {
                buffer.push_str(ai_prefix);
                buffer.push_str(": ");
                buffer.push_str(&msg.text());
                buffer.push('\n');
            }
            AnyMessage::System(msg) => {
                buffer.push_str("System: ");
                buffer.push_str(&msg.text());
                buffer.push('\n');
            }
            AnyMessage::Tool(msg) => {
                buffer.push_str("Tool: ");
                buffer.push_str(&msg.text());
                buffer.push('\n');
            }
        }
    }
    
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serializable::Serializable;

    #[test]
    fn test_human_message() {
        let msg = HumanMessage::new("Hello, world!");
        assert_eq!(msg.text(), "Hello, world!");
        assert!(msg.is_human());
        assert!(!msg.is_ai());
        assert!(!msg.is_system());
        assert!(!msg.is_tool());
    }

    #[test]
    fn test_ai_message() {
        let msg = AIMessage::new("Hello! How can I help you?");
        assert_eq!(msg.text(), "Hello! How can I help you?");
        assert!(!msg.is_human());
        assert!(msg.is_ai());
        assert!(!msg.is_system());
        assert!(!msg.is_tool());
    }

    #[test]
    fn test_system_message() {
        let msg = SystemMessage::new("You are a helpful assistant.");
        assert_eq!(msg.text(), "You are a helpful assistant.");
        assert!(!msg.is_human());
        assert!(!msg.is_ai());
        assert!(msg.is_system());
        assert!(!msg.is_tool());
    }

    #[test]
    fn test_tool_message() {
        let msg = ToolMessage::new("Tool result", "call_123");
        assert_eq!(msg.text(), "Tool result");
        assert!(!msg.is_human());
        assert!(!msg.is_ai());
        assert!(!msg.is_system());
        assert!(msg.is_tool());
    }

    #[test]
    fn test_any_message() {
        let human = AnyMessage::human("Hello");
        let ai = AnyMessage::ai("Hi there!");
        
        assert!(human.is_human());
        assert!(ai.is_ai());
    }

    #[test]
    fn test_message_content_blocks() {
        let blocks = vec![
            ContentBlock::Text { text: "Hello".to_string() },
            ContentBlock::Image { 
                image_url: "https://example.com/image.jpg".to_string(),
                alt_text: Some("An image".to_string()),
            },
        ];
        
        let msg = HumanMessage::new_with_blocks(blocks);
        assert_eq!(msg.text(), "Hello");
    }

    #[test]
    fn test_get_buffer_string() {
        let messages = vec![
            AnyMessage::human("Hello"),
            AnyMessage::ai("Hi there!"),
        ];
        
        let buffer = get_buffer_string(&messages, "Human", "Assistant");
        assert!(buffer.contains("Human: Hello"));
        assert!(buffer.contains("Assistant: Hi there!"));
    }

    #[test]
    fn test_serialization() {
        let msg = HumanMessage::new("Test message");
        let json = msg.to_json().unwrap();
        let deserialized: HumanMessage = HumanMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);
    }
}
