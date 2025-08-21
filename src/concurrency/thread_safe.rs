// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Thread-safe data structures for AetherScript
//!
//! Provides atomic types, lock-free data structures, and thread-safe collections

use crate::error::SemanticError;
use crate::types::Type;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::hash::Hash;

/// Manager for thread-safe data structures
#[derive(Debug)]
pub struct ThreadSafeManager {
    /// Registered atomic values
    atomics: HashMap<String, AtomicInfo>,
    
    /// Registered thread-safe collections
    collections: HashMap<String, CollectionInfo>,
    
    /// Thread-safe statistics
    stats: ThreadSafeStats,
}

/// Information about atomic values
#[derive(Debug, Clone)]
pub struct AtomicInfo {
    pub name: String,
    pub atomic_type: AtomicType,
    pub created_at: std::time::Instant,
    pub operations_count: u64,
}

/// Information about thread-safe collections
#[derive(Debug, Clone)]
pub struct CollectionInfo {
    pub name: String,
    pub collection_type: CollectionType,
    pub element_type: Type,
    pub created_at: std::time::Instant,
    pub size: usize,
    pub operations_count: u64,
}

/// Atomic type variants
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomicType {
    Bool,
    I32,
    I64,
    U32,
    U64,
    Usize,
    Pointer,
}

/// Collection type variants
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollectionType {
    AtomicVector,
    ConcurrentHashMap,
    LockFreeQueue,
    LockFreeStack,
    ThreadSafeSet,
    RwLockVector,
}

/// Thread-safe statistics
#[derive(Debug, Default)]
pub struct ThreadSafeStats {
    pub total_atomics: usize,
    pub total_collections: usize,
    pub total_atomic_operations: u64,
    pub total_collection_operations: u64,
    pub contention_events: u64,
}

/// Atomic boolean wrapper
#[derive(Debug)]
pub struct AetherAtomicBool {
    name: String,
    inner: AtomicBool,
    operations: AtomicU64,
}

/// Atomic integer wrapper
#[derive(Debug)]
pub struct AetherAtomicI64 {
    name: String,
    inner: AtomicI64,
    operations: AtomicU64,
}

/// Atomic unsigned integer wrapper
#[derive(Debug)]
pub struct AetherAtomicU64 {
    name: String,
    inner: AtomicU64,
    operations: AtomicU64,
}

/// Thread-safe vector
#[derive(Debug)]
pub struct ThreadSafeVector<T> {
    name: String,
    inner: RwLock<Vec<T>>,
    operations: AtomicU64,
}

/// Concurrent hash map
#[derive(Debug)]
pub struct ConcurrentHashMap<K, V> {
    name: String,
    inner: RwLock<HashMap<K, V>>,
    operations: AtomicU64,
}

/// Lock-free queue (simplified implementation)
#[derive(Debug)]
pub struct LockFreeQueue<T> {
    name: String,
    inner: Mutex<std::collections::VecDeque<T>>, // Simplified with mutex for now
    operations: AtomicU64,
}

/// Lock-free stack (simplified implementation)
#[derive(Debug)]
pub struct LockFreeStack<T> {
    name: String,
    inner: Mutex<Vec<T>>, // Simplified with mutex for now
    operations: AtomicU64,
}

/// Thread-safe set
#[derive(Debug)]
pub struct ThreadSafeSet<T> {
    _phantom: std::marker::PhantomData<T>,
}

/// Memory ordering for atomic operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryOrdering {
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
}

impl ThreadSafeManager {
    pub fn new() -> Self {
        Self {
            atomics: HashMap::new(),
            collections: HashMap::new(),
            stats: ThreadSafeStats::default(),
        }
    }
    
    /// Create an atomic boolean
    pub fn create_atomic_bool(&mut self, name: String, initial_value: bool) -> Result<Arc<AetherAtomicBool>, SemanticError> {
        if self.atomics.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }
        
        let atomic = Arc::new(AetherAtomicBool::new(name.clone(), initial_value));
        
        let info = AtomicInfo {
            name: name.clone(),
            atomic_type: AtomicType::Bool,
            created_at: std::time::Instant::now(),
            operations_count: 0,
        };
        
        self.atomics.insert(name, info);
        self.stats.total_atomics += 1;
        
        Ok(atomic)
    }
    
    /// Create an atomic i64
    pub fn create_atomic_i64(&mut self, name: String, initial_value: i64) -> Result<Arc<AetherAtomicI64>, SemanticError> {
        if self.atomics.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }
        
        let atomic = Arc::new(AetherAtomicI64::new(name.clone(), initial_value));
        
        let info = AtomicInfo {
            name: name.clone(),
            atomic_type: AtomicType::I64,
            created_at: std::time::Instant::now(),
            operations_count: 0,
        };
        
        self.atomics.insert(name, info);
        self.stats.total_atomics += 1;
        
        Ok(atomic)
    }
    
    /// Create an atomic u64
    pub fn create_atomic_u64(&mut self, name: String, initial_value: u64) -> Result<Arc<AetherAtomicU64>, SemanticError> {
        if self.atomics.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }
        
        let atomic = Arc::new(AetherAtomicU64::new(name.clone(), initial_value));
        
        let info = AtomicInfo {
            name: name.clone(),
            atomic_type: AtomicType::U64,
            created_at: std::time::Instant::now(),
            operations_count: 0,
        };
        
        self.atomics.insert(name, info);
        self.stats.total_atomics += 1;
        
        Ok(atomic)
    }
    
    /// Create a thread-safe vector
    pub fn create_thread_safe_vector<T: Send + Sync + 'static>(
        &mut self,
        name: String,
        element_type: Type,
    ) -> Result<Arc<ThreadSafeVector<T>>, SemanticError> {
        if self.collections.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }
        
        let vector = Arc::new(ThreadSafeVector::new(name.clone()));
        
        let info = CollectionInfo {
            name: name.clone(),
            collection_type: CollectionType::RwLockVector,
            element_type,
            created_at: std::time::Instant::now(),
            size: 0,
            operations_count: 0,
        };
        
        self.collections.insert(name, info);
        self.stats.total_collections += 1;
        
        Ok(vector)
    }
    
    /// Create a concurrent hash map
    pub fn create_concurrent_hashmap<K: Hash + Eq + Clone + Send + Sync + 'static, V: Clone + Send + Sync + 'static>(
        &mut self,
        name: String,
        value_type: Type,
    ) -> Result<Arc<ConcurrentHashMap<K, V>>, SemanticError> {
        if self.collections.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }
        
        let hashmap = Arc::new(ConcurrentHashMap::new(name.clone()));
        
        let info = CollectionInfo {
            name: name.clone(),
            collection_type: CollectionType::ConcurrentHashMap,
            element_type: value_type,
            created_at: std::time::Instant::now(),
            size: 0,
            operations_count: 0,
        };
        
        self.collections.insert(name, info);
        self.stats.total_collections += 1;
        
        Ok(hashmap)
    }
    
    /// Create a lock-free queue
    pub fn create_lock_free_queue<T: Send + Sync + 'static>(
        &mut self,
        name: String,
        element_type: Type,
    ) -> Result<Arc<LockFreeQueue<T>>, SemanticError> {
        if self.collections.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }
        
        let queue = Arc::new(LockFreeQueue::new(name.clone()));
        
        let info = CollectionInfo {
            name: name.clone(),
            collection_type: CollectionType::LockFreeQueue,
            element_type,
            created_at: std::time::Instant::now(),
            size: 0,
            operations_count: 0,
        };
        
        self.collections.insert(name, info);
        self.stats.total_collections += 1;
        
        Ok(queue)
    }
    
    /// Get atomic info
    pub fn get_atomic_info(&self, name: &str) -> Option<&AtomicInfo> {
        self.atomics.get(name)
    }
    
    /// Get collection info
    pub fn get_collection_info(&self, name: &str) -> Option<&CollectionInfo> {
        self.collections.get(name)
    }
    
    /// Get statistics
    pub fn stats(&self) -> &ThreadSafeStats {
        &self.stats
    }
}

impl AetherAtomicBool {
    pub fn new(name: String, initial_value: bool) -> Self {
        Self {
            name,
            inner: AtomicBool::new(initial_value),
            operations: AtomicU64::new(0),
        }
    }
    
    /// Load the value
    pub fn load(&self, ordering: MemoryOrdering) -> bool {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.load(ordering.into())
    }
    
    /// Store a value
    pub fn store(&self, value: bool, ordering: MemoryOrdering) {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.store(value, ordering.into());
    }
    
    /// Compare and swap
    pub fn compare_and_swap(&self, current: bool, new: bool, ordering: MemoryOrdering) -> bool {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.compare_exchange(current, new, ordering.into(), ordering.into()).unwrap_or(current)
    }
    
    /// Fetch and set
    pub fn fetch_and(&self, value: bool, ordering: MemoryOrdering) -> bool {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.fetch_and(value, ordering.into())
    }
    
    /// Fetch or
    pub fn fetch_or(&self, value: bool, ordering: MemoryOrdering) -> bool {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.fetch_or(value, ordering.into())
    }
    
    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operations.load(Ordering::Relaxed)
    }
    
    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl AetherAtomicI64 {
    pub fn new(name: String, initial_value: i64) -> Self {
        Self {
            name,
            inner: AtomicI64::new(initial_value),
            operations: AtomicU64::new(0),
        }
    }
    
    /// Load the value
    pub fn load(&self, ordering: MemoryOrdering) -> i64 {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.load(ordering.into())
    }
    
    /// Store a value
    pub fn store(&self, value: i64, ordering: MemoryOrdering) {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.store(value, ordering.into());
    }
    
    /// Compare and swap
    pub fn compare_and_swap(&self, current: i64, new: i64, ordering: MemoryOrdering) -> i64 {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.compare_exchange(current, new, ordering.into(), ordering.into()).unwrap_or(current)
    }
    
    /// Fetch and add
    pub fn fetch_add(&self, value: i64, ordering: MemoryOrdering) -> i64 {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.fetch_add(value, ordering.into())
    }
    
    /// Fetch and subtract
    pub fn fetch_sub(&self, value: i64, ordering: MemoryOrdering) -> i64 {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.fetch_sub(value, ordering.into())
    }
    
    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operations.load(Ordering::Relaxed)
    }
    
    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl AetherAtomicU64 {
    pub fn new(name: String, initial_value: u64) -> Self {
        Self {
            name,
            inner: AtomicU64::new(initial_value),
            operations: AtomicU64::new(0),
        }
    }
    
    /// Load the value
    pub fn load(&self, ordering: MemoryOrdering) -> u64 {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.load(ordering.into())
    }
    
    /// Store a value
    pub fn store(&self, value: u64, ordering: MemoryOrdering) {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.store(value, ordering.into());
    }
    
    /// Compare and swap
    pub fn compare_and_swap(&self, current: u64, new: u64, ordering: MemoryOrdering) -> u64 {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.compare_exchange(current, new, ordering.into(), ordering.into()).unwrap_or(current)
    }
    
    /// Fetch and add
    pub fn fetch_add(&self, value: u64, ordering: MemoryOrdering) -> u64 {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.fetch_add(value, ordering.into())
    }
    
    /// Fetch and subtract
    pub fn fetch_sub(&self, value: u64, ordering: MemoryOrdering) -> u64 {
        self.operations.fetch_add(1, Ordering::Relaxed);
        self.inner.fetch_sub(value, ordering.into())
    }
    
    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operations.load(Ordering::Relaxed)
    }
    
    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T> ThreadSafeVector<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            inner: RwLock::new(Vec::new()),
            operations: AtomicU64::new(0),
        }
    }
    
    /// Push an element
    pub fn push(&self, value: T) -> Result<(), SemanticError> {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let mut vec = self.inner.write().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire write lock".to_string(),
        })?;
        vec.push(value);
        Ok(())
    }
    
    /// Pop an element
    pub fn pop(&self) -> Result<Option<T>, SemanticError> {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let mut vec = self.inner.write().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire write lock".to_string(),
        })?;
        Ok(vec.pop())
    }
    
    /// Get length
    pub fn len(&self) -> Result<usize, SemanticError> {
        let vec = self.inner.read().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire read lock".to_string(),
        })?;
        Ok(vec.len())
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> Result<bool, SemanticError> {
        let vec = self.inner.read().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire read lock".to_string(),
        })?;
        Ok(vec.is_empty())
    }
    
    /// Get element at index
    pub fn get(&self, index: usize) -> Result<Option<T>, SemanticError> 
    where 
        T: Clone 
    {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let vec = self.inner.read().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire read lock".to_string(),
        })?;
        Ok(vec.get(index).cloned())
    }
    
    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operations.load(Ordering::Relaxed)
    }
    
    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<K, V> ConcurrentHashMap<K, V> 
where 
    K: Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(name: String) -> Self {
        Self {
            name,
            inner: RwLock::new(HashMap::new()),
            operations: AtomicU64::new(0),
        }
    }
    
    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) -> Result<Option<V>, SemanticError> {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let mut map = self.inner.write().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire write lock".to_string(),
        })?;
        Ok(map.insert(key, value))
    }
    
    /// Get a value by key
    pub fn get(&self, key: &K) -> Result<Option<V>, SemanticError> {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let map = self.inner.read().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire read lock".to_string(),
        })?;
        Ok(map.get(key).cloned())
    }
    
    /// Remove a key-value pair
    pub fn remove(&self, key: &K) -> Result<Option<V>, SemanticError> {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let mut map = self.inner.write().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire write lock".to_string(),
        })?;
        Ok(map.remove(key))
    }
    
    /// Check if key exists
    pub fn contains_key(&self, key: &K) -> Result<bool, SemanticError> {
        let map = self.inner.read().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire read lock".to_string(),
        })?;
        Ok(map.contains_key(key))
    }
    
    /// Get size
    pub fn len(&self) -> Result<usize, SemanticError> {
        let map = self.inner.read().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire read lock".to_string(),
        })?;
        Ok(map.len())
    }
    
    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operations.load(Ordering::Relaxed)
    }
    
    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T> LockFreeQueue<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            inner: Mutex::new(std::collections::VecDeque::new()),
            operations: AtomicU64::new(0),
        }
    }
    
    /// Enqueue an element
    pub fn enqueue(&self, value: T) -> Result<(), SemanticError> {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let mut queue = self.inner.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire queue lock".to_string(),
        })?;
        queue.push_back(value);
        Ok(())
    }
    
    /// Dequeue an element
    pub fn dequeue(&self) -> Result<Option<T>, SemanticError> {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let mut queue = self.inner.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire queue lock".to_string(),
        })?;
        Ok(queue.pop_front())
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> Result<bool, SemanticError> {
        let queue = self.inner.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire queue lock".to_string(),
        })?;
        Ok(queue.is_empty())
    }
    
    /// Get size
    pub fn len(&self) -> Result<usize, SemanticError> {
        let queue = self.inner.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire queue lock".to_string(),
        })?;
        Ok(queue.len())
    }
    
    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operations.load(Ordering::Relaxed)
    }
    
    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<T> LockFreeStack<T> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            inner: Mutex::new(Vec::new()),
            operations: AtomicU64::new(0),
        }
    }
    
    /// Push an element
    pub fn push(&self, value: T) -> Result<(), SemanticError> {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let mut stack = self.inner.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire stack lock".to_string(),
        })?;
        stack.push(value);
        Ok(())
    }
    
    /// Pop an element
    pub fn pop(&self) -> Result<Option<T>, SemanticError> {
        self.operations.fetch_add(1, Ordering::Relaxed);
        let mut stack = self.inner.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire stack lock".to_string(),
        })?;
        Ok(stack.pop())
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> Result<bool, SemanticError> {
        let stack = self.inner.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire stack lock".to_string(),
        })?;
        Ok(stack.is_empty())
    }
    
    /// Get size
    pub fn len(&self) -> Result<usize, SemanticError> {
        let stack = self.inner.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to acquire stack lock".to_string(),
        })?;
        Ok(stack.len())
    }
    
    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operations.load(Ordering::Relaxed)
    }
    
    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl From<MemoryOrdering> for Ordering {
    fn from(ordering: MemoryOrdering) -> Self {
        match ordering {
            MemoryOrdering::Relaxed => Ordering::Relaxed,
            MemoryOrdering::Acquire => Ordering::Acquire,
            MemoryOrdering::Release => Ordering::Release,
            MemoryOrdering::AcqRel => Ordering::AcqRel,
            MemoryOrdering::SeqCst => Ordering::SeqCst,
        }
    }
}

impl Default for ThreadSafeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;
    use crate::ast::PrimitiveType;
    
    #[test]
    fn test_thread_safe_manager_creation() {
        let manager = ThreadSafeManager::new();
        assert_eq!(manager.stats.total_atomics, 0);
        assert_eq!(manager.stats.total_collections, 0);
    }
    
    #[test]
    fn test_atomic_bool_creation() {
        let mut manager = ThreadSafeManager::new();
        let atomic = manager.create_atomic_bool("test_bool".to_string(), true).unwrap();
        
        assert_eq!(atomic.name(), "test_bool");
        assert_eq!(atomic.load(MemoryOrdering::Relaxed), true);
        assert_eq!(manager.stats.total_atomics, 1);
    }
    
    #[test]
    fn test_atomic_bool_operations() {
        let mut manager = ThreadSafeManager::new();
        let atomic = manager.create_atomic_bool("ops_test".to_string(), false).unwrap();
        
        // Test store and load
        atomic.store(true, MemoryOrdering::Relaxed);
        assert_eq!(atomic.load(MemoryOrdering::Relaxed), true);
        
        // Test compare and swap
        let old = atomic.compare_and_swap(true, false, MemoryOrdering::Relaxed);
        assert_eq!(old, true);
        assert_eq!(atomic.load(MemoryOrdering::Relaxed), false);
        
        // Check operation count
        assert!(atomic.operation_count() > 0);
    }
    
    #[test]
    fn test_atomic_i64_creation() {
        let mut manager = ThreadSafeManager::new();
        let atomic = manager.create_atomic_i64("test_i64".to_string(), 42).unwrap();
        
        assert_eq!(atomic.name(), "test_i64");
        assert_eq!(atomic.load(MemoryOrdering::Relaxed), 42);
        assert_eq!(manager.stats.total_atomics, 1);
    }
    
    #[test]
    fn test_atomic_i64_operations() {
        let mut manager = ThreadSafeManager::new();
        let atomic = manager.create_atomic_i64("math_test".to_string(), 10).unwrap();
        
        // Test fetch_add
        let old = atomic.fetch_add(5, MemoryOrdering::Relaxed);
        assert_eq!(old, 10);
        assert_eq!(atomic.load(MemoryOrdering::Relaxed), 15);
        
        // Test fetch_sub
        let old = atomic.fetch_sub(3, MemoryOrdering::Relaxed);
        assert_eq!(old, 15);
        assert_eq!(atomic.load(MemoryOrdering::Relaxed), 12);
    }
    
    #[test]
    fn test_thread_safe_vector_creation() {
        let mut manager = ThreadSafeManager::new();
        let vector = manager.create_thread_safe_vector::<i32>(
            "test_vector".to_string(),
            Type::primitive(PrimitiveType::Integer),
        ).unwrap();
        
        assert_eq!(vector.name(), "test_vector");
        assert_eq!(vector.len().unwrap(), 0);
        assert!(vector.is_empty().unwrap());
        assert_eq!(manager.stats.total_collections, 1);
    }
    
    #[test]
    fn test_thread_safe_vector_operations() {
        let mut manager = ThreadSafeManager::new();
        let vector = manager.create_thread_safe_vector::<String>(
            "string_vector".to_string(),
            Type::primitive(PrimitiveType::String),
        ).unwrap();
        
        // Test push
        assert!(vector.push("hello".to_string()).is_ok());
        assert!(vector.push("world".to_string()).is_ok());
        
        assert_eq!(vector.len().unwrap(), 2);
        assert!(!vector.is_empty().unwrap());
        
        // Test get
        assert_eq!(vector.get(0).unwrap(), Some("hello".to_string()));
        assert_eq!(vector.get(1).unwrap(), Some("world".to_string()));
        assert_eq!(vector.get(2).unwrap(), None);
        
        // Test pop
        assert_eq!(vector.pop().unwrap(), Some("world".to_string()));
        assert_eq!(vector.len().unwrap(), 1);
    }
    
    #[test]
    fn test_concurrent_hashmap_creation() {
        let mut manager = ThreadSafeManager::new();
        let hashmap = manager.create_concurrent_hashmap::<String, i32>(
            "test_map".to_string(),
            Type::primitive(PrimitiveType::Integer),
        ).unwrap();
        
        assert_eq!(hashmap.name(), "test_map");
        assert_eq!(hashmap.len().unwrap(), 0);
    }
    
    #[test]
    fn test_concurrent_hashmap_operations() {
        let mut manager = ThreadSafeManager::new();
        let hashmap = manager.create_concurrent_hashmap::<String, i32>(
            "ops_map".to_string(),
            Type::primitive(PrimitiveType::Integer),
        ).unwrap();
        
        // Test insert
        assert_eq!(hashmap.insert("key1".to_string(), 100).unwrap(), None);
        assert_eq!(hashmap.insert("key2".to_string(), 200).unwrap(), None);
        
        // Test overwrite
        assert_eq!(hashmap.insert("key1".to_string(), 150).unwrap(), Some(100));
        
        // Test get
        assert_eq!(hashmap.get(&"key1".to_string()).unwrap(), Some(150));
        assert_eq!(hashmap.get(&"key2".to_string()).unwrap(), Some(200));
        assert_eq!(hashmap.get(&"key3".to_string()).unwrap(), None);
        
        // Test contains_key
        assert!(hashmap.contains_key(&"key1".to_string()).unwrap());
        assert!(!hashmap.contains_key(&"key3".to_string()).unwrap());
        
        // Test remove
        assert_eq!(hashmap.remove(&"key1".to_string()).unwrap(), Some(150));
        assert_eq!(hashmap.remove(&"key1".to_string()).unwrap(), None);
        
        assert_eq!(hashmap.len().unwrap(), 1);
    }
    
    #[test]
    fn test_lock_free_queue_operations() {
        let mut manager = ThreadSafeManager::new();
        let queue = manager.create_lock_free_queue::<String>(
            "test_queue".to_string(),
            Type::primitive(PrimitiveType::String),
        ).unwrap();
        
        assert!(queue.is_empty().unwrap());
        
        // Test enqueue
        assert!(queue.enqueue("first".to_string()).is_ok());
        assert!(queue.enqueue("second".to_string()).is_ok());
        
        assert_eq!(queue.len().unwrap(), 2);
        assert!(!queue.is_empty().unwrap());
        
        // Test dequeue (FIFO)
        assert_eq!(queue.dequeue().unwrap(), Some("first".to_string()));
        assert_eq!(queue.dequeue().unwrap(), Some("second".to_string()));
        assert_eq!(queue.dequeue().unwrap(), None);
        
        assert!(queue.is_empty().unwrap());
    }
    
    #[test]
    fn test_memory_ordering_conversion() {
        assert_eq!(Ordering::from(MemoryOrdering::Relaxed), Ordering::Relaxed);
        assert_eq!(Ordering::from(MemoryOrdering::Acquire), Ordering::Acquire);
        assert_eq!(Ordering::from(MemoryOrdering::Release), Ordering::Release);
        assert_eq!(Ordering::from(MemoryOrdering::AcqRel), Ordering::AcqRel);
        assert_eq!(Ordering::from(MemoryOrdering::SeqCst), Ordering::SeqCst);
    }
}