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

//! Test runner utilities for organizing and executing tests

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Test suite for organizing related tests
pub struct TestSuite {
    pub name: String,
    pub tests: Vec<Test>,
    pub setup: Option<Box<dyn Fn() -> ()>>,
    pub teardown: Option<Box<dyn Fn() -> ()>>,
}

/// Individual test case
pub struct Test {
    pub name: String,
    pub test_fn: Box<dyn Fn() -> TestResult>,
    pub timeout: Option<Duration>,
    pub expected_to_fail: bool,
}

/// Test execution result
#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub message: String,
    pub execution_time: Duration,
    pub details: Option<String>,
}

/// Test runner for executing test suites
pub struct TestRunner {
    pub suites: Vec<TestSuite>,
    pub verbose: bool,
    pub parallel: bool,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tests: Vec::new(),
            setup: None,
            teardown: None,
        }
    }

    /// Add a test to the suite
    pub fn add_test<F>(mut self, name: &str, test_fn: F) -> Self
    where
        F: Fn() -> TestResult + 'static,
    {
        self.tests.push(Test {
            name: name.to_string(),
            test_fn: Box::new(test_fn),
            timeout: None,
            expected_to_fail: false,
        });
        self
    }

    /// Add a test with timeout
    pub fn add_test_with_timeout<F>(mut self, name: &str, timeout: Duration, test_fn: F) -> Self
    where
        F: Fn() -> TestResult + 'static,
    {
        self.tests.push(Test {
            name: name.to_string(),
            test_fn: Box::new(test_fn),
            timeout: Some(timeout),
            expected_to_fail: false,
        });
        self
    }

    /// Add a test that is expected to fail
    pub fn add_test_expected_fail<F>(mut self, name: &str, test_fn: F) -> Self
    where
        F: Fn() -> TestResult + 'static,
    {
        self.tests.push(Test {
            name: name.to_string(),
            test_fn: Box::new(test_fn),
            timeout: None,
            expected_to_fail: true,
        });
        self
    }

    /// Set setup function
    pub fn setup<F>(mut self, setup_fn: F) -> Self
    where
        F: Fn() -> () + 'static,
    {
        self.setup = Some(Box::new(setup_fn));
        self
    }

    /// Set teardown function
    pub fn teardown<F>(mut self, teardown_fn: F) -> Self
    where
        F: Fn() -> () + 'static,
    {
        self.teardown = Some(Box::new(teardown_fn));
        self
    }
}

impl TestResult {
    /// Create a successful test result
    pub fn success(message: &str) -> Self {
        Self {
            passed: true,
            message: message.to_string(),
            execution_time: Duration::from_secs(0),
            details: None,
        }
    }

    /// Create a successful test result with details
    pub fn success_with_details(message: &str, details: &str) -> Self {
        Self {
            passed: true,
            message: message.to_string(),
            execution_time: Duration::from_secs(0),
            details: Some(details.to_string()),
        }
    }

    /// Create a failed test result
    pub fn failure(message: &str) -> Self {
        Self {
            passed: false,
            message: message.to_string(),
            execution_time: Duration::from_secs(0),
            details: None,
        }
    }

    /// Create a failed test result with details
    pub fn failure_with_details(message: &str, details: &str) -> Self {
        Self {
            passed: false,
            message: message.to_string(),
            execution_time: Duration::from_secs(0),
            details: Some(details.to_string()),
        }
    }

    /// Set execution time
    pub fn with_time(mut self, time: Duration) -> Self {
        self.execution_time = time;
        self
    }
}

impl TestRunner {
    /// Create a new test runner
    pub fn new() -> Self {
        Self {
            suites: Vec::new(),
            verbose: false,
            parallel: false,
        }
    }

    /// Enable verbose output
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Enable parallel execution
    pub fn parallel(mut self) -> Self {
        self.parallel = true;
        self
    }

    /// Add a test suite
    pub fn add_suite(mut self, suite: TestSuite) -> Self {
        self.suites.push(suite);
        self
    }

    /// Run all test suites
    pub fn run(&self) -> TestRunResults {
        let start_time = Instant::now();
        let mut results = TestRunResults::new();

        for suite in &self.suites {
            if self.verbose {
                println!("Running test suite: {}", suite.name);
            }

            // Run setup
            if let Some(setup) = &suite.setup {
                setup();
            }

            // Run tests
            for test in &suite.tests {
                let test_start = Instant::now();

                if self.verbose {
                    print!("  Running test: {}... ", test.name);
                }

                let mut result = (test.test_fn)();
                result.execution_time = test_start.elapsed();

                // Handle expected failures
                if test.expected_to_fail {
                    result.passed = !result.passed;
                    if result.passed {
                        result.message = format!("Expected failure: {}", result.message);
                    }
                }

                if self.verbose {
                    if result.passed {
                        println!("PASS ({:?})", result.execution_time);
                    } else {
                        println!("FAIL ({:?}): {}", result.execution_time, result.message);
                    }
                }

                results.add_result(&suite.name, &test.name, result);
            }

            // Run teardown
            if let Some(teardown) = &suite.teardown {
                teardown();
            }

            if self.verbose {
                println!();
            }
        }

        results.total_time = start_time.elapsed();
        results
    }
}

/// Results of test run
pub struct TestRunResults {
    pub suite_results: HashMap<String, HashMap<String, TestResult>>,
    pub total_time: Duration,
}

impl TestRunResults {
    /// Create new results
    pub fn new() -> Self {
        Self {
            suite_results: HashMap::new(),
            total_time: Duration::from_secs(0),
        }
    }

    /// Add a test result
    pub fn add_result(&mut self, suite_name: &str, test_name: &str, result: TestResult) {
        self.suite_results
            .entry(suite_name.to_string())
            .or_insert_with(HashMap::new)
            .insert(test_name.to_string(), result);
    }

    /// Get total test count
    pub fn total_tests(&self) -> usize {
        self.suite_results.values().map(|tests| tests.len()).sum()
    }

    /// Get passed test count
    pub fn passed_tests(&self) -> usize {
        self.suite_results
            .values()
            .flat_map(|tests| tests.values())
            .filter(|result| result.passed)
            .count()
    }

    /// Get failed test count
    pub fn failed_tests(&self) -> usize {
        self.total_tests() - self.passed_tests()
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed_tests() == 0
    }

    /// Print summary
    pub fn print_summary(&self) {
        println!("\n=== Test Summary ===");
        println!("Total tests: {}", self.total_tests());
        println!("Passed: {}", self.passed_tests());
        println!("Failed: {}", self.failed_tests());
        println!("Total time: {:?}", self.total_time);

        if !self.all_passed() {
            println!("\n=== Failed Tests ===");
            for (suite_name, tests) in &self.suite_results {
                for (test_name, result) in tests {
                    if !result.passed {
                        println!("{}.{}: {}", suite_name, test_name, result.message);
                        if let Some(details) = &result.details {
                            println!("  Details: {}", details);
                        }
                    }
                }
            }
        }
    }
}

/// Convenience macro for creating test results
macro_rules! test_result {
    (pass, $msg:expr) => {
        TestResult::success($msg)
    };
    (pass, $msg:expr, $details:expr) => {
        TestResult::success_with_details($msg, $details)
    };
    (fail, $msg:expr) => {
        TestResult::failure($msg)
    };
    (fail, $msg:expr, $details:expr) => {
        TestResult::failure_with_details($msg, $details)
    };
}
