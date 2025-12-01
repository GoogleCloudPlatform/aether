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

//! Contract validation and metadata processing for AetherScript
//!
//! Implements contract assertion validation, metadata parsing, and runtime assertion generation

use crate::ast::{
    ComplexityExpectation, ContractAssertion, Expression, FailureAction, FunctionMetadata,
    PerformanceExpectation,
};
use crate::error::{SemanticError, SourceLocation};
use crate::types::{Type, TypeChecker};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Contract validation result
#[derive(Debug, Clone)]
pub struct ContractValidationResult {
    pub is_valid: bool,
    pub errors: Vec<SemanticError>,
    pub warnings: Vec<String>,
}

/// Contract validation context
#[derive(Debug)]
pub struct ContractContext {
    /// Function parameter types
    pub parameter_types: HashMap<String, Type>,
    /// Function return type
    pub return_type: Type,
    /// Type checker for expression validation
    pub type_checker: Rc<RefCell<TypeChecker>>,
}

/// Contract validator for function metadata
pub struct ContractValidator {
    /// Statistics about contracts processed
    pub stats: ContractStats,
}

/// Statistics about contract validation
#[derive(Debug, Default)]
pub struct ContractStats {
    pub functions_processed: usize,
    pub preconditions_validated: usize,
    pub postconditions_validated: usize,
    pub invariants_validated: usize,
    pub performance_expectations_checked: usize,
    pub complexity_expectations_checked: usize,
    pub contract_errors: usize,
    pub contract_warnings: usize,
}

impl ContractValidator {
    /// Create a new contract validator
    pub fn new() -> Self {
        Self {
            stats: ContractStats::default(),
        }
    }

    /// Validate function metadata including contracts
    pub fn validate_function_metadata(
        &mut self,
        metadata: &FunctionMetadata,
        context: &ContractContext,
        function_name: &str,
        function_location: &SourceLocation,
    ) -> Result<ContractValidationResult, SemanticError> {
        let mut result = ContractValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        self.stats.functions_processed += 1;

        // Validate preconditions
        for precondition in &metadata.preconditions {
            match self.validate_contract_assertion(precondition, context, "precondition") {
                Ok(warnings) => {
                    self.stats.preconditions_validated += 1;
                    result.warnings.extend(warnings);
                }
                Err(error) => {
                    self.stats.contract_errors += 1;
                    result.errors.push(error);
                    result.is_valid = false;
                }
            }
        }

        // Validate postconditions
        for postcondition in &metadata.postconditions {
            match self.validate_contract_assertion(postcondition, context, "postcondition") {
                Ok(warnings) => {
                    self.stats.postconditions_validated += 1;
                    result.warnings.extend(warnings);
                }
                Err(error) => {
                    self.stats.contract_errors += 1;
                    result.errors.push(error);
                    result.is_valid = false;
                }
            }
        }

        // Validate invariants
        for invariant in &metadata.invariants {
            match self.validate_contract_assertion(invariant, context, "invariant") {
                Ok(warnings) => {
                    self.stats.invariants_validated += 1;
                    result.warnings.extend(warnings);
                }
                Err(error) => {
                    self.stats.contract_errors += 1;
                    result.errors.push(error);
                    result.is_valid = false;
                }
            }
        }

        // Validate performance expectations
        if let Some(perf_expectation) = &metadata.performance_expectation {
            if let Err(error) = self.validate_performance_expectation(
                perf_expectation,
                function_name,
                function_location,
            ) {
                self.stats.contract_errors += 1;
                result.errors.push(error);
                result.is_valid = false;
            } else {
                self.stats.performance_expectations_checked += 1;
            }
        }

        // Validate complexity expectations
        if let Some(complexity_expectation) = &metadata.complexity_expectation {
            if let Err(error) = self.validate_complexity_expectation(
                complexity_expectation,
                function_name,
                function_location,
            ) {
                self.stats.contract_errors += 1;
                result.errors.push(error);
                result.is_valid = false;
            } else {
                self.stats.complexity_expectations_checked += 1;
            }
        }

        // Validate algorithm hints
        if let Some(algorithm_hint) = &metadata.algorithm_hint {
            if let Some(warning) = self.validate_algorithm_hint(algorithm_hint, function_name) {
                result.warnings.push(warning);
            }
        }

        // Validate exception specifications
        for exception_type in &metadata.throws_exceptions {
            if let Err(error) = context
                .type_checker
                .borrow()
                .ast_type_to_type(exception_type)
            {
                result.errors.push(SemanticError::InvalidType {
                    type_name: format!("{:?}", exception_type),
                    reason: format!("Exception type validation failed: {}", error),
                    location: function_location.clone(),
                });
                result.is_valid = false;
            }
        }

        self.stats.contract_warnings += result.warnings.len();
        Ok(result)
    }

    /// Validate a single contract assertion
    fn validate_contract_assertion(
        &self,
        assertion: &ContractAssertion,
        context: &ContractContext,
        assertion_type: &str,
    ) -> Result<Vec<String>, SemanticError> {
        let mut warnings = Vec::new();

        // Validate that the condition expression is boolean
        let condition_type = self.infer_expression_type(&assertion.condition, context)?;

        if !matches!(
            condition_type,
            Type::Primitive(crate::ast::PrimitiveType::Boolean)
        ) {
            return Err(SemanticError::TypeMismatch {
                expected: "Boolean".to_string(),
                found: condition_type.to_string(),
                location: assertion.source_location.clone(),
            });
        }

        // Validate failure action
        match assertion.failure_action {
            FailureAction::ThrowException => {
                // Check if function declares that it throws exceptions
                warnings.push(format!(
                    "{} uses ThrowException but function may need exception declaration",
                    assertion_type
                ));
            }
            FailureAction::LogWarning => {
                // Warning-only contracts are generally safe
            }
            FailureAction::AssertFail => {
                // Assert failures should be used carefully in production
                warnings.push(format!(
                    "{} uses AssertFail which may terminate program execution",
                    assertion_type
                ));
            }
        }

        // Validate message if present
        if let Some(message) = &assertion.message {
            if message.is_empty() {
                warnings.push(format!(
                    "{} has empty message - consider providing meaningful error description",
                    assertion_type
                ));
            }
        }

        Ok(warnings)
    }

    /// Validate performance expectations
    fn validate_performance_expectation(
        &self,
        expectation: &PerformanceExpectation,
        _function_name: &str,
        location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // Check for reasonable performance values
        match expectation.metric {
            crate::ast::PerformanceMetric::LatencyMs => {
                if expectation.target_value <= 0.0 {
                    return Err(SemanticError::InvalidContract {
                        contract_type: "PerformanceExpectation".to_string(),
                        reason: "Latency must be positive".to_string(),
                        location: location.clone(),
                    });
                }
                if expectation.target_value > 60000.0 {
                    // 1 minute
                    return Err(SemanticError::InvalidContract {
                        contract_type: "PerformanceExpectation".to_string(),
                        reason: "Latency expectation exceeds reasonable bounds (60s)".to_string(),
                        location: location.clone(),
                    });
                }
            }
            crate::ast::PerformanceMetric::ThroughputOpsPerSec => {
                if expectation.target_value <= 0.0 {
                    return Err(SemanticError::InvalidContract {
                        contract_type: "PerformanceExpectation".to_string(),
                        reason: "Throughput must be positive".to_string(),
                        location: location.clone(),
                    });
                }
            }
            crate::ast::PerformanceMetric::MemoryUsageBytes => {
                if expectation.target_value < 0.0 {
                    return Err(SemanticError::InvalidContract {
                        contract_type: "PerformanceExpectation".to_string(),
                        reason: "Memory usage cannot be negative".to_string(),
                        location: location.clone(),
                    });
                }
                if expectation.target_value > 1_000_000_000.0 {
                    // 1GB
                    return Err(SemanticError::InvalidContract {
                        contract_type: "PerformanceExpectation".to_string(),
                        reason: "Memory usage expectation exceeds reasonable bounds (1GB)"
                            .to_string(),
                        location: location.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate complexity expectations
    fn validate_complexity_expectation(
        &self,
        expectation: &ComplexityExpectation,
        _function_name: &str,
        location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        // Validate complexity notation format
        let valid_complexities = [
            "O(1)",
            "O(log n)",
            "O(n)",
            "O(n log n)",
            "O(n^2)",
            "O(n^3)",
            "O(2^n)",
            "O(n!)",
            "Θ(1)",
            "Θ(log n)",
            "Θ(n)",
            "Θ(n log n)",
            "Θ(n^2)",
            "Θ(n^3)",
            "Θ(2^n)",
            "Θ(n!)",
            "Ω(1)",
            "Ω(log n)",
            "Ω(n)",
            "Ω(n log n)",
            "Ω(n^2)",
            "Ω(n^3)",
            "Ω(2^n)",
            "Ω(n!)",
        ];

        if !valid_complexities.contains(&expectation.value.as_str()) {
            return Err(SemanticError::InvalidContract {
                contract_type: "ComplexityExpectation".to_string(),
                reason: format!(
                    "Invalid complexity notation: '{}'. Expected standard big-O notation.",
                    expectation.value
                ),
                location: location.clone(),
            });
        }

        // Warn about exponential complexities
        if expectation.value.contains("2^n") || expectation.value.contains("n!") {
            // This is a warning, not an error, but we'll include it in the contract validation
        }

        Ok(())
    }

    /// Validate algorithm hints
    fn validate_algorithm_hint(&self, hint: &str, function_name: &str) -> Option<String> {
        let known_algorithms = [
            "binary_search",
            "linear_search",
            "quick_sort",
            "merge_sort",
            "heap_sort",
            "bubble_sort",
            "insertion_sort",
            "selection_sort",
            "radix_sort",
            "dijkstra",
            "bellman_ford",
            "floyd_warshall",
            "dfs",
            "bfs",
            "dynamic_programming",
            "greedy",
            "divide_and_conquer",
            "backtracking",
            "hash_table",
            "binary_tree",
            "balanced_tree",
            "graph_traversal",
        ];

        if !known_algorithms.contains(&hint) {
            Some(format!("Function '{}' uses unknown algorithm hint '{}' - consider using a standard algorithm name", function_name, hint))
        } else {
            None
        }
    }

    /// Infer the type of an expression (simplified version)
    fn infer_expression_type(
        &self,
        expression: &Expression,
        context: &ContractContext,
    ) -> Result<Type, SemanticError> {
        match expression {
            Expression::BooleanLiteral { .. } => {
                Ok(Type::primitive(crate::ast::PrimitiveType::Boolean))
            }
            Expression::IntegerLiteral { .. } => {
                Ok(Type::primitive(crate::ast::PrimitiveType::Integer))
            }
            Expression::FloatLiteral { .. } => {
                Ok(Type::primitive(crate::ast::PrimitiveType::Float))
            }
            Expression::StringLiteral { .. } => {
                Ok(Type::primitive(crate::ast::PrimitiveType::String))
            }
            Expression::Variable {
                name,
                source_location,
            } => context
                .parameter_types
                .get(&name.name)
                .cloned()
                .ok_or_else(|| SemanticError::UndefinedSymbol {
                    symbol: name.name.clone(),
                    location: source_location.clone(),
                }),
            Expression::Equals {
                left: _, right: _, ..
            }
            | Expression::NotEquals {
                left: _, right: _, ..
            }
            | Expression::LessThan {
                left: _, right: _, ..
            }
            | Expression::LessThanOrEqual {
                left: _, right: _, ..
            }
            | Expression::GreaterThan {
                left: _, right: _, ..
            }
            | Expression::GreaterThanOrEqual {
                left: _, right: _, ..
            } => {
                // Comparison operators always return boolean
                Ok(Type::primitive(crate::ast::PrimitiveType::Boolean))
            }
            Expression::LogicalAnd { .. }
            | Expression::LogicalOr { .. }
            | Expression::LogicalNot { .. } => {
                // Logical operators always return boolean
                Ok(Type::primitive(crate::ast::PrimitiveType::Boolean))
            }
            Expression::Add { left, .. }
            | Expression::Subtract { left, .. }
            | Expression::Multiply { left, .. }
            | Expression::Divide { left, .. }
            | Expression::IntegerDivide { left, .. }
            | Expression::Modulo { left, .. } => {
                // Recursively infer type of left operand (assuming both are same type)
                self.infer_expression_type(left, context)
            }
            // For other expression types, we'd need full expression type inference
            // For now, return an error indicating unsupported expression
            _ => Err(SemanticError::UnsupportedFeature {
                feature: "Complex expression type inference in contracts".to_string(),
                location: SourceLocation::unknown(),
            }),
        }
    }

    /// Generate runtime assertion code for debug builds
    pub fn generate_runtime_assertions(
        &self,
        metadata: &FunctionMetadata,
        function_name: &str,
    ) -> String {
        let mut code = String::new();

        code.push_str(&format!(
            "// Runtime assertions for function {}\n",
            function_name
        ));

        // Generate precondition checks
        if !metadata.preconditions.is_empty() {
            code.push_str("#[cfg(debug_assertions)]\n");
            code.push_str("{\n");
            for (i, precondition) in metadata.preconditions.iter().enumerate() {
                let condition_code = self.expression_to_code(&precondition.condition);
                let default_message = format!("Precondition {} violated", i + 1);
                let message = precondition.message.as_deref().unwrap_or(&default_message);

                match precondition.failure_action {
                    FailureAction::ThrowException => {
                        code.push_str(&format!(
                            "    if !({}) {{ panic!(\"Precondition violation in {}: {}\"); }}\n",
                            condition_code, function_name, message
                        ));
                    }
                    FailureAction::LogWarning => {
                        code.push_str(&format!("    if !({}) {{ eprintln!(\"Warning: Precondition violation in {}: {}\"); }}\n", 
                                              condition_code, function_name, message));
                    }
                    FailureAction::AssertFail => {
                        code.push_str(&format!(
                            "    assert!({}, \"Precondition violation in {}: {}\");\n",
                            condition_code, function_name, message
                        ));
                    }
                }
            }
            code.push_str("}\n");
        }

        code
    }

    /// Convert expression to code (simplified)
    fn expression_to_code(&self, expression: &Expression) -> String {
        match expression {
            Expression::BooleanLiteral { value, .. } => value.to_string(),
            Expression::IntegerLiteral { value, .. } => value.to_string(),
            Expression::Variable { name, .. } => name.name.clone(),
            Expression::Equals { left, right, .. } => {
                format!(
                    "({} == {})",
                    self.expression_to_code(left),
                    self.expression_to_code(right)
                )
            }
            Expression::LessThan { left, right, .. } => {
                format!(
                    "({} < {})",
                    self.expression_to_code(left),
                    self.expression_to_code(right)
                )
            }
            Expression::LogicalAnd { operands, .. } => {
                let conditions: Vec<String> = operands
                    .iter()
                    .map(|op| self.expression_to_code(op))
                    .collect();
                format!("({})", conditions.join(" && "))
            }
            // Add more expression types as needed
            _ => "true".to_string(), // Fallback for unsupported expressions
        }
    }

    /// Get validation statistics
    pub fn get_stats(&self) -> &ContractStats {
        &self.stats
    }

    /// Reset validation statistics
    pub fn reset_stats(&mut self) {
        self.stats = ContractStats::default();
    }
}

impl Default for ContractValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        ComplexityNotation, ComplexityType, Identifier, PerformanceMetric, PrimitiveType,
    };
    use crate::error::SourceLocation;

    #[test]
    fn test_contract_validator_creation() {
        let validator = ContractValidator::new();
        assert_eq!(validator.stats.functions_processed, 0);
        assert_eq!(validator.stats.contract_errors, 0);
    }

    #[test]
    fn test_performance_expectation_validation() {
        let validator = ContractValidator::new();
        let location = SourceLocation::new("test.aether".to_string(), 1, 1, 0);

        // Valid latency expectation
        let valid_perf = PerformanceExpectation {
            metric: PerformanceMetric::LatencyMs,
            target_value: 100.0,
            context: Some("API response time".to_string()),
        };
        assert!(validator
            .validate_performance_expectation(&valid_perf, "test_func", &location)
            .is_ok());

        // Invalid latency expectation (negative)
        let invalid_perf = PerformanceExpectation {
            metric: PerformanceMetric::LatencyMs,
            target_value: -10.0,
            context: None,
        };
        assert!(validator
            .validate_performance_expectation(&invalid_perf, "test_func", &location)
            .is_err());

        // Invalid latency expectation (too high)
        let too_high_perf = PerformanceExpectation {
            metric: PerformanceMetric::LatencyMs,
            target_value: 100000.0,
            context: None,
        };
        assert!(validator
            .validate_performance_expectation(&too_high_perf, "test_func", &location)
            .is_err());
    }

    #[test]
    fn test_complexity_expectation_validation() {
        let validator = ContractValidator::new();
        let location = SourceLocation::new("test.aether".to_string(), 1, 1, 0);

        // Valid complexity expectation
        let valid_complexity = ComplexityExpectation {
            complexity_type: ComplexityType::Time,
            notation: ComplexityNotation::BigO,
            value: "O(n log n)".to_string(),
        };
        assert!(validator
            .validate_complexity_expectation(&valid_complexity, "test_func", &location)
            .is_ok());

        // Invalid complexity expectation
        let invalid_complexity = ComplexityExpectation {
            complexity_type: ComplexityType::Time,
            notation: ComplexityNotation::BigO,
            value: "O(invalid)".to_string(),
        };
        assert!(validator
            .validate_complexity_expectation(&invalid_complexity, "test_func", &location)
            .is_err());
    }

    #[test]
    fn test_algorithm_hint_validation() {
        let validator = ContractValidator::new();

        // Valid algorithm hint
        assert!(validator
            .validate_algorithm_hint("binary_search", "test_func")
            .is_none());

        // Invalid algorithm hint
        assert!(validator
            .validate_algorithm_hint("unknown_algorithm", "test_func")
            .is_some());
    }

    #[test]
    fn test_expression_type_inference() {
        let validator = ContractValidator::new();
        let mut parameter_types = HashMap::new();
        parameter_types.insert("x".to_string(), Type::primitive(PrimitiveType::Integer));

        let context = ContractContext {
            parameter_types,
            return_type: Type::primitive(PrimitiveType::Boolean),
            type_checker: Rc::new(RefCell::new(TypeChecker::new())),
        };

        // Test boolean literal
        let bool_expr = Expression::BooleanLiteral {
            value: true,
            source_location: SourceLocation::unknown(),
        };
        let inferred_type = validator
            .infer_expression_type(&bool_expr, &context)
            .unwrap();
        assert!(matches!(
            inferred_type,
            Type::Primitive(PrimitiveType::Boolean)
        ));

        // Test variable reference
        let var_expr = Expression::Variable {
            name: Identifier::new("x".to_string(), SourceLocation::unknown()),
            source_location: SourceLocation::unknown(),
        };
        let inferred_type = validator
            .infer_expression_type(&var_expr, &context)
            .unwrap();
        assert!(matches!(
            inferred_type,
            Type::Primitive(PrimitiveType::Integer)
        ));
    }

    #[test]
    fn test_runtime_assertion_generation() {
        let validator = ContractValidator::new();

        let metadata = FunctionMetadata {
            preconditions: vec![ContractAssertion {
                condition: Box::new(Expression::BooleanLiteral {
                    value: true,
                    source_location: SourceLocation::unknown(),
                }),
                failure_action: FailureAction::AssertFail,
                message: Some("Test precondition".to_string()),
                source_location: SourceLocation::unknown(),
                runtime_check: false,
            }],
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: Vec::new(),
            thread_safe: None,
            may_block: None,
        };

        let code = validator.generate_runtime_assertions(&metadata, "test_function");
        assert!(code.contains("Runtime assertions for function test_function"));
        assert!(code.contains("assert!"));
        assert!(code.contains("Test precondition"));
    }
}
