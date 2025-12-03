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
fn test_llvm_c_api_usage() {
    let test_program = r#"
module test_llvm_c_api;

// Simulation of LLVM-C API types (Opaque Pointers)
// typedef struct LLVMOpaqueContext *LLVMContextRef;
// typedef struct LLVMOpaqueModule *LLVMModuleRef;
// typedef struct LLVMOpaqueBuilder *LLVMBuilderRef;

// In Aether, we represent opaque structs via named types that are only declared (not defined)
// However, Aether currently requires struct definitions for Named types in FFI to ensure compatibility.
// For opaque pointers, we can use Pointer<Void> or define empty structs.
// Defining empty structs is safer as it provides nominal typing.

struct LLVMContext {}
struct LLVMM {}
struct LLVMBuilder {}
struct LLVMType {}
struct LLVMValue {}

// LLVM-C API Functions
@extern(library="LLVM")
func LLVMContextCreate() -> Pointer<LLVMContext>;

@extern(library="LLVM")
func LLVMContextDispose(C: Pointer<LLVMContext>) -> Void;

/*
// TODO: Investigate why "Keyword(Module)" error occurs with these declarations
@extern(library="LLVM")
func LLVMMCreateWithName(ModID: Pointer<Char>) -> Pointer<LLVMM>;

@extern(library="LLVM")
func LLVMMCreateWithNameInContext(ModID: Pointer<Char>, C: Pointer<LLVMContext>) -> Pointer<LLVMM>;

@extern(library="LLVM")
func LLVMDisposeModule(M: Pointer<LLVMM>) -> Void;
*/

@extern(library="LLVM")
func LLVMCreateBuilderInContext(C: Pointer<LLVMContext>) -> Pointer<LLVMBuilder>;

@extern(library="LLVM")
func LLVMDisposeBuilder(Builder: Pointer<LLVMBuilder>) -> Void;

// Using the API
func main() -> Int {
    let context: Pointer<LLVMContext> = LLVMContextCreate();
    //let module: Pointer<LLVMM> = LLVMMCreateWithNameInContext("my_module", context);
    let builder: Pointer<LLVMBuilder> = LLVMCreateBuilderInContext(context);

    // ... do something with builder ...

    LLVMDisposeBuilder(builder);
    //LLVMDisposeModule(module);
    LLVMContextDispose(context);
    
    return 0;
}
"#;

    let test_file = PathBuf::from("test_llvm_c_api.aether");
    fs::write(&test_file, test_program).expect("Failed to write test file");
    
    println!("Test file content:\n{}", fs::read_to_string(&test_file).unwrap());

    // Compile-only
    let output = Command::new(env!("CARGO_BIN_EXE_aether-compiler"))
        .arg("compile")
        .arg(&test_file)
        .arg("-c")
        .arg("-o")
        .arg("test_llvm_c_api.o")
        .output()
        .expect("Failed to run compiler");

    // Clean up
    fs::remove_file(&test_file).ok();
    fs::remove_file("test_llvm_c_api.o").ok();

    if !output.status.success() {
        panic!(
            "Compilation failed:\nstdout: {}
stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
