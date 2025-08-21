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

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use aether::lexer::Lexer;
use aether::parser::Parser;
use aether::semantic::SemanticAnalyzer;
use std::fs;

/// Load test fixture
fn load_fixture(filename: &str) -> String {
    let path = format!("tests/fixtures/{}", filename);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {}", path))
}

/// Generate a large AetherScript module for benchmarking
fn generate_large_module(num_constants: usize) -> String {
    let mut content = String::from("(DEFINE_MODULE\n  (NAME 'benchmark_module')\n  (INTENT \"Large module for benchmarking\")\n  (CONTENT\n");
    
    for i in 0..num_constants {
        content.push_str(&format!(
            "    (DECLARE_CONSTANT\n      (NAME 'CONST_{}')\n      (TYPE INTEGER)\n      (VALUE {})\n      (INTENT \"Constant {}\")\n    )\n",
            i, i, i
        ));
    }
    
    content.push_str("  )\n)");
    content
}

/// Benchmark lexical analysis
fn bench_lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    
    let test_cases = vec![
        ("simple", load_fixture("simple_module.aether")),
        ("complex", load_fixture("complex_expressions.aether")),
        ("large", load_fixture("large_file.aether")),
    ];
    
    for (name, source) in test_cases {
        group.bench_with_input(BenchmarkId::new("tokenize", name), &source, |b, source| {
            b.iter(|| {
                let mut lexer = Lexer::new(black_box(source), "benchmark.aether".to_string());
                black_box(lexer.tokenize().unwrap())
            })
        });
    }
    
    // Benchmark with different sizes
    for size in [10, 50, 100, 500].iter() {
        let source = generate_large_module(*size);
        group.bench_with_input(
            BenchmarkId::new("tokenize_size", size), 
            &source, 
            |b, source| {
                b.iter(|| {
                    let mut lexer = Lexer::new(black_box(source), "benchmark.aether".to_string());
                    black_box(lexer.tokenize().unwrap())
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark parsing
fn bench_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    
    // Pre-tokenize test cases
    let test_cases: Vec<(_, Vec<_>)> = vec![
        ("simple", {
            let source = load_fixture("simple_module.aether");
            let mut lexer = Lexer::new(&source, "simple.aether".to_string());
            lexer.tokenize().unwrap()
        }),
        ("complex", {
            let source = load_fixture("complex_expressions.aether");
            let mut lexer = Lexer::new(&source, "complex.aether".to_string());
            lexer.tokenize().unwrap()
        }),
        ("large", {
            let source = load_fixture("large_file.aether");
            let mut lexer = Lexer::new(&source, "large.aether".to_string());
            lexer.tokenize().unwrap()
        }),
    ];
    
    for (name, tokens) in test_cases {
        group.bench_with_input(BenchmarkId::new("parse", name), &tokens, |b, tokens| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(tokens.clone()));
                black_box(parser.parse_program().unwrap())
            })
        });
    }
    
    // Benchmark with different sizes
    for size in [10, 50, 100, 500].iter() {
        let source = generate_large_module(*size);
        let mut lexer = Lexer::new(&source, "benchmark.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("parse_size", size), 
            &tokens, 
            |b, tokens| {
                b.iter(|| {
                    let mut parser = Parser::new(black_box(tokens.clone()));
                    black_box(parser.parse_program().unwrap())
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark semantic analysis
fn bench_semantic_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("semantic_analysis");
    
    // Pre-parse test cases
    let test_cases = vec![
        ("simple", {
            let source = load_fixture("simple_module.aether");
            let mut lexer = Lexer::new(&source, "simple.aether".to_string());
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            parser.parse_program().unwrap()
        }),
        ("complex", {
            let source = load_fixture("complex_expressions.aether");
            let mut lexer = Lexer::new(&source, "complex.aether".to_string());
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            parser.parse_program().unwrap()
        }),
        ("large", {
            let source = load_fixture("large_file.aether");
            let mut lexer = Lexer::new(&source, "large.aether".to_string());
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            parser.parse_program().unwrap()
        }),
    ];
    
    for (name, program) in test_cases {
        group.bench_with_input(BenchmarkId::new("analyze", name), &program, |b, program| {
            b.iter(|| {
                let mut analyzer = SemanticAnalyzer::new();
                analyzer.analyze_program(black_box(program)).unwrap();
                black_box(())
            })
        });
    }
    
    // Benchmark with different sizes
    for size in [10, 50, 100, 500].iter() {
        let source = generate_large_module(*size);
        let mut lexer = Lexer::new(&source, "benchmark.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        
        group.bench_with_input(
            BenchmarkId::new("analyze_size", size), 
            &program, 
            |b, program| {
                b.iter(|| {
                    let mut analyzer = SemanticAnalyzer::new();
                    analyzer.analyze_program(black_box(program)).unwrap();
                    black_box(())
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark complete compilation pipeline
fn bench_complete_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("complete_pipeline");
    
    let test_cases = vec![
        ("simple", load_fixture("simple_module.aether")),
        ("complex", load_fixture("complex_expressions.aether")),
        ("large", load_fixture("large_file.aether")),
    ];
    
    for (name, source) in test_cases {
        group.bench_with_input(BenchmarkId::new("full_compilation", name), &source, |b, source| {
            b.iter(|| {
                // Complete pipeline: lexing -> parsing -> semantic analysis
                let mut lexer = Lexer::new(black_box(source), "benchmark.aether".to_string());
                let tokens = lexer.tokenize().unwrap();
                
                let mut parser = Parser::new(tokens);
                let program = parser.parse_program().unwrap();
                
                let mut analyzer = SemanticAnalyzer::new();
                analyzer.analyze_program(&program).unwrap();
                black_box(())
            })
        });
    }
    
    // Benchmark complete pipeline with different sizes
    for size in [10, 50, 100].iter() {
        let source = generate_large_module(*size);
        group.bench_with_input(
            BenchmarkId::new("full_compilation_size", size), 
            &source, 
            |b, source| {
                b.iter(|| {
                    let mut lexer = Lexer::new(black_box(source), "benchmark.aether".to_string());
                    let tokens = lexer.tokenize().unwrap();
                    
                    let mut parser = Parser::new(tokens);
                    let program = parser.parse_program().unwrap();
                    
                    let mut analyzer = SemanticAnalyzer::new();
                    analyzer.analyze_program(&program).unwrap();
                    black_box(())
                })
            }
        );
    }
    
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark AST creation and manipulation
    group.bench_function("ast_creation", |b| {
        b.iter(|| {
            let source = generate_large_module(100);
            let mut lexer = Lexer::new(&source, "benchmark.aether".to_string());
            let tokens = lexer.tokenize().unwrap();
            let mut parser = Parser::new(tokens);
            black_box(parser.parse_program().unwrap())
        })
    });
    
    // Benchmark symbol table operations
    group.bench_function("symbol_table_operations", |b| {
        let source = generate_large_module(100);
        let mut lexer = Lexer::new(&source, "benchmark.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        
        b.iter(|| {
            let mut analyzer = SemanticAnalyzer::new();
            analyzer.analyze_program(&program).unwrap();
            black_box(())
        })
    });
    
    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    // Test lexer error handling
    group.bench_function("lexer_error_recovery", |b| {
        let invalid_source = "invalid @#$%^& characters everywhere @#$%^&";
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(invalid_source), "error.aether".to_string());
            black_box(lexer.tokenize().unwrap_err())
        })
    });
    
    // Test parser error handling
    group.bench_function("parser_error_recovery", |b| {
        let malformed_source = "(((((((((";
        let mut lexer = Lexer::new(malformed_source, "error.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        
        b.iter(|| {
            let mut parser = Parser::new(black_box(tokens.clone()));
            black_box(parser.parse_program().unwrap_err())
        })
    });
    
    // Test semantic analysis error handling
    group.bench_function("semantic_error_detection", |b| {
        let source = load_fixture("type_errors.aether");
        let mut lexer = Lexer::new(&source, "error.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        
        b.iter(|| {
            let mut analyzer = SemanticAnalyzer::new();
            black_box(analyzer.analyze_program(black_box(&program)).unwrap_err())
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_semantic_analysis,
    bench_complete_pipeline,
    bench_memory_usage,
    bench_error_handling
);

criterion_main!(benches);