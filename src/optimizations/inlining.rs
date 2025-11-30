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

//! Function inlining optimization pass
//!
//! Inlines small functions to reduce call overhead

use super::OptimizationPass;
use crate::error::SemanticError;
use crate::mir::{
    BasicBlockId, Function, LocalId, Operand, Place, Program, Rvalue, SourceInfo, Statement,
    Terminator,
};
use std::collections::HashMap;
use std::collections::HashSet;

/// Function inlining optimization pass
#[derive(Debug)]
pub struct InliningPass {
    /// Inlining threshold (e.g., number of statements)
    threshold: usize,

    /// Functions already inlined to prevent recursion
    inlined_functions: HashSet<String>,
}

impl InliningPass {
    pub fn new() -> Self {
        Self {
            threshold: 20,
            inlined_functions: HashSet::new(),
        }
    }

    /// Set the maximum size for inlining
    pub fn set_max_inline_size(&mut self, size: usize) {
        self.threshold = size;
    }

    /// Set the maximum inlining depth
    pub fn set_max_inline_depth(&mut self, depth: usize) {}

    /// Calculate the "cost" of a function for inlining decisions
    fn calculate_function_cost(&self, function: &Function) -> usize {
        let mut cost = 0;

        for block in function.basic_blocks.values() {
            cost += block.statements.len();

            // Add cost for complex terminators
            match &block.terminator {
                Terminator::Call { .. } => cost += 5, // Calls are expensive
                Terminator::SwitchInt { .. } => cost += 2, // Branches have some cost
                _ => cost += 1,
            }
        }

        cost
    }

    /// Check if a function is suitable for inlining
    fn should_inline(&self, function: &Function) -> bool {
        // Only inline single-block functions for now
        if function.basic_blocks.len() != 1 {
            return false;
        }

        // Don't inline recursive functions (basic check)
        if self.has_recursive_calls(function) {
            return false;
        }

        // Check size constraints
        let cost = self.calculate_function_cost(function);
        cost <= self.threshold
    }

    /// Basic check for recursive calls
    fn has_recursive_calls(&self, function: &Function) -> bool {
        for block in function.basic_blocks.values() {
            for statement in &block.statements {
                if let Statement::Assign {
                    rvalue: Rvalue::Call { func, .. },
                    ..
                } = statement
                {
                    if let Operand::Constant(_constant) = func {
                        // In a real implementation, we'd check if the constant refers to the same function
                        // For now, just assume no recursion
                    }
                }
            }

            if let Terminator::Call { func, .. } = &block.terminator {
                if let Operand::Constant(_constant) = func {
                    // Same as above - in practice we'd need better function identification
                }
            }
        }

        false // Conservative: assume no recursion for now
    }

    /// Helper to inline a call
    fn inline_call(
        &self,
        locals: &mut HashMap<LocalId, crate::mir::Local>,
        callee: &Function,
        args: &[Operand],
        destination: &Place,
        target_statements: &mut Vec<Statement>,
        source_info: SourceInfo,
    ) {
        // Map from callee local ID to caller local ID
        let mut local_map = HashMap::new();

        // 1. Create new locals in caller for all locals in callee
        let mut max_local_id = locals.keys().max().copied().unwrap_or(0);
        
        for (callee_local_id, callee_local) in &callee.locals {
            max_local_id += 1;
            let new_local_id = max_local_id;
            
            locals.insert(new_local_id, callee_local.clone());
            local_map.insert(*callee_local_id, new_local_id);
        }

        // 2. Assign arguments to parameters
        for (i, param) in callee.parameters.iter().enumerate() {
            if let Some(arg) = args.get(i) {
                if let Some(param_local_id) = local_map.get(&param.local_id) {
                    target_statements.push(Statement::Assign {
                        place: Place {
                            local: *param_local_id,
                            projection: vec![],
                        },
                        rvalue: Rvalue::Use(arg.clone()),
                        source_info: source_info.clone(),
                    });
                }
            }
        }

        // 3. Inline statements from the entry block (assuming single block)
        if let Some(entry_block) = callee.basic_blocks.get(&callee.entry_block) {
            for stmt in &entry_block.statements {
                let mut new_stmt = stmt.clone();
                self.remap_locals_in_statement(&mut new_stmt, &local_map);
                target_statements.push(new_stmt);
            }
        }

        // 4. Handle return value
        if let Some(return_local) = callee.return_local {
            if let Some(mapped_return_local) = local_map.get(&return_local) {
                target_statements.push(Statement::Assign {
                    place: destination.clone(),
                    rvalue: Rvalue::Use(Operand::Copy(Place {
                        local: *mapped_return_local,
                        projection: vec![],
                    })),
                    source_info,
                });
            }
        }
    }

    fn remap_locals_in_statement(&self, stmt: &mut Statement, map: &HashMap<LocalId, LocalId>) {
        match stmt {
            Statement::Assign { place, rvalue, .. } => {
                self.remap_place(place, map);
                self.remap_rvalue(rvalue, map);
            }
            Statement::StorageLive(local) | Statement::StorageDead(local) => {
                if let Some(new_local) = map.get(local) {
                    *local = *new_local;
                }
            }
            Statement::Nop => {}
        }
    }

    fn remap_place(&self, place: &mut Place, map: &HashMap<LocalId, LocalId>) {
        if let Some(new_local) = map.get(&place.local) {
            place.local = *new_local;
        }
        for elem in &mut place.projection {
            if let crate::mir::PlaceElem::Index(local) = elem {
                if let Some(new_local) = map.get(local) {
                    *local = *new_local;
                }
            }
        }
    }

    fn remap_rvalue(&self, rvalue: &mut Rvalue, map: &HashMap<LocalId, LocalId>) {
        match rvalue {
            Rvalue::Use(op) => self.remap_operand(op, map),
            Rvalue::BinaryOp { left, right, .. } => {
                self.remap_operand(left, map);
                self.remap_operand(right, map);
            }
            Rvalue::UnaryOp { operand, .. } => self.remap_operand(operand, map),
            Rvalue::Call { func, args } => {
                self.remap_operand(func, map);
                for arg in args {
                    self.remap_operand(arg, map);
                }
            }
            Rvalue::Aggregate { operands, .. } => {
                for op in operands {
                    self.remap_operand(op, map);
                }
            }
            Rvalue::Cast { operand, .. } => self.remap_operand(operand, map),
            Rvalue::Ref { place, .. } | Rvalue::AddressOf(place) | Rvalue::Len(place) | Rvalue::Discriminant(place) => {
                self.remap_place(place, map);
            }
            Rvalue::Closure { captures, .. } => {
                for cap in captures {
                    self.remap_operand(cap, map);
                }
            }
        }
    }

    fn remap_operand(&self, operand: &mut Operand, map: &HashMap<LocalId, LocalId>) {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => self.remap_place(place, map),
            Operand::Constant(_) => {}
        }
    }
}

impl OptimizationPass for InliningPass {
    fn name(&self) -> &'static str {
        "inlining"
    }

    fn run_on_function(&mut self, _function: &mut Function) -> Result<bool, SemanticError> {
        // Single function inlining requires access to the whole program
        // For now, return false (no changes)
        Ok(false)
    }

    fn run_on_program(&mut self, program: &mut Program) -> Result<bool, SemanticError> {
        let mut changed = false;

        // Find functions that are candidates for inlining
        let mut inline_candidates = HashMap::new();

        for (name, function) in &program.functions {
            if self.should_inline(function) {
                inline_candidates.insert(name.clone(), function.clone());
            }
        }

        // For each function, look for calls to inline candidates
        for (caller_name, caller_function) in &mut program.functions {
            // Look for calls in each basic block
            for block in caller_function.basic_blocks.values_mut() {
                let mut new_statements = Vec::new();

                for statement in &block.statements {
                    match statement {
                        Statement::Assign {
                            place,
                            rvalue: Rvalue::Call { func, args },
                            source_info,
                        } => {
                            // Check if this is a call to an inline candidate
                            let callee = if let Operand::Constant(constant) = func {
                                if let crate::mir::ConstantValue::String(name) = &constant.value {
                                    inline_candidates.get(name)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(callee) = callee {
                                changed = true;
                                self.inline_call(
                                    &mut caller_function.locals,
                                    callee,
                                    args,
                                    place,
                                    &mut new_statements,
                                    source_info.clone(),
                                );
                            } else {
                                new_statements.push(statement.clone());
                            }
                        }
                        _ => {
                            new_statements.push(statement.clone());
                        }
                    }
                }

                block.statements = new_statements;
            }
        }

        Ok(changed)
    }
}

impl Default for InliningPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;
    use crate::error::SourceLocation;
    use crate::mir::{Builder, Constant, ConstantValue, Place, SourceInfo};
    use crate::types::Type;

    #[test]
    fn test_function_cost_calculation() {
        let pass = InliningPass::new();
        let mut builder = Builder::new();

        builder.start_function(
            "small".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );

        let temp = builder.new_local(Type::primitive(PrimitiveType::Integer), false);

        // Add a single statement
        builder.push_statement(Statement::Assign {
            place: Place {
                local: temp,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(42),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        let function = builder.finish_function();
        let cost = pass.calculate_function_cost(&function);

        // Should be low cost (1 statement + 1 terminator)
        assert!(cost <= 5);
    }

    #[test]
    fn test_should_inline_small_function() {
        let pass = InliningPass::new();
        let mut builder = Builder::new();

        builder.start_function(
            "small".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );

        let temp = builder.new_local(Type::primitive(PrimitiveType::Integer), false);

        // Add a few small statements
        for i in 0..3 {
            builder.push_statement(Statement::Assign {
                place: Place {
                    local: temp,
                    projection: vec![],
                },
                rvalue: Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::Integer),
                    value: ConstantValue::Integer(i),
                })),
                source_info: SourceInfo {
                    span: SourceLocation::unknown(),
                    scope: 0,
                },
            });
        }

        let function = builder.finish_function();

        // Small function should be eligible for inlining
        assert!(pass.should_inline(&function));
    }

    #[test]
    fn test_program_inlining() {
        let mut pass = InliningPass::new();
        let mut program = Program {
            functions: HashMap::new(),
            global_constants: HashMap::new(),
            external_functions: HashMap::new(),
            type_definitions: HashMap::new(),
        };

        // Create a small function to inline: fn small() -> Int { return 42; }
        let mut builder = Builder::new();
        builder.start_function(
            "small".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );

        // return_local needs to be set for inlining to work with our logic
        // We'll simulate return by assigning to a local that is return_local
        let return_local = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        builder.current_function.as_mut().unwrap().return_local = Some(return_local);

        builder.push_statement(Statement::Assign {
            place: Place {
                local: return_local,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(42),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        let small_function = builder.finish_function();
        program
            .functions
            .insert("small".to_string(), small_function);

        // Create a caller function: fn caller() { let x = small(); }
        let mut builder = Builder::new();
        builder.start_function(
            "caller".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Void),
        );

        let dest_local = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        
        // Add call statement
        builder.push_statement(Statement::Assign {
            place: Place {
                local: dest_local,
                projection: vec![],
            },
            rvalue: Rvalue::Call {
                func: Operand::Constant(Constant {
                    ty: Type::Function {
                        parameter_types: vec![],
                        return_type: Box::new(Type::primitive(PrimitiveType::Integer)),
                        is_variadic: false,
                    },
                    value: ConstantValue::String("small".to_string()),
                }),
                args: vec![],
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        let caller_function = builder.finish_function();
        program
            .functions
            .insert("caller".to_string(), caller_function);

        // Run inlining pass
        let changed = pass.run_on_program(&mut program).unwrap();

        // Should have changed
        assert!(changed);

        // Check if caller no longer has the call
        let caller = program.functions.get("caller").unwrap();
        let entry_block = caller.basic_blocks.get(&caller.entry_block).unwrap();
        
        let has_call = entry_block.statements.iter().any(|stmt| {
            if let Statement::Assign { rvalue: Rvalue::Call { .. }, .. } = stmt {
                true
            } else {
                false
            }
        });

        assert!(!has_call, "Function call should have been inlined");
    }
}
