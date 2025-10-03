---
sidebar_position: 5
---

# Example Selectors Guide

FerricLink provides comprehensive example selector functionality similar to LangChain's example selectors, with Rust-specific optimizations and additional features.

## Overview

Example selectors are crucial for building sophisticated LLM applications by:

- **Few-shot Learning**: Selecting relevant examples for prompts
- **Dynamic Prompt Construction**: Building context-aware prompts
- **Length Management**: Controlling prompt size and token limits
- **Semantic Similarity**: Finding similar examples using embeddings
- **Max Marginal Relevance**: Advanced example selection algorithms

## Basic Usage

### LengthBasedExampleSelector

The `LengthBasedExampleSelector` chooses examples that fit within a maximum length constraint:

```rust
use ferriclink_core::{LengthBasedExampleSelector, BaseExampleSelector};

// Create a selector with word count limit
let mut selector = LengthBasedExampleSelector::with_word_count(
    examples,
    50, // Max 50 words
);

// Select examples for input
let input = [("input".to_string(), "Tell me about AI".to_string())]
    .iter()
    .cloned()
    .collect();

let selected = selector.select_examples(&input)?;
println!("Selected {} examples", selected.len());
```

### Different Length Functions

```rust
use ferriclink_core::LengthBasedExampleSelector;

// Word count (default)
let word_selector = LengthBasedExampleSelector::with_word_count(examples, 50);

// Character count
let char_selector = LengthBasedExampleSelector::with_char_count(examples, 200);

// Custom length function
let custom_selector = LengthBasedExampleSelector::new(
    examples,
    100,
    Some(|text: &str| text.split(' ').count() * 2), // Double word count
);
```

## Advanced Selectors

### SemanticSimilarityExampleSelector

Uses embeddings and vector similarity search to find the most relevant examples:

```rust
use ferriclink_core::{
    SemanticSimilarityExampleSelector, BaseExampleSelector,
    vectorstores::InMemoryVectorStore,
    embeddings::MockEmbeddings,
};

// Create vector store with embeddings
let embeddings = MockEmbeddings::new("mock-embeddings", 384);
let vectorstore = InMemoryVectorStore::new_with_embeddings(Box::new(embeddings));

let mut selector = SemanticSimilarityExampleSelector::new(
    Box::new(vectorstore),
    3, // Select top 3 most similar examples
    None, // No example key filtering
    Some(vec!["input".to_string()]), // Only use input for similarity
    None, // No extra vectorstore kwargs
);

// Add examples
for example in examples {
    selector.aadd_example(example).await?;
}

// Select similar examples
let selected = selector.aselect_examples(&input).await?;
```

### MaxMarginalRelevanceExampleSelector

Balances relevance and diversity in example selection:

```rust
use ferriclink_core::MaxMarginalRelevanceExampleSelector;

let mut selector = MaxMarginalRelevanceExampleSelector::new(
    Box::new(vectorstore),
    2, // Select 2 examples
    5, // Fetch 5 for reranking
    None, // No example key filtering
    Some(vec!["input".to_string()]), // Only use input for similarity
    None, // No extra vectorstore kwargs
);

let selected = selector.aselect_examples(&input).await?;
```

## Utility Functions

### sorted_values

Sort values in a dictionary by key:

```rust
use ferriclink_core::example_selectors::sorted_values;

let mut example = HashMap::new();
example.insert("z".to_string(), "last".to_string());
example.insert("a".to_string(), "first".to_string());
example.insert("m".to_string(), "middle".to_string());

let sorted = sorted_values(&example);
// Returns: ["first", "middle", "last"]
```

## Integration Patterns

### Basic Few-Shot Learning

```rust
use ferriclink_core::{LengthBasedExampleSelector, BaseExampleSelector};

struct FewShotLLM {
    selector: LengthBasedExampleSelector,
    // ... other fields
}

impl FewShotLLM {
    async fn generate_with_examples(
        &self,
        input: &str,
        max_examples: usize,
    ) -> Result<String> {
        let input_vars = [("input".to_string(), input.to_string())]
            .iter()
            .cloned()
            .collect();
        
        let examples = self.selector.select_examples(&input_vars)?;
        
        // Build prompt with examples
        let mut prompt = String::new();
        for example in examples {
            prompt.push_str(&format!(
                "Input: {}\nOutput: {}\n\n",
                example.get("input").unwrap_or(&"".to_string()),
                example.get("output").unwrap_or(&"".to_string())
            ));
        }
        
        prompt.push_str(&format!("Input: {}\nOutput:", input));
        
        // Call LLM with constructed prompt
        // let response = self.llm.generate(&prompt).await?;
        // Ok(response)
        
        Ok("Generated response".to_string()) // Placeholder
    }
}
```

### Dynamic Prompt Construction

```rust
use ferriclink_core::{SemanticSimilarityExampleSelector, BaseExampleSelector};

struct DynamicPromptBuilder {
    selector: SemanticSimilarityExampleSelector,
    template: String,
}

impl DynamicPromptBuilder {
    async fn build_prompt(&self, input: &str) -> Result<String> {
        let input_vars = [("input".to_string(), input.to_string())]
            .iter()
            .cloned()
            .collect();
        
        let examples = self.selector.aselect_examples(&input_vars).await?;
        
        // Build dynamic prompt
        let mut prompt = self.template.clone();
        
        // Add examples section
        if !examples.is_empty() {
            prompt.push_str("\n\nExamples:\n");
            for (i, example) in examples.iter().enumerate() {
                prompt.push_str(&format!(
                    "{}. Input: {}\n   Output: {}\n",
                    i + 1,
                    example.get("input").unwrap_or(&"".to_string()),
                    example.get("output").unwrap_or(&"".to_string())
                ));
            }
        }
        
        prompt.push_str(&format!("\n\nInput: {}\nOutput:", input));
        
        Ok(prompt)
    }
}
```

### Hybrid Selection Strategy

```rust
use ferriclink_core::{
    LengthBasedExampleSelector, SemanticSimilarityExampleSelector,
    BaseExampleSelector,
};

struct HybridExampleSelector {
    length_selector: LengthBasedExampleSelector,
    semantic_selector: SemanticSimilarityExampleSelector,
    strategy: SelectionStrategy,
}

enum SelectionStrategy {
    LengthOnly,
    SemanticOnly,
    Hybrid { length_weight: f32, semantic_weight: f32 },
}

impl HybridExampleSelector {
    async fn select_examples(&self, input: &Example) -> Result<Vec<Example>> {
        match self.strategy {
            SelectionStrategy::LengthOnly => {
                self.length_selector.select_examples(input)
            }
            SelectionStrategy::SemanticOnly => {
                self.semantic_selector.aselect_examples(input).await
            }
            SelectionStrategy::Hybrid { length_weight, semantic_weight } => {
                // Combine both strategies
                let length_examples = self.length_selector.select_examples(input)?;
                let semantic_examples = self.semantic_selector.aselect_examples(input).await?;
                
                // Merge and deduplicate
                let mut combined = Vec::new();
                combined.extend(length_examples);
                combined.extend(semantic_examples);
                
                // Remove duplicates (simplified)
                combined.sort_by_key(|e| e.get("input").unwrap_or(&"".to_string()));
                combined.dedup_by_key(|e| e.get("input").unwrap_or(&"".to_string()));
                
                Ok(combined)
            }
        }
    }
}
```

## Performance Optimization

### Caching Examples

```rust
use ferriclink_core::{LengthBasedExampleSelector, BaseExampleSelector};
use std::collections::HashMap;

struct CachedExampleSelector {
    selector: LengthBasedExampleSelector,
    cache: HashMap<String, Vec<Example>>,
}

impl CachedExampleSelector {
    fn select_examples_cached(&mut self, input: &Example) -> Result<Vec<Example>> {
        let input_key = self.input_to_key(input);
        
        if let Some(cached) = self.cache.get(&input_key) {
            return Ok(cached.clone());
        }
        
        let selected = self.selector.select_examples(input)?;
        self.cache.insert(input_key, selected.clone());
        Ok(selected)
    }
    
    fn input_to_key(&self, input: &Example) -> String {
        // Create a cache key from input
        let mut values: Vec<_> = input.values().cloned().collect();
        values.sort();
        values.join("|")
    }
}
```

### Batch Processing

```rust
async fn process_batch(
    selector: &LengthBasedExampleSelector,
    inputs: Vec<Example>,
) -> Result<Vec<Vec<Example>>> {
    let mut results = Vec::new();
    
    for input in inputs {
        let selected = selector.select_examples(&input)?;
        results.push(selected);
    }
    
    Ok(results)
}
```

## Best Practices

### 1. Choose the Right Selector

```rust
// For simple length management
let length_selector = LengthBasedExampleSelector::with_word_count(examples, 100);

// For semantic relevance
let semantic_selector = SemanticSimilarityExampleSelector::new(/* ... */);

// For diversity + relevance
let mmr_selector = MaxMarginalRelevanceExampleSelector::new(/* ... */);
```

### 2. Optimize Length Functions

```rust
// Custom length function for specific use cases
let custom_length = |text: &str| {
    // Count tokens more accurately
    text.split_whitespace().count() + 
    text.matches(".").count() * 2 + // Sentences are "heavier"
    text.matches(",").count() * 0.5  // Commas are lighter
};

let selector = LengthBasedExampleSelector::new(examples, 100, Some(custom_length));
```

### 3. Handle Edge Cases

```rust
async fn robust_example_selection(
    selector: &impl BaseExampleSelector,
    input: &Example,
) -> Result<Vec<Example>> {
    // Try to select examples
    match selector.select_examples(input) {
        Ok(examples) if !examples.is_empty() => Ok(examples),
        Ok(_) => {
            // Fallback: return first few examples
            println!("Warning: No examples selected, using fallback");
            Ok(vec![]) // Or some default examples
        }
        Err(e) => {
            println!("Error selecting examples: {}", e);
            Ok(vec![]) // Graceful degradation
        }
    }
}
```

### 4. Monitor Performance

```rust
use std::time::Instant;

async fn benchmark_selector(
    selector: &impl BaseExampleSelector,
    inputs: Vec<Example>,
) -> Result<()> {
    let start = Instant::now();
    
    for input in inputs {
        let _ = selector.select_examples(&input)?;
    }
    
    let duration = start.elapsed();
    println!("Processed {} inputs in {:?}", inputs.len(), duration);
    
    Ok(())
}
```

## Comparison with LangChain

| Feature | LangChain Python | FerricLink Rust |
|---------|------------------|-----------------|
| **Base Interface** | `BaseExampleSelector` | `BaseExampleSelector` |
| **Length-Based** | `LengthBasedExampleSelector` | `LengthBasedExampleSelector` |
| **Semantic Similarity** | `SemanticSimilarityExampleSelector` | `SemanticSimilarityExampleSelector` |
| **MMR** | `MaxMarginalRelevanceExampleSelector` | `MaxMarginalRelevanceExampleSelector` |
| **Custom Length Functions** | ✅ | ✅ |
| **Async Support** | ✅ | ✅ |
| **Performance** | Medium | **High** |
| **Memory Safety** | Runtime checks | **Compile-time guarantees** |
| **Thread Safety** | GIL limitations | **True parallelism** |

## Troubleshooting

### Common Issues

1. **No Examples Selected**: Check length limits and input format
2. **Poor Semantic Similarity**: Ensure embeddings are properly configured
3. **Performance Issues**: Consider caching and batch processing
4. **Memory Usage**: Monitor vector store size and implement cleanup

### Debug Example Selection

```rust
fn debug_example_selection(
    selector: &LengthBasedExampleSelector,
    input: &Example,
) -> Result<()> {
    println!("Input: {:?}", input);
    println!("Total examples: {}", selector.len());
    println!("Total length: {} words", selector.total_length());
    
    let selected = selector.select_examples(input)?;
    println!("Selected {} examples:", selected.len());
    
    for (i, example) in selected.iter().enumerate() {
        let text = selector.example_to_text(example);
        let length = (selector.get_text_length)(&text);
        println!("  {}. Length: {} - {:?}", i + 1, length, example);
    }
    
    Ok(())
}
```

## Examples

See the [example selector usage example](../../examples/example_selector_usage) for a complete working demonstration of all example selector features.
