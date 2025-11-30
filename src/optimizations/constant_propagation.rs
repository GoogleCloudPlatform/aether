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

//! Constant propagation optimization pass
//!
//! Propagates constant values of variables to their uses

use super::OptimizationPass;
use crate::error::SemanticError;
use crate::mir::{ConstantValue, Function, LocalId, Operand, Rvalue, Statement};
use std::collections::HashMap;

/// Constant propagation optimization pass
pub struct ConstantPropagationPass {
    changed: bool,
    propagated_count: usize,
}

impl ConstantPropagationPass {
    pub fn new() -> Self {
        Self { 
            changed: false,
            propagated_count: 0,
        }
    }

    /// Run constant propagation on a basic block
    fn run_on_block(&mut self, block_statements: &mut Vec<Statement>) {
        // Map of local variables to their known constant values in the current block
        // Note: This is simple intra-block propagation.
        let mut known_constants: HashMap<LocalId, ConstantValue> = HashMap::new();

        for statement in block_statements {
            match statement {
                Statement::Assign { place, rvalue, .. } => {
                    // First, try to propagate constants into the rvalue
                    self.propagate_in_rvalue(rvalue, &known_constants);

                    // Then check if this assignment makes the target local a constant
                    if let Rvalue::Use(Operand::Constant(constant)) = rvalue {
                        // We only track simple variables, not projections (fields/indices)
                        if place.projection.is_empty() {
                            known_constants.insert(place.local, constant.value.clone());
                        }
                    } else {
                        // If assigned something else, invalidate any previous constant value for this local
                        if place.projection.is_empty() {
                            known_constants.remove(&place.local);
                        }
                    }
                }
                // Other statements might use locals too, but for now we focus on assignments
                _ => {}
            }
        }
    }

    fn propagate_in_rvalue(&mut self, rvalue: &mut Rvalue, constants: &HashMap<LocalId, ConstantValue>) {
        match rvalue {
            Rvalue::Use(operand) => self.propagate_in_operand(operand, constants),
            Rvalue::UnaryOp { operand, .. } => self.propagate_in_operand(operand, constants),
            Rvalue::BinaryOp { left, right, .. } => {
                self.propagate_in_operand(left, constants);
                self.propagate_in_operand(right, constants);
            }
            Rvalue::Cast { operand, .. } => self.propagate_in_operand(operand, constants),
            // Add other Rvalue variants as needed
            _ => {}
        }
    }

    fn propagate_in_operand(&mut self, operand: &mut Operand, constants: &HashMap<LocalId, ConstantValue>) {
        if let Operand::Copy(place) | Operand::Move(place) = operand {
            if place.projection.is_empty() {
                if let Some(const_val) = constants.get(&place.local) {
                    // Determine type from somewhere? 
                    // Ideally we need the type of the local to create a Constant.
                    // But Operand::Constant needs a Type. 
                    // Since we don't have easy access to function locals types here without passing function context,
                    // we might need to change the signature or strategy.
                    // However, ConstantValue usually implies the type (Integer, Float, etc).
                    // We can construct a Type from ConstantValue for primitive types.
                    
                    // Let's construct the type from the value
                    let ty = match const_val {
                        ConstantValue::Integer(_) => crate::types::Type::primitive(crate::ast::PrimitiveType::Integer),
                        ConstantValue::Float(_) => crate::types::Type::primitive(crate::ast::PrimitiveType::Float),
                        ConstantValue::Bool(_) => crate::types::Type::primitive(crate::ast::PrimitiveType::Boolean),
                        ConstantValue::String(_) => crate::types::Type::primitive(crate::ast::PrimitiveType::String),
                        // For Char, we need to handle it if ConstantValue supports it (it doesn't seem to currently?)
                        // Assuming ConstantValue definition from constant_folding.rs/mir/mod.rs
                        _ => return, 
                    };

                    *operand = Operand::Constant(crate::mir::Constant {
                        ty,
                        value: const_val.clone(),
                    });
                    self.changed = true;
                    self.propagated_count += 1;
                }
            }
        }
    }
}

impl OptimizationPass for ConstantPropagationPass {
    fn name(&self) -> &'static str {
        "constant-propagation"
    }

    fn run_on_function(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        self.changed = false;
        
        // We need to iterate over blocks. 
        // Since we are doing intra-block propagation, we don't need flow graph analysis yet.
        // However, we need access to function.locals to get types if we wanted to be 100% correct,
        // but inferring from value is okay for primitives.

        // To properly support `run_on_block` needing mutable access to statements 
        // while `propagate_in_operand` might need info, we structure it carefully.
        
        for block in function.basic_blocks.values_mut() {
            self.run_on_block(&mut block.statements);
        }

        Ok(self.changed)
    }
}

impl Default for ConstantPropagationPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;
    use crate::error::SourceLocation;
    use crate::mir::{Builder, Constant, ConstantValue, Operand, Place, Rvalue, SourceInfo, Statement};
    use crate::types::Type;

    #[test]
    fn test_constant_propagation() {
        let mut pass = ConstantPropagationPass::new();
        let mut builder = Builder::new();

        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );

        let x = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        let y = builder.new_local(Type::primitive(PrimitiveType::Integer), false);

        // x = 42
        builder.push_statement(Statement::Assign {
            place: Place { local: x, projection: vec![] },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(42),
            })),
            source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
        });

        // y = x
        builder.push_statement(Statement::Assign {
            place: Place { local: y, projection: vec![] },
            rvalue: Rvalue::Use(Operand::Copy(Place { local: x, projection: vec![] })),
            source_info: SourceInfo { span: SourceLocation::unknown(), scope: 0 },
        });

        let mut function = builder.finish_function();

        // Run constant propagation
        let changed = pass.run_on_function(&mut function).unwrap();
        assert!(changed);
        assert_eq!(pass.propagated_count, 1);

        // Check that y = x became y = 42
        let block = function.basic_blocks.values().next().unwrap();
        let stmt = &block.statements[1];
        
        if let Statement::Assign { rvalue, .. } = stmt {
            if let Rvalue::Use(Operand::Constant(constant)) = rvalue {
                assert_eq!(constant.value, ConstantValue::Integer(42));
            } else {
                panic!("Expected constant after propagation");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }
}
