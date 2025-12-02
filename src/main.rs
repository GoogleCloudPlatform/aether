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

//! AetherScript Compiler CLI
//!
//! Command-line interface for the AetherScript compiler

use aether::pipeline::CompileOptions;
use aether::Compiler;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

/// Format AST for human-readable display
fn format_ast_for_display(program: &aether::ast::Program) -> String {
    let mut output = String::new();
    output.push_str("Program {\n");

    for module in &program.modules {
        output.push_str(&format!("  Module '{}'\n", module.name.name));

        if let Some(intent) = &module.intent {
            output.push_str(&format!("    Intent: {}\n", intent));
        }

        // Format constants
        for constant in &module.constant_declarations {
            output.push_str(&format!(
                "    const {}: {}\n",
                constant.name.name,
                format_type(&constant.type_spec)
            ));
        }

        // Format functions
        for function in &module.function_definitions {
            output.push_str(&format!("    fn {}(", function.name.name));
            for (i, param) in function.parameters.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!(
                    "{}: {}",
                    param.name.name,
                    format_type(&param.param_type)
                ));
            }
            output.push_str(") -> ");
            output.push_str(&format_type(&function.return_type));
            output.push('\n');
        }
    }

    output.push_str("}\n");
    output
}

/// Format a type for display
fn format_type(type_spec: &aether::ast::TypeSpecifier) -> String {
    use aether::ast::{OwnershipKind, PrimitiveType};
    match type_spec {
        aether::ast::TypeSpecifier::Primitive { type_name, .. } => match type_name {
            PrimitiveType::Integer => "Integer".to_string(),
            PrimitiveType::Integer32 => "Integer32".to_string(),
            PrimitiveType::Integer64 => "Integer64".to_string(),
            PrimitiveType::Float => "Float".to_string(),
            PrimitiveType::Float32 => "Float32".to_string(),
            PrimitiveType::Float64 => "Float64".to_string(),
            PrimitiveType::String => "String".to_string(),
            PrimitiveType::Char => "Character".to_string(),
            PrimitiveType::Boolean => "Boolean".to_string(),
            PrimitiveType::Void => "Void".to_string(),
            PrimitiveType::SizeT => "SizeT".to_string(),
            PrimitiveType::UIntPtrT => "UIntPtrT".to_string(),
        },
        aether::ast::TypeSpecifier::Named { name, .. } => name.name.clone(),
        aether::ast::TypeSpecifier::Array { element_type, .. } => {
            format!("Array<{}>", format_type(element_type))
        }
        aether::ast::TypeSpecifier::Map {
            key_type,
            value_type,
            ..
        } => format!(
            "Map<{}, {}>",
            format_type(key_type),
            format_type(value_type)
        ),
        aether::ast::TypeSpecifier::Pointer {
            target_type,
            is_mutable,
            ..
        } => format!(
            "{}{}",
            if *is_mutable { "*mut " } else { "*" },
            format_type(target_type)
        ),
        aether::ast::TypeSpecifier::Function {
            parameter_types,
            return_type,
            ..
        } => {
            let param_strs: Vec<String> = parameter_types.iter().map(|t| format_type(t)).collect();
            format!(
                "({}) -> {}",
                param_strs.join(", "),
                format_type(return_type)
            )
        }
        aether::ast::TypeSpecifier::Generic {
            base_type,
            type_arguments,
            ..
        } => {
            format!(
                "{}<{}>",
                base_type.name,
                type_arguments
                    .iter()
                    .map(|t| format_type(t))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        aether::ast::TypeSpecifier::TypeParameter { name, .. } => name.name.clone(),
        aether::ast::TypeSpecifier::Owned {
            ownership,
            base_type,
            ..
        } => {
            let prefix = match ownership {
                OwnershipKind::Owned => "^",
                OwnershipKind::Borrowed => "&",
                OwnershipKind::BorrowedMut => "&mut ",
                OwnershipKind::Shared => "~",
            };
            format!("{}{}", prefix, format_type(base_type))
        }
    }
}

#[derive(Parser)]
#[command(name = "aether")]
#[command(about = "Compiler for the AetherScript programming language", long_about = None)]
#[command(version)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile AetherScript source files
    Compile {
        /// Input source file(s)
        #[arg(required = true)]
        input: Vec<PathBuf>,

        /// Output file name (defaults to first input file name without extension)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Optimization level (0-3)
        #[arg(short = 'O', long, default_value = "2")]
        optimization: u8,

        /// Generate debug information
        #[arg(short, long)]
        debug: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Keep intermediate files
        #[arg(long)]
        keep_intermediates: bool,

        /// Generate object file only (don't link)
        #[arg(short = 'c', long)]
        compile_only: bool,

        /// Compile as a library (shared object/dylib)
        #[arg(long)]
        library: bool,

        /// Additional library search paths
        #[arg(short = 'L', long = "library-path")]
        library_paths: Vec<PathBuf>,

        /// Link with library
        #[arg(short = 'l', long = "link")]
        link_libraries: Vec<String>,

        /// Enable formal verification of contracts (@pre/@post)
        #[arg(long)]
        verify: bool,
    },

    /// Check syntax without generating code
    Check {
        /// Input source file(s)
        #[arg(required = true)]
        input: Vec<PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run AetherScript program (compile and execute)
    Run {
        /// Input source file
        #[arg(required = true)]
        input: PathBuf,

        /// Program arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Print AST (Abstract Syntax Tree)
    Ast {
        /// Input source file
        #[arg(required = true)]
        input: PathBuf,

        /// Output directory (prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Print tokens
    Tokens {
        /// Input source file
        #[arg(required = true)]
        input: PathBuf,

        /// Output directory (prints to stdout if not specified)
        #[arg(short, long)]
        output: Option<String>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Run LSP server
    Lsp,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Lsp) => {
            use aether::lsp::Backend;
            use tower_lsp::{LspService, Server};
            
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, socket) = LspService::new(|client| Backend::new(client));
            Server::new(stdin, stdout, socket).serve(service).await;
            
            Ok(aether::pipeline::CompilationResult {
                executable_path: PathBuf::new(),
                intermediate_files: vec![],
                stats: Default::default(),
                verification_results: None,
            })
        }

        Some(Commands::Compile {
            input,
            output,
            optimization,
            debug,
            verbose,
            keep_intermediates,
            compile_only,
            library,
            library_paths,
            link_libraries,
            verify,
        }) => {
            let mut options = CompileOptions::default();
            options.optimization_level = optimization.min(3);
            options.debug_info = debug;
            options.verbose = verbose;
            options.keep_intermediates = keep_intermediates;
            options.emit_object_only = compile_only;
            options.compile_as_library = library;
            options.library_paths = library_paths;
            options.link_libraries = link_libraries;
            options.enable_verification = verify;

            if let Some(output_path) = output {
                options.output = Some(output_path);
            }

            let compiler = Compiler::with_options(options);
            match compiler.compile_files(&input) {
                Ok(result) => {
                    println!("Compilation completed successfully");
                    if verbose || cli.verbose {
                        println!("Output: {}", result.executable_path.display());
                    }
                    Ok(result)
                }
                Err(e) => Err(e),
            }
        }

        Some(Commands::Check { input, verbose }) => {
            let mut options = CompileOptions::default();
            options.verbose = verbose || cli.verbose;
            options.debug_info = cli.debug;
            // syntax_only still runs semantic analysis, just skips code generation
            options.syntax_only = true;

            if cli.debug {
                eprintln!("[DEBUG] Starting syntax check for {} files", input.len());
            }

            let mut total_errors = 0;
            let mut files_passed = 0;
            let mut files_failed = 0;

            if verbose || cli.verbose {
                eprintln!("[VERBOSE] Type checking {} file(s)", input.len());
            }

            // Check each file
            for file in &input {
                if verbose || cli.verbose {
                    println!("[VERBOSE] Checking {}...", file.display());
                }

                // Check if file exists
                if !file.exists() {
                    eprintln!("Error: File '{}' not found", file.display());
                    files_failed += 1;
                    total_errors += 1;
                    continue;
                }

                let compiler = Compiler::with_options(options.clone());
                match compiler.compile_files(&[file.clone()]) {
                    Ok(_) => {
                        files_passed += 1;
                        if verbose || cli.verbose {
                            println!("✓ {} - OK", file.display());
                        }
                    }
                    Err(e) => {
                        files_failed += 1;
                        total_errors += 1;
                        // Always print the error details, not just in verbose mode
                        eprintln!("Error in {}: {}", file.display(), e);
                        if verbose || cli.verbose {
                            println!("✗ {} - Error: {}", file.display(), e);
                        }
                        // Continue checking other files
                    }
                }
            }

            // Print summary
            if files_failed == 0 {
                println!("Type checking passed");
                println!("Files passed: {}", files_passed);
                println!("Total errors: 0");
                Ok(aether::pipeline::CompilationResult {
                    executable_path: PathBuf::new(),
                    intermediate_files: vec![],
                    stats: Default::default(),
                    verification_results: None,
                })
            } else {
                println!("Type checking failed");
                println!("Files passed: {}", files_passed);
                println!("Files with errors: {}", files_failed);
                println!("Total errors: {}", total_errors);
                Err(aether::error::CompilerError::SemanticError(
                    aether::error::SemanticError::TypeMismatch {
                        expected: "valid".to_string(),
                        found: "errors".to_string(),
                        location: aether::error::SourceLocation::unknown(),
                    },
                ))
            }
        }

        Some(Commands::Run {
            input,
            args,
            verbose,
        }) => {
            // First compile the program
            let mut options = CompileOptions::default();
            options.verbose = verbose;
            options.optimization_level = 2;

            let compiler = Compiler::with_options(options);
            match compiler.compile_files(&[input]) {
                Ok(result) => {
                    // Execute the compiled program
                    // Ensure path is absolute or has ./ prefix for current directory
                    let exe_path = if result.executable_path.is_absolute() {
                        result.executable_path.clone()
                    } else {
                        std::env::current_dir()
                            .unwrap_or_default()
                            .join(&result.executable_path)
                    };
                    let mut cmd = process::Command::new(&exe_path);
                    cmd.args(&args);

                    match cmd.status() {
                        Ok(status) => {
                            if !status.success() {
                                process::exit(status.code().unwrap_or(1));
                            }
                            Ok(result)
                        }
                        Err(e) => {
                            eprintln!("Failed to execute program: {}", e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => Err(e),
            }
        }

        Some(Commands::Ast {
            input,
            output,
            verbose: _,
        }) => {
            use aether::lexer::v2::Lexer as LexerV2;
            use aether::parser::v2::Parser as ParserV2;
            use std::fs;

            let content = match fs::read_to_string(&input) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", input.display(), e);
                    process::exit(1);
                }
            };

            // Parse using V2 parser
            let mut lexer = LexerV2::new(&content, input.display().to_string());
            let tokens = match lexer.tokenize() {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("Lexer error: {:?}", e);
                    process::exit(1);
                }
            };
            let mut parser = ParserV2::new(tokens);
            let ast = parser.parse_program();

            match ast {
                Ok(ast) => {
                    let output_content = format_ast_for_display(&ast);

                    if let Some(output_dir) = output {
                        let output_path = std::path::Path::new(&output_dir)
                            .join(input.file_stem().unwrap())
                            .with_extension("ast");
                        fs::create_dir_all(&output_dir).unwrap();
                        fs::write(output_path, output_content).unwrap();
                    } else {
                        println!("{}", output_content);
                    }
                    Ok(aether::pipeline::CompilationResult {
                        executable_path: PathBuf::new(),
                        intermediate_files: vec![],
                        stats: Default::default(),
                        verification_results: None,
                    })
                }
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    process::exit(1);
                }
            }
        }

        Some(Commands::Tokens {
            input,
            output,
            verbose: _,
        }) => {
            use aether::lexer::v2::Lexer as LexerV2;
            use std::fs;

            let content = match fs::read_to_string(&input) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", input.display(), e);
                    process::exit(1);
                }
            };

            let mut token_output = String::new();
            token_output.push_str(&format!("Tokens for {}:\n", input.display()));
            token_output.push_str("=================\n");

            let mut lexer = LexerV2::new(&content, input.display().to_string());
            match lexer.tokenize() {
                Ok(tokens) => {
                    for token in tokens {
                        if matches!(token.token_type, aether::lexer::v2::TokenType::Eof) {
                            break;
                        }
                        token_output.push_str(&format!(
                            "{:?} at {}:{}\n",
                            token.token_type, token.location.line, token.location.column
                        ));
                    }
                }
                Err(e) => {
                    eprintln!("Lexer error: {:?}", e);
                    process::exit(1);
                }
            }

            let output_content = token_output;

            if let Some(output_dir) = output {
                let output_path = std::path::Path::new(&output_dir)
                    .join(input.file_stem().unwrap())
                    .with_extension("tokens");
                fs::create_dir_all(&output_dir).unwrap();
                fs::write(output_path, output_content).unwrap();
            } else {
                println!("{}", output_content);
            }

            Ok(aether::pipeline::CompilationResult {
                executable_path: PathBuf::new(),
                intermediate_files: vec![],
                stats: Default::default(),
                verification_results: None,
            })
        }

        None => {
            // No subcommand provided - print error and help
            eprintln!("Error: No subcommand provided");
            eprintln!();
            use clap::CommandFactory;
            let mut cmd = Cli::command();
            cmd.print_help().unwrap();
            process::exit(1);
        }
    };

    match result {
        Ok(_) => {
            // Success
        }
        Err(e) => {
            eprintln!("Compilation failed: {}", e);
            process::exit(1);
        }
    }
}
