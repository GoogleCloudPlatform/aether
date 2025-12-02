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

use super::*;
use crate::ast::{self, Identifier, PrimitiveType};
use crate::error::SourceLocation;
use crate::mir::lowering::LoweringContext;
use crate::symbols::SymbolTable;
use crate::types::Type;
use std::collections::HashMap;

fn create_test_program_with_generic_function() -> ast::Program {
    let generic_id_func_ast = ast::Function {
        name: Identifier::new("identity".to_string(), SourceLocation::unknown()),
        intent: None,
        generic_parameters: vec![ast::GenericParameter {
            name: Identifier::new("T".to_string(), SourceLocation::unknown()),
            constraints: vec![],
            default_type: None,
            source_location: SourceLocation::unknown(),
        }],
        lifetime_parameters: vec![],
        where_clause: vec![],
        parameters: vec![ast::Parameter {
            name: Identifier::new("val".to_string(), SourceLocation::unknown()),
            param_type: Box::new(ast::TypeSpecifier::TypeParameter {
                name: Identifier::new("T".to_string(), SourceLocation::unknown()),
                constraints: vec![],
                source_location: SourceLocation::unknown(),
            }),
            intent: None,
            constraint: None,
            passing_mode: ast::PassingMode::ByValue,
            source_location: SourceLocation::unknown(),
        }],
        return_type: Box::new(ast::TypeSpecifier::TypeParameter {
            name: Identifier::new("T".to_string(), SourceLocation::unknown()),
            constraints: vec![],
            source_location: SourceLocation::unknown(),
        }),
        metadata: ast::FunctionMetadata::default(),
        body: ast::Block {
            statements: vec![ast::Statement::Return {
                value: Some(Box::new(ast::Expression::Variable {
                    name: Identifier::new("val".to_string(), SourceLocation::unknown()),
                    source_location: SourceLocation::unknown(),
                })),
                source_location: SourceLocation::unknown(),
            }],
            source_location: SourceLocation::unknown(),
        },
        export_info: None,
        is_async: false,
        source_location: SourceLocation::unknown(),
    };

    let main_func_ast = ast::Function {
        name: Identifier::new("main".to_string(), SourceLocation::unknown()),
        intent: None,
        generic_parameters: vec![],
        lifetime_parameters: vec![],
        where_clause: vec![],
        parameters: vec![],
        return_type: Box::new(ast::TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: SourceLocation::unknown(),
        }),
        metadata: ast::FunctionMetadata::default(),
        body: ast::Block {
            statements: vec![
                // Call identity<Int>(5)
                ast::Statement::VariableDeclaration {
                    name: Identifier::new("x".to_string(), SourceLocation::unknown()),
                    type_spec: Box::new(ast::TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Integer,
                        source_location: SourceLocation::unknown(),
                    }),
                    mutability: ast::Mutability::Immutable,
                    initial_value: Some(Box::new(ast::Expression::FunctionCall {
                        call: ast::FunctionCall {
                            function_reference: ast::FunctionReference::Local {
                                name: Identifier::new(
                                    "identity".to_string(),
                                    SourceLocation::unknown(),
                                ),
                            },
                            explicit_type_arguments: vec![ast::TypeSpecifier::Primitive {
                                type_name: PrimitiveType::Integer,
                                source_location: SourceLocation::unknown(),
                            }],
                            arguments: vec![ast::Argument {
                                parameter_name: Identifier::new(
                                    "val".to_string(),
                                    SourceLocation::unknown(),
                                ),
                                value: Box::new(ast::Expression::IntegerLiteral {
                                    value: 5,
                                    source_location: SourceLocation::unknown(),
                                }),
                                source_location: SourceLocation::unknown(),
                            }],
                            variadic_arguments: vec![],
                        },
                        source_location: SourceLocation::unknown(),
                    })),
                    intent: None,
                    source_location: SourceLocation::unknown(),
                },
                // Call identity<Float>(3.14)
                ast::Statement::VariableDeclaration {
                    name: Identifier::new("y".to_string(), SourceLocation::unknown()),
                    type_spec: Box::new(ast::TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Float,
                        source_location: SourceLocation::unknown(),
                    }),
                    mutability: ast::Mutability::Immutable,
                    initial_value: Some(Box::new(ast::Expression::FunctionCall {
                        call: ast::FunctionCall {
                            function_reference: ast::FunctionReference::Local {
                                name: Identifier::new(
                                    "identity".to_string(),
                                    SourceLocation::unknown(),
                                ),
                            },
                            explicit_type_arguments: vec![ast::TypeSpecifier::Primitive {
                                type_name: PrimitiveType::Float,
                                source_location: SourceLocation::unknown(),
                            }],
                            arguments: vec![ast::Argument {
                                parameter_name: Identifier::new(
                                    "val".to_string(),
                                    SourceLocation::unknown(),
                                ),
                                value: Box::new(ast::Expression::FloatLiteral {
                                    value: 3.14,
                                    source_location: SourceLocation::unknown(),
                                }),
                                source_location: SourceLocation::unknown(),
                            }],
                            variadic_arguments: vec![],
                        },
                        source_location: SourceLocation::unknown(),
                    })),
                    intent: None,
                    source_location: SourceLocation::unknown(),
                },
                ast::Statement::Return {
                    value: Some(Box::new(ast::Expression::IntegerLiteral {
                        value: 0,
                        source_location: SourceLocation::unknown(),
                    })),
                    source_location: SourceLocation::unknown(),
                },
            ],
            source_location: SourceLocation::unknown(),
        },
        export_info: None,
        is_async: false,
        source_location: SourceLocation::unknown(),
    };

    ast::Program {
        modules: vec![ast::Module {
            name: Identifier::new("test_module".to_string(), SourceLocation::unknown()),
            intent: None,
            imports: vec![],
            exports: vec![],
            type_definitions: vec![],
            trait_definitions: vec![],
            impl_blocks: vec![],
            constant_declarations: vec![],
            function_definitions: vec![generic_id_func_ast, main_func_ast],
            external_functions: vec![],
            source_location: SourceLocation::unknown(),
        }],
        source_location: SourceLocation::unknown(),
    }
}

#[test]
fn test_monomorphization_simple() {
    let ast_program = create_test_program_with_generic_function();

    // Phase 2: Semantic analysis (needed to build symbol table and infer types)
    let mut semantic_analyzer = crate::semantic::SemanticAnalyzer::new();
    semantic_analyzer
        .analyze_program(&ast_program)
        .expect("Semantic analysis should pass");
    let symbol_table = semantic_analyzer.get_symbol_table().clone();
    let captures = semantic_analyzer.get_captures().clone();

    // Phase 3: MIR generation
    let mut lowering_context = LoweringContext::with_symbol_table(symbol_table);
    lowering_context.set_captures(captures);
    let mut mir_program = lowering_context
        .lower_program(&ast_program)
        .expect("MIR lowering should succeed");

    // Initial check: generic function should be present, instantiated ones not yet.
    assert!(mir_program.functions.contains_key("test_module.identity"));
    assert!(mir_program.functions.contains_key("main"));
    assert!(!mir_program
        .functions
        .contains_key("test_module.identity_Integer"));
    assert!(!mir_program
        .functions
        .contains_key("test_module.identity_Float"));

    // Phase 3.1: Monomorphization
    let mut monomorphizer = Monomorphizer::new();
    monomorphizer.run(&mut mir_program);

    // Verify monomorphization results
    // 1. Check for instantiated functions
    let mangled_int_name = "test_module.identity_Integer".to_string();
    let mangled_float_name = "test_module.identity_Float".to_string();

    assert!(mir_program.functions.contains_key(&mangled_int_name));
    assert!(mir_program.functions.contains_key(&mangled_float_name));

    // 2. Check the contents of the instantiated functions (simple check for now)
    let int_identity = &mir_program.functions[&mangled_int_name];
    assert_eq!(int_identity.name, mangled_int_name);
    assert_eq!(
        int_identity.return_type,
        Type::primitive(PrimitiveType::Integer)
    );
    assert_eq!(
        int_identity.parameters[0].ty,
        Type::primitive(PrimitiveType::Integer)
    );

    let float_identity = &mir_program.functions[&mangled_float_name];
    assert_eq!(float_identity.name, mangled_float_name);
    assert_eq!(
        float_identity.return_type,
        Type::primitive(PrimitiveType::Float)
    );
    assert_eq!(
        float_identity.parameters[0].ty,
        Type::primitive(PrimitiveType::Float)
    );

    // 3. Check that calls in 'main' are updated to mangled names
    let main_func = mir_program
        .functions
        .get("main")
        .expect("main function not found");
    let mut found_int_call = false;
    let mut found_float_call = false;

    for (_block_id, block) in &main_func.basic_blocks {
        for stmt in &block.statements {
            if let Statement::Assign {
                rvalue:
                    Rvalue::Call {
                        func:
                            Operand::Constant(Constant {
                                value: ConstantValue::String(called_name),
                                ..
                            }),
                        ..
                    },
                ..
            } = stmt
            {
                if *called_name == mangled_int_name {
                    found_int_call = true;
                }
                if *called_name == mangled_float_name {
                    found_float_call = true;
                }
            }
        }
    }
    assert!(
        found_int_call,
        "Call to identity<Int> not monomorphized correctly"
    );
    assert!(
        found_float_call,
        "Call to identity<Float> not monomorphized correctly"
    );

    // The original generic function should still exist but may be unused
    assert!(mir_program.functions.contains_key("test_module.identity"));
}
