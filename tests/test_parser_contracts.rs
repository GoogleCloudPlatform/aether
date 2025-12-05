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

//! Tests for parsing functions with contract annotations

#[cfg(test)]
mod parser_contract_tests {
    use aether::lexer::v2::Lexer;
    use aether::parser::v2::Parser;

    #[test]
    fn test_parser_function_with_contracts() {
        let test_file_path = "tests/fixtures/function_with_contracts.aether";
        let source = std::fs::read_to_string(test_file_path).expect("Failed to read test file");

        let mut lexer = Lexer::new(&source, test_file_path.to_string());
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => panic!("Lexer error: {:?}", e),
        };

        let mut parser = Parser::new(tokens);
        let program = match parser.parse_program() {
            Ok(program) => program,
            Err(e) => panic!("Parser error: {:?}", e),
        };

        // Verify module was parsed
        assert_eq!(program.modules.len(), 1);
        let module = &program.modules[0];
        assert_eq!(module.name.name, "function_with_contracts");

        // Check that we have the test_division function
        assert_eq!(module.function_definitions.len(), 1);
        let function = &module.function_definitions[0];
        assert_eq!(function.name.name, "test_division");

        // Verify function has 2 parameters
        assert_eq!(function.parameters.len(), 2);
        assert_eq!(function.parameters[0].name.name, "x");
        assert_eq!(function.parameters[1].name.name, "y");

        println!("✓ Successfully parsed function with contracts");
    }

    #[test]
    fn test_parser_invalid_contracts() {
        let test_file_path = "tests/fixtures/invalid_contracts.aether";
        let source = std::fs::read_to_string(test_file_path).expect("Failed to read test file");

        let mut lexer = Lexer::new(&source, test_file_path.to_string());
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => panic!("Lexer error: {:?}", e),
        };

        let mut parser = Parser::new(tokens);
        let program = match parser.parse_program() {
            Ok(program) => program,
            Err(e) => panic!("Parser error: {:?}", e),
        };

        // Verify module was parsed
        assert_eq!(program.modules.len(), 1);
        let module = &program.modules[0];
        assert_eq!(module.name.name, "invalid_contracts");

        // Check that we have the bad_performance function
        assert_eq!(module.function_definitions.len(), 1);
        let function = &module.function_definitions[0];
        assert_eq!(function.name.name, "bad_performance");

        // Verify function has 1 parameter
        assert_eq!(function.parameters.len(), 1);
        assert_eq!(function.parameters[0].name.name, "x");

        println!(
            "✓ Successfully parsed function with invalid contracts (validation happens later)"
        );
    }
}
