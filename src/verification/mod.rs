//! Formal verification framework for AetherScript
//! 
//! Provides contract verification, invariant checking, and property proving

pub mod contracts;
pub mod contract_to_smt;
pub mod invariants;
pub mod solver;
pub mod vcgen;

use crate::error::{SemanticError, SourceLocation};
use crate::mir;
use std::collections::HashMap;

/// Verification result for a function or module
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Function or module name
    pub name: String,
    
    /// Whether verification succeeded
    pub verified: bool,
    
    /// Individual verification conditions and their results
    pub conditions: Vec<ConditionResult>,
    
    /// Counterexamples for failed conditions
    pub counterexamples: Vec<Counterexample>,
}

/// Result of verifying a single condition
#[derive(Debug, Clone)]
pub struct ConditionResult {
    /// Condition name/description
    pub name: String,
    
    /// The actual condition being verified
    pub condition: String,
    
    /// Whether the condition was verified
    pub verified: bool,
    
    /// Location in source code
    pub location: SourceLocation,
    
    /// Time taken to verify (in milliseconds)
    pub verification_time_ms: u64,
}

/// Counterexample for a failed verification
#[derive(Debug, Clone)]
pub struct Counterexample {
    /// Condition that failed
    pub condition_name: String,
    
    /// Variable assignments that cause the failure
    pub assignments: HashMap<String, Value>,
    
    /// Execution trace leading to the failure
    pub trace: Vec<String>,
}

/// Value in a counterexample
#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<Value>),
}

/// Main verification engine
pub struct VerificationEngine {
    /// SMT solver instance
    solver: solver::SmtSolver,
    
    /// Verification condition generator
    vcgen: vcgen::VcGenerator,
    
    /// Current verification context
    context: VerificationContext,
}

/// Context for verification
#[derive(Debug, Default)]
struct VerificationContext {
    /// Current function being verified
    current_function: Option<String>,
    
    /// Known function contracts
    function_contracts: HashMap<String, contracts::FunctionContract>,
    
    /// Loop invariants
    loop_invariants: HashMap<mir::BasicBlockId, invariants::LoopInvariant>,
    
    /// Global invariants
    global_invariants: Vec<invariants::GlobalInvariant>,
}

impl VerificationEngine {
    /// Create a new verification engine
    pub fn new() -> Self {
        Self {
            solver: solver::SmtSolver::new(),
            vcgen: vcgen::VcGenerator::new(),
            context: VerificationContext::default(),
        }
    }
    
    /// Verify a complete program
    pub fn verify_program(&mut self, program: &mir::Program) -> Result<Vec<VerificationResult>, SemanticError> {
        let mut results = Vec::new();
        
        // Verify each function
        for (name, function) in &program.functions {
            let result = self.verify_function(name, function)?;
            results.push(result);
        }
        
        Ok(results)
    }
    
    /// Verify a single function
    pub fn verify_function(&mut self, name: &str, function: &mir::Function) -> Result<VerificationResult, SemanticError> {
        self.context.current_function = Some(name.to_string());
        
        // Get function contract if it exists
        let contract = self.context.function_contracts.get(name).cloned();
        
        // Generate verification conditions
        let conditions = self.vcgen.generate_function_vcs(function, contract.as_ref())?;
        
        // Verify each condition
        let mut condition_results = Vec::new();
        let mut counterexamples = Vec::new();
        let mut all_verified = true;
        
        for vc in conditions {
            let start_time = std::time::Instant::now();
            
            match self.solver.check_condition(&vc) {
                Ok(solver::CheckResult::Verified) => {
                    condition_results.push(ConditionResult {
                        name: vc.name.clone(),
                        condition: vc.formula.to_string(),
                        verified: true,
                        location: vc.location.clone(),
                        verification_time_ms: start_time.elapsed().as_millis() as u64,
                    });
                }
                Ok(solver::CheckResult::Failed(model)) => {
                    all_verified = false;
                    
                    condition_results.push(ConditionResult {
                        name: vc.name.clone(),
                        condition: vc.formula.to_string(),
                        verified: false,
                        location: vc.location.clone(),
                        verification_time_ms: start_time.elapsed().as_millis() as u64,
                    });
                    
                    // Extract counterexample
                    let counterexample = self.extract_counterexample(&vc, model);
                    counterexamples.push(counterexample);
                }
                Err(e) => {
                    return Err(SemanticError::VerificationError {
                        message: format!("Failed to verify condition '{}': {}", vc.name, e),
                        location: vc.location,
                    });
                }
            }
        }
        
        Ok(VerificationResult {
            name: name.to_string(),
            verified: all_verified,
            conditions: condition_results,
            counterexamples,
        })
    }
    
    /// Add a function contract
    pub fn add_function_contract(&mut self, name: String, contract: contracts::FunctionContract) {
        self.context.function_contracts.insert(name, contract);
    }
    
    /// Add a loop invariant
    pub fn add_loop_invariant(&mut self, block_id: mir::BasicBlockId, invariant: invariants::LoopInvariant) {
        self.context.loop_invariants.insert(block_id, invariant);
    }
    
    /// Add a global invariant
    pub fn add_global_invariant(&mut self, invariant: invariants::GlobalInvariant) {
        self.context.global_invariants.push(invariant);
    }
    
    /// Extract a counterexample from a failed verification
    fn extract_counterexample(&self, vc: &solver::VerificationCondition, model: solver::Model) -> Counterexample {
        let mut assignments = HashMap::new();
        
        // Extract variable values from the model
        for (var_name, value) in model.assignments {
            assignments.insert(var_name, self.convert_solver_value(value));
        }
        
        Counterexample {
            condition_name: vc.name.clone(),
            assignments,
            trace: model.execution_trace,
        }
    }
    
    /// Convert solver value to our value representation
    fn convert_solver_value(&self, value: solver::SolverValue) -> Value {
        match value {
            solver::SolverValue::Int(n) => Value::Integer(n),
            solver::SolverValue::Real(f) => Value::Float(f),
            solver::SolverValue::Bool(b) => Value::Boolean(b),
            solver::SolverValue::String(s) => Value::String(s),
            solver::SolverValue::Array(values) => {
                Value::Array(values.into_iter().map(|v| self.convert_solver_value(v)).collect())
            }
        }
    }
}

impl Default for VerificationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verification_engine_creation() {
        let engine = VerificationEngine::new();
        assert!(engine.context.current_function.is_none());
        assert!(engine.context.function_contracts.is_empty());
    }
}