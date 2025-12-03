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

use aether::lexer::v2::{Lexer, TokenType, Keyword};

#[test]
fn test_repro_llvm_module_keyword_issue() {
    let source = r#"
module test_llvm_c_api;

struct LLVMContext {}
struct LLVMModule {}

@extern(library="LLVM")
func LLVMModuleCreateWithName(ModuleID: Pointer<Char>) -> Pointer<LLVMModule>;
"#;

    let mut lexer = Lexer::new(source, "test.aether".to_string());
    let tokens = lexer.tokenize().unwrap();

    for token in &tokens {
        println!("{:?} at {:?}", token.token_type, token.location);
    }

    // Check token at line 8 (func LLVMModuleCreateWithName...)
    // Tokens should be: At, Identifier(extern), LeftParen, ... RightParen
    // Keyword(Func), Identifier(LLVMModuleCreateWithName), ...

    // Find Identifier(LLVMModuleCreateWithName)
    let found = tokens.iter().find(|t| match &t.token_type {
        TokenType::Identifier(name) => name == "LLVMModuleCreateWithName",
        _ => false,
    });
    
    if found.is_some() {
        println!("Found Identifier(LLVMModuleCreateWithName) correctly.");
    } else {
        println!("Did NOT find Identifier(LLVMModuleCreateWithName). Checking for Keyword(Module)...");
        let found_mod = tokens.iter().find(|t| match &t.token_type {
            TokenType::Keyword(Keyword::Module) => true,
            _ => false,
        });
        if let Some(tok) = found_mod {
             println!("Found Keyword(Module) at {:?} lexeme='{}'", tok.location, tok.lexeme);
        }
    }
    
    assert!(found.is_some(), "Should have found LLVMModuleCreateWithName as identifier");
}
