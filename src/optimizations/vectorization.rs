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

//! Auto-vectorization for AetherScript
//!
//! Automatically detects and vectorizes loops that can benefit from SIMD instructions.
//! Analyzes data dependencies and memory access patterns to identify vectorization opportunities.

#![allow(dead_code)]

use crate::ast::PrimitiveType;
use crate::error::SemanticError;
use crate::mir::{
    BasicBlock, BinOp, Function, Operand, Place, Rvalue, Statement, Terminator, UnOp,
};
use crate::optimizations::OptimizationPass;
use crate::types::Type;
use std::collections::{HashMap, HashSet};

/// Auto-vectorization pass
#[derive(Debug)]
pub struct VectorizationPass {
    /// Vector width for different data types
    vector_widths: HashMap<PrimitiveType, usize>,

    /// Detected vectorizable loops
    vectorizable_loops: Vec<VectorizableLoop>,

    /// Data dependency analyzer
    dependency_analyzer: DependencyAnalyzer,
}

/// Information about a vectorizable loop
#[derive(Debug, Clone)]
pub struct VectorizableLoop {
    /// Header block of the loop
    pub header_block: usize,

    /// Induction variable
    pub induction_var: Place,

    /// Loop bounds (start, end, step)
    pub bounds: LoopBounds,

    /// Vectorizable statements in the loop
    pub vectorizable_statements: Vec<VectorizableStatement>,

    /// Estimated benefit of vectorization
    pub benefit_score: f64,

    /// Vector width that can be used
    pub vector_width: usize,
}

/// Loop bounds information
#[derive(Debug, Clone)]
pub struct LoopBounds {
    pub start: Operand,
    pub end: Operand,
    pub step: i64,
    pub is_known_count: bool,
    pub iteration_count: Option<usize>,
}

/// A statement that can be vectorized
#[derive(Debug, Clone)]
pub struct VectorizableStatement {
    /// Original statement
    pub statement_index: usize,

    /// Type of vectorization operation
    pub vector_op: VectorOperation,

    /// Input operands
    pub inputs: Vec<Operand>,

    /// Output place
    pub output: Place,

    /// Memory access pattern
    pub access_pattern: MemoryAccessPattern,
}

/// Types of vector operations
#[derive(Debug, Clone, PartialEq)]
pub enum VectorOperation {
    /// Arithmetic operations (add, sub, mul, div)
    Arithmetic(BinOp),

    /// Unary operations (neg, not)
    Unary(UnOp),

    /// Memory operations (load, store)
    Load,
    Store,

    /// Reduction operations (sum, max, min)
    Reduction(ReductionOp),

    /// Broadcast (splat scalar to vector)
    Broadcast,

    /// Shuffle operations
    Shuffle,
}

/// Reduction operations
#[derive(Debug, Clone, PartialEq)]
pub enum ReductionOp {
    Sum,
    Product,
    Max,
    Min,
    And,
    Or,
    Xor,
}

/// Memory access patterns
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryAccessPattern {
    /// Sequential access with stride 1
    Sequential,

    /// Strided access with constant stride
    Strided(i64),

    /// Gather/scatter (irregular access)
    Irregular,

    /// Broadcast (same value accessed)
    Broadcast,
}

/// Data dependency analyzer
#[derive(Debug, Default)]
pub struct DependencyAnalyzer {
    /// Read-after-write dependencies
    raw_deps: Vec<Dependency>,

    /// Write-after-read dependencies
    war_deps: Vec<Dependency>,

    /// Write-after-write dependencies
    waw_deps: Vec<Dependency>,
}

/// Data dependency between statements
#[derive(Debug, Clone)]
pub struct Dependency {
    pub from_statement: usize,
    pub to_statement: usize,
    pub distance: Option<i64>,
    pub dependency_type: DependencyType,
}

/// Types of data dependencies
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    Flow,   // RAW
    Anti,   // WAR
    Output, // WAW
    Input,  // RAR (not a true dependency)
}

impl VectorizationPass {
    pub fn new() -> Self {
        let mut vector_widths = HashMap::new();

        // Common SIMD vector widths for different data types
        vector_widths.insert(PrimitiveType::Integer, 4); // 4x i32 (128-bit)
        vector_widths.insert(PrimitiveType::Float, 4); // 4x f32 (128-bit)
        vector_widths.insert(PrimitiveType::Boolean, 16); // 16x bool (128-bit)

        Self {
            vector_widths,
            vectorizable_loops: Vec::new(),
            dependency_analyzer: DependencyAnalyzer::default(),
        }
    }

    /// Analyze function for vectorization opportunities
    pub fn analyze_function(&mut self, function: &Function) -> Result<(), SemanticError> {
        self.vectorizable_loops.clear();

        // Find loops in the function
        let loops = self.find_loops(function)?;

        // Analyze each loop for vectorization potential
        for loop_info in loops {
            if let Some(vectorizable) = self.analyze_loop(function, &loop_info)? {
                self.vectorizable_loops.push(vectorizable);
            }
        }

        Ok(())
    }

    /// Find loops in the function
    fn find_loops(&self, function: &Function) -> Result<Vec<LoopInfo>, SemanticError> {
        let mut loops = Vec::new();
        let mut visited = HashSet::new();

        // Simple loop detection using back edges
        for block_id in function.basic_blocks.keys() {
            if visited.contains(block_id) {
                continue;
            }

            if let Some(loop_info) = self.detect_simple_loop(function, *block_id as usize)? {
                loops.push(loop_info);
                visited.insert(*block_id);
            }
        }

        Ok(loops)
    }

    /// Detect a simple loop starting from a block
    fn detect_simple_loop(
        &self,
        function: &Function,
        start_block: usize,
    ) -> Result<Option<LoopInfo>, SemanticError> {
        let block = function
            .basic_blocks
            .get(&(start_block as u32))
            .ok_or_else(|| SemanticError::Internal {
                message: format!("Block {} not found", start_block),
            })?;

        // Look for loop pattern: header -> body -> back edge to header
        if let Terminator::SwitchInt { targets, .. } = &block.terminator {
            // Check if one of the targets points back to this block
            for target in &targets.targets {
                if *target == start_block as u32 {
                    return Ok(Some(LoopInfo {
                        header: start_block,
                        blocks: HashSet::new(),
                        induction_variable: None,
                    }));
                }
            }
            // Also check the otherwise target
            if targets.otherwise == start_block as u32 {
                return Ok(Some(LoopInfo {
                    header: start_block,
                    blocks: HashSet::new(),
                    induction_variable: None,
                }));
            }
        }

        Ok(None)
    }

    /// Analyze a loop for vectorization potential
    fn analyze_loop(
        &mut self,
        function: &Function,
        loop_info: &LoopInfo,
    ) -> Result<Option<VectorizableLoop>, SemanticError> {
        let header_block = function
            .basic_blocks
            .get(&(loop_info.header as u32))
            .ok_or_else(|| SemanticError::Internal {
                message: format!("Loop header block {} not found", loop_info.header),
            })?;

        // Find induction variable
        let induction_var = self.find_induction_variable(header_block)?;

        // Analyze loop bounds
        let bounds = self.analyze_loop_bounds(header_block, &induction_var)?;

        // Find vectorizable statements
        let vectorizable_statements = self.find_vectorizable_statements(function, loop_info)?;

        // Check data dependencies
        if !self.check_vectorization_legality(function, loop_info, &vectorizable_statements)? {
            return Ok(None);
        }

        // Calculate benefit score
        let benefit_score = self.calculate_benefit_score(&vectorizable_statements, &bounds);

        // Determine vector width
        let vector_width = self.determine_vector_width(function, &vectorizable_statements);

        // Only vectorize if beneficial
        if benefit_score > 1.0 && vector_width > 1 {
            Ok(Some(VectorizableLoop {
                header_block: loop_info.header,
                induction_var,
                bounds,
                vectorizable_statements,
                benefit_score,
                vector_width,
            }))
        } else {
            Ok(None)
        }
    }

    /// Find the induction variable of a loop
    fn find_induction_variable(&self, header_block: &BasicBlock) -> Result<Place, SemanticError> {
        // Look for pattern: i = i + 1 or i = i + step
        for statement in header_block.statements.iter() {
            if let Statement::Assign { place, rvalue, .. } = statement {
                if let Rvalue::BinaryOp {
                    op: BinOp::Add,
                    left,
                    right,
                } = rvalue
                {
                    // Check if left operand is the same as the place being assigned
                    if let Operand::Move(left_place) | Operand::Copy(left_place) = left {
                        if left_place.local == place.local {
                            // Check if right operand is a constant (step)
                            if let Operand::Constant(_) = right {
                                return Ok(place.clone());
                            }
                        }
                    }
                }
            }
        }

        Err(SemanticError::Internal {
            message: "No induction variable found".to_string(),
        })
    }

    /// Analyze loop bounds
    fn analyze_loop_bounds(
        &self,
        header_block: &BasicBlock,
        _induction_var: &Place,
    ) -> Result<LoopBounds, SemanticError> {
        // Look for terminator condition that compares induction variable
        if let Terminator::SwitchInt {
            discriminant: _discr,
            ..
        } = &header_block.terminator
        {
            // Simplified analysis - assume comparison with constant
            return Ok(LoopBounds {
                start: Operand::Constant(crate::mir::Constant {
                    ty: Type::primitive(PrimitiveType::Integer),
                    value: crate::mir::ConstantValue::Integer(0),
                }),
                end: Operand::Constant(crate::mir::Constant {
                    ty: Type::primitive(PrimitiveType::Integer),
                    value: crate::mir::ConstantValue::Integer(100),
                }),
                step: 1,
                is_known_count: false,
                iteration_count: None,
            });
        }

        // Default bounds if analysis fails
        Ok(LoopBounds {
            start: Operand::Constant(crate::mir::Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: crate::mir::ConstantValue::Integer(0),
            }),
            end: Operand::Constant(crate::mir::Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: crate::mir::ConstantValue::Integer(0),
            }),
            step: 1,
            is_known_count: false,
            iteration_count: None,
        })
    }

    /// Find statements that can be vectorized
    fn find_vectorizable_statements(
        &self,
        function: &Function,
        loop_info: &LoopInfo,
    ) -> Result<Vec<VectorizableStatement>, SemanticError> {
        let mut vectorizable = Vec::new();

        for &block_id in &loop_info.blocks {
            let block = function
                .basic_blocks
                .get(&(block_id as u32))
                .ok_or_else(|| SemanticError::Internal {
                    message: format!("Block {} not found", block_id),
                })?;

            for (index, statement) in block.statements.iter().enumerate() {
                if let Some(vectorizable_stmt) =
                    self.analyze_statement_for_vectorization(function, statement, index)?
                {
                    vectorizable.push(vectorizable_stmt);
                }
            }
        }

        Ok(vectorizable)
    }

    /// Analyze a statement for vectorization potential
    fn analyze_statement_for_vectorization(
        &self,
        function: &Function,
        statement: &Statement,
        index: usize,
    ) -> Result<Option<VectorizableStatement>, SemanticError> {
        if let Statement::Assign { place, rvalue, .. } = statement {
            match rvalue {
                Rvalue::BinaryOp { op, left, right } => {
                    // Check if this is a vectorizable arithmetic operation
                    if let Some(local) = function.locals.get(&place.local) {
                        if self.is_vectorizable_type(&local.ty) {
                            let access_pattern = self.analyze_memory_access_pattern(left, right);

                            return Ok(Some(VectorizableStatement {
                                statement_index: index,
                                vector_op: VectorOperation::Arithmetic(*op),
                                inputs: vec![left.clone(), right.clone()],
                                output: place.clone(),
                                access_pattern,
                            }));
                        }
                    }
                }
                Rvalue::UnaryOp { op, operand } => {
                    if let Some(local) = function.locals.get(&place.local) {
                        if self.is_vectorizable_type(&local.ty) {
                            let access_pattern = self.analyze_single_operand_access(operand);

                            return Ok(Some(VectorizableStatement {
                                statement_index: index,
                                vector_op: VectorOperation::Unary(*op),
                                inputs: vec![operand.clone()],
                                output: place.clone(),
                                access_pattern,
                            }));
                        }
                    }
                }
                Rvalue::Use(operand) => {
                    // Simple assignment/load
                    if let Some(local) = function.locals.get(&place.local) {
                        if self.is_vectorizable_type(&local.ty) {
                            let access_pattern = self.analyze_single_operand_access(operand);

                            return Ok(Some(VectorizableStatement {
                                statement_index: index,
                                vector_op: VectorOperation::Load,
                                inputs: vec![operand.clone()],
                                output: place.clone(),
                                access_pattern,
                            }));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(None)
    }

    /// Check if a type can be vectorized
    fn is_vectorizable_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Primitive(prim_ty) => {
                matches!(
                    prim_ty,
                    PrimitiveType::Integer | PrimitiveType::Float | PrimitiveType::Boolean
                )
            }
            _ => false,
        }
    }

    /// Analyze memory access pattern for binary operation
    fn analyze_memory_access_pattern(
        &self,
        left: &Operand,
        right: &Operand,
    ) -> MemoryAccessPattern {
        // Simplified analysis - assume sequential access for now
        match (left, right) {
            (Operand::Move(_) | Operand::Copy(_), Operand::Move(_) | Operand::Copy(_)) => {
                MemoryAccessPattern::Sequential
            }
            (Operand::Constant(_), _) | (_, Operand::Constant(_)) => MemoryAccessPattern::Broadcast,
        }
    }

    /// Analyze memory access pattern for single operand
    fn analyze_single_operand_access(&self, operand: &Operand) -> MemoryAccessPattern {
        match operand {
            Operand::Move(_) | Operand::Copy(_) => MemoryAccessPattern::Sequential,
            Operand::Constant(_) => MemoryAccessPattern::Broadcast,
        }
    }

    /// Check if vectorization is legal (no problematic dependencies)
    fn check_vectorization_legality(
        &mut self,
        function: &Function,
        loop_info: &LoopInfo,
        statements: &[VectorizableStatement],
    ) -> Result<bool, SemanticError> {
        // Analyze data dependencies
        self.dependency_analyzer
            .analyze_dependencies(function, loop_info)?;

        // Check for loop-carried dependencies that prevent vectorization
        for stmt in statements {
            if self.has_loop_carried_dependency(stmt)? {
                return Ok(false);
            }
        }

        // Check memory aliasing
        if self.has_memory_aliasing_issues(statements)? {
            return Ok(false);
        }

        Ok(true)
    }

    /// Check if statement has loop-carried dependencies
    fn has_loop_carried_dependency(
        &self,
        _statement: &VectorizableStatement,
    ) -> Result<bool, SemanticError> {
        // Simplified check - in reality this would be much more sophisticated
        Ok(false)
    }

    /// Check for memory aliasing issues
    fn has_memory_aliasing_issues(
        &self,
        _statements: &[VectorizableStatement],
    ) -> Result<bool, SemanticError> {
        // Simplified check - assume no aliasing for now
        Ok(false)
    }

    /// Calculate benefit score for vectorization
    fn calculate_benefit_score(
        &self,
        statements: &[VectorizableStatement],
        bounds: &LoopBounds,
    ) -> f64 {
        let mut score = 0.0;

        // Base score from number of vectorizable operations
        score += statements.len() as f64 * 2.0;

        // Bonus for known iteration count
        if bounds.is_known_count {
            score *= 1.5;
        }

        // Bonus for good memory access patterns
        for stmt in statements {
            match stmt.access_pattern {
                MemoryAccessPattern::Sequential => score += 1.0,
                MemoryAccessPattern::Strided(_) => score += 0.5,
                MemoryAccessPattern::Broadcast => score += 0.3,
                MemoryAccessPattern::Irregular => score -= 1.0,
            }
        }

        // Penalty for small loops
        if let Some(count) = bounds.iteration_count {
            if count < 8 {
                score *= 0.5;
            }
        }

        score
    }

    /// Determine optimal vector width
    fn determine_vector_width(
        &self,
        function: &Function,
        statements: &[VectorizableStatement],
    ) -> usize {
        let mut min_width = 16; // Start with maximum

        for stmt in statements {
            if let Some(local) = function.locals.get(&stmt.output.local) {
                if let Type::Primitive(prim_ty) = &local.ty {
                    if let Some(&width) = self.vector_widths.get(prim_ty) {
                        min_width = min_width.min(width);
                    }
                }
            }
        }

        min_width
    }

    /// Apply vectorization to the function
    fn apply_vectorization(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        let mut changed = false;

        for vectorizable_loop in &self.vectorizable_loops {
            if self.vectorize_loop(function, vectorizable_loop)? {
                changed = true;
            }
        }

        Ok(changed)
    }

    /// Vectorize a specific loop
    fn vectorize_loop(
        &self,
        _function: &mut Function,
        vectorizable_loop: &VectorizableLoop,
    ) -> Result<bool, SemanticError> {
        // This is a placeholder for actual vectorization transformation
        // In a real implementation, this would:
        // 1. Create vector versions of the loop body
        // 2. Add prologue and epilogue for partial vectors
        // 3. Replace scalar operations with vector intrinsics
        // 4. Update the control flow

        eprintln!(
            "Would vectorize loop at block {} with width {}",
            vectorizable_loop.header_block, vectorizable_loop.vector_width
        );

        Ok(false) // Not actually implemented yet
    }
}

/// Basic loop information
#[derive(Debug, Clone)]
struct LoopInfo {
    header: usize,
    blocks: HashSet<usize>,
    induction_variable: Option<Place>,
}

impl DependencyAnalyzer {
    /// Analyze data dependencies in a loop
    fn analyze_dependencies(
        &mut self,
        function: &Function,
        loop_info: &LoopInfo,
    ) -> Result<(), SemanticError> {
        self.raw_deps.clear();
        self.war_deps.clear();
        self.waw_deps.clear();

        // Analyze dependencies within each block
        for &block_id in &loop_info.blocks {
            let block = function
                .basic_blocks
                .get(&(block_id as u32))
                .ok_or_else(|| SemanticError::Internal {
                    message: format!("Block {} not found", block_id),
                })?;

            self.analyze_block_dependencies(block)?;
        }

        Ok(())
    }

    /// Analyze dependencies within a single block
    fn analyze_block_dependencies(&mut self, block: &BasicBlock) -> Result<(), SemanticError> {
        // Simple dependency analysis - check for read-after-write patterns
        for (i, stmt1) in block.statements.iter().enumerate() {
            for (j, stmt2) in block.statements.iter().enumerate().skip(i + 1) {
                if let Some(dep) = self.find_dependency(stmt1, stmt2, i, j)? {
                    match dep.dependency_type {
                        DependencyType::Flow => self.raw_deps.push(dep),
                        DependencyType::Anti => self.war_deps.push(dep),
                        DependencyType::Output => self.waw_deps.push(dep),
                        DependencyType::Input => {} // Not stored
                    }
                }
            }
        }

        Ok(())
    }

    /// Find dependency between two statements
    fn find_dependency(
        &self,
        stmt1: &Statement,
        stmt2: &Statement,
        index1: usize,
        index2: usize,
    ) -> Result<Option<Dependency>, SemanticError> {
        // Simplified dependency detection
        if let (
            Statement::Assign { place: place1, .. },
            Statement::Assign {
                rvalue: rvalue2, ..
            },
        ) = (stmt1, stmt2)
        {
            // Check if stmt2 reads what stmt1 writes (RAW)
            if self.rvalue_reads_place(rvalue2, place1) {
                return Ok(Some(Dependency {
                    from_statement: index1,
                    to_statement: index2,
                    distance: Some((index2 - index1) as i64),
                    dependency_type: DependencyType::Flow,
                }));
            }
        }

        Ok(None)
    }

    /// Check if an rvalue reads from a place
    fn rvalue_reads_place(&self, rvalue: &Rvalue, place: &Place) -> bool {
        match rvalue {
            Rvalue::Use(operand) => self.operand_reads_place(operand, place),
            Rvalue::BinaryOp { left, right, .. } => {
                self.operand_reads_place(left, place) || self.operand_reads_place(right, place)
            }
            Rvalue::UnaryOp { operand, .. } => self.operand_reads_place(operand, place),
            _ => false,
        }
    }

    /// Check if an operand reads from a place
    fn operand_reads_place(&self, operand: &Operand, place: &Place) -> bool {
        match operand {
            Operand::Move(op_place) | Operand::Copy(op_place) => op_place.local == place.local,
            Operand::Constant(_) => false,
        }
    }
}

impl OptimizationPass for VectorizationPass {
    fn name(&self) -> &'static str {
        "AutoVectorization"
    }

    fn run_on_function(&mut self, function: &mut Function) -> Result<bool, SemanticError> {
        // Analyze function for vectorization opportunities
        self.analyze_function(function)?;

        // Apply vectorization if beneficial
        self.apply_vectorization(function)
    }
}

impl Default for VectorizationPass {
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
    fn test_vectorization_pass_creation() {
        let pass = VectorizationPass::new();
        assert_eq!(pass.name(), "AutoVectorization");
        assert!(pass.vectorizable_loops.is_empty());
    }

    #[test]
    fn test_vectorizable_type_detection() {
        let pass = VectorizationPass::new();

        assert!(pass.is_vectorizable_type(&Type::primitive(PrimitiveType::Integer)));
        assert!(pass.is_vectorizable_type(&Type::primitive(PrimitiveType::Float)));
        assert!(pass.is_vectorizable_type(&Type::primitive(PrimitiveType::Boolean)));
        assert!(!pass.is_vectorizable_type(&Type::primitive(PrimitiveType::String)));
    }

    #[test]
    fn test_memory_access_pattern_analysis() {
        let pass = VectorizationPass::new();

        let const_operand = Operand::Constant(Constant {
            ty: Type::primitive(PrimitiveType::Integer),
            value: ConstantValue::Integer(42),
        });

        let pattern = pass.analyze_single_operand_access(&const_operand);
        assert_eq!(pattern, MemoryAccessPattern::Broadcast);
    }

    #[test]
    fn test_vector_width_determination() {
        let pass = VectorizationPass::new();

        let statements = vec![VectorizableStatement {
            statement_index: 0,
            vector_op: VectorOperation::Arithmetic(BinOp::Add),
            inputs: vec![],
            output: Place {
                local: 0,
                projection: vec![],
            },
            access_pattern: MemoryAccessPattern::Sequential,
        }];

        // Create a dummy function for testing
        let mut locals = HashMap::new();
        locals.insert(
            0,
            crate::mir::Local {
                ty: Type::primitive(PrimitiveType::Integer),
                is_mutable: true,
                source_info: None,
            },
        );

        let function = Function {
            name: "test".to_string(),
            parameters: vec![],
            return_type: Type::primitive(PrimitiveType::Void),
            locals,
            basic_blocks: HashMap::new(),
            entry_block: 0,
            return_local: None,
        };

        let width = pass.determine_vector_width(&function, &statements);
        assert_eq!(width, 4); // Expected width for integers
    }

    #[test]
    fn test_benefit_score_calculation() {
        let pass = VectorizationPass::new();

        let statements = vec![VectorizableStatement {
            statement_index: 0,
            vector_op: VectorOperation::Arithmetic(BinOp::Add),
            inputs: vec![],
            output: Place {
                local: 0,
                projection: vec![],
            },
            access_pattern: MemoryAccessPattern::Sequential,
        }];

        let bounds = LoopBounds {
            start: Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(0),
            }),
            end: Operand::Constant(Constant {
                ty: Type::primitive(PrimitiveType::Integer),
                value: ConstantValue::Integer(100),
            }),
            step: 1,
            is_known_count: true,
            iteration_count: Some(100),
        };

        let score = pass.calculate_benefit_score(&statements, &bounds);
        assert!(score > 0.0);
    }

    #[test]
    fn test_dependency_analyzer() {
        let mut analyzer = DependencyAnalyzer::default();
        let mut builder = Builder::new();

        // Create a simple block for testing
        let block = BasicBlock {
            id: 0,
            statements: vec![],
            terminator: Terminator::Return,
        };

        assert!(analyzer.analyze_block_dependencies(&block).is_ok());
        assert!(analyzer.raw_deps.is_empty());
    }
}
