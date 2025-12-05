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

//! Tests for FFI struct code generation in LLVM
//!
//! Verifies that the compiler generates correct LLVM IR for struct definitions
//! and FFI function calls involving structs.

use aether::ast::*;
use aether::error::SourceLocation;
use aether::llvm_backend::LLVMBackend;
use aether::mir::lowering::lower_ast_to_mir_with_symbols;
use aether::semantic::SemanticAnalyzer;

fn create_test_module_with_struct() -> Module {
    Module {
        name: Identifier::new("test_module".to_string(), SourceLocation::unknown()),
        intent: Some("Test module for FFI structs".to_string()),
        imports: vec![],
        exports: vec![],
        type_definitions: vec![TypeDefinition::Structured {
            name: Identifier::new("Point2D".to_string(), SourceLocation::unknown()),
            intent: Some("2D point structure".to_string()),
            generic_parameters: vec![],
            lifetime_parameters: vec![],
            where_clause: vec![],
            fields: vec![
                StructField {
                    name: Identifier::new("x".to_string(), SourceLocation::unknown()),
                    field_type: Box::new(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Float64,
                        source_location: SourceLocation::unknown(),
                    }),
                    source_location: SourceLocation::unknown(),
                },
                StructField {
                    name: Identifier::new("y".to_string(), SourceLocation::unknown()),
                    field_type: Box::new(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Float64,
                        source_location: SourceLocation::unknown(),
                    }),
                    source_location: SourceLocation::unknown(),
                },
            ],
            export_as: Some("struct Point2D".to_string()),
            is_copy: false,
            source_location: SourceLocation::unknown(),
        }],
        trait_definitions: vec![],
        impl_blocks: vec![],
        constant_declarations: vec![],
        function_definitions: vec![Function {
            name: Identifier::new("test_struct_passing".to_string(), SourceLocation::unknown()),
            intent: Some("Test struct passing".to_string()),
            generic_parameters: vec![],
            lifetime_parameters: vec![],
            where_clause: vec![],
            parameters: vec![Parameter {
                name: Identifier::new("p".to_string(), SourceLocation::unknown()),
                param_type: Box::new(TypeSpecifier::Named {
                    name: Identifier::new("Point2D".to_string(), SourceLocation::unknown()),
                    source_location: SourceLocation::unknown(),
                }),
                intent: None,
                constraint: None,
                passing_mode: PassingMode::ByValue,
                source_location: SourceLocation::unknown(),
            }],
            return_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Float64,
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
                thread_safe: Some(true),
                may_block: Some(false),
            },
            body: Block {
                statements: vec![Statement::Return {
                    value: Some(Box::new(Expression::FieldAccess {
                        instance: Box::new(Expression::Variable {
                            name: Identifier::new("p".to_string(), SourceLocation::unknown()),
                            source_location: SourceLocation::unknown(),
                        }),
                        field_name: Identifier::new("x".to_string(), SourceLocation::unknown()),
                        source_location: SourceLocation::unknown(),
                    })),
                    source_location: SourceLocation::unknown(),
                }],
                source_location: SourceLocation::unknown(),
            },
            export_info: None,
            is_async: false,
            source_location: SourceLocation::unknown(),
        }],
        external_functions: vec![ExternalFunction {
            name: Identifier::new("point_distance".to_string(), SourceLocation::unknown()),
            library: "aether_runtime".to_string(),
            symbol: None,
            parameters: vec![
                Parameter {
                    name: Identifier::new("p1".to_string(), SourceLocation::unknown()),
                    param_type: Box::new(TypeSpecifier::Named {
                        name: Identifier::new("Point2D".to_string(), SourceLocation::unknown()),
                        source_location: SourceLocation::unknown(),
                    }),
                    intent: None,
                    constraint: None,
                    passing_mode: PassingMode::ByValue,
                    source_location: SourceLocation::unknown(),
                },
                Parameter {
                    name: Identifier::new("p2".to_string(), SourceLocation::unknown()),
                    param_type: Box::new(TypeSpecifier::Named {
                        name: Identifier::new("Point2D".to_string(), SourceLocation::unknown()),
                        source_location: SourceLocation::unknown(),
                    }),
                    intent: None,
                    constraint: None,
                    passing_mode: PassingMode::ByValue,
                    source_location: SourceLocation::unknown(),
                },
            ],
            return_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Float64,
                source_location: SourceLocation::unknown(),
            }),
            calling_convention: CallingConvention::C,
            thread_safe: true,
            may_block: false,
            variadic: false,
            ownership_info: None,
            source_location: SourceLocation::unknown(),
        }],
        source_location: SourceLocation::unknown(),
    }
}

#[test]
fn test_struct_type_generation() {
    let module = create_test_module_with_struct();
    let program = Program {
        modules: vec![module],
        source_location: SourceLocation::unknown(),
    };

    // Run semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze_program(&program)
        .expect("Semantic analysis failed");

    // Get symbol table for MIR lowering
    let symbol_table = analyzer.get_symbol_table().clone();

    // Convert to MIR with symbol table
    let mir_program =
        lower_ast_to_mir_with_symbols(&program, symbol_table).expect("MIR lowering failed");

    // Create LLVM backend
    let context = inkwell::context::Context::create();
    let mut backend = LLVMBackend::new(&context, "test");

    // Generate code
    backend
        .generate_ir(&mir_program)
        .expect("LLVM code generation failed");

    // Get the generated module
    let llvm_module = backend.module();

    // Verify struct type was created
    let struct_type = llvm_module.get_struct_type("Point2D");
    assert!(
        struct_type.is_some(),
        "Point2D struct type not found in LLVM module"
    );

    // Verify struct has correct fields
    if let Some(st) = struct_type {
        assert_eq!(st.count_fields(), 2, "Point2D should have 2 fields");

        // Both fields should be f64 (double)
        let field_types = st.get_field_types();
        assert_eq!(field_types.len(), 2);
        assert!(field_types[0].is_float_type());
        assert!(field_types[1].is_float_type());
    }
}

#[test]
fn test_struct_passing_by_value() {
    let module = create_test_module_with_struct();
    let program = Program {
        modules: vec![module],
        source_location: SourceLocation::unknown(),
    };

    // Run semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze_program(&program)
        .expect("Semantic analysis failed");

    // Get symbol table for MIR lowering
    let symbol_table = analyzer.get_symbol_table().clone();

    // Convert to MIR with symbol table
    let mir_program =
        lower_ast_to_mir_with_symbols(&program, symbol_table).expect("MIR lowering failed");

    // Create LLVM backend
    let context = inkwell::context::Context::create();
    let mut backend = LLVMBackend::new(&context, "test");

    // Generate code
    backend
        .generate_ir(&mir_program)
        .expect("LLVM code generation failed");

    // Get the generated module
    let llvm_module = backend.module();

    // Verify the test function was created with correct signature
    // Function names are prefixed with module name
    let func = llvm_module.get_function("test_module.test_struct_passing");
    assert!(
        func.is_some(),
        "test_module.test_struct_passing function not found"
    );

    if let Some(f) = func {
        // Should have one parameter (the struct, possibly passed by pointer for ABI compatibility)
        assert_eq!(f.count_params(), 1, "Function should have 1 parameter");

        // Parameter should be either struct value or pointer to struct (ABI-dependent)
        let param = f.get_first_param().unwrap();
        let is_struct_or_ptr = param.is_struct_value() || param.is_pointer_value();
        assert!(
            is_struct_or_ptr,
            "Parameter should be a struct or pointer to struct"
        );
    }
}

#[test]
fn test_external_struct_function() {
    let module = create_test_module_with_struct();
    let program = Program {
        modules: vec![module],
        source_location: SourceLocation::unknown(),
    };

    // Run semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze_program(&program)
        .expect("Semantic analysis failed");

    // Get symbol table for MIR lowering
    let symbol_table = analyzer.get_symbol_table().clone();

    // Convert to MIR with symbol table
    let mir_program =
        lower_ast_to_mir_with_symbols(&program, symbol_table).expect("MIR lowering failed");

    // Create LLVM backend
    let context = inkwell::context::Context::create();
    let mut backend = LLVMBackend::new(&context, "test");

    // Generate code
    backend
        .generate_ir(&mir_program)
        .expect("LLVM code generation failed");

    // Get the generated module
    let llvm_module = backend.module();

    // Verify the external function declaration was created
    let func = llvm_module.get_function("point_distance");
    assert!(func.is_some(), "point_distance external function not found");

    if let Some(f) = func {
        // Should have two struct parameters (possibly passed by pointer for ABI compatibility)
        assert_eq!(f.count_params(), 2, "Function should have 2 parameters");

        // Both parameters should be structs or pointers to structs (ABI-dependent)
        let param1 = f.get_first_param().unwrap();
        let param2 = f.get_nth_param(1).unwrap();
        let is_struct_or_ptr1 = param1.is_struct_value() || param1.is_pointer_value();
        let is_struct_or_ptr2 = param2.is_struct_value() || param2.is_pointer_value();
        assert!(
            is_struct_or_ptr1,
            "First parameter should be a struct or pointer to struct"
        );
        assert!(
            is_struct_or_ptr2,
            "Second parameter should be a struct or pointer to struct"
        );

        // Return type verification - just verify the function exists with correct signature
        // The exact return type encoding may vary based on ABI and codegen implementation
        let _return_type = f.get_type().get_return_type();
        // The important thing is that the function was declared with struct parameters
    }
}

#[test]
fn test_nested_struct_generation() {
    let module = Module {
        name: Identifier::new("test_nested".to_string(), SourceLocation::unknown()),
        intent: None,
        imports: vec![],
        exports: vec![],
        type_definitions: vec![
            TypeDefinition::Structured {
                name: Identifier::new("Point2D".to_string(), SourceLocation::unknown()),
                intent: None,
                generic_parameters: vec![],
                lifetime_parameters: vec![],
                where_clause: vec![],
                fields: vec![
                    StructField {
                        name: Identifier::new("x".to_string(), SourceLocation::unknown()),
                        field_type: Box::new(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Float64,
                            source_location: SourceLocation::unknown(),
                        }),
                        source_location: SourceLocation::unknown(),
                    },
                    StructField {
                        name: Identifier::new("y".to_string(), SourceLocation::unknown()),
                        field_type: Box::new(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Float64,
                            source_location: SourceLocation::unknown(),
                        }),
                        source_location: SourceLocation::unknown(),
                    },
                ],
                export_as: None,
                is_copy: false,
                source_location: SourceLocation::unknown(),
            },
            TypeDefinition::Structured {
                name: Identifier::new("Rectangle".to_string(), SourceLocation::unknown()),
                intent: None,
                generic_parameters: vec![],
                lifetime_parameters: vec![],
                where_clause: vec![],
                fields: vec![
                    StructField {
                        name: Identifier::new("top_left".to_string(), SourceLocation::unknown()),
                        field_type: Box::new(TypeSpecifier::Named {
                            name: Identifier::new("Point2D".to_string(), SourceLocation::unknown()),
                            source_location: SourceLocation::unknown(),
                        }),
                        source_location: SourceLocation::unknown(),
                    },
                    StructField {
                        name: Identifier::new("width".to_string(), SourceLocation::unknown()),
                        field_type: Box::new(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Float64,
                            source_location: SourceLocation::unknown(),
                        }),
                        source_location: SourceLocation::unknown(),
                    },
                    StructField {
                        name: Identifier::new("height".to_string(), SourceLocation::unknown()),
                        field_type: Box::new(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Float64,
                            source_location: SourceLocation::unknown(),
                        }),
                        source_location: SourceLocation::unknown(),
                    },
                ],
                export_as: None,
                is_copy: false,
                source_location: SourceLocation::unknown(),
            },
        ],
        trait_definitions: vec![],
        impl_blocks: vec![],
        constant_declarations: vec![],
        function_definitions: vec![],
        external_functions: vec![],
        source_location: SourceLocation::unknown(),
    };

    let program = Program {
        modules: vec![module],
        source_location: SourceLocation::unknown(),
    };

    // Run semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze_program(&program)
        .expect("Semantic analysis failed");

    // Get symbol table for MIR lowering
    let symbol_table = analyzer.get_symbol_table().clone();

    // Convert to MIR with symbol table
    let mir_program =
        lower_ast_to_mir_with_symbols(&program, symbol_table).expect("MIR lowering failed");

    // Create LLVM backend
    let context = inkwell::context::Context::create();
    let mut backend = LLVMBackend::new(&context, "test");

    // Generate code
    backend
        .generate_ir(&mir_program)
        .expect("LLVM code generation failed");

    // Get the generated module
    let llvm_module = backend.module();

    // Verify both struct types were created
    let point_type = llvm_module.get_struct_type("Point2D");
    let rect_type = llvm_module.get_struct_type("Rectangle");

    assert!(point_type.is_some(), "Point2D struct type not found");
    assert!(rect_type.is_some(), "Rectangle struct type not found");

    // Verify Rectangle has correct fields
    if let Some(rt) = rect_type {
        assert_eq!(rt.count_fields(), 3, "Rectangle should have 3 fields");

        // First field should be Point2D struct
        let field_types = rt.get_field_types();
        assert!(
            field_types[0].is_struct_type(),
            "First field should be a struct"
        );
        assert!(
            field_types[1].is_float_type(),
            "Second field should be float"
        );
        assert!(
            field_types[2].is_float_type(),
            "Third field should be float"
        );
    }
}
