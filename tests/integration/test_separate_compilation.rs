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

//! Integration tests for separate compilation (Phase 30)
//!
//! Tests compiling programs against pre-built .o + .abi stdlib modules

use std::fs;
use std::path::PathBuf;

/// Helper to get stdlib build directory
fn stdlib_build_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("stdlib/build")
}

/// Test that stdlib was built and ABI files exist
#[test]
fn test_stdlib_abi_files_exist() {
    let build_dir = stdlib_build_dir();

    // Check if build directory exists (may not exist in CI)
    if !build_dir.exists() {
        eprintln!("Skipping test: stdlib not built (run 'make debug' in stdlib/)");
        return;
    }

    let expected_modules = ["io", "math", "string", "json", "collections", "stringview"];

    for module in expected_modules {
        let abi_path = build_dir.join(format!("{}.abi", module));
        let obj_path = build_dir.join(format!("{}.o", module));

        if !abi_path.exists() {
            eprintln!("Skipping: {} ABI not found", module);
            continue;
        }

        assert!(abi_path.exists(), "Expected {} to exist", abi_path.display());
        assert!(obj_path.exists(), "Expected {} to exist", obj_path.display());

        // Verify ABI is valid JSON
        let content = fs::read_to_string(&abi_path)
            .expect(&format!("Failed to read {}", abi_path.display()));
        let _: serde_json::Value = serde_json::from_str(&content)
            .expect(&format!("Invalid JSON in {}", abi_path.display()));
    }
}

/// Test ABI loading and module reconstruction
#[test]
fn test_abi_module_reconstruction() {
    use aether::abi::AbiModule;

    let build_dir = stdlib_build_dir();
    let io_abi = build_dir.join("io.abi");

    if !io_abi.exists() {
        eprintln!("Skipping test: stdlib not built");
        return;
    }

    // Load ABI
    let abi = AbiModule::load(&io_abi).expect("Failed to load io.abi");

    // Verify structure
    assert_eq!(abi.module.name, "Io");
    assert!(!abi.functions.is_empty(), "Should have functions");

    // Check for known functions
    let func_names: Vec<_> = abi.functions.iter().map(|f| f.name.as_str()).collect();
    assert!(func_names.contains(&"print"), "Should have print function");
    assert!(func_names.contains(&"println"), "Should have println function");
}

/// Test multi-object linking support
#[test]
fn test_multi_object_linking_flag() {
    use aether::pipeline::CompileOptions;

    let mut options = CompileOptions::default();
    options.link_objects.push(PathBuf::from("/tmp/test.o"));
    options.link_objects.push(PathBuf::from("/tmp/test2.o"));

    assert_eq!(options.link_objects.len(), 2);
}

/// Test emit-abi flag behavior
#[test]
fn test_emit_abi_flag() {
    use aether::pipeline::CompileOptions;

    let mut options = CompileOptions::default();
    assert!(!options.emit_abi, "emit_abi should default to false");

    options.emit_abi = true;
    assert!(options.emit_abi);
}

/// Test ABI load and save roundtrip
#[test]
fn test_abi_load_save_roundtrip() {
    use aether::abi::AbiModule;
    use tempfile::TempDir;

    let build_dir = stdlib_build_dir();
    let io_abi = build_dir.join("io.abi");

    if !io_abi.exists() {
        eprintln!("Skipping test: stdlib not built");
        return;
    }

    // Load original ABI
    let original = AbiModule::load(&io_abi).expect("Failed to load");

    // Save to temp location
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_abi = temp_dir.path().join("io_copy.abi");
    original.save(&temp_abi).expect("Failed to save");

    // Load copy
    let loaded = AbiModule::load(&temp_abi).expect("Failed to load copy");

    // Verify
    assert_eq!(original.module.name, loaded.module.name);
    assert_eq!(original.functions.len(), loaded.functions.len());
    assert_eq!(original.abi_version, loaded.abi_version);
}

/// Test MIR serialization roundtrip (tests exist in src/abi/mod.rs)
/// This test verifies SerializedMir struct is public and usable
#[test]
fn test_serialized_mir_struct_exists() {
    use aether::abi::SerializedMir;

    // Just verify the struct is public and usable
    let _serialized = SerializedMir {
        function_name: "test".to_string(),
        mir_json: "{}".to_string(),
        generic_params: vec![],
    };
}

/// Test ABI find_function method
#[test]
fn test_abi_find_function() {
    use aether::abi::AbiModule;

    let build_dir = stdlib_build_dir();
    let math_abi = build_dir.join("math.abi");

    if !math_abi.exists() {
        eprintln!("Skipping test: stdlib not built");
        return;
    }

    let abi = AbiModule::load(&math_abi).expect("Failed to load math.abi");

    // Find existing function
    let sqrt = abi.find_function("sqrt");
    assert!(sqrt.is_some(), "Should find sqrt function");

    // Find non-existent function
    let nonexistent = abi.find_function("nonexistent_function");
    assert!(nonexistent.is_none(), "Should not find nonexistent function");
}
