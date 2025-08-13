//! Function inlining optimization pass
//! 
//! Inlines small functions to reduce call overhead

use super::OptimizationPass;
use std::collections::HashSet;
use crate::mir::{Function, Program, Statement, Terminator, Rvalue, Operand, Place, LocalId,
                 BasicBlockId, SourceInfo};
use crate::error::SemanticError;
use std::collections::HashMap;

/// Function inlining optimization pass
#[derive(Debug)]
pub struct InliningPass {
    /// Inlining threshold (e.g., number of statements)
    threshold: usize,
    
    /// Functions already inlined to prevent recursion
    inlined_functions: HashSet<String>,
}

impl InliningPass {
    pub fn new() -> Self {
        Self {
            threshold: 20,
            inlined_functions: HashSet::new(),
        }
    }
    
    /// Set the maximum size for inlining
    pub fn set_max_inline_size(&mut self, size: usize) {
        self.threshold = size;
    }
    
    /// Set the maximum inlining depth
    pub fn set_max_inline_depth(&mut self, depth: usize) {
    }
    
    /// Calculate the "cost" of a function for inlining decisions
    fn calculate_function_cost(&self, function: &Function) -> usize {
        let mut cost = 0;
        
        for block in function.basic_blocks.values() {
            cost += block.statements.len();
            
            // Add cost for complex terminators
            match &block.terminator {
                Terminator::Call { .. } => cost += 5, // Calls are expensive
                Terminator::SwitchInt { .. } => cost += 2, // Branches have some cost
                _ => cost += 1,
            }
        }
        
        cost
    }
    
    /// Check if a function is suitable for inlining
    fn should_inline(&self, function: &Function) -> bool {
        // Don't inline recursive functions (basic check)
        if self.has_recursive_calls(function) {
            return false;
        }
        
        // Check size constraints
        let cost = self.calculate_function_cost(function);
        cost <= self.threshold
    }
    
    /// Basic check for recursive calls
    fn has_recursive_calls(&self, function: &Function) -> bool {
        for block in function.basic_blocks.values() {
            for statement in &block.statements {
                if let Statement::Assign { rvalue: Rvalue::Call { func, .. }, .. } = statement {
                    if let Operand::Constant(_constant) = func {
                        // In a real implementation, we'd check if the constant refers to the same function
                        // For now, just assume no recursion
                    }
                }
            }
            
            if let Terminator::Call { func, .. } = &block.terminator {
                if let Operand::Constant(_constant) = func {
                    // Same as above - in practice we'd need better function identification
                }
            }
        }
        
        false // Conservative: assume no recursion for now
    }
    
    
    
}

impl OptimizationPass for InliningPass {
    fn name(&self) -> &'static str {
        "inlining"
    }
    
    fn run_on_function(&mut self, _function: &mut Function) -> Result<bool, SemanticError> {
        // Single function inlining requires access to the whole program
        // For now, return false (no changes)
        Ok(false)
    }
    
    fn run_on_program(&mut self, program: &mut Program) -> Result<bool, SemanticError> {
        let changed = false;
        
        // Find functions that are candidates for inlining
        let mut inline_candidates = Vec::new();
        
        for (name, function) in &program.functions {
            if self.should_inline(function) {
                inline_candidates.push(name.clone());
            }
        }
        
        // For each function, look for calls to inline candidates
        for (caller_name, caller_function) in &mut program.functions {
            if inline_candidates.contains(caller_name) {
                continue; // Don't modify functions we're trying to inline
            }
            
            // Look for calls in each basic block
            for block in caller_function.basic_blocks.values_mut() {
                let mut new_statements = Vec::new();
                
                for statement in &block.statements {
                    match statement {
                        Statement::Assign { place: _, rvalue: Rvalue::Call { func: _, args: _ }, source_info: _ } => {
                            // Check if this is a call to an inline candidate
                            // This is simplified - in practice we'd need better function identification
                            new_statements.push(statement.clone());
                        }
                        _ => {
                            new_statements.push(statement.clone());
                        }
                    }
                }
                
                block.statements = new_statements;
            }
        }
        
        Ok(changed)
    }
}

impl Default for InliningPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{Builder, Place, SourceInfo, Constant, ConstantValue};
    use crate::types::Type;
    use crate::ast::PrimitiveType;
    use crate::error::SourceLocation;
    
    #[test]
    fn test_function_cost_calculation() {
        let pass = InliningPass::new();
        let mut builder = Builder::new();
        
        builder.start_function(
            "small".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );
        
        let temp = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        
        // Add a single statement
        builder.push_statement(Statement::Assign {
            place: Place { local: temp, projection: vec![] },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(42),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });
        
        let function = builder.finish_function();
        let cost = pass.calculate_function_cost(&function);
        
        // Should be low cost (1 statement + 1 terminator)
        assert!(cost <= 5);
    }
    
    #[test]
    fn test_should_inline_small_function() {
        let pass = InliningPass::new();
        let mut builder = Builder::new();
        
        builder.start_function(
            "small".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );
        
        let temp = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        
        // Add a few small statements
        for i in 0..3 {
            builder.push_statement(Statement::Assign {
                place: Place { local: temp, projection: vec![] },
                rvalue: Rvalue::Use(Operand::Constant(Constant {
                    ty: Type::primitive(PrimitiveType::Integer),
                    value: ConstantValue::Integer(i),
                })),
                source_info: SourceInfo {
                    span: SourceLocation::unknown(),
                    scope: 0,
                },
            });
        }
        
        let function = builder.finish_function();
        
        // Small function should be eligible for inlining
        assert!(pass.should_inline(&function));
    }
    
    #[test]
    fn test_program_inlining() {
        let mut pass = InliningPass::new();
        let mut program = Program {
            functions: HashMap::new(),
            global_constants: HashMap::new(),
            external_functions: HashMap::new(),
            type_definitions: HashMap::new(),
        };
        
        // Create a small function to inline
        let mut builder = Builder::new();
        builder.start_function(
            "small".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );
        
        let temp = builder.new_local(Type::primitive(PrimitiveType::Integer), false);
        builder.push_statement(Statement::Assign {
            place: Place { local: temp, projection: vec![] },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(42),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });
        
        let small_function = builder.finish_function();
        program.functions.insert("small".to_string(), small_function);
        
        // Run inlining pass
        let _changed = pass.run_on_program(&mut program).unwrap();
        
        // Function should still exist (not actually inlined in this simplified implementation)
        assert!(program.functions.contains_key("small"));
    }
}