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

//! Synchronization primitives for AetherScript
//!
//! Provides mutexes, semaphores, condition variables, and other sync primitives

use crate::error::SemanticError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Condvar, Mutex, RwLock};
use std::time::{Duration, Instant};

/// Manager for synchronization primitives
#[derive(Debug)]
pub struct SyncPrimitiveManager {
    /// Registered mutexes
    mutexes: HashMap<String, Arc<AetherMutex>>,

    /// Registered semaphores
    semaphores: HashMap<String, Arc<AetherSemaphore>>,

    /// Registered condition variables
    condition_variables: HashMap<String, Arc<AetherCondVar>>,

    /// Registered read-write locks
    rw_locks: HashMap<String, Arc<AetherRwLock>>,

    /// Registered barriers
    barriers: HashMap<String, Arc<AetherBarrier>>,

    /// Sync primitive statistics
    stats: SyncStats,
}

/// AetherScript mutex wrapper
#[derive(Debug)]
pub struct AetherMutex {
    /// Inner mutex
    inner: Mutex<MutexState>,

    /// Mutex name
    name: String,

    /// Metrics
    metrics: Mutex<MutexMetrics>,
}

/// AetherScript semaphore
#[derive(Debug)]
pub struct AetherSemaphore {
    /// Current permits
    permits: AtomicUsize,

    /// Maximum permits
    max_permits: usize,

    /// Condition variable for waiting
    condvar: Condvar,

    /// Mutex for condvar
    mutex: Mutex<()>,

    /// Semaphore name
    name: String,

    /// Metrics
    metrics: Mutex<SemaphoreMetrics>,
}

/// AetherScript condition variable
#[derive(Debug)]
pub struct AetherCondVar {
    /// Condition variable
    inner: Condvar,

    /// Associated mutex name
    associated_mutex: Option<String>,

    /// Name
    name: String,

    /// Metrics
    metrics: Mutex<CondVarMetrics>,
}

/// AetherScript read-write lock
#[derive(Debug)]
pub struct AetherRwLock {
    /// Inner RwLock
    inner: RwLock<RwLockState>,

    /// Name
    name: String,

    /// Metrics
    metrics: Mutex<RwLockMetrics>,
}

/// AetherScript barrier
#[derive(Debug)]
pub struct AetherBarrier {
    /// Barrier
    inner: Barrier,

    /// Name
    name: String,

    /// Metrics
    metrics: Mutex<BarrierMetrics>,
}

/// Mutex state
#[derive(Debug, Default)]
struct MutexState {
    locked: bool,
    owner_thread: Option<std::thread::ThreadId>,
}

/// RwLock state
#[derive(Debug, Default)]
struct RwLockState {}

/// Barrier state
#[derive(Debug)]
struct BarrierState {
    waiting: usize,
    generation: usize,
}

/// Synchronization statistics
#[derive(Debug, Default)]
pub struct SyncStats {
    pub total_mutexes: usize,
    pub total_semaphores: usize,
    pub total_condition_variables: usize,
    pub total_rw_locks: usize,
    pub total_barriers: usize,
    pub total_lock_acquisitions: u64,
    pub total_lock_contentions: u64,
    pub avg_lock_hold_time_ms: f64,
}

/// Mutex metrics
#[derive(Debug, Default)]
struct MutexMetrics {
    lock_count: u64,
    unlock_count: u64,
    contention_count: u64,
    total_wait_time_ms: f64,
    max_wait_time_ms: f64,
    total_hold_time_ms: f64,
}

/// Semaphore metrics
#[derive(Debug, Default)]
struct SemaphoreMetrics {
    acquire_count: u64,
    release_count: u64,
    wait_count: u64,
    total_wait_time_ms: f64,
    max_wait_time_ms: f64,
}

/// Condition variable metrics
#[derive(Debug, Default)]
struct CondVarMetrics {
    /// Number of notifications
    notifications: u64,

    /// Number of waits
    waits: u64,
}

/// Read-write lock metrics
#[derive(Debug, Default)]
struct RwLockMetrics {
    /// Read locks acquired
    read_locks: u64,

    /// Write locks acquired
    write_locks: u64,
}

/// Barrier metrics
#[derive(Debug, Default)]
struct BarrierMetrics {
    wait_count: u64,
    barrier_cycles: u64,
    total_wait_time_ms: f64,
    max_wait_time_ms: f64,
}

/// Lock result
pub type LockResult<T> = Result<T, LockError>;

/// Lock error types
#[derive(Debug, Clone)]
pub enum LockError {
    /// Lock timeout
    Timeout,

    /// Lock poisoned
    Poisoned,

    /// Would block
    WouldBlock,

    /// Already locked by current thread
    AlreadyLocked,

    /// Not locked by current thread
    NotLocked,
}

/// Mutex guard
pub struct MutexGuard<'a> {
    mutex: &'a AetherMutex,
    acquired_at: Instant,
}

/// Read guard
pub struct ReadGuard<'a> {
    rw_lock: &'a AetherRwLock,
    acquired_at: Instant,
}

/// Write guard
pub struct WriteGuard<'a> {
    rw_lock: &'a AetherRwLock,
    acquired_at: Instant,
}

impl SyncPrimitiveManager {
    pub fn new() -> Self {
        Self {
            mutexes: HashMap::new(),
            semaphores: HashMap::new(),
            condition_variables: HashMap::new(),
            rw_locks: HashMap::new(),
            barriers: HashMap::new(),
            stats: SyncStats::default(),
        }
    }

    /// Create a new mutex
    pub fn create_mutex(&mut self, name: String) -> Result<Arc<AetherMutex>, SemanticError> {
        if self.mutexes.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }

        let mutex = Arc::new(AetherMutex::new(name.clone()));
        self.mutexes.insert(name, mutex.clone());
        self.stats.total_mutexes += 1;

        Ok(mutex)
    }

    /// Create a new semaphore
    pub fn create_semaphore(
        &mut self,
        name: String,
        permits: usize,
    ) -> Result<Arc<AetherSemaphore>, SemanticError> {
        if self.semaphores.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }

        let semaphore = Arc::new(AetherSemaphore::new(name.clone(), permits));
        self.semaphores.insert(name, semaphore.clone());
        self.stats.total_semaphores += 1;

        Ok(semaphore)
    }

    /// Create a new condition variable
    pub fn create_condition_variable(
        &mut self,
        name: String,
        mutex_name: Option<String>,
    ) -> Result<Arc<AetherCondVar>, SemanticError> {
        if self.condition_variables.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }

        let condvar = Arc::new(AetherCondVar::new(name.clone(), mutex_name));
        self.condition_variables.insert(name, condvar.clone());
        self.stats.total_condition_variables += 1;

        Ok(condvar)
    }

    /// Create a new read-write lock
    pub fn create_rw_lock(&mut self, name: String) -> Result<Arc<AetherRwLock>, SemanticError> {
        if self.rw_locks.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }

        let rw_lock = Arc::new(AetherRwLock::new(name.clone()));
        self.rw_locks.insert(name, rw_lock.clone());
        self.stats.total_rw_locks += 1;

        Ok(rw_lock)
    }

    /// Create a new barrier
    pub fn create_barrier(
        &mut self,
        name: String,
        thread_count: usize,
    ) -> Result<Arc<AetherBarrier>, SemanticError> {
        if self.barriers.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }

        let barrier = Arc::new(AetherBarrier::new(name.clone(), thread_count));
        self.barriers.insert(name, barrier.clone());
        self.stats.total_barriers += 1;

        Ok(barrier)
    }

    /// Get mutex by name
    pub fn get_mutex(&self, name: &str) -> Option<Arc<AetherMutex>> {
        self.mutexes.get(name).cloned()
    }

    /// Get semaphore by name
    pub fn get_semaphore(&self, name: &str) -> Option<Arc<AetherSemaphore>> {
        self.semaphores.get(name).cloned()
    }

    /// Get condition variable by name
    pub fn get_condition_variable(&self, name: &str) -> Option<Arc<AetherCondVar>> {
        self.condition_variables.get(name).cloned()
    }

    /// Get read-write lock by name
    pub fn get_rw_lock(&self, name: &str) -> Option<Arc<AetherRwLock>> {
        self.rw_locks.get(name).cloned()
    }

    /// Get barrier by name
    pub fn get_barrier(&self, name: &str) -> Option<Arc<AetherBarrier>> {
        self.barriers.get(name).cloned()
    }

    /// Get statistics
    pub fn stats(&self) -> &SyncStats {
        &self.stats
    }
}

impl AetherMutex {
    pub fn new(name: String) -> Self {
        Self {
            inner: Mutex::new(MutexState::default()),
            name,
            metrics: Mutex::new(MutexMetrics::default()),
        }
    }

    /// Lock the mutex
    pub fn lock(&self) -> LockResult<MutexGuard> {
        let start_time = Instant::now();

        match self.inner.lock() {
            Ok(mut state) => {
                if state.locked {
                    if state.owner_thread == Some(std::thread::current().id()) {
                        return Err(LockError::AlreadyLocked);
                    }
                    // Record contention
                    if let Ok(mut metrics) = self.metrics.lock() {
                        metrics.contention_count += 1;
                    }
                    return Err(LockError::WouldBlock);
                }

                state.locked = true;
                state.owner_thread = Some(std::thread::current().id());

                // Update metrics
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.lock_count += 1;
                    let wait_time = start_time.elapsed().as_millis() as f64;
                    metrics.total_wait_time_ms += wait_time;
                    if wait_time > metrics.max_wait_time_ms {
                        metrics.max_wait_time_ms = wait_time;
                    }
                }

                Ok(MutexGuard {
                    mutex: self,
                    acquired_at: Instant::now(),
                })
            }
            Err(_) => Err(LockError::Poisoned),
        }
    }

    /// Try to lock the mutex without blocking
    pub fn try_lock(&self) -> LockResult<MutexGuard> {
        match self.inner.try_lock() {
            Ok(mut state) => {
                if state.locked {
                    return Err(LockError::WouldBlock);
                }

                state.locked = true;
                state.owner_thread = Some(std::thread::current().id());

                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.lock_count += 1;
                }

                Ok(MutexGuard {
                    mutex: self,
                    acquired_at: Instant::now(),
                })
            }
            Err(_) => Err(LockError::WouldBlock),
        }
    }

    /// Get mutex name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl AetherSemaphore {
    pub fn new(name: String, permits: usize) -> Self {
        Self {
            permits: AtomicUsize::new(permits),
            max_permits: permits,
            condvar: Condvar::new(),
            mutex: Mutex::new(()),
            name,
            metrics: Mutex::new(SemaphoreMetrics::default()),
        }
    }

    /// Acquire a permit
    pub fn acquire(&self) -> LockResult<()> {
        let start_time = Instant::now();

        loop {
            let current = self.permits.load(Ordering::Acquire);
            if current > 0 {
                if self
                    .permits
                    .compare_exchange_weak(
                        current,
                        current - 1,
                        Ordering::Release,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    // Update metrics
                    if let Ok(mut metrics) = self.metrics.lock() {
                        metrics.acquire_count += 1;
                        let wait_time = start_time.elapsed().as_millis() as f64;
                        metrics.total_wait_time_ms += wait_time;
                        if wait_time > metrics.max_wait_time_ms {
                            metrics.max_wait_time_ms = wait_time;
                        }
                    }
                    return Ok(());
                }
            } else {
                // Wait for permits to become available
                let _guard = self.mutex.lock().unwrap();
                let _result = self.condvar.wait(_guard).unwrap();

                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.wait_count += 1;
                }
            }
        }
    }

    /// Try to acquire a permit without blocking
    pub fn try_acquire(&self) -> LockResult<()> {
        let current = self.permits.load(Ordering::Acquire);
        if current > 0 {
            if self
                .permits
                .compare_exchange_weak(current, current - 1, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.acquire_count += 1;
                }
                return Ok(());
            }
        }
        Err(LockError::WouldBlock)
    }

    /// Release a permit
    pub fn release(&self) {
        let old_permits = self.permits.fetch_add(1, Ordering::Release);
        if old_permits < self.max_permits {
            self.condvar.notify_one();

            if let Ok(mut metrics) = self.metrics.lock() {
                metrics.release_count += 1;
            }
        }
    }

    /// Get available permits
    pub fn available_permits(&self) -> usize {
        self.permits.load(Ordering::Acquire)
    }

    /// Get semaphore name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl AetherCondVar {
    pub fn new(name: String, associated_mutex: Option<String>) -> Self {
        Self {
            inner: Condvar::new(),
            associated_mutex,
            name,
            metrics: Mutex::new(CondVarMetrics::default()),
        }
    }

    /// Wait on the condition variable
    pub fn wait<'a>(&self, guard: MutexGuard<'a>) -> LockResult<MutexGuard<'a>> {
        let start_time = Instant::now();

        // In a real implementation, this would properly integrate with the mutex guard
        // For now, we'll simulate the wait
        std::thread::sleep(Duration::from_millis(1));

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.waits += 1;
        }

        Ok(guard)
    }

    /// Notify one waiting thread
    pub fn notify_one(&self) {
        self.inner.notify_one();

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.notifications += 1;
        }
    }

    /// Notify all waiting threads
    pub fn notify_all(&self) {
        self.inner.notify_all();

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.notifications += 1;
        }
    }

    /// Get condition variable name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl AetherRwLock {
    pub fn new(name: String) -> Self {
        Self {
            inner: RwLock::new(RwLockState::default()),
            name,
            metrics: Mutex::new(RwLockMetrics::default()),
        }
    }

    /// Acquire a read lock
    pub fn read(&self) -> LockResult<ReadGuard> {
        let start_time = Instant::now();

        match self.inner.read() {
            Ok(_guard) => {
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.read_locks += 1;
                }

                Ok(ReadGuard {
                    rw_lock: self,
                    acquired_at: start_time,
                })
            }
            Err(_) => Err(LockError::Poisoned),
        }
    }

    /// Acquire a write lock
    pub fn write(&self) -> LockResult<WriteGuard> {
        let start_time = Instant::now();

        match self.inner.write() {
            Ok(_guard) => {
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.write_locks += 1;
                }

                Ok(WriteGuard {
                    rw_lock: self,
                    acquired_at: start_time,
                })
            }
            Err(_) => Err(LockError::Poisoned),
        }
    }

    /// Get lock name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl AetherBarrier {
    pub fn new(name: String, thread_count: usize) -> Self {
        Self {
            inner: Barrier::new(thread_count),
            name,
            metrics: Mutex::new(BarrierMetrics::default()),
        }
    }

    /// Wait at the barrier
    pub fn wait(&self) -> LockResult<bool> {
        let start_time = Instant::now();

        let result = self.inner.wait();

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.wait_count += 1;
            let wait_time = start_time.elapsed().as_millis() as f64;
            metrics.total_wait_time_ms += wait_time;
            if wait_time > metrics.max_wait_time_ms {
                metrics.max_wait_time_ms = wait_time;
            }
        }

        Ok(result.is_leader())
    }

    /// Get barrier name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> Drop for MutexGuard<'a> {
    fn drop(&mut self) {
        // Unlock the mutex
        if let Ok(mut state) = self.mutex.inner.lock() {
            state.locked = false;
            state.owner_thread = None;

            // Update metrics
            if let Ok(mut metrics) = self.mutex.metrics.lock() {
                metrics.unlock_count += 1;
                let hold_time = self.acquired_at.elapsed().as_millis() as f64;
                metrics.total_hold_time_ms += hold_time;
            }
        }
    }
}

impl<'a> Drop for ReadGuard<'a> {
    fn drop(&mut self) {
        // Update metrics
        if let Ok(mut metrics) = self.rw_lock.metrics.lock() {}
    }
}

impl<'a> Drop for WriteGuard<'a> {
    fn drop(&mut self) {
        // Update metrics
        if let Ok(mut metrics) = self.rw_lock.metrics.lock() {}
    }
}

impl Default for SyncPrimitiveManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_primitive_manager_creation() {
        let manager = SyncPrimitiveManager::new();
        assert_eq!(manager.stats.total_mutexes, 0);
        assert_eq!(manager.stats.total_semaphores, 0);
    }

    #[test]
    fn test_mutex_creation() {
        let mut manager = SyncPrimitiveManager::new();
        let mutex = manager.create_mutex("test_mutex".to_string()).unwrap();

        assert_eq!(mutex.name(), "test_mutex");
        assert_eq!(manager.stats.total_mutexes, 1);
    }

    #[test]
    fn test_mutex_locking() {
        let mut manager = SyncPrimitiveManager::new();
        let mutex = manager.create_mutex("lock_test".to_string()).unwrap();

        let guard = mutex.lock();
        assert!(guard.is_ok());

        // Try to lock again (should fail)
        let second_guard = mutex.try_lock();
        assert!(second_guard.is_err());
    }

    #[test]
    fn test_semaphore_creation() {
        let mut manager = SyncPrimitiveManager::new();
        let semaphore = manager
            .create_semaphore("test_semaphore".to_string(), 3)
            .unwrap();

        assert_eq!(semaphore.name(), "test_semaphore");
        assert_eq!(semaphore.available_permits(), 3);
        assert_eq!(manager.stats.total_semaphores, 1);
    }

    #[test]
    fn test_semaphore_acquire_release() {
        let mut manager = SyncPrimitiveManager::new();
        let semaphore = manager
            .create_semaphore("acquire_test".to_string(), 2)
            .unwrap();

        // Acquire permits
        assert!(semaphore.try_acquire().is_ok());
        assert_eq!(semaphore.available_permits(), 1);

        assert!(semaphore.try_acquire().is_ok());
        assert_eq!(semaphore.available_permits(), 0);

        // Should fail when no permits available
        assert!(semaphore.try_acquire().is_err());

        // Release a permit
        semaphore.release();
        assert_eq!(semaphore.available_permits(), 1);

        // Should succeed now
        assert!(semaphore.try_acquire().is_ok());
    }

    #[test]
    fn test_rw_lock_creation() {
        let mut manager = SyncPrimitiveManager::new();
        let rw_lock = manager.create_rw_lock("test_rw_lock".to_string()).unwrap();

        assert_eq!(rw_lock.name(), "test_rw_lock");
        assert_eq!(manager.stats.total_rw_locks, 1);
    }

    #[test]
    fn test_rw_lock_read_write() {
        let mut manager = SyncPrimitiveManager::new();
        let rw_lock = manager.create_rw_lock("rw_test".to_string()).unwrap();

        // Multiple readers should be allowed
        let _read_guard1 = rw_lock.read().unwrap();
        let _read_guard2 = rw_lock.read().unwrap();

        // Drop readers before testing writer
        drop(_read_guard1);
        drop(_read_guard2);

        // Writer should work
        let _write_guard = rw_lock.write().unwrap();
    }

    #[test]
    fn test_barrier_creation() {
        let mut manager = SyncPrimitiveManager::new();
        let barrier = manager
            .create_barrier("test_barrier".to_string(), 3)
            .unwrap();

        assert_eq!(barrier.name(), "test_barrier");
        assert_eq!(manager.stats.total_barriers, 1);
    }

    #[test]
    fn test_duplicate_names() {
        let mut manager = SyncPrimitiveManager::new();

        assert!(manager.create_mutex("duplicate".to_string()).is_ok());
        assert!(manager.create_mutex("duplicate".to_string()).is_err());
    }
}
