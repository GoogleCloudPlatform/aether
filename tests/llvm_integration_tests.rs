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

//! Integration tests for LLVM backend code generation

use aether::llvm_backend::{LLVMBackend, TargetArch};
use aether::mir::Program;
use std::collections::HashMap;
use std::fs;
use inkwell::context::Context;

#[test]
fn test_llvm_backend_initialization() {
    LLVMBackend::initialize_targets();
    
    let context = Context::create();
    let backend = LLVMBackend::new(&context, "test_module");
    
    assert_eq!(backend.module().get_name().to_str().unwrap(), "test_module");
    assert!(backend.verify().is_ok());
}

#[test]
fn test_empty_program_generation() {
    LLVMBackend::initialize_targets();
    
    let context = Context::create();
    let mut backend = LLVMBackend::new(&context, "empty_program");
    
    let program = Program {
        functions: HashMap::new(),
        global_constants: HashMap::new(),
        external_functions: HashMap::new(),
        type_definitions: HashMap::new(),
    };
    
    assert!(backend.generate_ir(&program).is_ok());
    assert!(backend.verify().is_ok());
    
    let ir_string = backend.get_ir_string();
    assert!(ir_string.contains("empty_program"));
}

#[test]
fn test_target_architecture_setting() {
    LLVMBackend::initialize_targets();
    
    let context = Context::create();
    let mut backend = LLVMBackend::new(&context, "target_test");
    
    // Test setting different target architectures
    let targets = vec![
        TargetArch::X86_64,
        TargetArch::AArch64,
        TargetArch::X86,
    ];
    
    for target in targets {
        let result = backend.set_target_triple(target.target_triple());
        assert!(result.is_ok(), "Failed to set target: {:?}", target);
    }
}

#[test]
fn test_ir_file_output() {
    LLVMBackend::initialize_targets();
    
    let context = Context::create();
    let mut backend = LLVMBackend::new(&context, "file_output_test");
    
    let program = Program {
        functions: HashMap::new(),
        global_constants: HashMap::new(),
        external_functions: HashMap::new(),
        type_definitions: HashMap::new(),
    };
    
    assert!(backend.generate_ir(&program).is_ok());
    
    // Test writing IR to file
    let temp_path = "test_output.ll";
    let result = backend.write_ir_to_file(temp_path);
    assert!(result.is_ok());
    
    // Verify file was created and has content
    let content = fs::read_to_string(temp_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("file_output_test"));
    
    // Clean up
    let _ = fs::remove_file(temp_path);
}

#[test]
fn test_object_file_generation() {
    LLVMBackend::initialize_targets();
    
    let context = Context::create();
    let mut backend = LLVMBackend::new(&context, "object_test");
    
    // Set target triple
    let arch = TargetArch::native();
    assert!(backend.set_target_triple(arch.target_triple()).is_ok());
    
    let program = Program {
        functions: HashMap::new(),
        global_constants: HashMap::new(),
        external_functions: HashMap::new(),
        type_definitions: HashMap::new(),
    };
    
    assert!(backend.generate_ir(&program).is_ok());
    assert!(backend.verify().is_ok());
    
    // Test writing object file
    let temp_path = "test_output.o";
    let result = backend.write_object_file(temp_path);
    assert!(result.is_ok());
    
    // Verify file was created
    assert!(std::path::Path::new(temp_path).exists());
    let metadata = fs::metadata(temp_path).unwrap();
    assert!(metadata.len() > 0);
    
    // Clean up
    let _ = fs::remove_file(temp_path);
}

#[test]
fn test_assembly_file_generation() {
    LLVMBackend::initialize_targets();
    
    let context = Context::create();
    let mut backend = LLVMBackend::new(&context, "assembly_test");
    
    // Set target triple
    let arch = TargetArch::native();
    assert!(backend.set_target_triple(arch.target_triple()).is_ok());
    
    let program = Program {
        functions: HashMap::new(),
        global_constants: HashMap::new(),
        external_functions: HashMap::new(),
        type_definitions: HashMap::new(),
    };
    
    assert!(backend.generate_ir(&program).is_ok());
    assert!(backend.verify().is_ok());
    
    // Test writing assembly file
    let temp_path = "test_output.s";
    let result = backend.write_assembly_file(temp_path);
    assert!(result.is_ok());
    
    // Verify file was created and has content
    assert!(std::path::Path::new(temp_path).exists());
    let content = fs::read_to_string(temp_path).unwrap();
    assert!(!content.is_empty());
    
    // Clean up
    let _ = fs::remove_file(temp_path);
}