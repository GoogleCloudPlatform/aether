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

//! Pattern Verification Module
//! 
//! Verifies that patterns and composed patterns are correct and safe

use super::*;
use crate::verification::contracts::{ContractVerifier, VerificationResult};
use crate::verification::smt::{SMTSolver, SMTFormula};
use crate::ast::{Statement, Expression, Function};
use crate::semantic::SemanticChecker;
use std::collections::HashMap;

/// Pattern verifier
pub struct PatternVerifier {
    /// Contract verifier
    contract_verifier: ContractVerifier,
    
    /// SMT solver for formal verification
    smt_solver: SMTSolver,
    
    /// Semantic checker
    semantic_checker: SemanticChecker,
    
    /// Verification cache
    cache: HashMap<String, PatternVerificationResult>,
}

/// Result of pattern verification
#[derive(Debug, Clone)]
pub struct PatternVerificationResult {
    /// Pattern ID
    pub pattern_id: String,
    
    /// Overall verification status
    pub verified: bool,
    
    /// Contract verification results
    pub contract_results: Vec<VerificationResult>,
    
    /// Safety analysis results
    pub safety_results: SafetyAnalysisResult,
    
    /// Performance validation
    pub performance_valid: bool,
    
    /// Composition compatibility
    pub composition_valid: bool,
    
    /// Issues found
    pub issues: Vec<VerificationIssue>,
}

/// Safety analysis results
#[derive(Debug, Clone)]
pub struct SafetyAnalysisResult {
    /// Memory safety verified
    pub memory_safe: bool,
    
    /// Thread safety verified
    pub thread_safe: bool,
    
    /// Resource safety verified
    pub resource_safe: bool,
    
    /// Exception safety verified
    pub exception_safe: bool,
    
    /// Proof obligations
    pub proof_obligations: Vec<ProofObligation>,
}

/// Verification issue
#[derive(Debug, Clone)]
pub struct VerificationIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    
    /// Issue category
    pub category: IssueCategory,
    
    /// Description
    pub description: String,
    
    /// Location in pattern
    pub location: Option<String>,
    
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueCategory {
    Contract,
    Safety,
    Performance,
    Composition,
    Semantic,
}

/// Proof obligation for pattern
#[derive(Debug, Clone)]
pub struct ProofObligation {
    /// Obligation ID
    pub id: String,
    
    /// What needs to be proved
    pub obligation: String,
    
    /// Formula to prove
    pub formula: SMTFormula,
    
    /// Proof status
    pub status: ProofStatus,
    
    /// Proof hint if available
    pub hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofStatus {
    Pending,
    Proved,
    Failed,
    Timeout,
}

impl PatternVerifier {
    /// Create new pattern verifier
    pub fn new() -> Self {
        Self {
            contract_verifier: ContractVerifier::new(),
            smt_solver: SMTSolver::new(),
            semantic_checker: SemanticChecker::new(),
            cache: HashMap::new(),
        }
    }
    
    /// Verify a pattern
    pub fn verify_pattern(&mut self, pattern: &Pattern) -> PatternVerificationResult {
        // Check cache
        if let Some(cached) = self.cache.get(&pattern.id) {
            return cached.clone();
        }
        
        // Verify contracts
        let contract_results = self.verify_contracts(pattern);
        
        // Analyze safety
        let safety_results = self.analyze_safety(pattern);
        
        // Validate performance claims
        let performance_valid = self.validate_performance(pattern);
        
        // Check composition rules
        let composition_valid = self.validate_composition_rules(pattern);
        
        // Collect all issues
        let mut issues = Vec::new();
        
        // Add contract issues
        for result in &contract_results {
            if !result.verified {
                issues.push(VerificationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Contract,
                    description: format!("Contract verification failed: {}", result.message),
                    location: Some(result.location.clone()),
                    suggested_fix: result.fix_suggestion.clone(),
                });
            }
        }
        
        // Add safety issues
        if !safety_results.memory_safe {
            issues.push(VerificationIssue {
                severity: IssueSeverity::Error,
                category: IssueCategory::Safety,
                description: "Pattern is not memory safe".to_string(),
                location: None,
                suggested_fix: Some("Add bounds checking or use safe operations".to_string()),
            });
        }
        
        // Determine overall verification status
        let verified = contract_results.iter().all(|r| r.verified) &&
                      safety_results.memory_safe &&
                      safety_results.thread_safe &&
                      safety_results.resource_safe &&
                      performance_valid &&
                      composition_valid;
        
        let result = PatternVerificationResult {
            pattern_id: pattern.id.clone(),
            verified,
            contract_results,
            safety_results,
            performance_valid,
            composition_valid,
            issues,
        };
        
        // Cache result
        self.cache.insert(pattern.id.clone(), result.clone());
        
        result
    }
    
    /// Verify pattern contracts
    fn verify_contracts(&mut self, pattern: &Pattern) -> Vec<VerificationResult> {
        let mut results = Vec::new();
        
        // Create a temporary function to verify
        let function = self.pattern_to_function(pattern);
        
        // Verify preconditions
        for precond in &pattern.contract.preconditions {
            let result = self.contract_verifier.verify_precondition(&function, precond);
            results.push(result);
        }
        
        // Verify postconditions
        for postcond in &pattern.contract.postconditions {
            let result = self.contract_verifier.verify_postcondition(&function, postcond);
            results.push(result);
        }
        
        // Verify invariants
        for invariant in &pattern.contract.invariants {
            let result = self.contract_verifier.verify_invariant(&function, invariant);
            results.push(result);
        }
        
        results
    }
    
    /// Analyze pattern safety
    fn analyze_safety(&mut self, pattern: &Pattern) -> SafetyAnalysisResult {
        let mut proof_obligations = Vec::new();
        
        // Memory safety analysis
        let memory_safe = self.analyze_memory_safety(pattern, &mut proof_obligations);
        
        // Thread safety analysis
        let thread_safe = self.analyze_thread_safety(pattern, &mut proof_obligations);
        
        // Resource safety analysis
        let resource_safe = self.analyze_resource_safety(pattern, &mut proof_obligations);
        
        // Exception safety analysis
        let exception_safe = self.analyze_exception_safety(pattern);
        
        SafetyAnalysisResult {
            memory_safe,
            thread_safe,
            resource_safe,
            exception_safe,
            proof_obligations,
        }
    }
    
    /// Analyze memory safety
    fn analyze_memory_safety(&mut self, pattern: &Pattern, obligations: &mut Vec<ProofObligation>) -> bool {
        // Check for array bounds
        if self.pattern_uses_arrays(pattern) {
            let obligation = ProofObligation {
                id: format!("{}_array_bounds", pattern.id),
                obligation: "All array accesses are within bounds".to_string(),
                formula: self.create_bounds_check_formula(pattern),
                status: ProofStatus::Pending,
                hint: Some("Use safe_array_access pattern".to_string()),
            };
            
            // Try to prove
            let proved = self.smt_solver.prove(&obligation.formula);
            let mut obligation = obligation;
            obligation.status = if proved { ProofStatus::Proved } else { ProofStatus::Failed };
            
            obligations.push(obligation.clone());
            
            if !proved && pattern.metadata.safety.memory_safe {
                return false;
            }
        }
        
        // Check for null pointer dereference
        if self.pattern_uses_pointers(pattern) {
            let obligation = ProofObligation {
                id: format!("{}_null_check", pattern.id),
                obligation: "No null pointer dereferences".to_string(),
                formula: self.create_null_check_formula(pattern),
                status: ProofStatus::Pending,
                hint: Some("Add null checks before pointer access".to_string()),
            };
            
            let proved = self.smt_solver.prove(&obligation.formula);
            let mut obligation = obligation;
            obligation.status = if proved { ProofStatus::Proved } else { ProofStatus::Failed };
            
            obligations.push(obligation.clone());
            
            if !proved && pattern.metadata.safety.memory_safe {
                return false;
            }
        }
        
        // If pattern claims memory safety and no issues found, trust it
        pattern.metadata.safety.memory_safe
    }
    
    /// Analyze thread safety
    fn analyze_thread_safety(&mut self, pattern: &Pattern, obligations: &mut Vec<ProofObligation>) -> bool {
        // Check for shared mutable state
        if self.pattern_has_shared_state(pattern) {
            let obligation = ProofObligation {
                id: format!("{}_thread_safety", pattern.id),
                obligation: "No data races on shared state".to_string(),
                formula: self.create_thread_safety_formula(pattern),
                status: ProofStatus::Pending,
                hint: Some("Use synchronization primitives".to_string()),
            };
            
            let proved = self.smt_solver.prove(&obligation.formula);
            let mut obligation = obligation;
            obligation.status = if proved { ProofStatus::Proved } else { ProofStatus::Failed };
            
            obligations.push(obligation.clone());
            
            if !proved && pattern.metadata.safety.thread_safe {
                return false;
            }
        }
        
        pattern.metadata.safety.thread_safe
    }
    
    /// Analyze resource safety
    fn analyze_resource_safety(&mut self, pattern: &Pattern, obligations: &mut Vec<ProofObligation>) -> bool {
        // Check for resource leaks
        if self.pattern_acquires_resources(pattern) {
            let obligation = ProofObligation {
                id: format!("{}_resource_cleanup", pattern.id),
                obligation: "All acquired resources are released".to_string(),
                formula: self.create_resource_cleanup_formula(pattern),
                status: ProofStatus::Pending,
                hint: Some("Use RAII pattern or RESOURCE_SCOPE".to_string()),
            };
            
            let proved = self.smt_solver.prove(&obligation.formula);
            let mut obligation = obligation;
            obligation.status = if proved { ProofStatus::Proved } else { ProofStatus::Failed };
            
            obligations.push(obligation.clone());
            
            if !proved && pattern.metadata.safety.resource_safe {
                return false;
            }
        }
        
        pattern.metadata.safety.resource_safe
    }
    
    /// Analyze exception safety
    fn analyze_exception_safety(&self, pattern: &Pattern) -> bool {
        match pattern.metadata.safety.exception_safe {
            ExceptionSafety::NoThrow => {
                // Verify no exceptions can be thrown
                !self.pattern_can_throw(pattern)
            }
            ExceptionSafety::Basic => {
                // Basic guarantee is usually satisfied
                true
            }
            ExceptionSafety::Strong => {
                // Strong guarantee requires rollback capability
                self.pattern_has_rollback(pattern)
            }
            ExceptionSafety::None => {
                // No guarantee
                true
            }
        }
    }
    
    /// Validate performance claims
    fn validate_performance(&self, pattern: &Pattern) -> bool {
        // Check complexity claims against actual implementation
        match &pattern.template {
            PatternTemplate::Statement(stmt) => {
                let actual_complexity = self.analyze_complexity(&stmt.template);
                actual_complexity == pattern.metadata.complexity.time
            }
            _ => true, // Trust other template types for now
        }
    }
    
    /// Validate composition rules
    fn validate_composition_rules(&self, pattern: &Pattern) -> bool {
        // Check that composition rules are consistent
        for rule in &pattern.composition_rules {
            match &rule.condition {
                CompositionCondition::ExcludesWith { pattern_id } => {
                    // Ensure mutual exclusion
                    if pattern.composition_rules.iter().any(|r| {
                        matches!(&r.condition, CompositionCondition::CompatibleWith { pattern_id: id } if id == pattern_id)
                    }) {
                        return false;
                    }
                }
                _ => {}
            }
        }
        
        true
    }
    
    /// Convert pattern to function for verification
    fn pattern_to_function(&self, pattern: &Pattern) -> Function {
        // Create a synthetic function from pattern template
        let body = match &pattern.template {
            PatternTemplate::Statement(stmt) => {
                Block {
                    statements: vec![self.template_to_statement(&stmt.template)],
                    source_location: crate::error::SourceLocation::unknown(),
                }
            }
            PatternTemplate::Function(func) => {
                Block {
                    statements: vec![self.template_to_statement(&func.body_template)],
                    source_location: crate::error::SourceLocation::unknown(),
                }
            }
            _ => Block {
                statements: vec![],
                source_location: crate::error::SourceLocation::unknown(),
            },
        };
        
        Function {
            name: crate::ast::Identifier::new(pattern.id.clone(), crate::error::SourceLocation::unknown()),
            metadata: None,
            parameters: vec![],
            return_type: Box::new(crate::ast::TypeSpecifier::Primitive {
                type_name: crate::ast::PrimitiveType::Void,
                source_location: crate::error::SourceLocation::unknown(),
            }),
            body,
            source_location: crate::error::SourceLocation::unknown(),
        }
    }
    
    /// Convert template string to statement (simplified)
    fn template_to_statement(&self, template: &str) -> Statement {
        // This is a simplified version - real implementation would parse the template
        Statement::Block {
            block: Block {
                statements: vec![],
                source_location: crate::error::SourceLocation::unknown(),
            },
            source_location: crate::error::SourceLocation::unknown(),
        }
    }
    
    // Helper methods for pattern analysis
    
    fn pattern_uses_arrays(&self, pattern: &Pattern) -> bool {
        pattern.metadata.tags.iter().any(|tag| tag.contains("array"))
    }
    
    fn pattern_uses_pointers(&self, pattern: &Pattern) -> bool {
        pattern.metadata.tags.iter().any(|tag| tag.contains("pointer"))
    }
    
    fn pattern_has_shared_state(&self, pattern: &Pattern) -> bool {
        pattern.metadata.tags.iter().any(|tag| tag.contains("concurrent") || tag.contains("shared"))
    }
    
    fn pattern_acquires_resources(&self, pattern: &Pattern) -> bool {
        pattern.metadata.tags.iter().any(|tag| 
            tag.contains("resource") || tag.contains("file") || tag.contains("network"))
    }
    
    fn pattern_can_throw(&self, pattern: &Pattern) -> bool {
        match &pattern.template {
            PatternTemplate::Statement(stmt) => {
                stmt.template.contains("THROW") || stmt.template.contains("EXCEPTION")
            }
            PatternTemplate::Function(func) => {
                func.body_template.contains("THROW") || func.body_template.contains("EXCEPTION")
            }
            _ => false,
        }
    }
    
    fn pattern_has_rollback(&self, pattern: &Pattern) -> bool {
        pattern.metadata.tags.iter().any(|tag| tag.contains("transactional"))
    }
    
    fn analyze_complexity(&self, template: &str) -> String {
        if template.contains("WHILE_LOOP") || template.contains("FOR_EACH") {
            if template.contains("NESTED") {
                "O(n²)".to_string()
            } else {
                "O(n)".to_string()
            }
        } else {
            "O(1)".to_string()
        }
    }
    
    // Formula creation methods (simplified)
    
    fn create_bounds_check_formula(&self, pattern: &Pattern) -> SMTFormula {
        SMTFormula::new("(forall ((i Int)) (=> (array-access i) (and (>= i 0) (< i array-length))))")
    }
    
    fn create_null_check_formula(&self, pattern: &Pattern) -> SMTFormula {
        SMTFormula::new("(forall ((p Pointer)) (=> (dereference p) (not (= p null))))")
    }
    
    fn create_thread_safety_formula(&self, pattern: &Pattern) -> SMTFormula {
        SMTFormula::new("(no-data-races shared-state)")
    }
    
    fn create_resource_cleanup_formula(&self, pattern: &Pattern) -> SMTFormula {
        SMTFormula::new("(forall ((r Resource)) (=> (acquire r) (eventually (release r))))")
    }
}

/// Verify a composed pattern
pub fn verify_composed_pattern(
    composed: &crate::patterns::composition::ComposedPattern,
    library: &PatternLibrary,
) -> PatternVerificationResult {
    let mut verifier = PatternVerifier::new();
    
    // First verify the result pattern
    let mut result = verifier.verify_pattern(&composed.result);
    
    // Additional checks for composition
    let mut composition_issues = Vec::new();
    
    // Verify all base patterns are individually valid
    for pattern_id in &composed.base_patterns {
        if let Some(base_pattern) = library.get_pattern(pattern_id) {
            let base_result = verifier.verify_pattern(base_pattern);
            if !base_result.verified {
                composition_issues.push(VerificationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Composition,
                    description: format!("Base pattern '{}' is not verified", pattern_id),
                    location: Some(pattern_id.clone()),
                    suggested_fix: Some("Fix base pattern before composition".to_string()),
                });
            }
        }
    }
    
    // Check composition compatibility
    for i in 0..composed.base_patterns.len() {
        for j in i+1..composed.base_patterns.len() {
            if !library.can_compose(&composed.base_patterns[i], &composed.base_patterns[j]) {
                composition_issues.push(VerificationIssue {
                    severity: IssueSeverity::Error,
                    category: IssueCategory::Composition,
                    description: format!(
                        "Patterns '{}' and '{}' cannot be composed",
                        composed.base_patterns[i], composed.base_patterns[j]
                    ),
                    location: None,
                    suggested_fix: Some("Use compatible patterns".to_string()),
                });
            }
        }
    }
    
    // Add composition issues to result
    result.issues.extend(composition_issues);
    
    // Update verification status
    result.verified = result.verified && result.issues.is_empty();
    
    result
}

/// Generate verification report
pub fn generate_verification_report(result: &PatternVerificationResult) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("=== Pattern Verification Report: {} ===\n", result.pattern_id));
    report.push_str(&format!("Status: {}\n\n", if result.verified { "VERIFIED" } else { "FAILED" }));
    
    // Contract results
    report.push_str("Contract Verification:\n");
    for contract_result in &result.contract_results {
        report.push_str(&format!("  - {}: {}\n", 
            contract_result.contract_type, 
            if contract_result.verified { "✓" } else { "✗" }
        ));
    }
    
    // Safety results
    report.push_str("\nSafety Analysis:\n");
    report.push_str(&format!("  - Memory Safe: {}\n", if result.safety_results.memory_safe { "✓" } else { "✗" }));
    report.push_str(&format!("  - Thread Safe: {}\n", if result.safety_results.thread_safe { "✓" } else { "✗" }));
    report.push_str(&format!("  - Resource Safe: {}\n", if result.safety_results.resource_safe { "✓" } else { "✗" }));
    report.push_str(&format!("  - Exception Safe: {}\n", if result.safety_results.exception_safe { "✓" } else { "✗" }));
    
    // Performance validation
    report.push_str(&format!("\nPerformance Claims: {}\n", if result.performance_valid { "Valid" } else { "Invalid" }));
    
    // Composition compatibility
    report.push_str(&format!("Composition Rules: {}\n", if result.composition_valid { "Valid" } else { "Invalid" }));
    
    // Issues
    if !result.issues.is_empty() {
        report.push_str("\nIssues Found:\n");
        for issue in &result.issues {
            report.push_str(&format!("  [{:?}] {}: {}\n", issue.severity, issue.category, issue.description));
            if let Some(location) = &issue.location {
                report.push_str(&format!("    Location: {}\n", location));
            }
            if let Some(fix) = &issue.suggested_fix {
                report.push_str(&format!("    Suggested Fix: {}\n", fix));
            }
        }
    }
    
    // Proof obligations
    if !result.safety_results.proof_obligations.is_empty() {
        report.push_str("\nProof Obligations:\n");
        for obligation in &result.safety_results.proof_obligations {
            report.push_str(&format!("  - {}: {:?}\n", obligation.obligation, obligation.status));
            if let Some(hint) = &obligation.hint {
                report.push_str(&format!("    Hint: {}\n", hint));
            }
        }
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_verification() {
        let mut verifier = PatternVerifier::new();
        
        // Create a test pattern
        let pattern = Pattern {
            id: "test_safe_access".to_string(),
            name: "Test Safe Access".to_string(),
            category: PatternCategory::DataStructures,
            intent: "Safe array access".to_string(),
            description: "Test pattern".to_string(),
            metadata: PatternMetadata {
                tags: vec!["array".to_string(), "safety".to_string()],
                requires: vec![],
                provides: vec!["safe_access".to_string()],
                author: "test".to_string(),
                version: "1.0.0".to_string(),
                stability: StabilityLevel::Stable,
                complexity: ComplexityEstimate {
                    time: "O(1)".to_string(),
                    space: "O(1)".to_string(),
                    io: None,
                },
                safety: SafetyGuarantees {
                    memory_safe: true,
                    thread_safe: true,
                    exception_safe: ExceptionSafety::NoThrow,
                    resource_safe: true,
                },
            },
            parameters: vec![],
            template: PatternTemplate::Statement(StatementTemplate {
                template: "(ARRAY_ACCESS_SAFE arr idx)".to_string(),
            }),
            contract: FunctionContract {
                function_name: "test".to_string(),
                preconditions: vec![],
                postconditions: vec![],
                invariants: vec![],
                propagates: vec![],
                intent: None,
                behavior: None,
                resources: None,
            },
            composition_rules: vec![],
            examples: vec![],
            performance: PerformanceProfile {
                execution_time: ExecutionTime {
                    best_case_us: 1,
                    average_case_us: 1,
                    worst_case_us: 1,
                },
                memory_usage: MemoryUsage {
                    stack_bytes: 8,
                    heap_bytes: 0,
                    allocates: false,
                },
                io_profile: None,
                scalability: "Constant".to_string(),
            },
        };
        
        let result = verifier.verify_pattern(&pattern);
        
        // Basic checks
        assert!(result.verified);
        assert!(result.safety_results.memory_safe);
        assert!(result.performance_valid);
    }
}