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

//! Enhanced Function Contracts for LLM-First Verification
//!
//! Implements rich contract specifications including preconditions, postconditions,
//! invariants, and semantic metadata for formal verification.
//!
//! This module is part of the LLM-first design where contracts are:
//! - Explicitly stated with proof hints
//! - Automatically verified at compile time when possible
//! - Include semantic intent validation
//! - Support compositional reasoning

use crate::ast;
use crate::error::SourceLocation;
use crate::semantic::metadata::{BehavioralSpec, IntentSpec, ResourceContract};
use crate::types::Type;
use std::collections::{HashMap, HashSet};

/// Enhanced Function Contract for LLM-First Verification
#[derive(Debug, Clone)]
pub struct FunctionContract {
    /// Function name
    pub function_name: String,

    /// Preconditions (requires clauses) with proof hints
    pub preconditions: Vec<EnhancedCondition>,

    /// Postconditions (ensures clauses) with proof hints
    pub postconditions: Vec<EnhancedCondition>,

    /// Invariants that must hold throughout execution
    pub invariants: Vec<EnhancedCondition>,

    /// Variables that may be modified
    pub modifies: HashSet<String>,

    /// Pure function flag (no side effects)
    pub is_pure: bool,

    /// Termination measure for proving termination
    pub decreases: Option<Expression>,

    /// Semantic intent specification
    pub intent: Option<IntentSpec>,

    /// Behavioral specification
    pub behavior: Option<BehavioralSpec>,

    /// Resource usage contract
    pub resources: Option<ResourceContract>,

    /// Failure actions for contract violations
    pub failure_actions: HashMap<String, FailureAction>,

    /// Contract propagation rules
    pub propagation: ContractPropagation,

    /// Proof obligations generated from this contract
    pub proof_obligations: Vec<ProofObligation>,
}

/// Enhanced condition with proof hints for LLM understanding
#[derive(Debug, Clone)]
pub struct EnhancedCondition {
    /// Condition name/label
    pub name: String,

    /// The actual condition expression
    pub expression: Expression,

    /// Source location
    pub location: SourceLocation,

    /// Proof hint for LLM understanding
    pub proof_hint: Option<String>,

    /// Failure action if condition is violated
    pub failure_action: FailureAction,

    /// Verification method hint
    pub verification_hint: VerificationHint,
}

/// Original condition structure for backwards compatibility
#[derive(Debug, Clone)]
pub struct Condition {
    /// Condition name/label
    pub name: String,

    /// The actual condition expression
    pub expression: Expression,

    /// Source location
    pub location: SourceLocation,
}

/// Failure action when a contract is violated
#[derive(Debug, Clone)]
pub enum FailureAction {
    /// Throw an exception with the given message
    ThrowException(String),
    /// Return an error value
    ReturnError(String),
    /// Log and continue
    LogAndContinue,
    /// Abort execution
    Abort,
    /// Custom handler function
    CustomHandler(String),
}

/// Hint for verification method
#[derive(Debug, Clone)]
pub enum VerificationHint {
    /// Use SMT solver
    SMTSolver,
    /// Use symbolic execution
    SymbolicExecution,
    /// Use abstract interpretation
    AbstractInterpretation,
    /// Simple static check
    StaticCheck,
    /// Runtime check only
    RuntimeOnly,
    /// Custom verification method
    Custom(String),
}

/// Contract propagation rules
#[derive(Debug, Clone, Default)]
pub struct ContractPropagation {
    /// Propagate preconditions to callers
    pub propagate_preconditions: bool,
    /// Combine postconditions in sequence
    pub combine_postconditions: bool,
    /// Maintain invariants through calls
    pub maintain_invariants: bool,
    /// Custom propagation rules
    pub custom_rules: Vec<String>,
}

/// Proof obligation generated from contracts
#[derive(Debug, Clone)]
pub struct ProofObligation {
    /// Unique identifier
    pub id: String,
    /// Description of what needs to be proved
    pub description: String,
    /// The formula to prove
    pub formula: Expression,
    /// Context assumptions
    pub assumptions: Vec<Expression>,
    /// Verification method to use
    pub method: VerificationMethod,
    /// Priority for verification
    pub priority: VerificationPriority,
}

/// Verification method for proof obligations
#[derive(Debug, Clone)]
pub enum VerificationMethod {
    /// Direct proof using SMT solver
    DirectProof,
    /// Proof by induction
    Induction,
    /// Proof by contradiction
    Contradiction,
    /// Model checking
    ModelChecking,
    /// Symbolic execution
    SymbolicExecution,
    /// Use Z3 SMT solver specifically
    Z3Solver,
}

/// Priority for verification
#[derive(Debug, Clone, Copy)]
pub enum VerificationPriority {
    Critical, // Must verify for correctness
    High,     // Should verify for safety
    Medium,   // Good to verify
    Low,      // Optional verification
}

/// Expression in contracts
#[derive(Debug, Clone)]
pub enum Expression {
    /// Variable reference
    Variable(String),

    /// Constant value
    Constant(ConstantValue),

    /// Binary operation
    BinaryOp {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expression>,
    },

    /// Function call (for pure functions)
    Call {
        function: String,
        args: Vec<Expression>,
    },

    /// Array access
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },

    /// Field access
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },

    /// Quantifier
    Quantifier {
        kind: QuantifierKind,
        variables: Vec<(String, Type)>,
        body: Box<Expression>,
    },

    /// Old value (for postconditions)
    Old(Box<Expression>),

    /// Result value (for postconditions)
    Result,

    /// Array/sequence length
    Length(Box<Expression>),

    /// Type predicate
    IsType { expr: Box<Expression>, ty: Type },

    // Enhanced expressions for LLM-first contracts
    /// Semantic predicate (e.g., "is_valid_email")
    SemanticPredicate {
        predicate: String,
        args: Vec<Expression>,
    },

    /// Temporal operator (for invariants)
    Temporal {
        op: TemporalOp,
        expr: Box<Expression>,
    },

    /// Set membership
    InSet {
        element: Box<Expression>,
        set: Box<Expression>,
    },

    /// Range expression
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },

    /// Pattern matching
    Matches {
        expr: Box<Expression>,
        pattern: String,
    },

    /// Aggregate operations
    Aggregate {
        op: AggregateOp,
        collection: Box<Expression>,
        predicate: Option<Box<Expression>>,
    },

    /// Let binding for local definitions
    Let {
        bindings: Vec<(String, Expression)>,
        body: Box<Expression>,
    },
}

/// Constant values in contracts
#[derive(Debug, Clone)]
pub enum ConstantValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
}

/// Binary operators in contracts
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,
    Implies,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
}

/// Unary operators in contracts
#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    // Arithmetic
    Neg,

    // Logical
    Not,

    // Bitwise
    BitNot,
}

/// Quantifier kinds
#[derive(Debug, Clone, Copy)]
pub enum QuantifierKind {
    Forall,
    Exists,
}

/// Temporal operators for invariants
#[derive(Debug, Clone, Copy)]
pub enum TemporalOp {
    /// Always (globally)
    Always,
    /// Eventually
    Eventually,
    /// Until
    Until,
    /// Since
    Since,
    /// Next
    Next,
}

/// Aggregate operations for collections
#[derive(Debug, Clone, Copy)]
pub enum AggregateOp {
    /// Sum of elements
    Sum,
    /// Product of elements
    Product,
    /// Count of elements
    Count,
    /// Minimum element
    Min,
    /// Maximum element
    Max,
    /// All elements satisfy predicate
    All,
    /// Any element satisfies predicate
    Any,
    /// Average of elements
    Average,
}

impl FunctionContract {
    /// Create a new empty contract with LLM-first defaults
    pub fn new(function_name: String) -> Self {
        Self {
            function_name,
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
            modifies: HashSet::new(),
            is_pure: false,
            decreases: None,
            intent: None,
            behavior: None,
            resources: None,
            failure_actions: HashMap::new(),
            propagation: ContractPropagation {
                propagate_preconditions: true,
                combine_postconditions: true,
                maintain_invariants: true,
                custom_rules: Vec::new(),
            },
            proof_obligations: Vec::new(),
        }
    }

    /// Add an enhanced precondition with proof hint
    pub fn add_enhanced_precondition(
        &mut self,
        name: String,
        expr: Expression,
        location: SourceLocation,
        proof_hint: Option<String>,
        failure_action: FailureAction,
        verification_hint: VerificationHint,
    ) {
        self.preconditions.push(EnhancedCondition {
            name: name.clone(),
            expression: expr,
            location,
            proof_hint,
            failure_action: failure_action.clone(),
            verification_hint,
        });

        // Store failure action for quick lookup
        self.failure_actions.insert(name, failure_action);
    }

    /// Add a simple precondition (for backwards compatibility)
    pub fn add_precondition(&mut self, name: String, expr: Expression, location: SourceLocation) {
        self.add_enhanced_precondition(
            name,
            expr,
            location,
            None,
            FailureAction::ThrowException("Precondition violation".to_string()),
            VerificationHint::SMTSolver,
        );
    }

    /// Add an enhanced postcondition with proof hint
    pub fn add_enhanced_postcondition(
        &mut self,
        name: String,
        expr: Expression,
        location: SourceLocation,
        proof_hint: Option<String>,
        failure_action: FailureAction,
        verification_hint: VerificationHint,
    ) {
        self.postconditions.push(EnhancedCondition {
            name: name.clone(),
            expression: expr,
            location,
            proof_hint,
            failure_action: failure_action.clone(),
            verification_hint,
        });

        self.failure_actions.insert(name, failure_action);
    }

    /// Add a simple postcondition (for backwards compatibility)
    pub fn add_postcondition(&mut self, name: String, expr: Expression, location: SourceLocation) {
        self.add_enhanced_postcondition(
            name,
            expr,
            location,
            None,
            FailureAction::ThrowException("Postcondition violation".to_string()),
            VerificationHint::SMTSolver,
        );
    }

    /// Add an invariant that must hold throughout execution
    pub fn add_invariant(
        &mut self,
        name: String,
        expr: Expression,
        location: SourceLocation,
        proof_hint: Option<String>,
    ) {
        self.invariants.push(EnhancedCondition {
            name,
            expression: expr,
            location,
            proof_hint,
            failure_action: FailureAction::Abort,
            verification_hint: VerificationHint::SMTSolver,
        });
    }

    /// Add a variable to the modifies set
    pub fn add_modifies(&mut self, var: String) {
        self.modifies.insert(var);
    }

    /// Mark function as pure
    pub fn set_pure(&mut self, is_pure: bool) {
        self.is_pure = is_pure;
    }

    /// Set termination measure
    pub fn set_decreases(&mut self, expr: Expression) {
        self.decreases = Some(expr);
    }

    /// Set semantic intent
    pub fn set_intent(&mut self, intent: IntentSpec) {
        self.intent = Some(intent);
    }

    /// Set behavioral specification
    pub fn set_behavior(&mut self, behavior: BehavioralSpec) {
        self.behavior = Some(behavior);
    }

    /// Set resource contract
    pub fn set_resources(&mut self, resources: ResourceContract) {
        self.resources = Some(resources);
    }

    /// Generate proof obligations from this contract
    pub fn generate_proof_obligations(&mut self) -> Vec<ProofObligation> {
        let mut obligations = Vec::new();
        let mut _obligation_id = 0;

        // Generate obligations for preconditions
        for (i, pre) in self.preconditions.iter().enumerate() {
            _obligation_id += 1;
            obligations.push(ProofObligation {
                id: format!("{}_pre_{}", self.function_name, i),
                description: format!("Precondition '{}' is satisfiable", pre.name),
                formula: Expression::Quantifier {
                    kind: QuantifierKind::Exists,
                    variables: vec![], // Would be populated with function parameters
                    body: Box::new(pre.expression.clone()),
                },
                assumptions: vec![],
                method: match &pre.verification_hint {
                    VerificationHint::SMTSolver => VerificationMethod::Z3Solver,
                    VerificationHint::SymbolicExecution => VerificationMethod::SymbolicExecution,
                    _ => VerificationMethod::DirectProof,
                },
                priority: VerificationPriority::High,
            });
        }

        // Generate obligations for postconditions
        for (i, post) in self.postconditions.iter().enumerate() {
            _obligation_id += 1;

            // Collect preconditions as assumptions
            let assumptions: Vec<Expression> = self
                .preconditions
                .iter()
                .map(|pre| pre.expression.clone())
                .collect();

            obligations.push(ProofObligation {
                id: format!("{}_post_{}", self.function_name, i),
                description: format!(
                    "Postcondition '{}' holds when preconditions are met",
                    post.name
                ),
                formula: Expression::BinaryOp {
                    op: BinaryOp::Implies,
                    left: Box::new(conjunction(assumptions.clone())),
                    right: Box::new(post.expression.clone()),
                },
                assumptions,
                method: match &post.verification_hint {
                    VerificationHint::SMTSolver => VerificationMethod::Z3Solver,
                    _ => VerificationMethod::DirectProof,
                },
                priority: VerificationPriority::Critical,
            });
        }

        // Store generated obligations
        self.proof_obligations = obligations.clone();
        obligations
    }
}

/// Helper to create a conjunction of expressions
fn conjunction(exprs: Vec<Expression>) -> Expression {
    if exprs.is_empty() {
        Expression::Constant(ConstantValue::Boolean(true))
    } else if exprs.len() == 1 {
        exprs.into_iter().next().unwrap()
    } else {
        exprs
            .into_iter()
            .reduce(|acc, expr| Expression::BinaryOp {
                op: BinaryOp::And,
                left: Box::new(acc),
                right: Box::new(expr),
            })
            .unwrap()
    }
}

impl Expression {
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            Expression::Variable(name) => name.clone(),
            Expression::Constant(c) => match c {
                ConstantValue::Integer(n) => n.to_string(),
                ConstantValue::Float(f) => f.to_string(),
                ConstantValue::Boolean(b) => b.to_string(),
                ConstantValue::String(s) => format!("{}", s),
                ConstantValue::Null => "null".to_string(),
            },
            Expression::BinaryOp { op, left, right } => {
                format!(
                    "({} {} {})",
                    left.to_string(),
                    op.to_string(),
                    right.to_string()
                )
            }
            Expression::UnaryOp { op, operand } => {
                format!("({} {})", op.to_string(), operand.to_string())
            }
            Expression::Call { function, args } => {
                let args_str = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", function, args_str)
            }
            Expression::ArrayAccess { array, index } => {
                format!("{}[{}]", array.to_string(), index.to_string())
            }
            Expression::FieldAccess { object, field } => {
                format!("{}.{}", object.to_string(), field)
            }
            Expression::Quantifier {
                kind,
                variables,
                body,
            } => {
                let vars_str = variables
                    .iter()
                    .map(|(name, _)| name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} {}. {}", kind.to_string(), vars_str, body.to_string())
            }
            Expression::Old(expr) => format!("old({})", expr.to_string()),
            Expression::Result => "result".to_string(),
            Expression::Length(expr) => format!("len({})", expr.to_string()),
            Expression::IsType { expr, ty } => format!("is_type({}, {:?})", expr.to_string(), ty),

            // Enhanced expressions
            Expression::SemanticPredicate { predicate, args } => {
                let args_str = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", predicate, args_str)
            }
            Expression::Temporal { op, expr } => {
                format!("{} {}", op.to_string(), expr.to_string())
            }
            Expression::InSet { element, set } => {
                format!("{} in {}", element.to_string(), set.to_string())
            }
            Expression::Range {
                start,
                end,
                inclusive,
            } => {
                if *inclusive {
                    format!("[{}, {}]", start.to_string(), end.to_string())
                } else {
                    format!("[{}, {})", start.to_string(), end.to_string())
                }
            }
            Expression::Matches { expr, pattern } => {
                format!("{} matches \"{}\"", expr.to_string(), pattern)
            }
            Expression::Aggregate {
                op,
                collection,
                predicate,
            } => match predicate {
                Some(pred) => format!(
                    "{}({} | {})",
                    op.to_string(),
                    collection.to_string(),
                    pred.to_string()
                ),
                None => format!("{}({})", op.to_string(), collection.to_string()),
            },
            Expression::Let { bindings, body } => {
                let bindings_str = bindings
                    .iter()
                    .map(|(name, expr)| format!("{} = {}", name, expr.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("let {} in {}", bindings_str, body.to_string())
            }
        }
    }
}

impl BinaryOp {
    pub fn to_string(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::Implies => ">=<",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
        }
    }
}

impl UnaryOp {
    pub fn to_string(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
        }
    }
}

impl QuantifierKind {
    pub fn to_string(&self) -> &'static str {
        match self {
            QuantifierKind::Forall => "forall",
            QuantifierKind::Exists => "exists",
        }
    }
}

impl TemporalOp {
    pub fn to_string(&self) -> &'static str {
        match self {
            TemporalOp::Always => "always",
            TemporalOp::Eventually => "eventually",
            TemporalOp::Until => "until",
            TemporalOp::Since => "since",
            TemporalOp::Next => "next",
        }
    }
}

impl AggregateOp {
    pub fn to_string(&self) -> &'static str {
        match self {
            AggregateOp::Sum => "sum",
            AggregateOp::Product => "product",
            AggregateOp::Count => "count",
            AggregateOp::Min => "min",
            AggregateOp::Max => "max",
            AggregateOp::All => "all",
            AggregateOp::Any => "any",
            AggregateOp::Average => "avg",
        }
    }
}

/// Parse contracts from AST annotations (original version)
pub fn parse_contracts(_function: &ast::Function) -> Option<FunctionContract> {
    // In a real implementation, we would parse contract annotations
    // from the AST (e.g., from specially formatted comments or attributes)

    // For now, return None as contracts aren't yet part of the AST
    None
}

/// Parse enhanced LLM-first contracts from AST
pub fn parse_enhanced_contracts(function: &ast::Function) -> Option<FunctionContract> {
    let _contract = FunctionContract::new(function.name.name.clone());

    // In a full implementation, we would parse:
    // 1. PRECONDITION nodes with PROOF_HINT and FAILURE_ACTION
    // 2. POSTCONDITION nodes with semantic predicates
    // 3. INVARIANT nodes with temporal operators
    // 4. INTENT specifications
    // 5. BEHAVIORAL_SPEC metadata
    // 6. RESOURCE_CONTRACT specifications

    // Example of what we would parse:
    // (PRECONDITION
    //   (PREDICATE_NOT_EQUALS denominator 0.0)
    //   (FAILURE_ACTION THROW_EXCEPTION)
    //   (PROOF_HINT "denominator != 0 is checked before division"))

    // For now, return None until AST supports these constructs
    None
}

/// Contract verifier with enhanced LLM-first features
pub struct EnhancedContractVerifier {
    /// SMT solver interface
    smt_solver: Option<Box<dyn SmtSolverInterface>>,
    /// Contract repository
    contracts: HashMap<String, FunctionContract>,
    /// Verification cache
    cache: HashMap<String, VerificationResult>,
}

/// Interface for SMT solvers
pub trait SmtSolverInterface: Send + Sync {
    /// Check if a formula is satisfiable
    fn check_sat(&mut self, formula: &Expression) -> Result<SatResult, String>;
    /// Get a model if satisfiable
    fn get_model(&mut self) -> Result<HashMap<String, ConstantValue>, String>;
    /// Add an assertion
    fn assert(&mut self, formula: &Expression) -> Result<(), String>;
    /// Push a new context
    fn push(&mut self) -> Result<(), String>;
    /// Pop a context
    fn pop(&mut self) -> Result<(), String>;
}

/// Satisfiability result
#[derive(Debug, Clone)]
pub enum SatResult {
    Sat,
    Unsat,
    Unknown,
    Timeout,
}

/// Verification result for a contract
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Is the contract verified?
    pub verified: bool,
    /// Individual condition results
    pub conditions: Vec<ConditionResult>,
    /// Counterexamples if any
    pub counterexamples: Vec<Counterexample>,
    /// Proof certificates if generated
    pub proofs: Vec<ProofCertificate>,
}

/// Result for a single condition
#[derive(Debug, Clone)]
pub struct ConditionResult {
    pub condition_name: String,
    pub verified: bool,
    pub verification_time_ms: u64,
    pub method_used: VerificationMethod,
}

/// Counterexample for a failed condition
#[derive(Debug, Clone)]
pub struct Counterexample {
    pub condition_name: String,
    pub variable_assignments: HashMap<String, ConstantValue>,
    pub execution_trace: Vec<String>,
}

/// Proof certificate for verified conditions
#[derive(Debug, Clone)]
pub struct ProofCertificate {
    pub condition_name: String,
    pub proof_steps: Vec<String>,
    pub assumptions_used: Vec<String>,
    pub method: VerificationMethod,
}

impl EnhancedContractVerifier {
    pub fn new() -> Self {
        Self {
            smt_solver: None,
            contracts: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    /// Set the SMT solver to use
    pub fn set_solver(&mut self, solver: Box<dyn SmtSolverInterface>) {
        self.smt_solver = Some(solver);
    }

    /// Add a contract to verify
    pub fn add_contract(&mut self, contract: FunctionContract) {
        self.contracts
            .insert(contract.function_name.clone(), contract);
    }

    /// Verify all contracts
    pub fn verify_all(&mut self) -> HashMap<String, VerificationResult> {
        let mut results = HashMap::new();

        // Collect contract names first to avoid borrowing issues
        let contract_names: Vec<String> = self.contracts.keys().cloned().collect();

        for name in contract_names {
            if let Some(cached) = self.cache.get(&name) {
                results.insert(name.clone(), cached.clone());
            } else if self.contracts.contains_key(&name) {
                let contract = self.contracts.get(&name).unwrap().clone();
                let result = self.verify_contract(&contract);
                self.cache.insert(name.clone(), result.clone());
                results.insert(name, result);
            }
        }

        results
    }

    /// Verify a single contract
    fn verify_contract(&mut self, contract: &FunctionContract) -> VerificationResult {
        let mut conditions = Vec::new();
        let mut counterexamples = Vec::new();
        let mut proofs = Vec::new();
        let mut all_verified = true;

        // Verify preconditions
        for pre in &contract.preconditions {
            let (verified, time_ms, proof) = self.verify_condition(&pre, &contract.preconditions);
            conditions.push(ConditionResult {
                condition_name: pre.name.clone(),
                verified,
                verification_time_ms: time_ms,
                method_used: VerificationMethod::Z3Solver,
            });

            if !verified {
                all_verified = false;
                // Generate counterexample
                if let Some(solver) = &mut self.smt_solver {
                    if let Ok(model) = solver.get_model() {
                        counterexamples.push(Counterexample {
                            condition_name: pre.name.clone(),
                            variable_assignments: model,
                            execution_trace: vec!["Precondition check failed".to_string()],
                        });
                    }
                }
            } else if let Some(p) = proof {
                proofs.push(p);
            }
        }

        // Verify postconditions with preconditions as assumptions
        for post in &contract.postconditions {
            let (verified, time_ms, proof) =
                self.verify_postcondition(&post, &contract.preconditions);
            conditions.push(ConditionResult {
                condition_name: post.name.clone(),
                verified,
                verification_time_ms: time_ms,
                method_used: VerificationMethod::Z3Solver,
            });

            if !verified {
                all_verified = false;
            } else if let Some(p) = proof {
                proofs.push(p);
            }
        }

        VerificationResult {
            verified: all_verified,
            conditions,
            counterexamples,
            proofs,
        }
    }

    /// Verify a single condition
    fn verify_condition(
        &mut self,
        condition: &EnhancedCondition,
        _context: &[EnhancedCondition],
    ) -> (bool, u64, Option<ProofCertificate>) {
        let start = std::time::Instant::now();

        if let Some(solver) = &mut self.smt_solver {
            // Push a new context
            let _ = solver.push();

            // Assert the negation of the condition (to check for unsatisfiability)
            let negated = Expression::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(condition.expression.clone()),
            };

            if solver.assert(&negated).is_ok() {
                match solver.check_sat(&negated) {
                    Ok(SatResult::Unsat) => {
                        let _ = solver.pop();
                        let elapsed = start.elapsed().as_millis() as u64;

                        // Generate proof certificate
                        let proof = ProofCertificate {
                            condition_name: condition.name.clone(),
                            proof_steps: vec![
                                "Assumed negation of condition".to_string(),
                                "SMT solver found unsatisfiability".to_string(),
                                "Therefore, condition is always true".to_string(),
                            ],
                            assumptions_used: vec![],
                            method: VerificationMethod::Z3Solver,
                        };

                        return (true, elapsed, Some(proof));
                    }
                    Ok(SatResult::Sat) => {
                        let _ = solver.pop();
                        let elapsed = start.elapsed().as_millis() as u64;
                        return (false, elapsed, None);
                    }
                    _ => {
                        let _ = solver.pop();
                        let elapsed = start.elapsed().as_millis() as u64;
                        return (false, elapsed, None);
                    }
                }
            }

            let _ = solver.pop();
        }

        let elapsed = start.elapsed().as_millis() as u64;
        (false, elapsed, None)
    }

    /// Verify a postcondition with preconditions as assumptions
    fn verify_postcondition(
        &mut self,
        postcondition: &EnhancedCondition,
        preconditions: &[EnhancedCondition],
    ) -> (bool, u64, Option<ProofCertificate>) {
        let start = std::time::Instant::now();

        if let Some(solver) = &mut self.smt_solver {
            let _ = solver.push();

            // Assert all preconditions
            for pre in preconditions {
                let _ = solver.assert(&pre.expression);
            }

            // Check if postcondition holds given preconditions
            let negated_post = Expression::UnaryOp {
                op: UnaryOp::Not,
                operand: Box::new(postcondition.expression.clone()),
            };

            if solver.assert(&negated_post).is_ok() {
                match solver.check_sat(&negated_post) {
                    Ok(SatResult::Unsat) => {
                        let _ = solver.pop();
                        let elapsed = start.elapsed().as_millis() as u64;

                        let proof = ProofCertificate {
                            condition_name: postcondition.name.clone(),
                            proof_steps: vec![
                                "Assumed all preconditions".to_string(),
                                "Assumed negation of postcondition".to_string(),
                                "SMT solver found unsatisfiability".to_string(),
                                "Therefore, postcondition follows from preconditions".to_string(),
                            ],
                            assumptions_used: preconditions
                                .iter()
                                .map(|p| p.name.clone())
                                .collect(),
                            method: VerificationMethod::Z3Solver,
                        };

                        return (true, elapsed, Some(proof));
                    }
                    _ => {
                        let _ = solver.pop();
                        let elapsed = start.elapsed().as_millis() as u64;
                        return (false, elapsed, None);
                    }
                }
            }

            let _ = solver.pop();
        }

        let elapsed = start.elapsed().as_millis() as u64;
        (false, elapsed, None)
    }
}


#[cfg(test)]
#[path = "contracts_tests.rs"]
mod contracts_tests;
