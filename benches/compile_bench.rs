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

//! Compilation performance benchmarks
//!
//! Measures performance of various compilation phases

use aether::{pipeline::CompileOptions, Compiler};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test source file with specified complexity
fn create_test_source(num_functions: usize, statements_per_function: usize) -> String {
    let mut source = String::new();
    source.push_str("(DEFINE_MODULE\n");
    source.push_str("  (NAME benchmark_test)\n");
    source.push_str("  (INTENT \"Performance benchmark test\")\n");
    source.push_str("  (CONTENT\n");

    for i in 0..num_functions {
        source.push_str(&format!("    (DEFINE_FUNCTION\n"));
        source.push_str(&format!("      (NAME \"func_{}\")\n", i));
        source.push_str(&format!("      (INTENT \"Test function {}\")\n", i));
        source.push_str("      (RETURNS INTEGER)\n");
        source.push_str("      (BODY\n");

        // Create a chain of operations
        for j in 0..statements_per_function {
            if j == 0 {
                source
                    .push_str("        (DECLARE_VARIABLE (NAME \"x\") (TYPE INTEGER) (VALUE 0))\n");
            } else if j == statements_per_function - 1 {
                source.push_str("        (RETURN_VALUE x)\n");
            } else {
                source.push_str(&format!("        (ASSIGN (TARGET_VARIABLE x) (SOURCE_EXPRESSION (EXPRESSION_ADD x {})))\n", j));
            }
        }

        source.push_str("      ))\n");
    }

    // Add main function
    source.push_str("    (DEFINE_FUNCTION\n");
    source.push_str("      (NAME \"main\")\n");
    source.push_str("      (INTENT \"Main entry point\")\n");
    source.push_str("      (RETURNS INTEGER)\n");
    source.push_str("      (BODY\n");
    source.push_str("        (RETURN_VALUE 0)\n");
    source.push_str("      ))\n");

    source.push_str("  ))\n");
    source
}

/// Benchmark compilation of small programs
fn bench_small_program(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path().join("small.aether");
    let source = create_test_source(5, 10);
    fs::write(&source_path, source).unwrap();

    c.bench_function("compile_small_program", |b| {
        b.iter(|| {
            let compiler = Compiler::new().optimization_level(2);
            let _ = compiler.compile_files(&[black_box(source_path.clone())]);
        });
    });
}

/// Benchmark compilation of medium programs
fn bench_medium_program(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path().join("medium.aether");
    let source = create_test_source(20, 50);
    fs::write(&source_path, source).unwrap();

    c.bench_function("compile_medium_program", |b| {
        b.iter(|| {
            let compiler = Compiler::new().optimization_level(2);
            let _ = compiler.compile_files(&[black_box(source_path.clone())]);
        });
    });
}

/// Benchmark compilation of large programs
fn bench_large_program(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let source_path = temp_dir.path().join("large.aether");
    let source = create_test_source(100, 100);
    fs::write(&source_path, source).unwrap();

    c.bench_function("compile_large_program", |b| {
        b.iter(|| {
            let compiler = Compiler::new().optimization_level(2);
            let _ = compiler.compile_files(&[black_box(source_path.clone())]);
        });
    });
}

/// Benchmark lexing phase only
fn bench_lexing(c: &mut Criterion) {
    use aether::lexer::Lexer;

    let source = create_test_source(50, 50);

    c.bench_function("lexing_50_functions", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(&source), "bench.aether".to_string());
            let _ = lexer.tokenize();
        });
    });
}

/// Benchmark parsing phase only
fn bench_parsing(c: &mut Criterion) {
    use aether::lexer::Lexer;
    use aether::parser::Parser;

    let source = create_test_source(50, 50);
    let mut lexer = Lexer::new(&source, "bench.aether".to_string());
    let tokens = lexer.tokenize().unwrap();

    c.bench_function("parsing_50_functions", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(tokens.clone()));
            let _ = parser.parse_module();
        });
    });
}

/// Benchmark semantic analysis phase
fn bench_semantic_analysis(c: &mut Criterion) {
    use aether::ast::Program;
    use aether::lexer::Lexer;
    use aether::parser::Parser;
    use aether::semantic::SemanticAnalyzer;

    let source = create_test_source(50, 50);
    let mut lexer = Lexer::new(&source, "bench.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    let program = Program {
        modules: vec![module],
        source_location: aether::error::SourceLocation::unknown(),
    };

    c.bench_function("semantic_analysis_50_functions", |b| {
        b.iter(|| {
            let mut analyzer = SemanticAnalyzer::new();
            let _ = analyzer.analyze_program(black_box(&program));
        });
    });
}

/// Benchmark MIR generation
fn bench_mir_generation(c: &mut Criterion) {
    use aether::ast::Program;
    use aether::lexer::Lexer;
    use aether::mir;
    use aether::parser::Parser;
    use aether::semantic::SemanticAnalyzer;

    let source = create_test_source(50, 50);
    let mut lexer = Lexer::new(&source, "bench.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    let program = Program {
        modules: vec![module],
        source_location: aether::error::SourceLocation::unknown(),
    };

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).unwrap();

    c.bench_function("mir_generation_50_functions", |b| {
        b.iter(|| {
            let _ = mir::lowering::lower_ast_to_mir(black_box(&program));
        });
    });
}

/// Benchmark optimization passes
fn bench_optimization(c: &mut Criterion) {
    use aether::ast::Program;
    use aether::lexer::Lexer;
    use aether::mir;
    use aether::optimizations::OptimizationManager;
    use aether::parser::Parser;
    use aether::semantic::SemanticAnalyzer;

    let source = create_test_source(50, 50);
    let mut lexer = Lexer::new(&source, "bench.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let module = parser.parse_module().unwrap();

    let program = Program {
        modules: vec![module],
        source_location: aether::error::SourceLocation::unknown(),
    };

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).unwrap();

    let mir_program = mir::lowering::lower_ast_to_mir(&program).unwrap();

    c.bench_function("optimization_50_functions", |b| {
        b.iter(|| {
            let mut mir_copy = mir_program.clone();
            let mut opt_manager = OptimizationManager::create_default_pipeline();
            let _ = opt_manager.optimize_program(black_box(&mut mir_copy));
        });
    });
}

criterion_group!(
    benches,
    bench_small_program,
    bench_medium_program,
    bench_large_program,
    bench_lexing,
    bench_parsing,
    bench_semantic_analysis,
    bench_mir_generation,
    bench_optimization
);
criterion_main!(benches);
