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

//! Dead code elimination optimization pass
//! 
//! Removes unreachable code and unused assignments

use super::OptimizationPass;
use crate::mir::{
    Function, Statement, Terminator, BasicBlockId, LocalId, Operand, Rvalue,
};
use crate::error::SemanticError;
use std::collections::HashSet;

/// Dead code elimination optimization pass
pub struct DeadCodeEliminationPass {
    removed_statements: usize,
    removed_blocks: usize,
}

impl DeadCodeEliminationPass {
    pub fn new() -> Self {
        Self {
            removed_statements: 0,
            removed_blocks: 0,
        }
    }
    
    /// Find all reachable basic blocks
    fn find_reachable_blocks(&self, function: &Function) -> HashSet<BasicBlockId> {
        let mut reachable = HashSet::new();
        let mut worklist = vec![function.entry_block];
        
        while let Some(block_id) = worklist.pop() {
            if reachable.insert(block_id) {
                if let Some(block) = function.basic_blocks.get(&block_id) {
                    // Add successors to worklist
                    match &block.terminator {
                        Terminator::Goto { target } => {
                            if !reachable.contains(target) {
                                worklist.push(*target);
                            }
                        }
                        Terminator::SwitchInt { targets, .. } => {
                            for &target in &targets.targets {
                                if !reachable.contains(&target) {
                                    worklist.push(target);
                                }
                            }
                            if !reachable.contains(&targets.otherwise) {
                                worklist.push(targets.otherwise);
                            }
                        }
                        Terminator::Call { target, cleanup, .. } => {
                            if let Some(target) = target {
                                if !reachable.contains(target) {
                                    worklist.push(*target);
                                }
                            }
                            if let Some(cleanup) = cleanup {
                                if !reachable.contains(cleanup) {
                                    worklist.push(*cleanup);
                                }
                            }
                        }
                        Terminator::Drop { target, unwind, .. } => {
                            if !reachable.contains(target) {
                                worklist.push(*target);
                            }
                            if let Some(unwind) = unwind {
                                if !reachable.contains(unwind) {
                                    worklist.push(*unwind);
                                }
                            }
                        }
                        Terminator::Assert { target, cleanup, .. } => {
                            if !reachable.contains(target) {
                                worklist.push(*target);
                            }
                            if let Some(cleanup) = cleanup {
                                if !reachable.contains(cleanup) {
                                    worklist.push(*cleanup);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        reachable
    }
    
    /// Remove unreachable basic blocks
    fn remove_unreachable_blocks(&mut self, function: &mut Function) -> bool {
        let reachable = self.find_reachable_blocks(function);
        let original_count = function.basic_blocks.len();
        
        // Remove unreachable blocks
        function.basic_blocks.retain(|&block_id, _| reachable.contains(&block_id));
        
        let removed = original_count - function.basic_blocks.len();
        self.removed_blocks += removed;
        removed > 0
    }
    
    /// Check if a local is used in an operand
    fn local_used_in_operand(&self, operand: &Operand, local: LocalId) -> bool {
        match operand {
            Operand::Copy(place) | Operand::Move(place) => place.local == local,
            Operand::Constant(_) => false,
        }
    }
    
    /// Check if a local is used in an rvalue
    fn local_used_in_rvalue(&self, rvalue: &Rvalue, local: LocalId) -> bool {
        match rvalue {
            Rvalue::Use(operand) => self.local_used_in_operand(operand, local),
            Rvalue::BinaryOp { left, right, .. } => {
                self.local_used_in_operand(left, local) || self.local_used_in_operand(right, local)
            }
            Rvalue::UnaryOp { operand, .. } => self.local_used_in_operand(operand, local),
            Rvalue::Call { func, args } => {
                self.local_used_in_operand(func, local) || 
                args.iter().any(|arg| self.local_used_in_operand(arg, local))
            }
            Rvalue::Aggregate { operands, .. } => {
                operands.iter().any(|operand| self.local_used_in_operand(operand, local))
            }
            Rvalue::Cast { operand, .. } => self.local_used_in_operand(operand, local),
            Rvalue::Ref { place, .. } => place.local == local,
            Rvalue::Len(place) | Rvalue::Discriminant(place) => place.local == local,
        }
    }
    
    /// Check if a local is used in a terminator
    fn local_used_in_terminator(&self, terminator: &Terminator, local: LocalId) -> bool {
        match terminator {
            Terminator::SwitchInt { discriminant, .. } => {
                self.local_used_in_operand(discriminant, local)
            }
            Terminator::Call { func, args, destination, .. } => {
                self.local_used_in_operand(func, local) ||
                args.iter().any(|arg| self.local_used_in_operand(arg, local)) ||
                destination.local == local
            }
            Terminator::Drop { place, .. } => place.local == local,
            Terminator::Assert { condition, .. } => self.local_used_in_operand(condition, local),
            _ => false,
        }
    }
    
    /// Find all locals that are used (not just assigned to)
    fn find_used_locals(&self, function: &Function) -> HashSet<LocalId> {
        let mut used = HashSet::new();
        
        // Function parameters are always considered used
        for param in &function.parameters {
            used.insert(param.local_id);
        }
        
        for block in function.basic_blocks.values() {
            // Check statements
            for statement in &block.statements {
                match statement {
                    Statement::Assign { rvalue, .. } => {
                        // Check rvalue for used locals
                        for local_id in function.locals.keys() {
                            if self.local_used_in_rvalue(rvalue, *local_id) {
                                used.insert(*local_id);
                            }
                        }
                    }
                    Statement::StorageLive(local) | Statement::StorageDead(local) => {
                        used.insert(*local);
                    }
                    Statement::Nop => {}
                }
            }
            
            // Check terminator
            for local_id in function.locals.keys() {
                if self.local_used_in_terminator(&block.terminator, *local_id) {
                    used.insert(*local_id);
                }
            }
        }
        
        used
    }
    
    /// Remove assignments to unused locals
    fn remove_dead_assignments(&mut self, function: &mut Function) -> bool {
        let used_locals = self.find_used_locals(function);
        let mut changed = false;
        
        for block in function.basic_blocks.values_mut() {
            let mut new_statements = Vec::new();
            
            for statement in &block.statements {
                match statement {
                    Statement::Assign { place, rvalue, .. } => {
                        // Keep assignment if the local is used, has side effects, or is a function call
                        let has_side_effects = !place.projection.is_empty() || matches!(rvalue, Rvalue::Call { .. });
                        if used_locals.contains(&place.local) || has_side_effects {
                            new_statements.push(statement.clone());
                        } else {
                            // Remove this dead assignment
                            self.removed_statements += 1;
                            changed = true;
                        }
                    }
                    _ => {
                        new_statements.push(statement.clone());
                    }
                }
            }
            
            block.statements = new_statements;
        }
        
        changed
    }
    
    /// Remove unused locals from the function
    fn remove_unused_locals(&mut self, function: &mut Function) -> bool {
        let used_locals = self.find_used_locals(function);
        let original_count = function.locals.len();
        
        // Remove unused locals
        function.locals.retain(|&local_id, _| used_locals.contains(&local_id));
        
        let removed = original_count - function.locals.len();
        removed > 0
    }
}

impl OptimizationPass for DeadCodeEliminationPass {
    fn name(&self) -> &'static str {
        "dead-code-elimination"
    }
    
    fn run_on_function(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        let mut changed = false;
        
        // Remove unreachable basic blocks
        changed |= self.remove_unreachable_blocks(function);
        
        // Remove assignments to unused locals
        changed |= self.remove_dead_assignments(function);
        
        // Remove unused locals
        changed |= self.remove_unused_locals(function);
        
        Ok(changed)
    }
}

impl Default for DeadCodeEliminationPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{Builder, Place, SourceInfo, Rvalue, Operand, Constant, ConstantValue};
    use crate::types::Type;
    use crate::ast::PrimitiveType;
    use crate::error::SourceLocation;
    
    #[test]
    fn test_dead_assignment_removal() {
        let mut pass = DeadCodeEliminationPass::new();
        let mut builder = Builder::new();
        
        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );
        
        let temp1 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        let temp2 = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        
        // Add dead assignment: temp1 = 42
        builder.push_statement(Statement::Assign {
            place: Place { local: temp1, projection: vec![] },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(42),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });
        
        // Add live assignment: temp2 = 24 (temp2 is used later)
        builder.push_statement(Statement::Assign {
            place: Place { local: temp2, projection: vec![] },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(24),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });
        
        let mut function = builder.finish_function();
        let original_stmt_count = function.basic_blocks.values()
            .map(|block| block.statements.len())
            .sum::<usize>();
        
        // Run dead code elimination
        let changed = pass.run_on_function(&mut function).unwrap();
        assert!(changed);
        
        let new_stmt_count = function.basic_blocks.values()
            .map(|block| block.statements.len())
            .sum::<usize>();
        
        // Should have removed the dead assignment
        assert!(new_stmt_count < original_stmt_count);
        assert_eq!(pass.removed_statements, 2); // Both statements are dead since neither is used
    }
    
    #[test]
    fn test_unreachable_block_removal() {
        let mut pass = DeadCodeEliminationPass::new();
        let mut builder = Builder::new();
        
        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Void),
        );
        
        let reachable_block = builder.new_block();
        let unreachable_block = builder.new_block();
        
        // Set up reachable block that returns
        builder.switch_to_block(reachable_block);
        builder.set_terminator(Terminator::Return);
        
        // Set up unreachable block
        builder.switch_to_block(unreachable_block);
        builder.set_terminator(Terminator::Return);
        
        // Jump to reachable block from entry
        builder.switch_to_block(builder.current_block.unwrap());
        builder.set_terminator(Terminator::Goto { target: reachable_block });
        
        let mut function = builder.finish_function();
        let original_block_count = function.basic_blocks.len();
        
        // Run dead code elimination
        let changed = pass.run_on_function(&mut function).unwrap();
        assert!(changed);
        
        let new_block_count = function.basic_blocks.len();
        
        // Should have removed some unreachable blocks
        assert!(new_block_count < original_block_count);
    }
}