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

use aether::ast::*;
use aether::error::{SemanticError, SourceLocation};
use aether::semantic::SemanticAnalyzer;

fn create_ownership_test_module(function_body: Vec<Statement>) -> Program {
    let loc = SourceLocation::unknown();

    // Define a consume function that takes ownership
    let consume_func = Function {
        name: Identifier::new("consume".to_string(), loc.clone()),
        intent: None,
        generic_parameters: Vec::new(),
        lifetime_parameters: Vec::new(),
        where_clause: Vec::new(),
        parameters: vec![Parameter {
            name: Identifier::new("s".to_string(), loc.clone()),
            param_type: Box::new(TypeSpecifier::Owned {
                ownership: OwnershipKind::Owned,
                base_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::String,
                    source_location: loc.clone(),
                }),
                lifetime: None,
                source_location: loc.clone(),
            }),
            intent: None,
            constraint: None,
            passing_mode: PassingMode::ByValue,
            source_location: loc.clone(),
        }],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: loc.clone(),
        }),
        metadata: FunctionMetadata {
            preconditions: vec![],
            postconditions: vec![],
            invariants: vec![],
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: vec![],
            thread_safe: None,
            may_block: None,
        },
        body: Block {
            statements: vec![],
            source_location: loc.clone(),
        },
        export_info: None,
        is_async: false,
        source_location: loc.clone(),
    };

    // Define a borrow function that takes a reference
    let borrow_func = Function {
        name: Identifier::new("borrow".to_string(), loc.clone()),
        intent: None,
        generic_parameters: Vec::new(),
        lifetime_parameters: Vec::new(),
        where_clause: Vec::new(),
        parameters: vec![Parameter {
            name: Identifier::new("s".to_string(), loc.clone()),
            param_type: Box::new(TypeSpecifier::Owned {
                ownership: OwnershipKind::Borrowed,
                base_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::String,
                    source_location: loc.clone(),
                }),
                lifetime: None,
                source_location: loc.clone(),
            }),
            intent: None,
            constraint: None,
            passing_mode: PassingMode::ByValue,
            source_location: loc.clone(),
        }],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: loc.clone(),
        }),
        metadata: FunctionMetadata {
            preconditions: vec![],
            postconditions: vec![],
            invariants: vec![],
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: vec![],
            thread_safe: None,
            may_block: None,
        },
        body: Block {
            statements: vec![],
            source_location: loc.clone(),
        },
        export_info: None,
        is_async: false,
        source_location: loc.clone(),
    };

    // Define the test function
    let test_func = Function {
        name: Identifier::new("test_main".to_string(), loc.clone()),
        intent: None,
        generic_parameters: Vec::new(),
        lifetime_parameters: Vec::new(),
        where_clause: Vec::new(),
        parameters: vec![],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: loc.clone(),
        }),
        metadata: FunctionMetadata {
            preconditions: vec![],
            postconditions: vec![],
            invariants: vec![],
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: vec![],
            thread_safe: None,
            may_block: None,
        },
        body: Block {
            statements: function_body,
            source_location: loc.clone(),
        },
        export_info: None,
        is_async: false,
        source_location: loc.clone(),
    };

    let module = Module {
        name: Identifier::new("ownership_test".to_string(), loc.clone()),
        intent: None,
        imports: vec![],
        exports: vec![],
        type_definitions: vec![],
        trait_definitions: vec![],
        impl_blocks: vec![],
        constant_declarations: vec![],
        function_definitions: vec![consume_func, borrow_func, test_func],
        external_functions: vec![],
        source_location: loc.clone(),
    };

    Program {
        modules: vec![module],
        source_location: loc,
    }
}

#[test]
fn test_valid_borrowing() {
    let loc = SourceLocation::unknown();
    let body = vec![
        // let s: ^String = "hello";
        Statement::VariableDeclaration {
            name: Identifier::new("s".to_string(), loc.clone()),
            type_spec: Box::new(TypeSpecifier::Owned {
                ownership: OwnershipKind::Owned,
                base_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::String,
                    source_location: loc.clone(),
                }),
                lifetime: None,
                source_location: loc.clone(),
            }),
            mutability: Mutability::Immutable,
            initial_value: Some(Box::new(Expression::StringLiteral {
                value: "hello".to_string(),
                source_location: loc.clone(),
            })),
            intent: None,
            source_location: loc.clone(),
        },
        // borrow(s);
        Statement::FunctionCall {
            call: FunctionCall {
                function_reference: FunctionReference::Local {
                    name: Identifier::new("borrow".to_string(), loc.clone()),
                },
                explicit_type_arguments: vec![],
                arguments: vec![Argument {
                    parameter_name: Identifier::new("s".to_string(), loc.clone()),
                    value: Box::new(Expression::Variable {
                        name: Identifier::new("s".to_string(), loc.clone()),
                        source_location: loc.clone(),
                    }),
                    source_location: loc.clone(),
                }],
                variadic_arguments: vec![],
            },
            source_location: loc.clone(),
        },
        // consume(s);
        Statement::FunctionCall {
            call: FunctionCall {
                function_reference: FunctionReference::Local {
                    name: Identifier::new("consume".to_string(), loc.clone()),
                },
                explicit_type_arguments: vec![],
                arguments: vec![Argument {
                    parameter_name: Identifier::new("s".to_string(), loc.clone()),
                    value: Box::new(Expression::Variable {
                        name: Identifier::new("s".to_string(), loc.clone()),
                        source_location: loc.clone(),
                    }),
                    source_location: loc.clone(),
                }],
                variadic_arguments: vec![],
            },
            source_location: loc.clone(),
        },
    ];

    let program = create_ownership_test_module(body);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    assert!(result.is_ok(), "Valid borrowing should pass analysis");
}

#[test]
fn test_use_after_move() {
    let loc = SourceLocation::unknown();
    let body = vec![
        // let s: ^String = "hello";
        Statement::VariableDeclaration {
            name: Identifier::new("s".to_string(), loc.clone()),
            type_spec: Box::new(TypeSpecifier::Owned {
                ownership: OwnershipKind::Owned,
                base_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::String,
                    source_location: loc.clone(),
                }),
                lifetime: None,
                source_location: loc.clone(),
            }),
            mutability: Mutability::Immutable,
            initial_value: Some(Box::new(Expression::StringLiteral {
                value: "hello".to_string(),
                source_location: loc.clone(),
            })),
            intent: None,
            source_location: loc.clone(),
        },
        // consume(s);
        Statement::FunctionCall {
            call: FunctionCall {
                function_reference: FunctionReference::Local {
                    name: Identifier::new("consume".to_string(), loc.clone()),
                },
                explicit_type_arguments: vec![],
                arguments: vec![Argument {
                    parameter_name: Identifier::new("s".to_string(), loc.clone()),
                    value: Box::new(Expression::Variable {
                        name: Identifier::new("s".to_string(), loc.clone()),
                        source_location: loc.clone(),
                    }),
                    source_location: loc.clone(),
                }],
                variadic_arguments: vec![],
            },
            source_location: loc.clone(),
        },
        // borrow(s); // Should fail here
        Statement::FunctionCall {
            call: FunctionCall {
                function_reference: FunctionReference::Local {
                    name: Identifier::new("borrow".to_string(), loc.clone()),
                },
                explicit_type_arguments: vec![],
                arguments: vec![Argument {
                    parameter_name: Identifier::new("s".to_string(), loc.clone()),
                    value: Box::new(Expression::Variable {
                        name: Identifier::new("s".to_string(), loc.clone()),
                        source_location: loc.clone(),
                    }),
                    source_location: loc.clone(),
                }],
                variadic_arguments: vec![],
            },
            source_location: loc.clone(),
        },
    ];

    let program = create_ownership_test_module(body);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(result.is_err(), "Use after move should fail analysis");
    let errors = result.unwrap_err();
    
    let use_after_move_error = errors.iter().find(|e| matches!(e, SemanticError::UseAfterMove { .. }));
    assert!(use_after_move_error.is_some(), "Expected UseAfterMove error");
}

#[test]
fn test_move_on_assignment() {
    let loc = SourceLocation::unknown();
    let body = vec![
        // let s1: ^String = "hello";
        Statement::VariableDeclaration {
            name: Identifier::new("s1".to_string(), loc.clone()),
            type_spec: Box::new(TypeSpecifier::Owned {
                ownership: OwnershipKind::Owned,
                base_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::String,
                    source_location: loc.clone(),
                }),
                lifetime: None,
                source_location: loc.clone(),
            }),
            mutability: Mutability::Mutable, // Mutable to allow move? Actually assignment usually works for declared vars
            initial_value: Some(Box::new(Expression::StringLiteral {
                value: "hello".to_string(),
                source_location: loc.clone(),
            })),
            intent: None,
            source_location: loc.clone(),
        },
        // let s2: ^String = s1; // Moves s1 to s2
        Statement::VariableDeclaration {
            name: Identifier::new("s2".to_string(), loc.clone()),
            type_spec: Box::new(TypeSpecifier::Owned {
                ownership: OwnershipKind::Owned,
                base_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::String,
                    source_location: loc.clone(),
                }),
                lifetime: None,
                source_location: loc.clone(),
            }),
            mutability: Mutability::Immutable,
            initial_value: Some(Box::new(Expression::Variable {
                name: Identifier::new("s1".to_string(), loc.clone()),
                source_location: loc.clone(),
            })),
            intent: None,
            source_location: loc.clone(),
        },
        // consume(s1); // Should fail, s1 moved
        Statement::FunctionCall {
            call: FunctionCall {
                function_reference: FunctionReference::Local {
                    name: Identifier::new("consume".to_string(), loc.clone()),
                },
                explicit_type_arguments: vec![],
                arguments: vec![Argument {
                    parameter_name: Identifier::new("s".to_string(), loc.clone()),
                    value: Box::new(Expression::Variable {
                        name: Identifier::new("s1".to_string(), loc.clone()),
                        source_location: loc.clone(),
                    }),
                    source_location: loc.clone(),
                }],
                variadic_arguments: vec![],
            },
            source_location: loc.clone(),
        },
    ];

    let program = create_ownership_test_module(body);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    assert!(result.is_err(), "Use after move assignment should fail analysis");
    
    // Note: Currently variable declaration initialization doesn't trigger move tracking in semantic analyzer
    // because it handles expressions generically. We might need to check if we implemented move tracking for Variable expressions.
    // Let's check `analyze_expression` in `src/semantic/mod.rs`.
    // It checks `symbol.is_moved`. But does it SET `is_moved`?
    // Usually assignment sets it. VariableDeclaration initialization is basically an assignment.
    // If we didn't implement it yet, this test will fail to catch the error (so the test itself fails assertion).
    // This is good - it validates our implementation status.
}