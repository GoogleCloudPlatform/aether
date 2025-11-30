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

//! Documentation generation system for AetherScript
//!
//! Provides comprehensive documentation generation including API docs,
//! tutorials, examples, and reference manuals with multiple output formats.

pub mod examples;
pub mod generator;
pub mod parser;
pub mod renderer;
pub mod tutorial;

use crate::error::SemanticError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Documentation generator for AetherScript
#[derive(Debug)]
pub struct DocumentationGenerator {
    /// Generation configuration
    config: DocConfig,

    /// Documentation parser
    parser: parser::DocParser,

    /// Content renderer
    renderer: renderer::DocRenderer,

    /// Example manager
    examples: examples::ExampleManager,

    /// Generated documentation
    documentation: Documentation,
}

/// Documentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocConfig {
    /// Project name
    pub project_name: String,

    /// Project version
    pub project_version: String,

    /// Output directory
    pub output_dir: PathBuf,

    /// Source directories
    pub source_dirs: Vec<PathBuf>,

    /// Include private items
    pub include_private: bool,

    /// Output format
    pub output_format: OutputFormat,

    /// Generate examples
    pub generate_examples: bool,

    /// Generate tutorials
    pub generate_tutorials: bool,

    /// Output formats
    pub output_formats: Vec<OutputFormat>,

    /// Theme configuration
    pub theme: ThemeConfig,

    /// Search configuration
    pub search: SearchConfig,
}

/// Output formats for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    /// HTML documentation
    Html {
        /// Enable JavaScript
        javascript: bool,
        /// CSS theme
        theme: String,
        /// Enable search
        search: bool,
    },

    /// Markdown documentation
    Markdown {
        /// Include table of contents
        toc: bool,
        /// GitHub flavored markdown
        github_flavored: bool,
    },

    /// PDF documentation
    Pdf {
        /// Page size
        page_size: String,
        /// Include bookmarks
        bookmarks: bool,
    },

    /// JSON API documentation
    Json {
        /// Pretty print
        pretty: bool,
        /// Include source locations
        include_source: bool,
    },
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Primary color
    pub primary_color: String,

    /// Secondary color
    pub secondary_color: String,

    /// Font family
    pub font_family: String,

    /// Code font family
    pub code_font_family: String,

    /// Custom CSS
    pub custom_css: Option<String>,

    /// Logo URL
    pub logo_url: Option<String>,

    /// Favicon URL
    pub favicon_url: Option<String>,
}

/// Search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Enable search
    pub enabled: bool,

    /// Search index type
    pub index_type: SearchIndexType,

    /// Maximum results
    pub max_results: usize,

    /// Search weights
    pub weights: SearchWeights,
}

/// Search index types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchIndexType {
    /// Client-side JavaScript search
    ClientSide,

    /// Server-side search
    ServerSide,

    /// Elasticsearch integration
    Elasticsearch { endpoint: String },
}

/// Search result weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWeights {
    /// Title weight
    pub title: f32,

    /// Description weight
    pub description: f32,

    /// Content weight
    pub content: f32,

    /// Tags weight
    pub tags: f32,
}

/// Complete documentation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    /// Project metadata
    pub metadata: ProjectMetadata,

    /// API documentation
    pub api: ApiDocumentation,

    /// Tutorials
    pub tutorials: Vec<Tutorial>,

    /// Examples
    pub examples: Vec<Example>,

    /// Reference manual
    pub reference: ReferenceManual,

    /// Search index
    pub search_index: Option<SearchIndex>,
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Project name
    pub name: String,

    /// Project version
    pub version: String,

    /// Project description
    pub description: Option<String>,

    /// Authors
    pub authors: Vec<String>,

    /// License
    pub license: Option<String>,

    /// Homepage
    pub homepage: Option<String>,

    /// Repository
    pub repository: Option<String>,

    /// Documentation URL
    pub documentation_url: Option<String>,

    /// Generation timestamp
    pub generated_at: std::time::SystemTime,
}

/// API documentation structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocumentation {
    /// Modules
    pub modules: Vec<ModuleDoc>,

    /// Functions
    pub functions: Vec<FunctionDoc>,

    /// Types
    pub types: Vec<TypeDoc>,

    /// Constants
    pub constants: Vec<ConstantDoc>,

    /// Macros
    pub macros: Vec<MacroDoc>,
}

/// Module documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDoc {
    /// Module name
    pub name: String,

    /// Module path
    pub path: String,

    /// Description
    pub description: Option<String>,

    /// Documentation comments
    pub docs: String,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub source_location: SourceLocation,

    /// Submodules
    pub submodules: Vec<String>,

    /// Public items
    pub items: Vec<ItemSummary>,

    /// Examples
    pub examples: Vec<CodeExample>,
}

/// Function documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDoc {
    /// Function name
    pub name: String,

    /// Full path
    pub path: String,

    /// Description
    pub description: Option<String>,

    /// Documentation comments
    pub docs: String,

    /// Function signature
    pub signature: FunctionSignature,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub source_location: SourceLocation,

    /// Parameters
    pub parameters: Vec<ParameterDoc>,

    /// Return type
    pub return_type: Option<TypeReference>,

    /// Examples
    pub examples: Vec<CodeExample>,

    /// Related functions
    pub related: Vec<String>,
}

/// Type documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDoc {
    /// Type name
    pub name: String,

    /// Full path
    pub path: String,

    /// Description
    pub description: Option<String>,

    /// Documentation comments
    pub docs: String,

    /// Type kind
    pub kind: TypeKind,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub source_location: SourceLocation,

    /// Type parameters
    pub type_parameters: Vec<TypeParameterDoc>,

    /// Fields (for structs)
    pub fields: Vec<FieldDoc>,

    /// Variants (for enums)
    pub variants: Vec<VariantDoc>,

    /// Methods
    pub methods: Vec<String>, // References to function docs

    /// Trait implementations
    pub trait_impls: Vec<TraitImplDoc>,

    /// Examples
    pub examples: Vec<CodeExample>,
}

/// Constant documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDoc {
    /// Constant name
    pub name: String,

    /// Full path
    pub path: String,

    /// Description
    pub description: Option<String>,

    /// Documentation comments
    pub docs: String,

    /// Constant type
    pub const_type: TypeReference,

    /// Constant value
    pub value: Option<String>,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub source_location: SourceLocation,
}

/// Macro documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDoc {
    /// Macro name
    pub name: String,

    /// Full path
    pub path: String,

    /// Description
    pub description: Option<String>,

    /// Documentation comments
    pub docs: String,

    /// Macro signature
    pub signature: String,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub source_location: SourceLocation,

    /// Examples
    pub examples: Vec<CodeExample>,
}

/// Item visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

/// Source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file: PathBuf,

    /// Line number
    pub line: usize,

    /// Column number
    pub column: usize,

    /// Span length
    pub span: usize,
}

/// Item summary for module listings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSummary {
    /// Item name
    pub name: String,

    /// Item kind
    pub kind: ItemKind,

    /// Brief description
    pub description: Option<String>,

    /// Visibility
    pub visibility: Visibility,
}

/// Item kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemKind {
    Function,
    Type,
    Struct,
    Constant,
    Macro,
    Module,
}

/// Function signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,

    /// Type parameters
    pub type_parameters: Vec<TypeParameterDoc>,

    /// Parameters
    pub parameters: Vec<ParameterDoc>,

    /// Return type
    pub return_type: Option<TypeReference>,

    /// Async function
    pub is_async: bool,

    /// Unsafe function
    pub is_unsafe: bool,
}

/// Parameter documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDoc {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub param_type: TypeReference,

    /// Description
    pub description: Option<String>,

    /// Default value
    pub default_value: Option<String>,
}

/// Type reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeReference {
    /// Type name
    pub name: String,

    /// Full path
    pub path: String,

    /// Type arguments
    pub type_args: Vec<TypeReference>,

    /// Is reference
    pub is_reference: bool,

    /// Is mutable
    pub is_mutable: bool,
}

/// Type parameter documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeParameterDoc {
    /// Parameter name
    pub name: String,

    /// Bounds
    pub bounds: Vec<String>,

    /// Default type
    pub default_type: Option<TypeReference>,

    /// Description
    pub description: Option<String>,
}

/// Type kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Struct,
    Enum,
    Union,
    Trait,
    TypeAlias,
    FunctionPointer,
}

/// Field documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDoc {
    /// Field name
    pub name: String,

    /// Field type
    pub field_type: TypeReference,

    /// Description
    pub description: Option<String>,

    /// Visibility
    pub visibility: Visibility,
}

/// Enum variant documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantDoc {
    /// Variant name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Variant data
    pub data: VariantData,
}

/// Enum variant data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariantData {
    /// Unit variant
    Unit,

    /// Tuple variant
    Tuple(Vec<TypeReference>),

    /// Struct variant
    Struct(Vec<FieldDoc>),
}

/// Trait implementation documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitImplDoc {
    /// Trait name
    pub trait_name: String,

    /// Trait path
    pub trait_path: String,

    /// Type parameters
    pub type_parameters: Vec<TypeParameterDoc>,

    /// Implemented methods
    pub methods: Vec<String>,
}

/// Code example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    /// Example title
    pub title: Option<String>,

    /// Example description
    pub description: Option<String>,

    /// Example code
    pub code: String,

    /// Expected output
    pub output: Option<String>,

    /// Language
    pub language: String,

    /// Example type
    pub example_type: ExampleType,
}

/// Example types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExampleType {
    Basic,
    Intermediate,
    Advanced,
    Tutorial,
    Test,
    Benchmark,
}

/// Tutorial structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    /// Tutorial title
    pub title: String,

    /// Tutorial description
    pub description: String,

    /// Tutorial content
    pub content: String,

    /// Tutorial sections
    pub sections: Vec<TutorialSection>,

    /// Prerequisites
    pub prerequisites: Vec<String>,

    /// Difficulty level
    pub difficulty: DifficultyLevel,

    /// Estimated time
    pub estimated_time: Option<String>,

    /// Tags
    pub tags: Vec<String>,
}

/// Tutorial section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialSection {
    /// Section title
    pub title: String,

    /// Section content
    pub content: String,

    /// Code examples
    pub examples: Vec<CodeExample>,

    /// Exercises
    pub exercises: Vec<Exercise>,
}

/// Exercise
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    /// Exercise title
    pub title: String,

    /// Exercise description
    pub description: String,

    /// Starting code
    pub starter_code: Option<String>,

    /// Solution
    pub solution: Option<String>,

    /// Hints
    pub hints: Vec<String>,
}

/// Difficulty levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Example structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Example name
    pub name: String,

    /// Example description
    pub description: String,

    /// Example category
    pub category: String,

    /// Source code
    pub source_code: String,

    /// Expected output
    pub expected_output: Option<String>,

    /// Dependencies
    pub dependencies: Vec<String>,

    /// Build instructions
    pub build_instructions: Option<String>,

    /// Tags
    pub tags: Vec<String>,
}

/// Reference manual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceManual {
    /// Manual sections
    pub sections: Vec<ManualSection>,

    /// Appendices
    pub appendices: Vec<Appendix>,

    /// Glossary
    pub glossary: HashMap<String, String>,

    /// Index
    pub index: Vec<IndexEntry>,
}

/// Manual section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualSection {
    /// Section title
    pub title: String,

    /// Section content
    pub content: String,

    /// Subsections
    pub subsections: Vec<ManualSection>,

    /// Cross-references
    pub references: Vec<CrossReference>,
}

/// Appendix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appendix {
    /// Appendix title
    pub title: String,

    /// Appendix content
    pub content: String,

    /// Appendix type
    pub appendix_type: AppendixType,
}

/// Appendix types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppendixType {
    Grammar,
    KeywordList,
    OperatorPrecedence,
    StandardLibrary,
    ErrorCodes,
    VersionHistory,
}

/// Index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    /// Term
    pub term: String,

    /// Page references
    pub pages: Vec<String>,

    /// Cross-references
    pub see_also: Vec<String>,
}

/// Cross-reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// Reference text
    pub text: String,

    /// Target
    pub target: String,

    /// Reference type
    pub ref_type: ReferenceType,
}

/// Reference types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    Internal,
    External,
    Api,
    Tutorial,
    Example,
}

/// Search index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    /// Indexed documents
    pub documents: Vec<SearchDocument>,

    /// Term index
    pub terms: HashMap<String, Vec<usize>>,

    /// Metadata
    pub metadata: SearchMetadata,
}

/// Search document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    /// Document ID
    pub id: String,

    /// Document title
    pub title: String,

    /// Document URL
    pub url: String,

    /// Document content
    pub content: String,

    /// Document type
    pub doc_type: DocumentType,

    /// Tags
    pub tags: Vec<String>,
}

/// Document types for search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Module,
    Function,
    Type,
    Constant,
    Macro,
    Tutorial,
    Example,
    Reference,
}

/// Search metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    /// Total documents
    pub total_documents: usize,

    /// Total terms
    pub total_terms: usize,

    /// Index size
    pub index_size: usize,

    /// Last updated
    pub last_updated: std::time::SystemTime,
}

impl DocumentationGenerator {
    /// Create a new documentation generator
    pub fn new(config: DocConfig) -> Result<Self, SemanticError> {
        let parser = parser::DocParser::new(&config)?;
        let renderer = renderer::DocRenderer::new(&config)?;
        let examples = examples::ExampleManager::new(&config)?;

        let documentation = Documentation {
            metadata: ProjectMetadata {
                name: config.project_name.clone(),
                version: config.project_version.clone(),
                description: None,
                authors: vec![],
                license: None,
                homepage: None,
                repository: None,
                documentation_url: None,
                generated_at: std::time::SystemTime::now(),
            },
            api: ApiDocumentation {
                modules: vec![],
                functions: vec![],
                types: vec![],
                constants: vec![],
                macros: vec![],
            },
            tutorials: vec![],
            examples: vec![],
            reference: ReferenceManual {
                sections: vec![],
                appendices: vec![],
                glossary: HashMap::new(),
                index: vec![],
            },
            search_index: None,
        };

        Ok(Self {
            config,
            parser,
            renderer,
            examples,
            documentation,
        })
    }

    /// Generate complete documentation
    pub fn generate(&mut self) -> Result<(), SemanticError> {
        // Parse source files for API documentation
        self.generate_api_documentation()?;

        // Generate tutorials
        if self.config.generate_tutorials {
            self.generate_tutorials()?;
        }

        // Generate examples
        if self.config.generate_examples {
            self.generate_examples()?;
        }

        // Generate reference manual
        self.generate_reference_manual()?;

        // Build search index
        if self.config.search.enabled {
            self.build_search_index()?;
        }

        // Render documentation in all formats
        self.render_documentation()?;

        Ok(())
    }

    /// Generate API documentation from source code
    pub fn generate_api_documentation(&mut self) -> Result<(), SemanticError> {
        for source_dir in &self.config.source_dirs {
            let modules = self.parser.parse_directory(source_dir)?;

            for module in modules {
                self.documentation.api.modules.push(module);
            }
        }

        // Extract functions, types, constants, and macros from modules
        self.extract_api_items()?;

        Ok(())
    }

    /// Generate tutorials
    pub fn generate_tutorials(&mut self) -> Result<(), SemanticError> {
        let tutorial_content = self.create_language_tutorial()?;
        self.documentation.tutorials.push(tutorial_content);

        let advanced_tutorial = self.create_advanced_tutorial()?;
        self.documentation.tutorials.push(advanced_tutorial);

        Ok(())
    }

    /// Generate examples
    pub fn generate_examples(&mut self) -> Result<(), SemanticError> {
        let examples = self.examples.generate_examples()?;
        self.documentation.examples = examples;

        Ok(())
    }

    /// Generate reference manual
    pub fn generate_reference_manual(&mut self) -> Result<(), SemanticError> {
        // Language syntax section
        let syntax_section = self.create_syntax_section()?;
        self.documentation.reference.sections.push(syntax_section);

        // Standard library section
        let stdlib_section = self.create_stdlib_section()?;
        self.documentation.reference.sections.push(stdlib_section);

        // Appendices
        let grammar_appendix = self.create_grammar_appendix()?;
        self.documentation
            .reference
            .appendices
            .push(grammar_appendix);

        Ok(())
    }

    /// Build search index
    pub fn build_search_index(&mut self) -> Result<(), SemanticError> {
        let mut documents = Vec::new();
        let mut doc_id = 0;

        // Index API documentation
        for module in &self.documentation.api.modules {
            documents.push(SearchDocument {
                id: format!("module_{}", doc_id),
                title: module.name.clone(),
                url: format!("api/{}.html", module.path),
                content: format!(
                    "{} {}",
                    module.description.as_deref().unwrap_or(""),
                    module.docs
                ),
                doc_type: DocumentType::Module,
                tags: vec!["api".to_string(), "module".to_string()],
            });
            doc_id += 1;
        }

        // Index tutorials
        for tutorial in &self.documentation.tutorials {
            documents.push(SearchDocument {
                id: format!("tutorial_{}", doc_id),
                title: tutorial.title.clone(),
                url: format!(
                    "tutorials/{}.html",
                    tutorial.title.to_lowercase().replace(' ', "_")
                ),
                content: format!("{} {}", tutorial.description, tutorial.content),
                doc_type: DocumentType::Tutorial,
                tags: tutorial.tags.clone(),
            });
            doc_id += 1;
        }

        // Build term index
        let mut terms = HashMap::new();
        for (idx, doc) in documents.iter().enumerate() {
            let words: Vec<&str> = doc.content.split_whitespace().collect();
            for word in words {
                let normalized_word = word.to_lowercase();
                terms
                    .entry(normalized_word)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        self.documentation.search_index = Some(SearchIndex {
            documents,
            terms,
            metadata: SearchMetadata {
                total_documents: doc_id,
                total_terms: 0, // Would be calculated properly
                index_size: 0,  // Would be calculated properly
                last_updated: std::time::SystemTime::now(),
            },
        });

        Ok(())
    }

    /// Render documentation in all configured formats
    pub fn render_documentation(&mut self) -> Result<(), SemanticError> {
        std::fs::create_dir_all(&self.config.output_dir)?;

        for format in &self.config.output_formats {
            match format {
                OutputFormat::Html { .. } => {
                    self.renderer
                        .render_html(&self.documentation, &self.config.output_dir)?;
                }
                OutputFormat::Markdown { .. } => {
                    self.renderer
                        .render_markdown(&self.documentation, &self.config.output_dir)?;
                }
                OutputFormat::Pdf { .. } => {
                    self.renderer
                        .render_pdf(&self.documentation, &self.config.output_dir)?;
                }
                OutputFormat::Json { .. } => {
                    self.renderer
                        .render_json(&self.documentation, &self.config.output_dir)?;
                }
            }
        }

        Ok(())
    }

    // Helper methods for generating content

    fn extract_api_items(&mut self) -> Result<(), SemanticError> {
        // Extract all functions, types, constants, and macros from modules
        // This would be implemented based on the actual AST structure
        Ok(())
    }

    fn create_language_tutorial(&self) -> Result<Tutorial, SemanticError> {
        Ok(Tutorial {
            title: "AetherScript Language Tutorial".to_string(),
            description: "A comprehensive introduction to the AetherScript programming language"
                .to_string(),
            content: "Welcome to AetherScript!".to_string(),
            sections: vec![TutorialSection {
                title: "Getting Started".to_string(),
                content: "Learn the basics of AetherScript syntax and semantics".to_string(),
                examples: vec![CodeExample {
                    title: Some("Hello World".to_string()),
                    description: Some("Your first AetherScript program".to_string()),
                    code: r#"(println "Hello, World!")"#.to_string(),
                    output: Some("Hello, World!".to_string()),
                    language: "aetherscript".to_string(),
                    example_type: ExampleType::Basic,
                }],
                exercises: vec![],
            }],
            prerequisites: vec![],
            difficulty: DifficultyLevel::Beginner,
            estimated_time: Some("2 hours".to_string()),
            tags: vec!["tutorial".to_string(), "beginner".to_string()],
        })
    }

    fn create_advanced_tutorial(&self) -> Result<Tutorial, SemanticError> {
        Ok(Tutorial {
            title: "Advanced AetherScript Concepts".to_string(),
            description: "Explore advanced features of AetherScript".to_string(),
            content: "Advanced topics in AetherScript programming".to_string(),
            sections: vec![],
            prerequisites: vec!["Basic AetherScript knowledge".to_string()],
            difficulty: DifficultyLevel::Advanced,
            estimated_time: Some("4 hours".to_string()),
            tags: vec!["tutorial".to_string(), "advanced".to_string()],
        })
    }

    fn create_syntax_section(&self) -> Result<ManualSection, SemanticError> {
        Ok(ManualSection {
            title: "Language Syntax".to_string(),
            content: "Complete syntax specification for AetherScript".to_string(),
            subsections: vec![],
            references: vec![],
        })
    }

    fn create_stdlib_section(&self) -> Result<ManualSection, SemanticError> {
        Ok(ManualSection {
            title: "Standard Library".to_string(),
            content: "Reference for the AetherScript standard library".to_string(),
            subsections: vec![],
            references: vec![],
        })
    }

    fn create_grammar_appendix(&self) -> Result<Appendix, SemanticError> {
        Ok(Appendix {
            title: "Language Grammar".to_string(),
            content: "Formal grammar specification for AetherScript".to_string(),
            appendix_type: AppendixType::Grammar,
        })
    }
}

impl Default for DocConfig {
    fn default() -> Self {
        Self {
            project_name: "AetherScript".to_string(),
            project_version: "1.0.0".to_string(),
            output_dir: PathBuf::from("docs"),
            source_dirs: vec![PathBuf::from("src")],
            include_private: false,
            generate_examples: true,
            generate_tutorials: true,
            output_format: OutputFormat::Html {
                javascript: true,
                theme: "default".to_string(),
                search: true,
            },
            output_formats: vec![],
            theme: ThemeConfig::default(),
            search: SearchConfig::default(),
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary_color: "#007acc".to_string(),
            secondary_color: "#f0f0f0".to_string(),
            font_family: "system-ui, sans-serif".to_string(),
            code_font_family: "'Fira Code', 'Courier New', monospace".to_string(),
            custom_css: None,
            logo_url: None,
            favicon_url: None,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            index_type: SearchIndexType::ClientSide,
            max_results: 50,
            weights: SearchWeights {
                title: 2.0,
                description: 1.5,
                content: 1.0,
                tags: 1.2,
            },
        }
    }
}


#[cfg(test)]
mod tests;
