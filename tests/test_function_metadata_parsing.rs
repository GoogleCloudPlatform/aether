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
use aether::lexer::v2::Lexer;
use aether::parser::v2::Parser;

#[test]
fn test_function_with_precondition() {
    let source = r#"
module test_module {
    @pre({b != 0})
    func safe_divide(a: Int, b: Int) -> Int {
        return {a / b};
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_module().unwrap();
    let function = &program.function_definitions[0];

    assert_eq!(function.name.name, "safe_divide");
    assert_eq!(function.metadata.preconditions.len(), 1);
    // message not supported in @pre yet
    assert_eq!(
        function.metadata.preconditions[0].failure_action,
        FailureAction::ThrowException
    );
}

#[test]
fn test_function_with_postcondition() {
    let source = r#"
module test_module {
    @post({return_value >= 0})
    func abs(x: Int) -> Int {
        if {x < 0} { return {-x}; } else { return x; }
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_module().unwrap();
    let function = &program.function_definitions[0];

    assert_eq!(function.metadata.postconditions.len(), 1);
}

#[test]
fn test_function_with_performance_expectation() {
    let source = r#"
module test_module {
    @perf(metric="LatencyMs", target=10.0, context="Average case")
    func fast_function() -> Void {
        return;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_module().unwrap();
    let function = &program.function_definitions[0];

    assert!(function.metadata.performance_expectation.is_some());
    let perf = function.metadata.performance_expectation.as_ref().unwrap();
    assert_eq!(perf.metric, PerformanceMetric::LatencyMs);
    assert_eq!(perf.target_value, 10.0);
    assert_eq!(perf.context, Some("Average case".to_string()));
}

#[test]
fn test_function_with_complexity_expectation() {
    let source = r#"
module test_module {
    @complexity(type="Time", notation="BigO", value="n log n")
    func sort_function(arr: Array<Int>) -> Array<Int> {
        return arr;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_module().unwrap();
    let function = &program.function_definitions[0];

    assert!(function.metadata.complexity_expectation.is_some());
    let complexity = function.metadata.complexity_expectation.as_ref().unwrap();
    assert_eq!(complexity.complexity_type, ComplexityType::Time);
    assert_eq!(complexity.notation, ComplexityNotation::BigO);
    assert_eq!(complexity.value, "n log n");
}

#[test]
fn test_function_with_algorithm_hint() {
    let source = r#"
module test_module {
    @algo("divide-and-conquer")
    func merge_sort(arr: Array<Int>) -> Array<Int> {
        return arr;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_module().unwrap();
    let function = &program.function_definitions[0];

    assert_eq!(
        function.metadata.algorithm_hint,
        Some("divide-and-conquer".to_string())
    );
}

#[test]
fn test_function_with_multiple_metadata() {
    let source = r#"
module test_module {
    @pre({arr.length > 0})
    @post({return_value == -1 || arr[return_value] == target})
    @algo("binary search")
    @complexity(type="Time", notation="BigO", value="log n")
    @perf(metric="LatencyMs", target=0.1)
    @thread_safe(true)
    @may_block(false)
    func binary_search(arr: Array<Int>, target: Int) -> Int {
        return -1;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let program = parser.parse_module().unwrap();
    let function = &program.function_definitions[0];

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
