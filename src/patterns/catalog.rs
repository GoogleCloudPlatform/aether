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

//! Standard Pattern Catalog
//!
//! Pre-verified patterns for common programming tasks

use super::*;
use crate::verification::contracts::{BinaryOp, Expression as ContractExpr};
use std::collections::HashMap;

/// Helper to create a default FunctionContract for patterns
fn create_pattern_contract(name: &str) -> FunctionContract {
    FunctionContract {
        function_name: name.to_string(),
        preconditions: vec![],
        postconditions: vec![],
        invariants: vec![],
        modifies: Default::default(),
        is_pure: true,
        decreases: None,
        intent: None,
        behavior: None,
        resources: None,
        failure_actions: HashMap::new(),
        propagation: Default::default(),
        proof_obligations: vec![],
    }
}

/// Load all standard patterns
pub fn load_all_patterns() -> Vec<Pattern> {
    let mut patterns = Vec::new();

    // Data structure patterns
    patterns.extend(load_data_structure_patterns());

    // Algorithm patterns
    patterns.extend(load_algorithm_patterns());

    // I/O patterns
    patterns.extend(load_io_patterns());

    // Resource management patterns
    patterns.extend(load_resource_patterns());

    // Error handling patterns
    patterns.extend(load_error_patterns());

    // Validation patterns
    patterns.extend(load_validation_patterns());

    patterns
}

/// Load data structure patterns
fn load_data_structure_patterns() -> Vec<Pattern> {
    vec![
        // Safe array access pattern
        Pattern {
            id: "safe_array_access".to_string(),
            name: "Safe Array Access".to_string(),
            category: PatternCategory::DataStructures,
            intent: "Access array element with bounds checking".to_string(),
            description: "Safely access an array element with automatic bounds checking to prevent out-of-bounds errors".to_string(),
            metadata: PatternMetadata {
                tags: vec!["array".to_string(), "bounds-check".to_string(), "safety".to_string()],
                requires: vec![],
                provides: vec!["bounded_access".to_string()],
                author: "aetherlang".to_string(),
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
            parameters: vec![
                PatternParameter {
                    name: "array_expr".to_string(),
                    param_type: ParameterType::Expression,
                    description: "The array to access".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "index_expr".to_string(),
                    param_type: ParameterType::Expression,
                    description: "The index to access".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "default_value".to_string(),
                    param_type: ParameterType::Expression,
                    description: "Default value if index out of bounds".to_string(),
                    default: None,
                    constraints: vec![],
                },
            ],
            template: PatternTemplate::Expression(ExpressionTemplate {
                template: r#"
(IF_CONDITION
  (LOGICAL_AND
    (PREDICATE_GREATER_THAN_OR_EQUAL_TO {{index_expr}} (INTEGER_LITERAL 0))
    (PREDICATE_LESS_THAN {{index_expr}} (ARRAY_LENGTH {{array_expr}})))
  (THEN_EXECUTE
    (GET_ARRAY_ELEMENT {{array_expr}} {{index_expr}}))
  (ELSE_EXECUTE
    {{default_value}}))
"#.to_string(),
            }),
            contract: {
                let mut contract = create_pattern_contract("safe_array_access");
                // intent: "Safe array element access with bounds checking"
                contract
            },
            composition_rules: vec![
                CompositionRule {
                    id: "compose_with_iteration".to_string(),
                    condition: CompositionCondition::CompatibleWith {
                        pattern_id: "array_iteration".to_string(),
                    },
                    action: CompositionAction::Nested {
                        parent_param: "body".to_string(),
                    },
                    priority: 10,
                },
            ],
            examples: vec![
                PatternExample {
                    name: "Access with default".to_string(),
                    description: "Access array element with -1 as default".to_string(),
                    parameters: HashMap::from([
                        ("array_expr".to_string(), ParameterValue::Identifier("data".to_string())),
                        ("index_expr".to_string(), ParameterValue::Identifier("i".to_string())),
                        ("default_value".to_string(), ParameterValue::Integer(-1)),
                    ]),
                    preview: "if (i >= 0 && i < data.length) data[i] else -1".to_string(),
                    verified: true,
                },
            ],
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
                scalability: "O(1) - constant time".to_string(),
            },
        },

        // Dynamic array pattern
        Pattern {
            id: "dynamic_array".to_string(),
            name: "Dynamic Array".to_string(),
            category: PatternCategory::DataStructures,
            intent: "Create and manage a dynamically-sized array".to_string(),
            description: "A growable array implementation with automatic resizing".to_string(),
            metadata: PatternMetadata {
                tags: vec!["array".to_string(), "dynamic".to_string(), "resizable".to_string(), "vector".to_string()],
                requires: vec!["memory_allocation".to_string()],
                provides: vec!["dynamic_storage".to_string()],
                author: "aetherlang".to_string(),
                version: "1.0.0".to_string(),
                stability: StabilityLevel::Stable,
                complexity: ComplexityEstimate {
                    time: "O(1) amortized append".to_string(),
                    space: "O(n)".to_string(),
                    io: None,
                },
                safety: SafetyGuarantees {
                    memory_safe: true,
                    thread_safe: false,
                    exception_safe: ExceptionSafety::Strong,
                    resource_safe: true,
                },
            },
            parameters: vec![
                PatternParameter {
                    name: "element_type".to_string(),
                    param_type: ParameterType::TypeName,
                    description: "Type of elements to store".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "initial_capacity".to_string(),
                    param_type: ParameterType::IntegerConstant,
                    description: "Initial capacity".to_string(),
                    default: Some(ParameterValue::Integer(16)),
                    constraints: vec![ParameterConstraint::Range { min: 1, max: 1024 }],
                },
            ],
            template: PatternTemplate::Module(ModuleTemplate {
                name_template: "dynamic_array_{{element_type}}".to_string(),
                imports: vec![],
                types: vec![
                    r#"(DEFINE_STRUCTURED_TYPE
  (NAME "DynamicArray_{{element_type}}")
  (FIELD (NAME "data") (TYPE (POINTER_TO {{element_type}})))
  (FIELD (NAME "size") (TYPE INTEGER))
  (FIELD (NAME "capacity") (TYPE INTEGER)))"#.to_string()
                ],
                functions: vec![
                    FunctionTemplate {
                        name_template: "create_dynamic_array_{{element_type}}".to_string(),
                        parameters: vec![],
                        return_type_template: "DynamicArray_{{element_type}}".to_string(),
                        body_template: r#"(RESOURCE_SCOPE
  (NAME "array_creation")
  (ACQUIRE_RESOURCE
    (RESOURCE_TYPE "memory_buffer")
    (RESOURCE_BINDING "buffer")
    (VALUE (CALL_FUNCTION
      (NAME "aether_alloc")
      (ARGUMENT (EXPRESSION_MULTIPLY {{initial_capacity}} (SIZEOF {{element_type}})))))
    (CLEANUP "aether_free"))
  (BODY
    (CONSTRUCT
      (TYPE "DynamicArray_{{element_type}}")
      (FIELD_VALUE (NAME "data") (VALUE buffer))
      (FIELD_VALUE (NAME "size") (VALUE (INTEGER_LITERAL 0)))
      (FIELD_VALUE (NAME "capacity") (VALUE (INTEGER_LITERAL {{initial_capacity}})))))))"#.to_string(),
                        contract_template: Some("(POSTCONDITION (PREDICATE_NOT_EQUALS RETURNED_VALUE NULL))".to_string()),
                    },
                ],
            }),
            contract: {
                let mut contract = create_pattern_contract("dynamic_array");
                contract.resources = Some(crate::semantic::metadata::ResourceContract {
                    max_memory_mb: Some(100),
                    max_file_handles: None,
                    max_execution_time_ms: None,
                    max_bandwidth_kbps: None,
                    max_cpu_cores: None,
                    enforcement: crate::semantic::metadata::EnforcementLevel::Enforce,
                });
                // intent: "Dynamic array with automatic resizing"
                contract
            },
            composition_rules: vec![],
            examples: vec![],
            performance: PerformanceProfile {
                execution_time: ExecutionTime {
                    best_case_us: 10,
                    average_case_us: 20,
                    worst_case_us: 1000, // When resizing
                },
                memory_usage: MemoryUsage {
                    stack_bytes: 24,
                    heap_bytes: 1024, // Depends on capacity
                    allocates: true,
                },
                io_profile: None,
                scalability: "O(n) space, O(1) amortized insertion".to_string(),
            },
        },
    ]
}

/// Load algorithm patterns
fn load_algorithm_patterns() -> Vec<Pattern> {
    vec![
        // Binary search pattern
        Pattern {
            id: "binary_search".to_string(),
            name: "Binary Search".to_string(),
            category: PatternCategory::Algorithms,
            intent: "Find element in sorted array using binary search".to_string(),
            description: "Efficient O(log n) search in a sorted array".to_string(),
            metadata: PatternMetadata {
                tags: vec![
                    "search".to_string(),
                    "binary-search".to_string(),
                    "sorted".to_string(),
                    "algorithm".to_string(),
                ],
                requires: vec!["sorted_array".to_string()],
                provides: vec!["efficient_search".to_string()],
                author: "aetherlang".to_string(),
                version: "1.0.0".to_string(),
                stability: StabilityLevel::Stable,
                complexity: ComplexityEstimate {
                    time: "O(log n)".to_string(),
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
            parameters: vec![
                PatternParameter {
                    name: "array_name".to_string(),
                    param_type: ParameterType::Identifier,
                    description: "Name of the sorted array".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "target_value".to_string(),
                    param_type: ParameterType::Expression,
                    description: "Value to search for".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "element_type".to_string(),
                    param_type: ParameterType::TypeName,
                    description: "Type of array elements".to_string(),
                    default: Some(ParameterValue::Type("INTEGER".to_string())),
                    constraints: vec![],
                },
            ],
            template: PatternTemplate::Function(FunctionTemplate {
                name_template: "binary_search_{{element_type}}".to_string(),
                parameters: vec![
                    ParameterTemplate {
                        name_template: "arr".to_string(),
                        type_template: "(ARRAY_OF_TYPE {{element_type}})".to_string(),
                    },
                    ParameterTemplate {
                        name_template: "target".to_string(),
                        type_template: "{{element_type}}".to_string(),
                    },
                ],
                return_type_template: "INTEGER".to_string(),
                body_template: r#"(BODY
  (DECLARE_VARIABLE (NAME "left") (TYPE INTEGER) (INITIAL_VALUE (INTEGER_LITERAL 0)))
  (DECLARE_VARIABLE (NAME "right") (TYPE INTEGER)
    (INITIAL_VALUE (EXPRESSION_SUBTRACT (ARRAY_LENGTH arr) (INTEGER_LITERAL 1))))

  (LOOP_WHILE_CONDITION
    (PREDICATE_LESS_THAN_OR_EQUAL_TO left right)
    (BODY
      (DECLARE_VARIABLE (NAME "mid") (TYPE INTEGER)
        (INITIAL_VALUE (EXPRESSION_INTEGER_DIVIDE
          (EXPRESSION_ADD left right) (INTEGER_LITERAL 2))))

      (IF_CONDITION
        (PREDICATE_EQUALS (GET_ARRAY_ELEMENT arr mid) target)
        (THEN_EXECUTE (RETURN_VALUE mid))
        (ELSE_IF_CONDITION
          (PREDICATE_LESS_THAN (GET_ARRAY_ELEMENT arr mid) target)
          (THEN_EXECUTE (ASSIGN (TARGET_VARIABLE "left")
            (SOURCE_EXPRESSION (EXPRESSION_ADD mid (INTEGER_LITERAL 1)))))
          (ELSE_EXECUTE (ASSIGN (TARGET_VARIABLE "right")
            (SOURCE_EXPRESSION (EXPRESSION_SUBTRACT mid (INTEGER_LITERAL 1)))))))))

  (RETURN_VALUE (INTEGER_LITERAL -1)))"#
                    .to_string(),
                contract_template: Some(
                    r#"(PRECONDITION
  (CUSTOM "array_is_sorted(arr)")
  (FAILURE_ACTION THROW_EXCEPTION)
  (PROOF_HINT "Array must be sorted for binary search"))
(POSTCONDITION
  (CUSTOM "result == -1 || arr[result] == target")
  (PROOF_HINT "Returns index of target or -1 if not found"))"#
                        .to_string(),
                ),
            }),
            contract: {
                let mut contract = create_pattern_contract("binary_search");
                // intent: "Binary search in sorted array"
                contract
            },
            composition_rules: vec![],
            examples: vec![PatternExample {
                name: "Search for integer".to_string(),
                description: "Binary search for integer in sorted array".to_string(),
                parameters: HashMap::from([(
                    "element_type".to_string(),
                    ParameterValue::Type("INTEGER".to_string()),
                )]),
                preview: "Returns index of element or -1".to_string(),
                verified: true,
            }],
            performance: PerformanceProfile {
                execution_time: ExecutionTime {
                    best_case_us: 1,
                    average_case_us: 10,
                    worst_case_us: 20,
                },
                memory_usage: MemoryUsage {
                    stack_bytes: 24,
                    heap_bytes: 0,
                    allocates: false,
                },
                io_profile: None,
                scalability: "O(log n) time complexity".to_string(),
            },
        },
        // Map/filter/reduce pattern
        Pattern {
            id: "map_filter_reduce".to_string(),
            name: "Map-Filter-Reduce".to_string(),
            category: PatternCategory::Algorithms,
            intent: "Transform, filter, and aggregate array data".to_string(),
            description: "Functional programming pattern for data transformation".to_string(),
            metadata: PatternMetadata {
                tags: vec![
                    "functional".to_string(),
                    "map".to_string(),
                    "filter".to_string(),
                    "reduce".to_string(),
                    "transform".to_string(),
                ],
                requires: vec![],
                provides: vec!["data_transformation".to_string()],
                author: "aetherlang".to_string(),
                version: "1.0.0".to_string(),
                stability: StabilityLevel::Stable,
                complexity: ComplexityEstimate {
                    time: "O(n)".to_string(),
                    space: "O(n)".to_string(),
                    io: None,
                },
                safety: SafetyGuarantees {
                    memory_safe: true,
                    thread_safe: true,
                    exception_safe: ExceptionSafety::Strong,
                    resource_safe: true,
                },
            },
            parameters: vec![
                PatternParameter {
                    name: "input_array".to_string(),
                    param_type: ParameterType::Identifier,
                    description: "Input array to process".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "map_expr".to_string(),
                    param_type: ParameterType::Expression,
                    description: "Transformation expression".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "filter_predicate".to_string(),
                    param_type: ParameterType::Expression,
                    description: "Filter condition".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "reduce_expr".to_string(),
                    param_type: ParameterType::Expression,
                    description: "Reduction expression".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "initial_value".to_string(),
                    param_type: ParameterType::Expression,
                    description: "Initial value for reduction".to_string(),
                    default: None,
                    constraints: vec![],
                },
            ],
            template: PatternTemplate::Statement(StatementTemplate {
                template: r#"(BLOCK
  ;; Allocate result array
  (RESOURCE_SCOPE
    (NAME "transform_scope")
    (ACQUIRE_RESOURCE
      (RESOURCE_TYPE "memory_buffer")
      (RESOURCE_BINDING "temp_array")
      (VALUE (CALL_FUNCTION
        (NAME "aether_alloc")
        (ARGUMENT (EXPRESSION_MULTIPLY
          (ARRAY_LENGTH {{input_array}})
          (SIZEOF (ELEMENT_TYPE {{input_array}}))))))
      (CLEANUP "aether_free"))
    (BODY
      (DECLARE_VARIABLE (NAME "write_index") (TYPE INTEGER)
        (INITIAL_VALUE (INTEGER_LITERAL 0)) (MUTABLE TRUE))

      ;; Map and filter
      (LOOP_FOR_EACH_ELEMENT
        (COLLECTION {{input_array}})
        (ELEMENT_BINDING "elem")
        (BODY
          (DECLARE_VARIABLE (NAME "mapped")
            (INITIAL_VALUE {{map_expr}}))
          (IF_CONDITION {{filter_predicate}}
            (THEN_EXECUTE
              (BLOCK
                (SET_ARRAY_ELEMENT temp_array write_index mapped)
                (ASSIGN (TARGET_VARIABLE "write_index")
                  (SOURCE_EXPRESSION (EXPRESSION_ADD write_index (INTEGER_LITERAL 1)))))))))

      ;; Reduce
      (DECLARE_VARIABLE (NAME "result")
        (INITIAL_VALUE {{initial_value}}) (MUTABLE TRUE))
      (LOOP_FIXED_ITERATIONS
        (COUNTER "i")
        (FROM (INTEGER_LITERAL 0))
        (TO write_index)
        (DO
          (ASSIGN (TARGET_VARIABLE "result")
            (SOURCE_EXPRESSION {{reduce_expr}})))))))"#
                    .to_string(),
            }),
            contract: {
                let mut contract = create_pattern_contract("map_filter_reduce");
                // intent: "Functional data transformation"
                contract
            },
            composition_rules: vec![],
            examples: vec![],
            performance: PerformanceProfile {
                execution_time: ExecutionTime {
                    best_case_us: 100,
                    average_case_us: 500,
                    worst_case_us: 1000,
                },
                memory_usage: MemoryUsage {
                    stack_bytes: 32,
                    heap_bytes: 4096,
                    allocates: true,
                },
                io_profile: None,
                scalability: "O(n) time and space".to_string(),
            },
        },
    ]
}

/// Load I/O patterns
fn load_io_patterns() -> Vec<Pattern> {
    vec![
        // Safe file reading pattern
        Pattern {
            id: "safe_file_read".to_string(),
            name: "Safe File Read".to_string(),
            category: PatternCategory::InputOutput,
            intent: "Read file contents safely with automatic resource cleanup".to_string(),
            description:
                "Read entire file into memory with proper error handling and resource management"
                    .to_string(),
            metadata: PatternMetadata {
                tags: vec![
                    "file".to_string(),
                    "io".to_string(),
                    "read".to_string(),
                    "safe".to_string(),
                ],
                requires: vec!["file_system".to_string()],
                provides: vec!["file_content".to_string()],
                author: "aetherlang".to_string(),
                version: "1.0.0".to_string(),
                stability: StabilityLevel::Stable,
                complexity: ComplexityEstimate {
                    time: "O(n)".to_string(),
                    space: "O(n)".to_string(),
                    io: Some("File read".to_string()),
                },
                safety: SafetyGuarantees {
                    memory_safe: true,
                    thread_safe: false,
                    exception_safe: ExceptionSafety::Strong,
                    resource_safe: true,
                },
            },
            parameters: vec![
                PatternParameter {
                    name: "filename".to_string(),
                    param_type: ParameterType::Expression,
                    description: "Path to file to read".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "max_size".to_string(),
                    param_type: ParameterType::IntegerConstant,
                    description: "Maximum file size in bytes".to_string(),
                    default: Some(ParameterValue::Integer(1024 * 1024)), // 1MB
                    constraints: vec![ParameterConstraint::Range {
                        min: 1,
                        max: 100 * 1024 * 1024,
                    }],
                },
            ],
            template: PatternTemplate::Function(FunctionTemplate {
                name_template: "read_file_safe".to_string(),
                parameters: vec![ParameterTemplate {
                    name_template: "path".to_string(),
                    type_template: "STRING".to_string(),
                }],
                return_type_template: "STRING".to_string(),
                body_template: r#"(BODY
  (TRY_EXECUTE
    (PROTECTED_BLOCK
      (RESOURCE_SCOPE
        (NAME "file_read_scope")
        (ACQUIRE_RESOURCE
          (RESOURCE_TYPE "file_handle")
          (RESOURCE_BINDING "file")
          (VALUE (CALL_FUNCTION
            (NAME "file_open")
            (ARGUMENT path)
            (ARGUMENT (STRING_LITERAL "r"))))
          (CLEANUP "file_close"))
        (BODY
          ;; Check file size
          (DECLARE_VARIABLE (NAME "size") (TYPE INTEGER)
            (INITIAL_VALUE (CALL_FUNCTION (NAME "file_size") (ARGUMENT file))))

          (IF_CONDITION
            (PREDICATE_GREATER_THAN size (INTEGER_LITERAL {{max_size}}))
            (THEN_EXECUTE
              (THROW_EXCEPTION
                (CALL_FUNCTION
                  (NAME "create_error")
                  (ARGUMENT (STRING_LITERAL "File too large"))))))

          ;; Read file content
          (RESOURCE_SCOPE
            (NAME "buffer_scope")
            (ACQUIRE_RESOURCE
              (RESOURCE_TYPE "memory_buffer")
              (RESOURCE_BINDING "buffer")
              (VALUE (CALL_FUNCTION
                (NAME "aether_alloc")
                (ARGUMENT (EXPRESSION_ADD size (INTEGER_LITERAL 1)))))
              (CLEANUP "aether_free"))
            (BODY
              (CALL_FUNCTION
                (NAME "file_read")
                (ARGUMENT file)
                (ARGUMENT buffer)
                (ARGUMENT size))

              ;; Null-terminate
              (SET_ARRAY_ELEMENT buffer size (INTEGER_LITERAL 0))

              (RETURN_VALUE (CALL_FUNCTION
                (NAME "string_from_buffer")
                (ARGUMENT buffer))))))))
    (CATCH_EXCEPTION
      (EXCEPTION_TYPE "FileError")
      (BINDING_VARIABLE "e")
      (HANDLER_BLOCK
        (RETURN_VALUE (STRING_LITERAL ""))))))"#
                    .to_string(),
                contract_template: Some(
                    r#"(PRECONDITION
  (PREDICATE_NOT_EQUALS path "")
  (FAILURE_ACTION THROW_EXCEPTION)
  (PROOF_HINT "Path must not be empty"))
(POSTCONDITION
  (CUSTOM "result != NULL")
  (PROOF_HINT "Always returns a valid string, empty on error"))"#
                        .to_string(),
                ),
            }),
            contract: {
                let mut contract = create_pattern_contract("safe_file_read");
                contract.resources = Some(crate::semantic::metadata::ResourceContract {
                    max_memory_mb: Some(10),
                    max_file_handles: Some(1),
                    max_execution_time_ms: None,
                    max_bandwidth_kbps: None,
                    max_cpu_cores: None,
                    enforcement: crate::semantic::metadata::EnforcementLevel::Enforce,
                });
                // intent: "Read file safely with size limits"
                contract
            },
            composition_rules: vec![],
            examples: vec![],
            performance: PerformanceProfile {
                execution_time: ExecutionTime {
                    best_case_us: 1000,
                    average_case_us: 10000,
                    worst_case_us: 100000,
                },
                memory_usage: MemoryUsage {
                    stack_bytes: 64,
                    heap_bytes: 1024 * 1024,
                    allocates: true,
                },
                io_profile: Some(IOProfile {
                    reads: true,
                    writes: false,
                    network: false,
                    file: true,
                }),
                scalability: "O(n) where n is file size".to_string(),
            },
        },
    ]
}

/// Load resource management patterns
fn load_resource_patterns() -> Vec<Pattern> {
    vec![
        // RAII pattern
        Pattern {
            id: "raii_wrapper".to_string(),
            name: "RAII Wrapper".to_string(),
            category: PatternCategory::ResourceManagement,
            intent: "Wrap resource in RAII pattern for automatic cleanup".to_string(),
            description: "Resource Acquisition Is Initialization pattern ensures cleanup"
                .to_string(),
            metadata: PatternMetadata {
                tags: vec![
                    "raii".to_string(),
                    "resource".to_string(),
                    "cleanup".to_string(),
                    "automatic".to_string(),
                ],
                requires: vec![],
                provides: vec!["automatic_cleanup".to_string()],
                author: "aetherlang".to_string(),
                version: "1.0.0".to_string(),
                stability: StabilityLevel::Stable,
                complexity: ComplexityEstimate {
                    time: "O(1)".to_string(),
                    space: "O(1)".to_string(),
                    io: None,
                },
                safety: SafetyGuarantees {
                    memory_safe: true,
                    thread_safe: false,
                    exception_safe: ExceptionSafety::Strong,
                    resource_safe: true,
                },
            },
            parameters: vec![
                PatternParameter {
                    name: "resource_type".to_string(),
                    param_type: ParameterType::StringConstant,
                    description: "Type of resource to manage".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "acquire_expr".to_string(),
                    param_type: ParameterType::Expression,
                    description: "Expression to acquire resource".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "cleanup_func".to_string(),
                    param_type: ParameterType::StringConstant,
                    description: "Cleanup function name".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "operation".to_string(),
                    param_type: ParameterType::Block,
                    description: "Operations to perform with resource".to_string(),
                    default: None,
                    constraints: vec![],
                },
            ],
            template: PatternTemplate::Statement(StatementTemplate {
                template: r#"(RESOURCE_SCOPE
  (NAME "raii_scope")
  (ACQUIRE_RESOURCE
    (RESOURCE_TYPE "{{resource_type}}")
    (RESOURCE_BINDING "resource")
    (VALUE {{acquire_expr}})
    (CLEANUP "{{cleanup_func}}"))
  (CLEANUP_ORDER "REVERSE_ACQUISITION")
  (BODY {{operation}}))"#
                    .to_string(),
            }),
            contract: {
                let mut contract = create_pattern_contract("raii_wrapper");
                // intent: "RAII resource management"
                contract
            },
            composition_rules: vec![CompositionRule {
                id: "nest_raii".to_string(),
                condition: CompositionCondition::CompatibleWith {
                    pattern_id: "raii_wrapper".to_string(),
                },
                action: CompositionAction::Nested {
                    parent_param: "operation".to_string(),
                },
                priority: 10,
            }],
            examples: vec![],
            performance: PerformanceProfile {
                execution_time: ExecutionTime {
                    best_case_us: 1,
                    average_case_us: 10,
                    worst_case_us: 100,
                },
                memory_usage: MemoryUsage {
                    stack_bytes: 8,
                    heap_bytes: 0,
                    allocates: false,
                },
                io_profile: None,
                scalability: "O(1) overhead".to_string(),
            },
        },
    ]
}

/// Load error handling patterns
fn load_error_patterns() -> Vec<Pattern> {
    vec![
        // Result type pattern
        Pattern {
            id: "result_type".to_string(),
            name: "Result Type".to_string(),
            category: PatternCategory::ErrorHandling,
            intent: "Handle errors using Result type pattern".to_string(),
            description: "Explicit error handling with Result<T, E> pattern".to_string(),
            metadata: PatternMetadata {
                tags: vec![
                    "error".to_string(),
                    "result".to_string(),
                    "option".to_string(),
                    "maybe".to_string(),
                ],
                requires: vec![],
                provides: vec!["explicit_errors".to_string()],
                author: "aetherlang".to_string(),
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
            parameters: vec![
                PatternParameter {
                    name: "ok_type".to_string(),
                    param_type: ParameterType::TypeName,
                    description: "Success value type".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "error_type".to_string(),
                    param_type: ParameterType::TypeName,
                    description: "Error value type".to_string(),
                    default: Some(ParameterValue::Type("STRING".to_string())),
                    constraints: vec![],
                },
            ],
            template: PatternTemplate::Module(ModuleTemplate {
                name_template: "result_{{ok_type}}_{{error_type}}".to_string(),
                imports: vec![],
                types: vec![r#"(DEFINE_ENUMERATION_TYPE
  (NAME "Result_{{ok_type}}_{{error_type}}")
  (VARIANT (NAME "Ok") (DATA {{ok_type}}))
  (VARIANT (NAME "Err") (DATA {{error_type}})))"#
                    .to_string()],
                functions: vec![
                    FunctionTemplate {
                        name_template: "is_ok_{{ok_type}}_{{error_type}}".to_string(),
                        parameters: vec![ParameterTemplate {
                            name_template: "result".to_string(),
                            type_template: "Result_{{ok_type}}_{{error_type}}".to_string(),
                        }],
                        return_type_template: "BOOLEAN".to_string(),
                        body_template: r#"(MATCH result
  (CASE "Ok" (RETURN_VALUE (BOOLEAN_LITERAL TRUE)))
  (CASE "Err" (RETURN_VALUE (BOOLEAN_LITERAL FALSE))))"#
                            .to_string(),
                        contract_template: None,
                    },
                    FunctionTemplate {
                        name_template: "unwrap_or_{{ok_type}}".to_string(),
                        parameters: vec![
                            ParameterTemplate {
                                name_template: "result".to_string(),
                                type_template: "Result_{{ok_type}}_{{error_type}}".to_string(),
                            },
                            ParameterTemplate {
                                name_template: "default".to_string(),
                                type_template: "{{ok_type}}".to_string(),
                            },
                        ],
                        return_type_template: "{{ok_type}}".to_string(),
                        body_template: r#"(MATCH result
  (CASE "Ok" (BINDING "value") (RETURN_VALUE value))
  (CASE "Err" (RETURN_VALUE default)))"#
                            .to_string(),
                        contract_template: None,
                    },
                ],
            }),
            contract: {
                let mut contract = create_pattern_contract("result_type");
                // intent: "Explicit error handling with Result type"
                contract
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
                    stack_bytes: 16,
                    heap_bytes: 0,
                    allocates: false,
                },
                io_profile: None,
                scalability: "O(1) overhead".to_string(),
            },
        },
    ]
}

/// Load validation patterns
fn load_validation_patterns() -> Vec<Pattern> {
    vec![
        // Input validation pattern
        Pattern {
            id: "input_validation".to_string(),
            name: "Input Validation".to_string(),
            category: PatternCategory::Validation,
            intent: "Validate and sanitize user input".to_string(),
            description: "Comprehensive input validation with clear error messages".to_string(),
            metadata: PatternMetadata {
                tags: vec![
                    "validation".to_string(),
                    "input".to_string(),
                    "sanitize".to_string(),
                    "security".to_string(),
                ],
                requires: vec![],
                provides: vec!["validated_input".to_string()],
                author: "aetherlang".to_string(),
                version: "1.0.0".to_string(),
                stability: StabilityLevel::Stable,
                complexity: ComplexityEstimate {
                    time: "O(n)".to_string(),
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
            parameters: vec![
                PatternParameter {
                    name: "input_value".to_string(),
                    param_type: ParameterType::Expression,
                    description: "Value to validate".to_string(),
                    default: None,
                    constraints: vec![],
                },
                PatternParameter {
                    name: "validation_type".to_string(),
                    param_type: ParameterType::Choice {
                        options: vec![
                            "email".to_string(),
                            "url".to_string(),
                            "phone".to_string(),
                            "integer".to_string(),
                            "positive_number".to_string(),
                        ],
                    },
                    description: "Type of validation to perform".to_string(),
                    default: None,
                    constraints: vec![],
                },
            ],
            template: PatternTemplate::Function(FunctionTemplate {
                name_template: "validate_{{validation_type}}".to_string(),
                parameters: vec![ParameterTemplate {
                    name_template: "input".to_string(),
                    type_template: "STRING".to_string(),
                }],
                return_type_template: "BOOLEAN".to_string(),
                body_template: r#"(BODY
  {{#if (eq validation_type "email")}}
  (LOGICAL_AND
    (PREDICATE_NOT_EQUALS input "")
    (EXPRESSION_CONTAINS input "@")
    (EXPRESSION_CONTAINS (SUBSTRING input
      (EXPRESSION_ADD (STRING_INDEX_OF input "@") (INTEGER_LITERAL 1))
      (STRING_LENGTH input))
      "."))
  {{/if}}
  {{#if (eq validation_type "positive_number")}}
  (TRY_EXECUTE
    (PROTECTED_BLOCK
      (DECLARE_VARIABLE (NAME "num") (TYPE INTEGER)
        (INITIAL_VALUE (CALL_FUNCTION (NAME "parse_int") (ARGUMENT input))))
      (RETURN_VALUE (PREDICATE_GREATER_THAN num (INTEGER_LITERAL 0))))
    (CATCH_EXCEPTION
      (EXCEPTION_TYPE "ParseError")
      (HANDLER_BLOCK (RETURN_VALUE (BOOLEAN_LITERAL FALSE)))))
  {{/if}})"#
                    .to_string(),
                contract_template: Some(
                    r#"(POSTCONDITION
  (CUSTOM "result implies input_is_valid")
  (PROOF_HINT "True result means input passed validation"))"#
                        .to_string(),
                ),
            }),
            contract: {
                let mut contract = create_pattern_contract("input_validation");
                // intent: "Validate user input"
                contract
            },
            composition_rules: vec![CompositionRule {
                id: "chain_validations".to_string(),
                condition: CompositionCondition::CompatibleWith {
                    pattern_id: "input_validation".to_string(),
                },
                action: CompositionAction::Sequential,
                priority: 10,
            }],
            examples: vec![],
            performance: PerformanceProfile {
                execution_time: ExecutionTime {
                    best_case_us: 1,
                    average_case_us: 10,
                    worst_case_us: 100,
                },
                memory_usage: MemoryUsage {
                    stack_bytes: 32,
                    heap_bytes: 0,
                    allocates: false,
                },
                io_profile: None,
                scalability: "O(n) where n is input length".to_string(),
            },
        },
    ]
}
