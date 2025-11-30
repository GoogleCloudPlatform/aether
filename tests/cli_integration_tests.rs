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
use std::path::Path;
use std::process::Command;

/// Helper function to run the aether CLI and capture output
fn run_aether_cli(args: &[&str]) -> (String, String, i32) {
    let output = Command::new("cargo")
        .args(["run", "--"])
        .args(args)
        .output()
        .expect("Failed to execute aether CLI");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    // On Unix systems, exit codes might be transformed. If we get -1, try to extract from status
    let final_exit_code = if exit_code == -1 {
        if output.status.success() {
            0
        } else {
            1
        }
    } else {
        exit_code
    };

    (stdout, stderr, final_exit_code)
}

/// Helper function to get test fixture path
fn fixture_path(filename: &str) -> String {
    format!("tests/fixtures/{}", filename)
}

#[test]
fn test_cli_help_command() {
    let (stdout, _stderr, exit_code) = run_aether_cli(&["--help"]);

    assert_eq!(exit_code, 0);
    assert!(stdout.contains("Compiler for the AetherScript programming language"));
    assert!(stdout.contains("compile"));
    assert!(stdout.contains("check"));
    assert!(stdout.contains("ast"));
    assert!(stdout.contains("tokens"));
}

#[test]
fn test_cli_version_flag() {
    let (stdout, _stderr, exit_code) = run_aether_cli(&["--version"]);

    assert_eq!(exit_code, 0);
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_cli_check_valid_file() {
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["check", &fixture_path("simple_module.aether")]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stdout.contains("Type checking passed"));
    assert!(stdout.contains("Files passed: 1"));
}

#[test]
fn test_cli_check_invalid_file() {
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["check", &fixture_path("type_errors.aether")]);

    assert_eq!(exit_code, 1);
    assert!(stdout.contains("Type checking failed"));
    assert!(stderr.contains("Type mismatch") || stdout.contains("Type mismatch"));
}

#[test]
fn test_cli_check_nonexistent_file() {
    let (stdout, stderr, exit_code) = run_aether_cli(&["check", "nonexistent_file.aether"]);

    assert_eq!(exit_code, 1);
    assert!(stderr.contains("not found") || stdout.contains("not found"));
}

#[test]
fn test_cli_check_multiple_files() {
    let (stdout, stderr, exit_code) = run_aether_cli(&[
        "check",
        &fixture_path("simple_module.aether"),
        &fixture_path("empty_module.aether"),
    ]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stdout.contains("Files passed: 2"));
    assert!(stdout.contains("Total errors: 0"));
}

#[test]
fn test_cli_check_mixed_files() {
    let (stdout, _stderr, exit_code) = run_aether_cli(&[
        "check",
        &fixture_path("simple_module.aether"),
        &fixture_path("type_errors.aether"),
    ]);

    assert_eq!(exit_code, 1);
    assert!(stdout.contains("Files passed: 1"));
    assert!(stdout.contains("Files with errors: 1"));
}

#[test]
fn test_cli_check_verbose_mode() {
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["check", &fixture_path("simple_module.aether"), "--verbose"]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stderr.contains("[VERBOSE]") || stdout.contains("VERBOSE"));
    assert!(stderr.contains("Type checking 1 file(s)") || stdout.contains("Processing:"));
}

#[test]
fn test_cli_check_debug_mode() {
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["check", &fixture_path("simple_module.aether"), "--debug"]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stderr.contains("[DEBUG]") || stderr.contains("DEBUG"));
}

#[test]
fn test_cli_ast_command() {
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["ast", &fixture_path("simple_module.aether")]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stdout.contains("Program {"));
    assert!(stdout.contains("Module 'simple_module'"));
    assert!(stdout.contains("const VERSION: String"));
    assert!(stdout.contains("const MAX_ITEMS: Int"));
}

#[test]
fn test_cli_ast_output_to_directory() {
    let output_dir = "tests/output_ast";

    // Clean up any existing output
    let _ = fs::remove_dir_all(output_dir);

    let (stdout, stderr, exit_code) = run_aether_cli(&[
        "ast",
        &fixture_path("simple_module.aether"),
        "--output",
        output_dir,
    ]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);

    let output_file = Path::new(output_dir).join("simple_module.ast");

    // Add a small delay to ensure file system operations complete
    std::thread::sleep(std::time::Duration::from_millis(100));

    assert!(output_file.exists(), "AST output file should exist");

    let ast_content = fs::read_to_string(&output_file).unwrap();
    assert!(ast_content.contains("Program {"));
    assert!(ast_content.contains("Module 'simple_module'"));

    // Clean up
    let _ = fs::remove_dir_all(output_dir);
}

#[test]
fn test_cli_tokens_command() {
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["tokens", &fixture_path("simple_module.aether")]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stdout.contains("Tokens for"));
    assert!(stdout.contains("LeftBrace"));
    assert!(stdout.contains("Keyword(Module)"));
    assert!(stdout.contains("Identifier(\"simple_module\")"));
    assert!(stdout.contains("RightBrace"));
}

#[test]
fn test_cli_tokens_output_to_directory() {
    let output_dir = "tests/output_tokens";

    // Clean up any existing output
    let _ = fs::remove_dir_all(output_dir);

    let (stdout, stderr, exit_code) = run_aether_cli(&[
        "tokens",
        &fixture_path("simple_module.aether"),
        "--output",
        output_dir,
    ]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);

    let output_file = Path::new(output_dir).join("simple_module.tokens");

    // Add a small delay to ensure file system operations complete
    std::thread::sleep(std::time::Duration::from_millis(100));

    assert!(output_file.exists(), "Tokens output file should exist");

    let tokens_content = fs::read_to_string(&output_file).unwrap();
    assert!(tokens_content.contains("LeftBrace"));
    assert!(tokens_content.contains("Keyword(Module)"));

    // Clean up
    let _ = fs::remove_dir_all(output_dir);
}

#[test]
fn test_cli_compile_command() {
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["compile", &fixture_path("simple_module.aether")]);

    // Since compilation is not yet implemented, we expect success but with placeholder message
    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stdout.contains("Compilation completed successfully"));
}

#[test]
fn test_cli_compile_with_output() {
    let (stdout, stderr, exit_code) = run_aether_cli(&[
        "compile",
        &fixture_path("simple_module.aether"),
        "--output",
        "test_output",
    ]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stdout.contains("Compilation completed successfully"));
}

#[test]
fn test_cli_error_exit_codes() {
    // Test with nonexistent file
    let (_stdout, _stderr, exit_code) = run_aether_cli(&["check", "does_not_exist.aether"]);
    assert_eq!(exit_code, 1);

    // Test with type error file
    let (_stdout, _stderr, exit_code) =
        run_aether_cli(&["check", &fixture_path("type_errors.aether")]);
    assert_eq!(exit_code, 1);

    // Test with syntax error file
    let (_stdout, _stderr, exit_code) =
        run_aether_cli(&["ast", &fixture_path("syntax_errors.aether")]);
    assert_eq!(exit_code, 1);
}

#[test]
fn test_cli_global_verbose_flag() {
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["--verbose", "check", &fixture_path("simple_module.aether")]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stderr.contains("[VERBOSE]") || stdout.contains("VERBOSE"));
}

#[test]
fn test_cli_global_debug_flag() {
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["--debug", "check", &fixture_path("simple_module.aether")]);

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stderr.contains("[DEBUG]") || stderr.contains("DEBUG"));
}

#[test]
fn test_cli_no_subcommand() {
    let (_stdout, stderr, exit_code) = run_aether_cli(&[]);

    assert_eq!(exit_code, 1);
    assert!(stderr.contains("No subcommand provided"));
}

#[test]
fn test_cli_invalid_subcommand() {
    let (_stdout, stderr, exit_code) = run_aether_cli(&["invalid_command"]);

    // clap should handle this and provide help
    assert_ne!(exit_code, 0);
}

#[test]
fn test_performance_large_file() {
    use std::time::Instant;

    let start = Instant::now();
    let (stdout, stderr, exit_code) =
        run_aether_cli(&["check", &fixture_path("large_file.aether")]);
    let duration = start.elapsed();

    assert_eq!(exit_code, 0, "Stderr: {}", stderr);
    assert!(stdout.contains("Type checking passed"));
    assert!(stdout.contains("Files passed: 1"));

    // Performance assertion: should complete within 5 seconds
    assert!(
        duration.as_secs() < 5,
        "Large file processing took too long: {:?}",
        duration
    );
}
