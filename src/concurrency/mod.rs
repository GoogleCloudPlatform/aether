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

//! Concurrency primitives for AetherScript
//!
//! Provides async/await, channels, thread-safe data structures, and actor model support

pub mod async_runtime;
pub mod channels;
pub mod sync_primitives;
pub mod actors;
pub mod thread_safe;

use crate::error::SemanticError;
use crate::types::Type;
use std::collections::HashMap;

/// Concurrency manager for handling concurrent execution
#[derive(Debug)]
pub struct ConcurrencyManager {
    /// Active runtime for async execution
    runtime: async_runtime::AsyncRuntime,
    
    /// Channel registry for message passing
    channels: channels::ChannelRegistry,
    
    /// Actor system for actor model support
    actors: actors::ActorSystem,
    
    /// Thread-safe data structure manager
    thread_safe: thread_safe::ThreadSafeManager,
    
    /// Synchronization primitives
    sync_primitives: sync_primitives::SyncPrimitiveManager,
}

impl ConcurrencyManager {
    /// Create a new concurrency manager
    pub fn new() -> Self {
        Self {
            runtime: async_runtime::AsyncRuntime::new(),
            channels: channels::ChannelRegistry::new(),
            actors: actors::ActorSystem::new(),
            thread_safe: thread_safe::ThreadSafeManager::new(),
            sync_primitives: sync_primitives::SyncPrimitiveManager::new(),
        }
    }
    
    /// Start the concurrency manager
    pub fn start(&mut self) -> Result<(), SemanticError> {
        self.runtime.start()?;
        self.actors.start()?;
        Ok(())
    }
    
    /// Stop the concurrency manager
    pub fn stop(&mut self) -> Result<(), SemanticError> {
        self.actors.stop()?;
        self.runtime.stop()?;
        Ok(())
    }
    
    /// Get the async runtime
    pub fn runtime(&self) -> &async_runtime::AsyncRuntime {
        &self.runtime
    }
    
    /// Get the channel registry
    pub fn channels(&self) -> &channels::ChannelRegistry {
        &self.channels
    }
    
    /// Get mutable channel registry
    pub fn channels_mut(&mut self) -> &mut channels::ChannelRegistry {
        &mut self.channels
    }
    
    /// Get the actor system
    pub fn actors(&self) -> &actors::ActorSystem {
        &self.actors
    }
    
    /// Get mutable actor system
    pub fn actors_mut(&mut self) -> &mut actors::ActorSystem {
        &mut self.actors
    }
    
    /// Get the thread-safe manager
    pub fn thread_safe(&self) -> &thread_safe::ThreadSafeManager {
        &self.thread_safe
    }
    
    /// Get mutable thread-safe manager
    pub fn thread_safe_mut(&mut self) -> &mut thread_safe::ThreadSafeManager {
        &mut self.thread_safe
    }
    
    /// Get the sync primitives manager
    pub fn sync_primitives(&self) -> &sync_primitives::SyncPrimitiveManager {
        &self.sync_primitives
    }
    
    /// Get mutable sync primitives manager
    pub fn sync_primitives_mut(&mut self) -> &mut sync_primitives::SyncPrimitiveManager {
        &mut self.sync_primitives
    }
}

impl Default for ConcurrencyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Concurrency analysis for semantic checking
#[derive(Debug)]
pub struct ConcurrencyAnalyzer {
    /// Active futures/tasks
    futures: HashMap<String, FutureInfo>,
    
    /// Channel types and safety
    channel_types: HashMap<String, ChannelTypeInfo>,
    
    /// Actor type information
    actor_types: HashMap<String, ActorTypeInfo>,
    
    /// Thread safety analysis
    thread_safety: ThreadSafetyAnalyzer,
}

/// Information about a future/async function
#[derive(Debug, Clone)]
pub struct FutureInfo {
    pub name: String,
    pub return_type: Type,
    pub is_send: bool,
    pub is_sync: bool,
    pub captures: Vec<String>,
}

/// Information about channel types
#[derive(Debug, Clone)]
pub struct ChannelTypeInfo {
    pub sender_type: Type,
    pub receiver_type: Type,
    pub message_type: Type,
    pub is_bounded: bool,
    pub capacity: Option<usize>,
}

/// Information about actor types
#[derive(Debug, Clone)]
pub struct ActorTypeInfo {
    pub name: String,
    pub message_types: Vec<Type>,
    pub state_type: Type,
    pub is_supervisor: bool,
}

/// Thread safety analyzer
#[derive(Debug)]
pub struct ThreadSafetyAnalyzer {
    /// Variables that need to be Send
    send_required: HashMap<String, bool>,
    
    /// Shared data analysis
    shared_data: HashMap<String, SharedDataInfo>,
}

/// Information about shared data
#[derive(Debug, Clone)]
pub struct SharedDataInfo {
    pub data_type: Type,
    pub access_pattern: AccessPattern,
    pub synchronization: SynchronizationType,
}

/// Access pattern for shared data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessPattern {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    WriteHeavy,
    ReadHeavy,
}

/// Type of synchronization needed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynchronizationType {
    None,
    Mutex,
    RwLock,
    Atomic,
    Channel,
    ActorMessage,
}

impl ConcurrencyAnalyzer {
    pub fn new() -> Self {
        Self {
            futures: HashMap::new(),
            channel_types: HashMap::new(),
            actor_types: HashMap::new(),
            thread_safety: ThreadSafetyAnalyzer::new(),
        }
    }
    
    /// Analyze a future/async function
    pub fn analyze_future(&mut self, name: String, info: FutureInfo) -> Result<(), SemanticError> {
        // Check if the future is properly typed
        if !self.is_valid_future_type(&info.return_type) {
            return Err(SemanticError::InvalidType {
                type_name: format!("Future<{}>", info.return_type),
                reason: "Invalid future return type".to_string(),
                location: crate::error::SourceLocation::unknown(),
            });
        }
        
        // Analyze captured variables for Send/Sync requirements
        for capture in &info.captures {
            self.thread_safety.analyze_capture(capture, &info)?;
        }
        
        self.futures.insert(name, info);
        Ok(())
    }
    
    /// Analyze channel type safety
    pub fn analyze_channel(&mut self, name: String, info: ChannelTypeInfo) -> Result<(), SemanticError> {
        // Verify message type is Send
        if !self.is_send_type(&info.message_type) {
            return Err(SemanticError::InvalidType {
                type_name: info.message_type.to_string(),
                reason: "Channel message type must be Send".to_string(),
                location: crate::error::SourceLocation::unknown(),
            });
        }
        
        self.channel_types.insert(name, info);
        Ok(())
    }
    
    /// Analyze actor type safety
    pub fn analyze_actor(&mut self, name: String, info: ActorTypeInfo) -> Result<(), SemanticError> {
        // Verify all message types are Send
        for msg_type in &info.message_types {
            if !self.is_send_type(msg_type) {
                return Err(SemanticError::InvalidType {
                    type_name: msg_type.to_string(),
                    reason: "Actor message types must be Send".to_string(),
                    location: crate::error::SourceLocation::unknown(),
                });
            }
        }
        
        self.actor_types.insert(name, info);
        Ok(())
    }
    
    /// Check if a type is a valid future return type
    fn is_valid_future_type(&self, _ty: &Type) -> bool {
        // For now, allow all types as future return types
        // In a full implementation, we'd check for specific constraints
        true
    }
    
    /// Check if a type implements Send
    fn is_send_type(&self, _ty: &Type) -> bool {
        // For now, assume all types are Send
        // In a full implementation, we'd have proper trait checking
        true
    }
    
    /// Check if a type implements Sync
    
    
    /// Get future information
    pub fn get_future(&self, name: &str) -> Option<&FutureInfo> {
        self.futures.get(name)
    }
    
    /// Get channel information
    pub fn get_channel(&self, name: &str) -> Option<&ChannelTypeInfo> {
        self.channel_types.get(name)
    }
    
    /// Get actor information
    pub fn get_actor(&self, name: &str) -> Option<&ActorTypeInfo> {
        self.actor_types.get(name)
    }
}

impl ThreadSafetyAnalyzer {
    pub fn new() -> Self {
        Self {
            send_required: HashMap::new(),
            shared_data: HashMap::new(),
        }
    }
    
    /// Analyze a captured variable in an async context
    pub fn analyze_capture(&mut self, capture: &str, _future_info: &FutureInfo) -> Result<(), SemanticError> {
        // Mark the captured variable as needing Send
        self.send_required.insert(capture.to_string(), true);
        Ok(())
    }
    
    /// Analyze shared data access
    pub fn analyze_shared_data(
        &mut self,
        name: String,
        data_type: Type,
        access_pattern: AccessPattern,
    ) -> SynchronizationType {
        let sync_type = match access_pattern {
            AccessPattern::ReadOnly => SynchronizationType::None,
            AccessPattern::WriteOnly => SynchronizationType::Mutex,
            AccessPattern::ReadWrite => SynchronizationType::RwLock,
            AccessPattern::WriteHeavy => SynchronizationType::Mutex,
            AccessPattern::ReadHeavy => SynchronizationType::RwLock,
        };
        
        let info = SharedDataInfo {
            data_type,
            access_pattern,
            synchronization: sync_type.clone(),
        };
        
        self.shared_data.insert(name, info);
        sync_type
    }
}

impl Default for ConcurrencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ThreadSafetyAnalyzer {
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
    fn test_concurrency_manager_creation() {
        let manager = ConcurrencyManager::new();
        assert!(manager.runtime.is_idle());
    }
    
    #[test]
    fn test_concurrency_analyzer() {
        let mut analyzer = ConcurrencyAnalyzer::new();
        
        let future_info = FutureInfo {
            name: "test_future".to_string(),
            return_type: Type::primitive(PrimitiveType::Integer),
            is_send: true,
            is_sync: false,
            captures: vec!["x".to_string()],
        };
        
        assert!(analyzer.analyze_future("test_future".to_string(), future_info).is_ok());
        assert!(analyzer.get_future("test_future").is_some());
    }
    
    #[test]
    fn test_channel_type_analysis() {
        let mut analyzer = ConcurrencyAnalyzer::new();
        
        let channel_info = ChannelTypeInfo {
            sender_type: Type::primitive(PrimitiveType::String),
            receiver_type: Type::primitive(PrimitiveType::String),
            message_type: Type::primitive(PrimitiveType::String),
            is_bounded: true,
            capacity: Some(100),
        };
        
        assert!(analyzer.analyze_channel("test_channel".to_string(), channel_info).is_ok());
        assert!(analyzer.get_channel("test_channel").is_some());
    }
    
    #[test]
    fn test_thread_safety_analysis() {
        let mut analyzer = ThreadSafetyAnalyzer::new();
        
        let sync_type = analyzer.analyze_shared_data(
            "shared_counter".to_string(),
            Type::primitive(PrimitiveType::Integer),
            AccessPattern::ReadWrite,
        );
        
        assert_eq!(sync_type, SynchronizationType::RwLock);
    }
    
    #[test]
    fn test_access_pattern_synchronization() {
        let mut analyzer = ThreadSafetyAnalyzer::new();
        
        assert_eq!(
            analyzer.analyze_shared_data(
                "readonly".to_string(),
                Type::primitive(PrimitiveType::String),
                AccessPattern::ReadOnly,
            ),
            SynchronizationType::None
        );
        
        assert_eq!(
            analyzer.analyze_shared_data(
                "writeonly".to_string(),
                Type::primitive(PrimitiveType::String),
                AccessPattern::WriteOnly,
            ),
            SynchronizationType::Mutex
        );
        
        assert_eq!(
            analyzer.analyze_shared_data(
                "readheavy".to_string(),
                Type::primitive(PrimitiveType::String),
                AccessPattern::ReadHeavy,
            ),
            SynchronizationType::RwLock
        );
    }
}