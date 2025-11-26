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

//! LLVM Backend for AetherScript
//!
//! Translates optimized MIR to LLVM IR and generates machine code

pub mod codegen;
pub mod context;
pub mod types;
pub mod values;

use crate::error::SemanticError;
use crate::mir::{self, Program};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::AddressSpace;
use inkwell::OptimizationLevel;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Type IDs for cleanup functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeId {
    Unknown = 0,
    String = 1,
    Array = 2,
    Map = 3,
}

/// Information about a local that needs cleanup
#[derive(Debug, Clone)]
struct LocalCleanupInfo {
    local_id: mir::LocalId,
    type_id: TypeId,
}

/// Information about values that need cleanup
#[derive(Debug, Clone)]
struct CleanupInfo {
    /// Locals that need to be dropped when going out of scope
    owned_locals: HashMap<mir::LocalId, TypeId>,
    /// Mapping from basic block to locals that need cleanup when exiting that block
    block_cleanup: HashMap<mir::BasicBlockId, Vec<LocalCleanupInfo>>,
}

impl CleanupInfo {
    fn new() -> Self {
        Self {
            owned_locals: HashMap::new(),
            block_cleanup: HashMap::new(),
        }
    }

    /// Mark a local as needing cleanup
    fn add_owned_local(
        &mut self,
        local_id: mir::LocalId,
        block_id: mir::BasicBlockId,
        type_id: TypeId,
    ) {
        self.owned_locals.insert(local_id, type_id);
        self.block_cleanup
            .entry(block_id)
            .or_insert_with(Vec::new)
            .push(LocalCleanupInfo { local_id, type_id });
    }

    /// Get locals that need cleanup for a block
    fn get_cleanup_locals(&self, block_id: &mir::BasicBlockId) -> Vec<LocalCleanupInfo> {
        self.block_cleanup
            .get(block_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get type ID for a local
    fn get_type_id(&self, local_id: &mir::LocalId) -> TypeId {
        self.owned_locals
            .get(local_id)
            .copied()
            .unwrap_or(TypeId::Unknown)
    }
}

/// LLVM backend for code generation
pub struct LLVMBackend<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    target_machine: Option<TargetMachine>,
    function_declarations: Option<HashMap<String, FunctionValue<'ctx>>>,
    string_globals: HashMap<String, PointerValue<'ctx>>,
    type_definitions: HashMap<String, crate::types::TypeDefinition>,
}

impl<'ctx> LLVMBackend<'ctx> {
    /// Create a new LLVM backend instance
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);

        Self {
            context,
            module,
            target_machine: None,
            function_declarations: None,
            string_globals: HashMap::new(),
            type_definitions: HashMap::new(),
        }
    }

    /// Convert an AetherScript type to an LLVM basic type
    fn get_basic_type(&self, ty: &crate::types::Type) -> inkwell::types::BasicTypeEnum<'ctx> {
        match ty {
            crate::types::Type::Primitive(prim) => match prim {
                crate::ast::PrimitiveType::Integer => self.context.i32_type().into(),
                crate::ast::PrimitiveType::Integer64 => self.context.i64_type().into(),
                crate::ast::PrimitiveType::Float => self.context.f64_type().into(),
                crate::ast::PrimitiveType::Boolean => self.context.i32_type().into(), // Use i32 for bool
                crate::ast::PrimitiveType::String => self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into(),
                crate::ast::PrimitiveType::Char => self.context.i8_type().into(), // Use i8 for char
                _ => self.context.i32_type().into(), // Default fallback
            },
            crate::types::Type::Array { .. } => {
                // Arrays are represented as pointers to the array data structure
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            crate::types::Type::Map { .. } => {
                // Maps are represented as pointers to the map data structure
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            crate::types::Type::Named { name, .. } => {
                // For structs, we need to use the actual struct type if it's been defined
                // For FFI, structs are always passed by pointer
                if let Some(type_def) = self.type_definitions.get(name) {
                    if let crate::types::TypeDefinition::Struct { .. } = type_def {
                        // Return pointer to struct for FFI compatibility
                        self.context
                            .i8_type()
                            .ptr_type(AddressSpace::default())
                            .into()
                    } else {
                        // Non-struct named types - use opaque pointer
                        self.context
                            .i8_type()
                            .ptr_type(AddressSpace::default())
                            .into()
                    }
                } else {
                    // Unknown type - use opaque pointer
                    self.context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .into()
                }
            }
            crate::types::Type::Pointer { .. } => {
                // Pointers are represented as i8*
                self.context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .into()
            }
            _ => self.context.i32_type().into(), // Default for complex types
        }
    }

    /// Get the basic type from a local ID
    fn get_basic_type_from_local(
        &self,
        local_id: mir::LocalId,
        function: &mir::Function,
    ) -> Result<inkwell::types::BasicTypeEnum<'ctx>, SemanticError> {
        // Check if it's a local
        if let Some(local) = function.locals.get(&local_id) {
            Ok(self.get_basic_type(&local.ty))
        } else {
            // Check if it's a parameter
            for param in &function.parameters {
                if param.local_id == local_id {
                    return Ok(self.get_basic_type(&param.ty));
                }
            }
            Err(SemanticError::CodeGenError {
                message: format!("Local {} not found", local_id),
            })
        }
    }

    /// Check if a type needs cleanup and return the type ID
    fn needs_cleanup(&self, ty: &crate::types::Type) -> Option<TypeId> {
        use crate::types::{OwnershipKind, Type};

        match ty {
            Type::Owned {
                ownership: OwnershipKind::Owned,
                base_type,
            } => {
                // Determine the type ID based on the base type
                match base_type.as_ref() {
                    Type::Primitive(crate::ast::PrimitiveType::String) => Some(TypeId::String),
                    Type::Array { .. } => Some(TypeId::Array),
                    Type::Map { .. } => Some(TypeId::Map),
                    Type::Named { .. } => Some(TypeId::Unknown), // Custom types - would need more info
                    _ => None,                                   // Other types don't need cleanup
                }
            }
            _ => None,
        }
    }

    /// Get the size of a type in bytes
    fn get_type_size(&self, ty: &crate::types::Type) -> u64 {
        match ty {
            crate::types::Type::Primitive(prim) => match prim {
                crate::ast::PrimitiveType::Integer | crate::ast::PrimitiveType::Integer32 => 4,
                crate::ast::PrimitiveType::Integer64 => 8,
                crate::ast::PrimitiveType::Float | crate::ast::PrimitiveType::Float64 => 8,
                crate::ast::PrimitiveType::Float32 => 4,
                crate::ast::PrimitiveType::Boolean => 4, // i32
                crate::ast::PrimitiveType::Char => 1,    // i8
                crate::ast::PrimitiveType::String => 8,  // pointer
                _ => 8,                                  // Default to pointer size
            },
            crate::types::Type::Array { .. } => 8,   // pointer
            crate::types::Type::Named { .. } => 8,   // pointer for structs
            crate::types::Type::Pointer { .. } => 8, // pointer
            _ => 8,                                  // Default to pointer size
        }
    }

    /// Calculate field offset for a given struct type and field index
    fn calculate_field_offset(&self, struct_name: &str, field_index: usize) -> u64 {
        if let Some(type_def) = self.type_definitions.get(struct_name) {
            if let crate::types::TypeDefinition::Struct { fields, .. } = type_def {
                let mut current_offset = 0u64;
                for (i, (_, field_ty)) in fields.iter().enumerate() {
                    if i == field_index {
                        break;
                    }
                    let field_size = self.get_type_size(field_ty);
                    current_offset += field_size;
                    // Add alignment padding
                    let alignment = field_size.min(8);
                    if current_offset % alignment != 0 {
                        current_offset = (current_offset + alignment - 1) / alignment * alignment;
                    }
                }
                return current_offset;
            }
        }
        // Fallback: assume 8-byte fields
        (field_index * 8) as u64
    }

    /// Initialize LLVM targets
    pub fn initialize_targets() {
        Target::initialize_all(&InitializationConfig::default());
    }

    /// Set the target triple for code generation
    pub fn set_target_triple(&mut self, triple: &str) -> Result<(), String> {
        let target_triple = TargetTriple::create(triple);
        self.module.set_triple(&target_triple);

        let target = Target::from_triple(&target_triple)
            .map_err(|e| format!("Failed to create target: {}", e))?;

        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic", // CPU
                "",        // Features
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or("Failed to create target machine")?;

        self.module
            .set_data_layout(&target_machine.get_target_data().get_data_layout());
        self.target_machine = Some(target_machine);

        Ok(())
    }

    /// Generate LLVM IR from MIR program
    pub fn generate_ir(&mut self, program: &Program) -> Result<(), SemanticError> {
        eprintln!(
            "MIR program has {} functions, {} constants, {} externals, {} types",
            program.functions.len(),
            program.global_constants.len(),
            program.external_functions.len(),
            program.type_definitions.len()
        );

        // Store type definitions
        self.type_definitions = program.type_definitions.clone();

        // First, create a type converter and define all struct types
        let mut type_converter = types::TypeConverter::new(self.context);
        for (name, type_def) in &program.type_definitions {
            if let crate::types::TypeDefinition::Struct { fields, .. } = type_def {
                eprintln!("Defining LLVM struct type for '{}'", name);
                type_converter.define_struct_type(name, fields)?;
            }
        }

        // First pass: declare all functions
        let mut function_declarations: HashMap<String, FunctionValue<'ctx>> = HashMap::new();

        // Declare built-in functions
        self.declare_builtin_functions(&mut function_declarations)?;

        // Declare external functions
        for (name, ext_func) in &program.external_functions {
            // Skip if already declared (e.g., as a builtin)
            if function_declarations.contains_key(name) {
                continue;
            }

            let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = ext_func
                .parameters
                .iter()
                .map(|param_ty| self.get_basic_type(param_ty).into())
                .collect();

            let fn_type = match &ext_func.return_type {
                crate::types::Type::Primitive(crate::ast::PrimitiveType::Void) => self
                    .context
                    .void_type()
                    .fn_type(&param_types, ext_func.variadic),
                crate::types::Type::Primitive(crate::ast::PrimitiveType::Integer) => self
                    .context
                    .i32_type()
                    .fn_type(&param_types, ext_func.variadic),
                crate::types::Type::Primitive(crate::ast::PrimitiveType::Integer64) => self
                    .context
                    .i64_type()
                    .fn_type(&param_types, ext_func.variadic),
                crate::types::Type::Primitive(crate::ast::PrimitiveType::Float) => self
                    .context
                    .f64_type()
                    .fn_type(&param_types, ext_func.variadic),
                crate::types::Type::Primitive(crate::ast::PrimitiveType::Boolean) => self
                    .context
                    .i32_type()
                    .fn_type(&param_types, ext_func.variadic),
                crate::types::Type::Primitive(crate::ast::PrimitiveType::String) => self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .fn_type(&param_types, ext_func.variadic),
                _ => {
                    // For other types, default to i32
                    self.context
                        .i32_type()
                        .fn_type(&param_types, ext_func.variadic)
                }
            };

            let llvm_func = self.module.add_function(name, fn_type, None);
            function_declarations.insert(name.clone(), llvm_func);
        }

        for (name, function) in &program.functions {
            // Special handling for main function
            if name == "main" {
                // Check if main has argc/argv parameters
                let has_argc_argv = function.parameters.len() == 2
                    && matches!(
                        &function.parameters[0].ty,
                        crate::types::Type::Primitive(crate::ast::PrimitiveType::Integer)
                    )
                    && matches!(&function.parameters[1].ty, crate::types::Type::Array { .. });

                if has_argc_argv {
                    // Standard C main signature: int main(int argc, char* argv[])
                    let i32_type = self.context.i32_type();
                    let i8_type = self.context.i8_type();
                    let argv_type = i8_type
                        .ptr_type(AddressSpace::default())
                        .ptr_type(AddressSpace::default());
                    let param_types = vec![i32_type.into(), argv_type.into()];
                    let fn_type = i32_type.fn_type(&param_types, false);
                    let llvm_func = self.module.add_function(name, fn_type, None);
                    function_declarations.insert(name.clone(), llvm_func);
                } else if function.parameters.is_empty() {
                    // Create wrapper main that calls user's main
                    // First, declare user's main with a different name
                    let user_main_name = "__aether_main";
                    let fn_type = self.context.i32_type().fn_type(&[], false);
                    let user_main = self.module.add_function(user_main_name, fn_type, None);
                    function_declarations.insert(user_main_name.to_string(), user_main);

                    // Create the real main function
                    let i32_type = self.context.i32_type();
                    let i8_type = self.context.i8_type();
                    let argv_type = i8_type
                        .ptr_type(AddressSpace::default())
                        .ptr_type(AddressSpace::default());
                    let param_types = vec![i32_type.into(), argv_type.into()];
                    let main_fn_type = i32_type.fn_type(&param_types, false);
                    let main_func = self.module.add_function("main", main_fn_type, None);

                    // Generate wrapper body
                    let builder = self.context.create_builder();
                    let entry = self.context.append_basic_block(main_func, "entry");
                    builder.position_at_end(entry);

                    // Call runtime init
                    if let Some(init_fn) = self.module.get_function("aether_runtime_init") {
                        let _ = builder.build_call(init_fn, &[], "call_runtime_init");
                    }

                    // Call user's main
                    match builder.build_call(user_main, &[], "call_user_main") {
                        Ok(call_site) => {
                            let return_value =
                                if let Some(basic_value) = call_site.try_as_basic_value().left() {
                                    basic_value.into_int_value()
                                } else {
                                    i32_type.const_int(0, false)
                                };
                            builder.build_return(Some(&return_value));
                        }
                        Err(_) => {
                            // If call fails, just return 0
                            builder.build_return(Some(&i32_type.const_int(0, false)));
                        }
                    }
                } else {
                    // Main with non-standard parameters - error
                    return Err(SemanticError::CodeGenError {
                        message: "main function must either have no parameters or (argc: INTEGER, argv: ARRAY_OF_TYPE STRING)".to_string()
                    });
                }
            } else {
                // Regular function
                let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = function
                    .parameters
                    .iter()
                    .map(|param| {
                        match &param.ty {
                            crate::types::Type::Primitive(crate::ast::PrimitiveType::Integer) => {
                                self.context.i32_type().into()
                            }
                            crate::types::Type::Primitive(crate::ast::PrimitiveType::Integer64) => {
                                self.context.i64_type().into()
                            }
                            crate::types::Type::Primitive(crate::ast::PrimitiveType::Float) => {
                                self.context.f64_type().into()
                            }
                            crate::types::Type::Primitive(crate::ast::PrimitiveType::Boolean) => {
                                self.context.i32_type().into()
                            }
                            crate::types::Type::Primitive(crate::ast::PrimitiveType::String) => {
                                self.context
                                    .i8_type()
                                    .ptr_type(AddressSpace::default())
                                    .into()
                            }
                            crate::types::Type::Primitive(crate::ast::PrimitiveType::Char) => {
                                self.context.i8_type().into()
                            }
                            crate::types::Type::Named { .. } => {
                                // Named types (structs, enums) are passed as pointers
                                self.context
                                    .i8_type()
                                    .ptr_type(AddressSpace::default())
                                    .into()
                            }
                            crate::types::Type::Array { .. } => {
                                // Arrays are passed as pointers (void*)
                                self.context
                                    .i8_type()
                                    .ptr_type(AddressSpace::default())
                                    .into()
                            }
                            crate::types::Type::Map { .. } => {
                                // Maps are passed as pointers (void*)
                                self.context
                                    .i8_type()
                                    .ptr_type(AddressSpace::default())
                                    .into()
                            }
                            _ => self.context.i32_type().into(),
                        }
                    })
                    .collect();

                let fn_type = match &function.return_type {
                    crate::types::Type::Primitive(crate::ast::PrimitiveType::Void) => {
                        self.context.void_type().fn_type(&param_types, false)
                    }
                    crate::types::Type::Primitive(crate::ast::PrimitiveType::Integer) => {
                        self.context.i32_type().fn_type(&param_types, false)
                    }
                    crate::types::Type::Primitive(crate::ast::PrimitiveType::Integer64) => {
                        self.context.i64_type().fn_type(&param_types, false)
                    }
                    crate::types::Type::Primitive(crate::ast::PrimitiveType::Float) => {
                        self.context.f64_type().fn_type(&param_types, false)
                    }
                    crate::types::Type::Primitive(crate::ast::PrimitiveType::Boolean) => {
                        self.context.i32_type().fn_type(&param_types, false)
                    }
                    crate::types::Type::Primitive(crate::ast::PrimitiveType::String) => self
                        .context
                        .i8_type()
                        .ptr_type(AddressSpace::default())
                        .fn_type(&param_types, false),
                    crate::types::Type::Primitive(crate::ast::PrimitiveType::Char) => {
                        self.context.i8_type().fn_type(&param_types, false)
                    }
                    crate::types::Type::Named { .. } => {
                        // Named types (structs, enums) are returned as pointers
                        self.context
                            .i8_type()
                            .ptr_type(AddressSpace::default())
                            .fn_type(&param_types, false)
                    }
                    crate::types::Type::Array { .. } => {
                        // Arrays are returned as pointers (void*)
                        self.context
                            .i8_type()
                            .ptr_type(AddressSpace::default())
                            .fn_type(&param_types, false)
                    }
                    crate::types::Type::Map { .. } => {
                        // Maps are returned as pointers (void*)
                        self.context
                            .i8_type()
                            .ptr_type(AddressSpace::default())
                            .fn_type(&param_types, false)
                    }
                    _ => self.context.i32_type().fn_type(&param_types, false),
                };

                let llvm_func = self.module.add_function(name, fn_type, None);
                function_declarations.insert(name.clone(), llvm_func);
            }
        }

        // Store function declarations for use in call generation
        self.function_declarations = Some(function_declarations);

        // Second pass: generate function bodies
        for (name, function) in &program.functions {
            eprintln!("Processing MIR function: {}", name);
            if name == "main" && function.parameters.is_empty() {
                // For parameterless main, we generate it as __aether_main
                self.generate_function_body_only("__aether_main", function)?;
            } else {
                self.generate_function_body_only(name, function)?;
            }
        }

        Ok(())
    }

    /// Generate function body only (assumes function already declared)
    fn generate_function_body_only(
        &mut self,
        name: &str,
        function: &mir::Function,
    ) -> Result<(), SemanticError> {
        let llvm_func = self
            .function_declarations
            .as_ref()
            .and_then(|decls| decls.get(name))
            .ok_or_else(|| SemanticError::CodeGenError {
                message: format!("Function {} not found in declarations", name),
            })?;

        self.generate_function_body(function, *llvm_func)?;
        Ok(())
    }

    /// Generate function body from MIR
    fn generate_function_body(
        &mut self,
        function: &mir::Function,
        llvm_func: FunctionValue<'ctx>,
    ) -> Result<(), SemanticError> {
        use inkwell::basic_block::BasicBlock;
        use std::collections::HashMap;

        // Create cleanup tracking
        let mut cleanup_info = CleanupInfo::new();

        // Create mapping from MIR block IDs to LLVM basic blocks
        let mut llvm_blocks: HashMap<mir::BasicBlockId, BasicBlock<'ctx>> = HashMap::new();

        // Create all LLVM basic blocks first
        // Create entry block first with the correct name
        let entry_block = self.context.append_basic_block(llvm_func, "entry");
        llvm_blocks.insert(function.entry_block, entry_block);

        // Create other blocks
        for &block_id in function.basic_blocks.keys() {
            if block_id != function.entry_block {
                let block_name = format!("bb{}", block_id);
                let llvm_block = self.context.append_basic_block(llvm_func, &block_name);
                llvm_blocks.insert(block_id, llvm_block);
            }
        }

        // Create builder
        let builder = self.context.create_builder();

        // Create stack allocations for all locals (except parameters)
        let mut local_allocas: HashMap<mir::LocalId, PointerValue<'ctx>> = HashMap::new();

        // Position builder at entry block start for allocas
        builder.position_at_end(llvm_blocks[&function.entry_block]);

        // Allocate stack slots for non-parameter locals
        for (&local_id, local) in &function.locals {
            if !function.parameters.iter().any(|p| p.local_id == local_id) {
                let local_type = self.get_basic_type(&local.ty);
                let alloca = builder
                    .build_alloca(local_type, &format!("local_{}", local_id))
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
                local_allocas.insert(local_id, alloca);

                // Track if this local has ownership and needs cleanup
                if let Some(type_id) = self.needs_cleanup(&local.ty) {
                    cleanup_info.add_owned_local(local_id, function.entry_block, type_id);
                }
            }
        }

        // Store parameters in their allocas
        for (i, param) in function.parameters.iter().enumerate() {
            if let Some(llvm_param) = llvm_func.get_nth_param(i as u32) {
                let param_type = self.get_basic_type(&param.ty);
                let alloca = builder
                    .build_alloca(param_type, &format!("param_{}", param.local_id))
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
                builder.build_store(alloca, llvm_param).map_err(|e| {
                    SemanticError::CodeGenError {
                        message: e.to_string(),
                    }
                })?;
                local_allocas.insert(param.local_id, alloca);
            }
        }

        // Process blocks in order, starting with entry block
        let mut block_order: Vec<_> = function.basic_blocks.keys().copied().collect();
        block_order.sort();
        // Ensure entry block is first
        if let Some(entry_pos) = block_order
            .iter()
            .position(|&id| id == function.entry_block)
        {
            block_order.swap(0, entry_pos);
        }

        // Process each basic block in order
        for &block_id in &block_order {
            let mir_block = &function.basic_blocks[&block_id];
            eprintln!(
                "Processing block {}: {} statements, terminator: {:?}",
                block_id,
                mir_block.statements.len(),
                mir_block.terminator
            );
            let llvm_block = llvm_blocks[&block_id];
            builder.position_at_end(llvm_block);

            // Process statements
            for (i, stmt) in mir_block.statements.iter().enumerate() {
                match stmt {
                    mir::Statement::Assign { place, rvalue, .. } => {
                        let result =
                            self.generate_rvalue(rvalue, &local_allocas, &builder, function)?;
                        if let Some(&alloca) = local_allocas.get(&place.local) {
                            builder.build_store(alloca, result).map_err(|e| {
                                SemanticError::CodeGenError {
                                    message: e.to_string(),
                                }
                            })?;
                        }
                    }
                    mir::Statement::StorageLive(_) | mir::Statement::StorageDead(_) => {
                        // Ignore storage markers for now
                    }
                    mir::Statement::Nop => {
                        // Do nothing
                    }
                }
            }

            // Process terminator
            match &mir_block.terminator {
                mir::Terminator::Return => {
                    // Generate cleanup code for owned values before returning
                    self.generate_cleanup_for_block(
                        &cleanup_info,
                        &block_id,
                        &local_allocas,
                        &builder,
                    )?;

                    if let Some(return_local) = function.return_local {
                        if let Some(&return_alloca) = local_allocas.get(&return_local) {
                            // Get the type of the return local
                            let return_type = function
                                .locals
                                .get(&return_local)
                                .map(|local| self.get_basic_type(&local.ty))
                                .unwrap_or_else(|| self.context.i32_type().into());

                            let return_value = builder
                                .build_load(return_type, return_alloca, "ret_val")
                                .map_err(|e| SemanticError::CodeGenError {
                                    message: e.to_string(),
                                })?;
                            eprintln!(
                                "Returning value for local {}: {:?}",
                                return_local, return_value
                            );
                            builder.build_return(Some(&return_value)).map_err(|e| {
                                SemanticError::CodeGenError {
                                    message: e.to_string(),
                                }
                            })?;
                        } else {
                            eprintln!("Return local {} not found, returning default", return_local);
                            // Return default value based on return type
                            let default_value: BasicValueEnum = match &function.return_type {
                                crate::types::Type::Primitive(crate::ast::PrimitiveType::Float) => {
                                    self.context.f64_type().const_float(0.0).into()
                                }
                                _ => self.context.i32_type().const_int(0, false).into(),
                            };
                            builder.build_return(Some(&default_value)).map_err(|e| {
                                SemanticError::CodeGenError {
                                    message: e.to_string(),
                                }
                            })?;
                        }
                    } else {
                        eprintln!("Void return");
                        // Void return
                        builder
                            .build_return(None)
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                    }
                }

                mir::Terminator::Goto { target } => {
                    let target_block = llvm_blocks[target];
                    builder
                        .build_unconditional_branch(target_block)
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                }

                mir::Terminator::SwitchInt {
                    discriminant,
                    targets,
                    ..
                } => {
                    let cond_value =
                        self.generate_operand(discriminant, &local_allocas, &builder, function)?;

                    // Extract integer value from BasicValueEnum
                    let int_value = match cond_value {
                        BasicValueEnum::IntValue(v) => v,
                        _ => {
                            return Err(SemanticError::CodeGenError {
                                message: "Expected integer value for switch".to_string(),
                            })
                        }
                    };

                    // For boolean conditions, use conditional branch
                    if targets.values.len() == 1 && targets.values[0] == 1 {
                        let then_block = llvm_blocks[&targets.targets[0]];
                        let else_block = llvm_blocks[&targets.otherwise];

                        // Convert integer to boolean
                        let zero = self.context.i32_type().const_int(0, false);
                        let is_true = builder
                            .build_int_compare(
                                inkwell::IntPredicate::NE,
                                int_value,
                                zero,
                                "is_true",
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;

                        builder
                            .build_conditional_branch(is_true, then_block, else_block)
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                    } else if targets.values.len() > 1 {
                        // Build a proper switch instruction for multiple cases
                        let mut cases = Vec::new();
                        for (i, value) in targets.values.iter().enumerate() {
                            let case_value =
                                self.context.i32_type().const_int(*value as u64, false);
                            let case_block = llvm_blocks[&targets.targets[i]];
                            cases.push((case_value, case_block));
                        }

                        let otherwise_block = llvm_blocks[&targets.otherwise];
                        builder
                            .build_switch(int_value, otherwise_block, &cases)
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                    } else {
                        // No cases, just jump to otherwise block
                        let target_block = llvm_blocks[&targets.otherwise];
                        builder
                            .build_unconditional_branch(target_block)
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                    }
                }

                mir::Terminator::Unreachable => {
                    builder
                        .build_unreachable()
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                }

                mir::Terminator::Drop {
                    place,
                    target,
                    unwind,
                } => {
                    // Generate drop call for the place
                    self.generate_drop_for_place(place, &local_allocas, &builder)?;

                    // Jump to target block
                    let target_block = llvm_blocks[target];
                    builder
                        .build_unconditional_branch(target_block)
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;

                    // TODO: Handle unwind path for exceptions
                    if unwind.is_some() {
                        eprintln!("WARNING: Unwind path in Drop terminator not yet implemented");
                    }
                }

                mir::Terminator::Assert { .. } => {
                    // TODO: Implement assertion checks
                    return Err(SemanticError::CodeGenError {
                        message: "Assert terminator not yet implemented".to_string(),
                    });
                }

                mir::Terminator::Call {
                    func,
                    args,
                    destination,
                    target,
                    ..
                } => {
                    eprintln!("DEBUG: Processing Terminator::Call");
                    eprintln!("DEBUG: Function: {:?}", func);
                    eprintln!("DEBUG: Args: {:?}", args);
                    eprintln!("DEBUG: Destination: {:?}", destination);
                    eprintln!("DEBUG: Target: {:?}", target);

                    // Generate the function call
                    let rvalue = mir::Rvalue::Call {
                        func: func.clone(),
                        args: args.clone(),
                    };
                    eprintln!("DEBUG: Created Rvalue::Call: {:?}", rvalue);
                    let result =
                        self.generate_rvalue(&rvalue, &local_allocas, &builder, function)?;
                    eprintln!("DEBUG: Call result: {:?}", result);

                    // Store result in destination
                    if let Some(&alloca) = local_allocas.get(&destination.local) {
                        builder.build_store(alloca, result).map_err(|e| {
                            SemanticError::CodeGenError {
                                message: e.to_string(),
                            }
                        })?;
                        eprintln!("DEBUG: Stored call result in local {}", destination.local);
                    } else {
                        eprintln!(
                            "DEBUG: No alloca found for destination local {}",
                            destination.local
                        );
                    }

                    // Jump to target block
                    if let Some(target_block) = target {
                        let llvm_target = llvm_blocks[target_block];
                        builder
                            .build_unconditional_branch(llvm_target)
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        eprintln!("DEBUG: Jumped to target block {}", target_block);
                    } else {
                        eprintln!("DEBUG: No target block for call");
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate code for an rvalue
    fn generate_rvalue(
        &mut self,
        rvalue: &mir::Rvalue,
        local_allocas: &HashMap<mir::LocalId, PointerValue<'ctx>>,
        builder: &Builder<'ctx>,
        function: &mir::Function,
    ) -> Result<BasicValueEnum<'ctx>, SemanticError> {
        eprintln!("DEBUG: generate_rvalue called with: {:?}", rvalue);
        match rvalue {
            mir::Rvalue::Use(operand) => {
                self.generate_operand(operand, local_allocas, builder, function)
            }

            mir::Rvalue::BinaryOp { op, left, right } => {
                let left_val = self.generate_operand(left, local_allocas, builder, function)?;
                let right_val = self.generate_operand(right, local_allocas, builder, function)?;

                match (op, left_val, right_val) {
                    // Integer operations
                    (mir::BinOp::Add, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        builder
                            .build_int_add(l, r, "add")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (mir::BinOp::Sub, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        builder
                            .build_int_sub(l, r, "sub")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (mir::BinOp::Mul, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        builder
                            .build_int_mul(l, r, "mul")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (mir::BinOp::Div, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        builder
                            .build_int_signed_div(l, r, "div")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (mir::BinOp::Rem, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        builder
                            .build_int_signed_rem(l, r, "rem")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    // Float operations
                    (
                        mir::BinOp::Add,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => builder
                        .build_float_add(l, r, "fadd")
                        .map(|v| v.into())
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        }),
                    (
                        mir::BinOp::Sub,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => builder
                        .build_float_sub(l, r, "fsub")
                        .map(|v| v.into())
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        }),
                    (
                        mir::BinOp::Mul,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => builder
                        .build_float_mul(l, r, "fmul")
                        .map(|v| v.into())
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        }),
                    (
                        mir::BinOp::Div,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => builder
                        .build_float_div(l, r, "fdiv")
                        .map(|v| v.into())
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        }),
                    (
                        mir::BinOp::Rem,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => builder
                        .build_float_rem(l, r, "frem")
                        .map(|v| v.into())
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        }),
                    // Integer comparisons
                    (mir::BinOp::Eq, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        // Ensure both operands have the same type by promoting to the larger type
                        let (left_coerced, right_coerced) =
                            if l.get_type().get_bit_width() != r.get_type().get_bit_width() {
                                if l.get_type().get_bit_width() > r.get_type().get_bit_width() {
                                    let r_extended = builder
                                        .build_int_z_extend(r, l.get_type(), "r_ext")
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    (l, r_extended)
                                } else {
                                    let l_extended = builder
                                        .build_int_z_extend(l, r.get_type(), "l_ext")
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    (l_extended, r)
                                }
                            } else {
                                (l, r)
                            };

                        let cmp = builder
                            .build_int_compare(
                                inkwell::IntPredicate::EQ,
                                left_coerced,
                                right_coerced,
                                "eq",
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "eq_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (mir::BinOp::Gt, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let cmp = builder
                            .build_int_compare(inkwell::IntPredicate::SGT, l, r, "gt")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "gt_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (mir::BinOp::Lt, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let cmp = builder
                            .build_int_compare(inkwell::IntPredicate::SLT, l, r, "lt")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "lt_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (mir::BinOp::Le, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let cmp = builder
                            .build_int_compare(inkwell::IntPredicate::SLE, l, r, "le")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "le_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (mir::BinOp::Ge, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let cmp = builder
                            .build_int_compare(inkwell::IntPredicate::SGE, l, r, "ge")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "ge_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (mir::BinOp::Ne, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        // Ensure both operands have the same type by promoting to the larger type
                        let (left_coerced, right_coerced) =
                            if l.get_type().get_bit_width() != r.get_type().get_bit_width() {
                                if l.get_type().get_bit_width() > r.get_type().get_bit_width() {
                                    let r_extended = builder
                                        .build_int_z_extend(r, l.get_type(), "r_ext")
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    (l, r_extended)
                                } else {
                                    let l_extended = builder
                                        .build_int_z_extend(l, r.get_type(), "l_ext")
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    (l_extended, r)
                                }
                            } else {
                                (l, r)
                            };

                        let cmp = builder
                            .build_int_compare(
                                inkwell::IntPredicate::NE,
                                left_coerced,
                                right_coerced,
                                "ne",
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "ne_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    // Logical AND (for boolean operands stored as integers)
                    (mir::BinOp::And, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        // Ensure both are same size by extending if needed
                        let (left_coerced, right_coerced) =
                            if l.get_type().get_bit_width() != r.get_type().get_bit_width() {
                                let target_type = self.context.i32_type();
                                let l_ext = builder
                                    .build_int_z_extend(l, target_type, "l_ext")
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;
                                let r_ext = builder
                                    .build_int_z_extend(r, target_type, "r_ext")
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;
                                (l_ext, r_ext)
                            } else {
                                (l, r)
                            };
                        builder
                            .build_and(left_coerced, right_coerced, "logical_and")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    // Logical OR (for boolean operands stored as integers)
                    (mir::BinOp::Or, BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        // Ensure both are same size by extending if needed
                        let (left_coerced, right_coerced) =
                            if l.get_type().get_bit_width() != r.get_type().get_bit_width() {
                                let target_type = self.context.i32_type();
                                let l_ext = builder
                                    .build_int_z_extend(l, target_type, "l_ext")
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;
                                let r_ext = builder
                                    .build_int_z_extend(r, target_type, "r_ext")
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;
                                (l_ext, r_ext)
                            } else {
                                (l, r)
                            };
                        builder
                            .build_or(left_coerced, right_coerced, "logical_or")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    // Float comparisons
                    (
                        mir::BinOp::Eq,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => {
                        let cmp = builder
                            .build_float_compare(inkwell::FloatPredicate::OEQ, l, r, "feq")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "eq_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (
                        mir::BinOp::Gt,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => {
                        let cmp = builder
                            .build_float_compare(inkwell::FloatPredicate::OGT, l, r, "fgt")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "gt_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (
                        mir::BinOp::Lt,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => {
                        let cmp = builder
                            .build_float_compare(inkwell::FloatPredicate::OLT, l, r, "flt")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "lt_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (
                        mir::BinOp::Le,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => {
                        let cmp = builder
                            .build_float_compare(inkwell::FloatPredicate::OLE, l, r, "fle")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "le_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (
                        mir::BinOp::Ge,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => {
                        let cmp = builder
                            .build_float_compare(inkwell::FloatPredicate::OGE, l, r, "fge")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "ge_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    (
                        mir::BinOp::Ne,
                        BasicValueEnum::FloatValue(l),
                        BasicValueEnum::FloatValue(r),
                    ) => {
                        let cmp = builder
                            .build_float_compare(inkwell::FloatPredicate::ONE, l, r, "fne")
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        builder
                            .build_int_z_extend(cmp, self.context.i32_type(), "ne_ext")
                            .map(|v| v.into())
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })
                    }
                    // Pointer offset operation
                    (
                        mir::BinOp::Offset,
                        BasicValueEnum::PointerValue(ptr),
                        BasicValueEnum::IntValue(offset),
                    ) => {
                        // For pointer arithmetic, we need to know the pointee type
                        // In a complete implementation, we would track this information
                        // For now, we'll use a simple heuristic based on the left operand's type

                        // Try to determine pointee type from the pointer operand
                        // This is a simplified approach - in a real implementation we'd track types more carefully
                        let pointee_type = match left {
                            mir::Operand::Copy(place) | mir::Operand::Move(place) => {
                                // Try to get type info from the local
                                if let Ok(local_type) =
                                    self.get_basic_type_from_local(place.local, function)
                                {
                                    // If it's a pointer type, extract the pointee type
                                    match local_type {
                                        inkwell::types::BasicTypeEnum::PointerType(_) => {
                                            // Default to i32 for now - in a real implementation we'd track this
                                            self.context.i32_type()
                                        }
                                        _ => self.context.i32_type(),
                                    }
                                } else {
                                    self.context.i32_type()
                                }
                            }
                            _ => self.context.i32_type(),
                        };

                        let indices = vec![offset];
                        let result_ptr =
                            unsafe { builder.build_gep(pointee_type, ptr, &indices, "ptr_offset") }
                                .map_err(|e| SemanticError::CodeGenError {
                                    message: e.to_string(),
                                })?;

                        Ok(result_ptr.into())
                    }
                    _ => Err(SemanticError::CodeGenError {
                        message: format!("Unsupported binary operation or type mismatch: {:?}", op),
                    }),
                }
            }

            mir::Rvalue::Call { func, args } => {
                eprintln!("DEBUG: Processing Rvalue::Call case");
                eprintln!("DEBUG: Function operand: {:?}", func);
                eprintln!("DEBUG: Arguments: {:?}", args);

                // Extract function name from the func operand
                let function_name = match func {
                    mir::Operand::Constant(constant) => match &constant.value {
                        mir::ConstantValue::String(name) => {
                            eprintln!("DEBUG: Extracted function name: {}", name);
                            name.clone()
                        }
                        _ => {
                            eprintln!("DEBUG: Function operand is not a string constant");
                            return Err(SemanticError::CodeGenError {
                                message: "Function call with non-string function reference"
                                    .to_string(),
                            });
                        }
                    },
                    _ => {
                        eprintln!("DEBUG: Function operand is not a constant");
                        return Err(SemanticError::CodeGenError {
                            message: "Function call with non-constant function reference"
                                .to_string(),
                        });
                    }
                };

                // Get the LLVM function first (to avoid borrowing conflicts)
                eprintln!("DEBUG: Looking up function: {}", function_name);
                if let Some(decls) = self.function_declarations.as_ref() {
                    eprintln!(
                        "DEBUG: Available function declarations: {:?}",
                        decls.keys().collect::<Vec<_>>()
                    );
                } else {
                    eprintln!("DEBUG: No function declarations available");
                }

                let llvm_func_value = self
                    .function_declarations
                    .as_ref()
                    .and_then(|decls| decls.get(&function_name))
                    .copied()
                    .ok_or_else(|| SemanticError::CodeGenError {
                        message: format!("Function {} not found", function_name),
                    })?;

                eprintln!("DEBUG: Found LLVM function: {:?}", llvm_func_value);

                // Generate argument values
                let mut arg_values = Vec::new();

                // Special handling for map functions that need pointer arguments
                if function_name == "map_insert" || function_name == "map_get" {
                    eprintln!(
                        "DEBUG: Special handling for map function: {}",
                        function_name
                    );

                    // For map functions, we need to pass pointers to the key and value
                    for (i, arg) in args.iter().enumerate() {
                        eprintln!("DEBUG: Processing argument {}: {:?}", i, arg);
                        let arg_value =
                            self.generate_operand(arg, local_allocas, builder, function)?;
                        eprintln!("DEBUG: Generated argument value: {:?}", arg_value);

                        if i == 0 {
                            // First argument is the map pointer, pass as-is
                            // But check if it's being loaded as an integer when it should be a pointer
                            match arg_value {
                                BasicValueEnum::IntValue(int_val) => {
                                    // This is likely a map pointer that was loaded incorrectly
                                    // We need to treat it as a pointer
                                    eprintln!("DEBUG: Map argument loaded as integer, need to handle specially");
                                    // The value in local_1 should be a pointer, not an integer
                                    // Let's load it as a pointer instead
                                    if let mir::Operand::Copy(place) | mir::Operand::Move(place) =
                                        arg
                                    {
                                        if let Some(&alloca) = local_allocas.get(&place.local) {
                                            // Load as pointer
                                            let loaded_ptr = builder
                                                .build_load(
                                                    self.context
                                                        .i8_type()
                                                        .ptr_type(AddressSpace::default()),
                                                    alloca,
                                                    "map_ptr",
                                                )
                                                .map_err(|e| SemanticError::CodeGenError {
                                                    message: e.to_string(),
                                                })?;
                                            arg_values.push(loaded_ptr.into());
                                        } else {
                                            return Err(SemanticError::CodeGenError {
                                                message: format!(
                                                    "No alloca found for map local {}",
                                                    place.local
                                                ),
                                            });
                                        }
                                    } else {
                                        return Err(SemanticError::CodeGenError {
                                            message: "Expected place operand for map argument"
                                                .to_string(),
                                        });
                                    }
                                }
                                BasicValueEnum::PointerValue(_) => {
                                    // Already a pointer, use as-is
                                    arg_values.push(arg_value.into());
                                }
                                _ => {
                                    arg_values.push(arg_value.into());
                                }
                            }
                        } else {
                            // Key and value arguments need to be passed as pointers
                            match arg_value {
                                BasicValueEnum::IntValue(int_val) => {
                                    // Allocate space for the integer and store it
                                    let alloca = builder
                                        .build_alloca(int_val.get_type(), &format!("map_arg_{}", i))
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    builder.build_store(alloca, int_val).map_err(|e| {
                                        SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        }
                                    })?;

                                    // Cast to i8* for the function call
                                    let i8_ptr_type =
                                        self.context.i8_type().ptr_type(AddressSpace::default());
                                    let casted_ptr = builder
                                        .build_pointer_cast(
                                            alloca,
                                            i8_ptr_type,
                                            &format!("map_arg_{}_ptr", i),
                                        )
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    arg_values.push(casted_ptr.into());
                                }
                                BasicValueEnum::PointerValue(ptr_val) => {
                                    // Already a pointer, just cast to i8*
                                    let i8_ptr_type =
                                        self.context.i8_type().ptr_type(AddressSpace::default());
                                    let casted_ptr = builder
                                        .build_pointer_cast(
                                            ptr_val,
                                            i8_ptr_type,
                                            &format!("map_arg_{}_ptr", i),
                                        )
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    arg_values.push(casted_ptr.into());
                                }
                                _ => {
                                    // For other types, allocate and store
                                    let alloca = builder
                                        .build_alloca(
                                            arg_value.get_type(),
                                            &format!("map_arg_{}", i),
                                        )
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    builder.build_store(alloca, arg_value).map_err(|e| {
                                        SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        }
                                    })?;

                                    let i8_ptr_type =
                                        self.context.i8_type().ptr_type(AddressSpace::default());
                                    let casted_ptr = builder
                                        .build_pointer_cast(
                                            alloca,
                                            i8_ptr_type,
                                            &format!("map_arg_{}_ptr", i),
                                        )
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    arg_values.push(casted_ptr.into());
                                }
                            }
                        }
                    }
                } else {
                    // Normal function call
                    for (i, arg) in args.iter().enumerate() {
                        eprintln!("DEBUG: Processing argument {}: {:?}", i, arg);
                        let arg_value =
                            self.generate_operand(arg, local_allocas, builder, function)?;
                        eprintln!("DEBUG: Generated argument value: {:?}", arg_value);
                        arg_values.push(arg_value.into());
                    }
                }

                eprintln!("DEBUG: Generated {} argument values", arg_values.len());

                // Generate the call
                eprintln!("DEBUG: Generating LLVM call to function: {}", function_name);
                let call_result = builder
                    .build_call(
                        llvm_func_value,
                        &arg_values,
                        &format!("call_{}", function_name),
                    )
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;

                eprintln!("DEBUG: Call generated successfully: {:?}", call_result);

                // Extract return value
                if let Some(basic_value) = call_result.try_as_basic_value().left() {
                    eprintln!("DEBUG: Call has return value: {:?}", basic_value);

                    // Special handling for map_get return value
                    if function_name == "map_get" {
                        // map_get returns a void* pointer to the value
                        // We need to cast it to the appropriate type and dereference it
                        if let BasicValueEnum::PointerValue(ptr_val) = basic_value {
                            // For now, assume integer values in the map
                            // TODO: This should use type information from the map type
                            let i32_ptr_type =
                                self.context.i32_type().ptr_type(AddressSpace::default());
                            let casted_ptr = builder
                                .build_pointer_cast(ptr_val, i32_ptr_type, "map_value_ptr")
                                .map_err(|e| SemanticError::CodeGenError {
                                    message: e.to_string(),
                                })?;

                            // Dereference the pointer to get the value
                            let loaded_value = builder
                                .build_load(self.context.i32_type(), casted_ptr, "map_value")
                                .map_err(|e| SemanticError::CodeGenError {
                                    message: e.to_string(),
                                })?;
                            Ok(loaded_value)
                        } else {
                            Err(SemanticError::CodeGenError {
                                message: "map_get did not return a pointer".to_string(),
                            })
                        }
                    } else {
                        Ok(basic_value)
                    }
                } else {
                    eprintln!("DEBUG: Call has void return, returning dummy value");
                    // Void return - return a dummy value
                    Ok(self.context.i32_type().const_int(0, false).into())
                }
            }

            mir::Rvalue::UnaryOp { op, operand } => {
                let operand_val =
                    self.generate_operand(operand, local_allocas, builder, function)?;

                match (op, operand_val) {
                    (mir::UnOp::Neg, BasicValueEnum::IntValue(v)) => builder
                        .build_int_neg(v, "neg")
                        .map(|v| v.into())
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        }),
                    (mir::UnOp::Neg, BasicValueEnum::FloatValue(v)) => builder
                        .build_float_neg(v, "fneg")
                        .map(|v| v.into())
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        }),
                    (mir::UnOp::Not, BasicValueEnum::IntValue(v)) => builder
                        .build_not(v, "not")
                        .map(|v| v.into())
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        }),
                    _ => Err(SemanticError::CodeGenError {
                        message: format!("Unsupported unary operation or type mismatch: {:?}", op),
                    }),
                }
            }

            mir::Rvalue::Cast {
                operand,
                kind: _,
                ty,
            } => {
                // Handle type casts
                eprintln!("DEBUG: Processing cast to type: {:?}", ty);

                // Get the operand value
                let operand_value =
                    self.generate_operand(operand, local_allocas, builder, function)?;

                // Check if this is a cast to string (TO_STRING operation)
                if matches!(
                    ty,
                    crate::types::Type::Primitive(crate::ast::PrimitiveType::String)
                ) {
                    // Generate call to int_to_string
                    eprintln!("DEBUG: Generating int_to_string call for cast");

                    let int_to_string_func = self
                        .function_declarations
                        .as_ref()
                        .and_then(|decls| decls.get("int_to_string"))
                        .copied()
                        .ok_or_else(|| SemanticError::CodeGenError {
                            message: "int_to_string function not found".to_string(),
                        })?;

                    let call_result = builder
                        .build_call(
                            int_to_string_func,
                            &[operand_value.into()],
                            "call_int_to_string",
                        )
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;

                    if let Some(basic_value) = call_result.try_as_basic_value().left() {
                        Ok(basic_value)
                    } else {
                        Err(SemanticError::CodeGenError {
                            message: "int_to_string returned void".to_string(),
                        })
                    }
                } else {
                    // For other casts, just pass through for now
                    Ok(operand_value)
                }
            }

            mir::Rvalue::Aggregate { kind, operands } => {
                match kind {
                    mir::AggregateKind::Struct(struct_name, field_names) => {
                        eprintln!("DEBUG: Generating struct aggregate for {}", struct_name);

                        // Look up struct definition to get field types
                        let field_types = if let Some(type_def) =
                            self.type_definitions.get(struct_name)
                        {
                            if let crate::types::TypeDefinition::Struct { fields, .. } = type_def {
                                fields.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>()
                            } else {
                                eprintln!("WARNING: {} is not a struct type", struct_name);
                                vec![]
                            }
                        } else {
                            eprintln!(
                                "WARNING: Struct {} not found in type definitions",
                                struct_name
                            );
                            vec![]
                        };

                        // Calculate total size and field offsets
                        let mut field_offsets = Vec::new();
                        let mut current_offset = 0u64;
                        let mut struct_size = 0u64;

                        for (i, field_type) in field_types.iter().enumerate() {
                            field_offsets.push(current_offset);
                            let field_size = self.get_type_size(field_type);
                            current_offset += field_size;
                            if i < field_types.len() - 1 {
                                // Add padding for alignment (simplified - align to field size)
                                let alignment = field_size.min(8);
                                if current_offset % alignment != 0 {
                                    current_offset =
                                        (current_offset + alignment - 1) / alignment * alignment;
                                }
                            }
                        }
                        struct_size = current_offset;

                        // Allocate space for the struct
                        let struct_type = self.context.i8_type().array_type(struct_size as u32);
                        let struct_alloca = builder
                            .build_alloca(struct_type, &format!("{}_alloca", struct_name))
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;

                        // Store each field value
                        for (i, operand) in operands.iter().enumerate() {
                            let field_value =
                                self.generate_operand(operand, local_allocas, builder, function)?;

                            // Use calculated offset for this field
                            let offset = field_offsets.get(i).copied().unwrap_or(0);

                            // Get pointer to field location
                            let indices = vec![
                                self.context.i32_type().const_int(0, false),
                                self.context.i32_type().const_int(offset, false),
                            ];

                            let field_ptr = unsafe {
                                builder.build_in_bounds_gep(
                                    struct_type,
                                    struct_alloca,
                                    &indices,
                                    &format!("{}_field_{}_ptr", struct_name, i),
                                )
                            }
                            .map_err(|e| {
                                SemanticError::CodeGenError {
                                    message: e.to_string(),
                                }
                            })?;

                            // Cast the pointer to the appropriate type
                            let field_type_ptr = match field_value {
                                BasicValueEnum::IntValue(_) => {
                                    self.context.i32_type().ptr_type(AddressSpace::default())
                                }
                                BasicValueEnum::FloatValue(_) => {
                                    self.context.f64_type().ptr_type(AddressSpace::default())
                                }
                                BasicValueEnum::PointerValue(_) => self
                                    .context
                                    .i8_type()
                                    .ptr_type(AddressSpace::default())
                                    .ptr_type(AddressSpace::default()),
                                _ => self.context.i32_type().ptr_type(AddressSpace::default()),
                            };

                            let typed_field_ptr = builder
                                .build_pointer_cast(
                                    field_ptr,
                                    field_type_ptr,
                                    &format!("{}_field_{}_typed_ptr", struct_name, i),
                                )
                                .map_err(|e| SemanticError::CodeGenError {
                                    message: e.to_string(),
                                })?;

                            // Store the value
                            builder
                                .build_store(typed_field_ptr, field_value)
                                .map_err(|e| SemanticError::CodeGenError {
                                    message: e.to_string(),
                                })?;
                        }

                        // Return the struct as a pointer
                        let struct_ptr = builder
                            .build_pointer_cast(
                                struct_alloca,
                                self.context.i8_type().ptr_type(AddressSpace::default()),
                                &format!("{}_ptr", struct_name),
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;

                        Ok(struct_ptr.into())
                    }

                    mir::AggregateKind::Array(_element_type) => {
                        // TODO: Implement array aggregate construction
                        Err(SemanticError::CodeGenError {
                            message: "Array aggregate construction not yet implemented".to_string(),
                        })
                    }

                    mir::AggregateKind::Tuple => {
                        // TODO: Implement tuple aggregate construction
                        Err(SemanticError::CodeGenError {
                            message: "Tuple aggregate construction not yet implemented".to_string(),
                        })
                    }

                    mir::AggregateKind::Enum(enum_name, variant_name) => {
                        // TODO: Implement enum aggregate construction
                        // For now, enums are represented as tagged unions
                        // Layout: [discriminant: i32][data: largest variant size]

                        eprintln!(
                            "DEBUG: Generating enum aggregate for {}::{}",
                            enum_name, variant_name
                        );

                        // Calculate the enum layout
                        let discriminant_size = self.get_enum_discriminant_size(enum_name);
                        let data_size = 8; // TODO: Calculate based on largest variant data
                        let enum_size = discriminant_size + data_size;
                        let enum_type = self.context.i8_type().array_type(enum_size as u32);
                        let enum_alloca = builder
                            .build_alloca(enum_type, &format!("{}_alloca", enum_name))
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;

                        // Map variant names to discriminant values based on declaration order
                        // This is a simple approach - variants get indices based on their position
                        let discriminant =
                            self.get_enum_variant_discriminant(enum_name, variant_name);

                        // Store discriminant
                        let disc_ptr = builder
                            .build_pointer_cast(
                                enum_alloca,
                                self.context.i32_type().ptr_type(AddressSpace::default()),
                                &format!("{}_disc_ptr", enum_name),
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;

                        let disc_value = self.context.i32_type().const_int(discriminant, false);
                        builder.build_store(disc_ptr, disc_value).map_err(|e| {
                            SemanticError::CodeGenError {
                                message: e.to_string(),
                            }
                        })?;

                        // Store associated value if present
                        if !operands.is_empty() {
                            // Get pointer to data area (offset by 4 bytes)
                            // First cast to i8* to do byte-level arithmetic
                            let i8_ptr = builder
                                .build_pointer_cast(
                                    enum_alloca,
                                    self.context.i8_type().ptr_type(AddressSpace::default()),
                                    "enum_i8_ptr",
                                )
                                .map_err(|e| SemanticError::CodeGenError {
                                    message: e.to_string(),
                                })?;

                            let indices = vec![self.context.i32_type().const_int(4, false)];

                            let data_ptr = unsafe {
                                builder.build_gep(
                                    self.context.i8_type(),
                                    i8_ptr,
                                    &indices,
                                    &format!("{}_data_ptr", variant_name),
                                )
                            }
                            .map_err(|e| {
                                SemanticError::CodeGenError {
                                    message: e.to_string(),
                                }
                            })?;

                            // Store the first operand (assumes single associated value)
                            if let Some(operand) = operands.first() {
                                let value = self.generate_operand(
                                    operand,
                                    local_allocas,
                                    builder,
                                    function,
                                )?;

                                // Cast data pointer to appropriate type
                                let value_type_ptr = match value {
                                    BasicValueEnum::IntValue(_) => {
                                        self.context.i32_type().ptr_type(AddressSpace::default())
                                    }
                                    BasicValueEnum::FloatValue(_) => {
                                        self.context.f64_type().ptr_type(AddressSpace::default())
                                    }
                                    BasicValueEnum::PointerValue(_) => self
                                        .context
                                        .i8_type()
                                        .ptr_type(AddressSpace::default())
                                        .ptr_type(AddressSpace::default()),
                                    _ => self.context.i32_type().ptr_type(AddressSpace::default()),
                                };

                                let typed_data_ptr = builder
                                    .build_pointer_cast(
                                        data_ptr,
                                        value_type_ptr,
                                        &format!("{}_data_typed_ptr", variant_name),
                                    )
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;

                                builder.build_store(typed_data_ptr, value).map_err(|e| {
                                    SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    }
                                })?;
                            }
                        }

                        // Return pointer to enum
                        let enum_ptr = builder
                            .build_pointer_cast(
                                enum_alloca,
                                self.context.i8_type().ptr_type(AddressSpace::default()),
                                &format!("{}_ptr", enum_name),
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;

                        Ok(enum_ptr.into())
                    }
                }
            }

            mir::Rvalue::Ref {
                place,
                mutability: _,
            } => {
                // Address-of operation - return pointer to the place
                if let Some(&alloca) = local_allocas.get(&place.local) {
                    // Handle projections if any
                    if place.projection.is_empty() {
                        // No projections, just return the alloca pointer
                        Ok(alloca.into())
                    } else {
                        // Handle projections (field access, array index, etc.)
                        let mut current_ptr = alloca;
                        let mut current_type =
                            self.get_basic_type_from_local(place.local, function)?;

                        // Track the current struct type name for nested field access
                        let mut current_struct_type: Option<String> =
                            if let Some(local_def) = function.locals.get(&place.local) {
                                if let crate::types::Type::Named { name, .. } = &local_def.ty {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                        for proj in &place.projection {
                            match proj {
                                mir::PlaceElem::Deref => {
                                    // Load the pointer and use that as the new base
                                    current_ptr = builder
                                        .build_load(
                                            self.context
                                                .i8_type()
                                                .ptr_type(AddressSpace::default()),
                                            current_ptr,
                                            "deref_load",
                                        )
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?
                                        .into_pointer_value();
                                }
                                mir::PlaceElem::Field { field, ty } => {
                                    // For field access, calculate offset based on current struct type
                                    let offset =
                                        if let Some(ref struct_name) = current_struct_type {
                                            self.calculate_field_offset(struct_name, *field as usize)
                                        } else {
                                            (*field * 8) as u64 // Default fallback
                                        };

                                    let indices = vec![
                                        self.context.i32_type().const_int(0, false),
                                        self.context.i32_type().const_int(offset, false),
                                    ];

                                    current_ptr = unsafe {
                                        builder.build_in_bounds_gep(
                                            current_type,
                                            current_ptr,
                                            &indices,
                                            &format!("field_{}_ptr", field),
                                        )
                                    }
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;

                                    // Update current struct type for next projection
                                    // If the field is a Named type (struct), we need to:
                                    // 1. Load the pointer (struct fields are stored as pointers)
                                    // 2. Update current_struct_type for next field access
                                    if let crate::types::Type::Named { name, .. } = ty {
                                        // Load the pointer to get the actual struct address
                                        current_ptr = builder
                                            .build_load(
                                                self.context
                                                    .i8_type()
                                                    .ptr_type(AddressSpace::default()),
                                                current_ptr,
                                                "loaded_struct_ptr",
                                            )
                                            .map_err(|e| SemanticError::CodeGenError {
                                                message: e.to_string(),
                                            })?
                                            .into_pointer_value();
                                        current_struct_type = Some(name.clone());
                                    } else {
                                        current_struct_type = None;
                                    }
                                }
                                mir::PlaceElem::Index(index_local) => {
                                    // Array indexing
                                    let index_val = if let Some(&index_alloca) =
                                        local_allocas.get(index_local)
                                    {
                                        builder
                                            .build_load(
                                                self.context.i32_type(),
                                                index_alloca,
                                                "index_val",
                                            )
                                            .map_err(|e| SemanticError::CodeGenError {
                                                message: e.to_string(),
                                            })?
                                    } else {
                                        return Err(SemanticError::CodeGenError {
                                            message: format!(
                                                "Index local {} not found",
                                                index_local
                                            ),
                                        });
                                    };

                                    let indices = vec![
                                        self.context.i32_type().const_int(0, false),
                                        index_val.into_int_value(),
                                    ];

                                    current_ptr = unsafe {
                                        builder.build_in_bounds_gep(
                                            current_type,
                                            current_ptr,
                                            &indices,
                                            "array_elem_ptr",
                                        )
                                    }
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;
                                }
                                _ => {
                                    return Err(SemanticError::CodeGenError {
                                        message: "Unsupported place projection".to_string(),
                                    });
                                }
                            }
                        }

                        Ok(current_ptr.into())
                    }
                } else {
                    Err(SemanticError::CodeGenError {
                        message: format!("Local {:?} not found in allocas", place.local),
                    })
                }
            }

            mir::Rvalue::Len(_) => {
                // TODO: Implement array/slice length
                Err(SemanticError::CodeGenError {
                    message: "Array length operation not yet implemented".to_string(),
                })
            }

            mir::Rvalue::Discriminant(place) => {
                // Get the discriminant of an enum
                // First, get the type of the enum to determine discriminant size
                let enum_type_name = if let Some(local) = function.locals.get(&place.local) {
                    match &local.ty {
                        crate::types::Type::Named { name, .. } => name.clone(),
                        _ => {
                            return Err(SemanticError::CodeGenError {
                                message: format!("Discriminant operation on non-enum type"),
                            });
                        }
                    }
                } else {
                    return Err(SemanticError::CodeGenError {
                        message: format!("Local {:?} not found in function", place.local),
                    });
                };

                let discriminant_size = self.get_enum_discriminant_size(&enum_type_name);

                // Get the pointer to the enum
                let enum_ptr = if let Some(&alloca) = local_allocas.get(&place.local) {
                    // Load the enum pointer
                    let loaded_ptr = builder
                        .build_load(
                            self.context.i8_type().ptr_type(AddressSpace::default()),
                            alloca,
                            "loaded_enum_ptr",
                        )
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                    loaded_ptr.into_pointer_value()
                } else {
                    return Err(SemanticError::CodeGenError {
                        message: format!("Local {:?} not found in allocas", place.local),
                    });
                };

                // Determine the discriminant type and cast appropriately
                let (disc_type, disc_ptr) = match discriminant_size {
                    1 => {
                        let ptr = builder
                            .build_pointer_cast(
                                enum_ptr,
                                self.context.i8_type().ptr_type(AddressSpace::default()),
                                "discriminant_ptr_i8",
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        (self.context.i8_type(), ptr)
                    }
                    2 => {
                        let ptr = builder
                            .build_pointer_cast(
                                enum_ptr,
                                self.context.i16_type().ptr_type(AddressSpace::default()),
                                "discriminant_ptr_i16",
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        (self.context.i16_type(), ptr)
                    }
                    _ => {
                        let ptr = builder
                            .build_pointer_cast(
                                enum_ptr,
                                self.context.i32_type().ptr_type(AddressSpace::default()),
                                "discriminant_ptr_i32",
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        (self.context.i32_type(), ptr)
                    }
                };

                // Load discriminant value
                let disc_value = builder
                    .build_load(disc_type, disc_ptr, "discriminant")
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;

                // Zero-extend to i32 if necessary for consistency
                let disc_i32 = if discriminant_size < 4 {
                    builder
                        .build_int_z_extend(
                            disc_value.into_int_value(),
                            self.context.i32_type(),
                            "discriminant_i32",
                        )
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?
                        .into()
                } else {
                    disc_value
                };

                Ok(disc_i32)
            }
        }
    }

    /// Create or get a global string constant
    fn get_or_create_string_global(&mut self, string_value: &str) -> PointerValue<'ctx> {
        // Check if we already have this string
        if let Some(&global_ptr) = self.string_globals.get(string_value) {
            return global_ptr;
        }

        // Create a new global string constant
        let string_bytes = string_value.as_bytes();
        let string_type = self
            .context
            .i8_type()
            .array_type(string_bytes.len() as u32 + 1); // +1 for null terminator

        // Create string data with null terminator
        let mut string_data = string_bytes.to_vec();
        string_data.push(0); // null terminator

        let string_const = self.context.const_string(&string_data, false);

        // Generate unique name for this string global
        let global_name = format!(".str.{}", self.string_globals.len());

        // Create global variable
        let global =
            self.module
                .add_global(string_type, Some(AddressSpace::default()), &global_name);
        global.set_initializer(&string_const);
        global.set_constant(true);
        global.set_unnamed_addr(true); // Allow optimization

        // Get pointer to the global
        let global_ptr = global.as_pointer_value();

        // Cache for future use
        self.string_globals
            .insert(string_value.to_string(), global_ptr);

        global_ptr
    }

    /// Generate code for an operand
    fn generate_operand(
        &mut self,
        operand: &mir::Operand,
        local_allocas: &HashMap<mir::LocalId, PointerValue<'ctx>>,
        builder: &Builder<'ctx>,
        function: &mir::Function,
    ) -> Result<BasicValueEnum<'ctx>, SemanticError> {
        eprintln!("DEBUG: generate_operand called with: {:?}", operand);
        match operand {
            mir::Operand::Copy(place) | mir::Operand::Move(place) => {
                if let Some(&alloca) = local_allocas.get(&place.local) {
                    // Handle projections (field access)
                    let mut current_ptr = alloca;

                    // Track the current struct type for nested field access
                    let mut current_struct_type: Option<String> =
                        if let Some(local_def) = function.locals.get(&place.local) {
                            if let crate::types::Type::Named { name, .. } = &local_def.ty {
                                Some(name.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                    // First load the actual struct pointer if this is a pointer to a struct
                    let mut struct_ptr = if place.projection.is_empty() {
                        current_ptr
                    } else {
                        // Load the pointer value since structs are stored as pointers
                        let loaded_ptr = builder
                            .build_load(
                                self.context.i8_type().ptr_type(AddressSpace::default()),
                                current_ptr,
                                "loaded_struct_ptr",
                            )
                            .map_err(|e| SemanticError::CodeGenError {
                                message: e.to_string(),
                            })?;
                        loaded_ptr.into_pointer_value()
                    };

                    for projection in &place.projection {
                        match projection {
                            mir::PlaceElem::Deref => {
                                // Dereference - load the pointer value and use it as the new base
                                let loaded_ptr = builder
                                    .build_load(
                                        self.context.i8_type().ptr_type(AddressSpace::default()),
                                        current_ptr,
                                        "deref_ptr",
                                    )
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;
                                current_ptr = loaded_ptr.into_pointer_value();
                                struct_ptr = current_ptr;
                            }
                            mir::PlaceElem::Field { field, ty } => {
                                // Calculate field offset based on the current struct type
                                let field_offset =
                                    if let Some(ref struct_name) = current_struct_type {
                                        self.calculate_field_offset(struct_name, *field as usize)
                                    } else {
                                        (*field as u64) * 8 // Fallback
                                    };

                                // Calculate field pointer with byte offset
                                let i8_ptr = builder
                                    .build_pointer_cast(
                                        struct_ptr,
                                        self.context.i8_type().ptr_type(AddressSpace::default()),
                                        "struct_i8_ptr",
                                    )
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;

                                let indices =
                                    vec![self.context.i32_type().const_int(field_offset, false)];
                                let field_ptr = unsafe {
                                    builder.build_gep(
                                        self.context.i8_type(),
                                        i8_ptr,
                                        &indices,
                                        &format!("field_{}_ptr", field),
                                    )
                                }
                                .map_err(|e| {
                                    SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    }
                                })?;

                                // Cast to appropriate field type
                                let field_type = self.get_basic_type(ty);
                                let field_ptr_type = match field_type {
                                    inkwell::types::BasicTypeEnum::IntType(t) => {
                                        t.ptr_type(AddressSpace::default()).into()
                                    }
                                    inkwell::types::BasicTypeEnum::FloatType(t) => {
                                        t.ptr_type(AddressSpace::default()).into()
                                    }
                                    inkwell::types::BasicTypeEnum::PointerType(t) => {
                                        t.ptr_type(AddressSpace::default()).into()
                                    }
                                    inkwell::types::BasicTypeEnum::ArrayType(t) => {
                                        t.ptr_type(AddressSpace::default()).into()
                                    }
                                    inkwell::types::BasicTypeEnum::StructType(t) => {
                                        t.ptr_type(AddressSpace::default()).into()
                                    }
                                    inkwell::types::BasicTypeEnum::VectorType(t) => {
                                        t.ptr_type(AddressSpace::default()).into()
                                    }
                                };
                                current_ptr = builder
                                    .build_pointer_cast(
                                        field_ptr,
                                        field_ptr_type,
                                        &format!("field_{}_typed_ptr", field),
                                    )
                                    .map_err(|e| SemanticError::CodeGenError {
                                        message: e.to_string(),
                                    })?;

                                // Update for nested field access: if this field is a Named type,
                                // load the pointer and update the current struct type
                                if let crate::types::Type::Named { name, .. } = ty {
                                    // Load the pointer to the nested struct
                                    let loaded_nested = builder
                                        .build_load(
                                            self.context.i8_type().ptr_type(AddressSpace::default()),
                                            current_ptr,
                                            "loaded_nested_struct",
                                        )
                                        .map_err(|e| SemanticError::CodeGenError {
                                            message: e.to_string(),
                                        })?;
                                    struct_ptr = loaded_nested.into_pointer_value();
                                    current_struct_type = Some(name.clone());
                                } else {
                                    current_struct_type = None;
                                }
                            }
                            _ => {
                                return Err(SemanticError::CodeGenError {
                                    message: "Unsupported place projection".to_string(),
                                });
                            }
                        }
                    }

                    // Load the value from the final pointer
                    let local_type = if place.projection.is_empty() {
                        // No projections, get the type of the local
                        function
                            .locals
                            .get(&place.local)
                            .map(|local| self.get_basic_type(&local.ty))
                            .or_else(|| {
                                // Check if it's a parameter
                                function
                                    .parameters
                                    .iter()
                                    .find(|p| p.local_id == place.local)
                                    .map(|p| self.get_basic_type(&p.ty))
                            })
                            .unwrap_or_else(|| self.context.i32_type().into())
                    } else {
                        // With projections, use the type from the last projection
                        if let Some(mir::PlaceElem::Field { ty, .. }) = place.projection.last() {
                            self.get_basic_type(ty)
                        } else {
                            self.context.i32_type().into()
                        }
                    };

                    let value = builder
                        .build_load(local_type, current_ptr, &format!("local_{}", place.local))
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                    Ok(value)
                } else {
                    eprintln!(
                        "ERROR: Local {} not found. Available locals: {:?}",
                        place.local,
                        local_allocas.keys().collect::<Vec<_>>()
                    );
                    Err(SemanticError::CodeGenError {
                        message: format!("Local {} not found", place.local),
                    })
                }
            }

            mir::Operand::Constant(constant) => {
                match &constant.value {
                    mir::ConstantValue::Integer(val) => {
                        Ok(self.context.i32_type().const_int(*val as u64, false).into())
                    }
                    mir::ConstantValue::Float(val) => {
                        Ok(self.context.f64_type().const_float(*val).into())
                    }
                    mir::ConstantValue::Bool(val) => Ok(self
                        .context
                        .i32_type()
                        .const_int(if *val { 1 } else { 0 }, false)
                        .into()),
                    mir::ConstantValue::Char(val) => {
                        Ok(self.context.i8_type().const_int(*val as u64, false).into())
                    }
                    mir::ConstantValue::String(string_value) => {
                        // Create global string and return pointer to it
                        let global_ptr = self.get_or_create_string_global(string_value);

                        // Create GEP to get pointer to first character
                        let zero = self.context.i32_type().const_zero();
                        let string_ptr = unsafe {
                            builder
                                .build_gep(
                                    self.context
                                        .i8_type()
                                        .array_type(string_value.len() as u32 + 1),
                                    global_ptr,
                                    &[zero, zero],
                                    "str_ptr",
                                )
                                .map_err(|e| SemanticError::CodeGenError {
                                    message: e.to_string(),
                                })?
                        };

                        Ok(string_ptr.into())
                    }
                    mir::ConstantValue::Null => {
                        // Null constants need type information
                        Err(SemanticError::CodeGenError {
                            message: "Null constants not yet implemented".to_string(),
                        })
                    }
                }
            }
        }
    }

    /// Get the discriminant value for an enum variant
    fn get_enum_variant_discriminant(&self, enum_name: &str, variant_name: &str) -> u64 {
        // Look up the enum definition from type definitions
        if let Some(type_def) = self.type_definitions.get(enum_name) {
            if let crate::types::TypeDefinition::Enum { variants, .. } = type_def {
                // Find the variant and return its discriminant
                for variant in variants {
                    if variant.name == variant_name {
                        return variant.discriminant as u64;
                    }
                }
                eprintln!(
                    "WARNING: Variant {} not found in enum {}",
                    variant_name, enum_name
                );
            } else {
                eprintln!("WARNING: Type {} is not an enum", enum_name);
            }
        } else {
            eprintln!(
                "WARNING: Enum type {} not found in type definitions",
                enum_name
            );
        }

        // Fallback: use a simple hash
        variant_name
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_add(b as u64))
            % 256
    }

    /// Get the discriminant type size needed for an enum
    fn get_enum_discriminant_size(&self, enum_name: &str) -> u64 {
        if let Some(type_def) = self.type_definitions.get(enum_name) {
            if let crate::types::TypeDefinition::Enum { variants, .. } = type_def {
                // Find the maximum discriminant value
                let max_discriminant = variants.iter().map(|v| v.discriminant).max().unwrap_or(0);

                // Determine the minimum size needed
                if max_discriminant <= 255 {
                    1 // u8
                } else if max_discriminant <= 65535 {
                    2 // u16
                } else {
                    4 // u32
                }
            } else {
                4 // Default to 4 bytes
            }
        } else {
            4 // Default to 4 bytes
        }
    }

    /// Get the LLVM IR as a string
    pub fn get_ir_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Verify the generated module
    pub fn verify(&self) -> Result<(), String> {
        // Print IR before verification for debugging
        eprintln!(
            "LLVM IR before verification:\n{}",
            self.module.print_to_string().to_string()
        );
        self.module.verify().map_err(|e| e.to_string())
    }

    /// Write LLVM IR to a file
    pub fn write_ir_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.module.print_to_file(path).map_err(|e| e.to_string())
    }

    /// Write object file
    pub fn write_object_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        // Verify the module first (temporarily disabled to debug function signature issues)
        // self.module.verify().map_err(|e| format!("Module verification failed: {}", e))?;

        // Print IR for debugging
        eprintln!("LLVM IR:\n{}", self.module.print_to_string().to_string());

        let target_machine = self
            .target_machine
            .as_ref()
            .ok_or("Target machine not set")?;

        target_machine
            .write_to_file(&self.module, FileType::Object, path.as_ref())
            .map_err(|e| e.to_string())
    }

    /// Write assembly file
    pub fn write_assembly_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let target_machine = self
            .target_machine
            .as_ref()
            .ok_or("Target machine not set")?;

        target_machine
            .write_to_file(&self.module, FileType::Assembly, path.as_ref())
            .map_err(|e| e.to_string())
    }

    /// Get the module reference
    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Get the context reference
    pub fn context(&self) -> &'ctx Context {
        self.context
    }

    /// Get the target machine
    pub fn get_target_machine(&self) -> Option<&TargetMachine> {
        self.target_machine.as_ref()
    }

    /// Generate cleanup code for owned values in a block
    fn generate_cleanup_for_block(
        &mut self,
        cleanup_info: &CleanupInfo,
        block_id: &mir::BasicBlockId,
        local_allocas: &HashMap<mir::LocalId, PointerValue<'ctx>>,
        builder: &Builder<'ctx>,
    ) -> Result<(), SemanticError> {
        let locals_to_drop = cleanup_info.get_cleanup_locals(block_id);

        for local_info in locals_to_drop {
            if let Some(&alloca) = local_allocas.get(&local_info.local_id) {
                self.generate_drop_for_alloca_with_type(alloca, local_info.type_id, builder)?;
            }
        }

        Ok(())
    }

    /// Generate drop call for a specific place
    fn generate_drop_for_place(
        &mut self,
        place: &mir::Place,
        local_allocas: &HashMap<mir::LocalId, PointerValue<'ctx>>,
        builder: &Builder<'ctx>,
    ) -> Result<(), SemanticError> {
        if let Some(&alloca) = local_allocas.get(&place.local) {
            // For Drop terminators, we typically don't have the type info readily available
            // Use Unknown type ID for now
            self.generate_drop_for_alloca_with_type(alloca, TypeId::Unknown, builder)?;
        }
        Ok(())
    }

    /// Generate the actual drop/cleanup code for an allocation with known type
    fn generate_drop_for_alloca_with_type(
        &mut self,
        alloca: PointerValue<'ctx>,
        type_id: TypeId,
        builder: &Builder<'ctx>,
    ) -> Result<(), SemanticError> {
        // Load the pointer from the alloca
        let ptr_type = self.context.i8_type().ptr_type(AddressSpace::default());
        let value_ptr = builder
            .build_load(ptr_type, alloca, "cleanup_load")
            .map_err(|e| SemanticError::CodeGenError {
                message: e.to_string(),
            })?
            .into_pointer_value();

        // Check if the pointer is null before cleaning up
        let null_ptr = ptr_type.const_null();
        let is_null = builder
            .build_int_compare(inkwell::IntPredicate::EQ, value_ptr, null_ptr, "is_null")
            .map_err(|e| SemanticError::CodeGenError {
                message: e.to_string(),
            })?;

        // Create cleanup block and continuation block
        let current_fn = builder.get_insert_block().unwrap().get_parent().unwrap();
        let cleanup_block = self.context.append_basic_block(current_fn, "cleanup");
        let continue_block = self
            .context
            .append_basic_block(current_fn, "cleanup_continue");

        // Branch based on null check
        builder
            .build_conditional_branch(is_null, continue_block, cleanup_block)
            .map_err(|e| SemanticError::CodeGenError {
                message: e.to_string(),
            })?;

        // Generate cleanup in cleanup block
        builder.position_at_end(cleanup_block);

        match type_id {
            TypeId::String => {
                // Call string-specific cleanup
                let drop_fn = self
                    .module
                    .get_function("aether_drop_string")
                    .unwrap_or_else(|| {
                        let drop_type = self.context.void_type().fn_type(
                            &[self
                                .context
                                .i8_type()
                                .ptr_type(AddressSpace::default())
                                .into()],
                            false,
                        );
                        self.module
                            .add_function("aether_drop_string", drop_type, None)
                    });
                builder
                    .build_call(drop_fn, &[value_ptr.into()], "drop_string")
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
            }
            TypeId::Array => {
                // Call array-specific cleanup
                let drop_fn = self
                    .module
                    .get_function("aether_drop_array")
                    .unwrap_or_else(|| {
                        let drop_type = self.context.void_type().fn_type(
                            &[self
                                .context
                                .i8_type()
                                .ptr_type(AddressSpace::default())
                                .into()],
                            false,
                        );
                        self.module
                            .add_function("aether_drop_array", drop_type, None)
                    });
                builder
                    .build_call(drop_fn, &[value_ptr.into()], "drop_array")
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
            }
            TypeId::Map => {
                // Call map-specific cleanup
                let drop_fn = self
                    .module
                    .get_function("aether_drop_map")
                    .unwrap_or_else(|| {
                        let drop_type = self.context.void_type().fn_type(
                            &[self
                                .context
                                .i8_type()
                                .ptr_type(AddressSpace::default())
                                .into()],
                            false,
                        );
                        self.module.add_function("aether_drop_map", drop_type, None)
                    });
                builder
                    .build_call(drop_fn, &[value_ptr.into()], "drop_map")
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
            }
            TypeId::Unknown => {
                // Call generic cleanup - would need runtime type info in production
                let drop_fn = self
                    .module
                    .get_function("aether_drop_value")
                    .unwrap_or_else(|| {
                        let drop_type = self.context.void_type().fn_type(
                            &[
                                self.context
                                    .i8_type()
                                    .ptr_type(AddressSpace::default())
                                    .into(),
                                self.context.i32_type().into(),
                            ],
                            false,
                        );
                        self.module
                            .add_function("aether_drop_value", drop_type, None)
                    });
                let type_id_val = self.context.i32_type().const_int(type_id as u64, false);
                builder
                    .build_call(
                        drop_fn,
                        &[value_ptr.into(), type_id_val.into()],
                        "drop_generic",
                    )
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
            }
        }

        // Jump to continuation
        builder
            .build_unconditional_branch(continue_block)
            .map_err(|e| SemanticError::CodeGenError {
                message: e.to_string(),
            })?;

        // Continue from here
        builder.position_at_end(continue_block);

        Ok(())
    }

    /// Generate main wrapper function
    pub fn generate_main_wrapper(&mut self) -> Result<(), SemanticError> {
        // Create main function type: i32 main(i32 argc, char** argv)
        let i32_type = self.context.i32_type();
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(AddressSpace::default());
        let i8_ptr_ptr_type = i8_ptr_type.ptr_type(AddressSpace::default());

        let main_type = i32_type.fn_type(&[i32_type.into(), i8_ptr_ptr_type.into()], false);

        let main_func = self.module.add_function("main", main_type, None);
        let entry = self.context.append_basic_block(main_func, "entry");

        let builder = self.context.create_builder();
        builder.position_at_end(entry);

        // For now, just return 0
        let zero = i32_type.const_int(0, false);
        let _ = builder.build_return(Some(&zero));

        Ok(())
    }

    /// Declare built-in functions like printf
    fn declare_builtin_functions(
        &self,
        function_declarations: &mut HashMap<String, FunctionValue<'ctx>>,
    ) -> Result<(), SemanticError> {
        // Don't declare printf here - it should come from external functions
        // The external function declaration will handle it properly

        // Declare runtime init function
        let void_type = self.context.void_type();
        let init_fn_type = void_type.fn_type(&[], false);
        let init_fn = self
            .module
            .add_function("aether_runtime_init", init_fn_type, None);
        function_declarations.insert("aether_runtime_init".to_string(), init_fn);

        let i32_type = self.context.i32_type();
        let i64_type = self.context.i64_type();
        let i8_type = self.context.i8_type();
        let i8_ptr_type = i8_type.ptr_type(AddressSpace::default());
        let void_type = self.context.void_type();

        // Array runtime functions
        // array_create: creates an array with given count
        // array_create(int count) -> void*
        let array_create_type = i8_ptr_type.fn_type(&[i32_type.into()], false);
        let array_create_fn = self
            .module
            .add_function("array_create", array_create_type, None);
        function_declarations.insert("array_create".to_string(), array_create_fn);

        // array_get: gets an element from an array
        // array_get(void* array, int index) -> int (for now, assume int arrays)
        let array_get_type = i32_type.fn_type(&[i8_ptr_type.into(), i32_type.into()], false);
        let array_get_fn = self.module.add_function("array_get", array_get_type, None);
        function_declarations.insert("array_get".to_string(), array_get_fn);

        // array_length: gets the length of an array
        // array_length(void* array) -> int
        let array_length_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
        let array_length_fn = self
            .module
            .add_function("array_length", array_length_type, None);
        function_declarations.insert("array_length".to_string(), array_length_fn);

        // array_set: sets an element in an array
        // array_set(void* array, int index, int value) -> void
        let void_type = self.context.void_type();
        let array_set_type = void_type.fn_type(
            &[i8_ptr_type.into(), i32_type.into(), i32_type.into()],
            false,
        );
        let array_set_fn = self.module.add_function("array_set", array_set_type, None);
        function_declarations.insert("array_set".to_string(), array_set_fn);

        // String runtime functions
        // string_concat: concatenates two strings
        // string_concat(char* str1, char* str2) -> char*
        let string_concat_type =
            i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let string_concat_fn = self
            .module
            .add_function("string_concat", string_concat_type, None);
        function_declarations.insert("string_concat".to_string(), string_concat_fn);

        // string_length: gets the length of a string
        // string_length(char* str) -> int
        let string_length_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
        let string_length_fn = self
            .module
            .add_function("string_length", string_length_type, None);
        function_declarations.insert("string_length".to_string(), string_length_fn);

        // string_char_at: gets character at index in string
        // string_char_at(char* str, int index) -> char
        let string_char_at_type = i8_type.fn_type(&[i8_ptr_type.into(), i32_type.into()], false);
        let string_char_at_fn =
            self.module
                .add_function("string_char_at", string_char_at_type, None);
        function_declarations.insert("string_char_at".to_string(), string_char_at_fn);

        // string_equals: checks if two strings are equal
        // string_equals(char* str1, char* str2) -> int
        let string_equals_type = i32_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let string_equals_fn = self
            .module
            .add_function("string_equals", string_equals_type, None);
        function_declarations.insert("string_equals".to_string(), string_equals_fn);

        // string_contains: checks if string contains substring
        // string_contains(char* haystack, char* needle) -> int
        let string_contains_type =
            i32_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let string_contains_fn =
            self.module
                .add_function("string_contains", string_contains_type, None);
        function_declarations.insert("string_contains".to_string(), string_contains_fn);

        // string_to_upper: converts string to uppercase
        // string_to_upper(char* str) -> char*
        let string_to_upper_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], false);
        let string_to_upper_fn =
            self.module
                .add_function("string_to_upper", string_to_upper_type, None);
        function_declarations.insert("string_to_upper".to_string(), string_to_upper_fn);

        // string_to_lower: converts string to lowercase
        // string_to_lower(char* str) -> char*
        let string_to_lower_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], false);
        let string_to_lower_fn =
            self.module
                .add_function("string_to_lower", string_to_lower_type, None);
        function_declarations.insert("string_to_lower".to_string(), string_to_lower_fn);

        // int_to_string: converts integer to string
        // int_to_string(int value) -> char*
        let int_to_string_type = i8_ptr_type.fn_type(&[i32_type.into()], false);
        let int_to_string_fn = self
            .module
            .add_function("int_to_string", int_to_string_type, None);
        function_declarations.insert("int_to_string".to_string(), int_to_string_fn);

        // puts: prints a string to stdout
        // puts(char* str) -> int
        let puts_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
        let puts_fn = self.module.add_function("puts", puts_type, None);
        function_declarations.insert("puts".to_string(), puts_fn);

        // string_to_int: converts string to integer
        // string_to_int(char* str) -> int
        let string_to_int_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
        let string_to_int_fn = self
            .module
            .add_function("string_to_int", string_to_int_type, None);
        function_declarations.insert("string_to_int".to_string(), string_to_int_fn);

        // Network functions
        // tcp_listen(int port) -> int
        let tcp_listen_type = i32_type.fn_type(&[i32_type.into()], false);
        let tcp_listen_fn = self
            .module
            .add_function("tcp_listen", tcp_listen_type, None);
        function_declarations.insert("tcp_listen".to_string(), tcp_listen_fn);

        // tcp_accept(int listener_id) -> int
        let tcp_accept_type = i32_type.fn_type(&[i32_type.into()], false);
        let tcp_accept_fn = self
            .module
            .add_function("tcp_accept", tcp_accept_type, None);
        function_declarations.insert("tcp_accept".to_string(), tcp_accept_fn);

        // tcp_connect(char* host, int port) -> int
        let tcp_connect_type = i32_type.fn_type(&[i8_ptr_type.into(), i32_type.into()], false);
        let tcp_connect_fn = self
            .module
            .add_function("tcp_connect", tcp_connect_type, None);
        function_declarations.insert("tcp_connect".to_string(), tcp_connect_fn);

        // tcp_read(int socket_id, char* buffer, int buffer_size) -> int
        let tcp_read_type = i32_type.fn_type(
            &[i32_type.into(), i8_ptr_type.into(), i32_type.into()],
            false,
        );
        let tcp_read_fn = self.module.add_function("tcp_read", tcp_read_type, None);
        function_declarations.insert("tcp_read".to_string(), tcp_read_fn);

        // tcp_write(int socket_id, char* data, int data_size) -> int
        let tcp_write_type = i32_type.fn_type(
            &[i32_type.into(), i8_ptr_type.into(), i32_type.into()],
            false,
        );
        let tcp_write_fn = self.module.add_function("tcp_write", tcp_write_type, None);
        function_declarations.insert("tcp_write".to_string(), tcp_write_fn);

        // tcp_close(int socket_id) -> void
        let tcp_close_type = void_type.fn_type(&[i32_type.into()], false);
        let tcp_close_fn = self.module.add_function("tcp_close", tcp_close_type, None);
        function_declarations.insert("tcp_close".to_string(), tcp_close_fn);

        // Memory management functions
        // aether_malloc(int64 size) -> int64
        let malloc_type = i64_type.fn_type(&[i64_type.into()], false);
        let malloc_fn = self.module.add_function("aether_malloc", malloc_type, None);
        function_declarations.insert("aether_malloc".to_string(), malloc_fn);

        // aether_free(int64 ptr) -> void
        let free_type = void_type.fn_type(&[i64_type.into()], false);
        let free_fn = self.module.add_function("aether_free", free_type, None);
        function_declarations.insert("aether_free".to_string(), free_fn);

        // aether_realloc(int64 ptr, int64 new_size) -> int64
        let realloc_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
        let realloc_fn = self
            .module
            .add_function("aether_realloc", realloc_type, None);
        function_declarations.insert("aether_realloc".to_string(), realloc_fn);

        // aether_gc_add_root(int64 ptr) -> void
        let gc_add_root_type = void_type.fn_type(&[i64_type.into()], false);
        let gc_add_root_fn = self
            .module
            .add_function("aether_gc_add_root", gc_add_root_type, None);
        function_declarations.insert("aether_gc_add_root".to_string(), gc_add_root_fn);

        // aether_gc_remove_root(int64 ptr) -> void
        let gc_remove_root_type = void_type.fn_type(&[i64_type.into()], false);
        let gc_remove_root_fn =
            self.module
                .add_function("aether_gc_remove_root", gc_remove_root_type, None);
        function_declarations.insert("aether_gc_remove_root".to_string(), gc_remove_root_fn);

        // aether_gc_collect() -> void
        let gc_collect_type = void_type.fn_type(&[], false);
        let gc_collect_fn = self
            .module
            .add_function("aether_gc_collect", gc_collect_type, None);
        function_declarations.insert("aether_gc_collect".to_string(), gc_collect_fn);

        // aether_memory_used() -> int
        let memory_used_type = i32_type.fn_type(&[], false);
        let memory_used_fn = self
            .module
            .add_function("aether_memory_used", memory_used_type, None);
        function_declarations.insert("aether_memory_used".to_string(), memory_used_fn);

        // aether_allocation_count() -> int
        let allocation_count_type = i32_type.fn_type(&[], false);
        let allocation_count_fn =
            self.module
                .add_function("aether_allocation_count", allocation_count_type, None);
        function_declarations.insert("aether_allocation_count".to_string(), allocation_count_fn);

        // HTTP and networking function aliases
        // tcp_server(char* host, int port) -> int
        let tcp_server_type = i32_type.fn_type(&[i8_ptr_type.into(), i32_type.into()], false);
        let tcp_server_fn = self
            .module
            .add_function("tcp_server", tcp_server_type, None);
        function_declarations.insert("tcp_server".to_string(), tcp_server_fn);

        // socket_accept(int server_fd) -> int
        let socket_accept_type = i32_type.fn_type(&[i32_type.into()], false);
        let socket_accept_fn = self
            .module
            .add_function("socket_accept", socket_accept_type, None);
        function_declarations.insert("socket_accept".to_string(), socket_accept_fn);

        // socket_receive(int socket_fd, int buffer_size) -> char*
        let socket_receive_type = i8_ptr_type.fn_type(&[i32_type.into(), i32_type.into()], false);
        let socket_receive_fn =
            self.module
                .add_function("socket_receive", socket_receive_type, None);
        function_declarations.insert("socket_receive".to_string(), socket_receive_fn);

        // socket_send(int socket_fd, char* data) -> int
        let socket_send_type = i32_type.fn_type(&[i32_type.into(), i8_ptr_type.into()], false);
        let socket_send_fn = self
            .module
            .add_function("socket_send", socket_send_type, None);
        function_declarations.insert("socket_send".to_string(), socket_send_fn);

        // socket_close(int socket_fd) -> void
        let socket_close_type = void_type.fn_type(&[i32_type.into()], false);
        let socket_close_fn = self
            .module
            .add_function("socket_close", socket_close_type, None);
        function_declarations.insert("socket_close".to_string(), socket_close_fn);

        // HTTP parsing functions
        // parse_request(char* request_data) -> char*
        let parse_request_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], false);
        let parse_request_fn = self
            .module
            .add_function("parse_request", parse_request_type, None);
        function_declarations.insert("parse_request".to_string(), parse_request_fn);

        // is_get(char* method) -> int
        let is_get_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
        let is_get_fn = self.module.add_function("is_get", is_get_type, None);
        function_declarations.insert("is_get".to_string(), is_get_fn);

        // create_response(int status_code, char* body) -> char*
        let create_response_type =
            i8_ptr_type.fn_type(&[i32_type.into(), i8_ptr_type.into()], false);
        let create_response_fn =
            self.module
                .add_function("create_response", create_response_type, None);
        function_declarations.insert("create_response".to_string(), create_response_fn);

        // json_response(char* json_body, int status_code) -> char*
        let json_response_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i32_type.into()], false);
        let json_response_fn = self
            .module
            .add_function("json_response", json_response_type, None);
        function_declarations.insert("json_response".to_string(), json_response_fn);

        // JSON manipulation functions
        // create_object() -> char*
        let create_object_type = i8_ptr_type.fn_type(&[], false);
        let create_object_fn = self
            .module
            .add_function("create_object", create_object_type, None);
        function_declarations.insert("create_object".to_string(), create_object_fn);

        // create_array() -> char*
        let create_json_array_type = i8_ptr_type.fn_type(&[], false);
        let create_json_array_fn =
            self.module
                .add_function("create_array", create_json_array_type, None);
        function_declarations.insert("create_array".to_string(), create_json_array_fn);

        // json_set_field(char* json_obj, char* field, char* value) -> char*
        let json_set_field_type = i8_ptr_type.fn_type(
            &[i8_ptr_type.into(), i8_ptr_type.into(), i8_ptr_type.into()],
            false,
        );
        let json_set_field_fn =
            self.module
                .add_function("json_set_field", json_set_field_type, None);
        function_declarations.insert("json_set_field".to_string(), json_set_field_fn);

        // stringify_json(char* json) -> char*
        let stringify_json_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], false);
        let stringify_json_fn =
            self.module
                .add_function("stringify_json", stringify_json_type, None);
        function_declarations.insert("stringify_json".to_string(), stringify_json_fn);

        // json_array_push(char* json_array, char* item) -> char*
        let json_array_push_type =
            i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let json_array_push_fn =
            self.module
                .add_function("json_array_push", json_array_push_type, None);
        function_declarations.insert("json_array_push".to_string(), json_array_push_fn);

        // json_array_length(char* json_array) -> int
        let json_array_length_type = i32_type.fn_type(&[i8_ptr_type.into()], false);
        let json_array_length_fn =
            self.module
                .add_function("json_array_length", json_array_length_type, None);
        function_declarations.insert("json_array_length".to_string(), json_array_length_fn);

        // from_string(char* s) -> char*
        let from_string_type = i8_ptr_type.fn_type(&[i8_ptr_type.into()], false);
        let from_string_fn = self
            .module
            .add_function("from_string", from_string_type, None);
        function_declarations.insert("from_string".to_string(), from_string_fn);

        // from_integer(int n) -> char*
        let from_integer_type = i8_ptr_type.fn_type(&[i32_type.into()], false);
        let from_integer_fn = self
            .module
            .add_function("from_integer", from_integer_type, None);
        function_declarations.insert("from_integer".to_string(), from_integer_fn);

        // Map runtime functions
        // map_new: creates a new empty map
        // map_new() -> void*
        let map_new_type = i8_ptr_type.fn_type(&[], false);
        let map_new_fn = self.module.add_function("map_new", map_new_type, None);
        function_declarations.insert("map_new".to_string(), map_new_fn);

        // map_insert: inserts a key-value pair into the map
        // map_insert(void* map, void* key, void* value) -> void
        let map_insert_type = void_type.fn_type(
            &[i8_ptr_type.into(), i8_ptr_type.into(), i8_ptr_type.into()],
            false,
        );
        let map_insert_fn = self
            .module
            .add_function("map_insert", map_insert_type, None);
        function_declarations.insert("map_insert".to_string(), map_insert_fn);

        // map_get: gets a value from the map
        // map_get(void* map, void* key) -> void*
        let map_get_type = i8_ptr_type.fn_type(&[i8_ptr_type.into(), i8_ptr_type.into()], false);
        let map_get_fn = self.module.add_function("map_get", map_get_type, None);
        function_declarations.insert("map_get".to_string(), map_get_fn);

        Ok(())
    }
}

/// Target architecture enumeration
#[derive(Debug, Clone, Copy)]
pub enum TargetArch {
    X86_64,
    AArch64,
    X86,
}

impl TargetArch {
    /// Get the LLVM target triple for this architecture
    pub fn target_triple(&self) -> &'static str {
        #[cfg(target_os = "macos")]
        match self {
            TargetArch::X86_64 => "x86_64-apple-darwin",
            TargetArch::AArch64 => "aarch64-apple-darwin",
            TargetArch::X86 => "i386-apple-darwin",
        }

        #[cfg(target_os = "linux")]
        match self {
            TargetArch::X86_64 => "x86_64-unknown-linux-gnu",
            TargetArch::AArch64 => "aarch64-unknown-linux-gnu",
            TargetArch::X86 => "i386-unknown-linux-gnu",
        }

        #[cfg(target_os = "windows")]
        match self {
            TargetArch::X86_64 => "x86_64-pc-windows-msvc",
            TargetArch::AArch64 => "aarch64-pc-windows-msvc",
            TargetArch::X86 => "i686-pc-windows-msvc",
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        "x86_64-unknown-linux-gnu"
    }

    /// Get the native target architecture
    pub fn native() -> Self {
        #[cfg(target_arch = "x86_64")]
        return TargetArch::X86_64;

        #[cfg(target_arch = "aarch64")]
        return TargetArch::AArch64;

        #[cfg(target_arch = "x86")]
        return TargetArch::X86;

        // Default fallback
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "x86")))]
        return TargetArch::X86_64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::Program;
    use std::collections::HashMap;

    #[test]
    fn test_llvm_backend_creation() {
        let context = Context::create();
        let backend = LLVMBackend::new(&context, "test_module");

        assert_eq!(backend.module().get_name().to_str().unwrap(), "test_module");
    }

    #[test]
    fn test_target_arch_native() {
        let arch = TargetArch::native();
        let triple = arch.target_triple();

        // Should be a valid target triple format
        assert!(triple.contains("-"));
    }

    #[test]
    fn test_llvm_backend_empty_program() {
        LLVMBackend::initialize_targets();

        let context = Context::create();
        let mut backend = LLVMBackend::new(&context, "empty_test");

        let program = Program {
            functions: HashMap::new(),
            global_constants: HashMap::new(),
            external_functions: HashMap::new(),
            type_definitions: HashMap::new(),
        };

        // Should be able to generate IR for empty program
        assert!(backend.generate_ir(&program).is_ok());

        // Should be able to verify empty module
        assert!(backend.verify().is_ok());
    }

    #[test]
    fn test_target_triple_setting() {
        LLVMBackend::initialize_targets();

        let context = Context::create();
        let mut backend = LLVMBackend::new(&context, "target_test");

        let arch = TargetArch::native();
        let result = backend.set_target_triple(arch.target_triple());

        assert!(result.is_ok());
    }
}
