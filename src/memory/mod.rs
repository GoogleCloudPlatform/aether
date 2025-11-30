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

//! Memory management system for AetherScript
//!
//! Implements deterministic memory management with region-based allocation,
//! reference counting, and linear types for zero-copy operations.

#![allow(dead_code)]

use crate::ast::*;
use crate::error::{SemanticError, SourceLocation};
use crate::types::{Type, TypeChecker};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Memory allocation strategy
#[derive(Debug, Clone, PartialEq)]
pub enum AllocationStrategy {
    /// Stack allocation (default for small, fixed-size values)
    Stack,
    /// Region-based heap allocation
    Region(RegionId),
    /// Reference counted heap allocation
    RefCounted,
    /// Linear type (single ownership, zero-copy)
    Linear,
}

/// Region identifier for region-based allocation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegionId(pub usize);

/// Memory region for grouped allocations
#[derive(Debug)]
pub enum MemoryPermissions {
    Read,
    Write,
    Execute,
}

#[derive(Debug)]
pub enum AllocationStatus {
    Free,
    Allocated,
}

#[derive(Debug)]
struct MemoryRegion {
    /// Start address
    start: usize,

    /// Current size
    size: usize,

    /// Permissions
    permissions: MemoryPermissions,

    /// Allocation status
    status: AllocationStatus,
}

/// Information about a single allocation
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub variable_name: String,
    pub allocation_type: Type,
    pub size_bytes: usize,
    pub strategy: AllocationStrategy,
    pub location: SourceLocation,
}

/// Memory analyzer for determining allocation strategies
pub struct MemoryAnalyzer {
    type_checker: Rc<RefCell<TypeChecker>>,
    regions: HashMap<RegionId, MemoryRegion>,
    active_region: Option<RegionId>,
    region_stack: Vec<Option<RegionId>>,
    next_region_id: usize,
    allocation_map: HashMap<String, AllocationInfo>,
    linear_types: HashSet<String>,
    escape_analysis: EscapeAnalyzer,
}

impl MemoryAnalyzer {
    pub fn new(type_checker: Rc<RefCell<TypeChecker>>) -> Self {
        Self {
            type_checker,
            regions: HashMap::new(),
            active_region: None,
            region_stack: Vec::new(),
            next_region_id: 0,
            allocation_map: HashMap::new(),
            linear_types: HashSet::new(),
            escape_analysis: EscapeAnalyzer::new(),
        }
    }

    /// Create a new memory region
    pub fn create_region(&mut self, _parent: Option<RegionId>) -> RegionId {
        let id = RegionId(self.next_region_id);
        self.next_region_id += 1;

        let region = MemoryRegion {
            start: 0,
            size: 0,
            permissions: MemoryPermissions::Read,
            status: AllocationStatus::Free,
        };

        self.regions.insert(id.clone(), region);
        id
    }

    /// Enter a memory region
    pub fn enter_region(&mut self, region_id: RegionId) {
        self.region_stack.push(self.active_region.clone());
        self.active_region = Some(region_id);
    }

    /// Exit current region and deallocate if needed
    pub fn exit_region(&mut self) -> Result<(), SemanticError> {
        if let Some(_region_id) = &self.active_region {
            // Restore the previous region from the stack
            if let Some(parent_region) = self.region_stack.pop() {
                self.active_region = parent_region;
                Ok(())
            } else {
                // Stack is empty, this was the root region
                self.active_region = None;
                Ok(())
            }
        } else {
            Err(SemanticError::Internal {
                message: "No active region to exit".to_string(),
            })
        }
    }

    /// Analyze expression memory usage (helper)
    fn analyze_expression_memory(
        &mut self,
        expr: &Expression,
        function: &Function,
        region_id: &RegionId,
    ) -> Result<(), SemanticError> {
        // Simple recursive walk for now
        match expr {
            Expression::AddressOf { operand, .. } => {
                self.analyze_expression_memory(operand, function, region_id)
            },
            Expression::Dereference { pointer, .. } => {
                self.analyze_expression_memory(pointer, function, region_id)
            },
            // Add other expression types as needed
            _ => Ok(()),
        }
    }

    /// Analyze a function for memory allocation strategy
    pub fn analyze_function(
        &mut self,
        function: &Function,
    ) -> Result<FunctionMemoryInfo, SemanticError> {
        // Create function-level region
        let function_region = self.create_region(self.active_region.clone());
        self.enter_region(function_region.clone());

        // Analyze parameters
        let mut param_allocations = Vec::new();
        for param in &function.parameters {
            let param_type = self
                .type_checker
                .borrow()
                .ast_type_to_type(&param.param_type)?;
            let allocation = self.determine_allocation_strategy(
                &param.name.name,
                &param_type,
                &param.source_location,
                false, // Parameters are not mutable by default
            )?;
            param_allocations.push(allocation);
        }

        // Analyze function body
        self.analyze_block(&function.body)?;

        // Perform escape analysis
        let escapes = self.escape_analysis.analyze_function(function)?;

        // Exit function region
        self.exit_region()?;

        Ok(FunctionMemoryInfo {
            region_id: function_region.clone(),
            parameter_allocations: param_allocations,
            local_allocations: self.get_region_allocations(&function_region),
            escaping_values: escapes,
        })
    }

    /// Analyze a block for memory allocations
    fn analyze_block(&mut self, block: &Block) -> Result<(), SemanticError> {
        // Create block-level region
        let block_region = self.create_region(self.active_region.clone());
        self.enter_region(block_region);

        for statement in &block.statements {
            self.analyze_statement(statement)?;
        }

        // Exit block region
        self.exit_region()?;

        Ok(())
    }

    /// Analyze a statement for memory allocations
    fn analyze_statement(&mut self, statement: &Statement) -> Result<(), SemanticError> {
        match statement {
            Statement::VariableDeclaration {
                name,
                type_spec,
                mutability,
                initial_value,
                source_location,
                ..
            } => {
                let var_type = self.type_checker.borrow().ast_type_to_type(type_spec)?;
                let is_mutable = matches!(mutability, Mutability::Mutable);

                // Check if this should be a linear type
                let is_linear = self.should_use_linear_type(&var_type, initial_value.as_deref());

                let allocation = self.determine_allocation_strategy(
                    &name.name,
                    &var_type,
                    source_location,
                    is_mutable,
                )?;

                if is_linear {
                    self.linear_types.insert(name.name.clone());
                }

                self.allocation_map.insert(name.name.clone(), allocation);
            }

            Statement::Assignment { target, value, .. } => {
                // Check for linear type violations
                if let AssignmentTarget::Variable { name } = target {
                    if self.linear_types.contains(&name.name) {
                        // Ensure the old value is consumed
                        self.validate_linear_type_assignment(&name.name, value)?;
                    }
                }
            }

            Statement::If {
                then_block,
                else_ifs,
                else_block,
                ..
            } => {
                self.analyze_block(then_block)?;
                for else_if in else_ifs {
                    self.analyze_block(&else_if.block)?;
                }
                if let Some(else_block) = else_block {
                    self.analyze_block(else_block)?;
                }
            }

            Statement::WhileLoop { body, .. }
            | Statement::ForEachLoop { body, .. }
            | Statement::FixedIterationLoop { body, .. } => {
                self.analyze_block(body)?;
            }

            Statement::TryBlock {
                protected_block,
                catch_clauses,
                finally_block,
                ..
            } => {
                self.analyze_block(protected_block)?;
                for catch in catch_clauses {
                    self.analyze_block(&catch.handler_block)?;
                }
                if let Some(finally) = finally_block {
                    self.analyze_block(finally)?;
                }
            }

            Statement::Expression { expr, .. } => {
                // Check for AddressOf in expression statements
                if let Expression::AddressOf {  .. } = &**expr {
                    // This is a simplification. Ideally we'd pass function/region context down.
                    // For now, we just need to make sure we don't crash on AddressOf.
                    // In a real implementation, we'd want to register the address-taken variable as escaping.
                    // But analyze_statement doesn't have access to Function/RegionId easily right now.
                    // Let's rely on EscapeAnalyzer for the heavy lifting.
                }
            }

            _ => {}
        }

        Ok(())
    }

    /// Determine allocation strategy for a variable
    fn determine_allocation_strategy(
        &self,
        name: &str,
        var_type: &Type,
        location: &SourceLocation,
        is_mutable: bool,
    ) -> Result<AllocationInfo, SemanticError> {
        let size = self.estimate_type_size(var_type);

        // Determine strategy based on type and size
        let strategy = if size <= 64 && !is_mutable && self.is_stack_eligible(var_type) {
            AllocationStrategy::Stack
        } else if self.should_use_ref_counting(var_type) {
            AllocationStrategy::RefCounted
        } else if let Some(region_id) = &self.active_region {
            AllocationStrategy::Region(region_id.clone())
        } else {
            AllocationStrategy::Stack // Fallback
        };

        Ok(AllocationInfo {
            variable_name: name.to_string(),
            allocation_type: var_type.clone(),
            size_bytes: size,
            strategy,
            location: location.clone(),
        })
    }

    /// Estimate size of a type in bytes
    fn estimate_type_size(&self, var_type: &Type) -> usize {
        match var_type {
            Type::Primitive(prim) => match prim {
                PrimitiveType::Boolean => 1,
                PrimitiveType::Integer32 | PrimitiveType::Float32 => 4,
                PrimitiveType::Integer
                | PrimitiveType::Integer64
                | PrimitiveType::Float
                | PrimitiveType::Float64 => 8,
                PrimitiveType::String => 24, // String header size
                _ => 8,                      // Default pointer size
            },
            Type::Pointer { .. } => 8,
            Type::Array { element_type, .. } => {
                // Array header + elements (estimate)
                24 + self.estimate_type_size(element_type) * 10
            }
            Type::Map { .. } => 48,      // HashMap header estimate
            Type::Function { .. } => 16, // Function pointer
            _ => 24,                     // Default struct size
        }
    }

    /// Check if type is eligible for stack allocation
    fn is_stack_eligible(&self, var_type: &Type) -> bool {
        match var_type {
            Type::Primitive(_) => true,
            Type::Pointer { .. } => true,
            Type::Function { .. } => true,
            _ => false,
        }
    }

    /// Check if type should use reference counting
    fn should_use_ref_counting(&self, var_type: &Type) -> bool {
        match var_type {
            Type::Array { .. } | Type::Map { .. } => true,
            Type::Named { .. } => true, // User-defined types
            _ => false,
        }
    }

    /// Check if a type should be linear
    fn should_use_linear_type(&self, var_type: &Type, _initial_value: Option<&Expression>) -> bool {
        // Large arrays or unique resources should be linear
        match var_type {
            Type::Array { .. } => {
                // Large arrays benefit from zero-copy
                self.estimate_type_size(var_type) > 1024
            }
            Type::Pointer {
                is_mutable: true, ..
            } => {
                // Mutable unique pointers should be linear
                true
            }
            _ => false,
        }
    }

    /// Validate linear type assignment
    fn validate_linear_type_assignment(
        &self,
        name: &str,
        value: &Expression,
    ) -> Result<(), SemanticError> {
        // Check that the value is a move, not a copy
        match value {
            Expression::Variable {
                name: var_name,
                source_location,
            } => {
                if self.linear_types.contains(&var_name.name) {
                    // This is a move - the source variable will be invalidated
                    Ok(())
                } else {
                    Err(SemanticError::InvalidType {
                        type_name: name.to_string(),
                        reason: "Cannot copy non-linear value to linear type".to_string(),
                        location: source_location.clone(),
                    })
                }
            }
            _ => Ok(()), // Other expressions create new values
        }
    }

    /// Get allocations for a specific region
    fn get_region_allocations(&self, _region_id: &RegionId) -> Vec<AllocationInfo> {
        vec![]
    }
}

/// Escape analysis to determine if values escape their scope
struct EscapeAnalyzer {
    escaping_values: HashSet<String>,
}

impl EscapeAnalyzer {
    fn new() -> Self {
        Self {
            escaping_values: HashSet::new(),
        }
    }

    fn analyze_function(&mut self, function: &Function) -> Result<HashSet<String>, SemanticError> {
        self.escaping_values.clear();

        // Analyze function body for escaping values
        self.analyze_block_for_escapes(&function.body)?;

        Ok(self.escaping_values.clone())
    }

    fn analyze_block_for_escapes(&mut self, block: &Block) -> Result<(), SemanticError> {
        for statement in &block.statements {
            self.analyze_statement_for_escapes(statement)?;
        }
        Ok(())
    }

    fn analyze_statement_for_escapes(
        &mut self,
        statement: &Statement,
    ) -> Result<(), SemanticError> {
        match statement {
            Statement::Return {
                value: Some(expr), ..
            } => {
                // Values returned from functions escape
                self.mark_escaping_expression(expr);
            }
            Statement::Assignment { target, value, .. } => {
                // Check if assignment causes escape
                if self.is_escaping_target(target) {
                    self.mark_escaping_expression(value);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn mark_escaping_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Variable { name, .. } => {
                self.escaping_values.insert(name.name.clone());
            }
            Expression::AddressOf { operand, .. } => {
                // Taking address of a variable causes it to escape
                self.mark_escaping_expression(operand);
            }
            _ => {}
        }
    }

    fn is_escaping_target(&self, target: &AssignmentTarget) -> bool {
        match target {
            AssignmentTarget::StructField { .. }
            | AssignmentTarget::ArrayElement { .. }
            | AssignmentTarget::MapValue { .. } => true,
            _ => false,
        }
    }
}

/// Memory information for a function
#[derive(Debug)]
pub struct FunctionMemoryInfo {
    pub region_id: RegionId,
    pub parameter_allocations: Vec<AllocationInfo>,
    pub local_allocations: Vec<AllocationInfo>,
    pub escaping_values: HashSet<String>,
}

/// Reference counting wrapper for heap allocations
#[derive(Debug)]
pub struct RefCounted<T> {
    value: Rc<RefCell<T>>,
}

impl<T> RefCounted<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            value: Rc::clone(&self.value),
        }
    }

    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.value)
    }
}

/// Linear type wrapper for zero-copy operations
#[derive(Debug)]
pub struct Linear<T> {
    value: Option<T>,
}

impl<T> Linear<T> {
    pub fn new(value: T) -> Self {
        Self { value: Some(value) }
    }

    pub fn take(mut self) -> T {
        self.value.take().expect("Linear value already consumed")
    }

    pub fn is_consumed(&self) -> bool {
        self.value.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_type() -> Type {
        Type::primitive(PrimitiveType::Integer)
    }

    #[test]
    fn test_memory_analyzer_creation() {
        let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
        let analyzer = MemoryAnalyzer::new(type_checker);
        assert_eq!(analyzer.next_region_id, 0);
        assert!(analyzer.regions.is_empty());
    }

    #[test]
    fn test_region_creation() {
        let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
        let mut analyzer = MemoryAnalyzer::new(type_checker);

        let region1 = analyzer.create_region(None);
        assert_eq!(region1.0, 0);

        let region2 = analyzer.create_region(Some(region1.clone()));
        assert_eq!(region2.0, 1);

        assert_eq!(analyzer.regions.len(), 2);
    }

    #[test]
    fn test_allocation_strategy_determination() {
        let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
        let analyzer = MemoryAnalyzer::new(type_checker);

        // Small primitive should use stack
        let int_type = Type::primitive(PrimitiveType::Integer);
        let alloc = analyzer
            .determine_allocation_strategy("x", &int_type, &SourceLocation::unknown(), false)
            .unwrap();

        assert_eq!(alloc.strategy, AllocationStrategy::Stack);
        assert_eq!(alloc.size_bytes, 8);

        // Array should use ref counting
        let array_type = Type::array(Type::primitive(PrimitiveType::Integer), None);
        let alloc = analyzer
            .determine_allocation_strategy("arr", &array_type, &SourceLocation::unknown(), false)
            .unwrap();

        assert_eq!(alloc.strategy, AllocationStrategy::RefCounted);
    }

    #[test]
    fn test_ref_counted_wrapper() {
        let rc1 = RefCounted::new(42);
        assert_eq!(rc1.strong_count(), 1);

        let rc2 = rc1.clone();
        assert_eq!(rc1.strong_count(), 2);
        assert_eq!(rc2.strong_count(), 2);
    }

    #[test]
    fn test_linear_type_wrapper() {
        let linear = Linear::new(vec![1, 2, 3]);
        assert!(!linear.is_consumed());

        let value = linear.take();
        assert_eq!(value, vec![1, 2, 3]);
    }

    #[test]
    #[should_panic(expected = "Linear value already consumed")]
    fn test_linear_type_double_consume() {
        let mut linear = Linear::new(42);
        // Consume the value
        let _ = std::mem::replace(&mut linear.value, None);
        // Try to take again - should panic
        linear.take();
    }
}
