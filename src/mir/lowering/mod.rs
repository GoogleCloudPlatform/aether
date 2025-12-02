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

#![allow(dead_code)]

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

    /// Map of concurrent block locations to captured variable names
    concurrent_captures: Option<HashMap<SourceLocation, std::collections::HashSet<String>>>,

    /// Imported modules in the current module (alias -> module name)
    imported_modules: HashMap<String, String>,

    /// Runtime postconditions for the current function (stored for checking at return points)
    runtime_postconditions: Vec<ast::ContractAssertion>,
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
            concurrent_captures: None,
            imported_modules: HashMap::new(),
            runtime_postconditions: Vec::new(),
        }
    }

    /// Create a new lowering context with a symbol table
    pub fn with_symbol_table(symbol_table: SymbolTable) -> Self {
        let mut ctx = Self::new();
        ctx.symbol_table = Some(symbol_table);
        ctx
    }

    /// Set the concurrent captures map
    pub fn set_captures(
        &mut self,
        captures: HashMap<SourceLocation, std::collections::HashSet<String>>,
    ) {
        self.concurrent_captures = Some(captures);
    }

    /// Ensure operand is compatible with target type (awaiting or casting if necessary)
    fn ensure_compatible_operand(
        &mut self,
        operand: Operand,
        target_type: &Type,
    ) -> Result<Operand, SemanticError> {
        // Check if operand is Future and target is not
        let op_type = self.infer_operand_type(&operand)?;

        if let Type::GenericInstance {
            base_type,
            type_arguments,
            ..
        } = &op_type
        {
            if base_type == "Future" && !type_arguments.is_empty() {
                // Operand is Future<T>
                // Check if target type is Future (or generic/unspecified)
                // If target is specifically NOT a future (e.g. Int), await it.
                let target_is_future = if let Type::GenericInstance { base_type, .. } = target_type
                {
                    base_type == "Future"
                } else {
                    false
                };

                if !target_is_future {
                    return self.maybe_await_operand(operand);
                }
            }
        }

        // Check for Numeric Casting (Int -> Int64)
        match (&op_type, target_type) {
            (
                Type::Primitive(ast::PrimitiveType::Integer),
                Type::Primitive(ast::PrimitiveType::Integer64),
            )
            | (
                Type::Primitive(ast::PrimitiveType::Integer32),
                Type::Primitive(ast::PrimitiveType::Integer64),
            ) => {
                let temp = self.builder.new_local(target_type.clone(), false);
                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: temp,
                        projection: vec![],
                    },
                    rvalue: Rvalue::Cast {
                        kind: CastKind::Numeric,
                        operand,
                        ty: target_type.clone(),
                    },
                    source_info: SourceInfo {
                        span: SourceLocation::unknown(),
                        scope: 0,
                    },
                });
                return Ok(Operand::Copy(Place {
                    local: temp,
                    projection: vec![],
                }));
            }
            _ => {}
        }

        Ok(operand)
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
        self.imported_modules.clear();

        // Register imports
        for import in &module.imports {
            let alias = if let Some(alias) = &import.alias {
                alias.name.clone()
            } else {
                import.module_name.name.clone()
            };
            self.imported_modules
                .insert(alias, import.module_name.name.clone());
        }

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
                symbol: ext_func.symbol.clone(),
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
        self.runtime_postconditions.clear();

        // Extract parameter info
        let mut params = Vec::new();
        for param in &function.parameters {
            let param_type = self.ast_type_to_mir_type(&param.param_type)?;
            params.push((param.name.name.clone(), param_type.clone()));
            // Also track parameter types for type inference
            self.var_types.insert(param.name.name.clone(), param_type);
        }

        let return_type = self.ast_type_to_mir_type(&function.return_type)?;

        let function_name = if let Some(mod_name) = &self.current_module {
            if function.name.name == "main" {
                "main".to_string()
            } else {
                format!("{}.{}", mod_name, function.name.name)
            }
        } else {
            function.name.name.clone()
        };

        // Start building the function
        self.builder
            .start_function(function_name.clone(), params, return_type.clone());

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

        // Store runtime postconditions for checking at return points
        for postcond in &function.metadata.postconditions {
            if postcond.runtime_check {
                self.runtime_postconditions.push(postcond.clone());
            }
        }

        // Emit runtime precondition checks at function entry
        for precond in &function.metadata.preconditions {
            if precond.runtime_check {
                self.emit_contract_assertion(&precond.condition, &precond.message, true)?;
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
        self.program.functions.insert(function_name, mir_function);

        Ok(())
    }

    /// Emit a runtime contract assertion (used for @pre/@post with check=runtime)
    ///
    /// This generates an Assert terminator that will panic at runtime if the condition is false.
    /// For postconditions, `is_precondition` should be false to allow `return_value` references.
    fn emit_contract_assertion(
        &mut self,
        condition: &ast::Expression,
        message: &Option<String>,
        is_precondition: bool,
    ) -> Result<(), SemanticError> {
        // Lower the condition expression to an operand
        let condition_operand = self.lower_expression(condition)?;

        // Create the continuation block (where execution goes after the check passes)
        let continue_block = self.builder.new_block();

        // Build the error message
        let msg = message.clone().unwrap_or_else(|| {
            if is_precondition {
                "Precondition violated".to_string()
            } else {
                "Postcondition violated".to_string()
            }
        });

        // Set the Assert terminator
        self.builder.set_terminator(Terminator::Assert {
            condition: condition_operand,
            expected: true,
            message: AssertMessage::Custom(msg),
            target: continue_block,
            cleanup: None,
        });

        // Switch to the continuation block for subsequent code
        self.builder.switch_to_block(continue_block);

        Ok(())
    }

    /// Emit postcondition checks for the current function
    /// Called before return statements to verify postconditions
    fn emit_postcondition_checks(&mut self) -> Result<(), SemanticError> {
        // Clone the postconditions to avoid borrow checker issues
        let postconditions = self.runtime_postconditions.clone();

        for postcond in &postconditions {
            // Need to substitute `return_value` references with the actual return local
            // For now, we lower the expression directly - the `return_value` identifier
            // should already be mapped if we're returning a value
            self.emit_contract_assertion(&postcond.condition, &postcond.message, false)?;
        }

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
                    if let ast::Expression::Lambda {
                        parameters,
                        return_type,
                        ..
                    } = init_expr.as_ref()
                    {
                        // Build function type from lambda signature
                        let param_types: Vec<Type> = parameters
                            .iter()
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
                            is_variadic: false,
                        };
                    }

                    let op = self.lower_expression(init_expr)?;

                    // If type is inferred, use the type of the initializer
                    if let Type::Named { name, .. } = &ty {
                        if name == "_inferred" {
                            ty = self.infer_operand_type(&op)?;
                        }
                    }

                    // Check compatibility and await if needed
                    let compatible_op = self.ensure_compatible_operand(op, &ty)?;
                    Some(compatible_op)
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
                                explicit_type_arguments: vec![],
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
                        let final_return_operand = if let ast::Expression::IntegerLiteral {
                            value: literal_value,
                            ..
                        } = &**return_expr
                        {
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

                        // Map return_value to the return local for postcondition checks
                        self.var_map
                            .insert("return_value".to_string(), return_local);
                    } else {
                        let _return_value = self.lower_expression(return_expr)?;
                    }
                }

                // Emit runtime postcondition checks before returning
                self.emit_postcondition_checks()?;

                self.builder.set_terminator(Terminator::Return);
            }
            ast::Statement::Concurrent {
                block,
                source_location,
            } => {
                // Look up captures for this block
                let captures = if let Some(captures_map) = &self.concurrent_captures {
                    if let Some(captured_names) = captures_map.get(source_location) {
                        let mut caps = Vec::new();
                        // Sort names to ensure deterministic order
                        let mut sorted_names: Vec<_> = captured_names.iter().collect();
                        sorted_names.sort();

                        for name in sorted_names {
                            if let Some(&local_id) = self.var_map.get(name) {
                                caps.push(Operand::Copy(Place {
                                    local: local_id,
                                    projection: vec![],
                                }));
                            }
                        }
                        caps
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                // Create a new block for the concurrent execution
                let concurrent_entry = self.builder.new_block();
                let after_concurrent = self.builder.new_block();

                // Terminate current block with Concurrent
                self.builder.set_terminator(Terminator::Concurrent {
                    block_id: concurrent_entry,
                    target: after_concurrent,
                    captures,
                });

                // Lower the concurrent block
                self.builder.switch_to_block(concurrent_entry);
                // We need to ensure the block eventually terminates or returns
                // For now, let's just compile the statements in the block
                // Note: In a real implementation, we'd want to wrap this in a closure/async context
                for stmt in &block.statements {
                    self.lower_statement(stmt)?;
                }

                // If the concurrent block falls through, it should return/end
                // For MIR structure, we might want it to jump to 'after_concurrent'
                // but semantically 'Concurrent' implies it runs separately.
                // The 'Concurrent' terminator in the parent block handles the "spawning".
                // The body of the concurrent block needs to be self-contained or jump back?
                // Actually, `Terminator::Concurrent` is a placeholder for "spawn this block".
                // The spawned block should probably end with Return (if it's a task) or similar.
                // Let's assume for now it just ends.
                if !self.builder.current_block_diverges() {
                    self.builder.set_terminator(Terminator::Return);
                }

                // Continue generation in the code after the concurrent block
                self.builder.switch_to_block(after_concurrent);
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
                source_location: _,
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
                source_location: _,
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
                source_location: _,
            } => {
                // Lower the expression - the result is discarded
                let _ = self.lower_expression(expr)?;
                // Expression statements are evaluated for their side effects only
            }

            ast::Statement::Match {
                value,
                arms,
                source_location: _,
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

        // Check if any arm has a guard or complex pattern
        let has_guards = arms.iter().any(|arm| arm.guard.is_some());
        let has_complex_patterns = arms.iter().any(|arm| {
            matches!(
                arm.pattern,
                ast::Pattern::Struct { .. }
                    | ast::Pattern::EnumVariant {
                        nested_pattern: Some(_),
                        ..
                    }
            )
        });

        // Create an end block to jump to after each arm
        let end_bb = self.builder.new_block();

        if has_guards || has_complex_patterns {
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
                ast::Pattern::Literal {
                    value: lit_expr, ..
                } => {
                    // Extract the integer value from the literal
                    if let ast::Expression::IntegerLiteral { value: int_val, .. } =
                        lit_expr.as_ref()
                    {
                        switch_values.push(*int_val as u128);
                        switch_targets.push(arm_bb);
                    } else if let ast::Expression::BooleanLiteral {
                        value: bool_val, ..
                    } = lit_expr.as_ref()
                    {
                        switch_values.push(if *bool_val { 1 } else { 0 });
                        switch_targets.push(arm_bb);
                    }
                }
                ast::Pattern::Wildcard { .. } => {
                    // This is the default case
                    wildcard_block = Some(arm_bb);
                }
                ast::Pattern::EnumVariant {
                    enum_name,
                    variant_name,
                    ..
                } => {
                    // Look up the variant's discriminant value
                    // For now, use variant index as discriminant
                    if let Some(enum_ident) = enum_name {
                        if let Some(enum_def) = self.program.type_definitions.get(&enum_ident.name)
                        {
                            if let crate::types::TypeDefinition::Enum { variants, .. } = enum_def {
                                if let Some(idx) =
                                    variants.iter().position(|v| v.name == variant_name.name)
                                {
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
                ast::Pattern::Struct { .. } => {
                    // Struct patterns should be handled by lower_match_with_guards
                    // But if we are here, treat as wildcard/unreachable for the switch construction
                    wildcard_block = Some(arm_bb);
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

        // For enum patterns, we ALWAYS need to extract the discriminant first
        // (even for simple enums without data, since they're stored as pointers)
        let switch_discriminant = if let Some(ast::Pattern::EnumVariant {
            enum_name: Some(_enum_ident),
            source_location,
            ..
        }) = arms.first().map(|a| &a.pattern)
        {
            // Extract the discriminant from the enum pointer
            let disc_local = self
                .builder
                .new_local(Type::primitive(ast::PrimitiveType::Integer), false);
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
            if let ast::Pattern::EnumVariant { bindings, .. } = &arm.pattern {
                if !bindings.is_empty() {
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
        let entry_bb = self
            .builder
            .current_block
            .expect("should have current block");

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

            // Convert match_op to a Place if possible (for field access)
            let match_place = match &match_op {
                Operand::Copy(place) | Operand::Move(place) => place.clone(),
                Operand::Constant(_) => {
                    // If it's a constant, we store it in a temporary to get a Place
                    let temp = self.builder.new_local(
                        Type::primitive(ast::PrimitiveType::Integer), // Type will be inferred/checked later
                        false,
                    );
                    self.builder.push_statement(Statement::Assign {
                        place: Place {
                            local: temp,
                            projection: vec![],
                        },
                        rvalue: Rvalue::Use(match_op.clone()),
                        source_info: SourceInfo {
                            span: SourceLocation::unknown(),
                            scope: 0,
                        },
                    });
                    Place {
                        local: temp,
                        projection: vec![],
                    }
                }
            };

            // Check if pattern matches
            let pattern_match_op = self.lower_pattern_check(&arm.pattern, &match_place)?;

            // Create a block for when pattern matches (to process bindings and guard)
            let pattern_matched_bb = self.builder.new_block();

            self.builder.set_terminator(Terminator::SwitchInt {
                discriminant: pattern_match_op,
                switch_ty: Type::primitive(ast::PrimitiveType::Boolean),
                targets: SwitchTargets {
                    values: vec![1], // true
                    targets: vec![pattern_matched_bb],
                    otherwise: fallthrough_bb,
                },
            });

            // In pattern_matched_bb, process bindings and guard
            self.builder.switch_to_block(pattern_matched_bb);

            // Extract bindings
            self.lower_pattern_bindings(&arm.pattern, &match_place, 0)?; // 0 variant idx is placeholder

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
                self.builder
                    .set_terminator(Terminator::Goto { target: body_bb });
            }

            // Generate the body block
            self.builder.switch_to_block(body_bb);
            self.lower_block(&arm.body)?;
            if !self.builder.current_block_diverges() {
                self.builder
                    .set_terminator(Terminator::Goto { target: end_bb });
            }
        }

        // Switch back to entry block and set terminator to first arm's check
        self.builder.switch_to_block(entry_bb);
        self.builder.set_terminator(Terminator::Goto {
            target: arm_check_blocks[0],
        });

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

    /// Helper to await an operand if it is a Future<T>
    fn maybe_await_operand(&mut self, operand: Operand) -> Result<Operand, SemanticError> {
        let ty = self.infer_operand_type(&operand)?;
        if let Type::GenericInstance {
            base_type,
            type_arguments,
            ..
        } = &ty
        {
            if base_type == "Future" && !type_arguments.is_empty() {
                let inner_type = &type_arguments[0];

                // Emit call to aether_async_wait
                // 1. Create temp for result pointer (i8*)
                let result_ptr_local = self.builder.new_local(
                    Type::Pointer {
                        target_type: Box::new(Type::primitive(PrimitiveType::Char)),
                        is_mutable: true,
                    },
                    false,
                );

                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: result_ptr_local,
                        projection: vec![],
                    },
                    rvalue: Rvalue::Call {
                        func: Operand::Constant(Constant {
                            ty: Type::primitive(PrimitiveType::String),
                            value: ConstantValue::String("aether_await".to_string()),
                        }),
                        explicit_type_arguments: vec![],
                        args: vec![operand],
                    },
                    source_info: SourceInfo {
                        span: SourceLocation::unknown(),
                        scope: 0,
                    },
                });

                // 2. Cast result pointer to inner_type*
                let typed_ptr_local = self.builder.new_local(
                    Type::Pointer {
                        target_type: Box::new(inner_type.clone()),
                        is_mutable: true,
                    },
                    false,
                );

                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: typed_ptr_local,
                        projection: vec![],
                    },
                    rvalue: Rvalue::Cast {
                        kind: CastKind::Pointer,
                        operand: Operand::Copy(Place {
                            local: result_ptr_local,
                            projection: vec![],
                        }),
                        ty: Type::Pointer {
                            target_type: Box::new(inner_type.clone()),
                            is_mutable: true,
                        },
                    },
                    source_info: SourceInfo {
                        span: SourceLocation::unknown(),
                        scope: 0,
                    },
                });

                // 3. Dereference to get value
                let value_local = self.builder.new_local(inner_type.clone(), false);

                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: value_local,
                        projection: vec![],
                    },
                    rvalue: Rvalue::Use(Operand::Copy(Place {
                        local: typed_ptr_local,
                        projection: vec![PlaceElem::Deref],
                    })),
                    source_info: SourceInfo {
                        span: SourceLocation::unknown(),
                        scope: 0,
                    },
                });

                return Ok(Operand::Copy(Place {
                    local: value_local,
                    projection: vec![],
                }));
            }
        }

        Ok(operand)
    }

    /// Get the index and type of a struct field
    fn get_struct_field_index(
        &self,
        struct_name: &str,
        field_name: &str,
    ) -> Result<(usize, Type), SemanticError> {
        if let Some(type_def) = self.program.type_definitions.get(struct_name) {
            if let crate::types::TypeDefinition::Struct { fields, .. } = type_def {
                if let Some(idx) = fields.iter().position(|(name, _)| name == field_name) {
                    return Ok((idx, fields[idx].1.clone()));
                }
            }
        }
        Err(SemanticError::UndefinedSymbol {
            symbol: format!("Field {} in struct {}", field_name, struct_name),
            location: SourceLocation::unknown(),
        })
    }

    /// Generate a boolean operand that is true if the pattern matches the value at place
    fn lower_pattern_check(
        &mut self,
        pattern: &ast::Pattern,
        place: &Place,
    ) -> Result<Operand, SemanticError> {
        match pattern {
            ast::Pattern::Wildcard { .. } => {
                // Always matches
                Ok(Operand::Constant(Constant {
                    ty: Type::primitive(ast::PrimitiveType::Boolean),
                    value: ConstantValue::Integer(1),
                }))
            }
            ast::Pattern::Literal { value, .. } => {
                // Evaluate literal
                let lit_op = self.lower_expression(value)?;

                // Generate comparison: place == literal
                let result_local = self
                    .builder
                    .new_local(Type::primitive(ast::PrimitiveType::Boolean), false);

                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: result_local,
                        projection: vec![],
                    },
                    rvalue: Rvalue::BinaryOp {
                        op: BinOp::Eq,
                        left: Operand::Copy(place.clone()),
                        right: lit_op,
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
            ast::Pattern::Struct {
                struct_name,
                fields,
                ..
            } => {
                // Check each field
                let mut conditions = Vec::new();

                for (field_name, field_pattern) in fields {
                    let (field_idx, field_type) =
                        self.get_struct_field_index(&struct_name.name, &field_name.name)?;

                    let field_place = Place {
                        local: place.local,
                        projection: place
                            .projection
                            .iter()
                            .cloned()
                            .chain(vec![PlaceElem::Field {
                                field: field_idx as u32,
                                ty: field_type,
                            }])
                            .collect(),
                    };

                    let field_check = self.lower_pattern_check(field_pattern, &field_place)?;
                    conditions.push(field_check);
                }

                // Combine conditions with AND
                if conditions.is_empty() {
                    Ok(Operand::Constant(Constant {
                        ty: Type::primitive(ast::PrimitiveType::Boolean),
                        value: ConstantValue::Integer(1),
                    }))
                } else {
                    let mut result_op = conditions[0].clone();
                    for i in 1..conditions.len() {
                        let right_op = conditions[i].clone();
                        let result_local = self
                            .builder
                            .new_local(Type::primitive(ast::PrimitiveType::Boolean), false);
                        self.builder.push_statement(Statement::Assign {
                            place: Place {
                                local: result_local,
                                projection: vec![],
                            },
                            rvalue: Rvalue::BinaryOp {
                                op: BinOp::And,
                                left: result_op,
                                right: right_op,
                            },
                            source_info: SourceInfo {
                                span: SourceLocation::unknown(),
                                scope: 0,
                            },
                        });
                        result_op = Operand::Copy(Place {
                            local: result_local,
                            projection: vec![],
                        });
                    }
                    Ok(result_op)
                }
            }
            ast::Pattern::EnumVariant { .. } => {
                // TODO: Implement enum variant check
                // For now assume true if used in struct match context
                Ok(Operand::Constant(Constant {
                    ty: Type::primitive(ast::PrimitiveType::Boolean),
                    value: ConstantValue::Integer(1),
                }))
            }
        }
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
                values,
                source_location,
            } => self.lower_enum_variant(enum_name, variant_name, values, source_location),

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
                mutability,
                source_location,
            } => self.lower_address_of(operand, *mutability, source_location),

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

            ast::Expression::MethodCall {
                receiver,
                method_name,
                arguments,
                source_location,
            } => self.lower_method_call(receiver, method_name, arguments, source_location),

            _ => Err(SemanticError::UnsupportedFeature {
                feature: "Expression type not yet implemented in MIR lowering".to_string(),
                location: SourceLocation::unknown(),
            }),
        }
    }

    fn lower_method_call(
        &mut self,
        receiver: &ast::Expression,
        method_name: &ast::Identifier,
        arguments: &[ast::Argument],
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Check for module function call (e.g. io.println)
        if let ast::Expression::Variable { name, .. } = receiver {
            if let Some(module_name) = self.imported_modules.get(&name.name) {
                // It's a module function call
                let qualified_name = format!("{}.{}", module_name, method_name.name);

                // Look up return type from symbol table if available, otherwise void
                let return_type = if let Some(st) = &self.symbol_table {
                    if let Some(func_sym) = st.lookup_symbol(&qualified_name) {
                        if let crate::types::Type::Function {
                            parameter_types,
                            return_type,
                            is_variadic: _,
                        } = &func_sym.symbol_type
                        {
                            // Register as external function if not already known
                            if !self.program.functions.contains_key(&qualified_name)
                                && !self
                                    .program
                                    .external_functions
                                    .contains_key(&qualified_name)
                            {
                                let ext_func = crate::mir::ExternalFunction {
                                    name: qualified_name.clone(),
                                    symbol: None, // Or lookup from somewhere? But for now None.
                                    parameters: parameter_types.clone(),
                                    return_type: *return_type.clone(),
                                    calling_convention: crate::mir::CallingConvention::C,
                                    variadic: false,
                                };
                                self.program
                                    .external_functions
                                    .insert(qualified_name.clone(), ext_func);
                            }
                            *return_type.clone()
                        } else {
                            crate::types::Type::primitive(ast::PrimitiveType::Void)
                        }
                    } else {
                        crate::types::Type::primitive(ast::PrimitiveType::Void)
                    }
                } else {
                    crate::types::Type::primitive(ast::PrimitiveType::Void)
                };

                let mut lowered_args = Vec::new();
                for arg in arguments {
                    lowered_args.push(self.lower_expression(&arg.value)?);
                }

                let result_local = self.builder.new_local(return_type, false);

                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: result_local,
                        projection: vec![],
                    },
                    rvalue: Rvalue::Call {
                        func: Operand::Constant(Constant {
                            ty: crate::types::Type::primitive(ast::PrimitiveType::String),
                            value: ConstantValue::String(qualified_name),
                        }),
                        explicit_type_arguments: vec![],
                        args: lowered_args,
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
        }

        // For map methods "insert" and "get", lower to map_insert/map_get runtime calls
        // In a real compiler, this would look up the type of receiver and dispatch appropriately
        // For now, we'll assume it's a map if the method name matches map operations

        if method_name.name == "insert" {
            // map.insert(key, value) -> map_insert(map, key, value)
            let map_op = self.lower_expression(receiver)?;

            if arguments.len() != 2 {
                return Err(SemanticError::ArgumentCountMismatch {
                    function: "map.insert".to_string(),
                    expected: 2,
                    found: arguments.len(),
                    location: source_location.clone(),
                });
            }

            let key_op = self.lower_expression(&arguments[0].value)?;
            let value_op = self.lower_expression(&arguments[1].value)?;

            // Call map_insert(map, key, value)
            let result_local = self
                .builder
                .new_local(Type::primitive(ast::PrimitiveType::Void), false);

            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: result_local,
                    projection: vec![],
                },
                rvalue: Rvalue::Call {
                    func: Operand::Constant(Constant {
                        ty: Type::primitive(ast::PrimitiveType::String),
                        value: ConstantValue::String("map_insert".to_string()),
                    }),
                    explicit_type_arguments: vec![],
                    args: vec![map_op, key_op, value_op],
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
        } else if method_name.name == "get" {
            // map.get(key) -> map_get(map, key)
            let map_op = self.lower_expression(receiver)?;

            if arguments.len() != 1 {
                return Err(SemanticError::ArgumentCountMismatch {
                    function: "map.get".to_string(),
                    expected: 1,
                    found: arguments.len(),
                    location: source_location.clone(),
                });
            }

            let key_op = self.lower_expression(&arguments[0].value)?;

            // Assume integer return for now (need generics for full support)
            let result_local = self
                .builder
                .new_local(Type::primitive(ast::PrimitiveType::Integer), false);

            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: result_local,
                    projection: vec![],
                },
                rvalue: Rvalue::Call {
                    func: Operand::Constant(Constant {
                        ty: Type::primitive(ast::PrimitiveType::String),
                        value: ConstantValue::String("map_get".to_string()),
                    }),
                    explicit_type_arguments: vec![],
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
        } else {
            // For other methods, try to find them in the type system
            // This requires knowing the type of the receiver, which we might not have fully resolved here
            // Fallback: treat as function call "ReceiverType_MethodName(receiver, args)"?
            // Or "MethodName(receiver, args)"?

            // For now, just error
            Err(SemanticError::UnsupportedFeature {
                feature: format!("Method call '{}' not supported yet", method_name.name),
                location: source_location.clone(),
            })
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

        // Handle implicit await for Futures
        let left_op = self.maybe_await_operand(left_op)?;
        let right_op = self.maybe_await_operand(right_op)?;

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

            let result_local = self
                .builder
                .new_local(Type::primitive(PrimitiveType::Boolean), false);
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

            let result_local = self
                .builder
                .new_local(Type::primitive(PrimitiveType::Boolean), false);
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

        let result_local = self
            .builder
            .new_local(Type::primitive(PrimitiveType::Boolean), false);
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

        // Look up captured variables in the CURRENT scope before switching to lambda builder
        let mut capture_operands = Vec::new();
        let mut capture_types = Vec::new();
        for capture in captures {
            if let Some(&local_id) = self.var_map.get(&capture.name.name) {
                let capture_type = self
                    .var_types
                    .get(&capture.name.name)
                    .cloned()
                    .unwrap_or_else(|| Type::primitive(PrimitiveType::Integer));
                capture_operands.push((
                    capture.name.name.clone(),
                    Operand::Copy(Place {
                        local: local_id,
                        projection: vec![],
                    }),
                ));
                capture_types.push((capture.name.name.clone(), capture_type));
            } else {
                return Err(SemanticError::UndefinedSymbol {
                    symbol: capture.name.name.clone(),
                    location: capture.source_location.clone(),
                });
            }
        }

        // Build parameters list: captures first, then regular parameters
        let mut mir_params = Vec::new();
        // Add captures as parameters
        for (name, ty) in &capture_types {
            mir_params.push((format!("__capture_{}", name), ty.clone()));
        }
        // Add regular parameters
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
        let saved_builder = std::mem::take(&mut self.builder);

        // Start building the lambda function with captures as extra parameters
        self.builder.start_function(
            lambda_name.clone(),
            mir_params.clone(),
            mir_return_type.clone(),
        );

        // Set up parameter mappings - first captures, then regular parameters
        if let Some(current_func) = &self.builder.current_function {
            let mut param_idx = 0;
            // Map captures (using original names, not __capture_ prefix)
            for (capture_name, capture_type) in &capture_types {
                let mir_param = &current_func.parameters[param_idx];
                self.var_map
                    .insert(capture_name.clone(), mir_param.local_id);
                self.var_types
                    .insert(capture_name.clone(), capture_type.clone());
                param_idx += 1;
            }
            // Map regular parameters
            for ast_param in parameters {
                let mir_param = &current_func.parameters[param_idx];
                self.var_map
                    .insert(ast_param.name.name.clone(), mir_param.local_id);
                self.var_types
                    .insert(ast_param.name.name.clone(), mir_param.ty.clone());
                param_idx += 1;
            }
        }

        // Create return local if not void
        let return_local = match &mir_return_type {
            Type::Primitive(PrimitiveType::Void) => None,
            _ => {
                let local_id = self.builder.new_local(mir_return_type.clone(), false);
                self.builder
                    .push_statement(Statement::StorageLive(local_id));
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
                        place: Place {
                            local: ret_local,
                            projection: vec![],
                        },
                        rvalue: Rvalue::Use(result),
                        source_info: SourceInfo {
                            span: SourceLocation::unknown(),
                            scope: 0,
                        },
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
        self.program
            .functions
            .insert(lambda_name.clone(), mir_function);

        // Restore previous builder state
        self.builder = saved_builder;
        self.var_map = saved_var_map;
        self.var_types = saved_var_types;
        self.return_local = saved_return_local;
        self.loop_stack = saved_loop_stack;

        // Create closure value with captured operands
        let closure_captures: Vec<Operand> = capture_operands
            .into_iter()
            .map(|(_, operand)| operand)
            .collect();

        // Create closure type (function pointer type) - includes captures and regular params
        let closure_type = Type::Function {
            parameter_types: mir_params.iter().map(|(_, ty)| ty.clone()).collect(),
            return_type: Box::new(mir_return_type),
            is_variadic: false,
        };

        // Create a local for the closure and assign the closure value
        let closure_local = self.builder.new_local(closure_type, false);
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: closure_local,
                projection: vec![],
            },
            rvalue: Rvalue::Closure {
                func_name: lambda_name,
                captures: closure_captures,
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
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
        // Resolve function name and reference
        let function_name = match &call.function_reference {
            ast::FunctionReference::Local { name, .. } => {
                if let Some(mod_name) = &self.current_module {
                    let qualified = format!("{}.{}", mod_name, name.name);

                    // Check if it's an external function (lowered before functions in lower_module)
                    if self.program.external_functions.contains_key(&name.name) {
                        name.name.clone()
                    } else if name.name == "main" {
                        "main".to_string()
                    } else {
                        // Assume it's a local function in this module (forward or backward ref)
                        qualified
                    }
                } else {
                    name.name.clone()
                }
            }
            ast::FunctionReference::Qualified { module, name, .. } => {
                format!("{}.{}", module.name, name.name)
            }
            ast::FunctionReference::External { name, .. } => name.name.clone(),
        };
        eprintln!("lower_function_call: function name = {}", function_name);

        // Determine parameter types and return type
        let (parameter_types, result_type) = if let Some(ext_func) =
            self.program.external_functions.get(&function_name)
        {
            eprintln!(
                "lower_function_call: found external function {}",
                function_name
            );
            (
                Some(ext_func.parameters.clone()),
                ext_func.return_type.clone(),
            )
        } else if let Some(func) = self.program.functions.get(&function_name) {
            eprintln!(
                "lower_function_call: found regular function {}",
                function_name
            );
            let params: Vec<Type> = func.parameters.iter().map(|p| p.ty.clone()).collect();
            (Some(params), func.return_type.clone())
        } else {
            // Check builtin printf
            if function_name == "printf" {
                (None, Type::primitive(ast::PrimitiveType::Integer))
            } else {
                // Try symbol table
                let mut resolved = None;
                if let Some(ref symbol_table) = self.symbol_table {
                    if let Some(symbol) = symbol_table.lookup_symbol(&function_name) {
                        if let Type::Function {
                            parameter_types,
                            return_type,
                            is_variadic: _,
                        } = &symbol.symbol_type
                        {
                            eprintln!(
                                "lower_function_call: found in symbol table {}",
                                function_name
                            );
                            resolved = Some((Some(parameter_types.clone()), *return_type.clone()));
                        }
                    }
                }
                resolved.unwrap_or_else(|| {
                    eprintln!(
                        "lower_function_call: WARNING - function {} not found, using default types",
                        function_name
                    );
                    (None, Type::primitive(ast::PrimitiveType::Integer))
                })
            }
        };

        // Lower arguments
        let mut arg_operands = Vec::new();
        for (i, arg) in call.arguments.iter().enumerate() {
            let mut arg_operand = self.lower_expression(&arg.value)?;

            // Check for implicit cast if expected type is known
            if let Some(ref params) = parameter_types {
                if let Some(expected_type) = params.get(i) {
                    arg_operand = self.ensure_compatible_operand(arg_operand, expected_type)?;
                }
            }

            arg_operands.push(arg_operand);
        }

        // Lower variadic arguments (for functions like printf)
        for arg_expr in &call.variadic_arguments {
            let arg_operand = self.lower_expression(arg_expr)?;
            arg_operands.push(arg_operand);
        }

        // Check if this is a call to a local variable (closure call)
        if let Some(&local_id) = self.var_map.get(&function_name) {
            eprintln!(
                "lower_function_call: {} is a local variable (closure)",
                function_name
            );

            // Get the closure function pointer from the local variable
            let func_operand = Operand::Copy(Place {
                local: local_id,
                projection: vec![],
            });

            // Get the return type from the variable's function type
            let result_type = if let Some(var_type) = self.var_types.get(&function_name) {
                match var_type {
                    Type::Function { return_type, .. } => (**return_type).clone(),
                    _ => Type::primitive(ast::PrimitiveType::Integer),
                }
            } else {
                Type::primitive(ast::PrimitiveType::Integer)
            };

            // Convert explicit type arguments to MIR Types
            let mir_explicit_type_arguments: Result<Vec<Type>, SemanticError> = call
                .explicit_type_arguments
                .iter()
                .map(|ts| self.ast_type_to_mir_type(ts))
                .collect();
            let mir_explicit_type_arguments = mir_explicit_type_arguments?;

            let result_local = self.builder.new_local(result_type, false);

            // Emit call assignment
            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: result_local,
                    projection: vec![],
                },
                rvalue: Rvalue::Call {
                    func: func_operand,
                    explicit_type_arguments: mir_explicit_type_arguments.clone(),
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
        let func_operand = Operand::Constant(Constant {
            ty: Type::primitive(ast::PrimitiveType::String),
            value: ConstantValue::String(function_name.clone()),
        });

        // Convert explicit type arguments to MIR Types
        let mir_explicit_type_arguments: Result<Vec<Type>, SemanticError> = call
            .explicit_type_arguments
            .iter()
            .map(|ts| self.ast_type_to_mir_type(ts))
            .collect();
        let mir_explicit_type_arguments = mir_explicit_type_arguments?;

        let result_local = self.builder.new_local(result_type, false);

        // Emit call assignment
        self.builder.push_statement(Statement::Assign {
            place: Place {
                local: result_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: func_operand,
                explicit_type_arguments: mir_explicit_type_arguments,
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

    /// Resolve field index and type for a struct
    fn resolve_field_index(
        &self,
        struct_type: &Type,
        field_name: &str,
    ) -> Result<(FieldIdx, Type), SemanticError> {
        // Unwrap pointer/owned/reference types
        let mut current_type = struct_type;
        loop {
            match current_type {
                Type::Owned { base_type, .. }
                | Type::Pointer {
                    target_type: base_type,
                    ..
                } => {
                    current_type = base_type;
                }
                _ => break,
            }
        }

        match current_type {
            Type::Named { name, module: _ } => {
                // Need symbol table to look up struct fields
                if let Some(st) = &self.symbol_table {
                    if let Some(type_def) = st.lookup_type_definition(name) {
                        if let crate::types::TypeDefinition::Struct { fields, .. } = type_def {
                            for (idx, (fname, ftype)) in fields.iter().enumerate() {
                                if fname == field_name {
                                    return Ok((idx as u32, ftype.clone()));
                                }
                            }
                            return Err(SemanticError::UnknownField {
                                struct_name: name.clone(),
                                field_name: field_name.to_string(),
                                location: SourceLocation::unknown(),
                            });
                        }
                    }
                }
                // If symbol table not available or type not found (shouldn't happen after semantic analysis)
                Err(SemanticError::UndefinedSymbol {
                    symbol: name.clone(),
                    location: SourceLocation::unknown(),
                })
            }
            _ => Err(SemanticError::TypeMismatch {
                expected: "struct".to_string(),
                found: current_type.to_string(),
                location: SourceLocation::unknown(),
            }),
        }
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
            ast::AssignmentTarget::StructField {
                instance,
                field_name,
            } => {
                // Lower the instance to a place
                let instance_op = self.lower_expression(instance)?;
                let mut place = match instance_op {
                    Operand::Copy(p) | Operand::Move(p) => p,
                    Operand::Constant(c) => {
                        let temp = self.builder.new_local(c.ty.clone(), false);
                        self.builder.push_statement(Statement::Assign {
                            place: Place {
                                local: temp,
                                projection: vec![],
                            },
                            rvalue: Rvalue::Use(Operand::Constant(c)),
                            source_info: SourceInfo {
                                span: SourceLocation::unknown(),
                                scope: 0,
                            },
                        });
                        Place {
                            local: temp,
                            projection: vec![],
                        }
                    }
                };

                // Get the type of the instance
                let instance_type = self.get_expression_type(instance)?;

                // Unwrap pointer/reference types and add Deref projections
                let mut current_type = &instance_type;
                loop {
                    match current_type {
                        Type::Owned { base_type, .. }
                        | Type::Pointer {
                            target_type: base_type,
                            ..
                        } => {
                            place.projection.push(PlaceElem::Deref);
                            current_type = base_type;
                        }
                        _ => break,
                    }
                }

                // Resolve field index
                let (field_idx, field_type) =
                    self.resolve_field_index(&instance_type, &field_name.name)?;

                // Add field projection
                place.projection.push(PlaceElem::Field {
                    field: field_idx,
                    ty: field_type,
                });

                Ok(place)
            }
            ast::AssignmentTarget::ArrayElement { array, index } => {
                let array_op = self.lower_expression(array)?;
                let mut place = match array_op {
                    Operand::Copy(p) | Operand::Move(p) => p,
                    Operand::Constant(_) => {
                        return Err(SemanticError::InvalidOperation {
                            operation: "array assignment".to_string(),
                            reason: "cannot assign to constant array".to_string(),
                            location: SourceLocation::unknown(),
                        })
                    }
                };

                let index_operand = self.lower_expression(index)?;
                let index_local = match index_operand {
                    Operand::Copy(p) | Operand::Move(p) => p.local,
                    Operand::Constant(c) => {
                        let temp = self.builder.new_local(c.ty.clone(), false);
                        self.builder.push_statement(Statement::Assign {
                            place: Place {
                                local: temp,
                                projection: vec![],
                            },
                            rvalue: Rvalue::Use(Operand::Constant(c)),
                            source_info: SourceInfo {
                                span: SourceLocation::unknown(),
                                scope: 0,
                            },
                        });
                        temp
                    }
                };

                place.projection.push(PlaceElem::Index(index_local));
                Ok(place)
            }
            ast::AssignmentTarget::MapValue { map: _, key: _ } => {
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
                ownership,
                ..
            } => {
                let base = self.ast_type_to_mir_type(base_type)?;
                let kind = match ownership {
                    ast::OwnershipKind::Owned => crate::types::OwnershipKind::Owned,
                    ast::OwnershipKind::Borrowed => crate::types::OwnershipKind::Borrowed,
                    ast::OwnershipKind::BorrowedMut => crate::types::OwnershipKind::MutableBorrow,
                    ast::OwnershipKind::Shared => crate::types::OwnershipKind::Shared,
                };
                Ok(Type::Owned {
                    ownership: kind,
                    base_type: Box::new(base),
                })
            }
            ast::TypeSpecifier::TypeParameter {
                name, constraints, ..
            } => {
                // Convert type parameter constraints to TypeConstraintInfo
                let constraint_infos: Vec<crate::types::TypeConstraintInfo> = constraints
                    .iter()
                    .filter_map(|c| match &c.constraint_type {
                        ast::TypeConstraintKind::TraitBound { trait_name } => {
                            Some(crate::types::TypeConstraintInfo::TraitBound {
                                trait_name: trait_name.name.clone(),
                                module: None,
                            })
                        }
                        ast::TypeConstraintKind::NumericBound => {
                            Some(crate::types::TypeConstraintInfo::NumericBound)
                        }
                        ast::TypeConstraintKind::EqualityBound => {
                            Some(crate::types::TypeConstraintInfo::EqualityBound)
                        }
                        ast::TypeConstraintKind::OrderBound => {
                            Some(crate::types::TypeConstraintInfo::OrderBound)
                        }
                        _ => None,
                    })
                    .collect();
                Ok(Type::generic(name.name.clone(), constraint_infos))
            }
            ast::TypeSpecifier::Generic {
                base_type,
                type_arguments,
                ..
            } => {
                // Convert generic instantiation like Vec<Int>
                let args: Result<Vec<Type>, SemanticError> = type_arguments
                    .iter()
                    .map(|arg| self.ast_type_to_mir_type(arg))
                    .collect();
                Ok(Type::generic_instance(
                    base_type.name.clone(),
                    args?,
                    self.current_module.clone(),
                ))
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
                    explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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
                    explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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
        let mut current_type = if let Some(func) = &self.builder.current_function {
            if let Some(local) = func.locals.get(&place.local) {
                local.ty.clone()
            } else {
                return Err(SemanticError::Internal {
                    message: format!("Local {:?} not found", place.local),
                });
            }
        } else {
            return Err(SemanticError::Internal {
                message: "No current function".to_string(),
            });
        };

        for elem in &place.projection {
            match elem {
                PlaceElem::Deref => match current_type {
                    Type::Pointer { target_type, .. }
                    | Type::Owned {
                        base_type: target_type,
                        ..
                    } => {
                        current_type = *target_type.clone();
                    }
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: "pointer type".to_string(),
                            found: current_type.to_string(),
                            location: SourceLocation::unknown(),
                        })
                    }
                },
                PlaceElem::Field { ty, .. } => {
                    current_type = ty.clone();
                }
                PlaceElem::Index(_) => match current_type {
                    Type::Array { element_type, .. } => {
                        current_type = *element_type.clone();
                    }
                    _ => {
                        return Err(SemanticError::TypeMismatch {
                            expected: "array type".to_string(),
                            found: current_type.to_string(),
                            location: SourceLocation::unknown(),
                        })
                    }
                },
                _ => {
                    return Err(SemanticError::UnsupportedFeature {
                        feature: "Non-field place projections".to_string(),
                        location: SourceLocation::unknown(),
                    })
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

    /// Lower field access expression
    fn lower_field_access(
        &mut self,
        instance: &ast::Expression,
        field_name: &ast::Identifier,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Lower the instance to a place (or temp)
        let instance_op = self.lower_expression(instance)?;
        let mut place = match instance_op {
            Operand::Copy(p) | Operand::Move(p) => p,
            Operand::Constant(c) => {
                let temp = self.builder.new_local(c.ty.clone(), false);
                self.builder.push_statement(Statement::Assign {
                    place: Place {
                        local: temp,
                        projection: vec![],
                    },
                    rvalue: Rvalue::Use(Operand::Constant(c)),
                    source_info: SourceInfo {
                        span: source_location.clone(),
                        scope: 0,
                    },
                });
                Place {
                    local: temp,
                    projection: vec![],
                }
            }
        };

        // Get the type of the instance
        let instance_type = self.get_expression_type(instance)?;

        // Unwrap pointer/reference types and add Deref projections
        let mut current_type = &instance_type;
        loop {
            match current_type {
                Type::Owned { base_type, .. }
                | Type::Pointer {
                    target_type: base_type,
                    ..
                } => {
                    place.projection.push(PlaceElem::Deref);
                    current_type = base_type;
                }
                _ => break,
            }
        }

        // Resolve field index
        let (field_idx, field_type) = self.resolve_field_index(&instance_type, &field_name.name)?;

        // Add field projection
        place.projection.push(PlaceElem::Field {
            field: field_idx,
            ty: field_type,
        });

        Ok(Operand::Copy(place))
    }

    /// Lower enum variant construction with known type
    fn lower_enum_variant_with_type(
        &mut self,
        enum_type_name: &str,
        variant_name: &ast::Identifier,
        values: &[ast::Expression],
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Lower the associated values
        let mut operands = Vec::new();
        for val in values {
            operands.push(self.lower_expression(val)?);
        }

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
        values: &[ast::Expression],
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
        self.lower_enum_variant_with_type(&enum_type_name, variant_name, values, source_location)
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
                bindings,
                nested_pattern,
                source_location: _,
            } => {
                // Handle nested pattern
                if let Some(ref nested_pat) = nested_pattern {
                    // For nested patterns, we need to extract the data and then match on it
                    // First, get the type of the variant's associated data
                    let data_type = if let Some(_st) = &self.symbol_table {
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
                            variant_name: _,
                            bindings: inner_bindings,
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

                            // For now, we'll just handle the binding if it exists (first one)
                            // Full nested matching would require generating additional switch statements
                            if let Some(inner_bind) = inner_bindings.first() {
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

                // If there are bindings (without nested pattern), extract the enum variant's associated data
                if !bindings.is_empty() && nested_pattern.is_none() {
                    let associated_types =
                        self.get_enum_variant_associated_types(enum_name, variant_name);

                    for (i, (binding_name, binding_type)) in
                        bindings.iter().zip(associated_types.iter()).enumerate()
                    {
                        // Create a local for the binding
                        let binding_local = self.builder.new_local(binding_type.clone(), false);

                        // Add to var_map and var_types so it can be referenced in the case body
                        self.var_map
                            .insert(binding_name.name.clone(), binding_local);
                        self.var_types
                            .insert(binding_name.name.clone(), binding_type.clone());

                        // Create a projection to access the data field
                        // We use Field indices 1, 2, 3... mapping to data offsets
                        let data_place = Place {
                            local: value_place.local,
                            projection: vec![PlaceElem::Field {
                                field: (i + 1) as u32,
                                ty: binding_type.clone(),
                            }],
                        };

                        // Copy the data to the binding local
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
            ast::Pattern::Struct {
                struct_name,
                fields,
                source_location: _,
            } => {
                for (field_name, field_pattern) in fields {
                    let (field_idx, field_type) =
                        self.get_struct_field_index(&struct_name.name, &field_name.name)?;

                    let field_place = Place {
                        local: value_place.local,
                        projection: value_place
                            .projection
                            .iter()
                            .cloned()
                            .chain(vec![PlaceElem::Field {
                                field: field_idx as u32,
                                ty: field_type,
                            }])
                            .collect(),
                    };

                    self.lower_pattern_bindings(field_pattern, &field_place, 0)?;
                }
            }
        }

        Ok(())
    }

    /// Get the type of an enum variant's associated data
    fn get_enum_variant_type(&self, variant_name: &ast::Identifier) -> Option<Type> {
        if let Some(st) = &self.symbol_table {
            // Search through all enum definitions to find this variant
            for type_def in st.get_type_definitions().values() {
                if let TypeDefinition::Enum { variants, .. } = type_def {
                    for variant in variants {
                        if variant.name == variant_name.name {
                            // For now, return the first associated type or error if multiple/none
                            // This is a temporary fix until we support tuples properly
                            return variant.associated_types.first().cloned();
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
                            // Return first associated type
                            return variant.associated_types.first().cloned();
                        }
                    }
                }
            }
        }

        // Fallback: search all enums for this variant (less precise)
        self.get_enum_variant_type(variant_name)
    }

    /// Get the associated types for a specific enum variant given the enum name
    fn get_enum_variant_associated_types(
        &self,
        enum_name: &Option<ast::Identifier>,
        variant_name: &ast::Identifier,
    ) -> Vec<Type> {
        // First try to look up by enum name if provided
        if let Some(enum_ident) = enum_name {
            // Check type_definitions from program (copied from symbol table)
            if let Some(type_def) = self.program.type_definitions.get(&enum_ident.name) {
                if let TypeDefinition::Enum { variants, .. } = type_def {
                    for variant in variants {
                        if variant.name == variant_name.name {
                            return variant.associated_types.clone();
                        }
                    }
                }
            }
        }

        // Fallback: search all enums for this variant (less precise)
        if let Some(st) = &self.symbol_table {
            for type_def in st.get_type_definitions().values() {
                if let TypeDefinition::Enum { variants, .. } = type_def {
                    for variant in variants {
                        if variant.name == variant_name.name {
                            return variant.associated_types.clone();
                        }
                    }
                }
            }
        }
        vec![]
    }

    /// Get type of an expression
    fn get_expression_type(&self, expr: &ast::Expression) -> Result<Type, SemanticError> {
        match expr {
            ast::Expression::Variable { name, .. } => {
                if let Some(st) = &self.symbol_table {
                    if let Some(symbol) = st.lookup_symbol(&name.name) {
                        return Ok(symbol.symbol_type.clone());
                    }
                }
                // Fallback if no symbol table (should generally not happen in valid code)
                // Or look up in local map if we track types there?
                if let Some(&local_id) = self.var_map.get(&name.name) {
                    if let Some(func) = &self.builder.current_function {
                        if let Some(local) = func.locals.get(&local_id) {
                            return Ok(local.ty.clone());
                        }
                    }
                }

                Err(SemanticError::UndefinedSymbol {
                    symbol: name.name.clone(),
                    location: name.source_location.clone(),
                })
            }
            ast::Expression::FieldAccess {
                instance,
                field_name,
                ..
            } => {
                let instance_type = self.get_expression_type(instance)?;
                let (_, field_type) = self.resolve_field_index(&instance_type, &field_name.name)?;
                Ok(field_type)
            }
            ast::Expression::AddressOf {
                operand,
                mutability,
                ..
            } => {
                let operand_type = self.get_expression_type(operand)?;
                // Use Owned type with Borrowed/MutableBorrow ownership to match semantic analysis
                if *mutability {
                    Ok(Type::Owned {
                        ownership: crate::types::OwnershipKind::MutableBorrow,
                        base_type: Box::new(operand_type),
                    })
                } else {
                    Ok(Type::Owned {
                        ownership: crate::types::OwnershipKind::Borrowed,
                        base_type: Box::new(operand_type),
                    })
                }
            }
            ast::Expression::Dereference { pointer, .. } => {
                let pointer_type = self.get_expression_type(pointer)?;
                match pointer_type {
                    Type::Pointer { target_type, .. }
                    | Type::Owned {
                        base_type: target_type,
                        ..
                    } => Ok(*target_type),
                    _ => Err(SemanticError::TypeMismatch {
                        expected: "pointer type".to_string(),
                        found: pointer_type.to_string(),
                        location: SourceLocation::unknown(),
                    }),
                }
            }
            ast::Expression::IntegerLiteral { .. } => Ok(Type::primitive(PrimitiveType::Integer)),
            ast::Expression::FloatLiteral { .. } => Ok(Type::primitive(PrimitiveType::Float)),
            ast::Expression::BooleanLiteral { .. } => Ok(Type::primitive(PrimitiveType::Boolean)),
            ast::Expression::StringLiteral { .. } => Ok(Type::primitive(PrimitiveType::String)),
            // Add other cases as needed
            _ => Ok(Type::primitive(PrimitiveType::Integer)), // Placeholder/Fallback
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
                explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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

    /// Lower address-of operator
    fn lower_address_of(
        &mut self,
        operand: &ast::Expression,
        mutability: bool,
        source_location: &SourceLocation,
    ) -> Result<Operand, SemanticError> {
        // Check the type of the operand
        let operand_type = self.get_expression_type(operand)?;

        // If it's a reference type (implicitly passed by pointer), &expr just returns the pointer
        let is_reference_type = match &operand_type {
            Type::Named { .. }
            | Type::Array { .. }
            | Type::Map { .. }
            | Type::Pointer { .. }
            | Type::Owned { .. } => true,
            // Strings are primitives but might be treated as reference types depending on implementation.
            // For now, assume Primitives are value types.
            _ => false,
        };

        if is_reference_type {
            // Just return the operand value (which is the pointer)
            self.lower_expression(operand)
        } else {
            // Value type: take address
            let operand_op = self.lower_expression(operand)?;

            let place = match operand_op {
                Operand::Copy(p) | Operand::Move(p) => p,
                Operand::Constant(c) => {
                    // Cannot take address of constant directly, store in temp
                    let temp = self.builder.new_local(c.ty.clone(), false);
                    self.builder.push_statement(Statement::Assign {
                        place: Place {
                            local: temp,
                            projection: vec![],
                        },
                        rvalue: Rvalue::Use(Operand::Constant(c)),
                        source_info: SourceInfo {
                            span: source_location.clone(),
                            scope: 0,
                        },
                    });
                    Place {
                        local: temp,
                        projection: vec![],
                    }
                }
            };

            // Create a temporary for the pointer
            let temp = self
                .builder
                .new_local(Type::pointer(operand_type, mutability), false);

            self.builder.push_statement(Statement::Assign {
                place: Place {
                    local: temp,
                    projection: vec![],
                },
                rvalue: Rvalue::AddressOf(place),
                source_info: SourceInfo {
                    span: source_location.clone(),
                    scope: 0,
                },
            });

            Ok(Operand::Copy(Place {
                local: temp,
                projection: vec![],
            }))
        }
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
        let _target_type = match pointer_type {
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
            projection: [pointer_place.projection.clone(), vec![PlaceElem::Deref]].concat(),
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
                explicit_type_arguments: vec![],
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
                    explicit_type_arguments: vec![],
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
                explicit_type_arguments: vec![],
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

/// Lower an AST program to MIR with symbol table and capture information
pub fn lower_ast_to_mir_with_symbols_and_captures(
    ast_program: &ast::Program,
    symbol_table: SymbolTable,
    captures: HashMap<SourceLocation, std::collections::HashSet<String>>,
) -> Result<Program, SemanticError> {
    let mut context = LoweringContext::with_symbol_table(symbol_table);
    context.set_captures(captures);
    context.lower_program(ast_program)
}

#[cfg(test)]
mod tests;
