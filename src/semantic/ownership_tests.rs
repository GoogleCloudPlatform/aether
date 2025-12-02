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

#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::error::{SemanticError, SourceLocation};
    use crate::semantic::SemanticAnalyzer;

    #[test]
    fn test_ownership_transfer() {
        let mut analyzer = SemanticAnalyzer::new();

        // Create a module with ownership-aware functions
        let module = Module {
            name: Identifier::new("test".to_string(), SourceLocation::unknown()),
            intent: None,
            imports: vec![],
            exports: vec![],
            type_definitions: vec![],
            trait_definitions: vec![],
            impl_blocks: vec![],
            constant_declarations: vec![],
            function_definitions: vec![
                // Function that takes ownership: fn consume(value: ^String)
                Function {
                    name: Identifier::new("consume".to_string(), SourceLocation::unknown()),
                    intent: None,
                    generic_parameters: vec![],
                    lifetime_parameters: vec![],
                    parameters: vec![Parameter {
                        name: Identifier::new("value".to_string(), SourceLocation::unknown()),
                        param_type: Box::new(TypeSpecifier::Owned {
                            ownership: OwnershipKind::Owned,
                            base_type: Box::new(TypeSpecifier::Primitive {
                                type_name: PrimitiveType::String,
                                source_location: SourceLocation::unknown(),
                            }),
                            lifetime: None,
                            source_location: SourceLocation::unknown(),
                        }),
                        intent: None,
                        constraint: None,
                        passing_mode: PassingMode::ByValue,
                        source_location: SourceLocation::unknown(),
                    }],
                    return_type: Box::new(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Void,
                        source_location: SourceLocation::unknown(),
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
                        source_location: SourceLocation::unknown(),
                    },
                    export_info: None,
                    is_async: false,
                    where_clause: vec![],
                    source_location: SourceLocation::unknown(),
                },
                // Function that borrows: fn borrow(value: &String)
                Function {
                    name: Identifier::new("borrow".to_string(), SourceLocation::unknown()),
                    intent: None,
                    generic_parameters: vec![],
                    lifetime_parameters: vec![],
                    parameters: vec![Parameter {
                        name: Identifier::new("value".to_string(), SourceLocation::unknown()),
                        param_type: Box::new(TypeSpecifier::Owned {
                            ownership: OwnershipKind::Borrowed,
                            base_type: Box::new(TypeSpecifier::Primitive {
                                type_name: PrimitiveType::String,
                                source_location: SourceLocation::unknown(),
                            }),
                            lifetime: None,
                            source_location: SourceLocation::unknown(),
                        }),
                        intent: None,
                        constraint: None,
                        passing_mode: PassingMode::ByValue,
                        source_location: SourceLocation::unknown(),
                    }],
                    return_type: Box::new(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Void,
                        source_location: SourceLocation::unknown(),
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
                        source_location: SourceLocation::unknown(),
                    },
                    export_info: None,
                    is_async: false,
                    where_clause: vec![],
                    source_location: SourceLocation::unknown(),
                },
            ],
            external_functions: vec![],
            source_location: SourceLocation::unknown(),
        };

        // Analyze the module to register the functions
        let result = analyzer.analyze_module(&module);
        assert!(result.is_ok(), "Module analysis failed: {:?}", result.err());

        // Now test ownership tracking with a function that uses these
        let test_function = Function {
            name: Identifier::new("test_ownership".to_string(), SourceLocation::unknown()),
            intent: None,
            generic_parameters: vec![],
            lifetime_parameters: vec![],
            parameters: vec![],
            return_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
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
                statements: vec![
                    // Declare an owned variable: let s: ^String = "Hello";
                    Statement::VariableDeclaration {
                        name: Identifier::new("s".to_string(), SourceLocation::unknown()),
                        type_spec: Box::new(TypeSpecifier::Owned {
                            ownership: OwnershipKind::Owned,
                            base_type: Box::new(TypeSpecifier::Primitive {
                                type_name: PrimitiveType::String,
                                source_location: SourceLocation::unknown(),
                            }),
                            lifetime: None,
                            source_location: SourceLocation::unknown(),
                        }),
                        mutability: Mutability::Immutable,
                        initial_value: Some(Box::new(Expression::StringLiteral {
                            value: "Hello".to_string(),
                            source_location: SourceLocation::unknown(),
                        })),
                        intent: None,
                        source_location: SourceLocation::unknown(),
                    },
                    // Borrow it (should work): borrow(s);
                    Statement::Expression {
                        expr: Box::new(Expression::FunctionCall {
                            call: FunctionCall {
                                function_reference: FunctionReference::Local {
                                    name: Identifier::new(
                                        "borrow".to_string(),
                                        SourceLocation::unknown(),
                                    ),
                                },
                                explicit_type_arguments: vec![],
                                arguments: vec![Argument {
                                    parameter_name: Identifier::new(
                                        "value".to_string(),
                                        SourceLocation::unknown(),
                                    ),
                                    value: Box::new(Expression::Variable {
                                        name: Identifier::new(
                                            "s".to_string(),
                                            SourceLocation::unknown(),
                                        ),
                                        source_location: SourceLocation::unknown(),
                                    }),
                                    source_location: SourceLocation::unknown(),
                                }],
                                variadic_arguments: vec![],
                            },
                            source_location: SourceLocation::unknown(),
                        }),
                        source_location: SourceLocation::unknown(),
                    },
                    // Move it (should work): consume(s);
                    Statement::Expression {
                        expr: Box::new(Expression::FunctionCall {
                            call: FunctionCall {
                                function_reference: FunctionReference::Local {
                                    name: Identifier::new(
                                        "consume".to_string(),
                                        SourceLocation::unknown(),
                                    ),
                                },
                                explicit_type_arguments: vec![],
                                arguments: vec![Argument {
                                    parameter_name: Identifier::new(
                                        "value".to_string(),
                                        SourceLocation::unknown(),
                                    ),
                                    value: Box::new(Expression::Variable {
                                        name: Identifier::new(
                                            "s".to_string(),
                                            SourceLocation::unknown(),
                                        ),
                                        source_location: SourceLocation::unknown(),
                                    }),
                                    source_location: SourceLocation::unknown(),
                                }],
                                variadic_arguments: vec![],
                            },
                            source_location: SourceLocation::unknown(),
                        }),
                        source_location: SourceLocation::unknown(),
                    },
                    // Try to use it again (should fail with use-after-move): borrow(s);
                    Statement::Expression {
                        expr: Box::new(Expression::FunctionCall {
                            call: FunctionCall {
                                function_reference: FunctionReference::Local {
                                    name: Identifier::new(
                                        "borrow".to_string(),
                                        SourceLocation::unknown(),
                                    ),
                                },
                                explicit_type_arguments: vec![],
                                arguments: vec![Argument {
                                    parameter_name: Identifier::new(
                                        "value".to_string(),
                                        SourceLocation::unknown(),
                                    ),
                                    value: Box::new(Expression::Variable {
                                        name: Identifier::new(
                                            "s".to_string(),
                                            SourceLocation::unknown(),
                                        ),
                                        source_location: SourceLocation::unknown(),
                                    }),
                                    source_location: SourceLocation::unknown(),
                                }],
                                variadic_arguments: vec![],
                            },
                            source_location: SourceLocation::unknown(),
                        }),
                        source_location: SourceLocation::unknown(),
                    },
                ],
                source_location: SourceLocation::unknown(),
            },
            export_info: None,
            is_async: false,
            where_clause: vec![],
            source_location: SourceLocation::unknown(),
        };

        // Analyze the test function - it should fail with use-after-move
        // Note: analyze_function_body is private, so we wrap it in a module to test via analyze_module
        // or expose it for tests. Since we can't easily change visibility here without modifying main code again,
        // let's put the test function in the module and expect analyze_module to fail.

        let mut test_module = module.clone();
        test_module.function_definitions.push(test_function);

        let mut analyzer2 = SemanticAnalyzer::new();
        let result = analyzer2.analyze_module(&test_module);

        assert!(result.is_err(), "Expected error for use-after-move");

        // Check that the error is specifically UseAfterMove
        // Since analyze_module returns a single error (the first one), this should work.
        if let Err(e) = result {
            match e {
                SemanticError::UseAfterMove { variable, .. } => {
                    assert_eq!(variable, "s");
                }
                _ => panic!("Expected UseAfterMove error, got {:?}", e),
            }
        }
    }
}
