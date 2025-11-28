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

//! AetherScript Compiler Library
//!
//! A compiler for the AetherScript programming language, designed for
//! optimal generation by Large Language Models (LLMs).

pub mod ast;
pub mod codegen;
pub mod concurrency;
pub mod contracts;
pub mod debug;
pub mod docs;
pub mod error;
pub mod ffi;
pub mod lexer;
pub mod llvm_backend;
pub mod lsp;
pub mod memory;
pub mod mir;
pub mod module_loader;
pub mod optimizations;
pub mod package;
pub mod parser;
pub mod patterns;
pub mod pipeline;
pub mod profiling;
pub mod release;
pub mod resource;
pub mod runtime;
pub mod semantic;
pub mod stdlib;
pub mod symbols;
pub mod types;
pub mod utils;
pub mod verification;

// External dependency stubs
pub mod external_stubs;

use crate::error::CompilerError;
use crate::pipeline::{CompilationPipeline, CompilationResult, CompileOptions};
use std::path::PathBuf;

/// The main compiler interface
pub struct Compiler {
    options: CompileOptions,
}

impl Compiler {
    /// Create a new compiler instance
    pub fn new() -> Self {
        Self {
            options: CompileOptions::default(),
        }
    }

    /// Create a compiler with custom options
    pub fn with_options(options: CompileOptions) -> Self {
        Self { options }
    }

    /// Set optimization level (0-3)
    pub fn optimization_level(mut self, level: u8) -> Self {
        self.options.optimization_level = level.min(3);
        self
    }

    /// Enable debug info generation
    pub fn debug_info(mut self, enable: bool) -> Self {
        self.options.debug_info = enable;
        self
    }

    /// Set output file path
    pub fn output(mut self, path: PathBuf) -> Self {
        self.options.output = Some(path);
        self
    }

    /// Set target triple
    pub fn target(mut self, triple: String) -> Self {
        self.options.target_triple = Some(triple);
        self
    }

    /// Add library search path
    pub fn library_path(mut self, path: PathBuf) -> Self {
        self.options.library_paths.push(path);
        self
    }

    /// Add library to link
    pub fn link_library(mut self, lib: String) -> Self {
        self.options.link_libraries.push(lib);
        self
    }

    /// Enable verbose output
    pub fn verbose(mut self, enable: bool) -> Self {
        self.options.verbose = enable;
        self
    }

    /// Keep intermediate files
    pub fn keep_intermediates(mut self, enable: bool) -> Self {
        self.options.keep_intermediates = enable;
        self
    }

    /// Enable profiling
    pub fn profile(mut self, enable: bool) -> Self {
        self.options.enable_profiling = enable;
        self
    }

    /// Enable or disable parallel compilation
    pub fn parallel(mut self, enable: bool) -> Self {
        self.options.parallel = enable;
        self
    }

    /// Compile a single source file
    pub fn compile_file(&self, input: PathBuf) -> Result<CompilationResult, CompilerError> {
        self.compile_files(&[input])
    }

    /// Compile multiple source files
    pub fn compile_files(&self, inputs: &[PathBuf]) -> Result<CompilationResult, CompilerError> {
        let mut pipeline = CompilationPipeline::new(self.options.clone());
        pipeline.compile_files(inputs)
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}
