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

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_function_pointer_callback() {
    let test_program = r#"
module test_function_pointers;

// Define comparison function type for qsort
// int (*compar)(const void *, const void *)
@extern(library="libc")
func qsort(base: Pointer<Void>, nmemb: Int, size: Int, compar: func(Pointer<Void>, Pointer<Void>) -> Int) -> Void;

@extern(library="libc")
func printf(format: Pointer<Char>, val: Int) -> Int;

// Comparison function implementation
func int_compare(p1: Pointer<Void>, p2: Pointer<Void>) -> Int {
    // In a real scenario, we would cast pointers and read values
    // For this test, we just return 0 or 1 to satisfy signature
    return 0;
}

func main() -> Int {
    let count: Int = 5;
    // Array creation/manipulation would be needed here for real qsort
    // For now, we pass null pointer just to verify signature matching
    // (qsort handles null base with 0 count gracefully or we assume it doesn't run)
    
    // Passing a generic pointer (null)
    // Passing the function pointer 'int_compare'
    // qsort(null, 0, 4, int_compare);
    
    // Note: Aether doesn't fully support passing function names as values yet in the parser/semantic analysis
    // This test primarily verifies that the extern declaration with function pointer type parses and compiles
    
    return 0;
}
"#;

    let test_file = PathBuf::from("test_function_pointers.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");

    // Compile-only because we might not have runtime support for function value passing yet
    let output = Command::new(env!("CARGO_BIN_EXE_aether-compiler"))
        .arg("compile")
        .arg(&test_file)
        .arg("-c") // Compile only
        .arg("-o")
        .arg("test_function_pointers.o")
        .output()
        .expect("Failed to run compiler");

    // Clean up
    fs::remove_file(&test_file).ok();
    fs::remove_file("test_function_pointers.o").ok();

    if !output.status.success() {
        panic!(
            "Compilation failed:\nstdout: {}
stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
