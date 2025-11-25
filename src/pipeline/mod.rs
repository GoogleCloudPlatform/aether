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

//! End-to-end compilation pipeline
//!
//! Integrates all compiler phases from source code to executable

use crate::ast::{Module, Program};
use crate::error::{CompilerError, SemanticError};
use crate::lexer::v2 as lexer_v2;
use crate::llvm_backend::LLVMBackend;
use crate::mir;
use crate::optimizations::OptimizationManager;
use crate::parser::v2 as parser_v2;
use crate::profiling::CompilationProfiler;
use crate::semantic::SemanticAnalyzer;
use crate::stdlib::StandardLibrary;

use inkwell::context::Context;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
// use std::sync::{Arc, Mutex};

/// Compilation options
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Output file path
    pub output: Option<PathBuf>,
    /// Optimization level (0-3)
    pub optimization_level: u8,
    /// Generate debug information
    pub debug_info: bool,
    /// Target triple (e.g., "x86_64-pc-linux-gnu")
    pub target_triple: Option<String>,
    /// Additional library paths
    pub library_paths: Vec<PathBuf>,
    /// Additional libraries to link
    pub link_libraries: Vec<String>,
    /// Verbose output
    pub verbose: bool,
    /// Keep intermediate files
    pub keep_intermediates: bool,
    /// Enable profiling
    pub enable_profiling: bool,
    /// Enable parallel compilation
    pub parallel: bool,
    /// Emit object file only (don't link)
    pub emit_object_only: bool,
    /// Check syntax only (don't generate code)
    pub syntax_only: bool,
    /// Compile as a library (shared object/dylib)
    pub compile_as_library: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            output: None,
            optimization_level: 2,
            debug_info: false,
            target_triple: None,
            library_paths: vec![],
            link_libraries: vec![],
            verbose: false,
            keep_intermediates: false,
            enable_profiling: false,
            parallel: true, // Enable parallel compilation by default
            emit_object_only: false,
            syntax_only: false,
            compile_as_library: false,
        }
    }
}

/// Compilation pipeline result
#[derive(Debug)]
pub struct CompilationResult {
    /// Output executable path
    pub executable_path: PathBuf,
    /// Intermediate files generated (if kept)
    pub intermediate_files: Vec<PathBuf>,
    /// Compilation statistics
    pub stats: CompilationStats,
}

/// Compilation statistics
#[derive(Debug, Default)]
pub struct CompilationStats {
    /// Lines of source code
    pub lines_of_code: usize,
    /// Number of modules compiled
    pub modules_compiled: usize,
    /// Number of functions compiled
    pub functions_compiled: usize,
    /// Total compilation time in milliseconds
    pub total_time_ms: u128,
    /// Time spent in each phase
    pub phase_times: std::collections::HashMap<String, u128>,
}

/// Main compilation pipeline
pub struct CompilationPipeline {
    options: CompileOptions,
    stdlib: StandardLibrary,
}

impl CompilationPipeline {
    /// Create a new compilation pipeline
    pub fn new(options: CompileOptions) -> Self {
        Self {
            options,
            stdlib: StandardLibrary::new(),
        }
    }

    /// Parse a single source file using V2 parser
    fn parse_file(&self, path: &Path, source: &str) -> Result<Module, CompilerError> {
        Self::parse_source(path, source)
    }

    /// Parse source code using V2 parser (static for use in parallel contexts)
    fn parse_source(path: &Path, source: &str) -> Result<Module, CompilerError> {
        let filename = path.to_string_lossy().to_string();

        // Use V2 lexer and parser (Swift/Rust-like syntax)
        let mut lexer = lexer_v2::Lexer::new(source, filename);
        let tokens = lexer.tokenize()?;
        let mut parser = parser_v2::Parser::new(tokens);
        parser.parse_module().map_err(CompilerError::from)
    }

    /// Compile multiple source files into a single executable
    pub fn compile_files(
        &mut self,
        input_files: &[PathBuf],
    ) -> Result<CompilationResult, CompilerError> {
        let start_time = std::time::Instant::now();
        let mut stats = CompilationStats::default();
        let mut intermediate_files = Vec::new();

        // Initialize profiler if enabled
        let mut profiler = CompilationProfiler::new();
        if self.options.enable_profiling {
            profiler.start_compilation();
        }

        // Phase 1: Parse all source files
        if self.options.verbose {
            println!("Phase 1: Parsing source files...");
        }
        let parse_start = std::time::Instant::now();
        let program = {
            let _timer = if self.options.enable_profiling {
                Some(profiler.start_phase("parsing"))
            } else {
                None
            };

            // Decide whether to use parallel or sequential parsing
            let modules = if self.options.parallel && input_files.len() > 1 {
                // Parallel parsing
                let results: Result<Vec<_>, _> = input_files
                    .par_iter()
                    .map(|input_file| {
                        // Read file
                        let source =
                            fs::read_to_string(input_file).map_err(|e| CompilerError::IoError {
                                message: format!("Failed to read {}: {}", input_file.display(), e),
                            })?;

                        let lines = source.lines().count();

                        // Parse with V2 parser
                        let module = Self::parse_source(input_file, &source)?;

                        Ok::<(Module, usize), CompilerError>((module, lines))
                    })
                    .collect();

                let parsed_modules = results?;

                // Update stats
                for (_, lines) in &parsed_modules {
                    stats.lines_of_code += lines;
                }

                parsed_modules.into_iter().map(|(m, _)| m).collect()
            } else {
                // Sequential parsing for single file or when parallel is disabled
                let mut modules = vec![];

                for input_file in input_files {
                    // Read file
                    let source =
                        fs::read_to_string(input_file).map_err(|e| CompilerError::IoError {
                            message: format!("Failed to read {}: {}", input_file.display(), e),
                        })?;

                    stats.lines_of_code += source.lines().count();

                    // Parse with appropriate syntax version
                    let module = self.parse_file(input_file, &source)?;

                    modules.push(module);
                }

                modules
            };

            Program {
                modules,
                source_location: crate::error::SourceLocation::unknown(),
            }
        };

        stats.modules_compiled = program.modules.len();
        stats
            .phase_times
            .insert("parsing".to_string(), parse_start.elapsed().as_millis());

        if self.options.enable_profiling {
            profiler.snapshot_memory("after_parsing");
        }

        // Phase 2: Semantic analysis
        if self.options.verbose {
            println!("Phase 2: Semantic analysis...");
        }
        let semantic_start = std::time::Instant::now();

        let symbol_table = {
            let _timer = if self.options.enable_profiling {
                Some(profiler.start_phase("semantic_analysis"))
            } else {
                None
            };

            let mut analyzer = SemanticAnalyzer::new();
            analyzer.analyze_program(&program)?;

            let analysis_stats = analyzer.get_statistics().clone();
            stats.functions_compiled = analysis_stats.functions_analyzed;

            // Extract symbol table for MIR lowering
            analyzer.get_symbol_table()
        };

        stats.phase_times.insert(
            "semantic_analysis".to_string(),
            semantic_start.elapsed().as_millis(),
        );

        if self.options.enable_profiling {
            profiler.snapshot_memory("after_semantic_analysis");
        }

        // If syntax-only mode, stop here
        if self.options.syntax_only {
            if self.options.verbose {
                println!("\nSyntax check completed successfully!");
            }

            stats.total_time_ms = start_time.elapsed().as_millis();

            // Return dummy result for syntax check
            return Ok(CompilationResult {
                executable_path: PathBuf::from("syntax-check-only"),
                intermediate_files,
                stats,
            });
        }

        // Phase 3: MIR generation
        if self.options.verbose {
            println!("Phase 3: Generating intermediate representation...");
        }
        let mir_start = std::time::Instant::now();

        let mut mir_program = {
            let _timer = if self.options.enable_profiling {
                Some(profiler.start_phase("mir_generation"))
            } else {
                None
            };

            eprintln!("AST has {} modules", program.modules.len());
            for module in &program.modules {
                eprintln!(
                    "  Module '{}' has {} functions",
                    module.name.name,
                    module.function_definitions.len()
                );
                for (idx, func) in module.function_definitions.iter().enumerate() {
                    eprintln!("    Function {}: {}", idx, func.name.name);
                }
                eprintln!(
                    "  Module '{}' has {} constants",
                    module.name.name,
                    module.constant_declarations.len()
                );
                for (idx, constant) in module.constant_declarations.iter().enumerate() {
                    eprintln!("    Constant {}: {}", idx, constant.name.name);
                }
            }

            mir::lowering::lower_ast_to_mir_with_symbols(&program, symbol_table)?
        };

        stats.phase_times.insert(
            "mir_generation".to_string(),
            mir_start.elapsed().as_millis(),
        );

        if self.options.enable_profiling {
            profiler.snapshot_memory("after_mir_generation");
        }

        // Phase 4: Optimization
        if self.options.verbose {
            println!("Phase 4: Running optimizations...");
        }
        let opt_start = std::time::Instant::now();

        if self.options.optimization_level > 0 {
            let _timer = if self.options.enable_profiling {
                Some(profiler.start_phase("optimization"))
            } else {
                None
            };

            let mut opt_manager = OptimizationManager::new();
            // Set up optimization passes based on level
            if self.options.optimization_level > 0 {
                opt_manager = OptimizationManager::create_default_pipeline();
            }
            opt_manager.optimize_program(&mut mir_program)?;
        }

        stats
            .phase_times
            .insert("optimization".to_string(), opt_start.elapsed().as_millis());

        if self.options.enable_profiling {
            profiler.snapshot_memory("after_optimization");
        }

        // Phase 5: LLVM code generation
        if self.options.verbose {
            println!("Phase 5: Generating LLVM IR...");
        }
        let codegen_start = std::time::Instant::now();

        let context = Context::create();
        let module_name = input_files
            .first()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .unwrap_or("main");

        let mut backend = LLVMBackend::new(&context, module_name);

        {
            let _timer = if self.options.enable_profiling {
                Some(profiler.start_phase("llvm_codegen"))
            } else {
                None
            };

            // Initialize LLVM targets
            LLVMBackend::initialize_targets();

            // Set target triple - use specified or native
            let target_triple = self.options.target_triple.clone().unwrap_or_else(|| {
                use crate::llvm_backend::TargetArch;
                TargetArch::native().target_triple().to_string()
            });
            backend.set_target_triple(&target_triple)?;

            // Generate LLVM IR from MIR
            backend.generate_ir(&mir_program)?;
        }

        stats.phase_times.insert(
            "llvm_codegen".to_string(),
            codegen_start.elapsed().as_millis(),
        );

        if self.options.enable_profiling {
            profiler.snapshot_memory("after_llvm_codegen");
        }

        // Phase 6: Object file generation
        if self.options.verbose {
            println!("Phase 6: Generating object file...");
        }
        let object_start = std::time::Instant::now();

        let object_file = self.generate_object_file(&backend, module_name)?;
        if self.options.keep_intermediates {
            intermediate_files.push(object_file.clone());
        }

        stats.phase_times.insert(
            "object_generation".to_string(),
            object_start.elapsed().as_millis(),
        );

        // Check if output is object file only
        let output_is_object = self
            .options
            .output
            .as_ref()
            .map(|p| p.extension().map(|e| e == "o").unwrap_or(false))
            .unwrap_or(false);

        let executable_path = if output_is_object || self.options.emit_object_only {
            // Just copy the object file to the output path
            let output_path = self
                .options
                .output
                .clone()
                .unwrap_or_else(|| object_file.clone());
            if object_file != output_path {
                fs::copy(&object_file, &output_path).map_err(|e| CompilerError::IoError {
                    message: format!("Failed to copy object file: {}", e),
                })?;
            }
            output_path
        } else {
            // Phase 7: Linking
            if self.options.verbose {
                if self.options.compile_as_library {
                    println!("Phase 7: Linking library...");
                } else {
                    println!("Phase 7: Linking executable...");
                }
            }
            let link_start = std::time::Instant::now();

            let output_path = if self.options.compile_as_library {
                self.link_library(&object_file, module_name)?
            } else {
                self.link_executable(&object_file, module_name)?
            };

            stats
                .phase_times
                .insert("linking".to_string(), link_start.elapsed().as_millis());

            output_path
        };

        // Clean up intermediate files if not keeping them
        if !self.options.keep_intermediates {
            for file in &intermediate_files {
                let _ = fs::remove_file(file);
            }
            intermediate_files.clear();
        }

        stats.total_time_ms = start_time.elapsed().as_millis();

        if self.options.verbose {
            println!("\nCompilation completed successfully!");
            println!("Total time: {}ms", stats.total_time_ms);
            println!("Output: {}", executable_path.display());
        }

        // Print profiling report if enabled
        if self.options.enable_profiling {
            profiler.print_summary();
        }

        Ok(CompilationResult {
            executable_path,
            intermediate_files,
            stats,
        })
    }

    /// Generate object file from LLVM module
    fn generate_object_file(
        &self,
        backend: &LLVMBackend,
        base_name: &str,
    ) -> Result<PathBuf, CompilerError> {
        let object_path = PathBuf::from(format!("{}.o", base_name));

        // Write object file
        backend.write_object_file(&object_path)?;

        Ok(object_path)
    }

    /// Link object file(s) into executable
    fn link_executable(
        &self,
        object_file: &Path,
        base_name: &str,
    ) -> Result<PathBuf, CompilerError> {
        let output_path = self
            .options
            .output
            .clone()
            .unwrap_or_else(|| PathBuf::from(base_name));

        // Use system linker (ld or clang)
        let mut cmd = if cfg!(target_os = "macos") {
            Command::new("clang")
        } else {
            Command::new("cc")
        };

        cmd.arg("-o").arg(&output_path);
        cmd.arg(object_file);

        // Add library paths
        for lib_path in &self.options.library_paths {
            cmd.arg(format!("-L{}", lib_path.display()));
        }

        // Add libraries
        for lib in &self.options.link_libraries {
            cmd.arg(format!("-l{}", lib));
        }

        // Add AetherScript runtime library directly
        let runtime_lib_path = PathBuf::from("/Users/keithballinger/Desktop/projects/logos/runtime/target/release/libaether_runtime.dylib");

        cmd.arg(&runtime_lib_path);

        // Add standard C library
        if !self.options.link_libraries.contains(&"c".to_string()) {
            cmd.arg("-lc");
        }

        // Add math library if needed
        if !self.options.link_libraries.contains(&"m".to_string()) {
            cmd.arg("-lm");
        }

        if self.options.verbose {
            println!("Linking command: {:?}", cmd);
        }

        let output = cmd.output().map_err(|e| CompilerError::IoError {
            message: format!("Failed to run linker: {}", e),
        })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(SemanticError::InvalidType {
                type_name: "linking".to_string(),
                reason: format!("linker error: {}", error_msg),
                location: crate::error::SourceLocation::unknown(),
            }
            .into());
        }

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&output_path)
                .map_err(|e| CompilerError::IoError {
                    message: format!(
                        "Failed to get metadata for {}: {}",
                        output_path.display(),
                        e
                    ),
                })?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&output_path, perms).map_err(|e| CompilerError::IoError {
                message: format!(
                    "Failed to set permissions on {}: {}",
                    output_path.display(),
                    e
                ),
            })?;
        }

        Ok(output_path)
    }

    /// Link object file(s) into a shared library
    fn link_library(&self, object_file: &Path, base_name: &str) -> Result<PathBuf, CompilerError> {
        let lib_extension = if cfg!(target_os = "macos") {
            "dylib"
        } else if cfg!(target_os = "windows") {
            "dll"
        } else {
            "so"
        };

        let lib_prefix = if cfg!(target_os = "windows") {
            ""
        } else {
            "lib"
        };

        let output_path = self.options.output.clone().unwrap_or_else(|| {
            PathBuf::from(format!("{}{}.{}", lib_prefix, base_name, lib_extension))
        });

        // Use system linker to create shared library
        let mut cmd = if cfg!(target_os = "macos") {
            let mut cmd = Command::new("clang");
            cmd.arg("-dynamiclib");
            cmd.arg("-install_name").arg(format!(
                "@rpath/{}",
                output_path.file_name().unwrap().to_string_lossy()
            ));
            cmd
        } else if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cl");
            cmd.arg("/LD");
            cmd
        } else {
            let mut cmd = Command::new("cc");
            cmd.arg("-shared");
            cmd.arg("-fPIC");
            cmd
        };

        cmd.arg("-o").arg(&output_path);
        cmd.arg(object_file);

        // Add library paths
        for lib_path in &self.options.library_paths {
            cmd.arg(format!("-L{}", lib_path.display()));
        }

        // Add libraries
        for lib in &self.options.link_libraries {
            cmd.arg(format!("-l{}", lib));
        }

        // Add AetherScript runtime library
        let runtime_lib_path = PathBuf::from("/Users/keithballinger/Desktop/projects/logos/runtime/target/release/libaether_runtime.dylib");
        cmd.arg(&runtime_lib_path);

        // Add standard C library
        if !self.options.link_libraries.contains(&"c".to_string()) {
            cmd.arg("-lc");
        }

        if self.options.verbose {
            println!("Library linking command: {:?}", cmd);
        }

        let output = cmd.output().map_err(|e| CompilerError::IoError {
            message: format!("Failed to run linker: {}", e),
        })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(SemanticError::InvalidType {
                type_name: "linking".to_string(),
                reason: format!("linker error: {}", error_msg),
                location: crate::error::SourceLocation::unknown(),
            }
            .into());
        }

        Ok(output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_options_default() {
        let opts = CompileOptions::default();
        assert_eq!(opts.optimization_level, 2);
        assert!(!opts.debug_info);
        assert!(!opts.verbose);
    }

    #[test]
    fn test_compilation_stats() {
        let mut stats = CompilationStats::default();
        stats.lines_of_code = 100;
        stats.modules_compiled = 2;
        stats.functions_compiled = 10;

        assert_eq!(stats.lines_of_code, 100);
        assert_eq!(stats.modules_compiled, 2);
        assert_eq!(stats.functions_compiled, 10);
    }

    #[test]
    fn test_pipeline_creation() {
        let opts = CompileOptions {
            verbose: true,
            optimization_level: 3,
            ..Default::default()
        };

        let pipeline = CompilationPipeline::new(opts);
        assert_eq!(pipeline.options.optimization_level, 3);
        assert!(pipeline.options.verbose);
    }

    #[test]
    fn test_parse_source_v2() {
        use std::path::Path;

        let v2_source = "module test;";
        let result = CompilationPipeline::parse_source(Path::new("test.aes"), v2_source);

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name.name, "test");
    }

    #[test]
    fn test_parse_source_v2_with_function() {
        use std::path::Path;

        let v2_source = r#"
module test;

func add(a: Int, b: Int) -> Int {
    return {a + b};
}
"#;
        let result = CompilationPipeline::parse_source(Path::new("test.aes"), v2_source);

        assert!(result.is_ok());
        let module = result.unwrap();
        assert_eq!(module.name.name, "test");
        assert_eq!(module.function_definitions.len(), 1);
        assert_eq!(module.function_definitions[0].name.name, "add");
    }
}
