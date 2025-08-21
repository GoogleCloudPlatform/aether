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

//! Actor model implementation for AetherScript
//!
//! Provides actor-based concurrency with message passing and supervision

use crate::error::SemanticError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Actor system for managing all actors
#[derive(Debug)]
pub struct ActorSystem {
    /// System state
    state: ActorSystemState,
    
    /// Registry of all actors
    actors: Arc<Mutex<HashMap<ActorId, ActorInfo>>>,
    
    /// Supervisor hierarchy
    supervisors: Arc<Mutex<HashMap<ActorId, SupervisorInfo>>>,
    
    /// Message dispatcher
    dispatcher: MessageDispatcher,
    
    /// System statistics
    stats: ActorSystemStats,
    
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
}

/// Actor system state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorSystemState {
    Idle,
    Starting,
    Running,
    Stopping,
    Stopped,
}

/// Configuration for the actor system
#[derive(Debug, Clone)]
pub struct ActorSystemConfig {
    /// Maximum number of actors
    pub max_actors: usize,
    
    /// Default mailbox size
    pub default_mailbox_size: usize,
    
    /// Actor startup timeout
    pub actor_startup_timeout: Duration,
    
    /// Message processing timeout
    pub message_timeout: Duration,
    
    /// Enable supervision
    pub enable_supervision: bool,
}

/// Actor system statistics
#[derive(Debug, Default)]
pub struct ActorSystemStats {
    /// Total actors created
    pub actors_created: AtomicU64,
    
    /// Total actors destroyed
    pub actors_destroyed: AtomicU64,
    
    /// Total messages sent
    pub messages_sent: AtomicU64,
    
    /// Total messages processed
    pub messages_processed: AtomicU64,
    
    /// Total supervision restarts
    pub supervision_restarts: AtomicU64,
    
    /// System uptime
    pub uptime_start: Option<Instant>,
}

/// Actor identifier
pub type ActorId = u64;

/// Individual actor
#[derive(Debug)]
pub struct Actor {
}

/// Actor state trait
pub trait ActorState: Send + std::fmt::Debug {
    /// Handle incoming message
    fn handle_message(&mut self, message: ActorMessage) -> ActorResult;
    
    /// Initialize actor
    fn initialize(&mut self) -> ActorResult {
        ActorResult::Success
    }
    
    /// Cleanup on shutdown
    fn cleanup(&mut self) -> ActorResult {
        ActorResult::Success
    }
}

/// Actor handle for sending messages
#[derive(Debug, Clone)]
pub struct ActorHandle {
    /// Actor ID
    id: ActorId,
    
    /// Actor name
    name: String,
    
    /// Message sender
    sender: mpsc::Sender<ActorMessage>,
}

/// Actor information stored in registry
#[derive(Debug, Clone)]
pub struct ActorInfo {
    pub id: ActorId,
    pub name: String,
    pub actor_type: String,
    pub parent: Option<ActorId>,
    pub children: Vec<ActorId>,
    pub created_at: Instant,
    pub state: ActorLifecycleState,
    pub message_count: u64,
    pub restart_count: u32,
}

/// Actor lifecycle state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorLifecycleState {
    Created,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    Restarting,
}

/// Supervisor information
#[derive(Debug, Clone)]
pub struct SupervisorInfo {
    pub id: ActorId,
    pub strategy: SupervisionStrategy,
    pub max_restarts: u32,
    pub restart_window: Duration,
    pub children: Vec<ActorId>,
}

/// Supervision strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionStrategy {
    /// Restart only the failed child
    OneForOne,
    
    /// Restart all children when one fails
    OneForAll,
    
    /// Restart failed child and all children started after it
    RestForOne,
    
    /// Stop all children and the supervisor
    Escalate,
}

/// Message dispatcher
#[derive(Debug)]
pub struct MessageDispatcher {
    /// Message routing table
    routes: Arc<Mutex<HashMap<ActorId, mpsc::Sender<ActorMessage>>>>,
    
    /// Dead letter queue
    dead_letters: Arc<Mutex<Vec<DeadLetter>>>,
    
    /// Dispatcher statistics
    stats: DispatcherStats,
}

/// Dead letter (undeliverable message)
#[derive(Debug, Clone)]
pub struct DeadLetter {
    pub message: ActorMessage,
    pub target: ActorId,
    pub reason: String,
    pub timestamp: Instant,
}

/// Dispatcher statistics
#[derive(Debug, Default)]
pub struct DispatcherStats {
    pub messages_routed: AtomicU64,
    pub dead_letters: AtomicU64,
    pub routing_errors: AtomicU64,
}

/// Actor message
#[derive(Debug, Clone)]
pub struct ActorMessage {
    /// Message ID
    pub id: u64,
    
    /// Sender actor ID
    pub sender: Option<ActorId>,
    
    /// Target actor ID
    pub target: ActorId,
    
    /// Message payload
    pub payload: MessagePayload,
    
    /// Message timestamp
    pub timestamp: Instant,
    
    /// Message priority
    pub priority: MessagePriority,
}

/// Message payload types
#[derive(Debug, Clone)]
pub enum MessagePayload {
    /// Text message
    Text(String),
    
    /// Integer value
    Integer(i64),
    
    /// Float value
    Float(f64),
    
    /// Boolean value
    Boolean(bool),
    
    /// Binary data
    Binary(Vec<u8>),
    
    /// Structured data
    Struct(HashMap<String, MessagePayload>),
    
    /// System message
    System(SystemMessage),
}

/// System messages
#[derive(Debug, Clone)]
pub enum SystemMessage {
    /// Start the actor
    Start,
    
    /// Stop the actor
    Stop,
    
    /// Restart the actor
    Restart,
    
    /// Query actor status
    Status,
    
    /// Supervision directive
    Supervise(SupervisionDirective),
}

/// Supervision directive
#[derive(Debug, Clone)]
pub enum SupervisionDirective {
    Resume,
    Restart,
    Stop,
    Escalate,
}

/// Message priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    System = 3,
}

/// Actor execution result
#[derive(Debug, Clone)]
pub enum ActorResult {
    /// Actor operation succeeded
    Success,
    
    /// Actor operation failed
    Failure(String),
    
    /// Actor should be restarted
    Restart,
    
    /// Actor should be stopped
    Stop,
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {
            state: ActorSystemState::Idle,
            actors: Arc::new(Mutex::new(HashMap::new())),
            supervisors: Arc::new(Mutex::new(HashMap::new())),
            dispatcher: MessageDispatcher::new(),
            stats: ActorSystemStats::default(),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Start the actor system
    pub fn start(&mut self) -> Result<(), SemanticError> {
        if self.state != ActorSystemState::Idle {
            return Err(SemanticError::Internal {
                message: format!("Actor system already started, current state: {:?}", self.state),
            });
        }
        
        self.state = ActorSystemState::Starting;
        self.shutdown.store(false, Ordering::Relaxed);
        self.stats.uptime_start = Some(Instant::now());
        self.state = ActorSystemState::Running;
        
        Ok(())
    }
    
    /// Stop the actor system
    pub fn stop(&mut self) -> Result<(), SemanticError> {
        if self.state != ActorSystemState::Running {
            return Err(SemanticError::Internal {
                message: format!("Actor system not running, current state: {:?}", self.state),
            });
        }
        
        self.state = ActorSystemState::Stopping;
        self.shutdown.store(true, Ordering::Relaxed);
        
        // Stop all actors
        self.stop_all_actors()?;
        
        self.state = ActorSystemState::Stopped;
        Ok(())
    }
    
    /// Create a new actor
    pub fn create_actor<S>(&mut self, name: String, _state: S) -> Result<ActorHandle, SemanticError>
    where
        S: ActorState + Send + 'static,
    {
        if self.state != ActorSystemState::Running {
            return Err(SemanticError::Internal {
                message: "Actor system is not running".to_string(),
            });
        }
        
        let actor_id = self.next_actor_id();
        let (sender, receiver) = mpsc::channel();
        
        let handle = ActorHandle {
            id: actor_id,
            name: name.clone(),
            sender: sender.clone(),
        };
        
        // Register the actor with the dispatcher
        self.dispatcher.register_actor(actor_id, sender.clone())?;
        
        // Register the actor
        let info = ActorInfo {
            id: actor_id,
            name: name.clone(),
            actor_type: std::any::type_name::<S>().to_string(),
            parent: None,
            children: Vec::new(),
            created_at: Instant::now(),
            state: ActorLifecycleState::Created,
            message_count: 0,
            restart_count: 0,
        };
        
        {
            let mut actors = self.actors.lock().map_err(|_| SemanticError::Internal {
                message: "Failed to lock actors registry".to_string(),
            })?;
            actors.insert(actor_id, info);
        }
        
        // Register with dispatcher
        self.dispatcher.register_actor(actor_id, sender)?;
        
        self.stats.actors_created.fetch_add(1, Ordering::Relaxed);
        
        Ok(handle)
    }
    
    /// Create a supervisor actor
    pub fn create_supervisor(
        &mut self,
        name: String,
        strategy: SupervisionStrategy,
    ) -> Result<ActorHandle, SemanticError> {
        let supervisor_id = self.next_actor_id();
        
        // Create supervisor info
        let supervisor_info = SupervisorInfo {
            id: supervisor_id,
            strategy,
            max_restarts: 3,
            restart_window: Duration::from_secs(60),
            children: Vec::new(),
        };
        
        {
            let mut supervisors = self.supervisors.lock().map_err(|_| SemanticError::Internal {
                message: "Failed to lock supervisors registry".to_string(),
            })?;
            supervisors.insert(supervisor_id, supervisor_info);
        }
        
        // Create the supervisor as a regular actor with special state
        let supervisor_state = SupervisorState::new(strategy);
        self.create_actor(name, supervisor_state)
    }
    
    /// Send a message to an actor
    pub fn send_message(&mut self, message: ActorMessage) -> Result<(), SemanticError> {
        self.dispatcher.dispatch_message(message)
    }
    
    /// Get actor information
    pub fn get_actor_info(&self, actor_id: ActorId) -> Option<ActorInfo> {
        self.actors.lock().ok()?.get(&actor_id).cloned()
    }
    
    /// Get system statistics
    pub fn stats(&self) -> &ActorSystemStats {
        &self.stats
    }
    
    /// Check if system is running
    pub fn is_running(&self) -> bool {
        self.state == ActorSystemState::Running
    }
    
    fn next_actor_id(&mut self) -> ActorId {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    }
    
    fn stop_all_actors(&mut self) -> Result<(), SemanticError> {
        let actors = self.actors.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to lock actors registry".to_string(),
        })?;
        
        for (actor_id, _) in actors.iter() {
            let stop_message = ActorMessage {
                id: 0,
                sender: None,
                target: *actor_id,
                payload: MessagePayload::System(SystemMessage::Stop),
                timestamp: Instant::now(),
                priority: MessagePriority::System,
            };
            
            let _ = self.dispatcher.dispatch_message(stop_message);
        }
        
        Ok(())
    }
}

impl MessageDispatcher {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(Mutex::new(HashMap::new())),
            dead_letters: Arc::new(Mutex::new(Vec::new())),
            stats: DispatcherStats::default(),
        }
    }
    
    /// Register an actor with the dispatcher
    pub fn register_actor(&mut self, actor_id: ActorId, sender: mpsc::Sender<ActorMessage>) -> Result<(), SemanticError> {
        let mut routes = self.routes.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to lock routing table".to_string(),
        })?;
        
        routes.insert(actor_id, sender);
        Ok(())
    }
    
    /// Dispatch a message to an actor
    pub fn dispatch_message(&self, message: ActorMessage) -> Result<(), SemanticError> {
        let routes = self.routes.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to lock routing table".to_string(),
        })?;
        
        if let Some(sender) = routes.get(&message.target) {
            match sender.send(message.clone()) {
                Ok(()) => {
                    self.stats.messages_routed.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
                Err(_) => {
                    // Actor mailbox is full or disconnected, send to dead letters
                    self.send_to_dead_letters(message, "Mailbox full or disconnected".to_string())?;
                    self.stats.routing_errors.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                }
            }
        } else {
            // Actor not found, send to dead letters
            self.send_to_dead_letters(message, "Actor not found".to_string())?;
            self.stats.routing_errors.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }
    
    fn send_to_dead_letters(&self, message: ActorMessage, reason: String) -> Result<(), SemanticError> {
        let mut dead_letters = self.dead_letters.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to lock dead letters queue".to_string(),
        })?;
        
        let dead_letter = DeadLetter {
            target: message.target,
            message,
            reason,
            timestamp: Instant::now(),
        };
        
        dead_letters.push(dead_letter);
        self.stats.dead_letters.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
}

impl ActorHandle {
    /// Send a message to this actor
    pub fn send(&self, payload: MessagePayload) -> Result<(), SemanticError> {
        self.send_with_priority(payload, MessagePriority::Normal)
    }
    
    /// Send a high priority message to this actor
    pub fn send_high_priority(&self, payload: MessagePayload) -> Result<(), SemanticError> {
        self.send_with_priority(payload, MessagePriority::High)
    }
    
    /// Send a message with specific priority
    pub fn send_with_priority(&self, payload: MessagePayload, priority: MessagePriority) -> Result<(), SemanticError> {
        let message = ActorMessage {
            id: 0, // Would be generated properly
            sender: None,
            target: self.id,
            payload,
            timestamp: Instant::now(),
            priority,
        };
        
        self.sender.send(message).map_err(|_| SemanticError::Internal {
            message: format!("Failed to send message to actor {}", self.name),
        })?;
        
        Ok(())
    }
    
    /// Get actor ID
    pub fn id(&self) -> ActorId {
        self.id
    }
    
    /// Get actor name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Simple supervisor state implementation
#[derive(Debug)]
struct SupervisorState {
}

impl SupervisorState {
    fn new(strategy: SupervisionStrategy) -> Self {
        Self {}
    }
}

impl ActorState for SupervisorState {
    fn handle_message(&mut self, message: ActorMessage) -> ActorResult {
        match message.payload {
            MessagePayload::System(SystemMessage::Supervise(directive)) => {
                // Handle supervision directive
                match directive {
                    SupervisionDirective::Resume => ActorResult::Success,
                    SupervisionDirective::Restart => ActorResult::Restart,
                    SupervisionDirective::Stop => ActorResult::Stop,
                    SupervisionDirective::Escalate => ActorResult::Failure("Escalated".to_string()),
                }
            }
            _ => ActorResult::Success,
        }
    }
}

impl Default for ActorSystemConfig {
    fn default() -> Self {
        Self {
            max_actors: 10000,
            default_mailbox_size: 1000,
            actor_startup_timeout: Duration::from_secs(10),
            message_timeout: Duration::from_secs(5),
            enable_supervision: true,
        }
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ActorSystemStats {
    pub fn uptime(&self) -> Option<Duration> {
        self.uptime_start.map(|start| start.elapsed())
    }
    
    pub fn actors_created(&self) -> u64 {
        self.actors_created.load(Ordering::Relaxed)
    }
    
    pub fn actors_destroyed(&self) -> u64 {
        self.actors_destroyed.load(Ordering::Relaxed)
    }
    
    pub fn messages_sent(&self) -> u64 {
        self.messages_sent.load(Ordering::Relaxed)
    }
    
    pub fn messages_processed(&self) -> u64 {
        self.messages_processed.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug)]
    struct TestActor {
        messages_received: u32,
    }
    
    impl TestActor {
        fn new() -> Self {
            Self {
                messages_received: 0,
            }
        }
    }
    
    impl ActorState for TestActor {
        fn handle_message(&mut self, _message: ActorMessage) -> ActorResult {
            self.messages_received += 1;
            ActorResult::Success
        }
    }
    
    #[test]
    fn test_actor_system_creation() {
        let system = ActorSystem::new();
        assert_eq!(system.state, ActorSystemState::Idle);
        assert!(!system.is_running());
    }
    
    #[test]
    fn test_actor_system_start_stop() {
        let mut system = ActorSystem::new();
        assert!(system.start().is_ok());
        assert!(system.is_running());
        assert!(system.stop().is_ok());
        assert_eq!(system.state, ActorSystemState::Stopped);
    }
    
    #[test]
    fn test_actor_creation() {
        let mut system = ActorSystem::new();
        assert!(system.start().is_ok());
        
        let test_state = TestActor::new();
        let handle = system.create_actor("test_actor".to_string(), test_state);
        assert!(handle.is_ok());
        
        let handle = handle.unwrap();
        assert_eq!(handle.name(), "test_actor");
        assert!(handle.id() > 0);
    }
    
    #[test]
    fn test_message_dispatcher() {
        let dispatcher = MessageDispatcher::new();
        assert_eq!(dispatcher.stats.messages_routed.load(Ordering::Relaxed), 0);
    }
    
    #[test]
    fn test_actor_handle_messaging() {
        let mut system = ActorSystem::new();
        assert!(system.start().is_ok());
        
        let test_state = TestActor::new();
        let handle = system.create_actor("test_actor".to_string(), test_state).unwrap();
        
        // For now, just test that we can create an actor handle
        // Full messaging would require implementing the actor runtime
        assert_eq!(handle.name(), "test_actor");
        assert!(handle.id() > 0);
    }
    
    #[test]
    fn test_supervision_strategy() {
        assert!(SupervisionStrategy::OneForOne != SupervisionStrategy::OneForAll);
    }
    
    #[test]
    fn test_message_priority() {
        assert!(MessagePriority::System > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }
}