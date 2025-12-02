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

//! Language Server Protocol implementation for AetherScript
//!
//! Provides IDE integration through LSP, including auto-completion, hover information,
//! go-to-definition, and real-time diagnostics.

use crate::error::{SemanticError, SourceLocation};
use crate::parser::v2::Parser;
use crate::semantic::SemanticAnalyzer;
use crate::types::Type;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Language Server for AetherScript
#[derive(Debug)]
pub struct DiagnosticsProvider;

#[derive(Debug)]
pub struct LanguageServer {
    /// Document manager
    document_manager: DocumentManager,

    /// Symbol index
    symbol_index: SymbolIndex,

    /// Completion provider
    completion_provider: CompletionProvider,

    /// Configuration
    config: LspConfig,
}

/// LSP configuration
#[derive(Debug, Clone)]
pub struct LspConfig {
    /// Server capabilities
    pub capabilities: ServerCapabilities,

    /// Server port
    pub port: u16,

    /// Maximum number of cached documents
    pub max_cached_documents: usize,

    /// Enable real-time diagnostics
    pub real_time_diagnostics: bool,

    /// Completion trigger characters
    pub completion_triggers: Vec<String>,
}

/// LSP server capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Text document sync kind
    pub text_document_sync: TextDocumentSyncKind,

    /// Provides hover information
    pub hover_provider: bool,

    /// Provides completion
    pub completion_provider: Option<CompletionOptions>,

    /// Provides go-to-definition
    pub definition_provider: bool,

    /// Provides diagnostics
    pub diagnostic_provider: bool,

    /// Provides document symbols
    pub document_symbol_provider: bool,

    /// Provides workspace symbols
    pub workspace_symbol_provider: bool,
}

/// Text document synchronization kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextDocumentSyncKind {
    None = 0,
    Full = 1,
    Incremental = 2,
}

impl Default for TextDocumentSyncKind {
    fn default() -> Self {
        TextDocumentSyncKind::Full
    }
}

/// Completion options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionOptions {
    /// Characters that trigger completion
    pub trigger_characters: Vec<String>,

    /// Resolve additional information for completion items
    pub resolve_provider: bool,
}

/// Document manager for tracking open files
#[derive(Default)]
pub struct DocumentManager {
    /// Open documents
    documents: HashMap<String, Document>,

    /// Document change notifications
    change_listeners: Vec<Box<dyn Fn(&str, &Document) + Send + Sync>>,
}

impl std::fmt::Debug for DocumentManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentManager")
            .field("documents", &self.documents)
            .field(
                "change_listeners",
                &format!("<{} listeners>", self.change_listeners.len()),
            )
            .finish()
    }
}

/// Represents an open document
#[derive(Debug, Clone)]
pub struct Document {
    /// Document URI
    pub uri: String,

    /// Language identifier
    pub language_id: String,

    /// Document version
    pub version: i32,

    /// Document content
    pub content: String,

    /// Parsed AST (cached)
    pub ast: Option<Arc<crate::ast::Module>>,

    /// Last parse error
    pub parse_error: Option<String>,

    /// Semantic analysis results
    pub semantic_info: Option<SemanticInfo>,
}

/// Semantic analysis information
#[derive(Debug, Clone)]
pub struct SemanticInfo {
    /// Symbol table
    pub symbols: HashMap<String, SymbolInfo>,

    /// Type information
    pub types: HashMap<SourceLocation, Type>,

    /// Diagnostics
    pub diagnostics: Vec<Diagnostic>,
}

/// Symbol information
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// Symbol name
    pub name: String,

    /// Symbol kind
    pub kind: SymbolKind,

    /// Symbol type
    pub symbol_type: Type,

    /// Definition location
    pub definition: SourceLocation,

    /// References to this symbol
    pub references: Vec<SourceLocation>,

    /// Documentation
    pub documentation: Option<String>,
}

/// Symbol kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymbolKind {
    File = 1,
    Module = 2,
    Namespace = 3,
    Package = 4,
    Class = 5,
    Method = 6,
    Property = 7,
    Field = 8,
    Constructor = 9,
    Enum = 10,
    Interface = 11,
    Function = 12,
    Variable = 13,
    Constant = 14,
    String = 15,
    Number = 16,
    Boolean = 17,
    Array = 18,
    Object = 19,
    Key = 20,
    Null = 21,
    EnumMember = 22,
    Struct = 23,
    Event = 24,
    Operator = 25,
    TypeParameter = 26,
}

/// Symbol index for fast lookup
#[derive(Clone)]
pub struct SymbolIndex {
    /// Document symbols
    pub document_symbols: HashMap<String, Vec<SymbolInfo>>,
}

impl Default for SymbolIndex {
    fn default() -> Self {
        Self {
            document_symbols: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for SymbolIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SymbolIndex")
            .field("document_symbols", &self.document_symbols)
            .finish()
    }
}

/// Diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Source range
    pub range: Range,

    /// Diagnostic severity
    pub severity: DiagnosticSeverity,

    /// Error code
    pub code: Option<String>,

    /// Human-readable message
    pub message: String,

    /// Source of the diagnostic
    pub source: Option<String>,

    /// Related information
    pub related_information: Vec<DiagnosticRelatedInformation>,
}

/// Source range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    /// Start position
    pub start: Position,

    /// End position
    pub end: Position,
}

/// Position in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-based)
    pub line: u32,

    /// Character offset in line (0-based)
    pub character: u32,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// Related diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    /// Location of related information
    pub location: Location,

    /// Message describing the relationship
    pub message: String,
}

/// Location in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Document URI
    pub uri: String,

    /// Range in the document
    pub range: Range,
}

/// Diagnostic engine
#[derive(Debug, Default)]
pub struct DiagnosticEngine {}

/// Completion provider
#[derive(Debug)]
pub struct CompletionProvider {
    /// Keyword completions
    keywords: Vec<CompletionItem>,
}

impl Default for CompletionProvider {
    fn default() -> Self {
        let keyword_names = vec![
            "def", "let", "if", "else", "match", "loop", "while", "for", "break", "continue",
            "return", "struct", "enum", "trait", "impl", "use", "module", "pub", "priv", "mut",
            "const", "static", "async", "await", "spawn",
        ];

        let keywords = keyword_names
            .into_iter()
            .map(|kw| CompletionItem {
                label: kw.to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some(format!("Keyword: {}", kw)),
                documentation: None,
                insert_text: Some(kw.to_string()),
                sort_text: None,
            })
            .collect();

        Self { keywords }
    }
}

/// Completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// The label of this completion item
    pub label: String,

    /// The kind of this completion item
    pub kind: CompletionItemKind,

    /// Additional details
    pub detail: Option<String>,

    /// Documentation
    pub documentation: Option<String>,

    /// Text to insert
    pub insert_text: Option<String>,

    /// Sort priority
    pub sort_text: Option<String>,
}

/// Completion item kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

/// Hover provider for providing hover information
#[derive(Debug, Clone)]
pub struct HoverProvider;

impl Default for HoverProvider {
    fn default() -> Self {
        Self {}
    }
}

/// Hover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverInfo {
    /// Contents to display
    pub contents: Vec<MarkedString>,

    /// Range to highlight
    pub range: Option<Range>,
}

/// Marked string for hover content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarkedString {
    /// Plain text
    String(String),

    /// Code block with language
    LanguageString { language: String, value: String },
}

/// Definition provider for go-to-definition functionality
#[derive(Debug, Clone)]
pub struct DefinitionProvider;

impl Default for DefinitionProvider {
    fn default() -> Self {
        Self {}
    }
}

impl LanguageServer {
    pub fn new(config: LspConfig) -> Self {
        let mut completion_provider = CompletionProvider::default();
        completion_provider.initialize_builtin_completions();

        Self {
            document_manager: DocumentManager::default(),
            symbol_index: SymbolIndex::default(),
            completion_provider,
            config,
        }
    }

    /// Start the language server
    pub fn start(&mut self) -> Result<(), SemanticError> {
        eprintln!("Starting AetherScript Language Server");

        // Initialize server capabilities
        self.initialize_capabilities();

        // Set up document change listeners
        self.setup_change_listeners();

        eprintln!("Language server started successfully");
        Ok(())
    }

    /// Initialize server capabilities
    fn initialize_capabilities(&mut self) {}

    /// Set up document change listeners
    fn setup_change_listeners(&mut self) {
        // This would set up real-time analysis triggers
        eprintln!("Document change listeners configured");
    }

    /// Handle document open
    pub fn did_open(
        &mut self,
        uri: String,
        language_id: String,
        version: i32,
        content: String,
    ) -> Result<(), SemanticError> {
        let mut document = Document {
            uri: uri.clone(),
            language_id,
            version,
            content,
            ast: None,
            parse_error: None,
            semantic_info: None,
        };

        // Parse and analyze document
        self.analyze_document(&mut document)?;

        self.document_manager.documents.insert(uri, document);
        Ok(())
    }

    /// Handle document change
    pub fn did_change(
        &mut self,
        uri: &str,
        version: i32,
        content: String,
    ) -> Result<(), SemanticError> {
        // Update document
        if let Some(document) = self.document_manager.documents.get_mut(uri) {
            document.version = version;
            document.content = content;
            document.ast = None; // Invalidate cache
            document.semantic_info = None;
        }

        // Re-analyze document - clone the document to avoid borrow conflicts
        let document_copy = self.document_manager.documents.get(uri).cloned();
        if let Some(mut document) = document_copy {
            self.analyze_document(&mut document)?;
            // Update the document in the manager
            self.document_manager
                .documents
                .insert(uri.to_string(), document);
        }

        Ok(())
    }

    /// Handle document close
    pub fn did_close(&mut self, uri: &str) {
        self.document_manager.documents.remove(uri);
    }

    /// Analyze a document
    fn analyze_document(&mut self, document: &mut Document) -> Result<(), SemanticError> {
        // Tokenize the document
        let mut lexer = crate::lexer::v2::Lexer::new(&document.content, document.uri.clone());
        let tokens = match lexer.tokenize() {
            Ok(tokens) => tokens,
            Err(e) => {
                document.parse_error = Some(format!("Lexical error: {}", e));
                return Ok(());
            }
        };

        // Parse the document
        let mut parser = Parser::new(tokens);
        match parser.parse_module() {
            Ok(ast) => {
                document.ast = Some(Arc::new(ast));
                document.parse_error = None;

                // Perform semantic analysis
                self.perform_semantic_analysis(document)?;

                // Update symbol index
                self.update_symbol_index(document)?;

                // Generate diagnostics
                self.generate_diagnostics(document)?;
            }
            Err(error) => {
                document.parse_error = Some(error.to_string());
            }
        }

        Ok(())
    }

    /// Perform semantic analysis
    fn perform_semantic_analysis(&self, document: &mut Document) -> Result<(), SemanticError> {
        if let Some(ref _ast) = document.ast {
            let mut _analyzer = SemanticAnalyzer::new();
            // Semantic analysis would be performed here
            // For now, just create empty semantic info
            let semantic_info = SemanticInfo {
                symbols: HashMap::new(), // Would be populated from analyzer
                types: HashMap::new(),   // Would be populated from analyzer
                diagnostics: vec![],     // Would be populated from analyzer
            };
            document.semantic_info = Some(semantic_info);
        }

        Ok(())
    }

    /// Update symbol index
    fn update_symbol_index(&mut self, document: &Document) -> Result<(), SemanticError> {
        if let Some(ref semantic_info) = document.semantic_info {
            for (_, symbol) in &semantic_info.symbols {
                self.symbol_index
                    .document_symbols
                    .entry(document.uri.clone())
                    .or_insert_with(Vec::new)
                    .push(symbol.clone());
            }
        }

        Ok(())
    }

    /// Generate diagnostics
    fn generate_diagnostics(&mut self, document: &Document) -> Result<(), SemanticError> {
        let mut diagnostics = Vec::new();

        if let Some(ref semantic_info) = document.semantic_info {
            diagnostics.extend(semantic_info.diagnostics.clone());
        }

        Ok(())
    }

    /// Provide completions at position
    pub fn completion(
        &self,
        uri: &str,
        _position: Position,
    ) -> Result<Vec<CompletionItem>, SemanticError> {
        let mut completions = self.completion_provider.keywords.clone();

        // Add context-sensitive completions
        if let Some(document) = self.document_manager.documents.get(uri) {
            if let Some(ref semantic_info) = document.semantic_info {
                // Add symbols as completions
                for symbol in semantic_info.symbols.values() {
                    completions.push(CompletionItem {
                        label: symbol.name.clone(),
                        kind: self.symbol_kind_to_completion_kind(symbol.kind.clone()),
                        detail: Some(format!("{}", symbol.symbol_type)),
                        documentation: symbol.documentation.clone(),
                        insert_text: Some(symbol.name.clone()),
                        sort_text: None,
                    });
                }
            }
        }

        Ok(completions)
    }

    /// Convert symbol kind to completion kind
    fn symbol_kind_to_completion_kind(&self, kind: SymbolKind) -> CompletionItemKind {
        match kind {
            SymbolKind::Function => CompletionItemKind::Function,
            SymbolKind::Variable => CompletionItemKind::Variable,
            SymbolKind::Constant => CompletionItemKind::Constant,
            SymbolKind::Class => CompletionItemKind::Class,
            SymbolKind::Method => CompletionItemKind::Method,
            _ => CompletionItemKind::Text,
        }
    }

    /// Provide hover information
    pub fn hover(&self, uri: &str, position: Position) -> Result<Option<HoverInfo>, SemanticError> {
        if let Some(document) = self.document_manager.documents.get(uri) {
            if let Some(ref semantic_info) = document.semantic_info {
                // Find symbol at position
                let location = SourceLocation {
                    file: uri.to_string(),
                    line: position.line as usize,
                    column: position.character as usize,
                    offset: 0, // We don't have the exact offset from LSP position
                };

                if let Some(symbol) = semantic_info.symbols.values().find(|s| {
                    s.definition.file == location.file && s.definition.line == location.line
                }) {
                    let hover_info = HoverInfo {
                        contents: vec![
                            MarkedString::LanguageString {
                                language: "aetherscript".to_string(),
                                value: format!("{}: {}", symbol.name, symbol.symbol_type),
                            },
                            MarkedString::String(
                                symbol
                                    .documentation
                                    .clone()
                                    .unwrap_or("No documentation available".to_string()),
                            ),
                        ],
                        range: Some(Range {
                            start: Position {
                                line: symbol.definition.line as u32,
                                character: symbol.definition.column as u32,
                            },
                            end: Position {
                                line: symbol.definition.line as u32,
                                character: (symbol.definition.column + symbol.name.len()) as u32,
                            },
                        }),
                    };

                    return Ok(Some(hover_info));
                }
            }
        }

        Ok(None)
    }

    /// Provide go-to-definition
    pub fn definition(
        &self,
        uri: &str,
        position: Position,
    ) -> Result<Vec<Location>, SemanticError> {
        if let Some(document) = self.document_manager.documents.get(uri) {
            if let Some(ref semantic_info) = document.semantic_info {
                // Find symbol at position and return its definition
                let location = SourceLocation {
                    file: uri.to_string(),
                    line: position.line as usize,
                    column: position.character as usize,
                    offset: 0, // We don't have the exact offset from LSP position
                };

                if let Some(symbol) = semantic_info.symbols.values().find(|s| {
                    s.references
                        .iter()
                        .any(|r| r.file == location.file && r.line == location.line)
                }) {
                    return Ok(vec![Location {
                        uri: symbol.definition.file.clone(),
                        range: Range {
                            start: Position {
                                line: symbol.definition.line as u32,
                                character: symbol.definition.column as u32,
                            },
                            end: Position {
                                line: symbol.definition.line as u32,
                                character: (symbol.definition.column + symbol.name.len()) as u32,
                            },
                        },
                    }]);
                }
            }
        }

        Ok(vec![])
    }

    /// Get diagnostics for document
    pub fn get_diagnostics(&self, _uri: &str) -> Vec<Diagnostic> {
        vec![]
    }

    /// Get server capabilities
    pub fn capabilities(&self) -> &ServerCapabilities {
        &self.config.capabilities
    }
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            capabilities: ServerCapabilities {
                text_document_sync: TextDocumentSyncKind::Incremental,
                hover_provider: true,
                completion_provider: Some(CompletionOptions {
                    trigger_characters: vec![".".to_string(), ":".to_string()],
                    resolve_provider: true,
                }),
                definition_provider: true,
                diagnostic_provider: true,
                document_symbol_provider: true,
                workspace_symbol_provider: true,
            },
            port: 7878,
            max_cached_documents: 100,
            real_time_diagnostics: true,
            completion_triggers: vec![".".to_string(), ":".to_string()],
        }
    }
}

impl CompletionProvider {
    /// Initialize built-in completions
    fn initialize_builtin_completions(&mut self) {
        self.keywords = vec![
            CompletionItem {
                label: "if".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Keyword: if".to_string()),
                documentation: None,
                insert_text: Some("if".to_string()),
                sort_text: None,
            },
            CompletionItem {
                label: "function".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Keyword: function".to_string()),
                documentation: None,
                insert_text: Some("function".to_string()),
                sort_text: None,
            },
            CompletionItem {
                label: "let".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Keyword: let".to_string()),
                documentation: None,
                insert_text: Some("let".to_string()),
                sort_text: None,
            },
            CompletionItem {
                label: "while".to_string(),
                kind: CompletionItemKind::Keyword,
                detail: Some("Keyword: while".to_string()),
                documentation: None,
                insert_text: Some("while".to_string()),
                sort_text: None,
            },
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_server_creation() {
        let config = LspConfig::default();
        let server = LanguageServer::new(config);

        assert!(server.config.capabilities.hover_provider);
        assert!(server.config.capabilities.definition_provider);
        assert!(server.config.capabilities.diagnostic_provider);
    }

    #[test]
    fn test_document_management() {
        let mut server = LanguageServer::new(LspConfig::default());

        let uri = "file:///test.aether".to_string();
        let content = "(define test 42)".to_string();

        assert!(server
            .did_open(uri.clone(), "aetherscript".to_string(), 1, content)
            .is_ok());
        assert!(server.document_manager.documents.contains_key(&uri));

        server.did_close(&uri);
        assert!(!server.document_manager.documents.contains_key(&uri));
    }

    #[test]
    fn test_completion_items() {
        let mut provider = CompletionProvider::default();
        provider.initialize_builtin_completions();

        assert!(!provider.keywords.is_empty());

        let if_completion = provider.keywords.iter().find(|item| item.label == "if");

        assert!(if_completion.is_some());
        assert_eq!(if_completion.unwrap().kind, CompletionItemKind::Keyword);
    }

    #[test]
    fn test_diagnostic_creation() {
        let diagnostic = Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            },
            severity: DiagnosticSeverity::Error,
            code: Some("E001".to_string()),
            message: "Test error".to_string(),
            source: Some("AetherScript".to_string()),
            related_information: vec![],
        };

        assert_eq!(diagnostic.message, "Test error");
        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn test_symbol_info() {
        let symbol = SymbolInfo {
            name: "test_var".to_string(),
            kind: SymbolKind::Variable,
            symbol_type: Type::primitive(crate::ast::PrimitiveType::Integer),
            definition: SourceLocation {
                file: "test.aether".to_string(),
                line: 10,
                column: 5,
                offset: 0,
            },
            references: vec![],
            documentation: Some("Test variable".to_string()),
        };

        assert_eq!(symbol.name, "test_var");
        assert_eq!(symbol.kind, SymbolKind::Variable);
    }
}
