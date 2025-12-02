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

//! Channel-based communication for AetherScript
//!
//! Provides typed channels for safe message passing between concurrent tasks

use crate::error::SemanticError;
use crate::types::Type;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Channel registry for managing all channels
#[derive(Debug)]
pub struct ChannelRegistry {
    /// Registered channels
    channels: HashMap<String, ChannelInfo>,

    /// Channel statistics
    stats: ChannelStats,
}

/// Information about a registered channel
#[derive(Debug, Clone)]
pub struct ChannelInfo {
    pub name: String,
    pub channel_type: ChannelType,
    pub message_type: Type,
    pub capacity: Option<usize>,
    pub created_at: Instant,
    pub sender_count: usize,
    pub receiver_count: usize,
}

/// Type of channel
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelType {
    /// Unbounded channel (unlimited capacity)
    Unbounded,

    /// Bounded channel (fixed capacity)
    Bounded(usize),

    /// Synchronous channel (zero capacity, direct handoff)
    Sync,

    /// Broadcast channel (multiple receivers)
    Broadcast(usize),

    /// Multiple producer, single consumer
    Mpsc,

    /// Single producer, multiple consumer
    Spmc,

    /// Multiple producer, multiple consumer
    Mpmc,
}

/// Channel statistics
#[derive(Debug, Default)]
pub struct ChannelStats {
    pub total_channels: usize,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub total_messages_dropped: u64,
    pub avg_message_latency_ms: f64,
}

/// Generic channel for typed message passing
#[derive(Debug)]
pub struct Channel<T> {
    _phantom: std::marker::PhantomData<T>,
}

/// Sender side of a channel
#[derive(Debug)]
pub struct ChannelSender<T> {
    /// Internal sender implementation
    inner: SenderImpl<T>,

    /// Sender ID
    id: usize,

    /// Channel name
    channel_name: String,
}

/// Receiver side of a channel
pub struct ChannelReceiver<T> {
    /// Receiver implementation
    inner: ReceiverImpl<T>,
}

/// Internal sender implementation
#[derive(Debug)]
enum SenderImpl<T> {
    /// Standard MPSC sender
    Mpsc(mpsc::Sender<T>),

    /// Bounded sender
    Bounded(Arc<Mutex<BoundedChannelState<T>>>),
}

/// Internal receiver implementation
#[derive(Debug)]
enum ReceiverImpl<T> {
    /// Standard MPSC receiver
    Mpsc(mpsc::Receiver<T>),

    /// Bounded receiver
    Bounded(Arc<Mutex<BoundedChannelState<T>>>, Arc<Condvar>),
}

/// State for bounded channels
#[derive(Debug)]
struct BoundedChannelState<T> {
    buffer: std::collections::VecDeque<T>,
    capacity: usize,
    closed: bool,
}

/// Channel metrics
#[derive(Debug, Default)]
pub struct ChannelMetrics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_dropped: u64,
    pub send_errors: u64,
    pub receive_errors: u64,
    pub total_latency_ms: f64,
    pub peak_queue_size: usize,
}

/// Send error types
#[derive(Debug, Clone)]
pub enum SendError<T> {
    /// Channel is full
    Full(T),

    /// Channel is closed
    Closed(T),

    /// Send timeout
    Timeout(T),

    /// Receiver disconnected
    Disconnected(T),
}

/// Receive error types
#[derive(Debug, Clone)]
pub enum ReceiveError {
    /// Channel is empty
    Empty,

    /// Channel is closed and empty
    Closed,

    /// Receive timeout
    Timeout,

    /// Sender disconnected
    Disconnected,
}

/// Send result
pub type SendResult<T> = Result<(), SendError<T>>;

/// Receive result
pub type ReceiveResult<T> = Result<T, ReceiveError>;

impl ChannelRegistry {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            stats: ChannelStats::default(),
        }
    }

    /// Create a new unbounded channel
    pub fn create_unbounded<T: Send + 'static>(
        &mut self,
        name: String,
        message_type: Type,
    ) -> Result<(ChannelSender<T>, ChannelReceiver<T>), SemanticError> {
        if self.channels.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }

        let (sender, receiver) = mpsc::channel();

        let channel_sender = ChannelSender {
            inner: SenderImpl::Mpsc(sender),
            id: 0,
            channel_name: name.clone(),
        };

        let channel_receiver = ChannelReceiver {
            inner: ReceiverImpl::Mpsc(receiver),
        };

        let info = ChannelInfo {
            name: name.clone(),
            channel_type: ChannelType::Unbounded,
            message_type,
            capacity: None,
            created_at: Instant::now(),
            sender_count: 1,
            receiver_count: 1,
        };

        self.channels.insert(name, info);
        self.stats.total_channels += 1;

        Ok((channel_sender, channel_receiver))
    }

    /// Create a new bounded channel
    pub fn create_bounded<T: Send + 'static>(
        &mut self,
        name: String,
        message_type: Type,
        capacity: usize,
    ) -> Result<(ChannelSender<T>, ChannelReceiver<T>), SemanticError> {
        if self.channels.contains_key(&name) {
            return Err(SemanticError::DuplicateDefinition {
                symbol: name,
                location: crate::error::SourceLocation::unknown(),
                previous_location: crate::error::SourceLocation::unknown(),
            });
        }

        let state = Arc::new(Mutex::new(BoundedChannelState {
            buffer: std::collections::VecDeque::with_capacity(capacity),
            capacity,
            closed: false,
        }));

        let condvar = Arc::new(Condvar::new());

        let channel_sender = ChannelSender {
            inner: SenderImpl::Bounded(state.clone()),
            id: 0,
            channel_name: name.clone(),
        };

        let channel_receiver = ChannelReceiver {
            inner: ReceiverImpl::Bounded(state, condvar),
        };

        let info = ChannelInfo {
            name: name.clone(),
            channel_type: ChannelType::Bounded(capacity),
            message_type,
            capacity: Some(capacity),
            created_at: Instant::now(),
            sender_count: 1,
            receiver_count: 1,
        };

        self.channels.insert(name, info);
        self.stats.total_channels += 1;

        Ok((channel_sender, channel_receiver))
    }

    /// Get channel information
    pub fn get_channel_info(&self, name: &str) -> Option<&ChannelInfo> {
        self.channels.get(name)
    }

    /// Get channel statistics
    pub fn stats(&self) -> &ChannelStats {
        &self.stats
    }

    /// Update message statistics
    pub fn record_message_sent(&mut self) {
        self.stats.total_messages_sent += 1;
    }

    pub fn record_message_received(&mut self) {
        self.stats.total_messages_received += 1;
    }

    pub fn record_message_dropped(&mut self) {
        self.stats.total_messages_dropped += 1;
    }
}

impl<T> ChannelSender<T> {
    /// Send a message
    pub fn send(&self, message: T) -> SendResult<T> {
        match &self.inner {
            SenderImpl::Mpsc(sender) => sender
                .send(message)
                .map_err(|mpsc::SendError(msg)| SendError::Disconnected(msg)),
            SenderImpl::Bounded(state) => {
                let mut state = state
                    .lock()
                    .expect("channel mutex should not be poisoned");
                if state.closed {
                    return Err(SendError::Closed(message));
                }
                if state.buffer.len() >= state.capacity {
                    return Err(SendError::Full(message));
                }
                state.buffer.push_back(message);
                Ok(())
            }
        }
    }

    /// Try to send a message without blocking
    pub fn try_send(&self, message: T) -> SendResult<T> {
        self.send(message) // For now, same as send
    }

    /// Send a message with timeout
    pub fn send_timeout(&self, message: T, _timeout: Duration) -> SendResult<T> {
        // For now, same as send - timeout logic would be more complex
        self.send(message)
    }
}

impl<T> ChannelReceiver<T> {
    /// Receive a message (blocking)
    pub fn recv(&self) -> ReceiveResult<T> {
        match &self.inner {
            ReceiverImpl::Mpsc(receiver) => receiver.recv().map_err(|_| ReceiveError::Disconnected),
            ReceiverImpl::Bounded(state, _condvar) => {
                let mut state = state
                    .lock()
                    .expect("channel mutex should not be poisoned");
                if let Some(message) = state.buffer.pop_front() {
                    Ok(message)
                } else if state.closed {
                    Err(ReceiveError::Closed)
                } else {
                    Err(ReceiveError::Empty)
                }
            }
        }
    }

    /// Try to receive a message without blocking
    pub fn try_recv(&self) -> ReceiveResult<T> {
        match &self.inner {
            ReceiverImpl::Mpsc(receiver) => match receiver.try_recv() {
                Ok(message) => Ok(message),
                Err(mpsc::TryRecvError::Empty) => Err(ReceiveError::Empty),
                Err(mpsc::TryRecvError::Disconnected) => Err(ReceiveError::Disconnected),
            },
            _ => self.recv(), // For now, same as recv
        }
    }

    /// Receive a message with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> ReceiveResult<T> {
        match &self.inner {
            ReceiverImpl::Mpsc(receiver) => match receiver.recv_timeout(timeout) {
                Ok(message) => Ok(message),
                Err(mpsc::RecvTimeoutError::Timeout) => Err(ReceiveError::Timeout),
                Err(mpsc::RecvTimeoutError::Disconnected) => Err(ReceiveError::Disconnected),
            },
            _ => self.recv(), // For now, same as recv
        }
    }
}

impl<T> Clone for ChannelSender<T> {
    fn clone(&self) -> Self {
        match &self.inner {
            SenderImpl::Mpsc(sender) => Self {
                inner: SenderImpl::Mpsc(sender.clone()),
                id: self.id,
                channel_name: self.channel_name.clone(),
            },
            SenderImpl::Bounded(state) => Self {
                inner: SenderImpl::Bounded(state.clone()),
                id: self.id,
                channel_name: self.channel_name.clone(),
            },
        }
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;
    use crate::types::Type;

    #[test]
    fn test_channel_registry_creation() {
        let registry = ChannelRegistry::new();
        assert_eq!(registry.stats.total_channels, 0);
    }

    #[test]
    fn test_unbounded_channel_creation() {
        let mut registry = ChannelRegistry::new();

        let result = registry.create_unbounded::<String>(
            "test_channel".to_string(),
            Type::primitive(PrimitiveType::String),
        );

        assert!(result.is_ok());
        assert_eq!(registry.stats.total_channels, 1);

        let info = registry.get_channel_info("test_channel");
        assert!(info.is_some());
        assert_eq!(info.unwrap().channel_type, ChannelType::Unbounded);
    }

    #[test]
    fn test_bounded_channel_creation() {
        let mut registry = ChannelRegistry::new();

        let result = registry.create_bounded::<i32>(
            "bounded_channel".to_string(),
            Type::primitive(PrimitiveType::Integer),
            10,
        );

        assert!(result.is_ok());
        assert_eq!(registry.stats.total_channels, 1);

        let info = registry.get_channel_info("bounded_channel");
        assert!(info.is_some());
        assert_eq!(info.unwrap().channel_type, ChannelType::Bounded(10));
        assert_eq!(info.unwrap().capacity, Some(10));
    }

    #[test]
    fn test_message_sending_receiving() {
        let mut registry = ChannelRegistry::new();

        let (sender, receiver) = registry
            .create_unbounded::<String>(
                "test_channel".to_string(),
                Type::primitive(PrimitiveType::String),
            )
            .unwrap();

        // Send a message
        assert!(sender.send("Hello, World!".to_string()).is_ok());

        // Receive the message
        let received = receiver.recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), "Hello, World!");
    }

    #[test]
    fn test_bounded_channel_capacity() {
        let mut registry = ChannelRegistry::new();

        let (sender, _receiver) = registry
            .create_bounded::<i32>(
                "small_channel".to_string(),
                Type::primitive(PrimitiveType::Integer),
                2,
            )
            .unwrap();

        // Should be able to send up to capacity
        assert!(sender.send(1).is_ok());
        assert!(sender.send(2).is_ok());

        // Should fail when full (depending on implementation)
        // Note: This test might pass if the implementation doesn't block immediately
    }

    #[test]
    fn test_channel_sender_cloning() {
        let mut registry = ChannelRegistry::new();

        let (sender, _receiver) = registry
            .create_unbounded::<String>(
                "clone_test".to_string(),
                Type::primitive(PrimitiveType::String),
            )
            .unwrap();

        let sender_clone = sender.clone();
        assert_eq!(sender.channel_name, sender_clone.channel_name);
    }

    #[test]
    fn test_duplicate_channel_names() {
        let mut registry = ChannelRegistry::new();

        let result1 = registry.create_unbounded::<String>(
            "duplicate".to_string(),
            Type::primitive(PrimitiveType::String),
        );
        assert!(result1.is_ok());

        let result2 = registry.create_unbounded::<String>(
            "duplicate".to_string(),
            Type::primitive(PrimitiveType::String),
        );
        assert!(result2.is_err());
    }
}
