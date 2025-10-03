//! Cache classes for FerricLink Core.
//!
//! **Cache** provides an optional caching layer for LLMs.
//!
//! Cache is useful for two reasons:
//!
//! - It can save you money by reducing the number of API calls you make to the LLM
//!   provider if you're often requesting the same completion multiple times.
//! - It can speed up your application by reducing the number of API calls you make
//!   to the LLM provider.
//!
//! Cache directly competes with Memory. See documentation for Pros and Cons.
//!
//! **Class hierarchy:**
//!
//! ```text
//! BaseCache --> <name>Cache  # Examples: InMemoryCache, RedisCache, GPTCache
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::errors::Result;
use crate::impl_serializable;
use crate::language_models::Generation;

/// Type alias for cached return values
pub type CachedGenerations = Vec<Generation>;

/// Interface for a caching layer for LLMs and Chat models.
///
/// The cache interface consists of the following methods:
///
/// - lookup: Look up a value based on a prompt and llm_string.
/// - update: Update the cache based on a prompt and llm_string.
/// - clear: Clear the cache.
///
/// In addition, the cache interface provides an async version of each method.
///
/// The default implementation of the async methods is to run the synchronous
/// method in an executor. It's recommended to override the async methods
/// and provide async implementations to avoid unnecessary overhead.
#[async_trait]
pub trait BaseCache: Send + Sync {
    /// Look up based on prompt and llm_string.
    ///
    /// A cache implementation is expected to generate a key from the 2-tuple
    /// of prompt and llm_string (e.g., by concatenating them with a delimiter).
    ///
    /// # Arguments
    ///
    /// * `prompt` - A string representation of the prompt.
    ///   In the case of a Chat model, the prompt is a non-trivial
    ///   serialization of the prompt into the language model.
    /// * `llm_string` - A string representation of the LLM configuration.
    ///   This is used to capture the invocation parameters of the LLM
    ///   (e.g., model name, temperature, stop tokens, max tokens, etc.).
    ///   These invocation parameters are serialized into a string
    ///   representation.
    ///
    /// # Returns
    ///
    /// On a cache miss, return None. On a cache hit, return the cached value.
    /// The cached value is a list of Generations (or subclasses).
    fn lookup(&self, prompt: &str, llm_string: &str) -> Result<Option<CachedGenerations>>;

    /// Update cache based on prompt and llm_string.
    ///
    /// The prompt and llm_string are used to generate a key for the cache.
    /// The key should match that of the lookup method.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A string representation of the prompt.
    ///   In the case of a Chat model, the prompt is a non-trivial
    ///   serialization of the prompt into the language model.
    /// * `llm_string` - A string representation of the LLM configuration.
    ///   This is used to capture the invocation parameters of the LLM
    ///   (e.g., model name, temperature, stop tokens, max tokens, etc.).
    ///   These invocation parameters are serialized into a string
    ///   representation.
    /// * `return_val` - The value to be cached. The value is a list of Generations
    ///   (or subclasses).
    fn update(&self, prompt: &str, llm_string: &str, return_val: CachedGenerations) -> Result<()>;

    /// Clear cache that can take additional keyword arguments.
    fn clear(&self) -> Result<()>;

    /// Async look up based on prompt and llm_string.
    ///
    /// A cache implementation is expected to generate a key from the 2-tuple
    /// of prompt and llm_string (e.g., by concatenating them with a delimiter).
    ///
    /// # Arguments
    ///
    /// * `prompt` - A string representation of the prompt.
    ///   In the case of a Chat model, the prompt is a non-trivial
    ///   serialization of the prompt into the language model.
    /// * `llm_string` - A string representation of the LLM configuration.
    ///   This is used to capture the invocation parameters of the LLM
    ///   (e.g., model name, temperature, stop tokens, max tokens, etc.).
    ///   These invocation parameters are serialized into a string
    ///   representation.
    ///
    /// # Returns
    ///
    /// On a cache miss, return None. On a cache hit, return the cached value.
    /// The cached value is a list of Generations (or subclasses).
    async fn alookup(&self, prompt: &str, llm_string: &str) -> Result<Option<CachedGenerations>>;

    /// Async update cache based on prompt and llm_string.
    ///
    /// The prompt and llm_string are used to generate a key for the cache.
    /// The key should match that of the lookup method.
    ///
    /// # Arguments
    ///
    /// * `prompt` - A string representation of the prompt.
    ///   In the case of a Chat model, the prompt is a non-trivial
    ///   serialization of the prompt into the language model.
    /// * `llm_string` - A string representation of the LLM configuration.
    ///   This is used to capture the invocation parameters of the LLM
    ///   (e.g., model name, temperature, stop tokens, max tokens, etc.).
    ///   These invocation parameters are serialized into a string
    ///   representation.
    /// * `return_val` - The value to be cached. The value is a list of Generations
    ///   (or subclasses).
    async fn aupdate(&self, prompt: &str, llm_string: &str, return_val: CachedGenerations) -> Result<()>;

    /// Async clear cache that can take additional keyword arguments.
    async fn aclear(&self) -> Result<()>;
}

/// Cache that stores things in memory.
///
/// This is a simple in-memory cache implementation that stores cached values
/// in a HashMap. It supports optional size limits and LRU eviction.
#[derive(Debug)]
pub struct InMemoryCache {
    /// The actual cache storage
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Maximum number of items to store in the cache
    max_size: Option<usize>,
    /// Statistics for monitoring
    stats: Arc<RwLock<CacheStats>>,
}

/// A cache entry that includes the data and metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached generations
    data: CachedGenerations,
    /// When this entry was created
    created_at: Instant,
    /// When this entry was last accessed
    last_accessed: Instant,
    /// Number of times this entry has been accessed
    access_count: u64,
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of cache updates
    pub updates: u64,
    /// Number of cache clears
    pub clears: u64,
    /// Total number of entries currently in cache
    pub current_size: usize,
    /// Maximum size the cache has reached
    pub max_size_reached: usize,
}

impl CacheStats {
    /// Get the hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Get the total number of requests
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }
}

impl InMemoryCache {
    /// Create a new in-memory cache with no size limit.
    pub fn new() -> Self {
        Self::with_max_size(None)
    }

    /// Create a new in-memory cache with a maximum size.
    ///
    /// # Arguments
    ///
    /// * `max_size` - The maximum number of items to store in the cache.
    ///   If None, the cache has no maximum size.
    ///   If the cache exceeds the maximum size, the oldest items are removed.
    ///
    /// # Panics
    ///
    /// Panics if `max_size` is Some(0).
    pub fn with_max_size(max_size: Option<usize>) -> Self {
        if let Some(size) = max_size {
            assert!(size > 0, "max_size must be greater than 0");
        }

        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Generate a cache key from prompt and llm_string.
    fn generate_key(prompt: &str, llm_string: &str) -> String {
        // Use a simple concatenation with a delimiter
        // In production, you might want to use a hash function
        format!("{}|||{}", prompt, llm_string)
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        let cache = self.cache.read().await;
        CacheStats {
            current_size: cache.len(),
            ..stats.clone()
        }
    }

    /// Get the current cache size.
    pub async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Check if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        let cache = self.cache.read().await;
        cache.is_empty()
    }

    /// Get the maximum cache size.
    pub fn max_size(&self) -> Option<usize> {
        self.max_size
    }

}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseCache for InMemoryCache {
    fn lookup(&self, prompt: &str, llm_string: &str) -> Result<Option<CachedGenerations>> {
        let key = Self::generate_key(prompt, llm_string);
        let mut cache = self.cache.try_write()
            .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Cache lock error: {}", e)))?;
        let mut stats = self.stats.try_write()
            .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Stats lock error: {}", e)))?;

        if let Some(entry) = cache.get_mut(&key) {
            // Update access information
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            stats.hits += 1;
            Ok(Some(entry.data.clone()))
        } else {
            stats.misses += 1;
            Ok(None)
        }
    }

    fn update(&self, prompt: &str, llm_string: &str, return_val: CachedGenerations) -> Result<()> {
        let key = Self::generate_key(prompt, llm_string);
        let mut cache = self.cache.try_write()
            .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Cache lock error: {}", e)))?;
        let mut stats = self.stats.try_write()
            .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Stats lock error: {}", e)))?;

        // Evict if needed before adding new entry
        if let Some(max_size) = self.max_size {
            if cache.len() >= max_size {
                // Find and remove the least recently used entry
                let mut oldest_key = None;
                let mut oldest_time = Instant::now();

                for (key, entry) in cache.iter() {
                    if entry.last_accessed < oldest_time {
                        oldest_time = entry.last_accessed;
                        oldest_key = Some(key.clone());
                    }
                }

                if let Some(key) = oldest_key {
                    cache.remove(&key);
                }
            }
        }

        let entry = CacheEntry {
            data: return_val,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
        };

        cache.insert(key, entry);
        stats.updates += 1;
        stats.max_size_reached = stats.max_size_reached.max(cache.len());

        Ok(())
    }

    fn clear(&self) -> Result<()> {
        let mut cache = self.cache.try_write()
            .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Cache lock error: {}", e)))?;
        let mut stats = self.stats.try_write()
            .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Stats lock error: {}", e)))?;

        cache.clear();
        stats.clears += 1;
        Ok(())
    }

    async fn alookup(&self, prompt: &str, llm_string: &str) -> Result<Option<CachedGenerations>> {
        let key = Self::generate_key(prompt, llm_string);
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = cache.get_mut(&key) {
            // Update access information
            entry.last_accessed = Instant::now();
            entry.access_count += 1;
            stats.hits += 1;
            Ok(Some(entry.data.clone()))
        } else {
            stats.misses += 1;
            Ok(None)
        }
    }

    async fn aupdate(&self, prompt: &str, llm_string: &str, return_val: CachedGenerations) -> Result<()> {
        let key = Self::generate_key(prompt, llm_string);
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        // Evict if needed before adding new entry
        if let Some(max_size) = self.max_size {
            if cache.len() >= max_size {
                // Find and remove the least recently used entry
                let mut oldest_key = None;
                let mut oldest_time = Instant::now();

                for (key, entry) in cache.iter() {
                    if entry.last_accessed < oldest_time {
                        oldest_time = entry.last_accessed;
                        oldest_key = Some(key.clone());
                    }
                }

                if let Some(key) = oldest_key {
                    cache.remove(&key);
                }
            }
        }

        let entry = CacheEntry {
            data: return_val,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
        };

        cache.insert(key, entry);
        stats.updates += 1;
        stats.max_size_reached = stats.max_size_reached.max(cache.len());

        Ok(())
    }

    async fn aclear(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        let mut stats = self.stats.write().await;

        cache.clear();
        stats.clears += 1;
        Ok(())
    }
}

impl_serializable!(CacheStats, ["ferriclink", "caches", "cache_stats"]);

/// A more advanced cache with TTL (Time To Live) support.
#[derive(Debug)]
pub struct TtlCache {
    /// The underlying cache
    inner: InMemoryCache,
    /// Default TTL for cache entries
    default_ttl: Duration,
}

impl TtlCache {
    /// Create a new TTL cache with the specified TTL.
    ///
    /// # Arguments
    ///
    /// * `default_ttl` - The default time-to-live for cache entries.
    /// * `max_size` - Optional maximum size for the cache.
    pub fn new(default_ttl: Duration, max_size: Option<usize>) -> Self {
        Self {
            inner: InMemoryCache::with_max_size(max_size),
            default_ttl,
        }
    }

    /// Get the default TTL.
    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        self.inner.stats().await
    }

    /// Check if an entry has expired.
    fn is_expired(entry: &CacheEntry, ttl: Duration) -> bool {
        entry.created_at.elapsed() > ttl
    }
}

#[async_trait]
impl BaseCache for TtlCache {
    fn lookup(&self, prompt: &str, llm_string: &str) -> Result<Option<CachedGenerations>> {
        // For TTL cache, we need to check expiration
        let key = InMemoryCache::generate_key(prompt, llm_string);
        let mut cache = self.inner.cache.try_write()
            .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Cache lock error: {}", e)))?;
        let mut stats = self.inner.stats.try_write()
            .map_err(|e| crate::errors::FerricLinkError::runtime(format!("Stats lock error: {}", e)))?;

        if let Some(entry) = cache.get(&key) {
            if Self::is_expired(entry, self.default_ttl) {
                // Entry has expired, remove it
                cache.remove(&key);
                stats.misses += 1;
                Ok(None)
            } else {
                // Entry is still valid, update access info
                let mut entry = entry.clone();
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                let data = entry.data.clone();
                cache.insert(key, entry);
                stats.hits += 1;
                Ok(Some(data))
            }
        } else {
            stats.misses += 1;
            Ok(None)
        }
    }

    fn update(&self, prompt: &str, llm_string: &str, return_val: CachedGenerations) -> Result<()> {
        self.inner.update(prompt, llm_string, return_val)
    }

    fn clear(&self) -> Result<()> {
        self.inner.clear()
    }

    async fn alookup(&self, prompt: &str, llm_string: &str) -> Result<Option<CachedGenerations>> {
        // For TTL cache, we need to check expiration
        let key = InMemoryCache::generate_key(prompt, llm_string);
        let mut cache = self.inner.cache.write().await;
        let mut stats = self.inner.stats.write().await;

        if let Some(entry) = cache.get(&key) {
            if Self::is_expired(entry, self.default_ttl) {
                // Entry has expired, remove it
                cache.remove(&key);
                stats.misses += 1;
                Ok(None)
            } else {
                // Entry is still valid, update access info
                let mut entry = entry.clone();
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                let data = entry.data.clone();
                cache.insert(key, entry);
                stats.hits += 1;
                Ok(Some(data))
            }
        } else {
            stats.misses += 1;
            Ok(None)
        }
    }

    async fn aupdate(&self, prompt: &str, llm_string: &str, return_val: CachedGenerations) -> Result<()> {
        self.inner.aupdate(prompt, llm_string, return_val).await
    }

    async fn aclear(&self) -> Result<()> {
        self.inner.aclear().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_models::Generation;

    fn create_test_generation(text: &str) -> Generation {
        Generation {
            text: text.to_string(),
            generation_info: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_in_memory_cache_basic() {
        let cache = InMemoryCache::new();
        
        // Test empty cache
        assert!(cache.lookup("test", "llm").unwrap().is_none());
        
        // Test update and lookup
        let generations = vec![create_test_generation("Hello, world!")];
        cache.update("test", "llm", generations.clone()).unwrap();
        
        let result = cache.lookup("test", "llm").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), generations);
    }

    #[test]
    fn test_in_memory_cache_with_max_size() {
        let cache = InMemoryCache::with_max_size(Some(2));
        
        // Add entries up to max size
        cache.update("test1", "llm", vec![create_test_generation("1")]).unwrap();
        cache.update("test2", "llm", vec![create_test_generation("2")]).unwrap();
        
        // This should evict the first entry
        cache.update("test3", "llm", vec![create_test_generation("3")]).unwrap();
        
        assert!(cache.lookup("test1", "llm").unwrap().is_none());
        assert!(cache.lookup("test2", "llm").unwrap().is_some());
        assert!(cache.lookup("test3", "llm").unwrap().is_some());
    }

    #[test]
    fn test_in_memory_cache_clear() {
        let cache = InMemoryCache::new();
        
        cache.update("test", "llm", vec![create_test_generation("Hello")]).unwrap();
        assert!(cache.lookup("test", "llm").unwrap().is_some());
        
        cache.clear().unwrap();
        assert!(cache.lookup("test", "llm").unwrap().is_none());
    }

    #[tokio::test]
    async fn test_in_memory_cache_async() {
        let cache = InMemoryCache::new();
        
        // Test async operations
        let generations = vec![create_test_generation("Async test")];
        cache.aupdate("test", "llm", generations.clone()).await.unwrap();
        
        let result = cache.alookup("test", "llm").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), generations);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = InMemoryCache::new();
        
        // Initial stats
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.updates, 0);
        
        // Test miss
        cache.alookup("test", "llm").await.unwrap();
        let stats = cache.stats().await;
        assert_eq!(stats.misses, 1);
        
        // Test update
        cache.aupdate("test", "llm", vec![create_test_generation("Hello")]).await.unwrap();
        let stats = cache.stats().await;
        assert_eq!(stats.updates, 1);
        
        // Test hit
        cache.alookup("test", "llm").await.unwrap();
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_ttl_cache() {
        let cache = TtlCache::new(Duration::from_millis(100), None);
        
        // Add entry
        cache.update("test", "llm", vec![create_test_generation("TTL test")]).unwrap();
        assert!(cache.lookup("test", "llm").unwrap().is_some());
        
        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));
        assert!(cache.lookup("test", "llm").unwrap().is_none());
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = InMemoryCache::generate_key("prompt1", "llm1");
        let key2 = InMemoryCache::generate_key("prompt2", "llm1");
        let key3 = InMemoryCache::generate_key("prompt1", "llm2");
        let key4 = InMemoryCache::generate_key("prompt1", "llm1");
        
        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_eq!(key1, key4);
    }
}
