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

//! AST to MIR lowering module
//!
//! Converts the high-level AST representation into MIR form

use crate::ast::{self, PrimitiveType};
use crate::error::{SemanticError, SourceLocation};
use crate::mir::Builder;
use crate::mir::*;
use crate::symbols::{SymbolKind, SymbolTable};
use crate::types::{Type, TypeDefinition};
use std::collections::HashMap;

/// Loop context for tracking break/continue targets
#[derive(Debug, Clone)]
struct LoopContext {
    /// Label for this loop (if any)
    label: Option<String>,
    /// Basic block to jump to for continue
    continue_block: BasicBlockId,
    /// Basic block to jump to for break
    break_block: BasicBlockId,
}

/// AST to MIR lowering context
pub struct LoweringContext {
    /// MIR builder
    builder: Builder,

    /// Variable name to local ID mapping
    var_map: HashMap<String, LocalId>,

    /// Variable name to type mapping for type inference
    var_types: HashMap<String, Type>,

    /// Current module being lowered
    current_module: Option<String>,

    /// Generated MIR program
    program: Program,

    /// Return value local for current function
    return_local: Option<LocalId>,

    /// Stack of loop contexts for break/continue
    loop_stack: Vec<LoopContext>,

    /// Symbol table from semantic analysis
    symbol_table: Option<SymbolTable>,

    /// Counter for generating unique lambda names
    lambda_counter: usize,
}

impl LoweringContext {
    pub fn new() -> Self {
        Self {
            builder: Builder::new(),
            var_map: HashMap::new(),
            var_types: HashMap::new(),
            current_module: None,
            program: Program {
                functions: HashMap::new(),
                global_constants: HashMap::new(),
                external_functions: HashMap::new(),
                type_definitions: HashMap::new(),
            },
            return_local: None,
            loop_stack: Vec::new(),
            symbol_table: None,
            lambda_counter: 0,
        }
    }

    /// Create a new lowering context with a symbol table
    pub fn with_symbol_table(symbol_table: SymbolTable) -> Self {
        let mut ctx = Self::new();
        ctx.symbol_table = Some(symbol_table);
        ctx
    }

    /// Lower an AST program to MIR
    pub fn lower_program(&mut self, ast_program: &ast::Program) -> Result<Program, SemanticError> {
        // Copy type definitions from symbol table if available
        if let Some(ref symbol_table) = self.symbol_table {
            self.program.type_definitions = symbol_table.get_type_definitions().clone();
        }

        for module in &ast_program.modules {
            self.lower_module(module)?;
        }

        Ok(self.program.clone())
    }

    /// Lower a module
    fn lower_module(&mut self, module: &ast::Module) -> Result<(), SemanticError> {
        self.current_module = Some(module.name.name.clone());

        // Lower constants
        for constant in &module.constant_declarations {
            self.lower_constant(constant)?;
        }

        // Lower external functions
        for ext_func in &module.external_functions {
            self.lower_external_function(ext_func)?;
        }

        // Lower functions
        for function in &module.function_definitions {
            self.lower_function(function)?;
        }

        Ok(())
    }

    /// Lower a constant declaration
    fn lower_constant(&mut self, constant: &ast::ConstantDeclaration) -> Result<(), SemanticError> {
        let const_value = self.evaluate_constant_expression(&constant.value)?;

        self.program.global_constants.insert(
            constant.name.name.clone(),
            Constant {
                ty: self.ast_type_to_mir_type(&constant.type_spec)?,
                value: const_value,
            },
        );

        Ok(())
    }

    /// Lower an external function
    fn lower_external_function(
        &mut self,
        ext_func: &ast::ExternalFunction,
    ) -> Result<(), SemanticError> {
        let mut param_types = Vec::new();
        for param in &ext_func.parameters {
            param_types.push(self.ast_type_to_mir_type(&param.param_type)?);
        }

        self.program.external_functions.insert(
            ext_func.name.name.clone(),
            ExternalFunction {
                name: ext_func.name.name.clone(),
                parameters: param_types,
                return_type: self.ast_type_to_mir_type(&ext_func.return_type)?,
                calling_convention: self.convert_calling_convention(&ext_func.calling_convention),
                variadic: ext_func.variadic,
            },
        );

        Ok(())
    }

    /// Lower a function definition
    fn lower_function(&mut self, function: &ast::Function) -> Result<(), SemanticError> {
        self.var_map.clear();
        self.var_types.clear();

        // Extract parameter info
        let mut params = Vec::new();
        for param in &function.parameters {
            let param_type = self.ast_type_to_mir_type(&param.param_type)?;
            params.push((param.name.name.clone(), param_type.clone()));
            // Also track parameter types for type inference
            self.var_types.insert(param.name.name.clone(), param_type);
        }

        let return_type = self.ast_type_to_mir_type(&function.return_type)?;

        // Start building the function
        self.builder
            .start_function(function.name.name.clone(), params, return_type.clone());

        // Create a local for the return value if not void
        let return_local = match &return_type {
            Type::Primitive(ast::PrimitiveType::Void) => None,
            _ => {
                let local_id = self.builder.new_local(return_type.clone(), false);
                self.builder
                    .push_statement(Statement::StorageLive(local_id));
                Some(local_id)
            }
        };
        self.return_local = return_local;

        // Map parameters to locals
        // The builder has already created locals for parameters, so we need to map
        // AST parameter names to the local IDs created by the builder
        if let Some(current_func) = &self.builder.current_function {
            for (ast_param, mir_param) in function
                .parameters
                .iter()
                .zip(current_func.parameters.iter())
            {
                self.var_map
                    .insert(ast_param.name.name.clone(), mir_param.local_id);
            }
        }

        // Lower function body
        self.lower_block(&function.body)?;

        // Add implicit return if needed
        if let Some(func) = &self.builder.current_function {
            if let Some(block_id) = self.builder.current_block {
                if let Some(block) = func.basic_blocks.get(&block_id) {
                    if matches!(block.terminator, Terminator::Unreachable) {
                        self.builder.set_terminator(Terminator::Return);
                    }
                }
            }
        }

        // Finish and add to program
        let mut mir_function = self.builder.finish_function();
        mir_function.return_local = self.return_local;
        self.program
            .functions
            .insert(function.name.name.clone(), mir_function);

        Ok(())
    }

    /// Lower a block
    fn lower_block(&mut self, block: &ast::Block) -> Result<(), SemanticError> {
        let _scope = self.builder.push_scope();

        eprintln!("Lowering block with {} statements", block.statements.len());
        for (i, statement) in block.statements.iter().enumerate() {
            eprintln!("Lowering statement {}: {:?}", i, statement);
            self.lower_statement(statement)?;
        }

        self.builder.pop_scope();
        Ok(())
    }

    /// Lower a statement
    fn lower_statement(&mut self, statement: &ast::Statement) -> Result<(), SemanticError> {
        match statement {
            ast::Statement::VariableDeclaration {
                name,
                type_spec,
                mutability,
                initial_value,
                source_location,
                ..
            } => {
                let mut ty = self.ast_type_to_mir_type(type_spec)?;
                let is_mutable = matches!(mutability, ast::Mutability::Mutable);

                // Initialize if value provided (before creating local to infer type)
                let init_value_opt = if let Some(init_expr) = initial_value {
                    // Check if it's a lambda expression and infer type
                    if let ast::Expression::Lambda { parameters, return_type, .. } = init_expr.as_ref() {
                        // Build function type from lambda signature
                        let param_types: Vec<Type> = parameters.iter()
                            .map(|p| self.ast_type_to_mir_type(&p.param_type))
                            .collect::<Result<Vec<_>, _>>()?;
                        let ret_type = if let Some(rt) = return_type {
                            self.ast_type_to_mir_type(rt)?
                        } else {
                            Type::primitive(ast::PrimitiveType::Integer)
                        };
                        ty = Type::Function {
                            parameter_types: param_types,
                            return_type: Box::new(ret_type),
                        };
                    }
                    Some(self.lower_expression(init_expr)?)
                } else {
                    None
                };

                let local_id = self.builder.new_local(ty.clone(), is_mutable);

                // Emit StorageLive
                self.builder
                    .push_statement(Statement::StorageLive(local_id));

                // Store variable mapping and type
                self.var_map.insert(name.name.clone(), local_id);
                self.var_types.insert(name.name.clone(), ty.clone());

                // Assign initial value if provided
                if let Some(init_value) = init_value_opt {
                    self.builder.push_statement(Statement::Assign {
                        place: Place {
                            local: local_id,
                            projection: vec![],
                        },
                        rvalue: Rvalue::Use(init_value),
                        source_info: SourceInfo {
                            span: source_location.clone(),
                            scope: 0, // TODO: proper scope tracking
                        },
                    });
                }
            }

            ast::Statement::Assignment {
                target,
                value,
                source_location,
            } => {
                match target {
                    ast::AssignmentTarget::MapValue { map, key } => {
                        // For map value assignments, we need to call map_insert
                        let map_op = self.lower_expression(map)?;
                        let key_op = self.lower_expression(key)?;
                        let value_op = self.lower_expression(value)?;

                        // Call map_insert
                        let result_local = self
                            .builder
                            .new_local(Type::primitive(PrimitiveType::Void), false);
                        self.builder.push_statement(Statement::Assign {
                            place: Place {
                                local: result_local,
                                projection: vec![],
                            },
                            rvalue: Rvalue::Call {
                                func: Operand::Constant(Constant {
                                    ty: Type::primitive(PrimitiveType::String),
                                    value: ConstantValue::String("map_insert".to_string()),
                                }),
                                args: vec![map_op, key_op, value_op],
                            },
                            source_info: SourceInfo {
                                span: source_location.clone(),
                                scope: 0,
                            },
                        });
                    }
                    _ => {
                        // For other assignment targets, use the normal path
                        let place = self.lower_assignment_target(target)?;
                        let rvalue = self.lower_expression_to_rvalue(value)?;

                        self.builder.push_statement(Statement::Assign {
                            place,
                            rvalue,
                            source_info: SourceInfo {
                                span: source_location.clone(),
                                scope: 0,
                            },
                        });
                    }
                }
            }

            ast::Statement::Return { value, .. } => {
                if let Some(return_expr) = value {
                    if let Some(return_local) = self.return_local {
                        // Explicitly generate Operand::Constant for integer literals
                        // to ensure return values aren't affected by CSE
                        let final_return_operand = if let ast::Expression::IntegerLiteral { value: literal_value, .. } = &**return_expr {
                            Operand::Constant(Constant {
                                ty: Type::primitive(PrimitiveType::Integer),
                                value: ConstantValue::Integer(*literal_value as i128),
                            })
                        } else {
                            // For non-literal return expressions, use the normal lowering path
                            self.lower_expression(return_expr)?
                        };

                        self.builder.push_statement(Statement::Assign {
                            place: Place {
                                local: return_local,
                                projection: vec![],
                            },
                            rvalue: Rvalue::Use(final_return_operand),
                            source_info: SourceInfo {
                                span: SourceLocation::unknown(),
                                scope: 0,
                            },
                        });
                    } else {
                        let _return_value = self.lower_expression(return_expr)?;
                    }
                }
                self.builder.set_terminator(Terminator::Return);
            }

            ast::Statement::If {
                condition,
                then_block,
                else_ifs,
                else_block,
                ..
            } => {
                self.lower_if_statement(condition, then_block, else_ifs, else_block)?;
            }

            ast::Statement::WhileLoop {
                condition,
                body,
                label,
                ..
            } => {
                self.lower_while_loop(condition, body, label)?;
            }

            ast::Statement::FunctionCall {
                call,
                source_location,
            } => {
                // Function calls as statements - we still need to emit the call
                // even if we ignore the return value
                eprintln!("Lowering FunctionCall statement: {:?}", call);
                let _result = self.lower_function_call(call, source_location)?;
                eprintln!("Function call lowered successfully");
                // The function call has already been emitted as an assignment in lower_function_call
            }

            ast::Statement::FixedIterationLoop {
                counter,
                from_value,
                to_value,
                step_value,
                inclusive,
                body,
                label,
                ..
            } => {
                self.lower_fixed_iteration_loop(
                    counter, from_value, to_value, step_value, *inclusive, body, label,
                )?;
            }

            ast::Statement::Break {
                target_label,
                source_location,
            } => {
                let target_block = self.find_break_target(target_label)?;
                self.builder.set_terminator(Terminator::Goto {
                    target: target_block,
                });
                // Create a new block for any subsequent dead code
                let dead_block = self.builder.new_block();
                self.builder.switch_to_block(dead_block);
            }

            ast::Statement::Continue {
                target_label,
                source_location,
            } => {
                let target_block = self.find_continue_target(target_label)?;
                self.builder.set_terminator(Terminator::Goto {
                    target: target_block,
                });
                // Create a new block for any subsequent dead code
                let dead_block = self.builder.new_block();
                self.builder.switch_to_block(dead_block);
            }

            ast::Statement::TryBlock {
                protected_block,
                catch_clauses,
                finally_block,
                source_location,
            } => {
                self.lower_try_block(
                    protected_block,
                    catch_clauses,
                    finally_block,
                    source_location,
                )?;
            }

            ast::Statement::Throw {
                exception,
                source_location,
            } => {
                self.lower_throw_statement(exception, source_location)?;
            }

            ast::Statement::ForEachLoop {
                collection,
                element_binding,
                element_type,
                index_binding,
                body,
                label,
                source_location,
            } => {
                self.lower_for_each_loop(
                    collection,
                    element_binding,
                    element_type,
                    index_binding,
                    body,
                    label,
                    source_location,
                )?;
            }

            ast::Statement::Expression {
                expr,
                source_location,
            } => {
                // Lower the expression - the result is discarded
                let _ = self.lower_expression(expr)?;
                // Expression statements are evaluated for their side effects only
            }

            ast::Statement::Match {
                value,
                arms,
                source_location,
            } => {
                self.lower_match_statement(value, arms)?;
            }

            _ => {
                // TODO: Implement other statement types
                return Err(SemanticError::UnsupportedFeature {
                    feature: "Statement type not yet implemented in MIR lowering".to_string(),
                    location: SourceLocation::unknown(),
                });
            }
        }

        Ok(())
    }

    /// Lower an if statement
    fn lower_if_statement(
        &mut self,
        condition: &ast::Expression,
        then_block: &ast::Block,
        else_ifs: &[ast::ElseIf],
        else_block: &Option<ast::Block>,
    ) -> Result<(), SemanticError> {
        let condition_op = self.lower_expression(condition)?;

        let then_bb = self.builder.new_block();
        let else_bb = self.builder.new_block();
        let end_bb = self.builder.new_block();

        // Branch on condition
        self.builder.set_terminator(Terminator::SwitchInt {
            discriminant: condition_op,
            switch_ty: Type::primitive(PrimitiveType::Boolean),
            targets: SwitchTargets {
                values: vec![1], // true = 1
                targets: vec![then_bb],
                otherwise: else_bb,
            },
        });

        // Then block
        self.builder.switch_to_block(then_bb);
        self.lower_block(then_block)?;
        // Only set goto if block doesn't already diverge (e.g., with return)
        if !self.builder.current_block_diverges() {
            self.builder
                .set_terminator(Terminator::Goto { target: end_bb });
        }

        // Else block (including else-ifs)
        self.builder.switch_to_block(else_bb);
        if !else_ifs.is_empty() || else_block.is_some() {
            // TODO: Handle else-ifs properly
            if let Some(else_block) = else_block {
                self.lower_block(else_block)?;
            }
        }
        // Only set goto if block doesn't already diverge (e.g., with return)
        if !self.builder.current_block_diverges() {
            self.builder
                .set_terminator(Terminator::Goto { target: end_bb });
        }

        // Continue at end block
        self.builder.switch_to_block(end_bb);

        Ok(())
    }

    /// Lower a match statement
    fn lower_match_statement(
        &mut self,
        value: &ast::Expression,
        arms: &[ast::MatchArm],
    ) -> Result<(), SemanticError> {
        // Lower the value being matched
        let match_op = self.lower_expression(value)?;

        // Check if any arm has a guard - if so, use sequential guard checking
        let has_guards = arms.iter().any(|arm| arm.guard.is_some());

        // Create an end block to jump to after each arm
        let end_bb = self.builder.new_block();

        if has_guards {
            // Sequential guard checking approach
            self.lower_match_with_guards(match_op, arms, end_bb)?;
        } else {
            // Use SwitchInt for simple pattern matching without guards
            self.lower_match_with_switch(match_op, arms, end_bb)?;
        }

        // Continue at end block
        self.builder.switch_to_block(end_bb);

        Ok(())
    }

    fn lower_match_with_switch(
        &mut self,
        match_op: Operand,
        arms: &[ast::MatchArm],
        end_bb: BasicBlockId,
    ) -> Result<(), SemanticError> {
        // Collect all the literal values and their target blocks
        let mut switch_values = Vec::new();
        let mut switch_targets = Vec::new();
        let mut wildcard_block = None;
        let mut arm_blocks = Vec::new();

        // First pass: create blocks for each arm and collect values
        for arm in arms {
            let arm_bb = self.builder.new_block();
            arm_blocks.push(arm_bb);

            match &arm.pattern {
                ast::Pattern::Literal { value: lit_expr, .. } => {
                    // Extract the integer value from the literal
                    if let ast::Expression::IntegerLiteral { value: int_val, .. } = lit_expr.as_ref() {
                        switch_values.push(*int_val as u128);
                        switch_targets.push(arm_bb);
                    } else if let ast::Expression::BooleanLiteral { value: bool_val, .. } = lit_expr.as_ref() {
                        switch_values.push(if *bool_val { 1 } else { 0 });
                        switch_targets.push(arm_bb);
                    }
                }
                ast::Pattern::Wildcard { .. } => {
                    // This is the default case
                    wildcard_block = Some(arm_bb);
                }
                ast::Pattern::EnumVariant { enum_name, variant_name, .. } => {
                    // Look up the variant's discriminant value
                    // For now, use variant index as discriminant
                    if let Some(enum_ident) = enum_name {
                        if let Some(enum_def) = self.program.type_definitions.get(&enum_ident.name) {
                            if let crate::types::TypeDefinition::Enum { variants, .. } = enum_def {
                                if let Some(idx) = variants.iter().position(|v| v.name == variant_name.name) {
                                    switch_values.push(idx as u128);
                                    switch_targets.push(arm_bb);
                                }
                            }
                        }
                    } else {
                        // No enum name - treat as wildcard for now
                        wildcard_block = Some(arm_bb);
                    }
                }
            }
        }

        // Use wildcard as otherwise target, or end block if no wildcard
        let otherwise = wildcard_block.unwrap_or(end_bb);

        // Get the value_place from match_op for binding extraction (before it's moved)
        let value_place = match &match_op {
            Operand::Copy(place) | Operand::Move(place) => place.clone(),
            Operand::Constant(_) => {
                // If it's a constant, we can't extract fields from it
                // This shouldn't happen for enums with data
                Place {
                    local: 0,
                    projection: vec![],
                }
            }
        };

        // Clone switch_values for use after the terminator is set
        let switch_values_copy = switch_values.clone();

        // Check if we're matching on an enum with associated data
        // If so, we need to extract the discriminant first
        let switch_discriminant = if let Some(ast::Pattern::EnumVariant { enum_name: Some(enum_ident), source_location, .. }) = arms.first().map(|a| &a.pattern) {
            // Check if this enum has associated data
            if let Some(enum_def) = self.program.type_definitions.get(&enum_ident.name) {
                if let crate::types::TypeDefinition::Enum { variants, .. } = enum_def {
                    let has_data = variants.iter().any(|v| v.associated_type.is_some());
                    if has_data {
                        // For enums with data, extract the discriminant from the pointer
                        let disc_local = self.builder.new_local(Type::primitive(ast::PrimitiveType::Integer), false);
                        self.builder.push_statement(Statement::Assign {
                            place: Place {
                                local: disc_local,
                                projection: vec![],
                            },
                            rvalue: Rvalue::Discriminant(value_place.clone()),
                            source_info: SourceInfo {
                                span: source_location.clone(),
                                scope: 0,
                            },
                        });
                        Operand::Copy(Place {
                            local: disc_local,
                            projection: vec![],
                        })
                    } else {
                        match_op.clone()
                    }
                } else {
                    match_op.clone()
                }
            } else {
                match_op.clone()
            }
        } else {
            match_op.clone()
        };

        // Emit the switch
        self.builder.set_terminator(Terminator::SwitchInt {
            discriminant: switch_discriminant,
            switch_ty: Type::primitive(ast::PrimitiveType::Integer),
            targets: SwitchTargets {
                values: switch_values,
                targets: switch_targets,
                otherwise,
            },
        });

        // Second pass: lower each arm's body
        for (i, (arm, &arm_bb)) in arms.iter().zip(arm_blocks.iter()).enumerate() {
            self.builder.switch_to_block(arm_bb);

            // Handle pattern bindings before lowering the body
            if let ast::Pattern::EnumVariant { binding, .. } = &arm.pattern {
                if binding.is_some() {
                    // Get the discriminant value for this arm
                    let variant_idx = if i < switch_values_copy.len() {
                        switch_values_copy[i]
                    } else {
                        0
                    };
                    self.lower_pattern_bindings(&arm.pattern, &value_place, variant_idx)?;
                }
            }

            self.lower_block(&arm.body)?;
            // Only set goto if block doesn't already diverge (e.g., with return)
            if !self.builder.current_block_diverges() {
                self.builder
                    .set_terminator(Terminator::Goto { target: end_bb });
            }
        }

        Ok(())
    }

    fn lower_match_with_guards(
        &mut self,
        match_op: Operand,
        arms: &[ast::MatchArm],
        end_bb: BasicBlockId,
    ) -> Result<(), SemanticError> {
        // For guards, we need to check each arm sequentially
        // Create a chain: check_arm1 -> (guard true -> body1, guard false -> check_arm2) -> ...

        // Save entry block to set its terminator after processing arms
        let entry_bb = self.builder.current_block.expect("should have current block");

        // Process arms in reverse order to build the chain
        let arm_count = arms.len();
        let mut arm_check_blocks = vec![0u32; arm_count];
        let mut arm_body_blocks = vec![0u32; arm_count];

        // Create all blocks first
        for i in 0..arm_count {
            arm_check_blocks[i] = self.builder.new_block();
            arm_body_blocks[i] = self.builder.new_block();
        }

        // Process arms in forward order (simpler to understand)
        for i in 0..arm_count {
            let arm = &arms[i];
            let check_bb = arm_check_blocks[i];
            let body_bb = arm_body_blocks[i];
            let fallthrough_bb = if i + 1 < arm_count {
                arm_check_blocks[i + 1]
            } else {
                end_bb
            };

            // Generate the check block
            self.builder.switch_to_block(check_bb);

            // If pattern has a binding, create a local for it
            if let ast::Pattern::Wildcard { binding: Some(ident), source_location } = &arm.pattern {
                // Create a local for the binding
                let local_id = self.builder.new_local(
                    Type::primitive(ast::PrimitiveType::Integer),
                    false, // not mutable
                );
                // Store the match value in the binding
                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: local_id,
                        projection: vec![],
                    },
                    rvalue: Rvalue::Use(match_op.clone()),
                    source_info: SourceInfo {
                        span: source_location.clone(),
                        scope: 0,
                    },
                });
                // Map the binding name for use in the guard and body
                self.var_map.insert(ident.name.clone(), local_id);
            }

            if let Some(guard) = &arm.guard {
                // Evaluate the guard condition
                let guard_op = self.lower_expression(guard)?;
                // Branch based on guard
                self.builder.set_terminator(Terminator::SwitchInt {
                    discriminant: guard_op,
                    switch_ty: Type::primitive(ast::PrimitiveType::Boolean),
                    targets: SwitchTargets {
                        values: vec![1], // true
                        targets: vec![body_bb],
                        otherwise: fallthrough_bb,
                    },
                });
            } else {
                // No guard, just go to body
                self.builder.set_terminator(Terminator::Goto { target: body_bb });
            }

            // Generate the body block
            self.builder.switch_to_block(body_bb);
            self.lower_block(&arm.body)?;
            if !self.builder.current_block_diverges() {
                self.builder.set_terminator(Terminator::Goto { target: end_bb });
            }
        }

        // Switch back to entry block and set terminator to first arm's check
        self.builder.switch_to_block(entry_bb);
        self.builder.set_terminator(Terminator::Goto { target: arm_check_blocks[0] });

        Ok(())
    }

    /// Lower a while loop
    fn lower_while_loop(
        &mut self,
        condition: &ast::Expression,
        body: &ast::Block,
        label: &Option<ast::Identifier>,
    ) -> Result<(), SemanticError> {
        let loop_head = self.builder.new_block();
        let loop_body = self.builder.new_block();
        let loop_end = self.builder.new_block();

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            label: label.as_ref().map(|id| id.name.clone()),
            continue_block: loop_head,
            break_block: loop_end,
        });

        // Jump to loop head
        self.builder
            .set_terminator(Terminator::Goto { target: loop_head });

        // Loop head: check condition
        self.builder.switch_to_block(loop_head);
        let condition_op = self.lower_expression(condition)?;
        self.builder.set_terminator(Terminator::SwitchInt {
            discriminant: condition_op,
            switch_ty: Type::primitive(PrimitiveType::Boolean),
            targets: SwitchTargets {
                values: vec![1], // true = 1
                targets: vec![loop_body],
                otherwise: loop_end,
            },
        });

        // Loop body
        self.builder.switch_to_block(loop_body);
        self.lower_block(body)?;
        // Only set goto if block doesn't already diverge (e.g., with return)
        if !self.builder.current_block_diverges() {
            self.builder
                .set_terminator(Terminator::Goto { target: loop_head });
        }

        // Pop loop context
        self.loop_stack.pop();

        // Continue after loop
        self.builder.switch_to_block(loop_end);

        Ok(())
    }

    /// Find the break target for the given label (or innermost loop if None)
    fn find_break_target(
        &self,
        target_label: &Option<ast::Identifier>,
    ) -> Result<BasicBlockId, SemanticError> {
        if let Some(label) = target_label {
            // Find the loop with the matching label
            for context in self.loop_stack.iter().rev() {
                if context.label.as_ref() == Some(&label.name) {
                    return Ok(context.break_block);
                }
            }
            Err(SemanticError::UndefinedSymbol {
                symbol: format!("loop label '{}'", label.name),
                location: label.source_location.clone(),
            })
        } else {
            // Break from the innermost loop
            self.loop_stack
                .last()
                .map(|context| context.break_block)
                .ok_or_else(|| SemanticError::UnsupportedFeature {
                    feature: "break statement outside of loop".to_string(),
                    location: SourceLocation::unknown(),
                })
        }
    }

    /// Find the continue target for the given label (or innermost loop if None)
    fn find_continue_target(
        &self,
        target_label: &Option<ast::Identifier>,
    ) -> Result<BasicBlockId, SemanticError> {
        if let Some(label) = target_label {
            // Find the loop with the matching label
            for context in self.loop_stack.iter().rev() {
                if context.label.as_ref() == Some(&label.name) {
                    return Ok(context.continue_block);
                }
            }
            Err(SemanticError::UndefinedSymbol {
                symbol: format!("loop label '{}'", label.name),
                location: label.source_location.clone(),
            })
        } else {
            // Continue from the innermost loop
            self.loop_stack
                .last()
                .map(|context| context.continue_block)
                .ok_or_else(|| SemanticError::UnsupportedFeature {
                    feature: "continue statement outside of loop".to_string(),
                    location: SourceLocation::unknown(),
                })
        }
    }

    /// Lower a fixed iteration loop (FOR loop)
    fn lower_fixed_iteration_loop(
        &mut self,
        counter: &ast::Identifier,
        from_value: &ast::Expression,
        to_value: &ast::Expression,
        step_value: &Option<Box<ast::Expression>>,
        inclusive: bool,
        body: &ast::Block,
        label: &Option<ast::Identifier>,
    ) -> Result<(), SemanticError> {
        // Create the counter variable
        let counter_type = Type::primitive(PrimitiveType::Integer);
        let counter_local = self.builder.new_local(counter_type.clone(), true);
        self.builder
            .push_statement(Statement::StorageLive(counter_local));
        self.var_map.insert(counter.name.clone(), counter_local);

        // Initialize counter with from_value
        let from_op = self.lower_expression(from_value)?;
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: counter_local,
                projection: vec![],
            },
            rvalue: Rvalue::Use(from_op),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // Evaluate to_value once
        let to_op = self.lower_expression(to_value)?;
        let to_local = self.builder.new_local(counter_type.clone(), false);
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: to_local,
                projection: vec![],
            },
            rvalue: Rvalue::Use(to_op),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // Evaluate step value (default to 1)
        let step_op = if let Some(step_expr) = step_value {
            self.lower_expression(step_expr)?
        } else {
            Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(1),
            })
        };
        let step_local = self.builder.new_local(counter_type.clone(), false);
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: step_local,
                projection: vec![],
            },
            rvalue: Rvalue::Use(step_op),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // Create loop blocks
        let loop_head = self.builder.new_block();
        let loop_body = self.builder.new_block();
        let loop_increment = self.builder.new_block();
        let loop_end = self.builder.new_block();

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            label: label.as_ref().map(|id| id.name.clone()),
            continue_block: loop_increment,
            break_block: loop_end,
        });

        // Jump to loop head
        self.builder
            .set_terminator(Terminator::Goto { target: loop_head });

        // Loop head: check if counter <= to_value (or < if not inclusive)
        self.builder.switch_to_block(loop_head);
        let condition_local = self
            .builder
            .new_local(Type::primitive(PrimitiveType::Boolean), false);
        let comparison_op = if inclusive { BinOp::Le } else { BinOp::Lt };
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: condition_local,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: comparison_op,
                left: Operand::Copy(Place {
                    local: counter_local,
                    projection: vec![],
                }),
                right: Operand::Copy(Place {
                    local: to_local,
                    projection: vec![],
                }),
            },
            source_info: SourceInfo {
                span: counter.source_location.clone(),
                scope: 0,
            },
        });

        self.builder.set_terminator(Terminator::SwitchInt {
            discriminant: Operand::Copy(Place {
                local: condition_local,
                projection: vec![],
            }),
            switch_ty: Type::primitive(PrimitiveType::Boolean),
            targets: SwitchTargets {
                values: vec![1], // true = 1
                targets: vec![loop_body],
                otherwise: loop_end,
            },
        });

        // Loop body
        self.builder.switch_to_block(loop_body);
        self.lower_block(body)?;
        // Only set goto if block doesn't already diverge (e.g., with return)
        if !self.builder.current_block_diverges() {
            self.builder.set_terminator(Terminator::Goto {
                target: loop_increment,
            });
        }

        // Increment block
        self.builder.switch_to_block(loop_increment);
        let increment_local = self.builder.new_local(counter_type, false);
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: increment_local,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Add,
                left: Operand::Copy(Place {
                    local: counter_local,
                    projection: vec![],
                }),
                right: Operand::Copy(Place {
                    local: step_local,
                    projection: vec![],
                }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: counter_local,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Copy(Place {
                local: increment_local,
                projection: vec![],
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        self.builder
            .set_terminator(Terminator::Goto { target: loop_head });

        // Pop loop context
        self.loop_stack.pop();

        // Continue after loop
        self.builder.switch_to_block(loop_end);

        // Clean up counter variable
        self.builder
            .push_statement(Statement::StorageDead(counter_local));
        self.var_map.remove(&counter.name);

        Ok(())
    }

    /// Lower an expression to an operand
    fn lower_expression(&mut self, expr: &ast::Expression) -> Result<Operand, SemanticError> {
        match expr {
            ast::Expression::IntegerLiteral { value, .. } => Ok(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(*value as i128),
            })),

            ast::Expression::FloatLiteral { value, .. } => Ok(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Float),
                value: ConstantValue::Float(*value),
            })),

            ast::Expression::BooleanLiteral { value, .. } => Ok(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Boolean),
                value: ConstantValue::Bool(*value),
            })),

            ast::Expression::StringLiteral { value, .. } => Ok(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::String),
                value: ConstantValue::String(value.clone()),
            })),

            ast::Expression::CharacterLiteral { value, .. } => Ok(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Char),
                value: ConstantValue::Char(*value),
            })),

            ast::Expression::Variable { name, .. } => {
                // First check local variables
                if let Some(&local_id) = self.var_map.get(&name.name) {
                    Ok(Operand::Copy(Place {
                        local: local_id,
                        projection: vec![],
                    }))
                // Then check global constants
                } else if let Some(constant) = self.program.global_constants.get(&name.name) {
                    Ok(Operand::Constant(constant.clone()))
                } else {
                    Err(SemanticError::UndefinedSymbol {
                        symbol: name.name.clone(),
                        location: name.source_location.clone(),
                    })
                }
            }

            ast::Expression::Add {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Add, left, right, source_location),

            ast::Expression::Subtract {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Sub, left, right, source_location),

            ast::Expression::Multiply {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Mul, left, right, source_location),

            ast::Expression::Divide {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Div, left, right, source_location),

            ast::Expression::Modulo {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Rem, left, right, source_location),

            ast::Expression::Equals {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Eq, left, right, source_location),

            ast::Expression::NotEquals {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Ne, left, right, source_location),

            ast::Expression::LessThan {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Lt, left, right, source_location),

            ast::Expression::GreaterThan {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Gt, left, right, source_location),

            ast::Expression::LessThanOrEqual {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Le, left, right, source_location),

            ast::Expression::GreaterThanOrEqual {
                left,
                right,
                source_location,
            } => self.lower_binary_op(BinOp::Ge, left, right, source_location),

            ast::Expression::FunctionCall {
                call,
                source_location,
            } => self.lower_function_call(call, source_location),

            ast::Expression::StringConcat {
                operands,
                source_location,
            } => self.lower_string_concat(operands, source_location),

            ast::Expression::StringLength {
                string,
                source_location,
            } => self.lower_string_length(string, source_location),

            ast::Expression::StringCharAt {
                string,
                index,
                source_location,
            } => self.lower_string_char_at(string, index, source_location),

            ast::Expression::Substring {
                string,
                start_index,
                length,
                source_location,
            } => self.lower_substring(string, start_index, length, source_location),

            ast::Expression::StringEquals {
                left,
                right,
                source_location,
            } => self.lower_string_equals(left, right, source_location),

            ast::Expression::StringContains {
                haystack,
                needle,
                source_location,
            } => self.lower_string_contains(haystack, needle, source_location),

            ast::Expression::ArrayLiteral {
                element_type,
                elements,
                source_location,
            } => self.lower_array_literal(element_type, elements, source_location),

            ast::Expression::ArrayAccess {
                array,
                index,
                source_location,
            } => self.lower_array_access(array, index, source_location),

            ast::Expression::ArrayLength {
                array,
                source_location,
            } => self.lower_array_length(array, source_location),

            ast::Expression::StructConstruct {
                type_name,
                field_values,
                source_location,
            } => self.lower_struct_construct(type_name, field_values, source_location),

            ast::Expression::FieldAccess {
                instance,
                field_name,
                source_location,
            } => self.lower_field_access(instance, field_name, source_location),

            ast::Expression::EnumVariant {
                enum_name,
                variant_name,
                value,
                source_location,
            } => self.lower_enum_variant(enum_name, variant_name, value, source_location),

            ast::Expression::Match {
                value,
                cases,
                source_location,
            } => self.lower_match_expression(value, cases, source_location),

            ast::Expression::TypeCast {
                value,
                target_type,
                failure_behavior: _,
                source_location,
            } => self.lower_type_cast(value, target_type, source_location),

            ast::Expression::AddressOf {
                operand,
                source_location,
            } => self.lower_address_of(operand, source_location),

            ast::Expression::Dereference {
                pointer,
                source_location,
            } => self.lower_dereference(pointer, source_location),

            ast::Expression::PointerArithmetic {
                pointer,
                offset,
                operation,
                source_location,
            } => self.lower_pointer_arithmetic(pointer, offset, operation, source_location),

            ast::Expression::MapLiteral {
                key_type,
                value_type,
                entries,
                source_location,
            } => self.lower_map_literal(key_type, value_type, entries, source_location),

            ast::Expression::MapAccess {
                map,
                key,
                source_location,
            } => self.lower_map_access(map, key, source_location),

            ast::Expression::LogicalAnd {
                operands,
                source_location,
            } => self.lower_logical_and(operands, source_location),

            ast::Expression::LogicalOr {
                operands,
                source_location,
            } => self.lower_logical_or(operands, source_location),

            ast::Expression::LogicalNot {
                operand,
                source_location,
            } => self.lower_logical_not(operand, source_location),

            ast::Expression::Negate {
                operand,
                source_location,
            } => self.lower_negate(operand, source_location),

            ast::Expression::Lambda {
                captures,
                parameters,
                return_type,
                body,
                source_location,
            } => self.lower_lambda(captures, parameters, return_type, body, source_location),

            _ => Err(SemanticError::UnsupportedFeature {
                feature: "Expression type not yet implemented in MIR lowering".to_string(),
                location: SourceLocation::unknown(),
            }),
        }
    }

    /// Lower a binary operation
    fn lower_binary_op(
        &mut self,
        op: BinOp,
        left: &ast::Expression,
        right: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let left_op = self.lower_expression(left)?;
        let right_op = self.lower_expression(right)?;

        // Try to infer operand types
        let left_type = self.infer_operand_type(&left_op)?;
        let right_type = self.infer_operand_type(&right_op)?;

        // Determine result type based on operation and operand types
        let result_type = match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::Mod => {
                // Numeric operations - result type follows operand types
                // If either operand is float, result is float
                if matches!(left_type, Type::Primitive(PrimitiveType::Float))
                    || matches!(right_type, Type::Primitive(PrimitiveType::Float))
                {
                    Type::primitive(PrimitiveType::Float)
                } else {
                    Type::primitive(PrimitiveType::Integer)
                }
            }
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                Type::primitive(PrimitiveType::Boolean)
            }
            BinOp::And | BinOp::Or => Type::primitive(PrimitiveType::Boolean),
            BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::Shl | BinOp::Shr => {
                // Bitwise operations always return integer
                Type::primitive(PrimitiveType::Integer)
            }
            BinOp::Offset => {
                // Pointer offset - return pointer type
                left_type
            }
        };

        // Create temporary for result
        let result_local = self.builder.new_local(result_type, false);

        // Emit assignment
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op,
                left: left_op,
                right: right_op,
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower a logical AND expression (short-circuit evaluation not implemented yet)
    fn lower_logical_and(
        &mut self,
        operands: &[ast::Expression],
        _source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        if operands.len() < 2 {
            return Err(SemanticError::UnsupportedFeature {
                feature: "LogicalAnd requires at least 2 operands".to_string(),
                location: SourceLocation::unknown(),
            });
        }

        // Start with first operand
        let mut result = self.lower_expression(&operands[0])?;

        // Chain with AND operations
        for operand in &operands[1..] {
            let right = self.lower_expression(operand)?;

            let result_local = self.builder.new_local(Type::primitive(PrimitiveType::Boolean), false);
            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: result_local,
                    projection: vec![],
                },
                rvalue: Rvalue::BinaryOp {
                    op: BinOp::And,
                    left: result,
                    right,
                },
                source_info: SourceInfo {
                    span: SourceLocation::unknown(),
                    scope: 0,
                },
            });

            result = Operand::Copy(Place {
                local: result_local,
                projection: vec![],
            });
        }

        Ok(result)
    }

    /// Lower a logical OR expression (short-circuit evaluation not implemented yet)
    fn lower_logical_or(
        &mut self,
        operands: &[ast::Expression],
        _source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        if operands.len() < 2 {
            return Err(SemanticError::UnsupportedFeature {
                feature: "LogicalOr requires at least 2 operands".to_string(),
                location: SourceLocation::unknown(),
            });
        }

        // Start with first operand
        let mut result = self.lower_expression(&operands[0])?;

        // Chain with OR operations
        for operand in &operands[1..] {
            let right = self.lower_expression(operand)?;

            let result_local = self.builder.new_local(Type::primitive(PrimitiveType::Boolean), false);
            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: result_local,
                    projection: vec![],
                },
                rvalue: Rvalue::BinaryOp {
                    op: BinOp::Or,
                    left: result,
                    right,
                },
                source_info: SourceInfo {
                    span: SourceLocation::unknown(),
                    scope: 0,
                },
            });

            result = Operand::Copy(Place {
                local: result_local,
                projection: vec![],
            });
        }

        Ok(result)
    }

    /// Lower a logical NOT expression
    fn lower_logical_not(
        &mut self,
        operand: &ast::Expression,
        _source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let operand_val = self.lower_expression(operand)?;

        let result_local = self.builder.new_local(Type::primitive(PrimitiveType::Boolean), false);
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::UnaryOp {
                op: UnOp::Not,
                operand: operand_val,
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower a negation expression
    fn lower_negate(
        &mut self,
        operand: &ast::Expression,
        _source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let operand_val = self.lower_expression(operand)?;

        // Infer the type from the operand
        let operand_type = self.infer_operand_type(&operand_val)?;

        let result_local = self.builder.new_local(operand_type, false);
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::UnaryOp {
                op: UnOp::Neg,
                operand: operand_val,
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower a lambda expression
    fn lower_lambda(
        &mut self,
        captures: &[ast::Capture],
        parameters: &[ast::Parameter],
        return_type: &Option<Box<ast::TypeSpecifier>>,
        body: &ast::LambdaBody,
        _source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Generate unique name for the lambda function
        let lambda_name = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        eprintln!("Lowering lambda to MIR function: {}", lambda_name);

        // Convert parameters to MIR types
        let mut mir_params = Vec::new();
        for param in parameters {
            let param_type = self.ast_type_to_mir_type(&param.param_type)?;
            mir_params.push((param.name.name.clone(), param_type));
        }

        // Determine return type
        let mir_return_type = if let Some(ret_type) = return_type {
            self.ast_type_to_mir_type(ret_type)?
        } else {
            // Infer from body - for now default to Int
            Type::primitive(PrimitiveType::Integer)
        };

        // Save current builder state
        let saved_var_map = std::mem::take(&mut self.var_map);
        let saved_var_types = std::mem::take(&mut self.var_types);
        let saved_return_local = self.return_local.take();
        let saved_loop_stack = std::mem::take(&mut self.loop_stack);

        // Use a fresh builder for the lambda
        let saved_builder = std::mem::replace(&mut self.builder, Builder::new());

        // Start building the lambda function
        self.builder.start_function(lambda_name.clone(), mir_params.clone(), mir_return_type.clone());

        // Set up parameter mappings
        if let Some(current_func) = &self.builder.current_function {
            for (ast_param, mir_param) in parameters.iter().zip(current_func.parameters.iter()) {
                self.var_map.insert(ast_param.name.name.clone(), mir_param.local_id);
                self.var_types.insert(ast_param.name.name.clone(), mir_param.ty.clone());
            }
        }

        // Create return local if not void
        let return_local = match &mir_return_type {
            Type::Primitive(PrimitiveType::Void) => None,
            _ => {
                let local_id = self.builder.new_local(mir_return_type.clone(), false);
                self.builder.push_statement(Statement::StorageLive(local_id));
                Some(local_id)
            }
        };
        self.return_local = return_local;

        // Lower the lambda body
        match body {
            ast::LambdaBody::Expression(expr) => {
                // Single expression - evaluate and return
                let result = self.lower_expression(expr)?;
                if let Some(ret_local) = self.return_local {
                    self.builder.push_statement(Statement::Assign {
                        place: Place { local: ret_local, projection: vec![] },
                        rvalue: Rvalue::Use(result),
                        source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
                    });
                }
                self.builder.set_terminator(Terminator::Return);
            }
            ast::LambdaBody::Block(block) => {
                // Block with statements
                self.lower_block(block)?;
                // Ensure we have a return terminator
                if let Some(func) = &self.builder.current_function {
                    if let Some(block_id) = self.builder.current_block {
                        if let Some(bb) = func.basic_blocks.get(&block_id) {
                            if matches!(bb.terminator, Terminator::Unreachable) {
                                self.builder.set_terminator(Terminator::Return);
                            }
                        }
                    }
                }
            }
        }

        // Finish the lambda function
        let mut mir_function = self.builder.finish_function();
        mir_function.return_local = self.return_local;

        // Add lambda function to program
        self.program.functions.insert(lambda_name.clone(), mir_function);

        // Restore previous builder state
        self.builder = saved_builder;
        self.var_map = saved_var_map;
        self.var_types = saved_var_types;
        self.return_local = saved_return_local;
        self.loop_stack = saved_loop_stack;

        // Create closure value
        // For now, captures are ignored (Phase 4 will handle them)
        let capture_operands: Vec<Operand> = captures.iter().map(|_c| {
            // TODO: Look up captured variable and create operand
            Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(0),
            })
        }).collect();

        // Create closure type (function pointer type)
        let closure_type = Type::Function {
            parameter_types: mir_params.iter().map(|(_, ty)| ty.clone()).collect(),
            return_type: Box::new(mir_return_type),
        };

        // Create a local for the closure and assign the closure value
        let closure_local = self.builder.new_local(closure_type, false);
        self.builder.push_statement(Statement::Assign {
            place: Place { local: closure_local, projection: vec![] },
            rvalue: Rvalue::Closure {
                func_name: lambda_name,
                captures: capture_operands,
            },
            source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
        });

        Ok(Operand::Copy(Place {
            local: closure_local,
            projection: vec![],
        }))
    }

    /// Lower a function call
    fn lower_function_call(
        &mut self,
        call: &ast::FunctionCall,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        eprintln!("lower_function_call: entering for call {:?}", call);
        // For now, only support local function references
        let function_name = match &call.function_reference {
            ast::FunctionReference::Local { name } => &name.name,
            _ => {
                return Err(SemanticError::UnsupportedFeature {
                    feature: "Non-local function references not yet supported".to_string(),
                    location: source_location.clone(),
                });
            }
        };
        eprintln!("lower_function_call: function name = {}", function_name);

        // Lower arguments
        let mut arg_operands = Vec::new();
        for arg in &call.arguments {
            let arg_operand = self.lower_expression(&arg.value)?;
            arg_operands.push(arg_operand);
        }

        // Lower variadic arguments (for functions like printf)
        for arg_expr in &call.variadic_arguments {
            let arg_operand = self.lower_expression(arg_expr)?;
            arg_operands.push(arg_operand);
        }

        // Check if this is a call to a local variable (closure call)
        if let Some(&local_id) = self.var_map.get(function_name) {
            eprintln!("lower_function_call: {} is a local variable (closure)", function_name);

            // Get the closure function pointer from the local variable
            let func_operand = Operand::Copy(Place {
                local: local_id,
                projection: vec![],
            });

            // Get the return type from the variable's function type
            let result_type = if let Some(var_type) = self.var_types.get(function_name) {
                match var_type {
                    Type::Function { return_type, .. } => (**return_type).clone(),
                    _ => Type::primitive(ast::PrimitiveType::Integer),
                }
            } else {
                Type::primitive(ast::PrimitiveType::Integer)
            };

            let result_local = self.builder.new_local(result_type, false);

            // Emit call assignment
            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: result_local,
                    projection: vec![],
                },
                rvalue: Rvalue::Call {
                    func: func_operand,
                    args: arg_operands,
                },
                source_info: SourceInfo {
                    span: source_location.clone(),
                    scope: 0,
                },
            });

            return Ok(Operand::Copy(Place {
                local: result_local,
                projection: vec![],
            }));
        }

        // Create function reference operand using the function name
        // We'll store the function name as a string constant for now
        // Skip validation for built-in functions
        let is_builtin = function_name == "printf";

        let func_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String(function_name.clone()),
        });

        // Determine the return type of the function
        let result_type = if let Some(ext_func) = self.program.external_functions.get(function_name)
        {
            // External function - use its declared return type
            eprintln!(
                "lower_function_call: found external function {} with return type {:?}",
                function_name, ext_func.return_type
            );
            ext_func.return_type.clone()
        } else if let Some(func) = self.program.functions.get(function_name) {
            // Regular function - use its declared return type
            eprintln!(
                "lower_function_call: found regular function {} with return type {:?}",
                function_name, func.return_type
            );
            func.return_type.clone()
        } else if is_builtin {
            // Built-in function - for now assume integer
            eprintln!(
                "lower_function_call: built-in function {}, assuming integer return",
                function_name
            );
            Type::primitive(ast::PrimitiveType::Integer)
        } else {
            // Try to look up in symbol table if available
            if let Some(ref symbol_table) = self.symbol_table {
                if let Some(symbol) = symbol_table.lookup_symbol(function_name) {
                    match &symbol.kind {
                        SymbolKind::Function => {
                            eprintln!("lower_function_call: found function {} in symbol table with return type {:?}", function_name, symbol.symbol_type);
                            // For functions, the symbol_type represents the function type
                            // We need to extract the return type from it
                            // For now, assume the symbol_type is the return type
                            symbol.symbol_type.clone()
                        }
                        _ => {
                            return Err(SemanticError::InvalidType {
                                type_name: function_name.clone(),
                                reason: "Symbol is not a function".to_string(),
                                location: source_location.clone(),
                            });
                        }
                    }
                } else {
                    eprintln!("lower_function_call: WARNING - function {} not found anywhere, defaulting to integer", function_name);
                    Type::primitive(ast::PrimitiveType::Integer)
                }
            } else {
                eprintln!("lower_function_call: WARNING - no symbol table, defaulting to integer for function {}", function_name);
                Type::primitive(ast::PrimitiveType::Integer)
            }
        };

        let result_local = self.builder.new_local(result_type, false);

        // Emit call assignment
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: func_operand,
                args: arg_operands,
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower an expression to an rvalue
    fn lower_expression_to_rvalue(
        &mut self,
        expr: &ast::Expression,
    ) -> Result<Rvalue, SemanticError> {
        let operand = self.lower_expression(expr)?;
        Ok(Rvalue::Use(operand))
    }

    /// Lower an assignment target
    fn lower_assignment_target(
        &mut self,
        target: &ast::AssignmentTarget,
    ) -> Result<Place, SemanticError> {
        match target {
            ast::AssignmentTarget::Variable { name } => {
                if let Some(&local_id) = self.var_map.get(&name.name) {
                    Ok(Place {
                        local: local_id,
                        projection: vec![],
                    })
                } else {
                    Err(SemanticError::UndefinedSymbol {
                        symbol: name.name.clone(),
                        location: name.source_location.clone(),
                    })
                }
            }
            ast::AssignmentTarget::MapValue { map, key } => {
                // For map assignment, we can't return a place directly
                // This will be handled specially in the assignment lowering
                Err(SemanticError::UnsupportedFeature {
                    feature: "Map value assignment requires special handling".to_string(),
                    location: SourceLocation::unknown(),
                })
            }
            _ => Err(SemanticError::UnsupportedFeature {
                feature: "Assignment target not yet implemented".to_string(),
                location: SourceLocation::unknown(),
            }),
        }
    }

    /// Evaluate a constant expression
    fn evaluate_constant_expression(
        &self,
        expr: &ast::Expression,
    ) -> Result<ConstantValue, SemanticError> {
        match expr {
            ast::Expression::IntegerLiteral { value, .. } => {
                Ok(ConstantValue::Integer(*value as i128))
            }
            ast::Expression::FloatLiteral { value, .. } => Ok(ConstantValue::Float(*value)),
            ast::Expression::BooleanLiteral { value, .. } => Ok(ConstantValue::Bool(*value)),
            ast::Expression::StringLiteral { value, .. } => {
                Ok(ConstantValue::String(value.clone()))
            }
            ast::Expression::CharacterLiteral { value, .. } => Ok(ConstantValue::Char(*value)),
            _ => Err(SemanticError::InvalidType {
                type_name: "constant".to_string(),
                reason: "Expression is not a compile-time constant".to_string(),
                location: SourceLocation::unknown(),
            }),
        }
    }

    /// Convert AST type to MIR type
    fn ast_type_to_mir_type(&self, ast_type: &ast::TypeSpecifier) -> Result<Type, SemanticError> {
        match ast_type {
            ast::TypeSpecifier::Primitive { type_name, .. } => Ok(Type::primitive(*type_name)),
            ast::TypeSpecifier::Named { name, .. } => {
                Ok(Type::named(name.name.clone(), self.current_module.clone()))
            }
            ast::TypeSpecifier::Array {
                element_type,
                size: _,
                ..
            } => {
                let elem_type = self.ast_type_to_mir_type(element_type)?;
                // TODO: Handle array size properly
                Ok(Type::array(elem_type, None))
            }
            ast::TypeSpecifier::Pointer {
                target_type,
                is_mutable,
                ..
            } => {
                let target = self.ast_type_to_mir_type(target_type)?;
                Ok(Type::pointer(target, *is_mutable))
            }
            ast::TypeSpecifier::Map {
                key_type,
                value_type,
                ..
            } => {
                let key_ty = self.ast_type_to_mir_type(key_type)?;
                let value_ty = self.ast_type_to_mir_type(value_type)?;
                Ok(Type::map(key_ty, value_ty))
            }
            ast::TypeSpecifier::Owned {
                base_type,
                ownership: _,
                ..
            } => {
                // For now, treat owned types as their base type in MIR
                // The ownership information is already tracked in the semantic layer
                self.ast_type_to_mir_type(base_type)
            }
            _ => Err(SemanticError::UnsupportedFeature {
                feature: format!("Type {:?} not yet supported in MIR", ast_type),
                location: SourceLocation::unknown(),
            }),
        }
    }

    /// Convert calling convention
    fn convert_calling_convention(&self, cc: &ast::CallingConvention) -> CallingConvention {
        match cc {
            ast::CallingConvention::C => CallingConvention::C,
            ast::CallingConvention::System => CallingConvention::System,
            _ => CallingConvention::Rust,
        }
    }

    /// Lower string concatenation
    fn lower_string_concat(
        &mut self,
        operands: &[ast::Expression],
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        if operands.len() < 2 {
            return Err(SemanticError::ArgumentCountMismatch {
                function: "STRING_CONCAT".to_string(),
                expected: 2,
                found: operands.len(),
                location: source_location.clone(),
            });
        }

        // Lower all operands
        let mut lowered_operands = Vec::new();
        for operand in operands {
            lowered_operands.push(self.lower_expression(operand)?);
        }

        // Chain multiple concatenations if more than 2 operands
        let mut result_operand = lowered_operands[0].clone();

        for i in 1..lowered_operands.len() {
            // Create function reference operand for string_concat
            let func_operand = Operand::Constant(Constant {
                ty: Type::primitive(ast::PrimitiveType::String),
                value: ConstantValue::String("string_concat".to_string()),
            });

            // Create temporary for result
            let result_local = self
                .builder
                .new_local(Type::primitive(ast::PrimitiveType::String), false);

            // Emit call assignment for this pair
            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: result_local,
                    projection: vec![],
                },
                rvalue: Rvalue::Call {
                    func: func_operand,
                    args: vec![result_operand, lowered_operands[i].clone()],
                },
                source_info: SourceInfo {
                    span: source_location.clone(),
                    scope: 0,
                },
            });

            // Update result for next iteration
            result_operand = Operand::Copy(Place {
                local: result_local,
                projection: vec![],
            });
        }

        Ok(result_operand)
    }

    /// Lower string length
    fn lower_string_length(
        &mut self,
        string: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let string_operand = self.lower_expression(string)?;

        // Create function reference operand for string_length
        let func_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String("string_length".to_string()),
        });

        // Create temporary for result
        let result_local = self
            .builder
            .new_local(Type::primitive(ast::PrimitiveType::Integer), false);

        // Emit call assignment
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: func_operand,
                args: vec![string_operand],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower string character access
    fn lower_string_char_at(
        &mut self,
        string: &ast::Expression,
        index: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let string_operand = self.lower_expression(string)?;
        let index_operand = self.lower_expression(index)?;

        // Create function reference operand for string_char_at
        let func_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String("string_char_at".to_string()),
        });

        // Create temporary for result
        let result_local = self
            .builder
            .new_local(Type::primitive(ast::PrimitiveType::Char), false);

        // Emit call assignment
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: func_operand,
                args: vec![string_operand, index_operand],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower substring
    fn lower_substring(
        &mut self,
        string: &ast::Expression,
        start: &ast::Expression,
        length: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let string_operand = self.lower_expression(string)?;
        let start_operand = self.lower_expression(start)?;
        let length_operand = self.lower_expression(length)?;

        // Create function reference operand for string_substring
        let func_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String("string_substring".to_string()),
        });

        // Create temporary for result
        let result_local = self
            .builder
            .new_local(Type::primitive(ast::PrimitiveType::String), false);

        // Emit call assignment
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: func_operand,
                args: vec![string_operand, start_operand, length_operand],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower string equals
    fn lower_string_equals(
        &mut self,
        left: &ast::Expression,
        right: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let left_operand = self.lower_expression(left)?;
        let right_operand = self.lower_expression(right)?;

        // Create function reference operand for string_compare
        let func_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String("string_compare".to_string()),
        });

        // Create temporary for comparison result
        let compare_local = self
            .builder
            .new_local(Type::primitive(ast::PrimitiveType::Integer), false);

        // Emit call assignment
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: compare_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: func_operand,
                args: vec![left_operand, right_operand],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        // Create temporary for equality result
        let result_local = self
            .builder
            .new_local(Type::primitive(ast::PrimitiveType::Boolean), false);

        // Compare result with 0 (equal strings return 0)
        let zero_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::Integer),
            value: ConstantValue::Integer(0),
        });

        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Eq,
                left: Operand::Copy(Place {
                    local: compare_local,
                    projection: vec![],
                }),
                right: zero_operand,
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower string contains
    fn lower_string_contains(
        &mut self,
        haystack: &ast::Expression,
        needle: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let string_operand = self.lower_expression(haystack)?;
        let substring_operand = self.lower_expression(needle)?;

        // Create function reference operand for string_find
        let func_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String("string_find".to_string()),
        });

        // Create temporary for find result
        let find_local = self
            .builder
            .new_local(Type::primitive(ast::PrimitiveType::Integer), false);

        // Emit call assignment
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: find_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: func_operand,
                args: vec![string_operand, substring_operand],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        // Create temporary for contains result
        let result_local = self
            .builder
            .new_local(Type::primitive(ast::PrimitiveType::Boolean), false);

        // Check if find result is not -1 (found)
        let neg_one_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::Integer),
            value: ConstantValue::Integer(-1),
        });

        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Ne,
                left: Operand::Copy(Place {
                    local: find_local,
                    projection: vec![],
                }),
                right: neg_one_operand,
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower an array literal expression
    fn lower_array_literal(
        &mut self,
        element_type: &ast::TypeSpecifier,
        elements: &[Box<ast::Expression>],
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Create the array with the right size first
        let count_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::Integer),
            value: ConstantValue::Integer(elements.len() as i128),
        });

        // Call array_create(count)
        let array_create_func = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String("array_create".to_string()),
        });

        let element_mir_type = self.ast_type_to_mir_type(element_type)?;
        let array_local = self.builder.new_local(
            Type::array(element_mir_type, None), // Correct array type
            false,
        );

        // Create the array
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: array_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: array_create_func,
                args: vec![count_operand],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        // Now set each element using array_set
        let array_set_func = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String("array_set".to_string()),
        });

        for (i, element) in elements.iter().enumerate() {
            let element_operand = self.lower_expression(element)?;
            let index_operand = Operand::Constant(Constant {
                ty: Type::primitive(ast::PrimitiveType::Integer),
                value: ConstantValue::Integer(i as i128),
            });

            let array_operand = Operand::Copy(Place {
                local: array_local,
                projection: vec![],
            });

            // Call array_set(array, index, value)
            let temp_local = self
                .builder
                .new_local(Type::primitive(ast::PrimitiveType::Void), false);

            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: temp_local,
                    projection: vec![],
                },
                rvalue: Rvalue::Call {
                    func: array_set_func.clone(),
                    args: vec![array_operand, index_operand, element_operand],
                },
                source_info: SourceInfo {
                    span: source_location.clone(),
                    scope: 0,
                },
            });
        }

        // Return the array
        Ok(Operand::Copy(Place {
            local: array_local,
            projection: vec![],
        }))
    }

    /// Lower an array access expression
    fn lower_array_access(
        &mut self,
        array: &ast::Expression,
        index: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Lower the array and index expressions
        let array_operand = self.lower_expression(array)?;
        let index_operand = self.lower_expression(index)?;

        // Create function reference for array_get
        let func_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String("array_get".to_string()),
        });

        // Create temporary for result
        let result_local = self.builder.new_local(
            Type::primitive(ast::PrimitiveType::Integer), // TODO: Use proper element type
            false,
        );

        // Emit call to array_get
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: func_operand,
                args: vec![array_operand, index_operand],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower an array length expression
    fn lower_array_length(
        &mut self,
        array: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Lower the array expression
        let array_operand = self.lower_expression(array)?;

        // Create function reference for array_length
        let func_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String("array_length".to_string()),
        });

        // Create temporary for result
        let result_local = self
            .builder
            .new_local(Type::primitive(ast::PrimitiveType::Integer), false);

        // Emit call to array_length
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: func_operand,
                args: vec![array_operand],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower a struct construction expression
    fn lower_struct_construct(
        &mut self,
        type_name: &ast::Identifier,
        field_values: &[ast::FieldValue],
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Create the struct type
        let struct_type = Type::named(type_name.name.clone(), self.current_module.clone());

        // Create a temporary for the struct
        let struct_local = self.builder.new_local(struct_type.clone(), false);

        // For now, we'll use a simplified approach - treat struct as an aggregate
        // In a real implementation, we'd need to:
        // 1. Allocate memory for the struct
        // 2. Initialize each field

        // Look up the struct definition to get the correct field order
        let type_def = self
            .symbol_table
            .as_ref()
            .and_then(|st| st.lookup_type_definition(&type_name.name))
            .ok_or_else(|| SemanticError::UndefinedSymbol {
                symbol: type_name.name.clone(),
                location: source_location.clone(),
            })?;

        let field_order: Vec<String> = match type_def {
            TypeDefinition::Struct { fields, .. } => {
                // Preserve declaration order from the struct definition
                fields.iter().map(|(name, _)| name.clone()).collect()
            }
            _ => {
                return Err(SemanticError::TypeMismatch {
                    expected: "struct type".to_string(),
                    found: "non-struct type".to_string(),
                    location: source_location.clone(),
                })
            }
        };

        // Create a map from field name to operand
        let mut field_value_map = HashMap::new();
        for field_value in field_values {
            let value_operand = self.lower_expression(&field_value.value)?;
            field_value_map.insert(field_value.field_name.name.clone(), value_operand);
        }

        // Build operands in the correct order
        let mut field_operands = Vec::new();
        for field_name in &field_order {
            if let Some(operand) = field_value_map.get(field_name) {
                field_operands.push(operand.clone());
            } else {
                return Err(SemanticError::MissingField {
                    struct_name: type_name.name.clone(),
                    field_name: field_name.clone(),
                    location: source_location.clone(),
                });
            }
        }

        // Use aggregate initialization
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: struct_local,
                projection: vec![],
            },
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Struct(type_name.name.clone(), field_order),
                operands: field_operands,
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Move(Place {
            local: struct_local,
            projection: vec![],
        }))
    }

    /// Get the type of a place
    fn get_type_of_place(&self, place: &Place) -> Result<Type, SemanticError> {
        // Start with the type of the local
        let local_type = if let Some(func) = &self.builder.current_function {
            if let Some(local_info) = func.locals.get(&place.local) {
                local_info.ty.clone()
            } else {
                // Check if it's a parameter
                for param in &func.parameters {
                    if param.local_id == place.local {
                        return Ok(param.ty.clone());
                    }
                }
                return Err(SemanticError::InternalError {
                    message: format!("Local {} not found", place.local),
                    location: SourceLocation::unknown(),
                });
            }
        } else {
            return Err(SemanticError::InternalError {
                message: "No current function in builder".to_string(),
                location: SourceLocation::unknown(),
            });
        };

        // Apply projections
        let mut current_type = local_type;
        for projection in &place.projection {
            match projection {
                PlaceElem::Field { field: _, ty } => {
                    // For field projections, the type is stored in the projection
                    current_type = ty.clone();
                }
                _ => {
                    // Other projections not implemented yet
                    return Err(SemanticError::UnsupportedFeature {
                        feature: "Non-field place projections".to_string(),
                        location: SourceLocation::unknown(),
                    });
                }
            }
        }

        Ok(current_type)
    }

    /// Infer the type of an operand
    fn infer_operand_type(&self, operand: &Operand) -> Result<Type, SemanticError> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => self.get_type_of_place(place),
            Operand::Constant(constant) => Ok(constant.ty.clone()),
        }
    }

    /// Lower a field access expression
    fn lower_field_access(
        &mut self,
        instance: &ast::Expression,
        field_name: &ast::Identifier,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Lower the instance expression
        let instance_operand = self.lower_expression(instance)?;

        // Convert to a place if it's not already
        let instance_place = match instance_operand {
            Operand::Copy(place) | Operand::Move(place) => place,
            Operand::Constant(_) => {
                return Err(SemanticError::InvalidOperation {
                    operation: "field access on constant".to_string(),
                    reason: "Cannot access fields of a constant value".to_string(),
                    location: source_location.clone(),
                });
            }
        };

        // Get the type of the instance to look up field information
        let instance_type = self.get_type_of_place(&instance_place)?;

        // Look up field index and type from the struct definition
        let (field_idx, field_type) = match &instance_type {
            Type::Named { name, .. } => {
                // Look up the struct definition
                let type_def = self
                    .symbol_table
                    .as_ref()
                    .and_then(|st| st.lookup_type_definition(name))
                    .ok_or_else(|| SemanticError::UndefinedSymbol {
                        symbol: name.clone(),
                        location: source_location.clone(),
                    })?;

                match type_def {
                    TypeDefinition::Struct { fields, .. } => {
                        // Find the field index by iterating through fields in declaration order
                        let mut field_index = None;
                        let mut field_ty = None;

                        // Fields are now stored in declaration order (Vec)
                        for (idx, (fname, ftype)) in fields.iter().enumerate() {
                            if fname == &field_name.name {
                                field_index = Some(idx as u32);
                                field_ty = Some(ftype.clone());
                                break;
                            }
                        }

                        match (field_index, field_ty) {
                            (Some(idx), Some(ty)) => (idx, ty),
                            _ => {
                                return Err(SemanticError::UndefinedSymbol {
                                    symbol: format!("{}.{}", name, field_name.name),
                                    location: source_location.clone(),
                                })
                            }
                        }
                    }
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: "struct type".to_string(),
                            found: "non-struct type".to_string(),
                            location: source_location.clone(),
                        })
                    }
                }
            }
            _ => {
                return Err(SemanticError::TypeMismatch {
                    expected: "named struct type".to_string(),
                    found: instance_type.to_string(),
                    location: source_location.clone(),
                })
            }
        };

        let field_place = Place {
            local: instance_place.local,
            projection: {
                let mut proj = instance_place.projection.clone();
                proj.push(PlaceElem::Field {
                    field: field_idx,
                    ty: field_type,
                });
                proj
            },
        };

        Ok(Operand::Copy(field_place))
    }

    /// Lower enum variant construction with known type
    fn lower_enum_variant_with_type(
        &mut self,
        enum_type_name: &str,
        variant_name: &ast::Identifier,
        value: &Option<Box<ast::Expression>>,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Lower the associated value if present
        let operands = if let Some(value_expr) = value {
            vec![self.lower_expression(value_expr)?]
        } else {
            vec![]
        };

        // Create the enum variant as an aggregate
        let result_local = self.builder.new_local(
            Type::Named {
                name: enum_type_name.to_string(),
                module: self.current_module.clone(),
            },
            false,
        );

        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Aggregate {
                kind: AggregateKind::Enum(enum_type_name.to_string(), variant_name.name.clone()),
                operands,
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Move(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower enum variant construction
    fn lower_enum_variant(
        &mut self,
        enum_name: &ast::Identifier,
        variant_name: &ast::Identifier,
        value: &Option<Box<ast::Expression>>,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Resolve the enum type properly
        let enum_type_name = if enum_name.name.is_empty() {
            // Try to find the enum type from the variant name
            if let Some(symbol_table) = &self.symbol_table {
                // Look through all type definitions to find which enum contains this variant
                let type_defs = symbol_table.get_type_definitions();
                let mut found_type_name = None;
                for (type_name, type_def) in type_defs {
                    if let TypeDefinition::Enum { variants, .. } = type_def {
                        if variants.iter().any(|v| v.name == variant_name.name) {
                            found_type_name = Some(type_name.clone());
                            break;
                        }
                    }
                }
                match found_type_name {
                    Some(type_name) => type_name,
                    None => {
                        return Err(SemanticError::UndefinedSymbol {
                            symbol: variant_name.name.clone(),
                            location: source_location.clone(),
                        })
                    }
                }
            } else {
                return Err(SemanticError::InternalError {
                    message: "No symbol table available for enum variant resolution".to_string(),
                    location: source_location.clone(),
                });
            }
        } else {
            enum_name.name.clone()
        };

        // Use the helper function
        self.lower_enum_variant_with_type(&enum_type_name, variant_name, value, source_location)
    }

    /// Lower match expression
    fn lower_match_expression(
        &mut self,
        value: &ast::Expression,
        cases: &[ast::MatchCase],
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Lower the value being matched
        let discriminant_op = self.lower_expression(value)?;

        // Get the discriminant of the enum
        let discriminant_local = self
            .builder
            .new_local(Type::primitive(ast::PrimitiveType::Integer), false);

        // Create a place from the operand for discriminant
        let value_place = match &discriminant_op {
            Operand::Copy(place) | Operand::Move(place) => place.clone(),
            Operand::Constant(_) => {
                // If it's a constant, store it in a temporary first
                // Get the type from the expression
                let temp_type = self.get_expression_type(value)?;
                let temp_local = self.builder.new_local(temp_type, false);
                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: temp_local,
                        projection: vec![],
                    },
                    rvalue: Rvalue::Use(discriminant_op.clone()),
                    source_info: SourceInfo {
                        span: source_location.clone(),
                        scope: 0,
                    },
                });
                Place {
                    local: temp_local,
                    projection: vec![],
                }
            }
        };

        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: discriminant_local,
                projection: vec![],
            },
            rvalue: Rvalue::Discriminant(value_place.clone()),
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        // Create blocks for each case and the join block
        let mut case_blocks = Vec::new();
        let join_block = self.builder.new_block();

        // Create result temporary - infer type from first case
        let result_type = if let Some(first_case) = cases.first() {
            self.get_expression_type(&first_case.body)?
        } else {
            Type::primitive(ast::PrimitiveType::Void)
        };
        let result_local = self.builder.new_local(result_type, false);

        // Get the enum type name from the value's type
        let enum_type = self.get_expression_type(value)?;
        let enum_name = match &enum_type {
            Type::Named { name, .. } => name.clone(),
            _ => {
                return Err(SemanticError::TypeMismatch {
                    expected: "enum type".to_string(),
                    found: enum_type.to_string(),
                    location: source_location.clone(),
                })
            }
        };

        // Create blocks for each case with proper discriminant values
        for case in cases.iter() {
            let case_block = self.builder.new_block();

            // Get the variant discriminant
            let discriminant = match &case.pattern {
                ast::Pattern::EnumVariant { variant_name, .. } => {
                    // Look up the enum definition to get the correct discriminant
                    if let Some(st) = &self.symbol_table {
                        if let Some(type_def) = st.lookup_type_definition(&enum_name) {
                            match type_def {
                                TypeDefinition::Enum { variants, .. } => {
                                    // Find the variant and get its discriminant
                                    variants
                                        .iter()
                                        .find(|v| v.name == variant_name.name)
                                        .map(|v| v.discriminant as u128)
                                        .unwrap_or_else(|| {
                                            eprintln!(
                                                "WARNING: Variant {} not found in enum {}, using 0",
                                                variant_name.name, enum_name
                                            );
                                            0
                                        })
                                }
                                _ => {
                                    eprintln!(
                                        "WARNING: Type {} is not an enum, using 0",
                                        enum_name
                                    );
                                    0
                                }
                            }
                        } else {
                            eprintln!("WARNING: Enum {} not found in type definitions, using variant position", enum_name);
                            // Fallback: use variant position based on common patterns
                            match variant_name.name.as_str() {
                                "Ok" | "Some" => 0,
                                "Error" | "None" => 1,
                                _ => 0,
                            }
                        }
                    } else {
                        eprintln!("WARNING: No symbol table available, using variant position");
                        0
                    }
                }
                _ => 0, // For wildcard patterns
            };

            eprintln!(
                "MIR: Case for variant {} has discriminant {}",
                match case.pattern {
                    ast::Pattern::EnumVariant {
                        ref variant_name, ..
                    } => &variant_name.name,
                    _ => "wildcard",
                },
                discriminant
            );
            case_blocks.push((discriminant, case_block));
        }

        // Emit switch terminator
        self.builder.set_terminator(Terminator::SwitchInt {
            discriminant: Operand::Copy(Place {
                local: discriminant_local,
                projection: vec![],
            }),
            switch_ty: Type::primitive(ast::PrimitiveType::Integer),
            targets: SwitchTargets {
                values: case_blocks.iter().map(|(v, _)| *v).collect(),
                targets: case_blocks.iter().map(|(_, b)| *b).collect(),
                otherwise: join_block, // TODO: Handle exhaustiveness
            },
        });

        // Lower each case
        for ((variant_idx, case_block), case) in case_blocks.iter().zip(cases.iter()) {
            self.builder.switch_to_block(*case_block);

            // Extract pattern bindings from the enum value
            self.lower_pattern_bindings(&case.pattern, &value_place, *variant_idx)?;

            // Lower the case body with bindings in scope
            let case_value = self.lower_expression(&case.body)?;

            // Assign to result
            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: result_local,
                    projection: vec![],
                },
                rvalue: Rvalue::Use(case_value),
                source_info: SourceInfo {
                    span: case.source_location.clone(),
                    scope: 0,
                },
            });

            // Jump to join block
            self.builder
                .set_terminator(Terminator::Goto { target: join_block });
        }

        // Continue in join block
        self.builder.switch_to_block(join_block);

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower pattern bindings
    fn lower_pattern_bindings(
        &mut self,
        pattern: &ast::Pattern,
        value_place: &Place,
        _variant_idx: u128,
    ) -> Result<(), SemanticError> {
        match pattern {
            ast::Pattern::EnumVariant {
                enum_name,
                variant_name,
                binding,
                nested_pattern,
                source_location: _,
            } => {
                // Handle nested pattern
                if let Some(ref nested_pat) = nested_pattern {
                    // For nested patterns, we need to extract the data and then match on it
                    // First, get the type of the variant's associated data
                    let data_type = if let Some(st) = &self.symbol_table {
                        // Look up the variant type from the enum definition
                        if let Some(enum_type) = self.get_enum_variant_type(variant_name) {
                            enum_type
                        } else {
                            eprintln!(
                                "MIR: Could not determine type for variant {}",
                                variant_name.name
                            );
                            Type::Error
                        }
                    } else {
                        Type::Error
                    };

                    // Create a place for the extracted data
                    let data_place = Place {
                        local: value_place.local,
                        projection: vec![PlaceElem::Field {
                            field: 1, // Data is at field 1 (after discriminant)
                            ty: data_type.clone(),
                        }],
                    };

                    // For nested enum patterns, we need to check the inner discriminant
                    match nested_pat.as_ref() {
                        ast::Pattern::EnumVariant {
                            variant_name: inner_variant,
                            binding: inner_binding,
                            ..
                        } => {
                            // Get the discriminant of the inner enum
                            let inner_discriminant_local = self
                                .builder
                                .new_local(Type::primitive(ast::PrimitiveType::Integer), false);

                            self.builder.push_statement(Statement::Assign {
                                place: Place {
                                    local: inner_discriminant_local,
                                    projection: vec![],
                                },
                                rvalue: Rvalue::Discriminant(data_place.clone()),
                                source_info: SourceInfo {
                                    span: variant_name.source_location.clone(),
                                    scope: 0,
                                },
                            });

                            // For now, we'll just handle the binding if it exists
                            // Full nested matching would require generating additional switch statements
                            if let Some(inner_bind) = inner_binding {
                                // Extract the data from the inner variant
                                let inner_data_place = Place {
                                    local: data_place.local,
                                    projection: vec![
                                        PlaceElem::Field {
                                            field: 1, // Outer data
                                            ty: data_type.clone(),
                                        },
                                        PlaceElem::Field {
                                            field: 1,                                         // Inner data (after inner discriminant)
                                            ty: Type::primitive(ast::PrimitiveType::Integer), // TODO: Get actual type
                                        },
                                    ],
                                };

                                // Create a local for the inner binding
                                let inner_binding_type =
                                    Type::primitive(ast::PrimitiveType::Integer); // TODO: Get actual type
                                let inner_binding_local =
                                    self.builder.new_local(inner_binding_type.clone(), false);

                                // Add to var_map and var_types
                                self.var_map
                                    .insert(inner_bind.name.clone(), inner_binding_local);
                                self.var_types
                                    .insert(inner_bind.name.clone(), inner_binding_type.clone());

                                // Copy the inner data to the binding
                                self.builder.push_statement(Statement::Assign {
                                    place: Place {
                                        local: inner_binding_local,
                                        projection: vec![],
                                    },
                                    rvalue: Rvalue::Use(Operand::Copy(inner_data_place)),
                                    source_info: SourceInfo {
                                        span: inner_bind.source_location.clone(),
                                        scope: 0,
                                    },
                                });

                                eprintln!(
                                    "MIR: Created binding {} for nested pattern",
                                    inner_bind.name
                                );
                            }
                        }
                        _ => {
                            eprintln!("MIR: Non-enum nested patterns not yet supported");
                        }
                    }
                }

                // If there's a binding (and no nested pattern), extract the enum variant's associated data
                if let Some(binding_name) = binding {
                    if nested_pattern.is_none() {
                        // Get the type of the associated data from the enum definition
                        let binding_type = self.get_enum_variant_associated_type(enum_name, variant_name)
                            .unwrap_or_else(|| {
                                eprintln!(
                                    "MIR: Could not find associated type for {:?}::{}",
                                    enum_name.as_ref().map(|e| &e.name),
                                    variant_name.name
                                );
                                Type::primitive(ast::PrimitiveType::Integer) // Default to Int
                            });

                        // Create a local for the binding
                        let binding_local = self.builder.new_local(binding_type.clone(), false);

                        // Add to var_map and var_types so it can be referenced in the case body
                        self.var_map
                            .insert(binding_name.name.clone(), binding_local);
                        self.var_types
                            .insert(binding_name.name.clone(), binding_type.clone());

                        // Generate code to extract the associated data
                        // The enum layout is: [discriminant: i32][data: variant data]
                        // We need to offset by the discriminant size (4 bytes) to get to the data

                        // For now, we'll use a simplified approach - cast the data area to the binding type
                        // In a real implementation, we'd need to properly handle the enum variant's data layout

                        // Create a projection to access the data field
                        let data_place = Place {
                            local: value_place.local,
                            projection: vec![PlaceElem::Field {
                                field: 1, // Field 1 is the data area (field 0 is discriminant)
                                ty: binding_type,
                            }],
                        };

                        // Copy the data to the binding local
                        eprintln!(
                            "MIR: Creating binding {} with type {:?} as local {}",
                            binding_name.name, &data_place.projection[0], binding_local
                        );
                        self.builder.push_statement(Statement::Assign {
                            place: Place {
                                local: binding_local,
                                projection: vec![],
                            },
                            rvalue: Rvalue::Use(Operand::Copy(data_place)),
                            source_info: SourceInfo {
                                span: binding_name.source_location.clone(),
                                scope: 0,
                            },
                        });
                    }
                }
            }
            ast::Pattern::Wildcard { binding, .. } => {
                // For wildcards, bind the entire value if requested
                if let Some(binding_name) = binding {
                    // Get the type from symbol table
                    let binding_type = if let Some(st) = &self.symbol_table {
                        if let Some(symbol) = st.lookup_symbol(&binding_name.name) {
                            match &symbol.kind {
                                SymbolKind::Variable | SymbolKind::Parameter => {
                                    symbol.symbol_type.clone()
                                }
                                _ => Type::Error,
                            }
                        } else {
                            Type::Error
                        }
                    } else {
                        Type::Error
                    };

                    // Create a local for the binding
                    let binding_local = self.builder.new_local(binding_type.clone(), false);
                    self.var_map
                        .insert(binding_name.name.clone(), binding_local);
                    self.var_types
                        .insert(binding_name.name.clone(), binding_type);

                    // Copy the entire value
                    self.builder.push_statement(Statement::Assign {
                        place: Place {
                            local: binding_local,
                            projection: vec![],
                        },
                        rvalue: Rvalue::Use(Operand::Copy(value_place.clone())),
                        source_info: SourceInfo {
                            span: binding_name.source_location.clone(),
                            scope: 0,
                        },
                    });
                }
            }
            ast::Pattern::Literal { .. } => {
                // Literal patterns don't create bindings
            }
        }

        Ok(())
    }

    /// Get the type of an enum variant's associated data
    fn get_enum_variant_type(&self, variant_name: &ast::Identifier) -> Option<Type> {
        if let Some(st) = &self.symbol_table {
            // Search through all enum definitions to find this variant
            for (_, type_def) in st.get_type_definitions() {
                if let TypeDefinition::Enum { variants, .. } = type_def {
                    for variant in variants {
                        if variant.name == variant_name.name {
                            return variant.associated_type.clone();
                        }
                    }
                }
            }
        }
        None
    }

    /// Get the associated type for a specific enum variant given the enum name
    fn get_enum_variant_associated_type(
        &self,
        enum_name: &Option<ast::Identifier>,
        variant_name: &ast::Identifier,
    ) -> Option<Type> {
        // First try to look up by enum name if provided
        if let Some(enum_ident) = enum_name {
            // Check type_definitions from program (copied from symbol table)
            if let Some(type_def) = self.program.type_definitions.get(&enum_ident.name) {
                if let TypeDefinition::Enum { variants, .. } = type_def {
                    for variant in variants {
                        if variant.name == variant_name.name {
                            return variant.associated_type.clone();
                        }
                    }
                }
            }
        }

        // Fallback: search all enums for this variant (less precise)
        self.get_enum_variant_type(variant_name)
    }

    /// Get the type of an expression
    fn get_expression_type(&self, expr: &ast::Expression) -> Result<Type, SemanticError> {
        // If we have a symbol table with type information, use it
        if let Some(st) = &self.symbol_table {
            // For now, we'll do basic type inference
            match expr {
                ast::Expression::IntegerLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::Integer))
                }
                ast::Expression::FloatLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::Float))
                }
                ast::Expression::BooleanLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::Boolean))
                }
                ast::Expression::StringLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::String))
                }
                ast::Expression::CharacterLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::Char))
                }
                ast::Expression::Variable { name, .. } => {
                    // First check local var_types mapping
                    if let Some(var_type) = self.var_types.get(&name.name) {
                        Ok(var_type.clone())
                    } else if let Some(symbol) = st.lookup_symbol(&name.name) {
                        Ok(symbol.symbol_type.clone())
                    } else {
                        Ok(Type::primitive(ast::PrimitiveType::Integer)) // Default
                    }
                }
                ast::Expression::EnumVariant { enum_name, .. } => Ok(Type::Named {
                    name: enum_name.name.clone(),
                    module: self.current_module.clone(),
                }),
                ast::Expression::FunctionCall { call, .. } => {
                    // Handle built-in functions
                    if let ast::FunctionReference::Local { name } = &call.function_reference {
                        match name.name.as_str() {
                            "STRING_CONCAT" => Ok(Type::primitive(ast::PrimitiveType::String)),
                            "TO_STRING" => Ok(Type::primitive(ast::PrimitiveType::String)),
                            "int_to_string" => Ok(Type::primitive(ast::PrimitiveType::String)),
                            _ => Ok(Type::primitive(ast::PrimitiveType::Integer)), // Default
                        }
                    } else {
                        Ok(Type::primitive(ast::PrimitiveType::Integer))
                    }
                }
                // For other expressions, use a default
                _ => Ok(Type::primitive(ast::PrimitiveType::String)), // Default to string for now
            }
        } else {
            // Without symbol table, use basic inference
            match expr {
                ast::Expression::IntegerLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::Integer))
                }
                ast::Expression::FloatLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::Float))
                }
                ast::Expression::BooleanLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::Boolean))
                }
                ast::Expression::StringLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::String))
                }
                ast::Expression::CharacterLiteral { .. } => {
                    Ok(Type::primitive(ast::PrimitiveType::Char))
                }
                ast::Expression::Variable { name, .. } => {
                    // Check local var_types mapping
                    if let Some(var_type) = self.var_types.get(&name.name) {
                        Ok(var_type.clone())
                    } else {
                        Ok(Type::primitive(ast::PrimitiveType::Integer)) // Default
                    }
                }
                _ => Ok(Type::primitive(ast::PrimitiveType::Integer)), // Default
            }
        }
    }

    /// Lower type cast expression
    fn lower_type_cast(
        &mut self,
        value: &ast::Expression,
        target_type: &ast::TypeSpecifier,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let operand = self.lower_expression(value)?;

        // Convert AST type to MIR type
        let target_ty = self.ast_type_to_mir_type(target_type)?;

        // Create temporary for result
        let result_local = self.builder.new_local(target_ty.clone(), false);

        // Determine cast kind
        let cast_kind = CastKind::Numeric; // TODO: Determine proper cast kind based on types

        // Emit cast
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Cast {
                kind: cast_kind,
                operand,
                ty: target_ty,
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower a try-catch-finally block
    fn lower_try_block(
        &mut self,
        protected_block: &ast::Block,
        catch_clauses: &[ast::CatchClause],
        finally_block: &Option<ast::Block>,
        _source_location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // For now, implement a simplified version that doesn't support actual exception handling
        // In a full implementation, we would:
        // 1. Set up exception landing pads
        // 2. Track exception propagation
        // 3. Generate cleanup code

        // Lower the protected block
        self.lower_block(protected_block)?;

        // For now, we'll just lower catch blocks as unreachable code
        // In a real implementation, these would be jumped to on exceptions
        for catch_clause in catch_clauses {
            let catch_block = self.builder.new_block();
            self.builder.switch_to_block(catch_block);

            // TODO: Add exception binding variable to scope
            if let Some(_binding) = &catch_clause.binding_variable {
                // Would bind the exception value here
            }

            self.lower_block(&catch_clause.handler_block)?;
        }

        // Lower finally block if present
        if let Some(finally) = finally_block {
            let finally_block_id = self.builder.new_block();
            self.builder.switch_to_block(finally_block_id);
            self.lower_block(finally)?;
        }

        // Continue with normal control flow
        let continue_block = self.builder.new_block();
        self.builder.switch_to_block(continue_block);

        Ok(())
    }

    /// Lower a throw statement
    fn lower_throw_statement(
        &mut self,
        exception: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // Lower the exception expression
        let exception_value = self.lower_expression(exception)?;

        // For now, we'll just generate an unreachable terminator
        // In a real implementation, this would unwind the stack
        let exception_local = self
            .builder
            .new_local(Type::primitive(PrimitiveType::Integer), false);
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: exception_local,
                projection: vec![],
            },
            rvalue: Rvalue::Use(exception_value),
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        // Mark this as a terminating statement
        self.builder.set_terminator(Terminator::Unreachable);

        // Create a new block for any subsequent dead code
        let dead_block = self.builder.new_block();
        self.builder.switch_to_block(dead_block);

        Ok(())
    }

    /// Lower a for-each loop
    fn lower_for_each_loop(
        &mut self,
        collection: &ast::Expression,
        element_binding: &ast::Identifier,
        element_type: &ast::TypeSpecifier,
        index_binding: &Option<ast::Identifier>,
        body: &ast::Block,
        _label: &Option<ast::Identifier>,
        _source_location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // Lower the collection expression
        let collection_operand = self.lower_expression(collection)?;

        // Get the element type
        let elem_type = self.ast_type_to_mir_type(element_type)?;

        // Create locals for the loop
        let index_local = self
            .builder
            .new_local(Type::primitive(PrimitiveType::Integer), false);
        let element_local = self.builder.new_local(elem_type.clone(), false);
        let collection_local = match collection_operand {
            Operand::Copy(place) | Operand::Move(place) => place.local,
            Operand::Constant(_) => {
                // If it's a constant, we need to store it in a local
                let local = self
                    .builder
                    .new_local(Type::array(elem_type.clone(), None), false);
                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local,
                        projection: vec![],
                    },
                    rvalue: Rvalue::Use(collection_operand),
                    source_info: SourceInfo {
                        span: _source_location.clone(),
                        scope: 0,
                    },
                });
                local
            }
        };

        // Store element binding
        self.var_map
            .insert(element_binding.name.clone(), element_local);
        self.var_types
            .insert(element_binding.name.clone(), elem_type.clone());

        // Store index binding if present
        if let Some(idx_binding) = index_binding {
            self.var_map.insert(idx_binding.name.clone(), index_local);
            self.var_types.insert(
                idx_binding.name.clone(),
                Type::primitive(PrimitiveType::Integer),
            );
        }

        // Initialize index to 0
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: index_local,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(0),
            })),
            source_info: SourceInfo {
                span: _source_location.clone(),
                scope: 0,
            },
        });

        // Create loop blocks
        let loop_head = self.builder.new_block();
        let loop_body = self.builder.new_block();
        let loop_end = self.builder.new_block();

        // Jump to loop head
        self.builder
            .set_terminator(Terminator::Goto { target: loop_head });

        // Loop head: check if index < array length
        self.builder.switch_to_block(loop_head);

        // Get array length
        let length_local = self
            .builder
            .new_local(Type::primitive(PrimitiveType::Integer), false);
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: length_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::String),
                    value: ConstantValue::String("array_length".to_string()),
                }),
                args: vec![Operand::Copy(Place {
                    local: collection_local,
                    projection: vec![],
                })],
            },
            source_info: SourceInfo {
                span: _source_location.clone(),
                scope: 0,
            },
        });

        // Compare index < length
        let cmp_local = self
            .builder
            .new_local(Type::primitive(PrimitiveType::Boolean), false);
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: cmp_local,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Lt,
                left: Operand::Copy(Place {
                    local: index_local,
                    projection: vec![],
                }),
                right: Operand::Copy(Place {
                    local: length_local,
                    projection: vec![],
                }),
            },
            source_info: SourceInfo {
                span: _source_location.clone(),
                scope: 0,
            },
        });

        // Branch on condition
        self.builder.set_terminator(Terminator::SwitchInt {
            discriminant: Operand::Copy(Place {
                local: cmp_local,
                projection: vec![],
            }),
            switch_ty: Type::primitive(PrimitiveType::Boolean),
            targets: SwitchTargets {
                values: vec![1],
                targets: vec![loop_body],
                otherwise: loop_end,
            },
        });

        // Loop body
        self.builder.switch_to_block(loop_body);

        // Get element at current index
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: element_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::String),
                    value: ConstantValue::String("array_get".to_string()),
                }),
                args: vec![
                    Operand::Copy(Place {
                        local: collection_local,
                        projection: vec![],
                    }),
                    Operand::Copy(Place {
                        local: index_local,
                        projection: vec![],
                    }),
                ],
            },
            source_info: SourceInfo {
                span: _source_location.clone(),
                scope: 0,
            },
        });

        // Lower the loop body
        self.lower_block(body)?;

        // Only increment and loop back if block doesn't diverge (e.g., with return)
        if !self.builder.current_block_diverges() {
            // Increment index
            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: index_local,
                    projection: vec![],
                },
                rvalue: Rvalue::BinaryOp {
                    op: BinOp::Add,
                    left: Operand::Copy(Place {
                        local: index_local,
                        projection: vec![],
                    }),
                    right: Operand::Constant(Constant {
                        ty: Type::primitive(PrimitiveType::Integer),
                        value: ConstantValue::Integer(1),
                    }),
                },
                source_info: SourceInfo {
                    span: _source_location.clone(),
                    scope: 0,
                },
            });

            // Jump back to loop head
            self.builder
                .set_terminator(Terminator::Goto { target: loop_head });
        }

        // Continue after loop
        self.builder.switch_to_block(loop_end);

        // Clean up variable mappings
        self.var_map.remove(&element_binding.name);
        self.var_types.remove(&element_binding.name);
        if let Some(idx_binding) = index_binding {
            self.var_map.remove(&idx_binding.name);
            self.var_types.remove(&idx_binding.name);
        }

        Ok(())
    }

    /// Lower address-of operation
    fn lower_address_of(
        &mut self,
        operand: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Get the place of the operand
        let operand_op = self.lower_expression(operand)?;

        // Convert operand to place
        let place = match operand_op {
            Operand::Copy(place) | Operand::Move(place) => place,
            Operand::Constant(_) => {
                return Err(SemanticError::InvalidOperation {
                    operation: "address-of".to_string(),
                    reason: "cannot take address of constant".to_string(),
                    location: source_location.clone(),
                });
            }
        };

        // Get the type of the operand
        let operand_type = self.get_expression_type(operand)?;
        let ptr_type = Type::pointer(operand_type, false);

        // Create temporary for the address
        let addr_local = self.builder.new_local(ptr_type, false);

        // Emit address-of operation
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: addr_local,
                projection: vec![],
            },
            rvalue: Rvalue::Ref {
                place,
                mutability: Mutability::Not,
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: addr_local,
            projection: vec![],
        }))
    }

    /// Lower dereference operation
    fn lower_dereference(
        &mut self,
        pointer: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let pointer_op = self.lower_expression(pointer)?;

        // Get the place of the pointer
        let pointer_place = match pointer_op {
            Operand::Copy(place) | Operand::Move(place) => place,
            Operand::Constant(_) => {
                return Err(SemanticError::InvalidOperation {
                    operation: "dereference".to_string(),
                    reason: "cannot dereference constant".to_string(),
                    location: source_location.clone(),
                });
            }
        };

        // Get the target type
        let pointer_type = self.get_expression_type(pointer)?;
        let target_type = match pointer_type {
            Type::Pointer { target_type, .. } => (*target_type).clone(),
            _ => {
                return Err(SemanticError::TypeMismatch {
                    expected: "pointer type".to_string(),
                    found: pointer_type.to_string(),
                    location: source_location.clone(),
                });
            }
        };

        // Create a place with dereference projection
        let deref_place = Place {
            local: pointer_place.local,
            projection: vec![pointer_place.projection.clone(), vec![PlaceElem::Deref]].concat(),
        };

        Ok(Operand::Copy(deref_place))
    }

    /// Lower pointer arithmetic
    fn lower_pointer_arithmetic(
        &mut self,
        pointer: &ast::Expression,
        offset: &ast::Expression,
        operation: &ast::PointerOp,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let pointer_op = self.lower_expression(pointer)?;
        let offset_op = self.lower_expression(offset)?;

        // Get pointer type
        let pointer_type = self.get_expression_type(pointer)?;

        // Create temporary for result
        let result_local = self.builder.new_local(pointer_type.clone(), false);

        // Determine the operation
        let bin_op = match operation {
            ast::PointerOp::Add => BinOp::Offset,
            ast::PointerOp::Subtract => {
                // For subtraction, we need to negate the offset first
                let neg_offset_local = self
                    .builder
                    .new_local(Type::primitive(PrimitiveType::Integer), false);
                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: neg_offset_local,
                        projection: vec![],
                    },
                    rvalue: Rvalue::UnaryOp {
                        op: UnOp::Neg,
                        operand: offset_op.clone(),
                    },
                    source_info: SourceInfo {
                        span: source_location.clone(),
                        scope: 0,
                    },
                });

                // Use the negated offset
                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: result_local,
                        projection: vec![],
                    },
                    rvalue: Rvalue::BinaryOp {
                        op: BinOp::Offset,
                        left: pointer_op,
                        right: Operand::Copy(Place {
                            local: neg_offset_local,
                            projection: vec![],
                        }),
                    },
                    source_info: SourceInfo {
                        span: source_location.clone(),
                        scope: 0,
                    },
                });

                return Ok(Operand::Copy(Place {
                    local: result_local,
                    projection: vec![],
                }));
            }
        };

        // Emit pointer offset operation
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: bin_op,
                left: pointer_op,
                right: offset_op,
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }

    /// Lower map literal
    fn lower_map_literal(
        &mut self,
        key_type: &ast::TypeSpecifier,
        value_type: &ast::TypeSpecifier,
        entries: &[ast::MapEntry],
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Convert AST types to MIR types
        let key_mir_type = self.ast_type_to_mir_type(key_type)?;
        let value_mir_type = self.ast_type_to_mir_type(value_type)?;
        let map_type = Type::map(key_mir_type, value_mir_type);

        // Create a new map
        let map_local = self.builder.new_local(map_type, false);

        // Call map_new runtime function
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: map_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::String),
                    value: ConstantValue::String("map_new".to_string()),
                }),
                args: vec![],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        // Insert each entry
        for entry in entries {
            let key_op = self.lower_expression(&entry.key)?;
            let value_op = self.lower_expression(&entry.value)?;

            // Call map_insert
            let _result_local = self
                .builder
                .new_local(Type::primitive(PrimitiveType::Void), false);
            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: _result_local,
                    projection: vec![],
                },
                rvalue: Rvalue::Call {
                    func: Operand::Constant(Constant {
                        ty: Type::primitive(PrimitiveType::String),
                        value: ConstantValue::String("map_insert".to_string()),
                    }),
                    args: vec![
                        Operand::Copy(Place {
                            local: map_local,
                            projection: vec![],
                        }),
                        key_op,
                        value_op,
                    ],
                },
                source_info: SourceInfo {
                    span: entry.source_location.clone(),
                    scope: 0,
                },
            });
        }

        Ok(Operand::Copy(Place {
            local: map_local,
            projection: vec![],
        }))
    }

    /// Lower map access
    fn lower_map_access(
        &mut self,
        map: &ast::Expression,
        key: &ast::Expression,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        let map_op = self.lower_expression(map)?;
        let key_op = self.lower_expression(key)?;

        // Get the value type from the map type
        let map_type = self.get_expression_type(map)?;
        let value_type = match map_type {
            Type::Map { value_type, .. } => (*value_type).clone(),
            _ => {
                return Err(SemanticError::TypeMismatch {
                    expected: "map type".to_string(),
                    found: map_type.to_string(),
                    location: source_location.clone(),
                });
            }
        };

        // Create temporary for result
        let result_local = self.builder.new_local(value_type, false);

        // Call map_get
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::String),
                    value: ConstantValue::String("map_get".to_string()),
                }),
                args: vec![map_op, key_op],
            },
            source_info: SourceInfo {
                span: source_location.clone(),
                scope: 0,
            },
        });

        Ok(Operand::Copy(Place {
            local: result_local,
            projection: vec![],
        }))
    }
}

impl Default for LoweringContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Lower an AST program to MIR
pub fn lower_ast_to_mir(ast_program: &ast::Program) -> Result<Program, SemanticError> {
    let mut context = LoweringContext::new();
    context.lower_program(ast_program)
}

/// Lower an AST program to MIR with symbol table information
pub fn lower_ast_to_mir_with_symbols(
    ast_program: &ast::Program,
    symbol_table: SymbolTable,
) -> Result<Program, SemanticError> {
    let mut context = LoweringContext::with_symbol_table(symbol_table);
    context.lower_program(ast_program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;
    use crate::ast::{self, Identifier};

    #[test]
    fn test_simple_function_lowering() {
        let mut ctx = LoweringContext::new();

        // Create a simple AST function
        let ast_func = ast::Function {
            name: Identifier::new("test".to_string(), SourceLocation::unknown()),
            intent: None,
            generic_parameters: vec![],
            parameters: vec![],
            return_type: Box::new(ast::TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                source_location: SourceLocation::unknown(),
            }),
            metadata: ast::FunctionMetadata {
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
            body: ast::Block {
                statements: vec![ast::Statement::Return {
                    value: Some(Box::new(ast::Expression::IntegerLiteral {
                        value: 42,
                        source_location: SourceLocation::unknown(),
                    })),
                    source_location: SourceLocation::unknown(),
                }],
                source_location: SourceLocation::unknown(),
            },
            export_info: None,
            source_location: SourceLocation::unknown(),
        };

        ctx.lower_function(&ast_func)
            .expect("Lowering should succeed");

        assert!(ctx.program.functions.contains_key("test"));
        let mir_func = &ctx.program.functions["test"];
        assert_eq!(mir_func.name, "test");
        assert_eq!(mir_func.basic_blocks.len(), 1);
    }
}
