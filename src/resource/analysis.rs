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

//! Resource Analysis for Deterministic Resource Management
//!
//! This module analyzes resource usage patterns and ensures proper
//! resource lifecycle management in AetherScript programs.

use crate::ast::resource::*;
use crate::ast::{Block, Expression, Function, Statement};
use crate::error::{SemanticError, SourceLocation};
use std::collections::HashMap;

/// Resource analyzer for tracking and validating resource usage
pub struct ResourceAnalyzer {
    /// Active resource scopes
    active_scopes: Vec<ScopeInfo>,

    /// Resource tracking information
    resource_tracking: HashMap<String, ResourceTracking>,

    /// Resource contracts by function
    contracts: HashMap<String, ResourceContract>,

    /// Analysis results
    results: ResourceAnalysisResults,
}

/// Information about an active resource scope
#[derive(Debug, Clone)]
struct ScopeInfo {
    scope_id: String,
    resources: Vec<TrackedResource>,
}

/// Tracked resource within a scope
#[derive(Debug, Clone)]
struct TrackedResource {
    binding: String,
    resource_type: String,
    acquisition_location: SourceLocation,
    is_released: bool,
    usage_count: usize,
}

/// Results of resource analysis
#[derive(Debug, Clone, Default)]
pub struct ResourceAnalysisResults {
    /// Maximum resources held simultaneously
    pub max_concurrent_resources: usize,

    /// Resource leaks detected
    pub leaks: Vec<ResourceLeak>,

    /// Double-release errors
    pub double_releases: Vec<DoubleRelease>,

    /// Use-after-release errors
    pub use_after_release: Vec<UseAfterRelease>,

    /// Contract violations
    pub contract_violations: Vec<ContractViolation>,

    /// Resource usage patterns
    pub usage_patterns: HashMap<String, UsagePattern>,

    /// Suggested optimizations
    pub optimizations: Vec<ResourceOptimization>,
}

/// Resource leak detection
#[derive(Debug, Clone)]
pub struct ResourceLeak {
    pub resource_type: String,
    pub binding: String,
    pub acquisition_location: SourceLocation,
    pub scope_exit_location: SourceLocation,
}

/// Double release detection
#[derive(Debug, Clone)]
pub struct DoubleRelease {
    pub resource_type: String,
    pub binding: String,
    pub first_release: SourceLocation,
    pub second_release: SourceLocation,
}

/// Use after release detection
#[derive(Debug, Clone)]
pub struct UseAfterRelease {
    pub resource_type: String,
    pub binding: String,
    pub release_location: SourceLocation,
    pub use_location: SourceLocation,
}

/// Contract violation
#[derive(Debug, Clone)]
pub struct ContractViolation {
    pub contract_type: String,
    pub limit: u64,
    pub actual: u64,
    pub location: SourceLocation,
}

/// Resource usage pattern
#[derive(Debug, Clone)]
pub struct UsagePattern {
    pub resource_type: String,
    pub avg_hold_time: f64,
    pub max_hold_time: u64,
    pub access_frequency: f64,
    pub typical_count: usize,
}

/// Resource optimization suggestion
#[derive(Debug, Clone)]
pub struct ResourceOptimization {
    pub optimization_type: OptimizationType,
    pub resource_type: String,
    pub location: SourceLocation,
    pub description: String,
    pub estimated_benefit: OptimizationBenefit,
}

#[derive(Debug, Clone)]
pub enum OptimizationType {
    /// Convert to resource pool
    UsePool,
    /// Reduce scope lifetime
    ReduceScope,
    /// Combine acquisitions
    BatchAcquisition,
    /// Early release
    EarlyRelease,
    /// Lazy acquisition
    LazyAcquisition,
}

#[derive(Debug, Clone)]
pub struct OptimizationBenefit {
    pub memory_saved_mb: Option<f64>,
    pub latency_reduced_ms: Option<f64>,
    pub resource_count_reduced: Option<usize>,
}

impl ResourceAnalyzer {
    pub fn new() -> Self {
        Self {
            active_scopes: Vec::new(),
            resource_tracking: HashMap::new(),
            contracts: HashMap::new(),
            results: ResourceAnalysisResults::default(),
        }
    }

    /// Analyze a function for resource usage
    pub fn analyze_function(&mut self, function: &Function) -> Result<(), SemanticError> {
        // Check if function has resource contract
        if let Some(contract) = self.extract_resource_contract(function) {
            self.contracts.insert(function.name.name.clone(), contract);
        }

        // Analyze function body
        self.analyze_block(&function.body)?;

        // Check for leaks at function exit
        self.check_function_exit_leaks(&function.source_location);

        Ok(())
    }

    /// Analyze a resource scope
    pub fn analyze_resource_scope(&mut self, scope: &ResourceScope) -> Result<(), SemanticError> {
        // Enter new scope
        let scope_info = ScopeInfo {
            scope_id: scope.scope_id.clone(),
            resources: Vec::new(),
        };

        self.active_scopes.push(scope_info);

        // Acquire resources
        for resource in &scope.resources {
            self.acquire_resource(resource)?;
        }

        // Update max concurrent resources
        let current_count = self.count_active_resources();
        if current_count > self.results.max_concurrent_resources {
            self.results.max_concurrent_resources = current_count;
        }

        // Analyze scope body
        self.analyze_block(&scope.body)?;

        // Release resources (in cleanup order)
        let cleanup_sequence = scope.get_cleanup_sequence();
        for (i, _cleanup) in cleanup_sequence.iter().enumerate() {
            let resource = &scope.resources[scope.resources.len() - 1 - i];
            self.release_resource(
                &resource.binding.name,
                &resource.resource_type,
                &scope.source_location,
            )?;
        }

        // Exit scope
        self.active_scopes.pop();

        Ok(())
    }

    /// Acquire a resource
    fn acquire_resource(&mut self, resource: &ResourceAcquisition) -> Result<(), SemanticError> {
        let tracked = TrackedResource {
            binding: resource.binding.name.clone(),
            resource_type: resource.resource_type.clone(),
            acquisition_location: resource.binding.source_location.clone(),
            is_released: false,
            usage_count: 0,
        };

        // Add to current scope
        if let Some(scope) = self.active_scopes.last_mut() {
            scope.resources.push(tracked.clone());
        }

        // Track globally
        self.resource_tracking.insert(
            resource.binding.name.clone(),
            ResourceTracking {
                resource_id: format!("{}_{}", resource.binding.name, uuid()),
                resource_type: resource.resource_type.clone(),
                acquired_at: current_timestamp(),
                acquired_location: resource.binding.source_location.clone(),
                owner_scope: self
                    .active_scopes
                    .last()
                    .map(|s| s.scope_id.clone())
                    .unwrap_or_default(),
                usage_stats: ResourceUsageStats {
                    access_count: 0,
                    last_accessed: None,
                    total_hold_time_ms: 0,
                    peak_memory_bytes: None,
                    network_bytes: None,
                },
            },
        );

        // Check contracts
        self.check_resource_contracts(&resource.resource_type)?;

        Ok(())
    }

    /// Release a resource
    fn release_resource(
        &mut self,
        binding: &str,
        resource_type: &str,
        location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // Find resource in active scopes
        let mut found = false;

        for scope in self.active_scopes.iter_mut().rev() {
            if let Some(resource) = scope.resources.iter_mut().find(|r| r.binding == binding) {
                if resource.is_released {
                    // Double release error
                    self.results.double_releases.push(DoubleRelease {
                        resource_type: resource_type.to_string(),
                        binding: binding.to_string(),
                        first_release: resource.acquisition_location.clone(),
                        second_release: location.clone(),
                    });
                    return Err(SemanticError::InvalidOperation {
                        operation: "double release".to_string(),
                        reason: format!("Resource '{}' already released", binding),
                        location: location.clone(),
                    });
                }

                resource.is_released = true;
                found = true;
                break;
            }
        }

        if !found {
            return Err(SemanticError::UndefinedSymbol {
                symbol: binding.to_string(),
                location: location.clone(),
            });
        }

        // Remove from tracking
        self.resource_tracking.remove(binding);

        Ok(())
    }

    /// Analyze a block for resource usage
    fn analyze_block(&mut self, block: &Block) -> Result<(), SemanticError> {
        for statement in &block.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    /// Analyze a statement for resource usage
    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), SemanticError> {
        match statement {
            Statement::FunctionCall { call, .. } => {
                // Check for resource usage in function arguments
                for arg in &call.arguments {
                    self.check_resource_usage_in_expression(&arg.value)?;
                }
            }

            Statement::Assignment {
                target: _, value, ..
            } => {
                self.check_resource_usage_in_expression(value)?;
            }

            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.check_resource_usage_in_expression(condition)?;
                self.analyze_block(then_block)?;
                if let Some(else_blk) = else_block {
                    self.analyze_block(else_blk)?;
                }
            }

            Statement::WhileLoop {
                condition, body, ..
            } => {
                self.check_resource_usage_in_expression(condition)?;
                self.analyze_block(body)?;
            }

            Statement::Return { value, .. } => {
                if let Some(val) = value {
                    self.check_resource_usage_in_expression(val)?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Check for resource usage in expressions
    fn check_resource_usage_in_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<(), SemanticError> {
        match expr {
            Expression::Variable {
                name,
                source_location,
            } => {
                // Check if this is a resource
                if let Some(tracking) = self.resource_tracking.get_mut(&name.name) {
                    tracking.usage_stats.access_count += 1;
                    tracking.usage_stats.last_accessed = Some(current_timestamp());

                    // Check if resource is released
                    for scope in &self.active_scopes {
                        if let Some(resource) =
                            scope.resources.iter().find(|r| r.binding == name.name)
                        {
                            if resource.is_released {
                                self.results.use_after_release.push(UseAfterRelease {
                                    resource_type: resource.resource_type.clone(),
                                    binding: name.name.clone(),
                                    release_location: resource.acquisition_location.clone(),
                                    use_location: source_location.clone(),
                                });

                                return Err(SemanticError::InvalidOperation {
                                    operation: "use after release".to_string(),
                                    reason: format!("Resource '{}' used after release", name.name),
                                    location: source_location.clone(),
                                });
                            }
                        }
                    }
                }
            }

            // Recursively check subexpressions
            Expression::Add { left, right, .. }
            | Expression::Subtract { left, right, .. }
            | Expression::Multiply { left, right, .. }
            | Expression::Divide { left, right, .. } => {
                self.check_resource_usage_in_expression(left)?;
                self.check_resource_usage_in_expression(right)?;
            }

            Expression::FunctionCall { call, .. } => {
                for arg in &call.arguments {
                    self.check_resource_usage_in_expression(&arg.value)?;
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Check for resource leaks at function exit
    fn check_function_exit_leaks(&mut self, exit_location: &SourceLocation) {
        for scope in &self.active_scopes {
            for resource in &scope.resources {
                if !resource.is_released {
                    self.results.leaks.push(ResourceLeak {
                        resource_type: resource.resource_type.clone(),
                        binding: resource.binding.clone(),
                        acquisition_location: resource.acquisition_location.clone(),
                        scope_exit_location: exit_location.clone(),
                    });
                }
            }
        }
    }

    /// Count currently active resources
    fn count_active_resources(&self) -> usize {
        self.active_scopes
            .iter()
            .flat_map(|s| &s.resources)
            .filter(|r| !r.is_released)
            .count()
    }

    /// Check resource contracts
    fn check_resource_contracts(&self, resource_type: &str) -> Result<(), SemanticError> {
        // Check global resource limits
        let active_count = self
            .active_scopes
            .iter()
            .flat_map(|s| &s.resources)
            .filter(|r| !r.is_released && r.resource_type == resource_type)
            .count();

        // Check against contracts
        for (_, contract) in &self.contracts {
            match resource_type {
                "file_handle" => {
                    if let Some(max_files) = contract.max_file_handles {
                        if active_count > max_files as usize {
                            return Err(SemanticError::InvalidOperation {
                                operation: "resource acquisition".to_string(),
                                reason: format!(
                                    "Exceeded max file handles: {} > {}",
                                    active_count, max_files
                                ),
                                location: SourceLocation::unknown(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Extract resource contract from function metadata
    fn extract_resource_contract(&self, _function: &Function) -> Option<ResourceContract> {
        // In a full implementation, this would parse metadata
        // For now, return None
        None
    }

    /// Generate optimization suggestions
    pub fn generate_optimizations(&mut self) {
        // Analyze usage patterns
        let patterns = self.analyze_usage_patterns();

        // Suggest resource pooling for frequently acquired resources
        for (resource_type, pattern) in &patterns {
            if pattern.access_frequency > 10.0 {
                self.results.optimizations.push(ResourceOptimization {
                    optimization_type: OptimizationType::UsePool,
                    resource_type: resource_type.clone(),
                    location: SourceLocation::unknown(),
                    description: format!(
                        "Resource '{}' is acquired {} times per second. Consider using a resource pool.",
                        resource_type, pattern.access_frequency
                    ),
                    estimated_benefit: OptimizationBenefit {
                        memory_saved_mb: None,
                        latency_reduced_ms: Some(pattern.avg_hold_time * 0.5),
                        resource_count_reduced: Some(pattern.typical_count / 2),
                    },
                });
            }
        }

        // Suggest scope reduction for long-held resources
        for scope in &self.active_scopes {
            for resource in &scope.resources {
                if resource.usage_count == 0 {
                    self.results.optimizations.push(ResourceOptimization {
                        optimization_type: OptimizationType::LazyAcquisition,
                        resource_type: resource.resource_type.clone(),
                        location: resource.acquisition_location.clone(),
                        description: format!(
                            "Resource '{}' is acquired but never used",
                            resource.binding
                        ),
                        estimated_benefit: OptimizationBenefit {
                            memory_saved_mb: Some(1.0), // Estimate
                            latency_reduced_ms: Some(10.0),
                            resource_count_reduced: Some(1),
                        },
                    });
                }
            }
        }
    }

    /// Analyze resource usage patterns
    fn analyze_usage_patterns(&mut self) -> HashMap<String, UsagePattern> {
        let mut patterns = HashMap::new();

        // Group by resource type
        let mut type_stats: HashMap<String, Vec<&ResourceTracking>> = HashMap::new();
        for tracking in self.resource_tracking.values() {
            type_stats
                .entry(tracking.resource_type.clone())
                .or_insert_with(Vec::new)
                .push(tracking);
        }

        // Calculate patterns
        for (resource_type, trackings) in type_stats {
            if !trackings.is_empty() {
                let avg_hold_time = trackings
                    .iter()
                    .map(|t| t.usage_stats.total_hold_time_ms as f64)
                    .sum::<f64>()
                    / trackings.len() as f64;

                let max_hold_time = trackings
                    .iter()
                    .map(|t| t.usage_stats.total_hold_time_ms)
                    .max()
                    .unwrap_or(0);

                let total_accesses: u64 =
                    trackings.iter().map(|t| t.usage_stats.access_count).sum();

                patterns.insert(
                    resource_type.clone(),
                    UsagePattern {
                        resource_type,
                        avg_hold_time,
                        max_hold_time,
                        access_frequency: total_accesses as f64 / 1000.0, // Per second estimate
                        typical_count: trackings.len(),
                    },
                );
            }
        }

        self.results.usage_patterns.clone_from(&patterns);
        patterns
    }

    /// Get analysis results
    pub fn get_results(&self) -> &ResourceAnalysisResults {
        &self.results
    }
}

// Helper functions
fn uuid() -> String {
    // Simple UUID generator for demo
    format!(
        "{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FunctionCall, FunctionReference, Identifier};

    #[test]
    fn test_resource_leak_detection() {
        let mut analyzer = ResourceAnalyzer::new();

        // Create a scope first
        analyzer.active_scopes.push(ScopeInfo {
            scope_id: "test_scope".to_string(),
            resources: Vec::new(),
        });

        // Simulate resource acquisition without release
        let resource = ResourceAcquisition {
            resource_type: "file_handle".to_string(),
            binding: Identifier::new("file".to_string(), SourceLocation::unknown()),
            acquisition: Expression::NullLiteral {
                source_location: SourceLocation::unknown(),
            },
            cleanup: CleanupSpecification::Function {
                name: "close".to_string(),
                pass_resource: true,
            },
            type_spec: None,
            parameters: vec![],
        };

        analyzer.acquire_resource(&resource).unwrap();
        analyzer.check_function_exit_leaks(&SourceLocation::unknown());

        assert_eq!(analyzer.results.leaks.len(), 1);
        assert_eq!(analyzer.results.leaks[0].resource_type, "file_handle");
    }

    #[test]
    fn test_double_release_detection() {
        let mut analyzer = ResourceAnalyzer::new();

        // Enter scope first
        analyzer.active_scopes.push(ScopeInfo {
            scope_id: "test_scope".to_string(),
            resources: Vec::new(),
        });

        // Create and acquire resource
        let resource = ResourceAcquisition {
            resource_type: "memory_buffer".to_string(),
            binding: Identifier::new("buffer".to_string(), SourceLocation::unknown()),
            acquisition: Expression::NullLiteral {
                source_location: SourceLocation::unknown(),
            },
            cleanup: CleanupSpecification::Function {
                name: "free".to_string(),
                pass_resource: true,
            },
            type_spec: None,
            parameters: vec![],
        };

        analyzer.acquire_resource(&resource).unwrap();

        // First release - should succeed
        analyzer
            .release_resource("buffer", "memory_buffer", &SourceLocation::unknown())
            .unwrap();

        // Second release - should fail
        let result =
            analyzer.release_resource("buffer", "memory_buffer", &SourceLocation::unknown());
        assert!(result.is_err());
        assert_eq!(analyzer.results.double_releases.len(), 1);
    }
}
