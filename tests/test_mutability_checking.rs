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

//! Tests for mutability and borrowing enforcement

use aether::lexer::Lexer;
use aether::parser::Parser;
use aether::semantic::SemanticAnalyzer;
use aether::error::SemanticError;

#[test]
fn test_use_after_move() {
    let source = r#"
(module test_move
    (function test () integer
        (let ((x 42))
            ;; Transfer ownership
            (consume_value x)
            ;; This should fail - x has been moved
            (+ x 1))))
            
    (function consume_value (^integer value) void
        (print_int value)))
"#;
    
    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    // Should fail with use after move error
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| matches!(e, SemanticError::UseAfterMove { .. })));
}

#[test]
fn test_immutable_borrow_while_borrowed() {
    let source = r#"
(module test_borrow
    (function test () integer
        (let ((x 42))
            ;; Borrow x immutably
            (let ((y (borrow_value &x)))
                ;; Can borrow again immutably
                (let ((z (borrow_value &x)))
                    (+ y z)))))
            
    (function borrow_value (&integer value) integer
        (+ value 1)))
"#;
    
    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    // Should succeed - multiple immutable borrows are allowed
    assert!(result.is_ok());
}

#[test]
fn test_mutable_borrow_while_immutably_borrowed() {
    let source = r#"
(module test_mut_borrow
    (function test () integer
        (let ((x 42))
            ;; Borrow x immutably
            (let ((y (borrow_value &x)))
                ;; Try to borrow mutably - should fail
                (modify_value &mut x)
                y)))
            
    (function borrow_value (&integer value) integer
        (+ value 1))
        
    (function modify_value (&mut integer value) void
        (set value (* value 2))))
"#;
    
    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    // Should fail - cannot mutably borrow while immutably borrowed
    assert!(result.is_err());
}

#[test]
fn test_immutable_variable_mutation() {
    let source = r#"
(module test_immutable
    (function test () integer
        (let ((x 42))  ;; x is immutable by default
            ;; Try to mutate x - should fail
            (set x 100)
            x)))
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
    assert!(errors.iter().any(|e| matches!(e, SemanticError::AssignToImmutable { .. })));
}

#[test]
fn test_mutable_variable_mutation() {
    let source = r#"
(module test_mutable
    (function test () integer
        (let ((mut x 42))  ;; x is mutable
            ;; Can mutate x
            (set x 100)
            x)))
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