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

//! Enhanced Metadata System for LLM-First Language
//! 
//! This module implements the rich metadata system that makes AetherScript
//! optimal for LLM code generation and verification.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Semantic type definition beyond primitive types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticType {
    /// Name of the semantic type (e.g., "email_address")
    pub name: String,
    /// Base primitive type
    pub base_type: String,
    /// Regex or other constraint
    pub constraint: Option<String>,
    /// Semantic category for LLM understanding
    pub semantic_category: String,
    /// Privacy level for data handling
    pub privacy_level: Option<PrivacyLevel>,
    /// Additional semantic hints
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Public,
    Internal,
    Confidential,
    PersonallyIdentifiable,
    HighlySensitive,
}

/// Behavioral specification for functions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BehavioralSpec {
    /// Is the function idempotent?
    pub idempotent: bool,
    /// Is it a pure function?
    pub pure: bool,
    /// Side effects the function may have
    pub side_effects: Vec<SideEffect>,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Retry policy if applicable
    pub retry_policy: Option<RetryPolicy>,
    /// Is the function deterministic?
    pub deterministic: bool,
    /// Can it be executed in parallel?
    pub thread_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    /// Type of side effect
    pub effect_type: SideEffectType,
    /// Target of the effect (e.g., "database", "file_system")
    pub target: String,
    /// Description of the effect
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffectType {
    ModifiesState,
    PerformsIO,
    NetworkRequest,
    ModifiesDatabase,
    SendsNotification,
    ModifiesFileSystem,
    AllocatesResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryPolicy {
    None,
    Fixed { attempts: u32, delay_ms: u64 },
    ExponentialBackoff { max_attempts: u32, initial_delay_ms: u64 },
    Custom(String),
}

/// Example-based specification for functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleSpec {
    /// Unique identifier for the example
    pub id: String,
    /// Input parameters with values
    pub inputs: HashMap<String, ExampleValue>,
    /// Expected output
    pub output: ExampleOutput,
    /// Execution trace for debugging
    pub execution_trace: Option<Vec<String>>,
    /// Preconditions that must hold
    pub preconditions: Vec<String>,
    /// Postconditions that should hold after
    pub postconditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExampleValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<ExampleValue>),
    Object(HashMap<String, ExampleValue>),
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleOutput {
    /// Did the execution succeed?
    pub success: bool,
    /// Return value if any
    pub result: Option<ExampleValue>,
    /// Error if any
    pub error: Option<String>,
    /// State changes caused
    pub state_changes: Vec<StateChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    /// What was modified
    pub target: String,
    /// Type of modification
    pub change_type: String,
    /// Description
    pub description: String,
}

/// Proof specification for formal verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSpec {
    /// Theorem being proved
    pub theorem: String,
    /// Given conditions
    pub givens: Vec<String>,
    /// Proof steps
    pub steps: Vec<ProofStep>,
    /// Verification method used
    pub method: VerificationMethod,
    /// QED marker
    pub qed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStep {
    /// Step identifier
    pub id: String,
    /// Statement in this step
    pub statement: String,
    /// Justification for the step
    pub justification: String,
    /// References to previous steps
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    DirectProof,
    Induction,
    Contradiction,
    ModelChecking,
    SymbolicExecution,
    SMTSolver,
}

/// Resource contract specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContract {
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
    /// Maximum number of file handles
    pub max_file_handles: Option<u32>,
    /// Maximum execution time in ms
    pub max_execution_time_ms: Option<u64>,
    /// Maximum network bandwidth in KB/s
    pub max_bandwidth_kbps: Option<u64>,
    /// Maximum CPU cores to use
    pub max_cpu_cores: Option<u32>,
    /// Enforcement mechanism
    pub enforcement: EnforcementLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementLevel {
    /// Just monitor and log
    Monitor,
    /// Warn when approaching limits
    Warn,
    /// Hard enforce with runtime errors
    Enforce,
    /// Enforce with graceful degradation
    GracefulDegrade,
}

/// Generation hints for LLMs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationHints {
    /// Preferred coding style
    pub style_preference: StylePreference,
    /// Patterns to include
    pub include_patterns: Vec<String>,
    /// Patterns to avoid
    pub avoid_patterns: Vec<String>,
    /// Optimization level
    pub optimization_level: OptimizationLevel,
    /// Additional hints
    pub custom_hints: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StylePreference {
    Defensive,      // Lots of validation
    Performance,    // Optimize for speed
    Readable,       // Optimize for clarity
    Minimal,        // Least code possible
    Comprehensive,  // Handle all edge cases
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Size,
    Speed,
    Memory,
    Energy,
}

/// Causality chain for tracing effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalityChain {
    /// Initial event
    pub initial_event: String,
    /// Chain of causation
    pub chain: Vec<CausalLink>,
    /// Is this chain traceable at runtime?
    pub traceable: bool,
    /// Invariants that must hold throughout
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalLink {
    /// Cause event/action
    pub cause: String,
    /// Effect event/action
    pub effect: String,
    /// Condition for this link
    pub condition: Option<String>,
    /// Probability or certainty
    pub certainty: f64,
}

/// Intent specification for semantic understanding
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentSpec {
    /// Primary intent description
    pub primary_intent: String,
    /// Business/domain purpose
    pub business_purpose: Option<String>,
    /// Technical implementation approach
    pub technical_approach: Option<String>,
    /// Success criteria
    pub success_criteria: Vec<String>,
    /// Failure modes
    pub failure_modes: Vec<FailureMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureMode {
    /// Description of failure
    pub description: String,
    /// Probability (0.0 to 1.0)
    pub probability: f64,
    /// Impact level
    pub impact: ImpactLevel,
    /// Mitigation strategy
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Negligible,
    Minor,
    Moderate,
    Major,
    Critical,
}

/// Semantic block for compositional building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticBlock {
    /// Block name
    pub name: String,
    /// What this block guarantees
    pub guarantees: BlockGuarantees,
    /// Composition rules
    pub composition_rules: CompositionRules,
    /// Required context
    pub required_context: Vec<String>,
    /// Provided capabilities
    pub provides: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockGuarantees {
    /// Atomicity guarantee
    pub atomicity: bool,
    /// Consistency level
    pub consistency: ConsistencyLevel,
    /// Isolation level
    pub isolation: IsolationLevel,
    /// Durability guarantee
    pub durability: bool,
    /// Additional guarantees
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    Eventual,
    Strong,
    Linearizable,
    Sequential,
    Causal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
    Snapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionRules {
    /// Can this block be nested?
    pub can_nest: bool,
    /// Can instances run in parallel?
    pub can_parallelize: bool,
    /// Maximum nesting depth
    pub max_nesting_depth: Option<u32>,
    /// Incompatible with these blocks
    pub incompatible_with: Vec<String>,
    /// Required wrapper blocks
    pub requires_wrapper: Vec<String>,
}

/// Complete metadata for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Function name
    pub name: String,
    /// Intent specification
    pub intent: IntentSpec,
    /// Behavioral specification
    pub behavior: Option<BehavioralSpec>,
    /// Example specifications
    pub examples: Vec<ExampleSpec>,
    /// Proof specifications
    pub proofs: Vec<ProofSpec>,
    /// Resource contracts
    pub resources: Option<ResourceContract>,
    /// Generation hints
    pub generation_hints: Option<GenerationHints>,
    /// Causality information
    pub causality: Option<CausalityChain>,
    /// Version information
    pub version: SemanticVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticVersion {
    /// Major version (breaking changes)
    pub major: u32,
    /// Minor version (new features)
    pub minor: u32,
    /// Patch version (bug fixes)
    pub patch: u32,
    /// Breaking changes description
    pub breaking_changes: Vec<String>,
    /// Migration guide
    pub migration_guide: Option<String>,
}

/// Metadata repository for the entire program
pub struct MetadataRepository {
    /// Function metadata
    pub functions: HashMap<String, FunctionMetadata>,
    /// Semantic types
    pub types: HashMap<String, SemanticType>,
    /// Semantic blocks
    pub blocks: HashMap<String, SemanticBlock>,
    /// Global generation hints
    pub global_hints: GenerationHints,
}

impl MetadataRepository {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            types: HashMap::new(),
            blocks: HashMap::new(),
            global_hints: GenerationHints {
                style_preference: StylePreference::Defensive,
                include_patterns: vec!["validation".to_string(), "error_handling".to_string()],
                avoid_patterns: vec!["unsafe".to_string(), "unwrap".to_string()],
                optimization_level: OptimizationLevel::None,
                custom_hints: HashMap::new(),
            },
        }
    }

    /// Register a semantic type
    pub fn register_type(&mut self, semantic_type: SemanticType) {
        self.types.insert(semantic_type.name.clone(), semantic_type);
    }

    /// Register function metadata
    pub fn register_function(&mut self, metadata: FunctionMetadata) {
        self.functions.insert(metadata.name.clone(), metadata);
    }

    /// Register a semantic block
    pub fn register_block(&mut self, block: SemanticBlock) {
        self.blocks.insert(block.name.clone(), block);
    }

    /// Get metadata for verification
    pub fn get_function_metadata(&self, name: &str) -> Option<&FunctionMetadata> {
        self.functions.get(name)
    }

    /// Check if a type is semantically valid
    pub fn validate_semantic_type(&self, type_name: &str, value: &str) -> Result<(), String> {
        if let Some(sem_type) = self.types.get(type_name) {
            if let Some(constraint) = &sem_type.constraint {
                // TODO: Implement regex validation
                // For now, just check if it's not empty
                if value.is_empty() {
                    return Err(format!("Value for {} cannot be empty", type_name));
                }
            }
            Ok(())
        } else {
            Err(format!("Unknown semantic type: {}", type_name))
        }
    }
}