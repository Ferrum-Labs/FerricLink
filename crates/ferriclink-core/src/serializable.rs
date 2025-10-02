//! Serializable trait for FerricLink objects

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::{FerricLinkError, Result};

/// Trait for objects that can be serialized and deserialized
///
/// This trait provides a standardized way to serialize FerricLink objects
/// to and from various formats, similar to LangChain's Serializable interface.
pub trait Serializable: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static {
    /// Get the namespace for this serializable object
    ///
    /// The namespace is used to identify the type of object when deserializing.
    /// It should be a hierarchical path like ["ferriclink", "messages", "human"].
    fn namespace() -> Vec<String>;

    /// Check if this object is serializable
    ///
    /// By default, all objects implementing this trait are serializable.
    /// Override this method to provide custom serialization logic.
    fn is_serializable() -> bool {
        true
    }

    /// Serialize this object to JSON
    fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(FerricLinkError::from)
    }

    /// Serialize this object to a pretty-printed JSON string
    fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(FerricLinkError::from)
    }

    /// Deserialize this object from JSON
    fn from_json(json: &str) -> Result<Self>
    where
        Self: Sized,
    {
        serde_json::from_str(json).map_err(FerricLinkError::from)
    }

    /// Serialize this object to a dictionary (HashMap)
    fn to_dict(&self) -> Result<HashMap<String, serde_json::Value>> {
        let json = self.to_json()?;
        serde_json::from_str(&json).map_err(FerricLinkError::from)
    }

    /// Deserialize this object from a dictionary (HashMap)
    fn from_dict(dict: &HashMap<String, serde_json::Value>) -> Result<Self>
    where
        Self: Sized,
    {
        let json = serde_json::to_string(dict)?;
        Self::from_json(&json)
    }

    /// Get the type name for this serializable object
    ///
    /// This is used for type identification during serialization/deserialization.
    fn type_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Helper macro to implement Serializable for a type
///
/// This macro provides a default implementation of the Serializable trait
/// for types that derive Serialize and Deserialize.
#[macro_export]
macro_rules! impl_serializable {
    ($type:ty, $namespace:expr) => {
        impl $crate::serializable::Serializable for $type {
            fn namespace() -> Vec<String> {
                $namespace.into_iter().map(|s| s.to_string()).collect()
            }
        }
    };
}

/// Trait for objects that can be loaded from serialized data
pub trait Loadable: Serializable {
    /// Load this object from serialized data
    fn load(data: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        let json = String::from_utf8(data.to_vec())
            .map_err(|e| FerricLinkError::generic(format!("Invalid UTF-8: {e}")))?;
        Self::from_json(&json)
    }

    /// Save this object to serialized data
    fn save(&self) -> Result<Vec<u8>> {
        let json = self.to_json()?;
        Ok(json.into_bytes())
    }
}

/// Helper macro to implement Loadable for a type
#[macro_export]
macro_rules! impl_loadable {
    ($type:ty) => {
        impl $crate::serializable::Loadable for $type {}
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestMessage {
        content: String,
        message_type: String,
    }

    impl_serializable!(TestMessage, ["ferriclink", "messages", "test"]);
    impl_loadable!(TestMessage);

    #[test]
    fn test_serialization() {
        let msg = TestMessage {
            content: "Hello, world!".to_string(),
            message_type: "test".to_string(),
        };

        // Test JSON serialization
        let json = msg.to_json().unwrap();
        let deserialized: TestMessage = TestMessage::from_json(&json).unwrap();
        assert_eq!(msg, deserialized);

        // Test pretty JSON
        let pretty_json = msg.to_json_pretty().unwrap();
        assert!(pretty_json.contains("Hello, world!"));

        // Test dictionary serialization
        let dict = msg.to_dict().unwrap();
        let from_dict: TestMessage = TestMessage::from_dict(&dict).unwrap();
        assert_eq!(msg, from_dict);
    }

    #[test]
    fn test_namespace() {
        let namespace = TestMessage::namespace();
        assert_eq!(namespace, vec!["ferriclink", "messages", "test"]);
    }

    #[test]
    fn test_type_name() {
        let type_name = TestMessage::type_name();
        assert!(type_name.contains("TestMessage"));
    }

    #[test]
    fn test_loadable() {
        let msg = TestMessage {
            content: "Test content".to_string(),
            message_type: "test".to_string(),
        };

        let data = msg.save().unwrap();
        let loaded: TestMessage = TestMessage::load(&data).unwrap();
        assert_eq!(msg, loaded);
    }
}
