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

//! Convert AST expressions to verification contract expressions
//!
//! This module bridges the gap between parsed contract conditions (AST expressions)
//! and the verification system's expression representation.

use crate::ast;
use crate::verification::contracts::{
    BinaryOp, ConstantValue, EnhancedCondition, Expression, FailureAction, FunctionContract,
    UnaryOp, VerificationHint,
};
use std::collections::HashSet;

/// Extract function name from a FunctionReference
fn get_function_name(func_ref: &ast::FunctionReference) -> String {
    match func_ref {
        ast::FunctionReference::Local { name } => name.name.clone(),
        ast::FunctionReference::Qualified { module, name } => {
            format!("{}::{}", module.name, name.name)
        }
        ast::FunctionReference::External { name } => name.name.clone(),
    }
}

/// Convert an AST expression to a verification contract expression
pub fn ast_expr_to_contract_expr(ast_expr: &ast::Expression) -> Result<Expression, String> {
    match ast_expr {
        // Literals
        ast::Expression::IntegerLiteral { value, .. } => {
            Ok(Expression::Constant(ConstantValue::Integer(*value)))
        }

        ast::Expression::FloatLiteral { value, .. } => {
            Ok(Expression::Constant(ConstantValue::Float(*value)))
        }

        ast::Expression::BooleanLiteral { value, .. } => {
            Ok(Expression::Constant(ConstantValue::Boolean(*value)))
        }

        ast::Expression::StringLiteral { value, .. } => {
            Ok(Expression::Constant(ConstantValue::String(value.clone())))
        }

        // Variable reference
        ast::Expression::Variable { name, .. } => {
            // Check for special contract keywords
            match name.name.as_str() {
                "result" | "__result__" => Ok(Expression::Result),
                _ => Ok(Expression::Variable(name.name.clone())),
            }
        }

        // Arithmetic operations
        ast::Expression::Add { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        ast::Expression::Subtract { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Sub,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        ast::Expression::Multiply { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Mul,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        ast::Expression::Divide { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Div,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        ast::Expression::Modulo { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Mod,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        // Comparison operations
        ast::Expression::Equals { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Eq,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        ast::Expression::NotEquals { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Ne,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        ast::Expression::LessThan { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Lt,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        ast::Expression::LessThanOrEqual { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Le,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        ast::Expression::GreaterThan { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Gt,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        ast::Expression::GreaterThanOrEqual { left, right, .. } => {
            let left_expr = ast_expr_to_contract_expr(left)?;
            let right_expr = ast_expr_to_contract_expr(right)?;
            Ok(Expression::BinaryOp {
                op: BinaryOp::Ge,
                left: Box::new(left_expr),
                right: Box::new(right_expr),
            })
        }

        // Logical operations - note: LogicalAnd/Or have Vec<Expression> operands
        ast::Expression::LogicalAnd { operands, .. } => {
            if operands.is_empty() {
                return Ok(Expression::Constant(ConstantValue::Boolean(true)));
            }
            let mut result = ast_expr_to_contract_expr(&operands[0])?;
            for operand in operands.iter().skip(1) {
                let right = ast_expr_to_contract_expr(operand)?;
                result = Expression::BinaryOp {
                    op: BinaryOp::And,
                    left: Box::new(result),
                    right: Box::new(right),
                };
            }
            Ok(result)
        }

        ast::Expression::LogicalOr { operands, .. } => {
            if operands.is_empty() {
                return Ok(Expression::Constant(ConstantValue::Boolean(false)));
            }
            let mut result = ast_expr_to_contract_expr(&operands[0])?;
            for operand in operands.iter().skip(1) {
                let right = ast_expr_to_contract_expr(operand)?;
                result = Expression::BinaryOp {
                    op: BinaryOp::Or,
                    left: Box::new(result),
                    right: Box::new(right),
                };
            }
            Ok(result)
        }

        // Unary operations
        ast::Expression::Negate { operand, .. } => {
            let operand_expr = ast_expr_to_contract_expr(operand)?;
            Ok(Expression::UnaryOp {
                op: UnaryOp::Neg,
                operand: Box::new(operand_expr),
            })
        }

        ast::Expression::LogicalNot { operand, .. } => {
            let operand_expr = ast_expr_to_contract_expr(operand)?;
            Ok(Expression::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(operand_expr),
            })
        }

        // Function call
        ast::Expression::FunctionCall { call, .. } => {
            let func_name = get_function_name(&call.function_reference);

            match func_name.as_str() {
                "old" => {
                    // old(expr) - reference to value at function entry
                    if call.arguments.len() != 1 {
                        return Err("old() requires exactly one argument".to_string());
                    }
                    let inner = ast_expr_to_contract_expr(&call.arguments[0].value)?;
                    Ok(Expression::Old(Box::new(inner)))
                }
                "len" | "length" => {
                    // length(collection)
                    if call.arguments.len() != 1 {
                        return Err("length() requires exactly one argument".to_string());
                    }
                    let inner = ast_expr_to_contract_expr(&call.arguments[0].value)?;
                    Ok(Expression::Length(Box::new(inner)))
                }
                _ => {
                    // Regular function call
                    let args: Result<Vec<_>, _> = call
                        .arguments
                        .iter()
                        .map(|arg| ast_expr_to_contract_expr(&arg.value))
                        .collect();

                    Ok(Expression::Call {
                        function: func_name,
                        args: args?,
                    })
                }
            }
        }

        // Field access
        ast::Expression::FieldAccess {
            instance,
            field_name,
            ..
        } => {
            let object_expr = ast_expr_to_contract_expr(instance)?;
            Ok(Expression::FieldAccess {
                object: Box::new(object_expr),
                field: field_name.name.clone(),
            })
        }

        // Array/index access
        ast::Expression::ArrayAccess { array, index, .. } => {
            let array_expr = ast_expr_to_contract_expr(array)?;
            let index_expr = ast_expr_to_contract_expr(index)?;
            Ok(Expression::ArrayAccess {
                array: Box::new(array_expr),
                index: Box::new(index_expr),
            })
        }

        // Array length
        ast::Expression::ArrayLength { array, .. } => {
            let inner = ast_expr_to_contract_expr(array)?;
            Ok(Expression::Length(Box::new(inner)))
        }

        // Match expression - convert simple match patterns
        ast::Expression::Match { value, cases, .. } => {
            // For now, only support simple switch-like patterns
            // Complex pattern matching would need more work
            if cases.is_empty() {
                return Err("Empty match expression in contract".to_string());
            }

            // Just convert the value being matched - full match conversion is complex
            let value_expr = ast_expr_to_contract_expr(value)?;

            // Return a placeholder that represents the match value
            // Full match semantics would need ITE chains
            Ok(value_expr)
        }

        // Unsupported expressions
        _ => Err(format!(
            "Unsupported AST expression type in contract: {:?}",
            std::mem::discriminant(ast_expr)
        )),
    }
}

/// Convert an AST ContractAssertion to a verification EnhancedCondition
pub fn ast_assertion_to_condition(
    assertion: &ast::ContractAssertion,
    name_prefix: &str,
    index: usize,
) -> Result<EnhancedCondition, String> {
    let expression = ast_expr_to_contract_expr(&assertion.condition)?;

    // Map AST FailureAction to verification FailureAction
    let failure_action = match &assertion.failure_action {
        ast::FailureAction::ThrowException => {
            FailureAction::ThrowException(assertion.message.clone().unwrap_or_default())
        }
        ast::FailureAction::LogWarning => FailureAction::LogAndContinue,
        ast::FailureAction::AssertFail => FailureAction::Abort,
    };

    Ok(EnhancedCondition {
        name: format!("{}_{}", name_prefix, index),
        expression,
        location: assertion.source_location.clone(),
        proof_hint: assertion.message.clone(),
        failure_action,
        verification_hint: VerificationHint::SMTSolver, // Default hint, consider mapping this if @hint is added later
        verification_mode: assertion.verification_mode, // Transfer the new field
    })
}

/// Extract a FunctionContract from an AST function definition
pub fn extract_function_contract(func: &ast::Function) -> Result<Option<FunctionContract>, String> {
    let metadata = &func.metadata;

    // If no contracts, return None
    if metadata.preconditions.is_empty()
        && metadata.postconditions.is_empty()
        && metadata.invariants.is_empty()
    {
        return Ok(None);
    }

    // Convert preconditions
    let preconditions: Result<Vec<_>, _> = metadata
        .preconditions
        .iter()
        .enumerate()
        .map(|(i, assertion)| ast_assertion_to_condition(assertion, "pre", i))
        .collect();

    // Convert postconditions
    let postconditions: Result<Vec<_>, _> = metadata
        .postconditions
        .iter()
        .enumerate()
        .map(|(i, assertion)| ast_assertion_to_condition(assertion, "post", i))
        .collect();

    // Convert invariants
    let invariants: Result<Vec<_>, _> = metadata
        .invariants
        .iter()
        .enumerate()
        .map(|(i, assertion)| ast_assertion_to_condition(assertion, "inv", i))
        .collect();

    Ok(Some(FunctionContract {
        function_name: func.name.name.clone(),
        preconditions: preconditions?,
        postconditions: postconditions?,
        invariants: invariants?,
        modifies: HashSet::new(),
        is_pure: false,
        decreases: None,
        intent: None,
        behavior: None,
        resources: None,
        failure_actions: std::collections::HashMap::new(),
        propagation: crate::verification::contracts::ContractPropagation::default(),
        proof_obligations: Vec::new(),
    }))
}

/// Extract all function contracts from a program's AST
pub fn extract_program_contracts(
    program: &ast::Program,
) -> Result<std::collections::HashMap<String, FunctionContract>, String> {
    let mut contracts = std::collections::HashMap::new();

    for module in &program.modules {
        for func in &module.function_definitions {
            if let Some(contract) = extract_function_contract(func)? {
                // Use '.' separator to match MIR function naming convention
                let full_name = format!("{}.{}", module.name.name, func.name.name);
                contracts.insert(full_name, contract);
            }
        }
    }

    Ok(contracts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SourceLocation;

    #[test]
    fn test_simple_expression_conversion() {
        // Test integer literal
        let ast_expr = ast::Expression::IntegerLiteral {
            value: 42,
            source_location: SourceLocation::unknown(),
        };

        let result = ast_expr_to_contract_expr(&ast_expr).unwrap();
        match result {
            Expression::Constant(ConstantValue::Integer(42)) => {}
            _ => panic!("Expected integer constant 42"),
        }
    }

    #[test]
    fn test_binary_expression_conversion() {
        // Test: x > 0
        let ast_expr = ast::Expression::GreaterThan {
            left: Box::new(ast::Expression::Variable {
                name: ast::Identifier::new("x".to_string(), SourceLocation::unknown()),
                source_location: SourceLocation::unknown(),
            }),
            right: Box::new(ast::Expression::IntegerLiteral {
                value: 0,
                source_location: SourceLocation::unknown(),
            }),
            source_location: SourceLocation::unknown(),
        };

        let result = ast_expr_to_contract_expr(&ast_expr).unwrap();
        match result {
            Expression::BinaryOp {
                op: BinaryOp::Gt, ..
            } => {}
            _ => panic!("Expected binary Gt operation"),
        }
    }

    #[test]
    fn test_result_keyword() {
        // Test result keyword in postcondition
        let ast_expr = ast::Expression::Variable {
            name: ast::Identifier::new("result".to_string(), SourceLocation::unknown()),
            source_location: SourceLocation::unknown(),
        };

        let result = ast_expr_to_contract_expr(&ast_expr).unwrap();
        match result {
            Expression::Result => {}
            _ => panic!("Expected Result expression"),
        }
    }
}
