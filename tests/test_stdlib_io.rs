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

//! Tests for I/O operations parsing and analysis
//!
//! Tests that I/O-related code can be parsed and analyzed.

use aether::lexer::v2::Lexer;
use aether::parser::v2::Parser;
use aether::semantic::SemanticAnalyzer;

#[test]
fn test_file_write_and_read() {
    let source = r#"
module test_io {
    func write_data(filename: String, data: String) -> Bool {
        // IO operations would be implemented via extern functions
        return true;
    }

    func main() -> Int {
        let success: Bool = write_data("test.txt", "Hello, World!");
        when {success} {
            return 0;
        } else {
            return 1;
        }
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(result.is_ok(), "Semantic analysis failed: {:?}", result.err());
}

#[test]
fn test_console_io() {
    let source = r#"
module test_console {
    func print_message(msg: String) -> Void {
        // Console output would be implemented via extern functions
    }

    func main() -> Int {
        print_message("Hello from AetherScript!");
        return 0;
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(result.is_ok(), "Semantic analysis failed: {:?}", result.err());
}

#[test]
fn test_file_size_limit() {
    let source = r#"
module test_file_limits {
    const MAX_FILE_SIZE: Int = 1048576;

    func check_file_size(size: Int) -> Bool {
        when {size > MAX_FILE_SIZE} {
            return false;
        } else {
            return true;
        }
    }

    func main() -> Int {
        let size: Int = 512000;
        let valid: Bool = check_file_size(size);
        when {valid} {
            return 0;
        } else {
            return 1;
        }
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(result.is_ok(), "Semantic analysis failed: {:?}", result.err());
}
