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

use crate::error::SourceLocation;
use crate::types::Type;
use std::collections::HashMap;
use z3::{Config, Context, Solver, Sort, ast};
use z3::ast::{Ast, Bool, Int, Real, Array};

/// SMT solver wrapper
pub struct SmtSolver {
    /// Z3 context
    context: Context,
    
    /// Current solver instance
    solver: Solver<'static>,
    
    /// Variable declarations
    variables: HashMap<String, SolverVariable>,
}

/// Variable in the solver
struct SolverVariable {
    /// Variable name
    name: String,
    
    /// Variable sort (type)
    sort: Sort<'static>,
    
    /// Z3 AST node
    ast: ast::Dynamic<'static>,
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
pub enum CheckResult {
    /// Condition is verified (unsatisfiable negation)
    Verified,
    
    /// Condition failed with counterexample
    Failed(Model),
}

/// Model (counterexample) from solver
pub struct Model {
    /// Variable assignments
    pub assignments: HashMap<String, SolverValue>,
    
    /// Execution trace (if available)
    pub execution_trace: Vec<String>,
}

/// Verification condition to check
pub struct VerificationCondition {
    /// Condition name
    pub name: String,
    
    /// The formula to verify
    pub formula: Formula,
    
    /// Source location
    pub location: SourceLocation,
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

impl SmtSolver {
    /// Create a new SMT solver
    pub fn new() -> Self {
        let config = Config::new();
        let context = Context::new(&config);
        let solver = Solver::new(&context);
        
        Self {
            context,
            solver,
            variables: HashMap::new(),
        }
    }
    
    /// Check a verification condition
    pub fn check_condition(&mut self, vc: &VerificationCondition) -> Result<CheckResult, String> {
        // Clear previous assertions
        self.solver.reset();
        self.variables.clear();
        
        // Convert formula to Z3 AST
        let z3_formula = self.formula_to_z3(&vc.formula)?;
        
        // Assert the negation (we want to prove the formula is always true)
        let negated = z3_formula.as_bool()
            .ok_or("Formula must be boolean")?
            .not();
        self.solver.assert(&negated);
        
        // Check satisfiability
        match self.solver.check() {
            z3::SatResult::Unsat => {
                // Negation is unsatisfiable, so original formula is valid
                Ok(CheckResult::Verified)
            }
            z3::SatResult::Sat => {
                // Found counterexample
                let model = self.extract_model()?;
                Ok(CheckResult::Failed(model))
            }
            z3::SatResult::Unknown => {
                Err("Solver returned unknown".to_string())
            }
        }
    }
    
    /// Convert our formula to Z3 AST
    fn formula_to_z3(&mut self, formula: &Formula) -> Result<ast::Dynamic<'static>, String> {
        match formula {
            Formula::Bool(b) => Ok(ast::Bool::from_bool(&self.context, *b).into()),
            
            Formula::Int(n) => Ok(ast::Int::from_i64(&self.context, *n).into()),
            
            Formula::Real(f) => Ok(ast::Real::from_real(&self.context, *f as i32, 1).into()),
            
            Formula::Var(name) => {
                if let Some(var) = self.variables.get(name) {
                    Ok(var.ast.clone())
                } else {
                    // Create new variable (assume integer for now)
                    let sort = Sort::int(&self.context);
                    let ast = ast::Int::new_const(&self.context, name.clone());
                    self.variables.insert(name.clone(), SolverVariable {
                        name: name.clone(),
                        sort,
                        ast: ast.clone().into(),
                    });
                    Ok(ast.into())
                }
            }
            
            Formula::Eq(left, right) => {
                let l = self.formula_to_z3(left)?;
                let r = self.formula_to_z3(right)?;
                Ok(l._eq(&r).into())
            }
            
            Formula::Lt(left, right) => {
                let l = self.formula_to_z3(left)?;
                let r = self.formula_to_z3(right)?;
                
                if let (Some(l_int), Some(r_int)) = (l.as_int(), r.as_int()) {
                    Ok(l_int.lt(&r_int).into())
                } else if let (Some(l_real), Some(r_real)) = (l.as_real(), r.as_real()) {
                    Ok(l_real.lt(&r_real).into())
                } else {
                    Err("Type mismatch in comparison".to_string())
                }
            }
            
            Formula::Le(left, right) => {
                let l = self.formula_to_z3(left)?;
                let r = self.formula_to_z3(right)?;
                
                if let (Some(l_int), Some(r_int)) = (l.as_int(), r.as_int()) {
                    Ok(l_int.le(&r_int).into())
                } else if let (Some(l_real), Some(r_real)) = (l.as_real(), r.as_real()) {
                    Ok(l_real.le(&r_real).into())
                } else {
                    Err("Type mismatch in comparison".to_string())
                }
            }
            
            Formula::Add(left, right) => {
                let l = self.formula_to_z3(left)?;
                let r = self.formula_to_z3(right)?;
                
                if let (Some(l_int), Some(r_int)) = (l.as_int(), r.as_int()) {
                    Ok((l_int + r_int).into())
                } else if let (Some(l_real), Some(r_real)) = (l.as_real(), r.as_real()) {
                    Ok((l_real + r_real).into())
                } else {
                    Err("Type mismatch in addition".to_string())
                }
            }
            
            Formula::And(formulas) => {
                let z3_formulas: Result<Vec<_>, _> = formulas.iter()
                    .map(|f| self.formula_to_z3(f))
                    .collect();
                let z3_formulas = z3_formulas?;
                
                let bool_formulas: Result<Vec<_>, _> = z3_formulas.iter()
                    .map(|f| f.as_bool().ok_or("Expected boolean in AND"))
                    .collect();
                let bool_formulas = bool_formulas?;
                
                let refs: Vec<&Bool> = bool_formulas.iter().collect();
                Ok(Bool::and(&self.context, &refs).into())
            }
            
            Formula::Or(formulas) => {
                let z3_formulas: Result<Vec<_>, _> = formulas.iter()
                    .map(|f| self.formula_to_z3(f))
                    .collect();
                let z3_formulas = z3_formulas?;
                
                let bool_formulas: Result<Vec<_>, _> = z3_formulas.iter()
                    .map(|f| f.as_bool().ok_or("Expected boolean in OR"))
                    .collect();
                let bool_formulas = bool_formulas?;
                
                let refs: Vec<&Bool> = bool_formulas.iter().collect();
                Ok(Bool::or(&self.context, &refs).into())
            }
            
            Formula::Not(formula) => {
                let f = self.formula_to_z3(formula)?;
                let bool_f = f.as_bool().ok_or("Expected boolean in NOT")?;
                Ok(bool_f.not().into())
            }
            
            Formula::Implies(left, right) => {
                let l = self.formula_to_z3(left)?;
                let r = self.formula_to_z3(right)?;
                let l_bool = l.as_bool().ok_or("Expected boolean in implies")?;
                let r_bool = r.as_bool().ok_or("Expected boolean in implies")?;
                Ok(l_bool.implies(&r_bool).into())
            }
            
            // TODO: Implement remaining formula types
            _ => Err("Formula type not yet implemented".to_string()),
        }
    }
    
    /// Extract model from solver
    fn extract_model(&self) -> Result<Model, String> {
        let model = self.solver.get_model()
            .ok_or("No model available")?;
        
        let mut assignments = HashMap::new();
        
        for (name, var) in &self.variables {
            if let Some(value) = model.eval(&var.ast, true) {
                let solver_value = self.z3_to_value(&value)?;
                assignments.insert(name.clone(), solver_value);
            }
        }
        
        Ok(Model {
            assignments,
            execution_trace: Vec::new(), // TODO: Extract trace if available
        })
    }
    
    /// Convert Z3 value to our value type
    fn z3_to_value(&self, value: &ast::Dynamic) -> Result<SolverValue, String> {
        if let Some(int_val) = value.as_int() {
            Ok(SolverValue::Int(int_val.as_i64().unwrap_or(0)))
        } else if let Some(bool_val) = value.as_bool() {
            Ok(SolverValue::Bool(bool_val.as_bool().unwrap_or(false)))
        } else if let Some(real_val) = value.as_real() {
            // Simplified real handling
            Ok(SolverValue::Real(0.0))
        } else {
            Err("Unknown value type".to_string())
        }
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
}