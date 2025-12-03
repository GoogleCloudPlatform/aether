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
fn test_starling_ffi_simulation() {
    let test_dir = PathBuf::from("tests/ffi_starling");
    let c_file = test_dir.join("starling_mock.c");
    let aether_file = test_dir.join("verify_starling.aether");
    let output_bin = test_dir.join("verify_starling");

    // 1. Compile C Mock Library
    // We create a dylib or just link it? 
    // Easier to compile C file to .o and link it with aether compiler if supported,
    // OR compile C file to a shared library.
    
    let lib_ext = if cfg!(target_os = "macos") { "dylib" } else { "so" };
    let lib_name = format!("libstarling_mock.{}", lib_ext);
    let lib_path = test_dir.join(&lib_name);

    let cc_status = Command::new("cc")
        .arg("-shared")
        .arg("-fPIC")
        .arg("-o")
        .arg(&lib_path)
        .arg(&c_file)
        .status()
        .expect("Failed to run C compiler");

    assert!(cc_status.success(), "Failed to compile starling_mock.c");

    // 2. Compile Aether Program
    // We need to tell the compiler where to find the library.
    // Assuming `aether compile` supports `-L` and `-l`.
    
    let compiler_bin = env!("CARGO_BIN_EXE_aether-compiler");
    let compile_output = Command::new(compiler_bin)
        .arg("compile")
        .arg(&aether_file)
        .arg("-o")
        .arg(&output_bin)
        .arg("-L")
        .arg(&test_dir) // Add test dir to library search path
        .arg("-l")
        .arg("starling_mock") // Link against starling_mock
        .output()
        .expect("Failed to run aether compiler");

    if !compile_output.status.success() {
         panic!(
            "Aether Compilation failed:\nstdout: {}
stderr: {}",
            String::from_utf8_lossy(&compile_output.stdout),
            String::from_utf8_lossy(&compile_output.stderr)
        );
    }

    // 3. Run the binary
    // Need to set LD_LIBRARY_PATH (or DYLD_LIBRARY_PATH on macOS)
    let mut run_cmd = Command::new(&output_bin);
    
    if cfg!(target_os = "macos") {
        run_cmd.env("DYLD_LIBRARY_PATH", &test_dir);
    } else {
        run_cmd.env("LD_LIBRARY_PATH", &test_dir);
    }

    let run_output = run_cmd.output().expect("Failed to run generated binary");

    if !run_output.status.success() {
         panic!(
            "Execution failed (Exit Code: {:?}):\nstdout: {}
stderr: {}",
            run_output.status.code(),
            String::from_utf8_lossy(&run_output.stdout),
            String::from_utf8_lossy(&run_output.stderr)
        );
    }
    
    let stdout = String::from_utf8_lossy(&run_output.stdout);
    println!("Output:\n{}", stdout);
    
    // Verify C mock was actually called
    assert!(stdout.contains("[C] starling_init called"));
    assert!(stdout.contains("[C] starling_tokenize called"));
    assert!(stdout.contains("[C] starling_generate called"));
}
