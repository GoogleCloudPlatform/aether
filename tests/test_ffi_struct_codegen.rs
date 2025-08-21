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
use aether::semantic::SemanticAnalyzer;
use aether::types::TypeChecker;
use aether::mir::lowering::lower_ast_to_mir;
use std::rc::Rc;
use std::cell::RefCell;

fn create_test_module_with_struct() -> Module {
    Module {
        name: Identifier::new("test_module".to_string(), SourceLocation::unknown()),
        intent: Some("Test module for FFI structs".to_string()),
        imports: vec![],
        exports: vec![],
        type_definitions: vec![
            TypeDefinition::Structured {
                name: Identifier::new("Point2D".to_string(), SourceLocation::unknown()),
                intent: Some("2D point structure".to_string()),
                generic_parameters: vec![],
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
                source_location: SourceLocation::unknown(),
            }
        ],
        constant_declarations: vec![],
        function_definitions: vec![
            Function {
                name: Identifier::new("test_struct_passing".to_string(), SourceLocation::unknown()),
                intent: Some("Test struct passing".to_string()),
                generic_parameters: vec![],
                parameters: vec![
                    Parameter {
                        name: Identifier::new("p".to_string(), SourceLocation::unknown()),
                        param_type: Box::new(TypeSpecifier::Named {
                            name: Identifier::new("Point2D".to_string(), SourceLocation::unknown()),
                            source_location: SourceLocation::unknown(),
                        }),
                        intent: None,
                        constraint: None,
                        passing_mode: PassingMode::ByValue,
                        source_location: SourceLocation::unknown(),
                    }
                ],
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
                    statements: vec![
                        Statement::Return {
                            value: Some(Box::new(Expression::FieldAccess {
                                instance: Box::new(Expression::Variable {
                                    name: Identifier::new("p".to_string(), SourceLocation::unknown()),
                                    source_location: SourceLocation::unknown(),
                                }),
                                field_name: Identifier::new("x".to_string(), SourceLocation::unknown()),
                                source_location: SourceLocation::unknown(),
                            })),
                            source_location: SourceLocation::unknown(),
                        }
                    ],
                    source_location: SourceLocation::unknown(),
                },
                export_info: None,
                source_location: SourceLocation::unknown(),
            }
        ],
        external_functions: vec![
            ExternalFunction {
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
            }
        ],
        source_location: SourceLocation::unknown(),
    }
}

#[test]
fn test_struct_type_generation() {
    let module = create_test_module_with_struct();
    let mut program = Program {
        modules: vec![module],
        source_location: SourceLocation::unknown(),
    };
    
    // Run semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis failed");
    
    // Convert to MIR
    let mir_program = lower_ast_to_mir(&program).expect("MIR lowering failed");
    
    // Create LLVM backend
    let context = inkwell::context::Context::create();
    let mut backend = LLVMBackend::new(&context, "test");
    
    // Generate code
    backend.generate_ir(&mir_program).expect("LLVM code generation failed");
    
    // Get the generated module
    let llvm_module = backend.module();
    
    // Verify struct type was created
    let struct_type = llvm_module.get_struct_type("Point2D");
    assert!(struct_type.is_some(), "Point2D struct type not found in LLVM module");
    
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
    let mut program = Program {
        modules: vec![module],
        source_location: SourceLocation::unknown(),
    };
    
    // Run semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis failed");
    
    // Convert to MIR
    let mir_program = lower_ast_to_mir(&program).expect("MIR lowering failed");
    
    // Create LLVM backend
    let context = inkwell::context::Context::create();
    let mut backend = LLVMBackend::new(&context, "test");
    
    // Generate code
    backend.generate_ir(&mir_program).expect("LLVM code generation failed");
    
    // Get the generated module
    let llvm_module = backend.module();
    
    // Verify the test function was created with correct signature
    let func = llvm_module.get_function("test_struct_passing");
    assert!(func.is_some(), "test_struct_passing function not found");
    
    if let Some(f) = func {
        // Should have one parameter (the struct)
        assert_eq!(f.count_params(), 1, "Function should have 1 parameter");
        
        // Parameter should be the struct type
        let param = f.get_first_param().unwrap();
        assert!(param.is_struct_value(), "Parameter should be a struct");
    }
}

#[test]
fn test_external_struct_function() {
    let module = create_test_module_with_struct();
    let mut program = Program {
        modules: vec![module],
        source_location: SourceLocation::unknown(),
    };
    
    // Run semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis failed");
    
    // Convert to MIR
    let mir_program = lower_ast_to_mir(&program).expect("MIR lowering failed");
    
    // Create LLVM backend
    let context = inkwell::context::Context::create();
    let mut backend = LLVMBackend::new(&context, "test");
    
    // Generate code
    backend.generate_ir(&mir_program).expect("LLVM code generation failed");
    
    // Get the generated module
    let llvm_module = backend.module();
    
    // Verify the external function declaration was created
    let func = llvm_module.get_function("point_distance");
    assert!(func.is_some(), "point_distance external function not found");
    
    if let Some(f) = func {
        // Should have two struct parameters
        assert_eq!(f.count_params(), 2, "Function should have 2 parameters");
        
        // Both parameters should be structs
        let param1 = f.get_first_param().unwrap();
        let param2 = f.get_nth_param(1).unwrap();
        assert!(param1.is_struct_value(), "First parameter should be a struct");
        assert!(param2.is_struct_value(), "Second parameter should be a struct");
        
        // Return type should be f64
        let return_type = f.get_type().get_return_type();
        assert!(return_type.is_some());
        assert!(return_type.unwrap().is_float_type());
    }
}

#[test]
fn test_nested_struct_generation() {
    let mut module = Module {
        name: Identifier::new("test_nested".to_string(), SourceLocation::unknown()),
        intent: None,
        imports: vec![],
        exports: vec![],
        type_definitions: vec![
            TypeDefinition::Structured {
                name: Identifier::new("Point2D".to_string(), SourceLocation::unknown()),
                intent: None,
                generic_parameters: vec![],
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
                source_location: SourceLocation::unknown(),
            },
            TypeDefinition::Structured {
                name: Identifier::new("Rectangle".to_string(), SourceLocation::unknown()),
                intent: None,
                generic_parameters: vec![],
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
                source_location: SourceLocation::unknown(),
            },
        ],
        constant_declarations: vec![],
        function_definitions: vec![],
        external_functions: vec![],
        source_location: SourceLocation::unknown(),
    };
    
    let mut program = Program {
        modules: vec![module],
        source_location: SourceLocation::unknown(),
    };
    
    // Run semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze_program(&program).expect("Semantic analysis failed");
    
    // Convert to MIR
    let mir_program = lower_ast_to_mir(&program).expect("MIR lowering failed");
    
    // Create LLVM backend
    let context = inkwell::context::Context::create();
    let mut backend = LLVMBackend::new(&context, "test");
    
    // Generate code
    backend.generate_ir(&mir_program).expect("LLVM code generation failed");
    
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
        assert!(field_types[0].is_struct_type(), "First field should be a struct");
        assert!(field_types[1].is_float_type(), "Second field should be float");
        assert!(field_types[2].is_float_type(), "Third field should be float");
    }
}