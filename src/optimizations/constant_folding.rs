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

//! Constant folding optimization pass
//! 
//! Evaluates constant expressions at compile time

use super::OptimizationPass;
use crate::mir::{
    Function, Statement, Rvalue, Operand, Constant, ConstantValue, BinOp, UnOp,
};
use crate::types::Type;
use crate::ast::PrimitiveType;
use crate::error::SemanticError;

/// Constant folding optimization pass
pub struct ConstantFoldingPass {
    changed: bool,
}

impl ConstantFoldingPass {
    pub fn new() -> Self {
        Self { changed: false }
    }
    
    /// Fold a binary operation on constants
    fn fold_binary_op(
        &self,
        op: BinOp,
        left: &ConstantValue,
        right: &ConstantValue,
    ) -> Option<ConstantValue> {
        match (left, right) {
            // Integer operations
            (ConstantValue::Integer(l), ConstantValue::Integer(r)) => {
                match op {
                    BinOp::Add => Some(ConstantValue::Integer(l.wrapping_add(*r))),
                    BinOp::Sub => Some(ConstantValue::Integer(l.wrapping_sub(*r))),
                    BinOp::Mul => Some(ConstantValue::Integer(l.wrapping_mul(*r))),
                    BinOp::Div if *r != 0 => Some(ConstantValue::Integer(l / r)),
                    BinOp::Rem if *r != 0 => Some(ConstantValue::Integer(l % r)),
                    BinOp::Eq => Some(ConstantValue::Bool(l == r)),
                    BinOp::Ne => Some(ConstantValue::Bool(l != r)),
                    BinOp::Lt => Some(ConstantValue::Bool(l < r)),
                    BinOp::Le => Some(ConstantValue::Bool(l <= r)),
                    BinOp::Gt => Some(ConstantValue::Bool(l > r)),
                    BinOp::Ge => Some(ConstantValue::Bool(l >= r)),
                    BinOp::BitAnd => Some(ConstantValue::Integer(l & r)),
                    BinOp::BitOr => Some(ConstantValue::Integer(l | r)),
                    BinOp::BitXor => Some(ConstantValue::Integer(l ^ r)),
                    BinOp::Shl => Some(ConstantValue::Integer(l << (r & 63))), // Mask to prevent overflow
                    BinOp::Shr => Some(ConstantValue::Integer(l >> (r & 63))),
                    _ => None,
                }
            }
            
            // Float operations
            (ConstantValue::Float(l), ConstantValue::Float(r)) => {
                match op {
                    BinOp::Add => Some(ConstantValue::Float(l + r)),
                    BinOp::Sub => Some(ConstantValue::Float(l - r)),
                    BinOp::Mul => Some(ConstantValue::Float(l * r)),
                    BinOp::Div if *r != 0.0 => Some(ConstantValue::Float(l / r)),
                    BinOp::Eq => Some(ConstantValue::Bool((l - r).abs() < f64::EPSILON)),
                    BinOp::Ne => Some(ConstantValue::Bool((l - r).abs() >= f64::EPSILON)),
                    BinOp::Lt => Some(ConstantValue::Bool(l < r)),
                    BinOp::Le => Some(ConstantValue::Bool(l <= r)),
                    BinOp::Gt => Some(ConstantValue::Bool(l > r)),
                    BinOp::Ge => Some(ConstantValue::Bool(l >= r)),
                    _ => None,
                }
            }
            
            // Boolean operations
            (ConstantValue::Bool(l), ConstantValue::Bool(r)) => {
                match op {
                    BinOp::Eq => Some(ConstantValue::Bool(l == r)),
                    BinOp::Ne => Some(ConstantValue::Bool(l != r)),
                    BinOp::BitAnd => Some(ConstantValue::Bool(*l && *r)),
                    BinOp::BitOr => Some(ConstantValue::Bool(*l || *r)),
                    BinOp::BitXor => Some(ConstantValue::Bool(*l ^ *r)),
                    _ => None,
                }
            }
            
            // String operations
            (ConstantValue::String(l), ConstantValue::String(r)) => {
                match op {
                    BinOp::Eq => Some(ConstantValue::Bool(l == r)),
                    BinOp::Ne => Some(ConstantValue::Bool(l != r)),
                    BinOp::Add => Some(ConstantValue::String(format!("{}{}", l, r))),
                    _ => None,
                }
            }
            
            _ => None,
        }
    }
    
    /// Fold a unary operation on a constant
    fn fold_unary_op(&self, op: UnOp, operand: &ConstantValue) -> Option<ConstantValue> {
        match (op, operand) {
            (UnOp::Not, ConstantValue::Bool(b)) => Some(ConstantValue::Bool(!b)),
            (UnOp::Neg, ConstantValue::Integer(i)) => Some(ConstantValue::Integer(-i)),
            (UnOp::Neg, ConstantValue::Float(f)) => Some(ConstantValue::Float(-f)),
            _ => None,
        }
    }
    
    /// Get the result type for a binary operation
    fn get_binary_result_type(&self, op: BinOp, left_ty: &Type) -> Type {
        match op {
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                Type::primitive(PrimitiveType::Boolean)
            }
            _ => left_ty.clone(),
        }
    }
    
    /// Optimize an rvalue
    fn optimize_rvalue(&mut self, rvalue: &mut Rvalue) {
        match rvalue {
            Rvalue::BinaryOp { op, left, right } => {
                if let (Operand::Constant(left_const), Operand::Constant(right_const)) = (left, right) {
                    if let Some(result) = self.fold_binary_op(*op, &left_const.value, &right_const.value) {
                        let result_type = self.get_binary_result_type(*op, &left_const.ty);
                        *rvalue = Rvalue::Use(Operand::Constant(Constant {
                            ty: result_type,
                            value: result,
                        }));
                        self.changed = true;
                    }
                }
            }
            
            Rvalue::UnaryOp { op, operand } => {
                if let Operand::Constant(const_operand) = operand {
                    if let Some(result) = self.fold_unary_op(*op, &const_operand.value) {
                        *rvalue = Rvalue::Use(Operand::Constant(Constant {
                            ty: const_operand.ty.clone(),
                            value: result,
                        }));
                        self.changed = true;
                    }
                }
            }
            
            _ => {}
        }
    }
}

impl OptimizationPass for ConstantFoldingPass {
    fn name(&self) -> &'static str {
        "constant-folding"
    }
    
    fn run_on_function(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        self.changed = false;
        
        for block in function.basic_blocks.values_mut() {
            for statement in &mut block.statements {
                if let Statement::Assign { rvalue, .. } = statement {
                    self.optimize_rvalue(rvalue);
                }
            }
        }
        
        Ok(self.changed)
    }
}

impl Default for ConstantFoldingPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{Builder, Place, SourceInfo};
    use crate::error::SourceLocation;
    
    #[test]
    fn test_integer_constant_folding() {
        let mut pass = ConstantFoldingPass::new();
        let mut builder = Builder::new();
        
        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );
        
        let temp = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        
        // Add statement: temp = 2 + 3
        builder.push_statement(Statement::Assign {
            place: Place { local: temp, projection: vec![] },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::Add,
                left: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::Integer),
                    value: ConstantValue::Integer(2),
                }),
                right: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::Integer),
                    value: ConstantValue::Integer(3),
                }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });
        
        let mut function = builder.finish_function();
        
        // Run constant folding
        let changed = pass.run_on_function(&mut function).unwrap();
        assert!(changed);
        
        // Check that the operation was folded to a constant
        let block = function.basic_blocks.values().next().unwrap();
        let stmt = &block.statements[0];
        
        if let Statement::Assign { rvalue, .. } = stmt {
            if let Rvalue::Use(Operand::Constant(constant)) = rvalue {
                assert_eq!(constant.value, ConstantValue::Integer(5));
            } else {
                panic!("Expected constant after folding");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }
    
    #[test]
    fn test_boolean_constant_folding() {
        let mut pass = ConstantFoldingPass::new();
        let mut builder = Builder::new();
        
        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Boolean),
        );
        
        let temp = builder.new_local(Type::primitive(PrimitiveType::Boolean), false);
        
        // Add statement: temp = true && false
        builder.push_statement(Statement::Assign {
            place: Place { local: temp, projection: vec![] },
            rvalue: Rvalue::BinaryOp {
                op: BinOp::BitAnd,
                left: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::Boolean),
                    value: ConstantValue::Bool(true),
                }),
                right: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::Boolean),
                    value: ConstantValue::Bool(false),
                }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });
        
        let mut function = builder.finish_function();
        
        // Run constant folding
        let changed = pass.run_on_function(&mut function).unwrap();
        assert!(changed);
        
        // Check that the operation was folded to false
        let block = function.basic_blocks.values().next().unwrap();
        let stmt = &block.statements[0];
        
        if let Statement::Assign { rvalue, .. } = stmt {
            if let Rvalue::Use(Operand::Constant(constant)) = rvalue {
                assert_eq!(constant.value, ConstantValue::Bool(false));
            } else {
                panic!("Expected constant after folding");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }
    
    #[test]
    fn test_unary_constant_folding() {
        let mut pass = ConstantFoldingPass::new();
        let mut builder = Builder::new();
        
        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );
        
        let temp = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        
        // Add statement: temp = -42
        builder.push_statement(Statement::Assign {
            place: Place { local: temp, projection: vec![] },
            rvalue: Rvalue::UnaryOp {
                op: UnOp::Neg,
                operand: Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::Integer),
                    value: ConstantValue::Integer(42),
                }),
            },
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });
        
        let mut function = builder.finish_function();
        
        // Run constant folding
        let changed = pass.run_on_function(&mut function).unwrap();
        assert!(changed);
        
        // Check that the operation was folded to -42
        let block = function.basic_blocks.values().next().unwrap();
        let stmt = &block.statements[0];
        
        if let Statement::Assign { rvalue, .. } = stmt {
            if let Rvalue::Use(Operand::Constant(constant)) = rvalue {
                assert_eq!(constant.value, ConstantValue::Integer(-42));
            } else {
                panic!("Expected constant after folding");
            }
        } else {
            panic!("Expected assignment statement");
        }
    }
}