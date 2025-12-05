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

//! Tests for pointer type code generation
//!
//! Tests that verify pointer types can be parsed and analyzed correctly.

use aether::lexer::v2::Lexer;
use aether::parser::v2::Parser;
use aether::semantic::SemanticAnalyzer;

#[test]
fn test_pointer_address_of() {
    let source = r#"
module test_pointers {
    func test_address_of() -> Int {
        let x: Int = 42;
        let ptr: Pointer<Int> = &x;
        return x;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    // Should compile successfully
    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_pointer_parameter() {
    let source = r#"
module test_pointers {
    func increment(ptr: Pointer<Int>) -> Int {
        return 0;
    }

    func test() -> Int {
        var x: Int = 42;
        return increment(&x);
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    // Should compile successfully
    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_pointer_return_type() {
    let source = r#"
module test_pointers {
    func get_pointer(x: Pointer<Int>) -> Pointer<Int> {
        return x;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    // Should compile successfully
    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}
