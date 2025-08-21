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

fn main() {
    let source = fs::read_to_string("test_simple_loop.aether").unwrap();
    let mut lexer = aether::lexer::Lexer::new(&source, "test_simple_loop.aether".to_string());
    let tokens = lexer.tokenize();
    
    match tokens {
        Ok(tokens) => {
            println!("Tokens: {:?}", tokens);
            let mut parser = aether::parser::Parser::new(tokens);
            let result = parser.parse_program();
            match result {
                Ok(program) => println!("Successfully parsed program with {} modules", program.modules.len()),
                Err(e) => println!("Parse error: {:?}", e),
            }
        }
        Err(e) => println!("Lexer error: {:?}", e),
    }
}