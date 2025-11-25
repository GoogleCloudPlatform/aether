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

//! Convert enhanced contract expressions to SMT formulas
//!
//! This module bridges the gap between our LLM-first contract expressions
//! and the SMT solver's formula representation.

use crate::types::Type;
use crate::verification::contracts::{
    AggregateOp, BinaryOp, ConstantValue, Expression, QuantifierKind, TemporalOp, UnaryOp,
};
use crate::verification::solver::Formula;
use std::collections::HashMap;

/// Convert a contract expression to an SMT formula
pub fn expression_to_formula(expr: &Expression) -> Result<Formula, String> {
    match expr {
        Expression::Variable(name) => Ok(Formula::Var(name.clone())),

        Expression::Constant(c) => match c {
            ConstantValue::Integer(n) => Ok(Formula::Int(*n)),
            ConstantValue::Float(f) => Ok(Formula::Real(*f)),
            ConstantValue::Boolean(b) => Ok(Formula::Bool(*b)),
            ConstantValue::String(s) => {
                Err(format!("Cannot convert string '{}' to SMT formula", s))
            }
            ConstantValue::Null => Err("Cannot convert null to SMT formula".to_string()),
        },

        Expression::BinaryOp { op, left, right } => {
            let left_formula = expression_to_formula(left)?;
            let right_formula = expression_to_formula(right)?;

            match op {
                BinaryOp::Add => Ok(Formula::Add(
                    Box::new(left_formula),
                    Box::new(right_formula),
                )),
                BinaryOp::Sub => Ok(Formula::Sub(
                    Box::new(left_formula),
                    Box::new(right_formula),
                )),
                BinaryOp::Mul => Ok(Formula::Mul(
                    Box::new(left_formula),
                    Box::new(right_formula),
                )),
                BinaryOp::Div => Ok(Formula::Div(
                    Box::new(left_formula),
                    Box::new(right_formula),
                )),
                BinaryOp::Mod => Ok(Formula::Mod(
                    Box::new(left_formula),
                    Box::new(right_formula),
                )),

                BinaryOp::Eq => Ok(Formula::Eq(Box::new(left_formula), Box::new(right_formula))),
                BinaryOp::Ne => Ok(Formula::Ne(Box::new(left_formula), Box::new(right_formula))),
                BinaryOp::Lt => Ok(Formula::Lt(Box::new(left_formula), Box::new(right_formula))),
                BinaryOp::Le => Ok(Formula::Le(Box::new(left_formula), Box::new(right_formula))),
                BinaryOp::Gt => Ok(Formula::Gt(Box::new(left_formula), Box::new(right_formula))),
                BinaryOp::Ge => Ok(Formula::Ge(Box::new(left_formula), Box::new(right_formula))),

                BinaryOp::And => Ok(Formula::And(vec![left_formula, right_formula])),
                BinaryOp::Or => Ok(Formula::Or(vec![left_formula, right_formula])),
                BinaryOp::Implies => Ok(Formula::Implies(
                    Box::new(left_formula),
                    Box::new(right_formula),
                )),

                BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor => Err(format!(
                    "Bitwise operations not yet supported in SMT: {:?}",
                    op
                )),
            }
        }

        Expression::UnaryOp { op, operand } => {
            let operand_formula = expression_to_formula(operand)?;

            match op {
                UnaryOp::Neg => Ok(Formula::Sub(
                    Box::new(Formula::Int(0)),
                    Box::new(operand_formula),
                )),
                UnaryOp::Not => Ok(Formula::Not(Box::new(operand_formula))),
                UnaryOp::BitNot => Err("Bitwise NOT not yet supported in SMT".to_string()),
            }
        }

        Expression::Quantifier {
            kind,
            variables,
            body,
        } => {
            let body_formula = expression_to_formula(body)?;

            match kind {
                QuantifierKind::Forall => {
                    Ok(Formula::Forall(variables.clone(), Box::new(body_formula)))
                }
                QuantifierKind::Exists => {
                    Ok(Formula::Exists(variables.clone(), Box::new(body_formula)))
                }
            }
        }

        Expression::ArrayAccess { array, index } => {
            let array_formula = expression_to_formula(array)?;
            let index_formula = expression_to_formula(index)?;
            Ok(Formula::Select(
                Box::new(array_formula),
                Box::new(index_formula),
            ))
        }

        Expression::Old(expr) => {
            // For old values, we need to track them separately in the solver
            // For now, add a prefix to the variable name
            match expr.as_ref() {
                Expression::Variable(name) => Ok(Formula::Var(format!("old_{}", name))),
                _ => Err("Old() can only be applied to variables".to_string()),
            }
        }

        Expression::Result => Ok(Formula::Var("__result__".to_string())),

        Expression::Length(expr) => {
            // Length would need special handling in the solver
            let _inner = expression_to_formula(expr)?;
            Err("Length operator not yet implemented in SMT".to_string())
        }

        // Enhanced expressions
        Expression::SemanticPredicate { predicate, args } => {
            // Semantic predicates need to be expanded to their definitions
            // For now, treat them as uninterpreted functions
            let _arg_formulas: Result<Vec<_>, _> =
                args.iter().map(|arg| expression_to_formula(arg)).collect();

            Err(format!(
                "Semantic predicate '{}' needs definition expansion",
                predicate
            ))
        }

        Expression::Temporal { op, expr } => {
            // Temporal operators require temporal logic support
            let _inner = expression_to_formula(expr)?;
            Err(format!(
                "Temporal operator {:?} not yet supported in SMT",
                op
            ))
        }

        Expression::InSet { element, set } => {
            // Set membership needs special encoding
            let _elem_formula = expression_to_formula(element)?;
            let _set_formula = expression_to_formula(set)?;
            Err("Set membership not yet implemented in SMT".to_string())
        }

        Expression::Range {
            start,
            end,
            inclusive,
        } => {
            // Ranges need to be expanded to constraints
            let _start_formula = expression_to_formula(start)?;
            let _end_formula = expression_to_formula(end)?;
            Err(format!(
                "Range expressions (inclusive={}) not yet implemented",
                inclusive
            ))
        }

        Expression::Matches { expr, pattern } => {
            // Pattern matching requires string theory support
            let _expr_formula = expression_to_formula(expr)?;
            Err(format!(
                "Pattern matching '{}' not yet implemented in SMT",
                pattern
            ))
        }

        Expression::Aggregate {
            op,
            collection,
            predicate,
        } => {
            // Aggregate operations need special encoding
            let _coll_formula = expression_to_formula(collection)?;
            if let Some(pred) = predicate {
                let _pred_formula = expression_to_formula(pred)?;
            }
            Err(format!(
                "Aggregate operation {:?} not yet implemented in SMT",
                op
            ))
        }

        Expression::Let { bindings, body } => {
            // Let expressions can be expanded
            let _body_formula = expression_to_formula(body)?;
            Err(format!(
                "Let expressions with {} bindings not yet implemented",
                bindings.len()
            ))
        }

        _ => Err(format!("Expression type not yet supported: {:?}", expr)),
    }
}

/// Implement the SMT solver interface for our enhanced contract verifier
pub struct Z3SmtSolver {
    // In a real implementation, this would contain the Z3 context
    // For now, it's a stub that delegates to the existing solver
    inner: crate::verification::solver::SmtSolver,
}

impl Z3SmtSolver {
    pub fn new() -> Self {
        Self {
            inner: crate::verification::solver::SmtSolver::new(),
        }
    }
}

impl crate::verification::contracts::SmtSolverInterface for Z3SmtSolver {
    fn check_sat(
        &mut self,
        formula: &Expression,
    ) -> Result<crate::verification::contracts::SatResult, String> {
        // Convert our expression to solver formula
        match expression_to_formula(formula) {
            Ok(smt_formula) => {
                // Create a verification condition
                let vc = crate::verification::solver::VerificationCondition {
                    name: "contract_check".to_string(),
                    formula: smt_formula,
                    location: crate::error::SourceLocation::unknown(),
                };

                // Check with the solver
                match self.inner.check_condition(&vc) {
                    Ok(crate::verification::solver::CheckResult::Verified) => {
                        Ok(crate::verification::contracts::SatResult::Unsat)
                    }
                    Ok(crate::verification::solver::CheckResult::Failed(_)) => {
                        Ok(crate::verification::contracts::SatResult::Sat)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    fn get_model(&mut self) -> Result<HashMap<String, ConstantValue>, String> {
        // Stub implementation
        Ok(HashMap::new())
    }

    fn assert(&mut self, _formula: &Expression) -> Result<(), String> {
        // Stub implementation
        Ok(())
    }

    fn push(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn pop(&mut self) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression_conversion() {
        // Test: x > 0
        let expr = Expression::BinaryOp {
            op: BinaryOp::Gt,
            left: Box::new(Expression::Variable("x".to_string())),
            right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
        };

        let formula = expression_to_formula(&expr).unwrap();
        assert!(matches!(formula, Formula::Gt(_, _)));
    }

    #[test]
    fn test_quantifier_conversion() {
        // Test: forall x. x > 0
        let expr = Expression::Quantifier {
            kind: QuantifierKind::Forall,
            variables: vec![(
                "x".to_string(),
                Type::primitive(crate::ast::PrimitiveType::Integer),
            )],
            body: Box::new(Expression::BinaryOp {
                op: BinaryOp::Gt,
                left: Box::new(Expression::Variable("x".to_string())),
                right: Box::new(Expression::Constant(ConstantValue::Integer(0))),
            }),
        };

        let formula = expression_to_formula(&expr).unwrap();
        assert!(matches!(formula, Formula::Forall(_, _)));
    }
}
