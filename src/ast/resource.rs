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

//! Resource Management AST Nodes for LLM-First Language
//!
//! This module defines AST nodes for explicit resource management,
//! ensuring deterministic allocation and cleanup.

use crate::ast::{Block, Expression, Identifier, TypeSpecifier};
use crate::error::SourceLocation;
use serde::{Deserialize, Serialize};

/// Resource scope construct for explicit resource management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceScope {
    /// Unique identifier for this scope
    pub scope_id: String,

    /// Resources to acquire
    pub resources: Vec<ResourceAcquisition>,

    /// Invariants that must hold within the scope
    pub invariants: Vec<String>,

    /// Body to execute with resources
    pub body: Block,

    /// Whether cleanup is guaranteed
    pub cleanup_guaranteed: bool,

    /// Cleanup order strategy
    pub cleanup_order: CleanupOrder,

    /// Source location
    pub source_location: SourceLocation,
}

/// Resource acquisition specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAcquisition {
    /// Resource type (e.g., "tcp_socket", "memory_buffer", "file_handle")
    pub resource_type: String,

    /// Variable to bind the resource to
    pub binding: Identifier,

    /// Acquisition expression
    pub acquisition: Expression,

    /// Cleanup function/method
    pub cleanup: CleanupSpecification,

    /// Optional type specification
    pub type_spec: Option<TypeSpecifier>,

    /// Resource-specific parameters
    pub parameters: Vec<ResourceParameter>,
}

/// Cleanup specification for a resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanupSpecification {
    /// Function to call for cleanup
    Function { name: String, pass_resource: bool },

    /// Method to call on the resource
    Method { name: String },

    /// Custom cleanup expression
    Expression(Expression),

    /// Automatic cleanup (runtime handles it)
    Automatic,
}

/// Resource-specific parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceParameter {
    pub name: String,
    pub value: Expression,
}

/// Order for resource cleanup
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CleanupOrder {
    /// Reverse order of acquisition (LIFO)
    ReverseAcquisition,

    /// Same order as acquisition (FIFO)
    ForwardAcquisition,

    /// Dependency-based order
    DependencyBased,

    /// Parallel cleanup where possible
    Parallel,
}

/// Resource usage contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContract {
    /// Target function or scope
    pub target: String,

    /// Maximum memory in MB
    pub max_memory_mb: Option<u64>,

    /// Maximum file handles
    pub max_file_handles: Option<u32>,

    /// Maximum execution time in ms
    pub max_execution_time_ms: Option<u64>,

    /// Maximum network bandwidth in KB/s
    pub max_bandwidth_kbps: Option<u64>,

    /// Maximum CPU cores
    pub max_cpu_cores: Option<u32>,

    /// Maximum threads
    pub max_threads: Option<u32>,

    /// Enforcement mechanism
    pub enforcement: ResourceEnforcement,

    /// Source location
    pub source_location: SourceLocation,
}

/// Resource enforcement strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResourceEnforcement {
    /// Just monitor and log
    Monitor,

    /// Warn when approaching limits
    Warn { threshold_percent: u8 },

    /// Hard enforce with runtime errors
    Enforce,

    /// Enforce with graceful degradation
    GracefulDegrade,

    /// Custom enforcement
    Custom,
}

/// Resource pool for managing shared resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Pool name
    pub name: String,

    /// Resource type in the pool
    pub resource_type: String,

    /// Minimum pool size
    pub min_size: u32,

    /// Maximum pool size
    pub max_size: u32,

    /// Pool initialization
    pub initialization: PoolInitialization,

    /// Acquisition timeout in ms
    pub acquisition_timeout_ms: Option<u64>,

    /// Resource validation before reuse
    pub validation: Option<Expression>,

    /// Reset function for resource reuse
    pub reset_function: Option<String>,
}

/// Pool initialization strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolInitialization {
    /// Create all resources upfront
    Eager,

    /// Create resources on demand
    Lazy,

    /// Create initial set, then on demand
    Hybrid { initial_size: u32 },
}

/// Resource lifecycle hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLifecycle {
    /// Called before acquisition
    pub pre_acquire: Option<Expression>,

    /// Called after successful acquisition
    pub post_acquire: Option<Expression>,

    /// Called before release
    pub pre_release: Option<Expression>,

    /// Called after release
    pub post_release: Option<Expression>,

    /// Called on acquisition failure
    pub on_acquire_failure: Option<Expression>,

    /// Called on release failure
    pub on_release_failure: Option<Expression>,
}

/// Resource tracking information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTracking {
    /// Unique resource ID
    pub resource_id: String,

    /// Resource type
    pub resource_type: String,

    /// Acquisition timestamp
    pub acquired_at: u64,

    /// Acquisition location
    pub acquired_location: SourceLocation,

    /// Current owner scope
    pub owner_scope: String,

    /// Usage statistics
    pub usage_stats: ResourceUsageStats,
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageStats {
    /// Number of times accessed
    pub access_count: u64,

    /// Last access timestamp
    pub last_accessed: Option<u64>,

    /// Total time held in ms
    pub total_hold_time_ms: u64,

    /// Peak memory usage if applicable
    pub peak_memory_bytes: Option<u64>,

    /// Network bytes transferred if applicable
    pub network_bytes: Option<NetworkStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connection_count: u32,
}

/// Resource scope builder for ergonomic construction
pub struct ResourceScopeBuilder {
    scope_id: String,
    resources: Vec<ResourceAcquisition>,
    invariants: Vec<String>,
    cleanup_guaranteed: bool,
    cleanup_order: CleanupOrder,
}

impl ResourceScopeBuilder {
    pub fn new(scope_id: String) -> Self {
        Self {
            scope_id,
            resources: Vec::new(),
            invariants: Vec::new(),
            cleanup_guaranteed: true,
            cleanup_order: CleanupOrder::ReverseAcquisition,
        }
    }

    pub fn add_resource(mut self, resource: ResourceAcquisition) -> Self {
        self.resources.push(resource);
        self
    }

    pub fn add_invariant(mut self, invariant: String) -> Self {
        self.invariants.push(invariant);
        self
    }

    pub fn cleanup_order(mut self, order: CleanupOrder) -> Self {
        self.cleanup_order = order;
        self
    }

    pub fn build(self, body: Block, location: SourceLocation) -> ResourceScope {
        ResourceScope {
            scope_id: self.scope_id,
            resources: self.resources,
            invariants: self.invariants,
            body,
            cleanup_guaranteed: self.cleanup_guaranteed,
            cleanup_order: self.cleanup_order,
            source_location: location,
        }
    }
}

/// Helper functions for resource management
impl ResourceScope {
    /// Get all resource bindings
    pub fn get_bindings(&self) -> Vec<&Identifier> {
        self.resources.iter().map(|r| &r.binding).collect()
    }

    /// Check if scope uses a specific resource type
    pub fn uses_resource_type(&self, resource_type: &str) -> bool {
        self.resources
            .iter()
            .any(|r| r.resource_type == resource_type)
    }

    /// Get cleanup functions in order
    pub fn get_cleanup_sequence(&self) -> Vec<&CleanupSpecification> {
        match self.cleanup_order {
            CleanupOrder::ReverseAcquisition => {
                self.resources.iter().rev().map(|r| &r.cleanup).collect()
            }
            CleanupOrder::ForwardAcquisition => self.resources.iter().map(|r| &r.cleanup).collect(),
            _ => {
                // For now, default to reverse order for other strategies
                self.resources.iter().rev().map(|r| &r.cleanup).collect()
            }
        }
    }
}

/// Common resource types
pub mod resource_types {
    pub const MEMORY_BUFFER: &str = "memory_buffer";
    pub const FILE_HANDLE: &str = "file_handle";
    pub const TCP_SOCKET: &str = "tcp_socket";
    pub const UDP_SOCKET: &str = "udp_socket";
    pub const MUTEX: &str = "mutex";
    pub const SEMAPHORE: &str = "semaphore";
    pub const THREAD: &str = "thread";
    pub const DATABASE_CONNECTION: &str = "database_connection";
    pub const HTTP_CLIENT: &str = "http_client";
    pub const TIMER: &str = "timer";
}

/// Common cleanup functions
pub mod cleanup_functions {
    pub const FREE: &str = "aether_free";
    pub const CLOSE: &str = "close";
    pub const SOCKET_CLOSE: &str = "socket_close";
    pub const FILE_CLOSE: &str = "file_close";
    pub const UNLOCK: &str = "unlock";
    pub const RELEASE: &str = "release";
    pub const JOIN: &str = "thread_join";
    pub const DISCONNECT: &str = "disconnect";
    pub const CANCEL: &str = "cancel";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_scope_builder() {
        let file_resource = ResourceAcquisition {
            resource_type: resource_types::FILE_HANDLE.to_string(),
            binding: Identifier::new("file".to_string(), SourceLocation::unknown()),
            acquisition: Expression::FunctionCall {
                call: crate::ast::FunctionCall {
                    function_reference: crate::ast::FunctionReference::Local {
                        name: Identifier::new("open_file".to_string(), SourceLocation::unknown()),
                    },
                    explicit_type_arguments: vec![],
                    arguments: vec![],
                    variadic_arguments: vec![],
                },
                source_location: SourceLocation::unknown(),
            },
            cleanup: CleanupSpecification::Function {
                name: cleanup_functions::FILE_CLOSE.to_string(),
                pass_resource: true,
            },
            type_spec: None,
            parameters: vec![],
        };

        let scope = ResourceScopeBuilder::new("file_operation_001".to_string())
            .add_resource(file_resource)
            .add_invariant("file != NULL".to_string())
            .cleanup_order(CleanupOrder::ReverseAcquisition)
            .build(
                Block {
                    statements: vec![],
                    source_location: SourceLocation::unknown(),
                },
                SourceLocation::unknown(),
            );

        assert_eq!(scope.scope_id, "file_operation_001");
        assert_eq!(scope.resources.len(), 1);
        assert_eq!(scope.invariants.len(), 1);
        assert!(scope.cleanup_guaranteed);
    }

    #[test]
    fn test_cleanup_sequence() {
        let mut resources = vec![];

        for i in 0..3 {
            resources.push(ResourceAcquisition {
                resource_type: format!("resource_{}", i),
                binding: Identifier::new(format!("r{}", i), SourceLocation::unknown()),
                acquisition: Expression::NullLiteral {
                    source_location: SourceLocation::unknown(),
                },
                cleanup: CleanupSpecification::Function {
                    name: format!("cleanup_{}", i),
                    pass_resource: true,
                },
                type_spec: None,
                parameters: vec![],
            });
        }

        let scope = ResourceScope {
            scope_id: "test".to_string(),
            resources,
            invariants: vec![],
            body: Block {
                statements: vec![],
                source_location: SourceLocation::unknown(),
            },
            cleanup_guaranteed: true,
            cleanup_order: CleanupOrder::ReverseAcquisition,
            source_location: SourceLocation::unknown(),
        };

        let cleanup_seq = scope.get_cleanup_sequence();
        assert_eq!(cleanup_seq.len(), 3);

        // Check reverse order
        if let CleanupSpecification::Function { name, .. } = cleanup_seq[0] {
            assert_eq!(name, "cleanup_2");
        }
    }
}
