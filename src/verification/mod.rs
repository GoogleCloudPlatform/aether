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

//! Formal verification framework for AetherScript
//!
//! Provides contract verification, invariant checking, and property proving

pub mod ast_to_contract;
pub mod contract_to_smt;
pub mod contracts;
pub mod invariants;
pub mod solver;
pub mod vcgen;

use crate::error::{SemanticError, SourceLocation};
use crate::mir;
use serde::{Deserialize, Serialize};
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

/// Verification strategy for a function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMode {
    /// Try abstract verification (with axioms) first, then fall back to instantiation
    Combined,
    /// Use only axioms (abstract verification)
    AbstractOnly,
    /// Ignore axioms (instantiation-only)
    InstantiationOnly,
}

/// Context for verification
#[derive(Debug, Default)]
struct VerificationContext {
    /// Current function being verified
    current_function: Option<String>,

    /// Known function contracts
    function_contracts: HashMap<String, contracts::FunctionContract>,

    /// Verification mode per function
    verification_modes: HashMap<String, VerificationMode>,

    /// Loop invariants
    loop_invariants: HashMap<mir::BasicBlockId, invariants::LoopInvariant>,

    /// Global invariants
    global_invariants: Vec<invariants::GlobalInvariant>,

    /// Global axioms asserted for all verification conditions
    global_axioms: Vec<solver::Formula>,
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
    pub fn verify_program(
        &mut self,
        program: &mir::Program,
    ) -> Result<Vec<VerificationResult>, SemanticError> {
        let mut results = Vec::new();

        // Verify each function
        for (name, function) in &program.functions {
            let result = self.verify_function(name, function)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Verify a single function
    pub fn verify_function(
        &mut self,
        name: &str,
        function: &mir::Function,
    ) -> Result<VerificationResult, SemanticError> {
        self.context.current_function = Some(name.to_string());

        // Get function contract if it exists
        let contract = self.resolve_contract_for(name);

        // Generate verification conditions
        let conditions = self
            .vcgen
            .generate_function_vcs(function, contract.as_ref())?;

        // Get function-level default verification mode
        let function_level_mode = self
            .context
            .verification_modes
            .get(name)
            .cloned()
            .unwrap_or(VerificationMode::Combined);

        let mut all_verified = true;
        let mut all_condition_results = Vec::new();
        let mut all_counterexamples = Vec::new();

        for vc in conditions {
            let mode_for_this_vc = vc.verification_mode.unwrap_or(function_level_mode);

            let cloned_global_axioms = self.context.global_axioms.clone();
            let axioms_for_pass: &[solver::Formula] = match mode_for_this_vc {
                VerificationMode::AbstractOnly => cloned_global_axioms.as_slice(),
                VerificationMode::InstantiationOnly => &[],
                VerificationMode::Combined => cloned_global_axioms.as_slice(),
            };

            let (pass1_results, pass1_counterexamples, pass1_verified) =
                self.verify_pass(&[vc.clone()], axioms_for_pass)?;

            let mut current_vc_verified = pass1_verified;
            let mut current_vc_results = pass1_results;
            let mut current_vc_counterexamples = pass1_counterexamples;

            if mode_for_this_vc == VerificationMode::Combined && !pass1_verified {
                let (pass2_results, pass2_counterexamples, pass2_verified) =
                    self.verify_pass(&[vc.clone()], &[])?;
                current_vc_verified = pass2_verified;
                current_vc_results = pass2_results;
                current_vc_counterexamples = pass2_counterexamples;
            }

            if !current_vc_verified {
                all_verified = false;
            }
            all_condition_results.append(&mut current_vc_results);
            all_counterexamples.append(&mut current_vc_counterexamples);
        }

        Ok(VerificationResult {
            name: name.to_string(),
            verified: all_verified,
            conditions: all_condition_results,
            counterexamples: all_counterexamples,
        })
    }

    /// Add a function contract
    pub fn add_function_contract(
        &mut self,
        name: String,
        mut contract: contracts::FunctionContract,
    ) {
        contract.function_name = name.clone();
        self.context.function_contracts.insert(name, contract);
    }

    /// Add a loop invariant
    pub fn add_loop_invariant(
        &mut self,
        block_id: mir::BasicBlockId,
        invariant: invariants::LoopInvariant,
    ) {
        self.context.loop_invariants.insert(block_id, invariant);
    }

    /// Add a global invariant
    pub fn add_global_invariant(&mut self, invariant: invariants::GlobalInvariant) {
        self.context.global_invariants.push(invariant);
    }

    /// Add a global axiom that will be available to every verification condition.
    pub fn add_axiom(&mut self, axiom: solver::Formula) {
        self.context.global_axioms.push(axiom);
    }

    /// Set verification mode for a function (default is Combined).
    pub fn set_verification_mode(&mut self, name: String, mode: VerificationMode) {
        self.context.verification_modes.insert(name, mode);
    }

    /// Run a single verification pass with the provided axioms.
    fn verify_pass(
        &mut self,
        conditions: &[solver::VerificationCondition],
        axioms: &[solver::Formula],
    ) -> Result<(Vec<ConditionResult>, Vec<Counterexample>, bool), SemanticError> {
        let mut condition_results = Vec::new();
        let mut counterexamples = Vec::new();
        let mut all_verified = true;

        for vc in conditions {
            let start_time = std::time::Instant::now();

            match self.solver.check_condition_with_axioms(vc, axioms) {
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
                    let counterexample = self.extract_counterexample(vc, model);
                    counterexamples.push(counterexample);
                }
                Err(e) => {
                    return Err(SemanticError::VerificationError {
                        message: format!("Failed to verify condition '{}': {}", vc.name, e),
                        location: vc.location.clone(),
                    });
                }
            }
        }

        Ok((condition_results, counterexamples, all_verified))
    }

    /// Extract a counterexample from a failed verification
    fn extract_counterexample(
        &self,
        vc: &solver::VerificationCondition,
        model: solver::Model,
    ) -> Counterexample {
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
            solver::SolverValue::Array(values) => Value::Array(
                values
                    .into_iter()
                    .map(|v| self.convert_solver_value(v))
                    .collect(),
            ),
        }
    }

    /// Resolve a contract for the given function name, falling back to generic bases for monomorphized functions.
    fn resolve_contract_for(&mut self, function_name: &str) -> Option<contracts::FunctionContract> {
        if let Some(contract) = self.context.function_contracts.get(function_name).cloned() {
            return Some(contract);
        }

        // Attempt to match monomorphized names of the form `generic_T1_T2`
        let mut best_match: Option<(usize, contracts::FunctionContract)> = None;
        for (base_name, contract) in &self.context.function_contracts {
            if function_name.starts_with(base_name) {
                let suffix = &function_name[base_name.len()..];
                if suffix.starts_with('_') && !suffix.is_empty() {
                    let base_len = base_name.len();
                    if best_match
                        .as_ref()
                        .is_none_or(|(existing_len, _)| base_len > *existing_len)
                    {
                        let mut cloned = contract.clone();
                        cloned.function_name = function_name.to_string();
                        best_match = Some((base_len, cloned));
                    }
                }
            }
        }

        if let Some((_, resolved)) = best_match {
            // Cache the resolved contract for future lookups.
            self.context
                .function_contracts
                .insert(function_name.to_string(), resolved.clone());
            Some(resolved)
        } else {
            None
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
    use crate::ast::PrimitiveType;
    use crate::mir;
    use crate::mir::{
        BasicBlock, Function, Operand, Parameter, Place, Program, Rvalue, Statement, Terminator,
    };
    use crate::types::Type;
    use std::collections::HashMap;

    #[test]
    fn test_verification_engine_creation() {
        let engine = VerificationEngine::new();
        assert!(engine.context.current_function.is_none());
        assert!(engine.context.function_contracts.is_empty());
    }

    #[test]
    fn applies_contracts_to_monomorphized_functions() {
        let mut engine = VerificationEngine::new();

        // Contract on the generic function: result must equal input parameter.
        let mut contract = contracts::FunctionContract::new("identity".to_string());
        contract.add_postcondition(
            "returns_input".to_string(),
            contracts::Expression::BinaryOp {
                op: contracts::BinaryOp::Eq,
                left: Box::new(contracts::Expression::Result),
                right: Box::new(contracts::Expression::Variable("x".to_string())),
            },
            SourceLocation::unknown(),
        );
        engine.add_function_contract("identity".to_string(), contract);

        // Minimal MIR for the generic function and a monomorphized instantiation.
        let int_type = Type::primitive(PrimitiveType::Integer);
        let parameters = vec![Parameter {
            name: "x".to_string(),
            ty: int_type.clone(),
            local_id: 0,
        }];
        let entry_block = BasicBlock {
            id: 0,
            statements: vec![],
            terminator: Terminator::Return,
        };

        let mut base_blocks = HashMap::new();
        base_blocks.insert(0, entry_block.clone());
        let mut instantiated_blocks = HashMap::new();
        instantiated_blocks.insert(0, entry_block);

        let base_function = Function {
            name: "identity".to_string(),
            parameters: parameters.clone(),
            return_type: int_type.clone(),
            locals: HashMap::new(),
            basic_blocks: base_blocks,
            entry_block: 0,
            return_local: Some(0),
        };

        let instantiated_function = Function {
            name: "identity_Int".to_string(),
            parameters,
            return_type: int_type,
            locals: HashMap::new(),
            basic_blocks: instantiated_blocks,
            entry_block: 0,
            return_local: Some(0),
        };

        let mut program = Program {
            functions: HashMap::new(),
            global_constants: HashMap::new(),
            external_functions: HashMap::new(),
            type_definitions: HashMap::new(),
        };
        program
            .functions
            .insert("identity".to_string(), base_function);
        program
            .functions
            .insert("identity_Int".to_string(), instantiated_function);

        let results = engine.verify_program(&program).unwrap();

        // The monomorphized function should reuse the generic contract.
        let instantiation = results
            .iter()
            .find(|res| res.name == "identity_Int")
            .expect("expected verification result for monomorphized function");
        assert!(!instantiation.conditions.is_empty());
        assert!(instantiation.verified);
    }

    #[test]
    fn applies_global_axioms_during_verification() {
        let int_type = Type::primitive(PrimitiveType::Integer);

        // Function: returns x but contract requires result == y, which needs an axiom.
        let parameters = vec![
            Parameter {
                name: "x".to_string(),
                ty: int_type.clone(),
                local_id: 0,
            },
            Parameter {
                name: "y".to_string(),
                ty: int_type.clone(),
                local_id: 1,
            },
        ];

        let assign_return = Statement::Assign {
            place: Place {
                local: 2,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Copy(Place {
                local: 0,
                projection: vec![],
            })),
            source_info: mir::SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        };

        let entry_block = BasicBlock {
            id: 0,
            statements: vec![assign_return],
            terminator: Terminator::Return,
        };

        let mut blocks = HashMap::new();
        blocks.insert(0, entry_block);

        let function = Function {
            name: "needs_axiom".to_string(),
            parameters: parameters.clone(),
            return_type: int_type.clone(),
            locals: {
                let mut locals = HashMap::new();
                locals.insert(
                    2,
                    mir::Local {
                        ty: int_type.clone(),
                        is_mutable: false,
                        source_info: None,
                    },
                );
                locals
            },
            basic_blocks: blocks,
            entry_block: 0,
            return_local: Some(2),
        };

        let mut contract = contracts::FunctionContract::new("needs_axiom".to_string());
        contract.add_postcondition(
            "result_matches_y".to_string(),
            contracts::Expression::BinaryOp {
                op: contracts::BinaryOp::Eq,
                left: Box::new(contracts::Expression::Result),
                right: Box::new(contracts::Expression::Variable("y".to_string())),
            },
            SourceLocation::unknown(),
        );

        // Without the axiom, verification should fail.
        let mut engine_without_axiom = VerificationEngine::new();
        engine_without_axiom.add_function_contract("needs_axiom".to_string(), contract.clone());
        let failed = engine_without_axiom
            .verify_function("needs_axiom", &function)
            .unwrap();
        assert!(!failed.verified);
        assert!(!failed.conditions.is_empty());
        assert!(!failed.conditions[0].verified);

        // Add an axiom equating x and y (locals 0 and 1), enabling the postcondition to prove.
        let mut engine_with_axiom = VerificationEngine::new();
        engine_with_axiom.add_function_contract("needs_axiom".to_string(), contract);
        engine_with_axiom.add_axiom(solver::Formula::Eq(
            Box::new(solver::Formula::Var("local_0".to_string())),
            Box::new(solver::Formula::Var("local_1".to_string())),
        ));

        let verified = engine_with_axiom
            .verify_function("needs_axiom", &function)
            .unwrap();
        assert!(verified.verified);
        assert!(!verified.conditions.is_empty());
        assert!(verified.conditions[0].verified);
    }

    #[test]
    fn combined_mode_falls_back_without_axioms() {
        let int_type = Type::primitive(PrimitiveType::Integer);

        // Function returns x directly.
        let parameters = vec![Parameter {
            name: "x".to_string(),
            ty: int_type.clone(),
            local_id: 0,
        }];

        let assign_return = Statement::Assign {
            place: Place {
                local: 1,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Copy(Place {
                local: 0,
                projection: vec![],
            })),
            source_info: mir::SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        };

        let entry_block = BasicBlock {
            id: 0,
            statements: vec![assign_return],
            terminator: Terminator::Return,
        };

        let mut blocks = HashMap::new();
        blocks.insert(0, entry_block);

        let function = Function {
            name: "combined_example".to_string(),
            parameters: parameters.clone(),
            return_type: int_type.clone(),
            locals: {
                let mut locals = HashMap::new();
                locals.insert(
                    1,
                    mir::Local {
                        ty: int_type.clone(),
                        is_mutable: false,
                        source_info: None,
                    },
                );
                locals
            },
            basic_blocks: blocks,
            entry_block: 0,
            return_local: Some(1),
        };

        let mut contract = contracts::FunctionContract::new("combined_example".to_string());
        contract.add_enhanced_postcondition(
            "result_matches_x".to_string(),
            contracts::Expression::BinaryOp {
                op: contracts::BinaryOp::Eq,
                left: Box::new(contracts::Expression::Result),
                right: Box::new(contracts::Expression::Variable("x".to_string())),
            },
            SourceLocation::unknown(),
            None,
            contracts::FailureAction::ThrowException("Postcondition violation".to_string()),
            contracts::VerificationHint::SMTSolver,
            None, // Default to Combined mode for this condition
        );

        // Add a contradictory axiom to force failure in the abstract pass for the default combined mode.
        let mut engine = VerificationEngine::new();
        engine.add_function_contract("combined_example".to_string(), contract);
        engine.add_axiom(solver::Formula::Not(Box::new(solver::Formula::Eq(
            Box::new(solver::Formula::Var("result".to_string())),
            Box::new(solver::Formula::Var("local_0".to_string())),
        ))));

        // Default mode is Combined: first pass with axioms should fail, fallback without axioms should pass.
        let result = engine
            .verify_function("combined_example", &function)
            .unwrap();
        assert!(result.verified);
        assert!(!result.conditions.is_empty());
        assert!(result.conditions[0].verified);
    }

    #[test]
    fn test_verification_modes_with_pragmas() {
        let int_type = Type::primitive(PrimitiveType::Integer);

        // A simple function: `fn test(x: Int, y: Int) -> Int { return x; }`
        let parameters = vec![
            Parameter {
                name: "x".to_string(),
                ty: int_type.clone(),
                local_id: 0,
            },
            Parameter {
                name: "y".to_string(),
                ty: int_type.clone(),
                local_id: 1,
            },
        ];
        let assign_return = Statement::Assign {
            place: Place {
                local: 2,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Copy(Place {
                local: 0,
                projection: vec![],
            })),
            source_info: mir::SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        };
        let entry_block = BasicBlock {
            id: 0,
            statements: vec![assign_return],
            terminator: Terminator::Return,
        };
        let mut blocks = HashMap::new();
        blocks.insert(0, entry_block);
        let function = Function {
            name: "test_modes".to_string(),
            parameters: parameters.clone(),
            return_type: int_type.clone(),
            locals: {
                let mut locals = HashMap::new();
                locals.insert(
                    2,
                    mir::Local {
                        ty: int_type.clone(),
                        is_mutable: false,
                        source_info: None,
                    },
                );
                locals
            },
            basic_blocks: blocks,
            entry_block: 0,
            return_local: Some(2),
        };

        let mut contract = contracts::FunctionContract::new("test_modes".to_string());

        // Condition 1: Requires axiom to pass (result == y). Explicitly set to AbstractOnly.
        // This condition should fail if only instantiation is used, but pass with abstract + axiom.
        contract.add_enhanced_postcondition(
            "axiom_dependent_abstract_only".to_string(),
            contracts::Expression::BinaryOp {
                op: contracts::BinaryOp::Eq,
                left: Box::new(contracts::Expression::Result),
                right: Box::new(contracts::Expression::Variable("y".to_string())),
            },
            SourceLocation::unknown(),
            None,
            contracts::FailureAction::ThrowException("AbstractOnly violation".to_string()),
            contracts::VerificationHint::SMTSolver,
            Some(VerificationMode::AbstractOnly),
        );

        // Condition 2: Fails with axiom, passes without (result == x). Explicitly set to InstantiationOnly.
        // This condition should pass with instantiation, but fail with abstract + contradictory axiom.
        contract.add_enhanced_postcondition(
            "instantiation_only_passes".to_string(),
            contracts::Expression::BinaryOp {
                op: contracts::BinaryOp::Eq,
                left: Box::new(contracts::Expression::Result),
                right: Box::new(contracts::Expression::Variable("x".to_string())),
            },
            SourceLocation::unknown(),
            None,
            contracts::FailureAction::ThrowException("InstantiationOnly violation".to_string()),
            contracts::VerificationHint::SMTSolver,
            Some(VerificationMode::InstantiationOnly),
        );

        // Condition 3: Should pass with Combined mode. (result == x). No specific mode set, defaults to Combined.
        contract.add_enhanced_postcondition(
            "combined_mode_default".to_string(),
            contracts::Expression::BinaryOp {
                op: contracts::BinaryOp::Eq,
                left: Box::new(contracts::Expression::Result),
                right: Box::new(contracts::Expression::Variable("x".to_string())),
            },
            SourceLocation::unknown(),
            None,
            contracts::FailureAction::ThrowException("CombinedMode violation".to_string()),
            contracts::VerificationHint::SMTSolver,
            None,
        );

        let mut engine = VerificationEngine::new();
        engine.add_function_contract("test_modes".to_string(), contract);

        // Add an axiom: x == y (local_0 == local_1). This helps 'axiom_dependent_abstract_only' but hurts 'instantiation_only_passes' if used in instantiation.
        engine.add_axiom(solver::Formula::Eq(
            Box::new(solver::Formula::Var("local_0".to_string())),
            Box::new(solver::Formula::Var("local_1".to_string())),
        ));

        // For the function as a whole, set default to Combined (though individual conditions override)
        engine.set_verification_mode("test_modes".to_string(), VerificationMode::Combined);

        let result = engine.verify_function("test_modes", &function).unwrap();

        assert!(result.verified);
        assert_eq!(result.conditions.len(), 3);

        let c1 = result
            .conditions
            .iter()
            .find(|c| c.name.contains("axiom_dependent_abstract_only"))
            .unwrap();
        assert!(c1.verified, "AbstractOnly condition should pass with axiom");

        let c2 = result
            .conditions
            .iter()
            .find(|c| c.name.contains("instantiation_only_passes"))
            .unwrap();
        assert!(
            c2.verified,
            "InstantiationOnly condition should pass without contradictory axiom"
        );

        let c3 = result
            .conditions
            .iter()
            .find(|c| c.name.contains("combined_mode_default"))
            .unwrap();
        assert!(c3.verified, "CombinedMode (default) condition should pass");
    }
}
