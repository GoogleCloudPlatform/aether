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

//! Tests for mutability enforcement
//!
//! Tests that verify `let` bindings are immutable and `var` bindings are mutable.

use aether::error::SemanticError;
use aether::lexer::v2::Lexer;
use aether::parser::v2::Parser;
use aether::semantic::SemanticAnalyzer;

#[test]
fn test_immutable_variable_mutation() {
    let source = r#"
module test_immutable {
    func test() -> Int {
        let x: Int = 42;
        // Try to mutate x - should fail since let is immutable
        x = 100;
        return x;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    // Should fail - cannot assign to immutable variable
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, SemanticError::AssignToImmutable { .. })));
}

#[test]
fn test_mutable_variable_mutation() {
    let source = r#"
module test_mutable {
    func test() -> Int {
        var x: Int = 42;
        // Can mutate x since var is mutable
        x = 100;
        return x;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    // Should succeed - can assign to mutable variable
    assert!(result.is_ok());
}

#[test]
fn test_multiple_mutations() {
    let source = r#"
module test_mutations {
    func test() -> Int {
        var x: Int = 0;
        x = 10;
        x = 20;
        x = 30;
        return x;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    // Should succeed - multiple assignments to mutable variable
    assert!(result.is_ok());
}

#[test]
fn test_let_and_var_in_same_function() {
    let source = r#"
module test_mixed {
    func test() -> Int {
        let immutable_val: Int = 10;
        var mutable_val: Int = 20;
        mutable_val = {mutable_val + immutable_val};
        return mutable_val;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    // Should succeed - mixing let and var properly
    assert!(result.is_ok());
}
