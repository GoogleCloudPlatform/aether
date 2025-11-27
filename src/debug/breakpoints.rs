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

//! Breakpoint management for AetherScript debugging
//!
//! Provides comprehensive breakpoint support including line breakpoints,
//! conditional breakpoints, watchpoints, and exception breakpoints.

use crate::error::{SemanticError, SourceLocation};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};

/// Breakpoint manager
#[derive(Debug, Default)]
pub struct BreakpointManager {
    /// All breakpoints indexed by ID
    breakpoints: HashMap<u32, Breakpoint>,

    /// Breakpoints by source location
    location_index: HashMap<SourceLocation, HashSet<u32>>,

    /// Breakpoints by function name
    function_index: HashMap<String, HashSet<u32>>,

    /// Next breakpoint ID
    next_id: AtomicU32,

    /// Watchpoints
    watchpoints: HashMap<u32, Watchpoint>,

    /// Exception breakpoints
    exception_breakpoints: HashMap<u32, ExceptionBreakpoint>,

    /// Global breakpoint settings
    settings: BreakpointSettings,
}

/// Individual breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Unique breakpoint ID
    pub id: u32,

    /// Breakpoint type
    pub breakpoint_type: BreakpointType,

    /// Source location
    pub location: SourceLocation,

    /// Whether breakpoint is enabled
    pub enabled: bool,

    /// Condition for conditional breakpoints
    pub condition: Option<BreakpointCondition>,

    /// Hit count tracking
    pub hit_count: u32,

    /// Ignore count (break after N hits)
    pub ignore_count: u32,

    /// Log message (for logging breakpoints)
    pub log_message: Option<String>,

    /// Breakpoint actions
    pub actions: Vec<BreakpointAction>,

    /// Metadata
    pub metadata: BreakpointMetadata,
}

/// Types of breakpoints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BreakpointType {
    /// Standard line breakpoint
    Line,

    /// Function entry breakpoint
    FunctionEntry,

    /// Function exit breakpoint
    FunctionExit,

    /// Conditional breakpoint
    Conditional,

    /// Logging breakpoint (tracepoint)
    Logging,

    /// Temporary breakpoint (one-time)
    Temporary,

    /// Hardware breakpoint
    Hardware,
}

/// Breakpoint condition
#[derive(Debug, Serialize, Deserialize)]
pub struct BreakpointCondition {
    /// Condition expression
    pub expression: String,

    /// Condition language
    pub language: ConditionLanguage,

    /// Compiled condition (internal)
    #[serde(skip)]
    pub compiled: Option<CompiledCondition>,
}

impl Clone for BreakpointCondition {
    fn clone(&self) -> Self {
        Self {
            expression: self.expression.clone(),
            language: self.language.clone(),
            compiled: None, // Don't clone the compiled condition
        }
    }
}

/// Condition expression languages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionLanguage {
    /// AetherScript expression
    AetherScript,

    /// Simple comparison (variable == value)
    Simple,

    /// Regular expression
    Regex,
}

/// Compiled condition for fast evaluation
pub struct CompiledCondition {
    /// Evaluation function
    pub evaluator: Box<dyn Fn(&DebugContext) -> bool + Send + Sync>,

    /// Variables referenced in condition
    pub referenced_variables: Vec<String>,
}

impl std::fmt::Debug for CompiledCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompiledCondition")
            .field("referenced_variables", &self.referenced_variables)
            .field("evaluator", &"<function>")
            .finish()
    }
}

/// Debug execution context
#[derive(Debug)]
pub struct DebugContext {
    /// Current function name
    pub current_function: String,

    /// Current source location
    pub current_location: SourceLocation,

    /// Variable values
    pub variables: HashMap<String, VariableValue>,

    /// Call stack
    pub call_stack: Vec<StackFrame>,

    /// Register values
    pub registers: HashMap<String, u64>,
}

/// Variable value in debug context
#[derive(Debug, Clone)]
pub struct VariableValue {
    /// Variable name
    pub name: String,

    /// Variable type
    pub var_type: String,

    /// String representation of value
    pub value: String,

    /// Raw bytes (if available)
    pub raw_bytes: Option<Vec<u8>>,

    /// Memory address (if applicable)
    pub address: Option<u64>,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Function name
    pub function_name: String,

    /// Source location
    pub location: SourceLocation,

    /// Frame pointer
    pub frame_pointer: u64,

    /// Return address
    pub return_address: u64,

    /// Local variables
    pub locals: HashMap<String, VariableValue>,
}

/// Breakpoint actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakpointAction {
    /// Stop execution
    Stop,

    /// Log message
    Log { message: String },

    /// Evaluate expression
    Evaluate { expression: String },

    /// Take snapshot
    Snapshot { include_memory: bool },

    /// Continue execution
    Continue,

    /// Send notification
    Notify { message: String },
}

/// Breakpoint metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointMetadata {
    /// When breakpoint was created
    pub created_at: std::time::SystemTime,

    /// Last hit time
    pub last_hit: Option<std::time::SystemTime>,

    /// Creator/source
    pub creator: String,

    /// Description
    pub description: Option<String>,

    /// Tags
    pub tags: Vec<String>,
}

/// Watchpoint for monitoring variable changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchpoint {
    /// Unique ID
    pub id: u32,

    /// Variable or expression to watch
    pub expression: String,

    /// Watch type
    pub watch_type: WatchType,

    /// Whether enabled
    pub enabled: bool,

    /// Last known value
    pub last_value: Option<String>,

    /// Hit count
    pub hit_count: u32,

    /// Condition (optional)
    pub condition: Option<BreakpointCondition>,
}

/// Types of watchpoints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WatchType {
    /// Break on read access
    Read,

    /// Break on write access
    Write,

    /// Break on read or write access
    ReadWrite,

    /// Break when value changes
    Change,
}

/// Exception breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionBreakpoint {
    /// Unique ID
    pub id: u32,

    /// Exception type pattern
    pub exception_pattern: String,

    /// Whether to break on caught exceptions
    pub break_on_caught: bool,

    /// Whether to break on uncaught exceptions
    pub break_on_uncaught: bool,

    /// Whether enabled
    pub enabled: bool,

    /// Condition (optional)
    pub condition: Option<BreakpointCondition>,
}

/// Global breakpoint settings
#[derive(Debug, Clone)]
pub struct BreakpointSettings {
    /// Break on all exceptions
    pub break_on_all_exceptions: bool,

    /// Break on caught exceptions
    pub break_on_caught_exceptions: bool,

    /// Break on uncaught exceptions
    pub break_on_uncaught_exceptions: bool,

    /// Enable just-in-time debugging
    pub enable_jit_debugging: bool,

    /// Maximum hit count before auto-disable
    pub max_hit_count: Option<u32>,

    /// Default ignore count
    pub default_ignore_count: u32,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            location_index: HashMap::new(),
            function_index: HashMap::new(),
            next_id: AtomicU32::new(1),
            watchpoints: HashMap::new(),
            exception_breakpoints: HashMap::new(),
            settings: BreakpointSettings::default(),
        }
    }

    /// Create a new breakpoint
    pub fn create_breakpoint(
        &mut self,
        breakpoint_type: BreakpointType,
        location: SourceLocation,
        condition: Option<BreakpointCondition>,
    ) -> Result<u32, SemanticError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let breakpoint = Breakpoint {
            id,
            breakpoint_type,
            location: location.clone(),
            enabled: true,
            condition,
            hit_count: 0,
            ignore_count: 0,
            log_message: None,
            actions: vec![BreakpointAction::Stop],
            metadata: BreakpointMetadata {
                created_at: std::time::SystemTime::now(),
                last_hit: None,
                creator: "user".to_string(),
                description: None,
                tags: vec![],
            },
        };

        // Index the breakpoint
        self.location_index
            .entry(location.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.breakpoints.insert(id, breakpoint);

        Ok(id)
    }

    /// Create a line breakpoint
    pub fn create_line_breakpoint(
        &mut self,
        file: String,
        line: u32,
        column: Option<u32>,
    ) -> Result<u32, SemanticError> {
        let location = SourceLocation {
            file,
            line: line as usize,
            column: column.unwrap_or(0) as usize,
            offset: 0,
        };

        self.create_breakpoint(BreakpointType::Line, location, None)
    }

    /// Create a conditional breakpoint
    pub fn create_conditional_breakpoint(
        &mut self,
        location: SourceLocation,
        condition: String,
    ) -> Result<u32, SemanticError> {
        let condition = BreakpointCondition {
            expression: condition,
            language: ConditionLanguage::AetherScript,
            compiled: None,
        };

        self.create_breakpoint(BreakpointType::Conditional, location, Some(condition))
    }

    /// Create a function breakpoint
    pub fn create_function_breakpoint(
        &mut self,
        function_name: String,
        on_entry: bool,
    ) -> Result<u32, SemanticError> {
        let location = SourceLocation {
            file: "".to_string(), // Function breakpoints may not have specific file
            line: 0,
            column: 0,
            offset: 0,
        };

        let breakpoint_type = if on_entry {
            BreakpointType::FunctionEntry
        } else {
            BreakpointType::FunctionExit
        };

        let id = self.create_breakpoint(breakpoint_type, location, None)?;

        // Index by function name
        self.function_index
            .entry(function_name)
            .or_insert_with(HashSet::new)
            .insert(id);

        Ok(id)
    }

    /// Create a watchpoint
    pub fn create_watchpoint(
        &mut self,
        expression: String,
        watch_type: WatchType,
        condition: Option<BreakpointCondition>,
    ) -> Result<u32, SemanticError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let watchpoint = Watchpoint {
            id,
            expression,
            watch_type,
            enabled: true,
            last_value: None,
            hit_count: 0,
            condition,
        };

        self.watchpoints.insert(id, watchpoint);
        Ok(id)
    }

    /// Create an exception breakpoint
    pub fn create_exception_breakpoint(
        &mut self,
        exception_pattern: String,
        break_on_caught: bool,
        break_on_uncaught: bool,
    ) -> Result<u32, SemanticError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let exception_breakpoint = ExceptionBreakpoint {
            id,
            exception_pattern,
            break_on_caught,
            break_on_uncaught,
            enabled: true,
            condition: None,
        };

        self.exception_breakpoints.insert(id, exception_breakpoint);
        Ok(id)
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: u32) -> Result<(), SemanticError> {
        if let Some(breakpoint) = self.breakpoints.remove(&id) {
            // Remove from location index
            if let Some(location_set) = self.location_index.get_mut(&breakpoint.location) {
                location_set.remove(&id);
                if location_set.is_empty() {
                    self.location_index.remove(&breakpoint.location);
                }
            }

            // Remove from function index
            for function_set in self.function_index.values_mut() {
                function_set.remove(&id);
            }

            Ok(())
        } else if self.watchpoints.remove(&id).is_some() {
            Ok(())
        } else if self.exception_breakpoints.remove(&id).is_some() {
            Ok(())
        } else {
            Err(SemanticError::Internal {
                message: format!("Breakpoint {} not found", id),
            })
        }
    }

    /// Enable/disable a breakpoint
    pub fn set_breakpoint_enabled(&mut self, id: u32, enabled: bool) -> Result<(), SemanticError> {
        if let Some(breakpoint) = self.breakpoints.get_mut(&id) {
            breakpoint.enabled = enabled;
            Ok(())
        } else if let Some(watchpoint) = self.watchpoints.get_mut(&id) {
            watchpoint.enabled = enabled;
            Ok(())
        } else if let Some(exception_bp) = self.exception_breakpoints.get_mut(&id) {
            exception_bp.enabled = enabled;
            Ok(())
        } else {
            Err(SemanticError::Internal {
                message: format!("Breakpoint {} not found", id),
            })
        }
    }

    /// Get breakpoint by ID
    pub fn get_breakpoint(&self, id: u32) -> Option<&Breakpoint> {
        self.breakpoints.get(&id)
    }

    /// Get all breakpoints
    pub fn get_all_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Get breakpoints at location
    pub fn get_breakpoints_at_location(&self, location: &SourceLocation) -> Vec<&Breakpoint> {
        if let Some(ids) = self.location_index.get(location) {
            ids.iter()
                .filter_map(|id| self.breakpoints.get(id))
                .collect()
        } else {
            vec![]
        }
    }

    /// Get breakpoints for function
    pub fn get_breakpoints_for_function(&self, function_name: &str) -> Vec<&Breakpoint> {
        if let Some(ids) = self.function_index.get(function_name) {
            ids.iter()
                .filter_map(|id| self.breakpoints.get(id))
                .collect()
        } else {
            vec![]
        }
    }

    /// Check if execution should break at location
    pub fn should_break_at_location(
        &mut self,
        location: &SourceLocation,
        context: &DebugContext,
    ) -> Result<Vec<u32>, SemanticError> {
        let mut triggered_breakpoint_ids = Vec::new();

        // First, collect the IDs of breakpoints to check
        let breakpoint_ids: Vec<u32> = if let Some(ids) = self.location_index.get(location) {
            ids.iter().cloned().collect()
        } else {
            vec![]
        };

        for id in breakpoint_ids {
            if let Some(breakpoint) = self.breakpoints.get(&id) {
                if !breakpoint.enabled {
                    continue;
                }

                // Check ignore count
                if breakpoint.hit_count < breakpoint.ignore_count {
                    continue;
                }

                // Check condition
                if let Some(ref condition) = breakpoint.condition {
                    if !self.evaluate_condition(condition, context)? {
                        continue;
                    }
                }

                triggered_breakpoint_ids.push(id);
            }
        }

        // Update hit counts
        for id in &triggered_breakpoint_ids {
            if let Some(bp) = self.breakpoints.get_mut(id) {
                bp.hit_count += 1;
                bp.metadata.last_hit = Some(std::time::SystemTime::now());
            }
        }

        Ok(triggered_breakpoint_ids)
    }

    /// Evaluate breakpoint condition
    fn evaluate_condition(
        &self,
        condition: &BreakpointCondition,
        context: &DebugContext,
    ) -> Result<bool, SemanticError> {
        // Simplified condition evaluation
        match condition.language {
            ConditionLanguage::Simple => {
                // Simple format: "variable == value"
                let parts: Vec<&str> = condition.expression.split("==").collect();
                if parts.len() == 2 {
                    let var_name = parts[0].trim();
                    let expected_value = parts[1].trim();

                    if let Some(var_value) = context.variables.get(var_name) {
                        return Ok(var_value.value == expected_value);
                    }
                }
                Ok(false)
            }
            ConditionLanguage::AetherScript => {
                // Would evaluate as AetherScript expression
                // For now, always return true
                Ok(true)
            }
            ConditionLanguage::Regex => {
                // Would use regex matching
                Ok(true)
            }
        }
    }

    /// Check watchpoints
    pub fn check_watchpoints(
        &mut self,
        context: &DebugContext,
    ) -> Result<Vec<&Watchpoint>, SemanticError> {
        let mut triggered_watchpoints = Vec::new();

        for watchpoint in self.watchpoints.values_mut() {
            if !watchpoint.enabled {
                continue;
            }

            // Check if watched variable changed
            if let Some(var_value) = context.variables.get(&watchpoint.expression) {
                let current_value = &var_value.value;

                let should_trigger = match &watchpoint.last_value {
                    Some(last) => last != current_value,
                    None => true, // First time seeing this variable
                };

                if should_trigger {
                    watchpoint.last_value = Some(current_value.clone());
                    watchpoint.hit_count += 1;
                    triggered_watchpoints.push(&*watchpoint);
                }
            }
        }

        Ok(triggered_watchpoints)
    }

    /// Get settings
    pub fn settings(&self) -> &BreakpointSettings {
        &self.settings
    }

    /// Update settings
    pub fn update_settings(&mut self, settings: BreakpointSettings) {
        self.settings = settings;
    }

    /// Clear all breakpoints
    pub fn clear_all(&mut self) {
        self.breakpoints.clear();
        self.location_index.clear();
        self.function_index.clear();
        self.watchpoints.clear();
        self.exception_breakpoints.clear();
    }

    /// Get statistics
    pub fn statistics(&self) -> BreakpointStatistics {
        BreakpointStatistics {
            total_breakpoints: self.breakpoints.len(),
            enabled_breakpoints: self.breakpoints.values().filter(|bp| bp.enabled).count(),
            total_watchpoints: self.watchpoints.len(),
            enabled_watchpoints: self.watchpoints.values().filter(|wp| wp.enabled).count(),
            total_exception_breakpoints: self.exception_breakpoints.len(),
            enabled_exception_breakpoints: self
                .exception_breakpoints
                .values()
                .filter(|ep| ep.enabled)
                .count(),
        }
    }
}

/// Breakpoint statistics
#[derive(Debug, Clone)]
pub struct BreakpointStatistics {
    pub total_breakpoints: usize,
    pub enabled_breakpoints: usize,
    pub total_watchpoints: usize,
    pub enabled_watchpoints: usize,
    pub total_exception_breakpoints: usize,
    pub enabled_exception_breakpoints: usize,
}

impl Default for BreakpointSettings {
    fn default() -> Self {
        Self {
            break_on_all_exceptions: false,
            break_on_caught_exceptions: false,
            break_on_uncaught_exceptions: true,
            enable_jit_debugging: false,
            max_hit_count: None,
            default_ignore_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_creation() {
        let mut manager = BreakpointManager::new();

        let location = SourceLocation {
            file: "test.aether".to_string(),
            line: 10,
            column: 5,
            offset: 0,
        };

        let id = manager
            .create_breakpoint(BreakpointType::Line, location.clone(), None)
            .unwrap();
        assert!(id > 0);

        let breakpoint = manager.get_breakpoint(id).unwrap();
        assert_eq!(breakpoint.location, location);
        assert!(breakpoint.enabled);
    }

    #[test]
    fn test_line_breakpoint() {
        let mut manager = BreakpointManager::new();

        let id = manager
            .create_line_breakpoint("test.aether".to_string(), 15, Some(10))
            .unwrap();
        let breakpoint = manager.get_breakpoint(id).unwrap();

        assert_eq!(breakpoint.breakpoint_type, BreakpointType::Line);
        assert_eq!(breakpoint.location.line, 15);
        assert_eq!(breakpoint.location.column, 10);
    }

    #[test]
    fn test_conditional_breakpoint() {
        let mut manager = BreakpointManager::new();

        let location = SourceLocation {
            file: "test.aether".to_string(),
            line: 20,
            column: 0,
            offset: 0,
        };

        let id = manager
            .create_conditional_breakpoint(location, "x == 42".to_string())
            .unwrap();
        let breakpoint = manager.get_breakpoint(id).unwrap();

        assert_eq!(breakpoint.breakpoint_type, BreakpointType::Conditional);
        assert!(breakpoint.condition.is_some());
        assert_eq!(breakpoint.condition.as_ref().unwrap().expression, "x == 42");
    }

    #[test]
    fn test_function_breakpoint() {
        let mut manager = BreakpointManager::new();

        let id = manager
            .create_function_breakpoint("main".to_string(), true)
            .unwrap();
        let breakpoint = manager.get_breakpoint(id).unwrap();

        assert_eq!(breakpoint.breakpoint_type, BreakpointType::FunctionEntry);

        let function_breakpoints = manager.get_breakpoints_for_function("main");
        assert_eq!(function_breakpoints.len(), 1);
    }

    #[test]
    fn test_watchpoint() {
        let mut manager = BreakpointManager::new();

        let id = manager
            .create_watchpoint("my_variable".to_string(), WatchType::Write, None)
            .unwrap();

        assert!(manager.watchpoints.contains_key(&id));
        let watchpoint = &manager.watchpoints[&id];
        assert_eq!(watchpoint.expression, "my_variable");
        assert_eq!(watchpoint.watch_type, WatchType::Write);
    }

    #[test]
    fn test_breakpoint_enable_disable() {
        let mut manager = BreakpointManager::new();

        let id = manager
            .create_line_breakpoint("test.aether".to_string(), 10, None)
            .unwrap();

        // Should be enabled by default
        assert!(manager.get_breakpoint(id).unwrap().enabled);

        // Disable
        assert!(manager.set_breakpoint_enabled(id, false).is_ok());
        assert!(!manager.get_breakpoint(id).unwrap().enabled);

        // Re-enable
        assert!(manager.set_breakpoint_enabled(id, true).is_ok());
        assert!(manager.get_breakpoint(id).unwrap().enabled);
    }

    #[test]
    fn test_breakpoint_removal() {
        let mut manager = BreakpointManager::new();

        let id = manager
            .create_line_breakpoint("test.aether".to_string(), 10, None)
            .unwrap();
        assert!(manager.get_breakpoint(id).is_some());

        assert!(manager.remove_breakpoint(id).is_ok());
        assert!(manager.get_breakpoint(id).is_none());

        // Removing non-existent breakpoint should fail
        assert!(manager.remove_breakpoint(id).is_err());
    }

    #[test]
    fn test_breakpoint_statistics() {
        let mut manager = BreakpointManager::new();

        // Create some breakpoints
        let id1 = manager
            .create_line_breakpoint("test.aether".to_string(), 10, None)
            .unwrap();
        let id2 = manager
            .create_line_breakpoint("test.aether".to_string(), 20, None)
            .unwrap();
        manager
            .create_watchpoint("var1".to_string(), WatchType::Change, None)
            .unwrap();

        // Disable one breakpoint
        manager.set_breakpoint_enabled(id2, false).unwrap();

        let stats = manager.statistics();
        assert_eq!(stats.total_breakpoints, 2);
        assert_eq!(stats.enabled_breakpoints, 1);
        assert_eq!(stats.total_watchpoints, 1);
        assert_eq!(stats.enabled_watchpoints, 1);
    }
}
