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

use aether::concurrency::{ConcurrencyManager, ConcurrencyAnalyzer};

#[test]
fn test_concurrency_manager_integration() {
    let mut manager = ConcurrencyManager::new();
    
    // Test that we can start and stop the concurrency manager
    assert!(manager.start().is_ok());
    assert!(manager.stop().is_ok());
}

#[test]
fn test_concurrency_analyzer_integration() {
    let analyzer = ConcurrencyAnalyzer::new();
    
    // Test that analyzer is created successfully
    assert!(analyzer.get_future("nonexistent").is_none());
    assert!(analyzer.get_channel("nonexistent").is_none());
    assert!(analyzer.get_actor("nonexistent").is_none());
}

#[test]
fn test_actor_system_integration() {
    use aether::concurrency::actors::ActorSystem;
    
    let mut system = ActorSystem::new();
    assert!(!system.is_running());
    
    assert!(system.start().is_ok());
    assert!(system.is_running());
    
    assert!(system.stop().is_ok());
    assert!(!system.is_running());
}

#[test]
fn test_async_runtime_integration() {
    use aether::concurrency::async_runtime::AsyncRuntime;
    
    let mut runtime = AsyncRuntime::new();
    assert!(runtime.is_idle());
    assert!(!runtime.is_running());
    
    assert!(runtime.start().is_ok());
    assert!(runtime.is_running());
    
    assert!(runtime.stop().is_ok());
}

#[test]
fn test_channel_registry_integration() {
    use aether::concurrency::channels::ChannelRegistry;
    use aether::types::Type;
    use aether::ast::PrimitiveType;
    
    let mut registry = ChannelRegistry::new();
    assert_eq!(registry.stats().total_channels, 0);
    
    let result = registry.create_unbounded::<String>(
        "test_channel".to_string(),
        Type::primitive(PrimitiveType::String),
    );
    
    assert!(result.is_ok());
    assert_eq!(registry.stats().total_channels, 1);
}

#[test]
fn test_thread_safe_manager_integration() {
    use aether::concurrency::thread_safe::ThreadSafeManager;
    
    let mut manager = ThreadSafeManager::new();
    assert_eq!(manager.stats().total_atomics, 0);
    
    let atomic_bool = manager.create_atomic_bool("test_bool".to_string(), true);
    assert!(atomic_bool.is_ok());
    assert_eq!(manager.stats().total_atomics, 1);
}

#[test]
fn test_sync_primitives_integration() {
    use aether::concurrency::sync_primitives::SyncPrimitiveManager;
    
    let mut manager = SyncPrimitiveManager::new();
    assert_eq!(manager.stats().total_mutexes, 0);
    
    let mutex = manager.create_mutex("test_mutex".to_string());
    assert!(mutex.is_ok());
    assert_eq!(manager.stats().total_mutexes, 1);
}