#[cfg(test)]
mod tests {
    use crate::semantic::SemanticAnalyzer;
    use crate::ast::*;
    use crate::types::{Type, OwnershipKind};
    use crate::error::{SourceLocation, SemanticError};
    
    #[test]
    fn test_ownership_transfer() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Create a module with ownership-aware functions
        let module = Module {
            name: Identifier::new("test".to_string(), SourceLocation::unknown()),
            imports: vec![],
            type_definitions: vec![],
            constant_declarations: vec![],
            function_definitions: vec![
                // Function that takes ownership
                FunctionDefinition {
                    name: Identifier::new("consume".to_string(), SourceLocation::unknown()),
                    parameters: vec![
                        Parameter {
                            name: Identifier::new("value".to_string(), SourceLocation::unknown()),
                            parameter_type: TypeSpecifier::Owned {
                                ownership: crate::ast::OwnershipKind::Owned,
                                base_type: Box::new(TypeSpecifier::Primitive {
                                    primitive_type: PrimitiveType::String,
                                    source_location: SourceLocation::unknown(),
                                }),
                                source_location: SourceLocation::unknown(),
                            },
                            default_value: None,
                            is_mutable: false,
                            source_location: SourceLocation::unknown(),
                        }
                    ],
                    return_type: TypeSpecifier::Primitive {
                        primitive_type: PrimitiveType::Void,
                        source_location: SourceLocation::unknown(),
                    },
                    body: Block {
                        statements: vec![],
                        source_location: SourceLocation::unknown(),
                    },
                    visibility: Visibility::Public,
                    is_entry: false,
                    metadata: None,
                    source_location: SourceLocation::unknown(),
                },
                // Function that borrows
                FunctionDefinition {
                    name: Identifier::new("borrow".to_string(), SourceLocation::unknown()),
                    parameters: vec![
                        Parameter {
                            name: Identifier::new("value".to_string(), SourceLocation::unknown()),
                            parameter_type: TypeSpecifier::Owned {
                                ownership: crate::ast::OwnershipKind::Borrowed,
                                base_type: Box::new(TypeSpecifier::Primitive {
                                    primitive_type: PrimitiveType::String,
                                    source_location: SourceLocation::unknown(),
                                }),
                                source_location: SourceLocation::unknown(),
                            },
                            default_value: None,
                            is_mutable: false,
                            source_location: SourceLocation::unknown(),
                        }
                    ],
                    return_type: TypeSpecifier::Primitive {
                        primitive_type: PrimitiveType::Void,
                        source_location: SourceLocation::unknown(),
                    },
                    body: Block {
                        statements: vec![],
                        source_location: SourceLocation::unknown(),
                    },
                    visibility: Visibility::Public,
                    is_entry: false,
                    metadata: None,
                    source_location: SourceLocation::unknown(),
                },
            ],
            external_functions: vec![],
            global_variables: vec![],
            source_location: SourceLocation::unknown(),
        };
        
        // Analyze the module to register the functions
        let result = analyzer.analyze_module(&module);
        assert!(result.is_ok());
        
        // Now test ownership tracking with a function that uses these
        let test_function = FunctionDefinition {
            name: Identifier::new("test_ownership".to_string(), SourceLocation::unknown()),
            parameters: vec![],
            return_type: TypeSpecifier::Primitive {
                primitive_type: PrimitiveType::Void,
                source_location: SourceLocation::unknown(),
            },
            body: Block {
                statements: vec![
                    // Declare an owned variable
                    Statement::VariableDeclaration {
                        name: Identifier::new("s".to_string(), SourceLocation::unknown()),
                        var_type: TypeSpecifier::Owned {
                            ownership: crate::ast::OwnershipKind::Owned,
                            base_type: Box::new(TypeSpecifier::Primitive {
                                primitive_type: PrimitiveType::String,
                                source_location: SourceLocation::unknown(),
                            }),
                            source_location: SourceLocation::unknown(),
                        },
                        value: Some(Expression::StringLiteral {
                            value: "Hello".to_string(),
                            source_location: SourceLocation::unknown(),
                        }),
                        is_mutable: false,
                        source_location: SourceLocation::unknown(),
                    },
                    // Borrow it (should work)
                    Statement::FunctionCall {
                        call: FunctionCall {
                            function_reference: FunctionReference::Local {
                                name: Identifier::new("borrow".to_string(), SourceLocation::unknown()),
                            },
                            arguments: vec![
                                Argument {
                                    name: None,
                                    value: Box::new(Expression::Variable {
                                        name: Identifier::new("s".to_string(), SourceLocation::unknown()),
                                        source_location: SourceLocation::unknown(),
                                    }),
                                }
                            ],
                            variadic_arguments: vec![],
                            source_location: SourceLocation::unknown(),
                        },
                        source_location: SourceLocation::unknown(),
                    },
                    // Move it (should work)
                    Statement::FunctionCall {
                        call: FunctionCall {
                            function_reference: FunctionReference::Local {
                                name: Identifier::new("consume".to_string(), SourceLocation::unknown()),
                            },
                            arguments: vec![
                                Argument {
                                    name: None,
                                    value: Box::new(Expression::Variable {
                                        name: Identifier::new("s".to_string(), SourceLocation::unknown()),
                                        source_location: SourceLocation::unknown(),
                                    }),
                                }
                            ],
                            variadic_arguments: vec![],
                            source_location: SourceLocation::unknown(),
                        },
                        source_location: SourceLocation::unknown(),
                    },
                    // Try to use it again (should fail with use-after-move)
                    Statement::FunctionCall {
                        call: FunctionCall {
                            function_reference: FunctionReference::Local {
                                name: Identifier::new("borrow".to_string(), SourceLocation::unknown()),
                            },
                            arguments: vec![
                                Argument {
                                    name: None,
                                    value: Box::new(Expression::Variable {
                                        name: Identifier::new("s".to_string(), SourceLocation::unknown()),
                                        source_location: SourceLocation::unknown(),
                                    }),
                                }
                            ],
                            variadic_arguments: vec![],
                            source_location: SourceLocation::unknown(),
                        },
                        source_location: SourceLocation::unknown(),
                    },
                ],
                source_location: SourceLocation::unknown(),
            },
            visibility: Visibility::Public,
            is_entry: false,
            metadata: None,
            source_location: SourceLocation::unknown(),
        };
        
        // Analyze the test function - it should fail with use-after-move
        let result = analyzer.analyze_function(&test_function);
        assert!(result.is_err());
        
        // Check that the error is specifically UseAfterMove
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