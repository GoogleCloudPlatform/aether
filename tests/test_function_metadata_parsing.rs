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

//! Tests for function metadata parsing

use aether::ast::{ComplexityNotation, ComplexityType, FailureAction, PerformanceMetric};
use aether::lexer::Lexer;
use aether::parser::Parser;

#[test]
fn test_function_with_precondition() {
    let source = r#"
(DEFINE_MODULE
  (NAME 'test_module')
  (CONTENT
    (DEFINE_FUNCTION
      (NAME 'safe_divide')
      (ACCEPTS_PARAMETER (NAME 'a') (TYPE INTEGER))
      (ACCEPTS_PARAMETER (NAME 'b') (TYPE INTEGER))
      (RETURNS INTEGER)
      (PRECONDITION (PREDICATE_NOT_EQUALS 'b' 0) ASSERT_FAIL "Divisor cannot be zero")
      (BODY
        (EXPRESSION_DIVIDE 'a' 'b')))))
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let function = &program.modules[0].function_definitions[0];

    assert_eq!(function.name.name, "safe_divide");
    assert_eq!(function.metadata.preconditions.len(), 1);
    assert_eq!(
        function.metadata.preconditions[0].message,
        Some("Divisor cannot be zero".to_string())
    );
    assert_eq!(
        function.metadata.preconditions[0].failure_action,
        FailureAction::AssertFail
    );
}

#[test]
fn test_function_with_postcondition() {
    let source = r#"
(DEFINE_MODULE
  (NAME 'test_module')
  (CONTENT
    (DEFINE_FUNCTION
      (NAME 'abs')
      (ACCEPTS_PARAMETER (NAME 'x') (TYPE INTEGER))
      (RETURNS INTEGER)
      (POSTCONDITION (PREDICATE_GREATER_THAN_OR_EQUAL_TO (RETURN_VALUE) 0) ASSERT_FAIL "Result must be non-negative")
      (BODY
        (IF_CONDITION (PREDICATE_LESS_THAN 'x' 0)
          (THEN_EXECUTE (EXPRESSION_NEGATE 'x'))
          (ELSE_EXECUTE 'x'))))))
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let function = &program.modules[0].function_definitions[0];

    assert_eq!(function.metadata.postconditions.len(), 1);
    assert_eq!(
        function.metadata.postconditions[0].message,
        Some("Result must be non-negative".to_string())
    );
}

#[test]
fn test_function_with_performance_expectation() {
    let source = r#"
(DEFINE_MODULE
  (NAME 'test_module')
  (CONTENT
    (DEFINE_FUNCTION
      (NAME 'fast_function')
      (RETURNS VOID)
      (PERFORMANCE_EXPECTATION LATENCY_MS 10.0 "Average case")
      (BODY
        (RETURN_VOID)))))
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let function = &program.modules[0].function_definitions[0];

    assert!(function.metadata.performance_expectation.is_some());
    let perf = function.metadata.performance_expectation.as_ref().unwrap();
    assert_eq!(perf.metric, PerformanceMetric::LatencyMs);
    assert_eq!(perf.target_value, 10.0);
    assert_eq!(perf.context, Some("Average case".to_string()));
}

#[test]
fn test_function_with_complexity_expectation() {
    let source = r#"
(DEFINE_MODULE
  (NAME 'test_module')
  (CONTENT
    (DEFINE_FUNCTION
      (NAME 'sort_function')
      (ACCEPTS_PARAMETER (NAME 'arr') (TYPE (ARRAY_OF_TYPE INTEGER)))
      (RETURNS (ARRAY_OF_TYPE INTEGER))
      (COMPLEXITY_EXPECTATION TIME BIG_O "n log n")
      (BODY
        'arr'))))
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let function = &program.modules[0].function_definitions[0];

    assert!(function.metadata.complexity_expectation.is_some());
    let complexity = function.metadata.complexity_expectation.as_ref().unwrap();
    assert_eq!(complexity.complexity_type, ComplexityType::Time);
    assert_eq!(complexity.notation, ComplexityNotation::BigO);
    assert_eq!(complexity.value, "n log n");
}

#[test]
fn test_function_with_algorithm_hint() {
    let source = r#"
(DEFINE_MODULE
  (NAME 'test_module')
  (CONTENT
    (DEFINE_FUNCTION
      (NAME 'merge_sort')
      (ACCEPTS_PARAMETER (NAME 'arr') (TYPE (ARRAY_OF_TYPE INTEGER)))
      (RETURNS (ARRAY_OF_TYPE INTEGER))
      (ALGORITHM_HINT "divide-and-conquer")
      (BODY
        'arr'))))
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let function = &program.modules[0].function_definitions[0];

    assert_eq!(
        function.metadata.algorithm_hint,
        Some("divide-and-conquer".to_string())
    );
}

#[test]
fn test_function_with_multiple_metadata() {
    let source = r#"
(DEFINE_MODULE
  (NAME 'test_module')
  (CONTENT
    (DEFINE_FUNCTION
      (NAME 'binary_search')
      (ACCEPTS_PARAMETER (NAME 'arr') (TYPE (ARRAY_OF_TYPE INTEGER)))
      (ACCEPTS_PARAMETER (NAME 'target') (TYPE INTEGER))
      (RETURNS INTEGER)
      (PRECONDITION (EXPRESSION_GREATER_THAN (ARRAY_LENGTH 'arr') 0) ASSERT_FAIL "Array must not be empty")
      (POSTCONDITION (LOGICAL_OR 
        (PREDICATE_EQUALS (RETURN_VALUE) -1)
        (PREDICATE_EQUALS (ARRAY_ACCESS 'arr' (RETURN_VALUE)) 'target'))
        ASSERT_FAIL "Must return valid index or -1")
      (ALGORITHM_HINT "binary search")
      (COMPLEXITY_EXPECTATION TIME BIG_O "log n")
      (PERFORMANCE_EXPECTATION LATENCY_MS 0.1)
      (THREAD_SAFE true)
      (MAY_BLOCK false)
      (BODY
        -1))))
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_program().unwrap();
    let function = &program.modules[0].function_definitions[0];

    assert_eq!(function.metadata.preconditions.len(), 1);
    assert_eq!(function.metadata.postconditions.len(), 1);
    assert_eq!(
        function.metadata.algorithm_hint,
        Some("binary search".to_string())
    );
    assert!(function.metadata.complexity_expectation.is_some());
    assert!(function.metadata.performance_expectation.is_some());
    assert_eq!(function.metadata.thread_safe, Some(true));
    assert_eq!(function.metadata.may_block, Some(false));
}
