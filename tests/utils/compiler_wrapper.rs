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

//! Compiler wrapper for integration testing

use aether::Compiler;
use aether::pipeline::CompilationResult;
use aether::pipeline::CompileOptions;
use aether::error::CompilerError;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::fs;

/// Wrapper around the AetherScript compiler for testing
pub struct TestCompiler {
    pub temp_dir: PathBuf,
    pub verbose: bool,
    pub optimization_level: u8,
    pub debug_info: bool,
}

impl TestCompiler {
    /// Create a new test compiler instance
    pub fn new(test_name: &str) -> Self {
        let temp_dir = super::create_temp_dir(test_name);
        
        Self {
            temp_dir,
            verbose: false,
            optimization_level: 0,
            debug_info: true,
        }
    }
    
    /// Enable verbose output
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
    
    /// Compile a single file
    pub fn compile_file<P: AsRef<Path>>(&self, input: P) -> TestCompilationResult {
        let input_path = input.as_ref();
        
        let compiler = Compiler::new()
            .optimization_level(self.optimization_level)
            .debug_info(self.debug_info)
            .verbose(self.verbose);
        
        let start_time = std::time::Instant::now();
        let result = compiler.compile_file(input_path.to_path_buf());
        let compilation_time = start_time.elapsed();
        
        TestCompilationResult {
            result,
            compilation_time,
            temp_dir: self.temp_dir.clone(),
            verbose: self.verbose,
        }
    }
    
    /// Compile multiple files
    pub fn compile_files<P: AsRef<Path>>(&self, inputs: &[P]) -> TestCompilationResult {
        let input_paths: Vec<PathBuf> = inputs.iter().map(|p| p.as_ref().to_path_buf()).collect();
        
        let compiler = Compiler::new()
            .optimization_level(self.optimization_level)
            .debug_info(self.debug_info)
            .verbose(self.verbose);
        
        let start_time = std::time::Instant::now();
        let result = compiler.compile_files(&input_paths);
        let compilation_time = start_time.elapsed();
        
        TestCompilationResult {
            result,
            compilation_time,
            temp_dir: self.temp_dir.clone(),
            verbose: self.verbose,
        }
    }
    
    /// Compile source code from string
    pub fn compile_source(&self, source: &str, filename: &str) -> TestCompilationResult {
        let source_file = self.temp_dir.join(filename);
        fs::write(&source_file, source).expect("Failed to write source file");
        self.compile_files(&[source_file])
    }
    
    /// Compile a multi-file project
    pub fn compile_project(&self, files: &[(&str, &str)]) -> TestCompilationResult {
        let project_files: Vec<PathBuf> = files.iter().map(|(filename, content)| {
            let file_path = self.temp_dir.join(filename);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create directory");
            }
            fs::write(&file_path, content).expect("Failed to write file");
            file_path
        }).collect();
        
        self.compile_files(&project_files)
    }
    
    /// Get temporary directory
    pub fn temp_dir(&self) -> &Path {
        &self.temp_dir
    }
}

impl Drop for TestCompiler {
    fn drop(&mut self) {
        if !self.verbose {
            super::cleanup_temp_dir(&self.temp_dir);
        }
    }
}

/// Result of a test compilation
pub struct TestCompilationResult {
    pub result: Result<CompilationResult, CompilerError>,
    pub compilation_time: std::time::Duration,
    pub temp_dir: PathBuf,
    pub verbose: bool,
}

impl TestCompilationResult {
    /// Check if compilation succeeded
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }
    
    /// Check if compilation failed
    pub fn is_failure(&self) -> bool {
        self.result.is_err()
    }
    
    /// Get the compilation result
    pub fn result(&self) -> &Result<CompilationResult, CompilerError> {
        &self.result
    }
    
    /// Get compilation error if any
    pub fn error(&self) -> Option<&CompilerError> {
        match &self.result {
            Err(e) => Some(e),
            Ok(_) => None,
        }
    }
    
    /// Get compilation success result if any
    pub fn success(&self) -> Option<&CompilationResult> {
        match &self.result {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    }
    
    /// Execute the compiled program
    pub fn execute(&self) -> ExecutionResult {
        if let Ok(compilation_result) = &self.result {
            let executable_path = &compilation_result.executable_path;
            
            if executable_path.exists() {
                let start_time = std::time::Instant::now();
                let output = Command::new(executable_path)
                    .output();
                let execution_time = start_time.elapsed();
                
                match output {
                    Ok(output) => ExecutionResult {
                        success: output.status.success(),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                        exit_code: output.status.code().unwrap_or(-1),
                        execution_time,
                    },
                    Err(e) => ExecutionResult {
                        success: false,
                        stdout: String::new(),
                        stderr: format!("Failed to execute: {}", e),
                        exit_code: -1,
                        execution_time,
                    }
                }
            } else {
                ExecutionResult {
                    success: false,
                    stdout: String::new(),
                    stderr: "Executable not found".to_string(),
                    exit_code: -1,
                    execution_time: std::time::Duration::from_secs(0),
                }
            }
        } else {
            ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: "Compilation failed".to_string(),
                exit_code: -1,
                execution_time: std::time::Duration::from_secs(0),
            }
        }
    }
    
    /// Print compilation details if verbose
    pub fn print_details(&self) {
        if self.verbose {
            println!("Compilation time: {:?}", self.compilation_time);
            match &self.result {
                Ok(result) => {
                    println!("Compilation successful");
                    println!("Output file: {}", result.executable_path.display());
                    // Note: Current CompilationResult doesn't track warnings
                    // This could be added in the future if needed
                }
                Err(error) => {
                    println!("Compilation failed: {}", error);
                }
            }
        }
    }
}

/// Result of program execution
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time: std::time::Duration,
}

impl ExecutionResult {
    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
    
    /// Get standard output
    pub fn stdout(&self) -> &str {
        &self.stdout
    }
    
    /// Get standard error
    pub fn stderr(&self) -> &str {
        &self.stderr
    }
    
    /// Get exit code
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }
    
    /// Print execution details
    pub fn print_details(&self) {
        println!("Execution time: {:?}", self.execution_time);
        println!("Exit code: {}", self.exit_code);
        if !self.stdout.is_empty() {
            println!("Stdout:\n{}", self.stdout);
        }
        if !self.stderr.is_empty() {
            println!("Stderr:\n{}", self.stderr);
        }
    }
}