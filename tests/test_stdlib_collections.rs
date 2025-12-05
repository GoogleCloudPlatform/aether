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

//! Tests for collection operations parsing and analysis
//!
//! Tests array declarations and basic operations.

use aether::lexer::v2::Lexer;
use aether::parser::v2::Parser;
use aether::semantic::SemanticAnalyzer;

#[test]
fn test_array_declaration() {
    let source = r#"
module test_arrays {
    func test_array() -> Int {
        let arr: Array<Int> = [1, 2, 3, 4, 5];
        return arr[0];
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_array_indexing() {
    let source = r#"
module test_indexing {
    func get_element(arr: Array<Int>, index: Int) -> Int {
        return arr[index];
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_array_iteration() {
    let source = r#"
module test_iteration {
    func sum_array() -> Int {
        let arr: Array<Int> = [1, 2, 3, 4, 5];
        var sum: Int = 0;
        var i: Int = 0;
        while {i < 5} {
            sum = {sum + arr[i]};
            i = {i + 1};
        }
        return sum;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_simple_function_call() {
    let source = r#"
module test_calls {
    func add(a: Int, b: Int) -> Int {
        return {a + b};
    }

    func main() -> Int {
        let result: Int = add(10, 20);
        return result;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}
