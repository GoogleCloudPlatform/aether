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

//! Tests for ownership annotation parsing

use aether::ast::{OwnershipKind, TypeSpecifier};
use aether::lexer::Lexer;
use aether::parser::Parser;

#[test]
fn test_owned_type_parsing() {
    let source = "(module test_ownership
        (function test (^integer) void))";

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let module = &program.modules[0];
    let function = &module.function_definitions[0];
    let param_type = &function.parameters[0].param_type;

    match param_type.as_ref() {
        TypeSpecifier::Owned { ownership, .. } => {
            assert_eq!(*ownership, OwnershipKind::Owned);
        }
        _ => panic!("Expected owned type"),
    }
}

#[test]
fn test_borrowed_type_parsing() {
    let source = "(module test_ownership
        (function test (&integer) void))";

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let module = &program.modules[0];
    let function = &module.function_definitions[0];
    let param_type = &function.parameters[0].param_type;

    match param_type.as_ref() {
        TypeSpecifier::Owned { ownership, .. } => {
            assert_eq!(*ownership, OwnershipKind::Borrowed);
        }
        _ => panic!("Expected borrowed type"),
    }
}

#[test]
fn test_mut_borrowed_type_parsing() {
    let source = "(module test_ownership
        (function test (&mut integer) void))";

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let module = &program.modules[0];
    let function = &module.function_definitions[0];
    let param_type = &function.parameters[0].param_type;

    match param_type.as_ref() {
        TypeSpecifier::Owned { ownership, .. } => {
            assert_eq!(*ownership, OwnershipKind::BorrowedMut);
        }
        _ => panic!("Expected mutable borrowed type"),
    }
}

#[test]
fn test_shared_type_parsing() {
    let source = "(module test_ownership
        (function test (~integer) void))";

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let module = &program.modules[0];
    let function = &module.function_definitions[0];
    let param_type = &function.parameters[0].param_type;

    match param_type.as_ref() {
        TypeSpecifier::Owned { ownership, .. } => {
            assert_eq!(*ownership, OwnershipKind::Shared);
        }
        _ => panic!("Expected shared type"),
    }
}
