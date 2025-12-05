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

//! Tests for pointer type parsing in V2 syntax
//!
//! Tests that verify `Pointer<T>` types are parsed correctly.

use aether::ast::TypeSpecifier;
use aether::lexer::v2::Lexer;
use aether::parser::v2::Parser;

#[test]
fn test_pointer_type_parsing() {
    let source = r#"
module test_pointers {
    func test(ptr: Pointer<Int>) -> Int {
        return 0;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    let module = &program.modules[0];
    let function = &module.function_definitions[0];
    let param_type = &function.parameters[0].param_type;

    // Should be a Pointer type
    match param_type.as_ref() {
        TypeSpecifier::Pointer { target_type, .. } => {
            // Target type should be Int (primitive)
            match target_type.as_ref() {
                TypeSpecifier::Primitive { .. } => {}
                _ => panic!("Expected primitive target type"),
            }
        }
        _ => panic!("Expected Pointer type, got {:?}", param_type),
    }
}

#[test]
fn test_nested_pointer_type_parsing() {
    let source = r#"
module test_nested_pointers {
    func test(ptr: Pointer<Pointer<Int>>) -> Int {
        return 0;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    let module = &program.modules[0];
    let function = &module.function_definitions[0];
    let param_type = &function.parameters[0].param_type;

    // Should be a Pointer<Pointer<Int>> type
    match param_type.as_ref() {
        TypeSpecifier::Pointer { target_type, .. } => match target_type.as_ref() {
            TypeSpecifier::Pointer {
                target_type: inner, ..
            } => match inner.as_ref() {
                TypeSpecifier::Primitive { .. } => {}
                _ => panic!("Expected primitive inner target type"),
            },
            _ => panic!("Expected nested Pointer type"),
        },
        _ => panic!("Expected Pointer type"),
    }
}

#[test]
fn test_address_of_expression_parsing() {
    let source = r#"
module test_address {
    func test() -> Pointer<Int> {
        let x: Int = 42;
        let ptr: Pointer<Int> = &x;
        return ptr;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    // If we get here, parsing succeeded
    assert_eq!(program.modules.len(), 1);
    assert_eq!(program.modules[0].function_definitions.len(), 1);
}

#[test]
fn test_pointer_parameter() {
    let source = r#"
module test_ptr_param {
    func modify(ptr: Pointer<Int>) -> Void {
        // Function accepting pointer
    }

    func test() -> Int {
        var x: Int = 42;
        modify(&x);
        return x;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().unwrap();

    assert_eq!(program.modules[0].function_definitions.len(), 2);
}
