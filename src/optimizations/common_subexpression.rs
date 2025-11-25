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

//! Common subexpression elimination optimization pass
//!
//! Eliminates redundant computations by reusing previously computed values

use super::OptimizationPass;
use crate::error::SemanticError;
use crate::mir::{BinOp, Function, LocalId, Operand, Place, Rvalue, Statement, UnOp};
use std::collections::HashMap;

/// Expression representation for CSE
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Expression {
    BinOp {
        op: BinOp,
        left: Operand,
        right: Operand,
    },
    UnOp {
        op: UnOp,
        operand: Operand,
    },
    Use(Operand),
}

impl Expression {
    /// Create an expression from an rvalue
    fn from_rvalue(rvalue: &Rvalue) -> Option<Self> {
        match rvalue {
            Rvalue::Use(operand) => Some(Expression::Use(operand.clone())),
            Rvalue::BinaryOp { op, left, right } => Some(Expression::BinOp {
                op: *op,
                left: left.clone(),
                right: right.clone(),
            }),
            Rvalue::UnaryOp { op, operand } => Some(Expression::UnOp {
                op: *op,
                operand: operand.clone(),
            }),
            _ => None, // Only handle simple expressions for now
        }
    }

    /// Check if the expression uses a given local
    fn uses_local(&self, local: LocalId) -> bool {
        match self {
            Expression::Use(operand) => operand_uses_local(operand, local),
            Expression::BinOp { left, right, .. } => {
                operand_uses_local(left, local) || operand_uses_local(right, local)
            }
            Expression::UnOp { operand, .. } => operand_uses_local(operand, local),
        }
    }
}

/// Check if an operand uses a given local
fn operand_uses_local(operand: &Operand, local: LocalId) -> bool {
    match operand {
        Operand::Copy(place) | Operand::Move(place) => place.local == local,
        Operand::Constant(_) => false,
    }
}

/// Common subexpression elimination optimization pass
pub struct CommonSubexpressionEliminationPass {
    eliminated_expressions: usize,
}

impl CommonSubexpressionEliminationPass {
    pub fn new() -> Self {
        Self {
            eliminated_expressions: 0,
        }
    }

    /// Perform CSE within a single basic block
    fn eliminate_in_block(&mut self, statements: &mut Vec<Statement>) -> bool {
        let mut changed = false;
        let mut available_expressions: HashMap<Expression, LocalId> = HashMap::new();

        for statement in statements.iter_mut() {
            match statement {
                Statement::Assign { place, rvalue, .. } => {
                    // Invalidate expressions that use the assigned local
                    available_expressions.retain(|expr, _| !expr.uses_local(place.local));

                    // Check if this expression is already available
                    if let Some(expr) = Expression::from_rvalue(rvalue) {
                        if let Some(&existing_local) = available_expressions.get(&expr) {
                            // Replace with use of existing computation
                            *rvalue = Rvalue::Use(Operand::Copy(Place {
                                local: existing_local,
                                projection: vec![],
                            }));
                            changed = true;
                            self.eliminated_expressions += 1;
                        } else {
                            // Record this expression as available
                            available_expressions.insert(expr, place.local);
                        }
                    }
                }

                Statement::StorageDead(local) => {
                    // Remove expressions computed by this local
                    available_expressions.retain(|_, &mut computed_by| computed_by != *local);
                    // Also remove expressions that use this local
                    available_expressions.retain(|expr, _| !expr.uses_local(*local));
                }

                _ => {}
            }
        }

        changed
    }
}

impl OptimizationPass for CommonSubexpressionEliminationPass {
    fn name(&self) -> &'static str {
        "common-subexpression-elimination"
    }

    fn run_on_function(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        let mut changed = false;

        // Run CSE on each basic block independently
        for block in function.basic_blocks.values_mut() {
            changed |= self.eliminate_in_block(&mut block.statements);
        }

        Ok(changed)
    }
}

impl Default for CommonSubexpressionEliminationPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;
    use crate::error::SourceLocation;
    use crate::mir::{BinOp, Builder, Constant, ConstantValue, Place, SourceInfo};
    use crate::types::Type;

    #[test]
    fn test_common_subexpression_elimination() {
        let mut pass = CommonSubexpressionEliminationPass::new();
        let mut builder = Builder::new();

        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );

        let a = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        let b = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        let temp1 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        let temp2 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);

        // a = 10
        builder.push_statement(Statement::Assign {
            place: Place {
                local: a,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(10),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // b = 20
        builder.push_statement(Statement::Assign {
            place: Place {
                local: b,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(20),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // temp1 = a + b
        builder.push_statement(Statement::Assign {
            place: Place {
                local: temp1,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Add,
                left: Operand::Copy(Place {
                    local: a,
                    projection: vec![],
                }),
                right: Operand::Copy(Place {
                    local: b,
                    projection: vec![],
                }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // temp2 = a + b  (common subexpression)
        builder.push_statement(Statement::Assign {
            place: Place {
                local: temp2,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Add,
                left: Operand::Copy(Place {
                    local: a,
                    projection: vec![],
                }),
                right: Operand::Copy(Place {
                    local: b,
                    projection: vec![],
                }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        let mut function = builder.finish_function();

        // Run CSE
        let changed = pass.run_on_function(&mut function).unwrap();
        assert!(changed);
        assert_eq!(pass.eliminated_expressions, 1);

        // Check that the second addition was replaced with a copy
        let block = function.basic_blocks.values().next().unwrap();
        let last_stmt = &block.statements[3];

        if let Statement::Assign { rvalue, .. } = last_stmt {
            match rvalue {
                Rvalue::Use(Operand::Copy(place)) => {
                    assert_eq!(place.local, temp1);
                }
                _ => panic!("Expected copy of temp1 after CSE"),
            }
        } else {
            panic!("Expected assignment statement");
        }
    }

    #[test]
    fn test_cse_with_invalidation() {
        let mut pass = CommonSubexpressionEliminationPass::new();
        let mut builder = Builder::new();

        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );

        let a = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        let b = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        let temp1 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        let temp2 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);

        // a = 10
        builder.push_statement(Statement::Assign {
            place: Place {
                local: a,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(10),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // b = 20
        builder.push_statement(Statement::Assign {
            place: Place {
                local: b,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(20),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // temp1 = a + b
        builder.push_statement(Statement::Assign {
            place: Place {
                local: temp1,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Add,
                left: Operand::Copy(Place {
                    local: a,
                    projection: vec![],
                }),
                right: Operand::Copy(Place {
                    local: b,
                    projection: vec![],
                }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // a = 30  (invalidates previous expression)
        builder.push_statement(Statement::Assign {
            place: Place {
                local: a,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(30),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        // temp2 = a + b  (not a common subexpression due to invalidation)
        builder.push_statement(Statement::Assign {
            place: Place {
                local: temp2,
                projection: vec![],
            },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Add,
                left: Operand::Copy(Place {
                    local: a,
                    projection: vec![],
                }),
                right: Operand::Copy(Place {
                    local: b,
                    projection: vec![],
                }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        let mut function = builder.finish_function();

        // Run CSE
        let changed = pass.run_on_function(&mut function).unwrap();
        // Should not change anything since the expression was invalidated
        assert!(!changed);
        assert_eq!(pass.eliminated_expressions, 0);
    }
}
