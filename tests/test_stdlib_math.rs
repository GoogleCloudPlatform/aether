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

//! Tests for math operations parsing and analysis
//!
//! Tests that math-related code can be parsed and analyzed.

use aether::lexer::v2::Lexer;
use aether::parser::v2::Parser;
use aether::semantic::SemanticAnalyzer;

#[test]
fn test_safe_arithmetic() {
    let source = r#"
module test_math {
    func safe_add(a: Int, b: Int) -> Int {
        return {a + b};
    }

    func safe_multiply(a: Int, b: Int) -> Int {
        return {a * b};
    }

    func main() -> Int {
        let sum: Int = safe_add(10, 20);
        let product: Int = safe_multiply(5, 6);
        return {sum + product};
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_sqrt_and_pow() {
    let source = r#"
module test_power {
    func power(base: Float64, exp: Int) -> Float64 {
        var result: Float64 = 1.0;
        var i: Int = 0;
        while {i < exp} {
            result = {result * base};
            i = {i + 1};
        }
        return result;
    }

    func main() -> Int {
        let p: Float64 = power(2.0, 10);
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

    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_trigonometric_functions() {
    let source = r#"
module test_trig {
    // Trig functions would be implemented via extern
    func sin_approx(x: Float64) -> Float64 {
        return x;  // Placeholder
    }

    func cos_approx(x: Float64) -> Float64 {
        return {1.0 - x};  // Placeholder
    }

    func main() -> Int {
        let s: Float64 = sin_approx(0.0);
        let c: Float64 = cos_approx(0.0);
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

    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}

#[test]
fn test_min_max_abs() {
    let source = r#"
module test_minmax {
    func min_int(a: Int, b: Int) -> Int {
        if {a < b} {
            return a;
        } else {
            return b;
        }
    }

    func max_int(a: Int, b: Int) -> Int {
        if {a > b} {
            return a;
        } else {
            return b;
        }
    }

    func abs_int(x: Int) -> Int {
        if {x < 0} {
            return {0 - x};
        } else {
            return x;
        }
    }

    func main() -> Int {
        let m: Int = min_int(5, 10);
        let n: Int = max_int(5, 10);
        let a: Int = abs_int({0 - 42});
        return {m + n + a};
    }
}
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parsing failed");

    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze_program(&program);

    assert!(
        result.is_ok(),
        "Semantic analysis failed: {:?}",
        result.err()
    );
}
