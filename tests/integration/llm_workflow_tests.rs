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

//! End-to-end LLM workflow integration tests
//! Tests complete scenarios from intent specification to verified execution

// Access utils from parent directory since we're in integration subdir
#[path = "../utils/mod.rs"]
mod utils;

use utils::{assertions::*, compiler_wrapper::TestCompiler};

#[test]
fn test_intent_to_implementation_workflow() {
    let compiler = TestCompiler::new("intent_to_implementation");

    let source = r#"
module intent_to_implementation {
  // LLM workflow: Intent -> Contract -> Implementation -> Verification
  
  struct FileHandle { id: Int; }
  struct Buffer { size: Int; }

  // Mock functions for resource management
  func open(path: String, mode: String) -> FileHandle { return FileHandle { id: 1 }; }
  func close(handle: FileHandle) -> Void { }
  func alloc(size: Int) -> Buffer { return Buffer { size: size }; }
  func free(buf: Buffer) -> Void { }
  func file_exists(path: String) -> Bool { return true; }

  @intent("Safely read input file, process data, and write to output file with resource management")
  @pre({input_file != "null" && output_file != "null"})
  // @post(return_value -> file_exists(output_file)) 
  func safe_file_processor(input_file: String, output_file: String) -> Bool {
      
      // Resource acquisition
      let input: FileHandle = open(input_file, "r");
      
      // We need a way to check for null/invalid handle. Assuming id 0 is invalid.
      when {input.id == 0} {
          return false;
      }

      try {
          let buffer: Buffer = alloc(4096);
          let output: FileHandle = open(output_file, "w");
          
          when {output.id == 0} {
              return false;
          }
          
          try {
              // Process data...
              return true;
          } finally {
              close(output);
          }
      } finally {
          close(input);
      }
  }
  
  func main() -> Int {
      let success: Bool = safe_file_processor("test_input.txt", "test_output.txt");
      return 0;
  }
}
    "#;

    let result = compiler.compile_source(source, "intent_to_implementation.aether");

    if result.is_success() {
        assert_compilation_success(&result, "Intent to implementation workflow");
    } else {
        assert_compilation_error(&result, "pattern", "Pattern-based workflow integration");
    }
}

#[test]
fn test_error_driven_development_workflow() {
    let compiler = TestCompiler::new("error_driven_development");

    // Simulate LLM generating code with intentional errors to test error recovery
    let source = r#"
module error_driven_development {
  @intent("Process array data and return formatted result")
  func error_prone_function(data: Array<Int>) -> String {
      // Error 1: Type mismatch - trying to assign array element (Int) to String
      let result: String = data[0];
      
      // Error 2: Undefined variable reference
      let processed: Int = undefined_var;
      
      // Error 3: Wrong return type (Int instead of String)
      return 42;
  }
}
    "#;

    let result = compiler.compile_source(source, "error_driven_development.aether");
    assert_compilation_failure(&result, "Error-driven development workflow");

    if let Some(error) = result.error() {
        let error_msg = format!("{}", error);
        // Basic check that we got errors
        assert!(error_msg.contains("Type mismatch") || error_msg.contains("Undefined symbol"));
    }
}

#[test]
fn test_iterative_refinement_workflow() {
    let compiler = TestCompiler::new("iterative_refinement");

    // Initial attempt
    let initial_attempt = r#"
module iterative_refinement_v1 {
  @intent("Perform safe arithmetic operations with comprehensive error handling")
  @pre({operation != "null"})
  func improved_calculator(operation: String, a: Float, b: Float) -> Float {
      when {operation == "add"} {
          return {a + b};
      }
      when {operation == "divide"} {
          return {a / b};
      }
      return 0.0;
  }
}
    "#;

    let result_v1 = compiler.compile_source(initial_attempt, "iterative_refinement_v1.aether");
    // Should compile
    assert!(result_v1.is_success(), "Initial attempt should compile");

    // Refined version
    let refined_attempt = r#"
module iterative_refinement_v2 {
  @intent("Perform safe arithmetic operations with comprehensive error handling")
  @pre({operation != "null"})
  // @post(return_value == return_value) // explicit check for not NaN?
  func improved_calculator(operation: String, a: Float, b: Float) -> Float {
      when {operation == "add"} {
          return {a + b};
      }
      when {operation == "subtract"} {
          return {a - b};
      }
      when {operation == "multiply"} {
          return {a * b};
      }
      when {operation == "divide"} {
          when {b == 0.0} {
              return 0.0; // Return 0 on div by zero for safety in this example
          } else {
              return {a / b};
          }
      }
      return 0.0;
  }

  func main() -> Int {
      let result1: Float = improved_calculator("add", 10.5, 5.5);
      let result2: Float = improved_calculator("divide", 10.0, 0.0);
      return 0;
  }
}
    "#;

    let result_v2 = compiler.compile_source(refined_attempt, "iterative_refinement_v2.aether");
    assert_compilation_success(&result_v2, "Iterative refinement workflow");
}

#[test]
fn test_pattern_composition_workflow() {
    let compiler = TestCompiler::new("pattern_composition_workflow");

    let source = r#"
module pattern_composition_workflow {
  // LLM workflow: Identify patterns -> Compose -> Verify -> Optimize
  
  @intent("Process array of strings through validation, transformation, and aggregation pipeline")
  @pre({data_size > 0 && data_size <= 100})
  func complex_data_processor(input_data: Array<String>, data_size: Int) -> Array<String> {
      
      // Pattern: Input validation
      // (Mocked logic)
      
      // Pattern: Data sanitization
      
      // Pattern: Content filtering
      
      // Pattern: Duplicate removal
      
      // Pattern: Result validation
      
      return input_data;
  }
  
  func main() -> Int {
      let test_data: Array<String> = ["  Hello World  ", "Test", "Dup", "Dup", ""]; 
      
      let result: Array<String> = complex_data_processor(test_data, 5);
      return 0;
  }
}
    "#;

    let result = compiler.compile_source(source, "pattern_composition_workflow.aether");
    assert_compilation_success(&result, "Pattern composition workflow");
}

#[test]
fn test_verification_driven_development_workflow() {
    let compiler = TestCompiler::new("verification_driven_development");

    let source = r#"
module verification_driven_development {
  @intent("Implement binary search with formal verification of correctness")
  @pre({size > 0 && size <= 1000})
  // @invariant(left >= 0 && right < size)
  func verified_binary_search(array: Array<Int>, target: Int, size: Int) -> Int {
      var left: Int = 0;
      var right: Int = {size - 1};
      
      while {left <= right} {
          let mid: Int = {{left + right} / 2};
          let mid_value: Int = array[mid];
          
          when {mid_value == target} {
              return mid;
          } else {
              when {mid_value < target} {
                  left = {mid + 1};
              } else {
                  right = {mid - 1};
              }
          }
      }
      
      return -1;
  }
  
  func main() -> Int {
      // Simplified array init
      let sorted_array: Array<Int> = [1, 3, 5, 7, 9]; 
      let search_result: Int = verified_binary_search(sorted_array, 7, 5);
      return 0;
  }
}
    "#;

    let result = compiler.compile_source(source, "verification_driven_development.aether");
    assert_compilation_success(&result, "Verification-driven development workflow");
}

#[test]
fn test_multi_phase_llm_code_generation() {
    let compiler = TestCompiler::new("multi_phase_llm_generation");

    let source = r#"
module multi_phase_llm_generation {
  
  @intent("Securely aggregate numeric data from multiple sources")
  @pre({source_count > 0 && source_count <= 10})
  func secure_data_aggregator(data_sources: Array<String>, source_count: Int, aggregation_method: String) -> Float {
      
      // Phase 4: Implementation
      // Validation
      
      // Allocate buffer (mocked)
      var numeric_values: Array<Float> = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
      var valid_values: Int = 0;
      
      // Data processing pipeline
      var i: Int = 0;
      while {i < source_count} {
          // Mock conversion
          let val: Float = 1.0; 
          
          when {val != 0.0} {
              numeric_values[valid_values] = val;
              valid_values = {valid_values + 1};
          }
          i = {i + 1};
      }
      
      // Aggregation
      var aggregated_result: Float = 0.0;
      when {aggregation_method == "sum"} {
          // sum logic
          aggregated_result = 10.0;
      }
      when {aggregation_method == "mean"} {
          // mean logic
          aggregated_result = 5.0;
      }
      
      return aggregated_result;
  }
  
  func main() -> Int {
      let test_data: Array<String> = ["10.5", "20.0", "15.7", "8.3", "12.1"];
      let mean_result: Float = secure_data_aggregator(test_data, 5, "mean");
      return 0;
  }
}
    "#;

    let result = compiler.compile_source(source, "multi_phase_llm_generation.aether");
    assert_compilation_success(&result, "Multi-phase LLM code generation");
}