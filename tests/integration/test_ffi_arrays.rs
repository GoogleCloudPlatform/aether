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
fn test_array_passing_to_c() {
    let test_program = r#"
module test_ffi_arrays;

@extern(library="clib")
func sum_array(arr: Array<Int>, len: Int) -> Int;

@extern(library="clib")
func process_buffer(buf: Pointer<Char>, size: Int) -> Void;

func main() -> Int {
    let numbers: Array<Int> = [1, 2, 3, 4, 5];
    
    // Pass array to C function
    // In C this would be: int sum_array(int64_t* arr, int64_t len);
    let sum: Int = sum_array(numbers, 5);
    
    return 0;
}
"#;

    let test_file = PathBuf::from("test_ffi_arrays.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");

    // Compile-only 
    let output = Command::new(env!("CARGO_BIN_EXE_aether-compiler"))
        .arg("compile")
        .arg(&test_file)
        .arg("-c") 
        .arg("-o")
        .arg("test_ffi_arrays.o")
        .output()
        .expect("Failed to run compiler");

    // Clean up
    fs::remove_file(&test_file).ok();
    fs::remove_file("test_ffi_arrays.o").ok();

    if !output.status.success() {
        panic!(
            "Compilation failed:\nstdout: {}
stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
