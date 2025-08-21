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

use aether::lexer::Lexer;
use aether::parser::Parser;
use std::fs;

fn main() {
    let content = fs::read_to_string("tests/test_error_recovery.aether").unwrap();
    let mut lexer = Lexer::new(&content, "tests/test_error_recovery.aether".to_string());
    let tokens = lexer.tokenize().unwrap();
    
    println!("Total tokens: {}", tokens.len());
    
    let mut parser = Parser::new(tokens);
    match parser.parse_program() {
        Ok(program) => {
            println!("Parse succeeded with {} modules", program.modules.len());
            if parser.has_errors() {
                println!("But found {} errors during parsing:", parser.get_errors().len());
                for error in parser.get_errors() {
                    println!("  - {}", error);
                }
            }
        }
        Err(e) => {
            println!("Parse failed: {}", e);
            if parser.has_errors() {
                println!("Additional errors:");
                for error in parser.get_errors() {
                    println!("  - {}", error);
                }
            }
        }
    }
}