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

//! Intent Analysis for Error Detection
//! 
//! Analyzes code to detect mismatches between stated intent and actual behavior,
//! providing detailed feedback for LLM correction.

use crate::ast::{Function, Statement, Expression, Block};
use crate::semantic::metadata::{IntentSpec, BehavioralSpec};
use crate::error::structured::IntentMismatch;
use std::collections::HashMap;

/// Intent analyzer for detecting behavior mismatches
pub struct IntentAnalyzer {
    /// Pattern database for common intents
    patterns: IntentPatternDatabase,
    
    /// Behavior inference engine
    behavior_engine: BehaviorInferenceEngine,
    
    /// Analysis cache
    cache: HashMap<String, IntentAnalysisResult>,
}

/// Result of intent analysis
#[derive(Debug, Clone)]
pub struct IntentAnalysisResult {
    /// Function name
    pub function_name: String,
    
    /// Stated intent
    pub stated_intent: Option<IntentSpec>,
    
    /// Inferred behavior
    pub inferred_behavior: InferredBehavior,
    
    /// Mismatches found
    pub mismatches: Vec<IntentMismatch>,
    
    /// Confidence in analysis
    pub confidence: f32,
}

/// Inferred behavior from code analysis
#[derive(Debug, Clone)]
pub struct InferredBehavior {
    /// What the code actually does
    pub primary_action: String,
    
    /// Side effects detected
    pub side_effects: Vec<String>,
    
    /// Error handling approach
    pub error_handling: ErrorHandlingStyle,
    
    /// Resource usage
    pub resource_usage: Vec<String>,
    
    /// Algorithmic pattern detected
    pub algorithm_pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ErrorHandlingStyle {
    None,
    ReturnError,
    ThrowException,
    Defensive,
    Mixed,
}

impl IntentAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: IntentPatternDatabase::new(),
            behavior_engine: BehaviorInferenceEngine::new(),
            cache: HashMap::new(),
        }
    }
    
    /// Analyze a function for intent mismatches
    pub fn analyze_function(&mut self, function: &Function) -> IntentAnalysisResult {
        // Check cache
        if let Some(cached) = self.cache.get(&function.name.name) {
            return cached.clone();
        }
        
        // Extract stated intent
        let stated_intent = function.intent.as_ref().map(|intent_str| {
            crate::semantic::metadata::IntentSpec {
                primary_intent: intent_str.clone(),
                business_purpose: None,
                technical_approach: None,
                success_criteria: Vec::new(),
                failure_modes: Vec::new(),
            }
        });
        
        // Infer actual behavior
        let inferred_behavior = self.behavior_engine.infer_behavior(function);
        
        // Find mismatches
        let mismatches = self.find_mismatches(&stated_intent, &inferred_behavior, function);
        
        // Calculate confidence
        let confidence = self.calculate_confidence(&inferred_behavior, &mismatches);
        
        let result = IntentAnalysisResult {
            function_name: function.name.name.clone(),
            stated_intent,
            inferred_behavior,
            mismatches,
            confidence,
        };
        
        // Cache result
        self.cache.insert(function.name.name.clone(), result.clone());
        
        result
    }
    
    /// Find mismatches between stated intent and inferred behavior
    fn find_mismatches(
        &self,
        stated_intent: &Option<IntentSpec>,
        inferred: &InferredBehavior,
        function: &Function,
    ) -> Vec<IntentMismatch> {
        let mut mismatches = Vec::new();
        
        if let Some(intent) = stated_intent {
            // Check primary intent
            if let Some(mismatch) = self.check_primary_intent(&intent.primary_intent, &inferred.primary_action) {
                mismatches.push(mismatch);
            }
            
            // Check for unexpected side effects
            if false { // TODO: Check pure behavior when metadata available
                if !inferred.side_effects.is_empty() {
                    mismatches.push(IntentMismatch {
                        stated_intent: "Pure function (no side effects)".to_string(),
                        detected_behavior: format!("Has side effects: {:?}", inferred.side_effects),
                        confidence: 0.9,
                        evidence: inferred.side_effects.clone(),
                    });
                }
            }
            
            // Check error handling expectations
            if intent.failure_modes.is_empty() && matches!(inferred.error_handling, ErrorHandlingStyle::None) {
                mismatches.push(IntentMismatch {
                    stated_intent: "No failure modes specified".to_string(),
                    detected_behavior: "No error handling found".to_string(),
                    confidence: 0.7,
                    evidence: vec!["Function lacks error handling for edge cases".to_string()],
                });
            }
        }
        
        mismatches
    }
    
    /// Check if primary intent matches behavior
    fn check_primary_intent(&self, stated: &str, inferred: &str) -> Option<IntentMismatch> {
        // Use pattern matching to check semantic equivalence
        let patterns = self.patterns.get_patterns_for_intent(stated);
        
        for pattern in patterns {
            if pattern.matches(inferred) {
                return None; // Match found
            }
        }
        
        // No match found
        Some(IntentMismatch {
            stated_intent: stated.to_string(),
            detected_behavior: inferred.to_string(),
            confidence: 0.8,
            evidence: vec![
                format!("Function name suggests: {}", stated),
                format!("Code analysis shows: {}", inferred),
            ],
        })
    }
    
    /// Calculate confidence in analysis
    fn calculate_confidence(&self, behavior: &InferredBehavior, mismatches: &[IntentMismatch]) -> f32 {
        let base_confidence = 0.8;
        
        // Reduce confidence if behavior is complex
        let complexity_penalty = if behavior.side_effects.len() > 3 { 0.1 } else { 0.0 };
        
        // Reduce confidence if many mismatches
        let mismatch_penalty = (mismatches.len() as f32) * 0.05;
        
        (base_confidence - complexity_penalty - mismatch_penalty).max(0.1)
    }
}

/// Database of intent patterns
struct IntentPatternDatabase {
    patterns: HashMap<String, Vec<IntentPattern>>,
}

struct IntentPattern {
    intent_keywords: Vec<String>,
    behavior_patterns: Vec<String>,
}

impl IntentPattern {
    fn matches(&self, behavior: &str) -> bool {
        let behavior_lower = behavior.to_lowercase();
        self.behavior_patterns.iter()
            .any(|pattern| behavior_lower.contains(pattern))
    }
}

impl IntentPatternDatabase {
    fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Division patterns
        patterns.insert("division".to_string(), vec![
            IntentPattern {
                intent_keywords: vec!["divide".to_string(), "division".to_string()],
                behavior_patterns: vec!["divides".to_string(), "division".to_string(), "quotient".to_string()],
            }
        ]);
        
        // Validation patterns
        patterns.insert("validation".to_string(), vec![
            IntentPattern {
                intent_keywords: vec!["validate".to_string(), "check".to_string(), "verify".to_string()],
                behavior_patterns: vec!["checks".to_string(), "validates".to_string(), "verifies".to_string()],
            }
        ]);
        
        // Calculation patterns
        patterns.insert("calculation".to_string(), vec![
            IntentPattern {
                intent_keywords: vec!["calculate".to_string(), "compute".to_string()],
                behavior_patterns: vec!["calculates".to_string(), "computes".to_string(), "returns result".to_string()],
            }
        ]);
        
        Self { patterns }
    }
    
    fn get_patterns_for_intent(&self, intent: &str) -> Vec<&IntentPattern> {
        let intent_lower = intent.to_lowercase();
        
        self.patterns.iter()
            .filter(|(key, _)| intent_lower.contains(key.as_str()))
            .flat_map(|(_, patterns)| patterns.iter())
            .collect()
    }
}

/// Engine for inferring behavior from code
struct BehaviorInferenceEngine {
    /// Statement analyzers
    analyzers: Vec<Box<dyn StatementAnalyzer>>,
}

trait StatementAnalyzer {
    fn analyze_statement(&self, stmt: &Statement, context: &mut BehaviorContext);
}

struct BehaviorContext {
    actions: Vec<String>,
    side_effects: Vec<String>,
    resource_usage: Vec<String>,
    has_error_handling: bool,
    has_loops: bool,
    has_conditionals: bool,
}

impl BehaviorInferenceEngine {
    fn new() -> Self {
        let analyzers: Vec<Box<dyn StatementAnalyzer>> = vec![
            Box::new(AssignmentAnalyzer),
            Box::new(FunctionCallAnalyzer),
            Box::new(ControlFlowAnalyzer),
            Box::new(ErrorHandlingAnalyzer),
        ];
        
        Self { analyzers }
    }
    
    /// Infer behavior from function
    fn infer_behavior(&self, function: &Function) -> InferredBehavior {
        let mut context = BehaviorContext {
            actions: Vec::new(),
            side_effects: Vec::new(),
            resource_usage: Vec::new(),
            has_error_handling: false,
            has_loops: false,
            has_conditionals: false,
        };
        
        // Analyze function body
        self.analyze_block(&function.body, &mut context);
        
        // Determine primary action
        let primary_action = self.determine_primary_action(&context, &function.name.name);
        
        // Determine error handling style
        let error_handling = if context.has_error_handling {
            ErrorHandlingStyle::Defensive
        } else {
            ErrorHandlingStyle::None
        };
        
        // Detect algorithm pattern
        let algorithm_pattern = self.detect_algorithm_pattern(&context);
        
        InferredBehavior {
            primary_action,
            side_effects: context.side_effects,
            error_handling,
            resource_usage: context.resource_usage,
            algorithm_pattern,
        }
    }
    
    /// Analyze a block of statements
    fn analyze_block(&self, block: &Block, context: &mut BehaviorContext) {
        for stmt in &block.statements {
            for analyzer in &self.analyzers {
                analyzer.analyze_statement(stmt, context);
            }
        }
    }
    
    /// Determine the primary action of the function
    fn determine_primary_action(&self, context: &BehaviorContext, function_name: &str) -> String {
        // Look for return statements
        if let Some(action) = context.actions.iter()
            .find(|a| a.contains("returns") || a.contains("calculates")) 
        {
            return action.clone();
        }
        
        // Look for main operations
        if !context.actions.is_empty() {
            return context.actions[0].clone();
        }
        
        // Fall back to function name analysis
        if function_name.starts_with("get_") {
            "Returns a value".to_string()
        } else if function_name.starts_with("set_") {
            "Sets a value".to_string()
        } else if function_name.starts_with("is_") || function_name.starts_with("has_") {
            "Checks a condition".to_string()
        } else {
            "Performs operations".to_string()
        }
    }
    
    /// Detect common algorithm patterns
    fn detect_algorithm_pattern(&self, context: &BehaviorContext) -> Option<String> {
        if context.has_loops && context.actions.iter().any(|a| a.contains("accumulate")) {
            Some("reduction".to_string())
        } else if context.has_loops && context.actions.iter().any(|a| a.contains("filter")) {
            Some("filtering".to_string())
        } else if context.has_conditionals && !context.has_loops {
            Some("decision".to_string())
        } else if context.actions.iter().any(|a| a.contains("divide")) {
            Some("arithmetic".to_string())
        } else {
            None
        }
    }
}

// Statement analyzer implementations

struct AssignmentAnalyzer;
impl StatementAnalyzer for AssignmentAnalyzer {
    fn analyze_statement(&self, stmt: &Statement, context: &mut BehaviorContext) {
        if let Statement::Assignment { target, value, .. } = stmt {
            context.actions.push(format!("Assigns to {:?}", target));
            
            // Check if assignment has side effects
            if has_side_effects(value) {
                context.side_effects.push("Variable modification".to_string());
            }
        }
    }
}

struct FunctionCallAnalyzer;
impl StatementAnalyzer for FunctionCallAnalyzer {
    fn analyze_statement(&self, stmt: &Statement, context: &mut BehaviorContext) {
        if let Statement::FunctionCall { call, .. } = stmt {
            let func_name = match &call.function_reference {
                crate::ast::FunctionReference::Local { name } => &name.name,
                _ => "external_function",
            };
            
            context.actions.push(format!("Calls function '{}'", func_name));
            
            // Check for known side-effect functions
            if is_side_effect_function(func_name) {
                context.side_effects.push(format!("Calls {} (has side effects)", func_name));
            }
            
            // Check for resource usage
            if is_resource_function(func_name) {
                context.resource_usage.push(format!("Uses {} resources", func_name));
            }
        }
    }
}

struct ControlFlowAnalyzer;
impl StatementAnalyzer for ControlFlowAnalyzer {
    fn analyze_statement(&self, stmt: &Statement, context: &mut BehaviorContext) {
        match stmt {
            Statement::If { .. } => {
                context.has_conditionals = true;
                context.actions.push("Makes conditional decisions".to_string());
            }
            Statement::WhileLoop { .. } | Statement::ForEachLoop { .. } => {
                context.has_loops = true;
                context.actions.push("Iterates over data".to_string());
            }
            Statement::Return { value, .. } => {
                if value.is_some() {
                    context.actions.push("Returns calculated value".to_string());
                }
            }
            _ => {}
        }
    }
}

struct ErrorHandlingAnalyzer;
impl StatementAnalyzer for ErrorHandlingAnalyzer {
    fn analyze_statement(&self, stmt: &Statement, context: &mut BehaviorContext) {
        match stmt {
            Statement::TryBlock { .. } => {
                context.has_error_handling = true;
                context.actions.push("Handles errors with try-catch".to_string());
            }
            Statement::Throw { .. } => {
                context.has_error_handling = true;
                context.actions.push("Throws exceptions".to_string());
            }
            _ => {}
        }
    }
}

// Helper functions

fn has_side_effects(expr: &Expression) -> bool {
    match expr {
        Expression::FunctionCall { .. } => true,
        Expression::ArrayAccess { .. } => false,
        _ => false,
    }
}

fn is_side_effect_function(name: &str) -> bool {
    matches!(name, "printf" | "write" | "append" | "delete" | "update" | "send")
}

fn is_resource_function(name: &str) -> bool {
    matches!(name, "open" | "connect" | "allocate" | "lock" | "acquire")
}

/// Generate intent mismatch report
pub fn generate_intent_report(result: &IntentAnalysisResult) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("=== Intent Analysis for {} ===\n", result.function_name));
    report.push_str(&format!("Confidence: {:.1}%\n\n", result.confidence * 100.0));
    
    if let Some(intent) = &result.stated_intent {
        report.push_str(&format!("Stated Intent: {}\n", intent.primary_intent));
    }
    
    report.push_str(&format!("Detected Behavior: {}\n", result.inferred_behavior.primary_action));
    
    if !result.inferred_behavior.side_effects.is_empty() {
        report.push_str(&format!("Side Effects: {:?}\n", result.inferred_behavior.side_effects));
    }
    
    if !result.mismatches.is_empty() {
        report.push_str("\n=== Mismatches Found ===\n");
        for (i, mismatch) in result.mismatches.iter().enumerate() {
            report.push_str(&format!("\n{}. Intent: {}\n", i + 1, mismatch.stated_intent));
            report.push_str(&format!("   Behavior: {}\n", mismatch.detected_behavior));
            report.push_str(&format!("   Confidence: {:.1}%\n", mismatch.confidence * 100.0));
            report.push_str("   Evidence:\n");
            for evidence in &mismatch.evidence {
                report.push_str(&format!("     - {}\n", evidence));
            }
        }
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Identifier, Block, PrimitiveType, TypeSpecifier, FunctionCall, FunctionReference};
    use crate::error::SourceLocation;
    
    #[test]
    fn test_intent_analysis() {
        let mut analyzer = IntentAnalyzer::new();
        
        // Create a simple function that says it divides but actually adds
        let function = Function {
            name: Identifier::new("safe_divide".to_string(), SourceLocation::unknown()),
            metadata: crate::ast::FunctionMetadata {
                preconditions: vec![],
                postconditions: vec![],
                invariants: vec![],
                algorithm_hint: Some("safe division".to_string()),
                performance_expectation: None,
                complexity_expectation: None,
                throws_exceptions: vec![],
                thread_safe: Some(true),
                may_block: Some(false),
            },
            parameters: vec![],
            return_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Float,
                source_location: SourceLocation::unknown(),
            }),
            body: Block {
                statements: vec![
                    // This would actually contain addition instead of division
                    Statement::Return {
                        value: Some(Box::new(Expression::Add {
                            left: Box::new(Expression::Variable {
                                name: Identifier::new("a".to_string(), SourceLocation::unknown()),
                                source_location: SourceLocation::unknown(),
                            }),
                            right: Box::new(Expression::Variable {
                                name: Identifier::new("b".to_string(), SourceLocation::unknown()),
                                source_location: SourceLocation::unknown(),
                            }),
                            source_location: SourceLocation::unknown(),
                        })),
                        source_location: SourceLocation::unknown(),
                    }
                ],
                source_location: SourceLocation::unknown(),
            },
            source_location: SourceLocation::unknown(),
            intent: Some("Perform safe division".to_string()),
            generic_parameters: vec![],
            export_info: None,
        };
        
        let result = analyzer.analyze_function(&function);
        
        // Should detect mismatch between "division" intent and actual behavior
        assert!(!result.mismatches.is_empty());
        assert!(result.confidence > 0.0);
    }
}