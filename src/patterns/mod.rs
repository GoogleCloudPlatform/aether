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

//! Verified Pattern Library for LLM Code Generation
//! 
//! This module provides a comprehensive library of verified code patterns
//! that can be composed to generate correct programs.

pub mod catalog;
pub mod composition;

use crate::ast::{Function, Statement, Expression, Block};
use crate::verification::contracts::FunctionContract;
use crate::error::SourceLocation;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// A verified code pattern that can be instantiated
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Unique pattern identifier
    pub id: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Pattern category
    pub category: PatternCategory,
    
    /// Intent description for LLM understanding
    pub intent: String,
    
    /// Detailed description
    pub description: String,
    
    /// Pattern metadata
    pub metadata: PatternMetadata,
    
    /// Template parameters
    pub parameters: Vec<PatternParameter>,
    
    /// The pattern template
    pub template: PatternTemplate,
    
    /// Verification contract
    pub contract: FunctionContract,
    
    /// Composition rules
    pub composition_rules: Vec<CompositionRule>,
    
    /// Usage examples
    pub examples: Vec<PatternExample>,
    
    /// Performance characteristics
    pub performance: PerformanceProfile,
}

/// Pattern categories for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternCategory {
    /// Data structure operations
    DataStructures,
    /// Algorithm implementations
    Algorithms,
    /// I/O operations
    InputOutput,
    /// Resource management
    ResourceManagement,
    /// Error handling
    ErrorHandling,
    /// Concurrency patterns
    Concurrency,
    /// Network operations
    Networking,
    /// Database operations
    Database,
    /// Security patterns
    Security,
    /// Validation and parsing
    Validation,
    /// Mathematical operations
    Mathematics,
    /// String manipulation
    StringProcessing,
    /// File operations
    FileOperations,
    /// Memory management
    MemoryManagement,
    /// State machines
    StateMachines,
}

/// Pattern metadata for search and discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetadata {
    /// Tags for search
    pub tags: Vec<String>,
    
    /// Required features
    pub requires: Vec<String>,
    
    /// Provides capabilities
    pub provides: Vec<String>,
    
    /// Author information
    pub author: String,
    
    /// Version
    pub version: String,
    
    /// Stability level
    pub stability: StabilityLevel,
    
    /// Complexity estimate
    pub complexity: ComplexityEstimate,
    
    /// Safety guarantees
    pub safety: SafetyGuarantees,
}

/// Stability level of a pattern
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StabilityLevel {
    /// Experimental pattern
    Experimental,
    /// Stable and tested
    Stable,
    /// Deprecated, use alternatives
    Deprecated { alternative: String },
}

/// Complexity estimate for pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityEstimate {
    /// Time complexity
    pub time: String,
    
    /// Space complexity
    pub space: String,
    
    /// I/O complexity if applicable
    pub io: Option<String>,
}

/// Safety guarantees provided by pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyGuarantees {
    /// Memory safety
    pub memory_safe: bool,
    
    /// Thread safety
    pub thread_safe: bool,
    
    /// Exception safety
    pub exception_safe: ExceptionSafety,
    
    /// Resource cleanup guaranteed
    pub resource_safe: bool,
}

/// Exception safety level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExceptionSafety {
    /// No exceptions thrown
    NoThrow,
    /// Basic exception safety
    Basic,
    /// Strong exception safety
    Strong,
    /// No guarantee
    None,
}

/// Template parameter for pattern instantiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternParameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub param_type: ParameterType,
    
    /// Description for LLM
    pub description: String,
    
    /// Default value if any
    pub default: Option<ParameterValue>,
    
    /// Validation constraints
    pub constraints: Vec<ParameterConstraint>,
}

/// Types of pattern parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// Type name parameter
    TypeName,
    /// Expression parameter
    Expression,
    /// Statement parameter
    Statement,
    /// Block parameter
    Block,
    /// Identifier parameter
    Identifier,
    /// Integer constant
    IntegerConstant,
    /// String constant
    StringConstant,
    /// Boolean flag
    BooleanFlag,
    /// Choice from options
    Choice { options: Vec<String> },
}

/// Parameter value for instantiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Type(String),
    Expression(Box<Expression>),
    Statement(Box<Statement>),
    Block(Block),
    Identifier(String),
    Integer(i64),
    String(String),
    Boolean(bool),
    Choice(String),
}

/// Constraint on parameter values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterConstraint {
    /// Must match regex
    Regex(String),
    /// Must be in range
    Range { min: i64, max: i64 },
    /// Must have minimum length
    MinLength(usize),
    /// Must have maximum length
    MaxLength(usize),
    /// Custom validation expression
    Custom(String),
}

/// The actual pattern template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternTemplate {
    /// Function pattern
    Function(FunctionTemplate),
    /// Statement pattern
    Statement(StatementTemplate),
    /// Expression pattern
    Expression(ExpressionTemplate),
    /// Module pattern
    Module(ModuleTemplate),
}

/// Function template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTemplate {
    /// Function name template
    pub name_template: String,
    
    /// Parameter templates
    pub parameters: Vec<ParameterTemplate>,
    
    /// Return type template
    pub return_type_template: String,
    
    /// Body template
    pub body_template: String,
    
    /// Contract template
    pub contract_template: Option<String>,
}

/// Parameter template for functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterTemplate {
    pub name_template: String,
    pub type_template: String,
}

/// Statement template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementTemplate {
    /// The statement template in S-expression format
    pub template: String,
}

/// Expression template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionTemplate {
    /// The expression template in S-expression format
    pub template: String,
}

/// Module template for larger patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleTemplate {
    /// Module name template
    pub name_template: String,
    
    /// Import templates
    pub imports: Vec<String>,
    
    /// Type definition templates
    pub types: Vec<String>,
    
    /// Function templates
    pub functions: Vec<FunctionTemplate>,
}

/// Rules for composing patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionRule {
    /// Rule identifier
    pub id: String,
    
    /// When this rule applies
    pub condition: CompositionCondition,
    
    /// How to compose
    pub action: CompositionAction,
    
    /// Priority for conflict resolution
    pub priority: u32,
}

/// Conditions for pattern composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositionCondition {
    /// Can compose with specific pattern
    CompatibleWith { pattern_id: String },
    /// Requires specific capability
    RequiresCapability { capability: String },
    /// Excludes specific pattern
    ExcludesWith { pattern_id: String },
    /// Custom condition
    Custom { expression: String },
}

/// Actions for pattern composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositionAction {
    /// Merge patterns sequentially
    Sequential,
    /// Nest one pattern in another
    Nested { parent_param: String },
    /// Replace a parameter
    Replace { parameter: String },
    /// Wrap with additional logic
    Wrap { wrapper_pattern: String },
}

/// Example usage of a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExample {
    /// Example name
    pub name: String,
    
    /// Description
    pub description: String,
    
    /// Parameter values
    pub parameters: HashMap<String, ParameterValue>,
    
    /// Expected output preview
    pub preview: String,
    
    /// Verification status
    pub verified: bool,
}

/// Performance profile of a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceProfile {
    /// Typical execution time
    pub execution_time: ExecutionTime,
    
    /// Memory usage
    pub memory_usage: MemoryUsage,
    
    /// I/O characteristics
    pub io_profile: Option<IOProfile>,
    
    /// Scalability notes
    pub scalability: String,
}

/// Execution time characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTime {
    /// Best case in microseconds
    pub best_case_us: u64,
    
    /// Average case in microseconds
    pub average_case_us: u64,
    
    /// Worst case in microseconds
    pub worst_case_us: u64,
}

/// Memory usage characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    /// Stack usage in bytes
    pub stack_bytes: u64,
    
    /// Heap usage in bytes
    pub heap_bytes: u64,
    
    /// Whether it allocates
    pub allocates: bool,
}

/// I/O profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOProfile {
    /// Reads data
    pub reads: bool,
    
    /// Writes data
    pub writes: bool,
    
    /// Network I/O
    pub network: bool,
    
    /// File I/O
    pub file: bool,
}

/// Pattern library interface
pub struct PatternLibrary {
    /// All patterns indexed by ID
    patterns: HashMap<String, Pattern>,
    
    /// Patterns indexed by category
    by_category: HashMap<PatternCategory, Vec<String>>,
    
    /// Patterns indexed by capability
    by_capability: HashMap<String, Vec<String>>,
    
    /// Composition graph
    composition_graph: CompositionGraph,
}

/// Graph of pattern composition relationships
pub struct CompositionGraph {
    /// Adjacency list of compatible patterns
    compatible: HashMap<String, Vec<String>>,
    
    /// Exclusion relationships
    exclusions: HashMap<String, Vec<String>>,
}

impl PatternLibrary {
    /// Create a new pattern library
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            by_category: HashMap::new(),
            by_capability: HashMap::new(),
            composition_graph: CompositionGraph {
                compatible: HashMap::new(),
                exclusions: HashMap::new(),
            },
        }
    }
    
    /// Load standard patterns
    pub fn load_standard_patterns(&mut self) {
        // Load from catalog
        self.add_patterns(crate::patterns::catalog::load_all_patterns());
    }
    
    /// Add patterns to the library
    pub fn add_patterns(&mut self, patterns: Vec<Pattern>) {
        for pattern in patterns {
            self.add_pattern(pattern);
        }
    }
    
    /// Add a single pattern
    pub fn add_pattern(&mut self, pattern: Pattern) {
        // Index by category
        self.by_category
            .entry(pattern.category.clone())
            .or_insert_with(Vec::new)
            .push(pattern.id.clone());
        
        // Index by capabilities
        for capability in &pattern.metadata.provides {
            self.by_capability
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(pattern.id.clone());
        }
        
        // Update composition graph
        for rule in &pattern.composition_rules {
            match &rule.condition {
                CompositionCondition::CompatibleWith { pattern_id } => {
                    self.composition_graph.compatible
                        .entry(pattern.id.clone())
                        .or_insert_with(Vec::new)
                        .push(pattern_id.clone());
                }
                CompositionCondition::ExcludesWith { pattern_id } => {
                    self.composition_graph.exclusions
                        .entry(pattern.id.clone())
                        .or_insert_with(Vec::new)
                        .push(pattern_id.clone());
                }
                _ => {}
            }
        }
        
        // Store pattern
        self.patterns.insert(pattern.id.clone(), pattern);
    }
    
    /// Find patterns by category
    pub fn find_by_category(&self, category: &PatternCategory) -> Vec<&Pattern> {
        self.by_category
            .get(category)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.patterns.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Find patterns providing capability
    pub fn find_by_capability(&self, capability: &str) -> Vec<&Pattern> {
        self.by_capability
            .get(capability)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.patterns.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Find patterns matching intent
    pub fn find_by_intent(&self, intent: &str) -> Vec<&Pattern> {
        let intent_lower = intent.to_lowercase();
        self.patterns
            .values()
            .filter(|p| {
                p.intent.to_lowercase().contains(&intent_lower) ||
                p.description.to_lowercase().contains(&intent_lower) ||
                p.metadata.tags.iter().any(|t| t.to_lowercase().contains(&intent_lower))
            })
            .collect()
    }
    
    /// Check if two patterns can be composed
    pub fn can_compose(&self, pattern1: &str, pattern2: &str) -> bool {
        // Check exclusions first
        if let Some(exclusions) = self.composition_graph.exclusions.get(pattern1) {
            if exclusions.contains(&pattern2.to_string()) {
                return false;
            }
        }
        
        // Check compatibility
        if let Some(compatible) = self.composition_graph.compatible.get(pattern1) {
            return compatible.contains(&pattern2.to_string());
        }
        
        // Default to true if no explicit rules
        true
    }
    
    /// Get a pattern by ID
    pub fn get_pattern(&self, id: &str) -> Option<&Pattern> {
        self.patterns.get(id)
    }
    
    /// Get all patterns
    pub fn all_patterns(&self) -> Vec<&Pattern> {
        self.patterns.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_library_creation() {
        let mut library = PatternLibrary::new();
        assert!(library.patterns.is_empty());
        
        // Create a test pattern
        let pattern = Pattern {
            id: "test_pattern".to_string(),
            name: "Test Pattern".to_string(),
            category: PatternCategory::Algorithms,
            intent: "Test pattern for unit tests".to_string(),
            description: "A simple test pattern".to_string(),
            metadata: PatternMetadata {
                tags: vec!["test".to_string()],
                requires: vec![],
                provides: vec!["test_capability".to_string()],
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
            template: PatternTemplate::Expression(ExpressionTemplate {
                template: "(INTEGER_LITERAL 42)".to_string(),
            }),
            contract: crate::verification::contracts::FunctionContract {
                function_name: "test".to_string(),
                preconditions: vec![],
                postconditions: vec![],
                invariants: vec![],
                modifies: Default::default(),
                is_pure: true,
                decreases: None,
                intent: None,
                behavior: None,
                resources: None,
                failure_actions: Default::default(),
                propagation: Default::default(),
                proof_obligations: vec![],
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
                    stack_bytes: 0,
                    heap_bytes: 0,
                    allocates: false,
                },
                io_profile: None,
                scalability: "Constant".to_string(),
            },
        };
        
        library.add_pattern(pattern);
        assert_eq!(library.patterns.len(), 1);
        assert_eq!(library.find_by_category(&PatternCategory::Algorithms).len(), 1);
        assert_eq!(library.find_by_capability("test_capability").len(), 1);
    }
}