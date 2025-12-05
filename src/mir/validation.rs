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

//! MIR validation passes
//!
//! Ensures MIR is well-formed and follows invariants

use super::cfg;
use super::dataflow::Location;
use super::*;
use std::collections::{HashMap, HashSet};

/// MIR validation errors
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Undefined local used
    UndefinedLocal { local: LocalId, location: Location },

    /// Type mismatch
    TypeMismatch {
        expected: Type,
        found: Type,
        location: Location,
    },

    /// Unreachable code
    UnreachableCode { block: BasicBlockId },

    /// Missing terminator
    MissingTerminator { block: BasicBlockId },

    /// Invalid CFG edge
    InvalidEdge {
        from: BasicBlockId,
        to: BasicBlockId,
    },

    /// Uninitialized local
    UninitializedLocal { local: LocalId, location: Location },
}

/// MIR validator
pub struct Validator {
    errors: Vec<ValidationError>,
}

impl Validator {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Validate a MIR function
    pub fn validate_function(&mut self, function: &Function) -> Result<(), Vec<ValidationError>> {
        self.errors.clear();

        // Check that all locals are defined
        self.check_locals(function);

        // Check basic block structure
        self.check_basic_blocks(function);

        // Check control flow graph
        self.check_cfg(function);

        // Check type consistency
        self.check_types(function);

        // Check SSA properties
        self.check_ssa(function);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Check that all locals are properly declared
    fn check_locals(&mut self, function: &Function) {
        let mut used_locals = HashSet::new();

        // Collect all used locals
        for (block_id, block) in &function.basic_blocks {
            for (stmt_idx, stmt) in block.statements.iter().enumerate() {
                let location = Location {
                    block: *block_id,
                    statement_index: Some(stmt_idx),
                };

                self.collect_statement_locals(stmt, &mut used_locals, location);
            }

            let term_location = Location {
                block: *block_id,
                statement_index: Some(block.statements.len()),
            };

            self.collect_terminator_locals(&block.terminator, &mut used_locals, term_location);
        }

        // Check parameters
        for param in &function.parameters {
            if !function.locals.contains_key(&param.local_id) {
                self.errors.push(ValidationError::UndefinedLocal {
                    local: param.local_id,
                    location: Location {
                        block: function.entry_block,
                        statement_index: None,
                    },
                });
            }
        }

        // Check all used locals are defined
        for (local, location) in used_locals {
            if !function.locals.contains_key(&local) {
                self.errors
                    .push(ValidationError::UndefinedLocal { local, location });
            }
        }
    }

    /// Collect locals used in a statement
    fn collect_statement_locals(
        &self,
        stmt: &Statement,
        used: &mut HashSet<(LocalId, Location)>,
        location: Location,
    ) {
        match stmt {
            Statement::Assign { place, rvalue, .. } => {
                used.insert((place.local, location));
                self.collect_rvalue_locals(rvalue, used, location);
            }
            Statement::StorageLive(local) | Statement::StorageDead(local) => {
                used.insert((*local, location));
            }
            Statement::Nop => {}
        }
    }

    /// Collect locals used in an rvalue
    fn collect_rvalue_locals(
        &self,
        rvalue: &Rvalue,
        used: &mut HashSet<(LocalId, Location)>,
        location: Location,
    ) {
        match rvalue {
            Rvalue::Use(op) => self.collect_operand_locals(op, used, location),
            Rvalue::BinaryOp { left, right, .. } => {
                self.collect_operand_locals(left, used, location);
                self.collect_operand_locals(right, used, location);
            }
            Rvalue::UnaryOp { operand, .. } => {
                self.collect_operand_locals(operand, used, location);
            }
            Rvalue::Call { func, args, .. } => {
                self.collect_operand_locals(func, used, location);
                for arg in args {
                    self.collect_operand_locals(arg, used, location);
                }
            }
            Rvalue::Aggregate { operands, .. } => {
                for op in operands {
                    self.collect_operand_locals(op, used, location);
                }
            }
            Rvalue::Cast { operand, .. } => {
                self.collect_operand_locals(operand, used, location);
            }
            Rvalue::Ref { place, .. } => {
                used.insert((place.local, location));
            }
            Rvalue::AddressOf(place) => {
                used.insert((place.local, location));
            }
            Rvalue::Len(place) | Rvalue::Discriminant(place) => {
                used.insert((place.local, location));
            }
            Rvalue::Closure { captures, .. } => {
                for capture in captures {
                    self.collect_operand_locals(capture, used, location);
                }
            }
        }
    }

    /// Collect locals used in an operand
    fn collect_operand_locals(
        &self,
        operand: &Operand,
        used: &mut HashSet<(LocalId, Location)>,
        location: Location,
    ) {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => {
                used.insert((place.local, location));
            }
            Operand::Constant(_) => {}
        }
    }

    /// Collect locals used in a terminator
    fn collect_terminator_locals(
        &self,
        term: &Terminator,
        used: &mut HashSet<(LocalId, Location)>,
        location: Location,
    ) {
        match term {
            Terminator::SwitchInt { discriminant, .. } => {
                self.collect_operand_locals(discriminant, used, location);
            }
            Terminator::Call {
                func,
                args,
                destination,
                ..
            } => {
                self.collect_operand_locals(func, used, location);
                for arg in args {
                    self.collect_operand_locals(arg, used, location);
                }
                used.insert((destination.local, location));
            }
            Terminator::Drop { place, .. } => {
                used.insert((place.local, location));
            }
            Terminator::Assert { condition, .. } => {
                self.collect_operand_locals(condition, used, location);
            }
            _ => {}
        }
    }

    /// Check basic block structure
    fn check_basic_blocks(&mut self, function: &Function) {
        // Check entry block exists
        if !function.basic_blocks.contains_key(&function.entry_block) {
            self.errors.push(ValidationError::InvalidEdge {
                from: function.entry_block,
                to: function.entry_block,
            });
        }

        // Check all blocks have terminators
        for (block_id, block) in &function.basic_blocks {
            if matches!(block.terminator, Terminator::Unreachable)
                && *block_id != function.entry_block
            {
                // Unreachable is only valid as a placeholder during construction
                self.errors
                    .push(ValidationError::MissingTerminator { block: *block_id });
            }
        }
    }

    /// Check control flow graph validity
    fn check_cfg(&mut self, function: &Function) {
        let mut reachable = HashSet::new();
        let mut worklist = vec![function.entry_block];

        // Find all reachable blocks
        while let Some(block_id) = worklist.pop() {
            if reachable.insert(block_id) {
                if let Some(block) = function.basic_blocks.get(&block_id) {
                    for succ in cfg::successors(block) {
                        if !reachable.contains(&succ) {
                            worklist.push(succ);
                        }
                    }
                }
            }
        }

        // Check all blocks are reachable
        for block_id in function.basic_blocks.keys() {
            if !reachable.contains(block_id) {
                self.errors
                    .push(ValidationError::UnreachableCode { block: *block_id });
            }
        }

        // Check all edges point to valid blocks
        for (block_id, block) in &function.basic_blocks {
            for succ in cfg::successors(block) {
                if !function.basic_blocks.contains_key(&succ) {
                    self.errors.push(ValidationError::InvalidEdge {
                        from: *block_id,
                        to: succ,
                    });
                }
            }
        }
    }

    /// Check type consistency
    fn check_types(&mut self, _function: &Function) {
        // TODO: Implement type checking
        // This would verify that:
        // - Binary operations have compatible operand types
        // - Assignments have matching types
        // - Function calls have correct argument types
        // - etc.
    }

    /// Check SSA properties
    fn check_ssa(&mut self, function: &Function) {
        let mut definitions = HashMap::new();

        // In SSA form, each local should be assigned at most once
        for (block_id, block) in &function.basic_blocks {
            for (stmt_idx, stmt) in block.statements.iter().enumerate() {
                if let Statement::Assign { place, .. } = stmt {
                    let location = Location {
                        block: *block_id,
                        statement_index: Some(stmt_idx),
                    };

                    if let Some(prev_location) = definitions.get(&place.local) {
                        // Multiple assignments to same local - not strict SSA
                        // This is actually okay in MIR, but we could warn about it
                        _ = prev_location;
                    }

                    definitions.insert(place.local, location);
                }
            }
        }
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

/// Dominance analysis for MIR
pub struct DominatorTree {
    /// Immediate dominator of each block
    idom: HashMap<BasicBlockId, BasicBlockId>,
}

impl DominatorTree {
    /// Compute dominator tree for a function
    pub fn compute(function: &Function) -> Self {
        let mut idom = HashMap::new();

        // Simple algorithm for now
        // In a full implementation, we'd use a more efficient algorithm
        // like Lengauer-Tarjan

        // Entry block dominates itself
        idom.insert(function.entry_block, function.entry_block);

        Self { idom }
    }

    /// Check if `a` dominates `b`
    pub fn dominates(&self, a: BasicBlockId, b: BasicBlockId) -> bool {
        let mut current = b;

        loop {
            if current == a {
                return true;
            }

            if let Some(&idom) = self.idom.get(&current) {
                if idom == current {
                    // Reached entry block
                    return a == current;
                }
                current = idom;
            } else {
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;

    #[test]
    fn test_validator_valid_function() {
        let mut builder = Builder::new();
        builder.start_function(
            "test".to_string(),
            vec![("x".to_string(), Type::primitive(PrimitiveType::Integer))],
            Type::primitive(PrimitiveType::Integer),
        );

        builder.set_terminator(Terminator::Return);
        let function = builder.finish_function();

        let mut validator = Validator::new();
        assert!(validator.validate_function(&function).is_ok());
    }

    #[test]
    fn test_validator_undefined_local() {
        let mut function = Function {
            name: "test".to_string(),
            parameters: vec![],
            return_type: Type::primitive(PrimitiveType::Integer),
            return_local: None,
            locals: HashMap::new(),
            basic_blocks: HashMap::new(),
            entry_block: 0,
        };

        // Create a block that uses undefined local
        let mut block = BasicBlock {
            id: 0,
            statements: vec![Statement::Assign {
                place: Place {
                    local: 999,
                    projection: vec![],
                }, // Undefined local
                rvalue: Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::Integer),
                    value: ConstantValue::Integer(42),
                })),
                source_info: SourceInfo {
                    span: SourceLocation::unknown(),
                    scope: 0,
                },
            }],
            terminator: Terminator::Return,
        };

        function.basic_blocks.insert(0, block);

        let mut validator = Validator::new();
        assert!(validator.validate_function(&function).is_err());
    }
}
