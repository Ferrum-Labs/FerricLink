//! Example demonstrating FerricLink's example selector functionality.
//!
//! This example shows how to use different example selectors for few-shot learning
//! and dynamic prompt construction, similar to LangChain's example selectors.

use ferriclink_core::{
    example_selectors::{
        BaseExampleSelector, LengthBasedExampleSelector, SemanticSimilarityExampleSelector,
        MaxMarginalRelevanceExampleSelector, Example, sorted_values,
    },
    vectorstores::InMemoryVectorStore,
    embeddings::MockEmbeddings,
};
use std::collections::HashMap;

/// Create sample examples for demonstration
fn create_sample_examples() -> Vec<Example> {
    vec![
        [("input".to_string(), "What is machine learning?".to_string()), 
         ("output".to_string(), "Machine learning is a subset of AI that enables computers to learn without being explicitly programmed.".to_string())]
            .iter()
            .cloned()
            .collect(),
        [("input".to_string(), "How does neural networks work?".to_string()),
         ("output".to_string(), "Neural networks are computing systems inspired by biological neural networks that process information through interconnected nodes.".to_string())]
            .iter()
            .cloned()
            .collect(),
        [("input".to_string(), "What is deep learning?".to_string()),
         ("output".to_string(), "Deep learning is a subset of machine learning that uses neural networks with multiple layers to model and understand complex patterns.".to_string())]
            .iter()
            .cloned()
            .collect(),
        [("input".to_string(), "Explain artificial intelligence".to_string()),
         ("output".to_string(), "Artificial intelligence is the simulation of human intelligence in machines that are programmed to think and learn like humans.".to_string())]
            .iter()
            .cloned()
            .collect(),
        [("input".to_string(), "What is natural language processing?".to_string()),
         ("output".to_string(), "Natural language processing is a field of AI that focuses on the interaction between computers and human language.".to_string())]
            .iter()
            .cloned()
            .collect(),
    ]
}

/// Demonstrate length-based example selection
async fn length_based_example() {
    println!("=== Length-Based Example Selector ===\n");
    
    let examples = create_sample_examples();
    let mut selector = LengthBasedExampleSelector::with_word_count(examples, 50);
    
    println!("Initial examples count: {}", selector.len());
    println!("Total length: {} words", selector.total_length());
    
    // Test input
    let input = [("input".to_string(), "Tell me about AI and ML".to_string())]
        .iter()
        .cloned()
        .collect();
    
    // Select examples based on length
    let selected = selector.select_examples(&input).unwrap();
    println!("\nSelected {} examples for input:", selected.len());
    
    for (i, example) in selected.iter().enumerate() {
        println!("  {}. Input: {}", i + 1, example.get("input").unwrap_or(&"N/A".to_string()));
        println!("     Output: {}", example.get("output").unwrap_or(&"N/A".to_string()));
    }
    
    // Add a new example
    let new_example = [("input".to_string(), "What is computer vision?".to_string()),
                      ("output".to_string(), "Computer vision is a field of AI that trains computers to interpret and understand visual information.".to_string())]
        .iter()
        .cloned()
        .collect();
    
    selector.add_example(new_example).unwrap();
    println!("\nAfter adding new example:");
    println!("  Examples count: {}", selector.len());
    println!("  Total length: {} words", selector.total_length());
    
    // Test with different length limits
    println!("\n--- Testing with different length limits ---");
    let short_selector = LengthBasedExampleSelector::with_word_count(create_sample_examples(), 20);
    let short_selected = short_selector.select_examples(&input).unwrap();
    println!("With 20 word limit: {} examples selected", short_selected.len());
    
    let long_selector = LengthBasedExampleSelector::with_word_count(create_sample_examples(), 100);
    let long_selected = long_selector.select_examples(&input).unwrap();
    println!("With 100 word limit: {} examples selected", long_selected.len());
}

/// Demonstrate semantic similarity example selection
async fn semantic_similarity_example() {
    println!("\n=== Semantic Similarity Example Selector ===\n");
    
    // Create a mock vector store with embeddings
    let embeddings = MockEmbeddings::new("mock-embeddings", 384);
    let vectorstore = InMemoryVectorStore::new_with_embeddings(Box::new(embeddings));
    
    let mut selector = SemanticSimilarityExampleSelector::new(
        Box::new(vectorstore),
        3, // Select top 3 most similar examples
        None, // No example key filtering
        Some(vec!["input".to_string()]), // Only use input for similarity
        None, // No extra vectorstore kwargs
    );
    
    // Add examples to the vector store
    let examples = create_sample_examples();
    for example in examples {
        selector.aadd_example(example).await.unwrap();
    }
    
    println!("Added {} examples to vector store", 5);
    
    // Test with different queries
    let test_queries: Vec<Example> = vec![
        [("input".to_string(), "What is artificial intelligence?".to_string())].iter().cloned().collect(),
        [("input".to_string(), "How do computers learn?".to_string())].iter().cloned().collect(),
        [("input".to_string(), "Explain image recognition".to_string())].iter().cloned().collect(),
    ];
    
    for (i, query) in test_queries.iter().enumerate() {
        println!("\n--- Query {}: {} ---", i + 1, query.get("input").unwrap_or(&"N/A".to_string()));
        
        let selected = selector.aselect_examples(query).await.unwrap();
        println!("Selected {} most similar examples:", selected.len());
        
        for (j, example) in selected.iter().enumerate() {
            println!("  {}. Input: {}", j + 1, example.get("input").unwrap_or(&"N/A".to_string()));
            println!("     Output: {}", example.get("output").unwrap_or(&"N/A".to_string()));
        }
    }
}

/// Demonstrate max marginal relevance example selection
async fn max_marginal_relevance_example() {
    println!("\n=== Max Marginal Relevance Example Selector ===\n");
    
    // Create a mock vector store with embeddings
    let embeddings = MockEmbeddings::new("mock-embeddings", 384);
    let vectorstore = InMemoryVectorStore::new_with_embeddings(Box::new(embeddings));
    
    let mut selector = MaxMarginalRelevanceExampleSelector::new(
        Box::new(vectorstore),
        2, // Select 2 examples
        5, // Fetch 5 for reranking
        None, // No example key filtering
        Some(vec!["input".to_string()]), // Only use input for similarity
        None, // No extra vectorstore kwargs
    );
    
    // Add examples to the vector store
    let examples = create_sample_examples();
    for example in examples {
        selector.aadd_example(example).await.unwrap();
    }
    
    println!("Added {} examples to vector store", 5);
    
    // Test MMR selection
    let query: Example = [("input".to_string(), "What is machine learning and AI?".to_string())]
        .iter()
        .cloned()
        .collect();
    
    println!("Query: {}", query.get("input").unwrap_or(&"N/A".to_string()));
    
    let selected = selector.aselect_examples(&query).await.unwrap();
    println!("\nSelected {} examples using MMR:", selected.len());
    
    for (i, example) in selected.iter().enumerate() {
        println!("  {}. Input: {}", i + 1, example.get("input").unwrap_or(&"N/A".to_string()));
        println!("     Output: {}", example.get("output").unwrap_or(&"N/A".to_string()));
    }
}

/// Demonstrate utility functions
fn utility_functions_example() {
    println!("\n=== Utility Functions ===\n");
    
    let mut example = HashMap::new();
    example.insert("z".to_string(), "last".to_string());
    example.insert("a".to_string(), "first".to_string());
    example.insert("m".to_string(), "middle".to_string());
    
    let sorted = sorted_values(&example);
    println!("Original example: {:?}", example);
    println!("Sorted values: {:?}", sorted);
    
    // Demonstrate with a more complex example
    let mut complex_example = HashMap::new();
    complex_example.insert("output".to_string(), "This is the output".to_string());
    complex_example.insert("input".to_string(), "This is the input".to_string());
    complex_example.insert("context".to_string(), "This is the context".to_string());
    
    let complex_sorted = sorted_values(&complex_example);
    println!("\nComplex example: {:?}", complex_example);
    println!("Sorted values: {:?}", complex_sorted);
}

/// Demonstrate different text length functions
fn text_length_functions_example() {
    println!("\n=== Text Length Functions ===\n");
    
    let examples = create_sample_examples();
    let text = "This is a sample text for length calculation";
    
    // Word count selector
    let word_selector = LengthBasedExampleSelector::with_word_count(examples.clone(), 100);
    let word_count = (word_selector.get_text_length)(text);
    println!("Word count for '{}': {}", text, word_count);
    
    // Character count selector
    let char_selector = LengthBasedExampleSelector::with_char_count(examples, 100);
    let char_count = (char_selector.get_text_length)(text);
    println!("Character count for '{}': {}", text, char_count);
    
    // Custom length function
    let custom_selector = LengthBasedExampleSelector::new(
        create_sample_examples(),
        100,
        Some(|text: &str| text.split(' ').count() * 2), // Custom: double word count
    );
    let custom_count = (custom_selector.get_text_length)(text);
    println!("Custom count (2x words) for '{}': {}", text, custom_count);
}

/// Demonstrate error handling
async fn error_handling_example() {
    println!("\n=== Error Handling ===\n");
    
    // Test sync methods that are not supported for vector-based selectors
    let embeddings = MockEmbeddings::new("mock-embeddings", 384);
    let vectorstore = InMemoryVectorStore::new_with_embeddings(Box::new(embeddings));
    
    let mut semantic_selector = SemanticSimilarityExampleSelector::new(
        Box::new(vectorstore),
        3,
        None,
        None,
        None,
    );
    
    let example: Example = [("input".to_string(), "test".to_string())]
        .iter()
        .cloned()
        .collect();
    
    // This should return an error
    match semantic_selector.add_example(example.clone()) {
        Ok(_) => println!("Unexpected: sync add_example succeeded"),
        Err(e) => println!("Expected error for sync add_example: {}", e),
    }
    
    // This should work
    match semantic_selector.aadd_example(example).await {
        Ok(_) => println!("Async add_example succeeded"),
        Err(e) => println!("Error in async add_example: {}", e),
    }
}

/// Demonstrate performance comparison
async fn performance_comparison() {
    println!("\n=== Performance Comparison ===\n");
    
    let examples = create_sample_examples();
    let input = [("input".to_string(), "AI and machine learning concepts".to_string())]
        .iter()
        .cloned()
        .collect();
    
    // Length-based selector (fast)
    let length_selector = LengthBasedExampleSelector::with_word_count(examples.clone(), 50);
    
    let start = std::time::Instant::now();
    let _length_selected = length_selector.select_examples(&input).unwrap();
    let length_duration = start.elapsed();
    
    println!("Length-based selection: {:?}", length_duration);
    
    // Semantic similarity selector (slower due to embeddings)
    let embeddings = MockEmbeddings::new("mock-embeddings", 384);
    let vectorstore = InMemoryVectorStore::new_with_embeddings(Box::new(embeddings));
    
    let mut semantic_selector = SemanticSimilarityExampleSelector::new(
        Box::new(vectorstore),
        3,
        None,
        None,
        None,
    );
    
    // Add examples first
    for example in examples {
        semantic_selector.aadd_example(example).await.unwrap();
    }
    
    let start = std::time::Instant::now();
    let _semantic_selected = semantic_selector.aselect_examples(&input).await.unwrap();
    let semantic_duration = start.elapsed();
    
    println!("Semantic similarity selection: {:?}", semantic_duration);
    
    let speedup = semantic_duration.as_nanos() as f64 / length_duration.as_nanos() as f64;
    println!("Length-based is {:.1}x faster than semantic similarity", speedup);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FerricLink Example Selector Usage Examples\n");
    
    // Run all examples
    length_based_example().await;
    semantic_similarity_example().await;
    max_marginal_relevance_example().await;
    utility_functions_example();
    text_length_functions_example();
    error_handling_example().await;
    performance_comparison().await;
    
    println!("\n✅ All example selector examples completed successfully!");
    println!("\n💡 Key Benefits of Example Selectors:");
    println!("  • Few-shot learning with relevant examples");
    println!("  • Dynamic prompt construction based on input");
    println!("  • Length management for token limits");
    println!("  • Semantic similarity for better context");
    println!("  • MMR for diverse example selection");
    
    Ok(())
}
