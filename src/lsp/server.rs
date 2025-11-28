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

use crate::pipeline::CompilationPipeline;
use crate::semantic::SemanticAnalyzer;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use std::path::Path;
use crate::error::CompilerError;
use crate::error::SourceLocation;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use crate::ast::Module;
use crate::symbols::SymbolTable;

#[derive(Debug, Default)]
struct State {
    documents: HashMap<Url, DocumentState>,
}

#[derive(Debug)]
struct DocumentState {
    text: String,
    module: Option<Module>,
    symbol_table: Option<SymbolTable>,
}

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    state: Arc<RwLock<State>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: Arc::new(RwLock::new(State::default())),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "AetherScript LSP initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("Opened: {}", params.text_document.uri))
            .await;
        
        self.validate_document(params.text_document.uri, params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("Changed: {}", params.text_document.uri))
            .await;
        
        // For full sync, content changes has one item with the full text
        if let Some(change) = params.content_changes.first() {
            self.validate_document(params.text_document.uri, change.text.clone()).await;
        }
    }
    
    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem {
                label: "Hello".to_string(),
                detail: Some("Some detail".to_string()),
                kind: Some(CompletionItemKind::TEXT),
                ..Default::default()
            },
            CompletionItem {
                label: "Bye".to_string(),
                detail: Some("Bye detail".to_string()),
                kind: Some(CompletionItemKind::TEXT),
                ..Default::default()
            },
        ])))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let state = self.state.read().unwrap();
        if let Some(doc_state) = state.documents.get(&uri) {
            if let Some(symbol_table) = &doc_state.symbol_table {
                // Find symbol at position
                // We need a way to query symbol table by location.
                // SymbolTable might store symbols by name, but we need location lookup.
                // Or we iterate all symbols.
                // Symbol has `declaration_location`.
                
                // We need to convert LSP Position (0-based) to SourceLocation (1-based lines, 1-based columns)
                // Also filenames might need normalized.
                
                let target_line = position.line as usize + 1;
                let target_col = position.character as usize + 1;
                
                // Simple check: find symbol defined exactly here or referenced here?
                // SymbolTable stores definitions. References might be in AST or separate index.
                // For hover, we usually want info about the symbol at the cursor, which could be a usage.
                // Without a full reference map, we can only show info if hovering over the definition.
                // Or if we traverse AST to find what is at that position.
                
                // For simplicity in this task, let's just check if we are hovering over a definition.
                
                for symbol in symbol_table.get_all_symbols() {
                    // Check if position is within definition location
                    // Assuming location is just start point, we might check proximity or token length
                    // For now, check exact start line/col match or close enough
                    if symbol.declaration_location.line == target_line &&
                       symbol.declaration_location.column <= target_col &&
                       symbol.declaration_location.column + symbol.name.len() >= target_col {
                           
                        return Ok(Some(Hover {
                            contents: HoverContents::Scalar(MarkedString::String(format!("{}: {}", symbol.name, symbol.symbol_type))),
                            range: None,
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
         let uri = params.text_document_position_params.text_document.uri;
         let position = params.text_document_position_params.position;

         let state = self.state.read().unwrap();
         if let Some(doc_state) = state.documents.get(&uri) {
             if let Some(symbol_table) = &doc_state.symbol_table {
                 let target_line = position.line as usize + 1;
                 let target_col = position.character as usize + 1;
                 
                 // First check if we are hovering over a definition
                 for symbol in symbol_table.get_all_symbols() {
                    if symbol.declaration_location.line == target_line &&
                       symbol.declaration_location.column <= target_col &&
                       symbol.declaration_location.column + symbol.name.len() >= target_col {
                           
                        let range = Self::location_to_range_static(&symbol.declaration_location);
                        return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                            uri: uri.clone(),
                            range,
                        })));
                    }
                 }
                 
                 // If not a definition, try to find the identifier at the cursor position from the text
                 let identifier_at_cursor = Self::find_identifier_at_position(&doc_state.text, position);
                 
                 if let Some(ident_name) = identifier_at_cursor {
                     // Look up the symbol in the symbol table
                     if let Some(symbol) = symbol_table.lookup_symbol(&ident_name) {
                         let range = Self::location_to_range_static(&symbol.declaration_location);
                         return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                             uri: uri.clone(),
                             range,
                         })));
                     }
                 }
             }
         }
         
         Ok(None)
    }
    
    fn semantic_error_to_diagnostic_static(error: &crate::error::SemanticError) -> Diagnostic {
}

impl Backend {
    fn find_identifier_at_position(text: &str, position: Position) -> Option<String> {
        let line_idx = position.line as usize;
        if let Some(line) = text.lines().nth(line_idx) {
            let col_idx = position.character as usize;
            
            // Simple check: find word boundaries around cursor
            // This is a heuristic; a proper lexer would be better but requires state
            
            if col_idx >= line.len() {
                return None;
            }
            
            // Logic replaced with simpler loop
            // Scan backwards
            let mut start_idx = col_idx;
            let chars: Vec<char> = line.chars().collect();
            
            if col_idx >= chars.len() {
                 return None;
            }

            let char_at_pos = chars[col_idx];
             if !char_at_pos.is_alphanumeric() && char_at_pos != '_' {
                return None;
            }
            
            while start_idx > 0 {
                let prev = start_idx - 1;
                let c = chars[prev];
                 if !c.is_alphanumeric() && c != '_' {
                     break;
                 }
                 start_idx = prev;
            }
            
            // Scan forwards
            let mut end_idx = col_idx;
            while end_idx < chars.len() {
                let c = chars[end_idx];
                if !c.is_alphanumeric() && c != '_' {
                    break;
                }
                end_idx += 1;
            }
            
            if start_idx < end_idx {
                return Some(line[start_idx..end_idx].to_string());
            }
        }
        None
    }

    async fn validate_document(&self, uri: Url, text: String) {
        let client = self.client.clone();
        let state = self.state.clone();
        let uri_clone = uri.clone();
        
        // Offload analysis to a blocking thread to avoid async/Send issues with Rc<RefCell>
        tokio::task::spawn_blocking(move || {
            let path = match uri_clone.to_file_path() {
                Ok(p) => p,
                Err(_) => return, // Ignore non-file URIs
            };

            // Parse the source
            let module_result = CompilationPipeline::parse_source(&path, &text, false);
            
            match module_result {
                Ok(module) => {
                    // Run semantic analysis
                    let program = crate::ast::Program {
                        modules: vec![module.clone()],
                        source_location: SourceLocation::unknown(),
                    };
                    
                    let mut analyzer = SemanticAnalyzer::new();
                    match analyzer.analyze_program(&program) {
                        Ok(_) => {
                            // Update state
                            {
                                let mut state_lock = state.write().unwrap();
                                state_lock.documents.insert(uri_clone.clone(), DocumentState {
                                    text: text.clone(),
                                    module: Some(module),
                                    symbol_table: Some(analyzer.get_symbol_table().clone()),
                                });
                            }

                            // Clear diagnostics if no errors
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async {
                                client.publish_diagnostics(uri_clone, vec![], None).await;
                            });
                        }
                        Err(errors) => {
                             // Update state even if errors? Maybe partial results?
                             // For now, only update on success or if we can salvage info.
                             // Let's update with what we have (maybe module is fine but semantics failed).
                             {
                                let mut state_lock = state.write().unwrap();
                                state_lock.documents.insert(uri_clone.clone(), DocumentState {
                                    text: text.clone(),
                                    module: Some(module),
                                    symbol_table: Some(analyzer.get_symbol_table().clone()), // Symbol table might be partial
                                });
                            }
                            
                            // Report semantic errors
                            let diagnostics = errors.into_iter().map(|err| {
                                Self::semantic_error_to_diagnostic_static(&err)
                            }).collect();
                            
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async {
                                client.publish_diagnostics(uri_clone, diagnostics, None).await;
                            });
                        }
                    }
                }
                Err(err) => {
                    // Report parse error
                    let diagnostic = Self::compiler_error_to_diagnostic_static(&err);
                    let rt = tokio::runtime::Handle::current();
                    rt.block_on(async {
                        client.publish_diagnostics(uri_clone, vec![diagnostic], None).await;
                    });
                }
            }
        }).await.unwrap();
    }
    
    fn compiler_error_to_diagnostic_static(error: &CompilerError) -> Diagnostic {
        match error {
            CompilerError::Parser { source } => {
                 match source {
                     crate::error::ParserError::UnexpectedToken { location, expected, found } => {
                         Diagnostic {
                             range: Self::location_to_range_static(location),
                             severity: Some(DiagnosticSeverity::ERROR),
                             message: format!("Unexpected token: found {}, expected {}", found, expected),
                             ..Default::default()
                         }
                     }
                     crate::error::ParserError::SyntaxError { message, location, suggestion: _ } => {
                        Diagnostic {
                            range: Self::location_to_range_static(location),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: message.clone(),
                            ..Default::default()
                        }
                     }
                     // Handle other parser errors...
                     _ => {
                         Diagnostic {
                             range: Range::default(), // Fallback
                             severity: Some(DiagnosticSeverity::ERROR),
                             message: source.to_string(),
                             ..Default::default()
                         }
                     }
                 }
            },
            CompilerError::Lexer { source } => {
                 match source {
                     crate::error::LexerError::UnexpectedCharacter { location, character } => {
                         Diagnostic {
                             range: Self::location_to_range_static(location),
                             severity: Some(DiagnosticSeverity::ERROR),
                             message: format!("Unexpected character: {}", character),
                             ..Default::default()
                         }
                     }
                     // Handle other lexer errors...
                     _ => {
                         Diagnostic {
                             range: Range::default(), // Fallback
                             severity: Some(DiagnosticSeverity::ERROR),
                             message: source.to_string(),
                             ..Default::default()
                         }
                     }
                 }
            }
            // Default fallback
            _ => Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                message: error.to_string(),
                ..Default::default()
            }
        }
    }

    fn semantic_error_to_diagnostic_static(error: &crate::error::SemanticError) -> Diagnostic {
        let (message, location) = match error {
            crate::error::SemanticError::UndefinedSymbol { symbol, location } => (format!("Undefined symbol: {}", symbol), location),
            crate::error::SemanticError::TypeMismatch { expected, found, location } => (format!("Type mismatch: expected {}, found {}", expected, found), location),
            crate::error::SemanticError::AssignToImmutable { variable, location } => (format!("Cannot assign to immutable variable: {}", variable), location),
            crate::error::SemanticError::UseBeforeInitialization { variable, location } => (format!("Variable used before initialization: {}", variable), location),
            crate::error::SemanticError::UseAfterMove { variable, location } => (format!("Variable used after move: {}", variable), location),
            // Add other cases as needed, falling back to a generic message
            _ => (error.to_string(), &SourceLocation::unknown()),
        };

        Diagnostic {
            range: Self::location_to_range_static(location),
            severity: Some(DiagnosticSeverity::ERROR),
            message,
            ..Default::default()
        }
    }

    fn location_to_range_static(location: &SourceLocation) -> Range {
        Range {
            start: Position {
                line: location.line as u32 - 1, // 1-based to 0-based
                character: location.column as u32 - 1,
            },
            end: Position {
                line: location.line as u32 - 1,
                character: location.column as u32, // Assuming length 1 for now, ideally we'd know length
            },
        }
    }
}
