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

//! Debugger integration for AetherScript
//!
//! Provides integration with external debuggers like GDB and LLDB through
//! standardized debugging interfaces and protocols.

use crate::error::{SemanticError, SourceLocation};
use crate::mir::Program;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

/// Debugger interface for AetherScript programs
#[derive(Debug)]
pub struct DebuggerInterface {
    /// Active debugging session
    session: Option<DebugSession>,

    /// Breakpoint manager
    breakpoints: BreakpointManager,

    /// Variable inspector
    variable_inspector: VariableInspector,

    /// Call stack tracker
    call_stack: CallStackTracker,

    /// Debugger configuration
    config: DebuggerConfig,
}

/// Active debugging session
#[derive(Debug)]
pub struct DebugSession {
    /// Target process ID
    target_pid: Option<u32>,

    /// Communication channel with debugger
    request_sender: Sender<DebugCommand>,

    /// Breakpoints
    breakpoints: Arc<Mutex<HashMap<String, Vec<Breakpoint>>>>,

    /// Current execution state
    execution_state: Arc<Mutex<ExecutionState>>,
}

/// Debugger configuration
#[derive(Debug, Clone)]
pub struct DebuggerConfig {
    /// Preferred debugger (GDB, LLDB)
    pub debugger_type: DebuggerType,

    /// Enable automatic symbol loading
    pub auto_load_symbols: bool,

    /// Enable source-level debugging
    pub source_level_debugging: bool,

    /// Maximum call stack depth to display
    pub max_stack_depth: usize,

    /// Enable variable value watching
    pub enable_watchpoints: bool,
}

/// Supported debugger types
#[derive(Debug, Clone, PartialEq)]
pub enum DebuggerType {
    Gdb,
    Lldb,
    Auto, // Auto-detect based on platform
}

/// Debug commands
#[derive(Debug, Clone)]
pub enum DebugCommand {
    /// Start debugging a program
    Start {
        program_path: String,
        args: Vec<String>,
    },

    /// Set breakpoint at location
    SetBreakpoint { file: String, line: u32 },

    /// Remove breakpoint
    RemoveBreakpoint { id: u32 },

    /// Continue execution
    Continue,

    /// Step to next line
    StepOver,

    /// Step into function
    StepInto,

    /// Step out of function
    StepOut,

    /// Examine variable
    ExamineVariable { name: String },

    /// Evaluate expression
    EvaluateExpression { expression: String },

    /// Get call stack
    GetCallStack,

    /// Stop debugging
    Stop,
}

/// Debug responses
#[derive(Debug, Clone)]
pub enum DebugResponse {
    /// Session started successfully
    SessionStarted { process_id: u32 },

    /// Breakpoint was set
    BreakpointSet { id: u32, location: SourceLocation },

    /// Execution stopped at breakpoint
    BreakpointHit { id: u32, location: SourceLocation },

    /// Variable value
    VariableValue {
        name: String,
        value: String,
        type_name: String,
    },

    /// Expression evaluation result
    ExpressionResult { result: String },

    /// Call stack information
    CallStack { frames: Vec<StackFrame> },

    /// Execution state changed
    StateChanged { new_state: ExecutionState },

    /// Error occurred
    Error { message: String },
}

/// Program execution state
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionState {
    NotStarted,
    Running,
    Paused,
    BreakpointHit,
    Stopped,
    Exited { exit_code: i32 },
    Error { message: String },
}

/// Target program information
#[derive(Debug, Clone)]
pub struct TargetInfo {
    /// Program executable path
    pub executable_path: String,

    /// Command line arguments
    pub arguments: Vec<String>,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Working directory
    pub working_directory: String,

    /// Process ID (when running)
    pub process_id: Option<u32>,
}

/// Stack frame information
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Frame index (0 = current frame)
    pub index: usize,

    /// Function name
    pub function_name: String,

    /// Source location
    pub location: SourceLocation,

    /// Frame address
    pub address: u64,

    /// Local variables in this frame
    pub locals: Vec<VariableInfo>,
}

/// Variable information
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// Variable name
    pub name: String,

    /// Variable type
    pub type_name: String,

    /// Current value as string
    pub value: String,

    /// Memory address (if applicable)
    pub address: Option<u64>,

    /// Whether variable can be modified
    pub is_mutable: bool,
}

/// Breakpoint manager
#[derive(Debug, Default)]
pub struct BreakpointManager {
    /// Active breakpoints
    breakpoints: HashMap<u32, Breakpoint>,

    /// Next breakpoint ID
    next_id: u32,
}

/// Breakpoint information
#[derive(Debug, Clone)]
pub struct Breakpoint {
    /// Unique breakpoint ID
    pub id: u32,

    /// Source location
    pub location: SourceLocation,

    /// Whether breakpoint is enabled
    pub enabled: bool,

    /// Condition (if conditional breakpoint)
    pub condition: Option<String>,

    /// Hit count
    pub hit_count: u32,

    /// Ignore count
    pub ignore_count: u32,
}

/// Variable inspector for examining program state
#[derive(Debug, Default)]
pub struct VariableInspector {
    /// Watched variables
    watched_variables: HashMap<String, WatchedVariable>,

    /// Variable history
    variable_history: Vec<VariableSnapshot>,
}

/// Watched variable
#[derive(Debug, Clone)]
pub struct WatchedVariable {
    /// Variable name or expression
    pub expression: String,

    /// Last known value
    pub last_value: Option<String>,

    /// Watch enabled
    pub enabled: bool,
}

/// Variable value snapshot
#[derive(Debug, Clone)]
pub struct VariableSnapshot {
    /// Variable name
    pub name: String,

    /// Value at time of snapshot
    pub value: String,

    /// Timestamp
    pub timestamp: std::time::SystemTime,

    /// Stack frame when captured
    pub frame_index: usize,
}

/// Call stack tracker
#[derive(Debug, Default)]
pub struct CallStackTracker {
    /// Current call stack
    current_stack: Vec<StackFrame>,

    /// Stack history
    stack_history: Vec<CallStackSnapshot>,
}

/// Call stack snapshot
#[derive(Debug, Clone)]
pub struct CallStackSnapshot {
    /// Stack frames at time of snapshot
    pub frames: Vec<StackFrame>,

    /// Timestamp
    pub timestamp: std::time::SystemTime,

    /// Event that triggered snapshot
    pub trigger: StackSnapshotTrigger,
}

/// Events that trigger stack snapshots
#[derive(Debug, Clone)]
pub enum StackSnapshotTrigger {
    Breakpoint,
    StepInto,
    StepOut,
    Exception,
    FunctionCall,
    FunctionReturn,
}

impl DebuggerInterface {
    pub fn new() -> Self {
        Self {
            session: None,
            breakpoints: BreakpointManager::default(),
            variable_inspector: VariableInspector::default(),
            call_stack: CallStackTracker::default(),
            config: DebuggerConfig::default(),
        }
    }

    /// Initialize debugger with program
    pub fn initialize(&mut self, program: &Program) -> Result<(), SemanticError> {
        // Detect available debugger
        let debugger_type = self.detect_debugger()?;
        self.config.debugger_type = debugger_type;

        eprintln!(
            "Initialized debugger interface with {:?}",
            self.config.debugger_type
        );
        eprintln!("Program has {} functions", program.functions.len());

        Ok(())
    }

    /// Detect available debugger on the system
    fn detect_debugger(&self) -> Result<DebuggerType, SemanticError> {
        // Try to find GDB first
        if let Ok(output) = Command::new("gdb").arg("--version").output() {
            if output.status.success() {
                return Ok(DebuggerType::Gdb);
            }
        }

        // Try to find LLDB
        if let Ok(output) = Command::new("lldb").arg("--version").output() {
            if output.status.success() {
                return Ok(DebuggerType::Lldb);
            }
        }

        Err(SemanticError::Internal {
            message: "No compatible debugger found (GDB or LLDB required)".to_string(),
        })
    }

    /// Start debugging session
    pub fn start_session(
        &mut self,
        program_path: String,
        args: Vec<String>,
    ) -> Result<(), SemanticError> {
        let (command_sender, command_receiver) = channel();
        let (response_sender, response_receiver) = channel();

        // Start debugger process
        let debugger_process = self.start_debugger_process(&program_path)?;

        // Start communication thread
        let debugger_type = self.config.debugger_type.clone();
        thread::spawn(move || {
            Self::debugger_communication_thread(debugger_type, command_receiver, response_sender);
        });

        let target_info = TargetInfo {
            executable_path: program_path,
            arguments: args,
            environment: std::env::vars().collect(),
            working_directory: std::env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            process_id: None,
        };

        self.session = Some(DebugSession {
            target_pid: None,
            request_sender: command_sender,
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
            execution_state: Arc::new(Mutex::new(ExecutionState::NotStarted)),
        });

        Ok(())
    }

    /// Start debugger process
    fn start_debugger_process(&self, program_path: &str) -> Result<Child, SemanticError> {
        let mut command = match self.config.debugger_type {
            DebuggerType::Gdb => {
                let mut cmd = Command::new("gdb");
                cmd.arg("--interpreter=mi3")
                    .arg("--quiet")
                    .arg(program_path);
                cmd
            }
            DebuggerType::Lldb => {
                let mut cmd = Command::new("lldb");
                cmd.arg("-o").arg("target create").arg(program_path);
                cmd
            }
            DebuggerType::Auto => {
                return self.start_debugger_process(program_path);
            }
        };

        let child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| SemanticError::Internal {
                message: format!("Failed to start debugger: {}", e),
            })?;

        Ok(child)
    }

    /// Debugger communication thread
    fn debugger_communication_thread(
        debugger_type: DebuggerType,
        command_receiver: Receiver<DebugCommand>,
        response_sender: Sender<DebugResponse>,
    ) {
        // This would handle actual communication with the debugger
        // For now, we'll simulate responses
        while let Ok(command) = command_receiver.recv() {
            let response = Self::simulate_debugger_response(&command, &debugger_type);
            if response_sender.send(response).is_err() {
                break;
            }
        }
    }

    /// Simulate debugger response (placeholder)
    fn simulate_debugger_response(
        command: &DebugCommand,
        _debugger_type: &DebuggerType,
    ) -> DebugResponse {
        match command {
            DebugCommand::Start { .. } => DebugResponse::SessionStarted { process_id: 12345 },
            DebugCommand::SetBreakpoint { file, line } => DebugResponse::BreakpointSet {
                id: 1,
                location: SourceLocation {
                    file: file.clone(),
                    line: *line as usize,
                    column: 1,
                    offset: 0,
                },
            },
            DebugCommand::ExamineVariable { name } => DebugResponse::VariableValue {
                name: name.clone(),
                value: "42".to_string(),
                type_name: "integer".to_string(),
            },
            _ => DebugResponse::Error {
                message: "Command not implemented".to_string(),
            },
        }
    }

    /// Set breakpoint at source location
    pub fn set_breakpoint(&mut self, file: &str, line: u32) -> Result<(), SemanticError> {
        let breakpoint = Breakpoint {
            id: self.breakpoints.next_id,
            location: SourceLocation {
                file: file.to_string(),
                line: line as usize,
                column: 1,
                offset: 0,
            },
            enabled: true,
            condition: None,
            hit_count: 0,
            ignore_count: 0,
        };

        self.breakpoints
            .breakpoints
            .insert(self.breakpoints.next_id, breakpoint);
        self.breakpoints.next_id += 1;

        // Send command to debugger if session is active
        if let Some(ref session) = self.session {
            let command = DebugCommand::SetBreakpoint {
                file: file.to_string(),
                line,
            };
            session
                .request_sender
                .send(command)
                .map_err(|e| SemanticError::Internal {
                    message: format!("Failed to send debugger command: {}", e),
                })?;
        }

        Ok(())
    }

    /// Remove breakpoint
    pub fn remove_breakpoint(&mut self, id: u32) -> Result<(), SemanticError> {
        if self.breakpoints.breakpoints.remove(&id).is_none() {
            return Err(SemanticError::Internal {
                message: format!("Breakpoint {} not found", id),
            });
        }

        // Send command to debugger if session is active
        if let Some(ref session) = self.session {
            let command = DebugCommand::RemoveBreakpoint { id };
            session
                .request_sender
                .send(command)
                .map_err(|e| SemanticError::Internal {
                    message: format!("Failed to send debugger command: {}", e),
                })?;
        }

        Ok(())
    }

    /// Get all breakpoints
    pub fn breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.breakpoints.values().collect()
    }

    /// Add watched variable
    pub fn add_watch(&mut self, expression: String) -> Result<(), SemanticError> {
        let watch = WatchedVariable {
            expression: expression.clone(),
            last_value: None,
            enabled: true,
        };

        self.variable_inspector
            .watched_variables
            .insert(expression, watch);
        Ok(())
    }

    /// Remove watched variable
    pub fn remove_watch(&mut self, expression: &str) -> Result<(), SemanticError> {
        if self
            .variable_inspector
            .watched_variables
            .remove(expression)
            .is_none()
        {
            return Err(SemanticError::Internal {
                message: format!("Watch expression '{}' not found", expression),
            });
        }
        Ok(())
    }

    /// Get current execution state
    pub fn execution_state(&self) -> ExecutionState {
        self.session
            .as_ref()
            .and_then(|s| s.execution_state.lock().ok())
            .map(|state| state.clone())
            .unwrap_or(ExecutionState::NotStarted)
    }

    /// Get current call stack
    pub fn call_stack(&self) -> &[StackFrame] {
        &self.call_stack.current_stack
    }

    /// Update configuration
    pub fn update_config(&mut self, config: DebuggerConfig) {
        self.config = config;
    }

    /// Get configuration
    pub fn config(&self) -> &DebuggerConfig {
        &self.config
    }
}

impl Default for DebuggerConfig {
    fn default() -> Self {
        Self {
            debugger_type: DebuggerType::Auto,
            auto_load_symbols: true,
            source_level_debugging: true,
            max_stack_depth: 100,
            enable_watchpoints: true,
        }
    }
}

impl BreakpointManager {
    /// Get breakpoint by ID
    pub fn get_breakpoint(&self, id: u32) -> Option<&Breakpoint> {
        self.breakpoints.get(&id)
    }

    /// Get all breakpoints
    pub fn all_breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Enable/disable breakpoint
    pub fn set_breakpoint_enabled(&mut self, id: u32, enabled: bool) -> Result<(), SemanticError> {
        if let Some(breakpoint) = self.breakpoints.get_mut(&id) {
            breakpoint.enabled = enabled;
            Ok(())
        } else {
            Err(SemanticError::Internal {
                message: format!("Breakpoint {} not found", id),
            })
        }
    }
}

impl VariableInspector {
    /// Get all watched variables
    pub fn watched_variables(&self) -> &HashMap<String, WatchedVariable> {
        &self.watched_variables
    }

    /// Get variable history
    pub fn variable_history(&self) -> &[VariableSnapshot] {
        &self.variable_history
    }

    /// Take variable snapshot
    pub fn take_snapshot(&mut self, name: String, value: String, frame_index: usize) {
        let snapshot = VariableSnapshot {
            name,
            value,
            timestamp: std::time::SystemTime::now(),
            frame_index,
        };
        self.variable_history.push(snapshot);
    }
}

impl CallStackTracker {
    /// Update current call stack
    pub fn update_stack(&mut self, frames: Vec<StackFrame>) {
        self.current_stack = frames;
    }

    /// Take stack snapshot
    pub fn take_snapshot(&mut self, trigger: StackSnapshotTrigger) {
        let snapshot = CallStackSnapshot {
            frames: self.current_stack.clone(),
            timestamp: std::time::SystemTime::now(),
            trigger,
        };
        self.stack_history.push(snapshot);
    }

    /// Get stack history
    pub fn stack_history(&self) -> &[CallStackSnapshot] {
        &self.stack_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_interface_creation() {
        let debugger = DebuggerInterface::new();
        assert!(debugger.session.is_none());
        assert_eq!(debugger.execution_state(), ExecutionState::NotStarted);
    }

    #[test]
    fn test_breakpoint_management() {
        let mut debugger = DebuggerInterface::new();

        // Set breakpoint
        assert!(debugger.set_breakpoint("test.aether", 10).is_ok());
        assert_eq!(debugger.breakpoints().len(), 1);

        // Remove breakpoint
        assert!(debugger.remove_breakpoint(0).is_ok());
        assert_eq!(debugger.breakpoints().len(), 0);
    }

    #[test]
    fn test_variable_watching() {
        let mut debugger = DebuggerInterface::new();

        // Add watch
        assert!(debugger.add_watch("my_variable".to_string()).is_ok());
        assert_eq!(debugger.variable_inspector.watched_variables.len(), 1);

        // Remove watch
        assert!(debugger.remove_watch("my_variable").is_ok());
        assert_eq!(debugger.variable_inspector.watched_variables.len(), 0);
    }

    #[test]
    fn test_debugger_config() {
        let config = DebuggerConfig::default();
        assert_eq!(config.debugger_type, DebuggerType::Auto);
        assert!(config.auto_load_symbols);
        assert!(config.source_level_debugging);
        assert_eq!(config.max_stack_depth, 100);
    }

    #[test]
    fn test_stack_frame() {
        let frame = StackFrame {
            index: 0,
            function_name: "main".to_string(),
            location: SourceLocation {
                file: "test.aether".to_string(),
                line: 10,
                column: 5,
                offset: 0,
            },
            address: 0x1000,
            locals: vec![],
        };

        assert_eq!(frame.function_name, "main");
        assert_eq!(frame.address, 0x1000);
    }

    #[test]
    fn test_execution_states() {
        assert_ne!(ExecutionState::NotStarted, ExecutionState::Running);
        assert_ne!(ExecutionState::Running, ExecutionState::Paused);

        if let ExecutionState::Exited { exit_code } = (ExecutionState::Exited { exit_code: 0 }) {
            assert_eq!(exit_code, 0);
        }
    }
}
