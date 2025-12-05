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

//! LLVM IR code generation from MIR

use super::types::TypeConverter;
use super::values::ValueConverter;
use crate::error::SemanticError;
use crate::mir::{
    BasicBlock as MirBasicBlock, BasicBlockId, BinOp, CallingConvention as MirCallingConvention,
    Constant as MirConstant, ExternalFunction, Function as MirFunction, LocalId, Operand, Place,
    Rvalue, Statement, Terminator, UnOp,
};
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};
use std::collections::HashMap;

/// LLVM IR code generator
pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: &'ctx Module<'ctx>,
    builder: Builder<'ctx>,
    type_converter: TypeConverter<'ctx>,
    value_converter: ValueConverter<'ctx>,

    // Current function state
    current_function: Option<FunctionValue<'ctx>>,
    local_values: HashMap<LocalId, PointerValue<'ctx>>,
    local_types: HashMap<LocalId, BasicTypeEnum<'ctx>>,
    basic_blocks: HashMap<BasicBlockId, BasicBlock<'ctx>>,
    return_local: Option<LocalId>,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// Generate a complete program
    pub fn generate_program(
        context: &'ctx Context,
        module: &'ctx Module<'ctx>,
        program: &crate::mir::Program,
    ) -> Result<(), SemanticError> {
        let mut generator = Self::new(context, module);

        // Pass type definitions to the type converter for alias resolution
        generator.type_converter.set_type_definitions(program.type_definitions.clone());

        // Declare the runtime initialization function
        generator.declare_runtime_init_function();

        // Generate global constants
        for (name, constant) in &program.global_constants {
            generator.generate_global_constant(name, constant)?;
        }

        // Generate external function declarations
        for (name, ext_func) in &program.external_functions {
            generator.generate_external_function(name, ext_func)?;
        }

        // Generate functions
        for (name, function) in &program.functions {
            generator.generate_function(name, function)?;
        }

        Ok(())
    }

    /// Create a new code generator
    fn new(context: &'ctx Context, module: &'ctx Module<'ctx>) -> Self {
        let builder = context.create_builder();
        let type_converter = TypeConverter::new(context);
        let value_converter = ValueConverter::new(context);

        Self {
            context,
            module,
            builder,
            type_converter,
            value_converter,
            current_function: None,
            local_values: HashMap::new(),
            local_types: HashMap::new(),
            basic_blocks: HashMap::new(),
            return_local: None,
        }
    }

    /// Declare the runtime initialization function
    fn declare_runtime_init_function(&self) {
        let void_type = self.context.void_type();
        let init_fn_type = void_type.fn_type(&[], false);
        self.module
            .add_function("aether_runtime_init", init_fn_type, None);
    }

    /// Generate a global constant
    pub fn generate_global_constant(
        &mut self,
        name: &str,
        constant: &MirConstant,
    ) -> Result<(), SemanticError> {
        let llvm_type = self.type_converter.convert_type(&constant.ty)?;
        let llvm_value = self
            .value_converter
            .convert_constant_value(&constant.value)?;

        let global = self
            .module
            .add_global(llvm_type, Some(AddressSpace::default()), name);
        global.set_initializer(&llvm_value);
        global.set_constant(true);

        Ok(())
    }

    /// Generate an external function declaration
    pub fn generate_external_function(
        &mut self,
        name: &str,
        ext_func: &ExternalFunction,
    ) -> Result<(), SemanticError> {
        let param_types: Result<Vec<_>, _> = ext_func
            .parameters
            .iter()
            .map(|param_type| self.type_converter.convert_type(param_type))
            .collect();
        let param_types = param_types?;

        // Handle return type
        let param_meta_types: Vec<BasicMetadataTypeEnum> =
            param_types.into_iter().map(|t| t.into()).collect();

        // Create function type
        let fn_type = match &ext_func.return_type {
            crate::types::Type::Primitive(crate::ast::PrimitiveType::Void) => {
                self.context.void_type().fn_type(&param_meta_types, false)
            }
            _ => {
                // Convert the return type
                let return_type = self.type_converter.convert_type(&ext_func.return_type)?;
                return_type.fn_type(&param_meta_types, false)
            }
        };

        // Use symbol name if available, otherwise use the function name
        let extern_symbol = ext_func.symbol.as_ref().unwrap_or(&ext_func.name);
        let _function = self.module.add_function(extern_symbol, fn_type, None);

        // Set calling convention
        match ext_func.calling_convention {
            MirCallingConvention::C => {
                // Default is already C calling convention
            }
            MirCallingConvention::System => {
                // Platform-specific system calling convention
                #[cfg(target_os = "windows")]
                _function.set_call_conventions(64); // Win64 calling convention
            }
            MirCallingConvention::Rust => {
                // Use default (C) calling convention for now
            }
        }

        Ok(())
    }

    /// Generate a function
    pub fn generate_function(
        &mut self,
        name: &str,
        mir_function: &MirFunction,
    ) -> Result<(), SemanticError> {
        // Convert parameter types
        let param_types: Result<Vec<_>, _> = mir_function
            .parameters
            .iter()
            .map(|param| self.type_converter.convert_type(&param.ty))
            .collect();
        let param_types = param_types?;

        // Create function type
        let param_meta_types: Vec<BasicMetadataTypeEnum> =
            param_types.iter().map(|t| (*t).into()).collect();

        // Create function type
        let fn_type = match &mir_function.return_type {
            crate::types::Type::Primitive(crate::ast::PrimitiveType::Void) => {
                self.context.void_type().fn_type(&param_meta_types, false)
            }
            _ => {
                // Convert the return type
                let return_type = self
                    .type_converter
                    .convert_type(&mir_function.return_type)?;
                return_type.fn_type(&param_meta_types, false)
            }
        };

        // Add function to module
        let function = self.module.add_function(name, fn_type, None);
        self.current_function = Some(function);
        self.return_local = mir_function.return_local;

        // Create basic blocks
        self.basic_blocks.clear();
        for block_id in mir_function.basic_blocks.keys() {
            let bb_name = format!("bb{}", block_id);
            let basic_block = self.context.append_basic_block(function, &bb_name);
            self.basic_blocks.insert(*block_id, basic_block);
        }

        // Create entry block for local variable allocations
        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        // If this is the main function, call runtime init
        if name == "main" {
            if let Some(init_fn) = self.module.get_function("aether_runtime_init") {
                self.builder
                    .build_call(init_fn, &[], "call_runtime_init")
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
            }
        }

        // Allocate space for parameters
        self.local_values.clear();
        self.local_types.clear();
        for (i, param) in mir_function.parameters.iter().enumerate() {
            let param_type = self.type_converter.convert_type(&param.ty)?;
            let alloca = self
                .builder
                .build_alloca(param_type, &param.name)
                .map_err(|e| SemanticError::CodeGenError {
                    message: e.to_string(),
                })?;

            // Store parameter value
            let param_value = function
                .get_nth_param(i as u32)
                .expect("LLVM function parameter count should match MIR function parameters");
            self.builder.build_store(alloca, param_value).map_err(|e| {
                SemanticError::CodeGenError {
                    message: e.to_string(),
                }
            })?;

            self.local_values.insert(param.local_id, alloca);
            self.local_types.insert(param.local_id, param_type);
        }

        // Allocate space for other locals
        for (local_id, local) in &mir_function.locals {
            // Skip parameters (already allocated)
            if self.local_values.contains_key(local_id) {
                continue;
            }

            let local_type = self.type_converter.convert_type(&local.ty)?;
            let alloca_name = format!("local_{}", local_id);
            let alloca = self
                .builder
                .build_alloca(local_type, &alloca_name)
                .map_err(|e| SemanticError::CodeGenError {
                    message: e.to_string(),
                })?;

            self.local_values.insert(*local_id, alloca);
            self.local_types.insert(*local_id, local_type);
        }

        // Jump to the entry block of the MIR function
        let mir_entry_block = self.basic_blocks[&mir_function.entry_block];
        self.builder
            .build_unconditional_branch(mir_entry_block)
            .map_err(|e| SemanticError::CodeGenError {
                message: e.to_string(),
            })?;

        // Generate code for each basic block
        for (block_id, mir_block) in &mir_function.basic_blocks {
            self.generate_basic_block(*block_id, mir_block)?;
        }

        Ok(())
    }

    /// Generate code for a basic block
    fn generate_basic_block(
        &mut self,
        block_id: BasicBlockId,
        mir_block: &MirBasicBlock,
    ) -> Result<(), SemanticError> {
        let llvm_block = self.basic_blocks[&block_id];
        self.builder.position_at_end(llvm_block);

        // Generate statements
        for statement in &mir_block.statements {
            self.generate_statement(statement)?;
        }

        // Generate terminator
        self.generate_terminator(&mir_block.terminator)?;

        Ok(())
    }

    /// Generate code for a statement
    fn generate_statement(&mut self, statement: &Statement) -> Result<(), SemanticError> {
        match statement {
            Statement::Assign { place, rvalue, .. } => {
                let value = self.generate_rvalue(rvalue)?;
                let place_ptr = self.generate_place(place)?;

                self.builder.build_store(place_ptr, value).map_err(|e| {
                    SemanticError::CodeGenError {
                        message: e.to_string(),
                    }
                })?;
            }

            Statement::StorageLive(_) | Statement::StorageDead(_) => {
                // These are handled by LLVM's memory management
            }

            Statement::Nop => {
                // No operation
            }
        }

        Ok(())
    }

    /// Generate code for an rvalue
    fn generate_rvalue(&mut self, rvalue: &Rvalue) -> Result<BasicValueEnum<'ctx>, SemanticError> {
        match rvalue {
            Rvalue::Use(operand) => self.generate_operand(operand),

            Rvalue::BinaryOp { op, left, right } => {
                let left_val = self.generate_operand(left)?;
                let right_val = self.generate_operand(right)?;
                self.generate_binary_op(*op, left_val, right_val)
            }

            Rvalue::UnaryOp { op, operand } => {
                let operand_val = self.generate_operand(operand)?;
                self.generate_unary_op(*op, operand_val)
            }

            _ => Err(SemanticError::UnsupportedFeature {
                feature: "Rvalue type not yet implemented in LLVM backend".to_string(),
                location: crate::error::SourceLocation::unknown(),
            }),
        }
    }

    /// Generate code for an operand
    fn generate_operand(
        &mut self,
        operand: &Operand,
    ) -> Result<BasicValueEnum<'ctx>, SemanticError> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                let place_ptr = self.generate_place(place)?;

                // Get the type of the local
                let place_type = self.local_types.get(&place.local).copied().ok_or_else(|| {
                    SemanticError::UndefinedSymbol {
                        symbol: format!("local_{}", place.local),
                        location: crate::error::SourceLocation::unknown(),
                    }
                })?;

                self.builder
                    .build_load(place_type, place_ptr, "load")
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })
            }

            Operand::Constant(constant) => {
                if let crate::mir::ConstantValue::Function(name) = &constant.value {
                    let func = self.module.get_function(name).ok_or_else(|| {
                        SemanticError::UndefinedSymbol {
                            symbol: name.clone(),
                            location: crate::error::SourceLocation::unknown(),
                        }
                    })?;
                    Ok(BasicValueEnum::PointerValue(
                        func.as_global_value().as_pointer_value(),
                    ))
                } else {
                    self.value_converter
                        .convert_constant_value(&constant.value)
                }
            }
        }
    }

    /// Generate code for a place (returns pointer to the place)
    fn generate_place(&mut self, place: &Place) -> Result<PointerValue<'ctx>, SemanticError> {
        // For now, only handle simple local variables
        if !place.projection.is_empty() {
            return Err(SemanticError::UnsupportedFeature {
                feature: "Place projections not yet implemented".to_string(),
                location: crate::error::SourceLocation::unknown(),
            });
        }

        self.local_values
            .get(&place.local)
            .copied()
            .ok_or_else(|| SemanticError::UndefinedSymbol {
                symbol: format!("local_{}", place.local),
                location: crate::error::SourceLocation::unknown(),
            })
    }

    /// Generate code for a binary operation
    fn generate_binary_op(
        &mut self,
        op: BinOp,
        left: BasicValueEnum<'ctx>,
        right: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, SemanticError> {
        match (left, right) {
            (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                let result = match op {
                    BinOp::Add => self.builder.build_int_add(l, r, "add"),
                    BinOp::Sub => self.builder.build_int_sub(l, r, "sub"),
                    BinOp::Mul => self.builder.build_int_mul(l, r, "mul"),
                    BinOp::Div => self.builder.build_int_signed_div(l, r, "div"),
                    BinOp::Rem => self.builder.build_int_signed_rem(l, r, "rem"),
                    BinOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, l, r, "eq"),
                    BinOp::Ne => self.builder.build_int_compare(IntPredicate::NE, l, r, "ne"),
                    BinOp::Lt => self
                        .builder
                        .build_int_compare(IntPredicate::SLT, l, r, "lt"),
                    BinOp::Le => self
                        .builder
                        .build_int_compare(IntPredicate::SLE, l, r, "le"),
                    BinOp::Gt => self
                        .builder
                        .build_int_compare(IntPredicate::SGT, l, r, "gt"),
                    BinOp::Ge => self
                        .builder
                        .build_int_compare(IntPredicate::SGE, l, r, "ge"),
                    BinOp::BitAnd => self.builder.build_and(l, r, "and"),
                    BinOp::BitOr => self.builder.build_or(l, r, "or"),
                    BinOp::BitXor => self.builder.build_xor(l, r, "xor"),
                    BinOp::Shl => self.builder.build_left_shift(l, r, "shl"),
                    BinOp::Shr => self.builder.build_right_shift(l, r, true, "shr"),
                    BinOp::Mod => self.builder.build_int_signed_rem(l, r, "mod"),
                    BinOp::And => self.builder.build_and(l, r, "logical_and"),
                    BinOp::Or => self.builder.build_or(l, r, "logical_or"),
                    BinOp::Offset => {
                        // For offset operations, this should be handled separately as it requires pointer arithmetic
                        return Err(SemanticError::CodeGenError {
                            message: "Offset operation should not be used with integer operands"
                                .to_string(),
                        });
                    }
                }
                .map_err(|e| SemanticError::CodeGenError {
                    message: e.to_string(),
                })?;

                Ok(result.into())
            }

            (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => match op {
                BinOp::Add => {
                    let result = self.builder.build_float_add(l, r, "fadd").map_err(|e| {
                        SemanticError::CodeGenError {
                            message: e.to_string(),
                        }
                    })?;
                    Ok(result.into())
                }
                BinOp::Sub => {
                    let result = self.builder.build_float_sub(l, r, "fsub").map_err(|e| {
                        SemanticError::CodeGenError {
                            message: e.to_string(),
                        }
                    })?;
                    Ok(result.into())
                }
                BinOp::Mul => {
                    let result = self.builder.build_float_mul(l, r, "fmul").map_err(|e| {
                        SemanticError::CodeGenError {
                            message: e.to_string(),
                        }
                    })?;
                    Ok(result.into())
                }
                BinOp::Div => {
                    let result = self.builder.build_float_div(l, r, "fdiv").map_err(|e| {
                        SemanticError::CodeGenError {
                            message: e.to_string(),
                        }
                    })?;
                    Ok(result.into())
                }
                BinOp::Rem => {
                    let result = self.builder.build_float_rem(l, r, "frem").map_err(|e| {
                        SemanticError::CodeGenError {
                            message: e.to_string(),
                        }
                    })?;
                    Ok(result.into())
                }
                BinOp::Eq => {
                    let result = self
                        .builder
                        .build_float_compare(FloatPredicate::OEQ, l, r, "feq")
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                    Ok(result.into())
                }
                BinOp::Ne => {
                    let result = self
                        .builder
                        .build_float_compare(FloatPredicate::ONE, l, r, "fne")
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                    Ok(result.into())
                }
                BinOp::Lt => {
                    let result = self
                        .builder
                        .build_float_compare(FloatPredicate::OLT, l, r, "flt")
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                    Ok(result.into())
                }
                BinOp::Le => {
                    let result = self
                        .builder
                        .build_float_compare(FloatPredicate::OLE, l, r, "fle")
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                    Ok(result.into())
                }
                BinOp::Gt => {
                    let result = self
                        .builder
                        .build_float_compare(FloatPredicate::OGT, l, r, "fgt")
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                    Ok(result.into())
                }
                BinOp::Ge => {
                    let result = self
                        .builder
                        .build_float_compare(FloatPredicate::OGE, l, r, "fge")
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                    Ok(result.into())
                }
                _ => Err(SemanticError::InvalidType {
                    type_name: "float".to_string(),
                    reason: format!("Unsupported float operation: {:?}", op),
                    location: crate::error::SourceLocation::unknown(),
                }),
            },

            _ => Err(SemanticError::InvalidType {
                type_name: "mixed".to_string(),
                reason: "Binary operations on mixed types not supported".to_string(),
                location: crate::error::SourceLocation::unknown(),
            }),
        }
    }

    /// Generate code for a unary operation
    fn generate_unary_op(
        &mut self,
        op: UnOp,
        operand: BasicValueEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, SemanticError> {
        match operand {
            BasicValueEnum::IntValue(val) => {
                let result = match op {
                    UnOp::Neg => {
                        let zero = self.context.i64_type().const_zero();
                        self.builder.build_int_sub(zero, val, "neg")
                    }
                    UnOp::Not => {
                        // For logical NOT on booleans, use XOR with 1 instead of bitwise NOT
                        // Bitwise NOT (~1 = -2) doesn't work for booleans because -2 is still truthy
                        // XOR with 1: 1 ^ 1 = 0, 0 ^ 1 = 1
                        let one = val.get_type().const_int(1, false);
                        self.builder.build_xor(val, one, "not")
                    }
                }
                .map_err(|e| SemanticError::CodeGenError {
                    message: e.to_string(),
                })?;

                Ok(result.into())
            }

            BasicValueEnum::FloatValue(val) => {
                let result = match op {
                    UnOp::Neg => self.builder.build_float_neg(val, "fneg"),
                    UnOp::Not => {
                        return Err(SemanticError::InvalidType {
                            type_name: "float".to_string(),
                            reason: "Logical not operation not supported on floats".to_string(),
                            location: crate::error::SourceLocation::unknown(),
                        })
                    }
                }
                .map_err(|e| SemanticError::CodeGenError {
                    message: e.to_string(),
                })?;

                Ok(result.into())
            }

            _ => Err(SemanticError::InvalidType {
                type_name: "unknown".to_string(),
                reason: format!("Unsupported unary operation: {:?}", op),
                location: crate::error::SourceLocation::unknown(),
            }),
        }
    }

    /// Generate code for a terminator
    fn generate_terminator(&mut self, terminator: &Terminator) -> Result<(), SemanticError> {
        match terminator {
            Terminator::Return => {
                if let Some(return_local) = self.return_local {
                    // Load the return value from the return local
                    let return_ptr = self.local_values[&return_local];
                    let return_type = self.local_types[&return_local];
                    let return_val = self
                        .builder
                        .build_load(return_type, return_ptr, "return_value")
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                    self.builder.build_return(Some(&return_val)).map_err(|e| {
                        SemanticError::CodeGenError {
                            message: e.to_string(),
                        }
                    })?;
                } else {
                    // Void return
                    self.builder
                        .build_return(None)
                        .map_err(|e| SemanticError::CodeGenError {
                            message: e.to_string(),
                        })?;
                }
            }

            Terminator::Goto { target } => {
                let target_block = self.basic_blocks[target];
                self.builder
                    .build_unconditional_branch(target_block)
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
            }

            Terminator::SwitchInt {
                discriminant,
                targets,
                ..
            } => {
                let _discriminant_val = self.generate_operand(discriminant)?;
                let _otherwise_block = self.basic_blocks[&targets.otherwise];
                // Simplified switch implementation - just jump to otherwise block
                let otherwise_block = self.basic_blocks[&targets.otherwise];
                self.builder
                    .build_unconditional_branch(otherwise_block)
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
            }

            Terminator::Unreachable => {
                self.builder
                    .build_unreachable()
                    .map_err(|e| SemanticError::CodeGenError {
                        message: e.to_string(),
                    })?;
            }

            _ => {
                return Err(SemanticError::UnsupportedFeature {
                    feature: "Terminator type not yet implemented in LLVM backend".to_string(),
                    location: crate::error::SourceLocation::unknown(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests are disabled due to LLVM lifetime complexities
    // The main LLVM backend functionality is tested in integration tests
    //
    // #[test]
    // fn test_codegen_empty_function() {
    //     // Test implementation would require complex lifetime management
    // }
}
