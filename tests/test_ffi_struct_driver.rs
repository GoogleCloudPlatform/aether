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

use aether::pipeline::{CompilationPipeline, CompileOptions};
use std::path::PathBuf;

fn main() {
    // Create compilation options
    let mut options = CompileOptions::default();
    options.output = Some(PathBuf::from("test_ffi_struct_simple"));
    options.keep_intermediates = true;

    // Create pipeline
    let mut pipeline = CompilationPipeline::new(options);

    // Compile the test file
    match pipeline.compile_files(&[PathBuf::from("test_ffi_struct_simple.aether")]) {
        Ok(result) => {
            println!("Compilation successful!");
            println!("Output: {:?}", result.executable_path);
        }
        Err(e) => {
            eprintln!("Compilation failed: {:?}", e);
            std::process::exit(1);
        }
    }
}
