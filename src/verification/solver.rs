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

//! SMT solver interface using Z3
//!
//! Provides formal verification capabilities via Z3 SMT solver

use crate::error::SourceLocation;
use crate::types::Type;
use crate::verification::VerificationMode;
use std::collections::HashMap;
use z3::ast::Ast;

#[derive(Debug)]
pub struct SmtSolver {
    // Solver is created fresh for each check to avoid lifetime issues
}

/// Solver value types
#[derive(Debug, Clone)]
pub enum SolverValue {
    Int(i64),
    Real(f64),
    Bool(bool),
    String(String),
    Array(Vec<SolverValue>),
}

/// Result of checking a condition
#[derive(Debug)]
pub enum CheckResult {
    /// Condition is verified (unsatisfiable negation)
    Verified,

    /// Condition failed with counterexample
    Failed(Model),
}

/// Model (counterexample) from solver
#[derive(Debug)]
pub struct Model {
    /// Variable assignments
    pub assignments: HashMap<String, SolverValue>,

    /// Execution trace (if available)
    pub execution_trace: Vec<String>,
}

/// Verification condition to check
#[derive(Debug, Clone)]
pub struct VerificationCondition {
    /// Condition name
    pub name: String,

    /// The formula to verify
    pub formula: Formula,

    /// Source location
    pub location: SourceLocation,

    /// Specific verification mode for this condition (overrides function-level)
    pub verification_mode: Option<VerificationMode>,
}

/// Logical formula
#[derive(Debug, Clone)]
pub enum Formula {
    /// Boolean constant
    Bool(bool),

    /// Integer constant
    Int(i64),

    /// Real constant
    Real(f64),

    /// Variable reference
    Var(String),

    /// Equality
    Eq(Box<Formula>, Box<Formula>),

    /// Inequality
    Ne(Box<Formula>, Box<Formula>),

    /// Less than
    Lt(Box<Formula>, Box<Formula>),

    /// Less than or equal
    Le(Box<Formula>, Box<Formula>),

    /// Greater than
    Gt(Box<Formula>, Box<Formula>),

    /// Greater than or equal
    Ge(Box<Formula>, Box<Formula>),

    /// Addition
    Add(Box<Formula>, Box<Formula>),

    /// Subtraction
    Sub(Box<Formula>, Box<Formula>),

    /// Multiplication
    Mul(Box<Formula>, Box<Formula>),

    /// Division
    Div(Box<Formula>, Box<Formula>),

    /// Modulo
    Mod(Box<Formula>, Box<Formula>),

    /// Logical AND
    And(Vec<Formula>),

    /// Logical OR
    Or(Vec<Formula>),

    /// Logical NOT
    Not(Box<Formula>),

    /// Implication
    Implies(Box<Formula>, Box<Formula>),

    /// If-then-else
    Ite(Box<Formula>, Box<Formula>, Box<Formula>),

    /// Universal quantifier
    Forall(Vec<(String, Type)>, Box<Formula>),

    /// Existential quantifier
    Exists(Vec<(String, Type)>, Box<Formula>),

    /// Array select
    Select(Box<Formula>, Box<Formula>),

    /// Array store
    Store(Box<Formula>, Box<Formula>, Box<Formula>),
}

impl Default for SmtSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl SmtSolver {
    /// Create a new SMT solver
    pub fn new() -> Self {
        Self {}
    }

    /// Check a verification condition using Z3
    pub fn check_condition(&mut self, vc: &VerificationCondition) -> Result<CheckResult, String> {
        self.check_condition_with_axioms(vc, &[])
    }

    /// Check a verification condition with additional axioms asserted.
    pub fn check_condition_with_axioms(
        &mut self,
        vc: &VerificationCondition,
        axioms: &[Formula],
    ) -> Result<CheckResult, String> {
        eprintln!("Verifying condition: {}", vc.name);

        // Create fresh Z3 context and solver for this check
        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);
        let solver = z3::Solver::new(&ctx);

        // Track variables we create
        let mut variables: HashMap<String, z3::ast::Int> = HashMap::new();

        // Convert formula to Z3 AST
        let z3_formula = formula_to_z3(&ctx, &vc.formula, &mut variables)?;

        // Assert axioms (assumptions) first.
        for axiom in axioms {
            let z3_axiom = formula_to_z3(&ctx, axiom, &mut variables)?;
            let z3_bool = z3_axiom
                .as_bool()
                .ok_or_else(|| "Axiom must evaluate to boolean".to_string())?;
            solver.assert(&z3_bool);
        }

        // To verify a formula is always true, we assert its negation
        // If the negation is unsatisfiable, the original is valid
        let z3_bool = z3_formula
            .as_bool()
            .ok_or_else(|| "Formula must evaluate to boolean".to_string())?;
        solver.assert(&z3_bool.not());

        // Check satisfiability
        match solver.check() {
            z3::SatResult::Unsat => {
                // Negation is unsatisfiable => original formula is valid
                eprintln!("  ✓ Verified");
                Ok(CheckResult::Verified)
            }
            z3::SatResult::Sat => {
                // Found counterexample
                eprintln!("  ✗ Failed - counterexample found");
                let model = if let Some(z3_model) = solver.get_model() {
                    extract_model(&z3_model, &variables)
                } else {
                    Model {
                        assignments: HashMap::new(),
                        execution_trace: vec![
                            "Counterexample exists but model unavailable".to_string()
                        ],
                    }
                };
                Ok(CheckResult::Failed(model))
            }
            z3::SatResult::Unknown => {
                eprintln!("  ? Unknown (solver timeout or complexity)");
                Err("Solver returned unknown - formula may be too complex".to_string())
            }
        }
    }
}

/// Convert our Formula to Z3 AST
fn formula_to_z3<'ctx>(
    ctx: &'ctx z3::Context,
    formula: &Formula,
    variables: &mut HashMap<String, z3::ast::Int<'ctx>>,
) -> Result<z3::ast::Dynamic<'ctx>, String> {
    use z3::ast::{Bool, Int};

    match formula {
        Formula::Bool(b) => Ok(Bool::from_bool(ctx, *b).into()),

        Formula::Int(n) => Ok(Int::from_i64(ctx, *n).into()),

        Formula::Real(f) => {
            // Approximate real as integer for now (TODO: use Real sort)
            Ok(Int::from_i64(ctx, *f as i64).into())
        }

        Formula::Var(name) => {
            if let Some(var) = variables.get(name) {
                Ok(var.clone().into())
            } else {
                // Create new integer variable
                let var = Int::new_const(ctx, name.as_str());
                variables.insert(name.clone(), var.clone());
                Ok(var.into())
            }
        }

        Formula::Eq(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            Ok(l._eq(&r).into())
        }

        Formula::Ne(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            Ok(l._eq(&r).not().into())
        }

        Formula::Lt(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_int = l.as_int().ok_or("Expected integer in <")?;
            let r_int = r.as_int().ok_or("Expected integer in <")?;
            Ok(l_int.lt(&r_int).into())
        }

        Formula::Le(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_int = l.as_int().ok_or("Expected integer in <=")?;
            let r_int = r.as_int().ok_or("Expected integer in <=")?;
            Ok(l_int.le(&r_int).into())
        }

        Formula::Gt(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_int = l.as_int().ok_or("Expected integer in >")?;
            let r_int = r.as_int().ok_or("Expected integer in >")?;
            Ok(l_int.gt(&r_int).into())
        }

        Formula::Ge(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_int = l.as_int().ok_or("Expected integer in >=")?;
            let r_int = r.as_int().ok_or("Expected integer in >=")?;
            Ok(l_int.ge(&r_int).into())
        }

        Formula::Add(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_int = l.as_int().ok_or("Expected integer in +")?;
            let r_int = r.as_int().ok_or("Expected integer in +")?;
            Ok(Int::add(ctx, &[&l_int, &r_int]).into())
        }

        Formula::Sub(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_int = l.as_int().ok_or("Expected integer in -")?;
            let r_int = r.as_int().ok_or("Expected integer in -")?;
            Ok(Int::sub(ctx, &[&l_int, &r_int]).into())
        }

        Formula::Mul(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_int = l.as_int().ok_or("Expected integer in *")?;
            let r_int = r.as_int().ok_or("Expected integer in *")?;
            Ok(Int::mul(ctx, &[&l_int, &r_int]).into())
        }

        Formula::Div(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_int = l.as_int().ok_or("Expected integer in /")?;
            let r_int = r.as_int().ok_or("Expected integer in /")?;
            Ok(l_int.div(&r_int).into())
        }

        Formula::Mod(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_int = l.as_int().ok_or("Expected integer in %")?;
            let r_int = r.as_int().ok_or("Expected integer in %")?;
            Ok(l_int.modulo(&r_int).into())
        }

        Formula::And(formulas) => {
            let z3_formulas: Result<Vec<_>, _> = formulas
                .iter()
                .map(|f| formula_to_z3(ctx, f, variables))
                .collect();
            let z3_formulas = z3_formulas?;

            let bool_formulas: Result<Vec<_>, _> = z3_formulas
                .iter()
                .map(|f| f.as_bool().ok_or("Expected boolean in AND"))
                .collect();
            let bool_formulas = bool_formulas?;

            let refs: Vec<&Bool> = bool_formulas.iter().collect();
            Ok(Bool::and(ctx, &refs).into())
        }

        Formula::Or(formulas) => {
            let z3_formulas: Result<Vec<_>, _> = formulas
                .iter()
                .map(|f| formula_to_z3(ctx, f, variables))
                .collect();
            let z3_formulas = z3_formulas?;

            let bool_formulas: Result<Vec<_>, _> = z3_formulas
                .iter()
                .map(|f| f.as_bool().ok_or("Expected boolean in OR"))
                .collect();
            let bool_formulas = bool_formulas?;

            let refs: Vec<&Bool> = bool_formulas.iter().collect();
            Ok(Bool::or(ctx, &refs).into())
        }

        Formula::Not(f) => {
            let inner = formula_to_z3(ctx, f, variables)?;
            let bool_inner = inner.as_bool().ok_or("Expected boolean in NOT")?;
            Ok(bool_inner.not().into())
        }

        Formula::Implies(left, right) => {
            let l = formula_to_z3(ctx, left, variables)?;
            let r = formula_to_z3(ctx, right, variables)?;
            let l_bool = l.as_bool().ok_or("Expected boolean in =>")?;
            let r_bool = r.as_bool().ok_or("Expected boolean in =>")?;
            Ok(l_bool.implies(&r_bool).into())
        }

        Formula::Ite(cond, then_branch, else_branch) => {
            let c = formula_to_z3(ctx, cond, variables)?;
            let t = formula_to_z3(ctx, then_branch, variables)?;
            let e = formula_to_z3(ctx, else_branch, variables)?;
            let c_bool = c.as_bool().ok_or("Expected boolean condition in ITE")?;

            // ITE with integers
            if let (Some(t_int), Some(e_int)) = (t.as_int(), e.as_int()) {
                Ok(c_bool.ite(&t_int, &e_int).into())
            } else if let (Some(t_bool), Some(e_bool)) = (t.as_bool(), e.as_bool()) {
                Ok(c_bool.ite(&t_bool, &e_bool).into())
            } else {
                Err("Type mismatch in ITE branches".to_string())
            }
        }

        Formula::Forall(vars, body) => {
            // Create bound variables
            let mut bound_vars: Vec<z3::ast::Int> = Vec::new();
            for (name, _ty) in vars {
                let var = Int::new_const(ctx, name.as_str());
                variables.insert(name.clone(), var.clone());
                bound_vars.push(var);
            }

            let body_z3 = formula_to_z3(ctx, body, variables)?;
            let body_bool = body_z3.as_bool().ok_or("Forall body must be boolean")?;

            let bound_refs: Vec<_> = bound_vars.iter().map(|v| v as &dyn Ast).collect();
            Ok(z3::ast::forall_const(ctx, &bound_refs, &[], &body_bool).into())
        }

        Formula::Exists(vars, body) => {
            // Create bound variables
            let mut bound_vars: Vec<z3::ast::Int> = Vec::new();
            for (name, _ty) in vars {
                let var = Int::new_const(ctx, name.as_str());
                variables.insert(name.clone(), var.clone());
                bound_vars.push(var);
            }

            let body_z3 = formula_to_z3(ctx, body, variables)?;
            let body_bool = body_z3.as_bool().ok_or("Exists body must be boolean")?;

            let bound_refs: Vec<_> = bound_vars.iter().map(|v| v as &dyn Ast).collect();
            Ok(z3::ast::exists_const(ctx, &bound_refs, &[], &body_bool).into())
        }

        Formula::Select(_array, _index) => Err("Array select not yet implemented".to_string()),

        Formula::Store(_array, _index, _value) => {
            Err("Array store not yet implemented".to_string())
        }
    }
}

/// Extract counterexample model from Z3
fn extract_model(z3_model: &z3::Model, variables: &HashMap<String, z3::ast::Int>) -> Model {
    let mut assignments = HashMap::new();

    for (name, var) in variables {
        if let Some(value) = z3_model.eval(var, true) {
            if let Some(i) = value.as_i64() {
                assignments.insert(name.clone(), SolverValue::Int(i));
            }
        }
    }

    Model {
        assignments,
        execution_trace: vec![],
    }
}

impl Formula {
    /// Convert to string for display
    pub fn to_string(&self) -> String {
        match self {
            Formula::Bool(b) => b.to_string(),
            Formula::Int(n) => n.to_string(),
            Formula::Real(f) => f.to_string(),
            Formula::Var(name) => name.clone(),
            Formula::Eq(l, r) => format!("({} = {})", l.to_string(), r.to_string()),
            Formula::Lt(l, r) => format!("({} < {})", l.to_string(), r.to_string()),
            Formula::Le(l, r) => format!("({} <= {})", l.to_string(), r.to_string()),
            Formula::Add(l, r) => format!("({} + {})", l.to_string(), r.to_string()),
            Formula::And(fs) => {
                let parts: Vec<_> = fs.iter().map(|f| f.to_string()).collect();
                format!("({})", parts.join(" && "))
            }
            Formula::Or(fs) => {
                let parts: Vec<_> = fs.iter().map(|f| f.to_string()).collect();
                format!("({})", parts.join(" || "))
            }
            Formula::Not(f) => format!("!{}", f.to_string()),
            Formula::Implies(l, r) => format!("({} => {})", l.to_string(), r.to_string()),
            _ => "...".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_verification() {
        let mut solver = SmtSolver::new();

        // Verify: x > 0 => x + 1 > 0
        let formula = Formula::Implies(
            Box::new(Formula::Gt(
                Box::new(Formula::Var("x".to_string())),
                Box::new(Formula::Int(0)),
            )),
            Box::new(Formula::Gt(
                Box::new(Formula::Add(
                    Box::new(Formula::Var("x".to_string())),
                    Box::new(Formula::Int(1)),
                )),
                Box::new(Formula::Int(0)),
            )),
        );

        let vc = VerificationCondition {
            name: "test".to_string(),
            formula,
            location: SourceLocation::unknown(),
            verification_mode: None,
        };

        match solver.check_condition(&vc) {
            Ok(CheckResult::Verified) => {
                // Expected result
            }
            Ok(CheckResult::Failed(_)) => {
                panic!("Verification should have succeeded");
            }
            Err(e) => {
                panic!("Solver error: {}", e);
            }
        }
    }

    #[test]
    fn test_verification_with_axioms() {
        let mut solver = SmtSolver::new();

        // VC requires x == y, which is not provable without an axiom.
        let vc = VerificationCondition {
            name: "needs_axiom".to_string(),
            formula: Formula::Eq(
                Box::new(Formula::Var("x".to_string())),
                Box::new(Formula::Var("y".to_string())),
            ),
            location: SourceLocation::unknown(),
            verification_mode: None,
        };

        // Without axioms, verification should fail.
        match solver.check_condition(&vc) {
            Ok(CheckResult::Failed(_)) => {}
            _ => panic!("Verification should fail without supporting axioms"),
        }

        // With an axiom asserting x == y, it should verify.
        let axioms = vec![Formula::Eq(
            Box::new(Formula::Var("x".to_string())),
            Box::new(Formula::Var("y".to_string())),
        )];
        match solver.check_condition_with_axioms(&vc, &axioms) {
            Ok(CheckResult::Verified) => {}
            other => panic!("Verification with axiom should succeed, got {:?}", other),
        }
    }
}
