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

//! Test utilities for AetherScript integration tests

pub mod assertions;
pub mod compiler_wrapper;
pub mod test_runner;

use std::fs;
use std::path::{Path, PathBuf};

/// Helper to get test fixture path
pub fn fixture_path(filename: &str) -> PathBuf {
    PathBuf::from("tests").join("fixtures").join(filename)
}

/// Load test fixture content
pub fn load_fixture(filename: &str) -> String {
    let path = fixture_path(filename);
    fs::read_to_string(&path).expect(&format!("Failed to load fixture: {}", path.display()))
}

/// Create temporary directory for test outputs
pub fn create_temp_dir(test_name: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir().join("aether_tests").join(test_name);
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    temp_dir
}

/// Clean up test directory
pub fn cleanup_temp_dir(path: &Path) {
    if path.exists() {
        fs::remove_dir_all(path).ok();
    }
}

/// Create a test project with multiple files
pub fn create_test_project(project_name: &str, files: &[(&str, &str)]) -> PathBuf {
    let project_dir = create_temp_dir(project_name);

    for (filename, content) in files {
        let file_path = project_dir.join(filename);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create directory");
        }
        fs::write(&file_path, content).expect("Failed to write file");
    }

    project_dir
}

/// Generate unique test identifier
pub fn test_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("test_{}", timestamp)
}
