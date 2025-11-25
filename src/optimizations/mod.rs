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

//! Optimization passes for MIR
//!
//! Implements fundamental optimization techniques including dead code elimination,
//! constant folding, and common subexpression elimination.

pub mod common_subexpression;
pub mod constant_folding;
pub mod dead_code_elimination;
pub mod inlining;

// Advanced optimization passes
pub mod interprocedural;
pub mod loop_optimizations;
pub mod profile_guided;
pub mod vectorization;
pub mod whole_program;

use crate::error::SemanticError;
use crate::mir::{Function, Program};

/// Trait for MIR optimization passes
pub trait OptimizationPass {
    /// Name of the optimization pass
    fn name(&self) -> &'static str;

    /// Run the optimization pass on a function
    fn run_on_function(&mut self, function: &mut Function) -> Result<bool, SemanticError>;

    /// Run the optimization pass on a program
    fn run_on_program(&mut self, program: &mut Program) -> Result<bool, SemanticError> {
        let mut changed = false;
        for function in program.functions.values_mut() {
            changed |= self.run_on_function(function)?;
        }
        Ok(changed)
    }
}

/// Optimization manager for running multiple passes
pub struct OptimizationManager {
    passes: Vec<Box<dyn OptimizationPass>>,
    max_iterations: usize,
}

impl OptimizationManager {
    /// Create a new optimization manager
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            max_iterations: 10,
        }
    }

    /// Add an optimization pass
    pub fn add_pass(&mut self, pass: Box<dyn OptimizationPass>) {
        self.passes.push(pass);
    }

    /// Set maximum number of iterations
    pub fn set_max_iterations(&mut self, max_iterations: usize) {
        self.max_iterations = max_iterations;
    }

    /// Run all optimization passes on a program
    pub fn optimize_program(&mut self, program: &mut Program) -> Result<(), SemanticError> {
        for _iteration in 0..self.max_iterations {
            let mut any_changed = false;

            for pass in &mut self.passes {
                let changed = pass.run_on_program(program)?;
                any_changed |= changed;
            }

            // If no passes made changes, we've reached a fixed point
            if !any_changed {
                break;
            }
        }

        Ok(())
    }

    /// Run all optimization passes on a function
    pub fn optimize_function(&mut self, function: &mut Function) -> Result<(), SemanticError> {
        for _iteration in 0..self.max_iterations {
            let mut any_changed = false;

            for pass in &mut self.passes {
                let changed = pass.run_on_function(function)?;
                any_changed |= changed;
            }

            // If no passes made changes, we've reached a fixed point
            if !any_changed {
                break;
            }
        }

        Ok(())
    }

    /// Create a default optimization pipeline
    pub fn create_default_pipeline() -> Self {
        let mut manager = Self::new();

        // Add optimization passes in order
        manager.add_pass(Box::new(constant_folding::ConstantFoldingPass::new()));
        manager.add_pass(Box::new(
            dead_code_elimination::DeadCodeEliminationPass::new(),
        ));
        manager.add_pass(Box::new(
            common_subexpression::CommonSubexpressionEliminationPass::new(),
        ));

        manager
    }

    /// Create an advanced optimization pipeline with all passes
    pub fn create_advanced_pipeline() -> Self {
        let mut manager = Self::new();

        // Basic optimizations first
        manager.add_pass(Box::new(constant_folding::ConstantFoldingPass::new()));
        manager.add_pass(Box::new(
            dead_code_elimination::DeadCodeEliminationPass::new(),
        ));

        // Advanced loop optimizations
        manager.add_pass(Box::new(loop_optimizations::LoopOptimizationPass::new()));

        // Interprocedural analysis
        manager.add_pass(Box::new(interprocedural::InterproceduralAnalysisPass::new()));

        // Auto-vectorization
        manager.add_pass(Box::new(vectorization::VectorizationPass::new()));

        // Common subexpression elimination after other optimizations
        manager.add_pass(Box::new(
            common_subexpression::CommonSubexpressionEliminationPass::new(),
        ));

        // Inlining pass
        manager.add_pass(Box::new(inlining::InliningPass::new()));

        manager
    }

    /// Create a profile-guided optimization pipeline
    pub fn create_pgo_pipeline(profile_data_path: &str) -> Result<Self, SemanticError> {
        let mut manager = Self::new();

        // Basic optimizations
        manager.add_pass(Box::new(constant_folding::ConstantFoldingPass::new()));
        manager.add_pass(Box::new(
            dead_code_elimination::DeadCodeEliminationPass::new(),
        ));

        // Profile-guided optimization
        manager.add_pass(Box::new(
            profile_guided::ProfileGuidedOptimizationPass::from_file(profile_data_path)?,
        ));

        // Advanced optimizations guided by profile data
        manager.add_pass(Box::new(loop_optimizations::LoopOptimizationPass::new()));
        manager.add_pass(Box::new(vectorization::VectorizationPass::new()));
        manager.add_pass(Box::new(
            common_subexpression::CommonSubexpressionEliminationPass::new(),
        ));

        Ok(manager)
    }

    /// Create whole program optimization pipeline
    pub fn create_whole_program_pipeline() -> Self {
        let mut manager = Self::new();

        // Whole program analysis must come first
        manager.add_pass(Box::new(whole_program::WholeProgramOptimizationPass::new()));

        // Interprocedural optimizations
        manager.add_pass(Box::new(interprocedural::InterproceduralAnalysisPass::new()));

        // Standard optimizations
        manager.add_pass(Box::new(constant_folding::ConstantFoldingPass::new()));
        manager.add_pass(Box::new(
            dead_code_elimination::DeadCodeEliminationPass::new(),
        ));
        manager.add_pass(Box::new(loop_optimizations::LoopOptimizationPass::new()));
        manager.add_pass(Box::new(vectorization::VectorizationPass::new()));
        manager.add_pass(Box::new(
            common_subexpression::CommonSubexpressionEliminationPass::new(),
        ));

        manager
    }
}

impl Default for OptimizationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::PrimitiveType;
    use crate::error::SourceLocation;
    use crate::mir::{
        Builder, Constant, ConstantValue, Operand, Place, Rvalue, SourceInfo, Statement,
    };
    use crate::types::Type;

    #[test]
    fn test_optimization_manager() {
        let mut manager = OptimizationManager::new();
        let mut program = Program {
            functions: std::collections::HashMap::new(),
            global_constants: std::collections::HashMap::new(),
            external_functions: std::collections::HashMap::new(),
            type_definitions: std::collections::HashMap::new(),
        };

        // Test with empty program
        assert!(manager.optimize_program(&mut program).is_ok());
    }

    #[test]
    fn test_default_pipeline() {
        let mut manager = OptimizationManager::create_default_pipeline();
        let mut builder = Builder::new();

        // Create a simple function for testing
        builder.start_function(
            "test".to_string(),
            vec![],
            Type::primitive(PrimitiveType::Integer),
        );

        let temp = builder.new_local(Type::primitive(PrimitiveType::Integer), false);

        // Add a constant assignment: temp = 42
        builder.push_statement(Statement::Assign {
            place: Place {
                local: temp,
                projection: vec![],
            },
            rvalue: Rvalue::Use(Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(42),
            })),
            source_info: SourceInfo {
                span: SourceLocation::unknown(),
                scope: 0,
            },
        });

        let mut function = builder.finish_function();

        // Run optimizations
        assert!(manager.optimize_function(&mut function).is_ok());

        // Function should still be valid after optimization
        assert_eq!(function.name, "test");
    }
}
