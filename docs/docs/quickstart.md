# Quickstart

```rust
use ferriclink_core::{
    messages::AnyMessage,
    language_models::{mock_chat_model, GenerationConfig},
    runnables::Runnable,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a mock chat model
    let chat_model = mock_chat_model("gpt-4o-mini");
    
    // Create a conversation
    let messages = vec![
        AnyMessage::human("Hello, how are you?"),
    ];
    
    // Generate a response
    let response = chat_model.generate_chat(
        messages,
        Some(GenerationConfig::new().with_temperature(0.7)),
        None,
    ).await?;
    
    println!("Response: {}", response.text());
    Ok(())
}
```