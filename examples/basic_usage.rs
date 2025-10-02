//! Basic usage example for FerricLink Core
//!
//! This example demonstrates the core functionality of FerricLink,
//! including messages, language models, and runnables.

use ferriclink_core::{
    messages::AnyMessage,
    language_models::{mock_chat_model, GenerationConfig},
    runnables::Runnable,
    callbacks::console_callback_handler,
    tools::function_tool,
    documents::Document,
    vectorstores::InMemoryVectorStore,
    retrievers::VectorStoreRetriever,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize FerricLink
    ferriclink_core::init()?;
    
    println!("🚀 FerricLink Core Basic Usage Example");
    println!("=====================================");
    
    // 1. Create a chat model
    println!("\n1. Creating a chat model...");
    let chat_model = mock_chat_model("gpt-3.5-turbo");
    
    // 2. Create a conversation
    println!("2. Creating a conversation...");
    let messages = vec![
        AnyMessage::human("Hello, how are you?"),
    ];
    
    // 3. Generate a response
    println!("3. Generating a response...");
    let response = chat_model.generate_chat(
        messages,
        Some(GenerationConfig::new().with_temperature(0.7)),
        None,
    ).await?;
    
    println!("   Response: {}", response.text());
    
    // 4. Create a simple tool
    println!("\n4. Creating a tool...");
    let add_tool = function_tool(
        "add",
        "Add two numbers",
        |args| {
            let a = args.get("a")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let b = args.get("b")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            Ok((a + b).to_string())
        },
    );
    
    println!("   Tool: {}", add_tool.name());
    println!("   Description: {}", add_tool.description());
    
    // 5. Use the tool
    println!("5. Using the tool...");
    use std::collections::HashMap;
    let mut args = HashMap::new();
    args.insert("a".to_string(), serde_json::Value::Number(serde_json::Number::from(5)));
    args.insert("b".to_string(), serde_json::Value::Number(serde_json::Number::from(3)));
    
    let result = add_tool.invoke(args, None).await?;
    println!("   5 + 3 = {}", result.content);
    
    // 6. Create a vector store and retriever
    println!("\n6. Creating a vector store...");
    let vector_store = Box::new(InMemoryVectorStore::new());
    
    // Add some documents
    let docs = vec![
        Document::new("Rust is a systems programming language"),
        Document::new("FerricLink is a Rust library for AI applications"),
        Document::new("LangChain is a Python library for AI applications"),
    ];
    
    vector_store.add_documents(docs, None).await?;
    println!("   Added {} documents to vector store", vector_store.len().await?);
    
    // 7. Create a retriever
    println!("7. Creating a retriever...");
    let retriever = VectorStoreRetriever::new(vector_store);
    
    // Search for relevant documents
    let search_result = retriever.get_relevant_documents("Rust programming", None).await?;
    println!("   Found {} relevant documents:", search_result.len());
    
    for (i, doc) in search_result.documents.iter().enumerate() {
        println!("     {}. {}", i + 1, doc.page_content);
    }
    
    // 8. Use callbacks
    println!("\n8. Using callbacks...");
    let callback_handler = console_callback_handler();
    let run_id = "example-run";
    let input = serde_json::json!({"message": "Hello from callback!"});
    
    callback_handler.on_start(run_id, &input).await?;
    callback_handler.on_success(run_id, &serde_json::json!({"result": "success"})).await?;
    
    println!("\n✅ Example completed successfully!");
    Ok(())
}
