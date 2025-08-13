//! Verification condition generator
//! 
//! Generates verification conditions from MIR code that can be checked by the SMT solver

use crate::error::{SemanticError, SourceLocation};
use crate::mir::{self, BasicBlockId, Operand, Rvalue, Statement, Terminator};
use crate::types::Type;
use crate::ast::PrimitiveType;
use super::contracts::{FunctionContract, Expression as ContractExpr, BinaryOp as ContractBinOp, ConstantValue};
use super::solver::{Formula, VerificationCondition};
use std::collections::{HashMap, HashSet};

/// Verification condition generator
pub struct VcGenerator {
    /// Counter for generating unique VC names
    vc_counter: usize,
    
    /// Current path condition
    path_condition: Vec<Formula>,
    
    /// Current variable state
    state: HashMap<String, Formula>,
}

/// Type of verification condition
#[derive(Debug, Clone)]
pub enum VcType {
    /// Precondition check
    Precondition,
    
    /// Postcondition check
    Postcondition,
    
    /// Loop invariant preservation
    LoopInvariantPreservation,
    
    /// Loop invariant entry
    LoopInvariantEntry,
    
    /// Assertion check
    Assertion,
    
    /// Array bounds check
    ArrayBounds,
    
    /// Division by zero check
    DivisionByZero,
    
    /// Null pointer check
    NullPointer,
}

impl VcGenerator {
    /// Create a new VC generator
    pub fn new() -> Self {
        Self {
            vc_counter: 0,
            path_condition: Vec::new(),
            state: HashMap::new(),
        }
    }
    
    /// Generate verification conditions for a function
    pub fn generate_function_vcs(
        &mut self,
        function: &mir::Function,
        contract: Option<&FunctionContract>,
    ) -> Result<Vec<VerificationCondition>, SemanticError> {
        let mut vcs = Vec::new();
        
        // Reset state for new function
        self.path_condition.clear();
        self.state.clear();
        
        // Initialize parameters in state
        for (idx, _param) in function.parameters.iter().enumerate() {
            let param_name = format!("param_{}", idx);
            self.state.insert(param_name.clone(), Formula::Var(param_name));
        }
        
        // Check preconditions at function entry
        if let Some(contract) = contract {
            for precond in &contract.preconditions {
                let formula = self.contract_expr_to_formula(&precond.expression)?;
                vcs.push(self.create_vc(
                    format!("precondition_{}", precond.name),
                    VcType::Precondition,
                    formula,
                    precond.location.clone(),
                ));
            }
        }
        
        // Process each basic block
        let entry_block = function.entry_block;
        let mut visited = HashSet::new();
        self.process_block(function, entry_block, &mut visited, &mut vcs, contract)?;
        
        Ok(vcs)
    }
    
    /// Process a basic block and generate VCs
    fn process_block(
        &mut self,
        function: &mir::Function,
        block_id: BasicBlockId,
        visited: &mut HashSet<BasicBlockId>,
        vcs: &mut Vec<VerificationCondition>,
        contract: Option<&FunctionContract>,
    ) -> Result<(), SemanticError> {
        if visited.contains(&block_id) {
            // Handle loops - check invariant preservation
            return Ok(());
        }
        visited.insert(block_id);
        
        let block = &function.basic_blocks[&block_id];
        
        // Process statements
        for stmt in &block.statements {
            self.process_statement(stmt, vcs)?;
        }
        
        // Process terminator
        match &block.terminator {
            Terminator::Return => {
                // Check postconditions
                if let Some(contract) = contract {
                    // Note: In a real implementation, we would track the return value
                    // through the MIR statements. For now, we assume it's stored in a
                    // special "return_value" local if there is one.
                    
                    for postcond in &contract.postconditions {
                        let formula = self.contract_expr_to_formula(&postcond.expression)?;
                        let vc_formula = self.apply_path_condition(formula);
                        vcs.push(self.create_vc(
                            format!("postcondition_{}", postcond.name),
                            VcType::Postcondition,
                            vc_formula,
                            postcond.location.clone(),
                        ));
                    }
                }
            }
            Terminator::SwitchInt { discriminant, targets, .. } => {
                let disc_formula = self.operand_to_formula(discriminant)?;
                
                // For now, handle only boolean switches (true/false)
                if targets.values.len() == 1 && targets.targets.len() == 1 {
                    // Assume value 0 means false, go to otherwise
                    // Assume value 1 means true, go to targets[0]
                    if targets.values[0] == 1 {
                        // True branch
                        self.path_condition.push(disc_formula.clone());
                        self.process_block(function, targets.targets[0], visited, vcs, contract)?;
                        self.path_condition.pop();
                        
                        // False branch (otherwise)
                        self.path_condition.push(Formula::Not(Box::new(disc_formula)));
                        self.process_block(function, targets.otherwise, visited, vcs, contract)?;
                        self.path_condition.pop();
                    }
                } else {
                    // TODO: Handle general switch statements
                    for &target in &targets.targets {
                        self.process_block(function, target, visited, vcs, contract)?;
                    }
                    self.process_block(function, targets.otherwise, visited, vcs, contract)?;
                }
            }
            Terminator::Goto { target } => {
                self.process_block(function, *target, visited, vcs, contract)?;
            }
            Terminator::Call { .. } => {
                // TODO: Handle function calls with contracts
            }
            Terminator::Drop { .. } => {
                // Drop is a memory operation - no verification needed for now
                // TODO: Could verify drop safety
            }
            Terminator::Assert { condition, expected, message: _, target, cleanup: _ } => {
                // Generate VC for assertion
                let cond_formula = self.operand_to_formula(condition)?;
                let assert_formula = if *expected {
                    cond_formula
                } else {
                    Formula::Not(Box::new(cond_formula))
                };
                
                let vc_formula = self.apply_path_condition(assert_formula);
                vcs.push(self.create_vc(
                    format!("assertion_{}", self.vc_counter),
                    VcType::Assertion,
                    vc_formula,
                    SourceLocation::unknown(), // TODO: Get proper location
                ));
                
                // Continue to target block
                self.process_block(function, *target, visited, vcs, contract)?;
            }
            Terminator::Unreachable => {
                // No VCs needed
            }
        }
        
        Ok(())
    }
    
    /// Process a statement and generate VCs
    fn process_statement(
        &mut self,
        stmt: &Statement,
        vcs: &mut Vec<VerificationCondition>,
    ) -> Result<(), SemanticError> {
        match stmt {
            Statement::Assign { place, rvalue, source_info } => {
                let value_formula = self.rvalue_to_formula(rvalue)?;
                
                // Check for division by zero
                if let Rvalue::BinaryOp { op: mir::BinOp::Div, left: _, right } = rvalue {
                    let divisor = self.operand_to_formula(right)?;
                    let not_zero = Formula::Ne(
                        Box::new(divisor),
                        Box::new(Formula::Int(0)),
                    );
                    let vc_formula = self.apply_path_condition(not_zero);
                    vcs.push(self.create_vc(
                        format!("div_by_zero_check_{}", self.vc_counter),
                        VcType::DivisionByZero,
                        vc_formula,
                        source_info.span.clone(),
                    ));
                }
                
                // Update state
                let local_name = format!("local_{}", place.local);
                self.state.insert(local_name, value_formula);
            }
            Statement::StorageLive(_) | Statement::StorageDead(_) => {
                // Storage markers don't affect verification
            }
            Statement::Nop => {
                // Nothing to do
            }
        }
        
        Ok(())
    }
    
    /// Convert an rvalue to a formula
    fn rvalue_to_formula(&mut self, rvalue: &Rvalue) -> Result<Formula, SemanticError> {
        match rvalue {
            Rvalue::Use(operand) => self.operand_to_formula(operand),
            Rvalue::BinaryOp { op, left, right } => {
                let left_formula = self.operand_to_formula(left)?;
                let right_formula = self.operand_to_formula(right)?;
                
                Ok(match op {
                    mir::BinOp::Add => Formula::Add(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Sub => Formula::Sub(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Mul => Formula::Mul(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Div => Formula::Div(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Mod => Formula::Mod(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Rem => Formula::Mod(Box::new(left_formula), Box::new(right_formula)), // Rem is same as Mod
                    mir::BinOp::Eq => Formula::Eq(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Ne => Formula::Ne(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Lt => Formula::Lt(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Le => Formula::Le(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Gt => Formula::Gt(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::Ge => Formula::Ge(Box::new(left_formula), Box::new(right_formula)),
                    mir::BinOp::And => Formula::And(vec![left_formula, right_formula]),
                    mir::BinOp::Or => Formula::Or(vec![left_formula, right_formula]),
                    mir::BinOp::BitAnd => Formula::And(vec![left_formula, right_formula]), // Treat as logical for verification
                    mir::BinOp::BitOr => Formula::Or(vec![left_formula, right_formula]), // Treat as logical for verification
                    mir::BinOp::BitXor => {
                        // XOR as (A && !B) || (!A && B)
                        let not_left = Formula::Not(Box::new(left_formula.clone()));
                        let not_right = Formula::Not(Box::new(right_formula.clone()));
                        Formula::Or(vec![
                            Formula::And(vec![left_formula, not_right]),
                            Formula::And(vec![not_left, right_formula]),
                        ])
                    }
                    mir::BinOp::Shl | mir::BinOp::Shr => {
                        // Bit shifts - conservatively return true for verification
                        Formula::Bool(true)
                    }
                    mir::BinOp::Offset => {
                        // Pointer offset - conservatively return true for verification
                        Formula::Bool(true)
                    }
                })
            }
            Rvalue::UnaryOp { op, operand } => {
                let operand_formula = self.operand_to_formula(operand)?;
                
                Ok(match op {
                    mir::UnOp::Neg => Formula::Sub(
                        Box::new(Formula::Int(0)),
                        Box::new(operand_formula),
                    ),
                    mir::UnOp::Not => Formula::Not(Box::new(operand_formula)),
                })
            }
            Rvalue::Cast { operand, ty: _, kind: _ } => {
                // For now, ignore casts in verification
                self.operand_to_formula(operand)
            }
            Rvalue::Call { .. } => {
                // Function calls - conservatively return symbolic value
                // TODO: Handle function contracts properly
                Ok(Formula::Var("call_result".to_string()))
            }
            Rvalue::Aggregate { .. } => {
                // Aggregate construction - return symbolic value
                // TODO: Model aggregate values properly
                Ok(Formula::Var("aggregate_value".to_string()))
            }
            Rvalue::Ref { .. } => {
                // Reference creation - return symbolic value
                // TODO: Model memory operations
                Ok(Formula::Var("ref_value".to_string()))
            }
            Rvalue::Len(_) => {
                // Array/slice length - return symbolic value
                Ok(Formula::Var("array_length".to_string()))
            }
            Rvalue::Discriminant(_) => {
                // Enum discriminant - return symbolic value
                Ok(Formula::Var("enum_discriminant".to_string()))
            }
        }
    }
    
    /// Convert an operand to a formula
    fn operand_to_formula(&self, operand: &Operand) -> Result<Formula, SemanticError> {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                let local_name = format!("local_{}", place.local);
                Ok(self.state.get(&local_name)
                    .cloned()
                    .unwrap_or_else(|| Formula::Var(local_name)))
            }
            Operand::Constant(constant) => {
                let value = &constant.value;
                Ok(match value {
                    mir::ConstantValue::Integer(n) => Formula::Int(*n as i64),
                    mir::ConstantValue::Float(f) => Formula::Real(*f),
                    mir::ConstantValue::Bool(b) => Formula::Bool(*b),
                    mir::ConstantValue::String(_) => {
                        // Strings not yet supported in verification
                        Formula::Bool(true)
                    }
                    mir::ConstantValue::Char(c) => Formula::Int(*c as i64),
                    mir::ConstantValue::Null => Formula::Bool(false),
                })
            }
        }
    }
    
    /// Convert a contract expression to a formula
    fn contract_expr_to_formula(&self, expr: &ContractExpr) -> Result<Formula, SemanticError> {
        match expr {
            ContractExpr::Variable(name) => {
                // Map to current state or create new variable
                Ok(self.state.get(name)
                    .cloned()
                    .unwrap_or_else(|| Formula::Var(name.clone())))
            }
            ContractExpr::Constant(c) => {
                Ok(match c {
                    ConstantValue::Integer(n) => Formula::Int(*n),
                    ConstantValue::Float(f) => Formula::Real(*f),
                    ConstantValue::Boolean(b) => Formula::Bool(*b),
                    ConstantValue::String(_) => Formula::Bool(true),
                    ConstantValue::Null => Formula::Bool(true),
                })
            }
            ContractExpr::BinaryOp { op, left, right } => {
                let left_formula = self.contract_expr_to_formula(left)?;
                let right_formula = self.contract_expr_to_formula(right)?;
                
                Ok(match op {
                    ContractBinOp::Add => Formula::Add(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Sub => Formula::Sub(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Mul => Formula::Mul(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Div => Formula::Div(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Mod => Formula::Mod(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Eq => Formula::Eq(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Ne => Formula::Ne(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Lt => Formula::Lt(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Le => Formula::Le(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Gt => Formula::Gt(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::Ge => Formula::Ge(Box::new(left_formula), Box::new(right_formula)),
                    ContractBinOp::And => Formula::And(vec![left_formula, right_formula]),
                    ContractBinOp::Or => Formula::Or(vec![left_formula, right_formula]),
                    ContractBinOp::Implies => Formula::Implies(Box::new(left_formula), Box::new(right_formula)),
                    _ => return Err(SemanticError::NotImplemented {
                        feature: format!("Contract operator {:?}", op),
                        location: SourceLocation::unknown(),
                    }),
                })
            }
            ContractExpr::Result => {
                Ok(self.state.get("result")
                    .cloned()
                    .unwrap_or_else(|| Formula::Var("result".to_string())))
            }
            _ => Err(SemanticError::NotImplemented {
                feature: "Complex contract expressions".to_string(),
                location: SourceLocation::unknown(),
            }),
        }
    }
    
    /// Apply path condition to a formula
    fn apply_path_condition(&self, formula: Formula) -> Formula {
        if self.path_condition.is_empty() {
            formula
        } else {
            // Path condition implies formula
            let path_cond = if self.path_condition.len() == 1 {
                self.path_condition[0].clone()
            } else {
                Formula::And(self.path_condition.clone())
            };
            Formula::Implies(Box::new(path_cond), Box::new(formula))
        }
    }
    
    /// Create a verification condition
    fn create_vc(
        &mut self,
        name: String,
        vc_type: VcType,
        formula: Formula,
        location: SourceLocation,
    ) -> VerificationCondition {
        self.vc_counter += 1;
        VerificationCondition {
            name: format!("{} ({})", name, format!("{:?}", vc_type)),
            formula,
            location,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{Function, BasicBlock};
    
    #[test]
    fn test_vc_generator_creation() {
        let vcgen = VcGenerator::new();
        assert_eq!(vcgen.vc_counter, 0);
        assert!(vcgen.path_condition.is_empty());
        assert!(vcgen.state.is_empty());
    }
    
    #[test]
    fn test_simple_function_vcs() {
        let mut vcgen = VcGenerator::new();
        
        // Create a simple function with no contracts
        let mut function = Function {
            name: "test".to_string(),
            parameters: vec![],
            return_type: Type::primitive(PrimitiveType::Integer),
            locals: HashMap::new(),
            basic_blocks: HashMap::new(),
            entry_block: 0,
            return_local: None,
        };
        
        // Add an empty entry block
        let entry_id = 0;
        let block = BasicBlock {
            id: entry_id,
            statements: vec![],
            terminator: Terminator::Return,
        };
        function.basic_blocks.insert(entry_id, block);
        function.entry_block = entry_id;
        
        let vcs = vcgen.generate_function_vcs(&function, None).unwrap();
        assert!(vcs.is_empty()); // No contracts, no VCs
    }
}