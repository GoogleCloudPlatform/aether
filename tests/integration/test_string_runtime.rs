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

//! Integration tests for string runtime functions

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_string_runtime_functions() {
    let test_program = r#"
module test_string_runtime {
    // External function declarations
    @extern(library="libc", variadic=true)
    func printf(format: String) -> Int;
    
    @extern(library="aether_runtime")
    func string_index_of(haystack: String, needle: String) -> Int;
    
    @extern(library="aether_runtime")
    func string_starts_with(str: String, prefix: String) -> Int;
    
    @extern(library="aether_runtime")
    func string_ends_with(str: String, suffix: String) -> Int;
    
    @extern(library="aether_runtime")
    func parse_float(str: String) -> Float;
    
    @extern(library="aether_runtime")
    func float_to_string(value: Float) -> String;
    
    func main() -> Int {
        // Test string_index_of
        let test_str: String = "Hello, World!";
        
        let index: Int = string_index_of(test_str, "World");
        
        // Expected: 7
        when {index != 7} {
            // printf("FAIL: string_index_of returned %d, expected 7\n", index);
            return 1;
        }
        
        // Test string_starts_with
        let starts: Int = string_starts_with(test_str, "Hello");
        
        // Expected: 1 (true)
        when {starts != 1} {
            // printf("FAIL: string_starts_with returned %d, expected 1\n", starts);
            return 1;
        }
        
        // Test string_ends_with
        let ends: Int = string_ends_with(test_str, "World!");
        
        // Expected: 1 (true)
        when {ends != 1} {
            // printf("FAIL: string_ends_with returned %d, expected 1\n", ends);
            return 1;
        }
        
        // Test parse_float
        let float_str: String = "3.14";
        
        let parsed: Float = parse_float(float_str);
        
        // Test float_to_string
        let float_back: String = float_to_string(parsed);
        
        // printf("All string runtime tests passed!\n");
        return 0;
    }
}
"#;

    // Write test program
    let test_file = PathBuf::from("test_string_runtime.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");

    // Compile the program
    let output = Command::new(env!("CARGO_BIN_EXE_aether-compiler"))
        .arg("compile")
        .arg(&test_file)
        .arg("-o")
        .arg("test_string_runtime")
        .output()
        .expect("Failed to run compiler");

    if !output.status.success() {
        panic!(
            "Compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Clean up
    fs::remove_file(test_file).ok();
    fs::remove_file("test_string_runtime").ok();
    fs::remove_file("test_string_runtime.o").ok();
}
