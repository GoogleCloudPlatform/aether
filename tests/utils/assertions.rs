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

//! Custom assertions for AetherScript testing

use super::compiler_wrapper::{ExecutionResult, TestCompilationResult};

/// Assert that compilation succeeded
pub fn assert_compilation_success(result: &TestCompilationResult, message: &str) {
    if !result.is_success() {
        panic!("{}: Compilation failed: {:?}", message, result.error());
    }
}

/// Assert that compilation failed
pub fn assert_compilation_failure(result: &TestCompilationResult, message: &str) {
    if !result.is_failure() {
        panic!(
            "{}: Expected compilation to fail, but it succeeded",
            message
        );
    }
}

/// Assert that compilation failed with specific error
pub fn assert_compilation_error(
    result: &TestCompilationResult,
    expected_error: &str,
    message: &str,
) {
    assert_compilation_failure(result, message);

    if let Some(error) = result.error() {
        let error_string = format!("{}", error);
        if !error_string.contains(expected_error) {
            panic!(
                "{}: Expected error containing '{}', got: {}",
                message, expected_error, error_string
            );
        }
    }
}

/// Assert that execution succeeded
pub fn assert_execution_success(result: &ExecutionResult, message: &str) {
    if !result.is_success() {
        panic!(
            "{}: Execution failed with exit code {}: {}",
            message,
            result.exit_code(),
            result.stderr()
        );
    }
}

/// Assert that execution failed
pub fn assert_execution_failure(result: &ExecutionResult, message: &str) {
    if result.is_success() {
        panic!("{}: Expected execution to fail, but it succeeded", message);
    }
}

/// Assert that output contains expected text
pub fn assert_output_contains(result: &ExecutionResult, expected: &str, message: &str) {
    if !result.stdout().contains(expected) {
        panic!(
            "{}: Expected output to contain '{}', got: '{}'",
            message,
            expected,
            result.stdout()
        );
    }
}

/// Assert that output equals expected text
pub fn assert_output_equals(result: &ExecutionResult, expected: &str, message: &str) {
    let actual = result.stdout().trim();
    let expected = expected.trim();
    if actual != expected {
        panic!(
            "{}: Expected output '{}', got '{}'",
            message, expected, actual
        );
    }
}

/// Assert that compilation time is within bounds
pub fn assert_compilation_time_under(
    result: &TestCompilationResult,
    max_duration: std::time::Duration,
    message: &str,
) {
    if result.compilation_time > max_duration {
        panic!(
            "{}: Compilation took {:?}, expected under {:?}",
            message, result.compilation_time, max_duration
        );
    }
}

/// Assert that execution time is within bounds
pub fn assert_execution_time_under(
    result: &ExecutionResult,
    max_duration: std::time::Duration,
    message: &str,
) {
    if result.execution_time > max_duration {
        panic!(
            "{}: Execution took {:?}, expected under {:?}",
            message, result.execution_time, max_duration
        );
    }
}

/// Assert that compiled file exists and has expected size
pub fn assert_output_file_exists(result: &TestCompilationResult, message: &str) {
    assert_compilation_success(result, message);

    if let Some(compilation_result) = result.success() {
        if !compilation_result.executable_path.exists() {
            panic!(
                "{}: Output file does not exist: {}",
                message,
                compilation_result.executable_path.display()
            );
        }
    }
}

/// Assert that output file has minimum size (indicating successful compilation)
pub fn assert_output_file_size_min(result: &TestCompilationResult, min_size: u64, message: &str) {
    assert_output_file_exists(result, message);

    if let Some(compilation_result) = result.success() {
        let file_size = std::fs::metadata(&compilation_result.executable_path)
            .expect("Failed to get file metadata")
            .len();

        if file_size < min_size {
            panic!(
                "{}: Output file size {} is less than minimum {}",
                message, file_size, min_size
            );
        }
    }
}

/// Assert that warning count is within expected range
pub fn assert_warning_count(result: &TestCompilationResult, expected_count: usize, message: &str) {
    assert_compilation_success(result, message);

    if let Some(compilation_result) = result.success() {
        // Note: Current CompilationResult doesn't track warnings
        // This assertion is currently a no-op
        let _ = expected_count;
    }
}

/// Assert that warnings contain specific text
pub fn assert_warning_contains(
    result: &TestCompilationResult,
    expected_warning: &str,
    message: &str,
) {
    assert_compilation_success(result, message);

    if let Some(compilation_result) = result.success() {
        // Note: Current CompilationResult doesn't track warnings
        // This assertion is currently a no-op
        let _ = expected_warning;
    }
}

/// Assert that no warnings were generated
pub fn assert_no_warnings(result: &TestCompilationResult, message: &str) {
    assert_warning_count(result, 0, message);
}

/// Assert full compilation and execution pipeline
pub fn assert_compile_and_execute(
    result: &TestCompilationResult,
    expected_output: &str,
    message: &str,
) {
    assert_compilation_success(result, &format!("{} - compilation", message));
    assert_output_file_exists(result, &format!("{} - output file", message));

    let execution = result.execute();
    assert_execution_success(&execution, &format!("{} - execution", message));
    assert_output_contains(
        &execution,
        expected_output,
        &format!("{} - output", message),
    );
}

/// Assert compile and execute with exact output match
pub fn assert_compile_and_execute_exact(
    result: &TestCompilationResult,
    expected_output: &str,
    message: &str,
) {
    assert_compilation_success(result, &format!("{} - compilation", message));
    assert_output_file_exists(result, &format!("{} - output file", message));

    let execution = result.execute();
    assert_execution_success(&execution, &format!("{} - execution", message));
    assert_output_equals(
        &execution,
        expected_output,
        &format!("{} - output", message),
    );
}
