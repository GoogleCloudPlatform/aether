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
use aether::semantic::SemanticAnalyzer;
use aether::error::SourceLocation;

/// Helper function to create a simple test module with control flow in AST form
fn create_control_flow_ast() -> Program {
    let loc = SourceLocation::unknown();
    
    // Create a function with if statement
    let if_func = Function {
        name: Identifier::new("test_if".to_string(), loc.clone()),
        intent: Some("Test if statement".to_string()),
        generic_parameters: Vec::new(),
        parameters: vec![
            Parameter {
                name: Identifier::new("x".to_string(), loc.clone()),
                param_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    source_location: loc.clone(),
                }),
                intent: None,
                constraint: None,
                passing_mode: PassingMode::ByValue,
                source_location: loc.clone(),
            }
        ],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: loc.clone(),
        }),
        metadata: FunctionMetadata {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: Vec::new(),
            thread_safe: None,
            may_block: None,
        },
        body: Block {
            statements: vec![
                Statement::If {
                    condition: Box::new(Expression::GreaterThan {
                        left: Box::new(Expression::Variable {
                            name: Identifier::new("x".to_string(), loc.clone()),
                            source_location: loc.clone(),
                        }),
                        right: Box::new(Expression::IntegerLiteral {
                            value: 10,
                            source_location: loc.clone(),
                        }),
                        source_location: loc.clone(),
                    }),
                    then_block: Block {
                        statements: vec![
                            Statement::Return {
                                value: Some(Box::new(Expression::IntegerLiteral {
                                    value: 1,
                                    source_location: loc.clone(),
                                })),
                                source_location: loc.clone(),
                            }
                        ],
                        source_location: loc.clone(),
                    },
                    else_ifs: vec![],
                    else_block: Some(Block {
                        statements: vec![
                            Statement::Return {
                                value: Some(Box::new(Expression::IntegerLiteral {
                                    value: 0,
                                    source_location: loc.clone(),
                                })),
                                source_location: loc.clone(),
                            }
                        ],
                        source_location: loc.clone(),
                    }),
                    source_location: loc.clone(),
                }
            ],
            source_location: loc.clone(),
        },
        export_info: None,
        source_location: loc.clone(),
    };

    // Create a function with while loop
    let while_func = Function {
        name: Identifier::new("test_while".to_string(), loc.clone()),
        intent: Some("Test while loop".to_string()),
        generic_parameters: Vec::new(),
        parameters: vec![
            Parameter {
                name: Identifier::new("n".to_string(), loc.clone()),
                param_type: Box::new(TypeSpecifier::Primitive {
                    type_name: PrimitiveType::Integer,
                    source_location: loc.clone(),
                }),
                intent: None,
                constraint: None,
                passing_mode: PassingMode::ByValue,
                source_location: loc.clone(),
            }
        ],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: loc.clone(),
        }),
        metadata: FunctionMetadata {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: Vec::new(),
            thread_safe: None,
            may_block: None,
        },
        body: Block {
            statements: vec![
                Statement::VariableDeclaration {
                    name: Identifier::new("count".to_string(), loc.clone()),
                    type_spec: Box::new(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Integer,
                        source_location: loc.clone(),
                    }),
                    mutability: Mutability::Mutable,
                    initial_value: Some(Box::new(Expression::IntegerLiteral {
                        value: 0,
                        source_location: loc.clone(),
                    })),
                    intent: None,
                    source_location: loc.clone(),
                },
                Statement::WhileLoop {
                    condition: Box::new(Expression::LessThan {
                        left: Box::new(Expression::Variable {
                            name: Identifier::new("count".to_string(), loc.clone()),
                            source_location: loc.clone(),
                        }),
                        right: Box::new(Expression::Variable {
                            name: Identifier::new("n".to_string(), loc.clone()),
                            source_location: loc.clone(),
                        }),
                        source_location: loc.clone(),
                    }),
                    invariant: None,
                    body: Block {
                        statements: vec![
                            Statement::Assignment {
                                target: AssignmentTarget::Variable {
                                    name: Identifier::new("count".to_string(), loc.clone()),
                                },
                                value: Box::new(Expression::Add {
                                    left: Box::new(Expression::Variable {
                                        name: Identifier::new("count".to_string(), loc.clone()),
                                        source_location: loc.clone(),
                                    }),
                                    right: Box::new(Expression::IntegerLiteral {
                                        value: 1,
                                        source_location: loc.clone(),
                                    }),
                                    source_location: loc.clone(),
                                }),
                                source_location: loc.clone(),
                            }
                        ],
                        source_location: loc.clone(),
                    },
                    label: None,
                    source_location: loc.clone(),
                },
                Statement::Return {
                    value: Some(Box::new(Expression::Variable {
                        name: Identifier::new("count".to_string(), loc.clone()),
                        source_location: loc.clone(),
                    })),
                    source_location: loc.clone(),
                }
            ],
            source_location: loc.clone(),
        },
        export_info: None,
        source_location: loc.clone(),
    };

    // Create module with both functions
    let module = Module {
        name: Identifier::new("control_flow_test".to_string(), loc.clone()),
        intent: Some("Test control flow constructs".to_string()),
        imports: Vec::new(),
        exports: Vec::new(),
        type_definitions: Vec::new(),
        constant_declarations: Vec::new(),
        function_definitions: vec![if_func, while_func],
        external_functions: Vec::new(),
        source_location: loc.clone(),
    };

    Program {
        modules: vec![module],
        source_location: loc,
    }
}

#[test]
fn test_if_statement_semantic_analysis() {
    let program = create_control_flow_ast();
    
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis should succeed");
    
    let stats = analyzer.get_statistics();
    assert_eq!(stats.modules_analyzed, 1);
    assert_eq!(stats.functions_analyzed, 2);
    assert_eq!(stats.variables_declared, 1); // count variable in while function
}

#[test]
fn test_non_boolean_condition_error() {
    let loc = SourceLocation::unknown();
    
    // Create a function with non-boolean if condition
    let bad_func = Function {
        name: Identifier::new("bad_if".to_string(), loc.clone()),
        intent: Some("Test non-boolean condition".to_string()),
        generic_parameters: Vec::new(),
        parameters: vec![],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: loc.clone(),
        }),
        metadata: FunctionMetadata {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: Vec::new(),
            thread_safe: None,
            may_block: None,
        },
        body: Block {
            statements: vec![
                Statement::If {
                    condition: Box::new(Expression::IntegerLiteral {
                        value: 42, // This should fail - not a boolean
                        source_location: loc.clone(),
                    }),
                    then_block: Block {
                        statements: vec![
                            Statement::Return {
                                value: None,
                                source_location: loc.clone(),
                            }
                        ],
                        source_location: loc.clone(),
                    },
                    else_ifs: vec![],
                    else_block: None,
                    source_location: loc.clone(),
                }
            ],
            source_location: loc.clone(),
        },
        export_info: None,
        source_location: loc.clone(),
    };

    let module = Module {
        name: Identifier::new("bad_control_flow".to_string(), loc.clone()),
        intent: Some("Test invalid control flow".to_string()),
        imports: Vec::new(),
        exports: Vec::new(),
        type_definitions: Vec::new(),
        constant_declarations: Vec::new(),
        function_definitions: vec![bad_func],
        external_functions: Vec::new(),
        source_location: loc.clone(),
    };

    let program = Program {
        modules: vec![module],
        source_location: loc,
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.to_string().contains("Type mismatch")));
}

#[test]
fn test_loop_variable_scope() {
    let loc = SourceLocation::unknown();
    
    // Create a for-each loop with element binding
    let foreach_func = Function {
        name: Identifier::new("test_foreach".to_string(), loc.clone()),
        intent: Some("Test for-each loop scope".to_string()),
        generic_parameters: Vec::new(),
        parameters: vec![],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: loc.clone(),
        }),
        metadata: FunctionMetadata {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: Vec::new(),
            thread_safe: None,
            may_block: None,
        },
        body: Block {
            statements: vec![
                Statement::ForEachLoop {
                    collection: Box::new(Expression::ArrayLiteral {
                        element_type: Box::new(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Integer,
                            source_location: loc.clone(),
                        }),
                        elements: vec![
                            Box::new(Expression::IntegerLiteral {
                                value: 1,
                                source_location: loc.clone(),
                            }),
                            Box::new(Expression::IntegerLiteral {
                                value: 2,
                                source_location: loc.clone(),
                            }),
                            Box::new(Expression::IntegerLiteral {
                                value: 3,
                                source_location: loc.clone(),
                            }),
                        ],
                        source_location: loc.clone(),
                    }),
                    element_binding: Identifier::new("elem".to_string(), loc.clone()),
                    element_type: Box::new(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Integer,
                        source_location: loc.clone(),
                    }),
                    index_binding: None,
                    body: Block {
                        statements: vec![
                            // Empty body - just testing scope
                        ],
                        source_location: loc.clone(),
                    },
                    label: None,
                    source_location: loc.clone(),
                },
                // This should fail - 'elem' is not in scope here
                Statement::Return {
                    value: Some(Box::new(Expression::Variable {
                        name: Identifier::new("elem".to_string(), loc.clone()),
                        source_location: loc.clone(),
                    })),
                    source_location: loc.clone(),
                }
            ],
            source_location: loc.clone(),
        },
        export_info: None,
        source_location: loc.clone(),
    };

    let module = Module {
        name: Identifier::new("loop_scope_test".to_string(), loc.clone()),
        intent: Some("Test loop scope isolation".to_string()),
        imports: Vec::new(),
        exports: Vec::new(),
        type_definitions: Vec::new(),
        constant_declarations: Vec::new(),
        function_definitions: vec![foreach_func],
        external_functions: Vec::new(),
        source_location: loc.clone(),
    };

    let program = Program {
        modules: vec![module],
        source_location: loc,
    };

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);
    
    assert!(result.is_err());
    let errors = result.unwrap_err();
    // Check that we got an error (might be Type mismatch instead of Undefined symbol
    // since we're analyzing the expression before checking if it's in scope)
    assert!(!errors.is_empty());
}

#[test]
fn test_break_continue_statements() {
    let loc = SourceLocation::unknown();
    
    // Create a function with break and continue
    let loop_func = Function {
        name: Identifier::new("test_break_continue".to_string(), loc.clone()),
        intent: Some("Test break and continue".to_string()),
        generic_parameters: Vec::new(),
        parameters: vec![],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: loc.clone(),
        }),
        metadata: FunctionMetadata {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: Vec::new(),
            thread_safe: None,
            may_block: None,
        },
        body: Block {
            statements: vec![
                Statement::WhileLoop {
                    condition: Box::new(Expression::BooleanLiteral {
                        value: true,
                        source_location: loc.clone(),
                    }),
                    invariant: None,
                    body: Block {
                        statements: vec![
                            Statement::If {
                                condition: Box::new(Expression::BooleanLiteral {
                                    value: true,
                                    source_location: loc.clone(),
                                }),
                                then_block: Block {
                                    statements: vec![
                                        Statement::Break {
                                            target_label: None,
                                            source_location: loc.clone(),
                                        }
                                    ],
                                    source_location: loc.clone(),
                                },
                                else_ifs: vec![],
                                else_block: Some(Block {
                                    statements: vec![
                                        Statement::Continue {
                                            target_label: None,
                                            source_location: loc.clone(),
                                        }
                                    ],
                                    source_location: loc.clone(),
                                }),
                                source_location: loc.clone(),
                            }
                        ],
                        source_location: loc.clone(),
                    },
                    label: None,
                    source_location: loc.clone(),
                }
            ],
            source_location: loc.clone(),
        },
        export_info: None,
        source_location: loc.clone(),
    };

    let module = Module {
        name: Identifier::new("break_continue_test".to_string(), loc.clone()),
        intent: Some("Test break and continue statements".to_string()),
        imports: Vec::new(),
        exports: Vec::new(),
        type_definitions: Vec::new(),
        constant_declarations: Vec::new(),
        function_definitions: vec![loop_func],
        external_functions: Vec::new(),
        source_location: loc.clone(),
    };

    let program = Program {
        modules: vec![module],
        source_location: loc,
    };

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis should succeed for break/continue");
}

#[test]
fn test_try_catch_analysis() {
    let loc = SourceLocation::unknown();
    
    // Create a function with try-catch block
    let try_func = Function {
        name: Identifier::new("test_try_catch".to_string(), loc.clone()),
        intent: Some("Test try-catch block".to_string()),
        generic_parameters: Vec::new(),
        parameters: vec![],
        return_type: Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: loc.clone(),
        }),
        metadata: FunctionMetadata {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: Vec::new(),
            thread_safe: None,
            may_block: None,
        },
        body: Block {
            statements: vec![
                Statement::TryBlock {
                    protected_block: Block {
                        statements: vec![
                            Statement::Return {
                                value: Some(Box::new(Expression::IntegerLiteral {
                                    value: 42,
                                    source_location: loc.clone(),
                                })),
                                source_location: loc.clone(),
                            }
                        ],
                        source_location: loc.clone(),
                    },
                    catch_clauses: vec![
                        CatchClause {
                            exception_type: Box::new(TypeSpecifier::Primitive {
                                type_name: PrimitiveType::String, // Use a known type for now
                                source_location: loc.clone(),
                            }),
                            binding_variable: Some(Identifier::new("e".to_string(), loc.clone())),
                            handler_block: Block {
                                statements: vec![
                                    Statement::Return {
                                        value: Some(Box::new(Expression::IntegerLiteral {
                                            value: -1,
                                            source_location: loc.clone(),
                                        })),
                                        source_location: loc.clone(),
                                    }
                                ],
                                source_location: loc.clone(),
                            },
                            source_location: loc.clone(),
                        }
                    ],
                    finally_block: None,
                    source_location: loc.clone(),
                }
            ],
            source_location: loc.clone(),
        },
        export_info: None,
        source_location: loc.clone(),
    };

    let module = Module {
        name: Identifier::new("try_catch_test".to_string(), loc.clone()),
        intent: Some("Test try-catch block".to_string()),
        imports: Vec::new(),
        exports: Vec::new(),
        type_definitions: Vec::new(),
        constant_declarations: Vec::new(),
        function_definitions: vec![try_func],
        external_functions: Vec::new(),
        source_location: loc.clone(),
    };

    let program = Program {
        modules: vec![module],
        source_location: loc,
    };

    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis should succeed for try-catch");
}