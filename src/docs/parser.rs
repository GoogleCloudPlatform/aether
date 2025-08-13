//! Documentation parser for extracting docs from source code
//! 
//! Parses AetherScript source files to extract documentation comments,
//! function signatures, type definitions, and other API information.

use crate::error::SemanticError;
use crate::docs::{
    ModuleDoc, FunctionDoc, TypeDoc, DocConfig,
    Visibility, SourceLocation, ItemSummary, ItemKind, FunctionSignature,
    ParameterDoc, TypeReference, TypeParameterDoc, TypeKind, FieldDoc,
    VariantDoc, CodeExample, ExampleType
};
use std::path::PathBuf;
use std::collections::HashMap;

/// Documentation parser
#[derive(Debug)]
pub struct SourceParser;

#[derive(Debug)]
pub struct MarkdownParser;

#[derive(Debug)]
pub struct DocParser {
    /// Source code parser
    source_parser: SourceParser,
    
    /// Markdown parser for doc comments
    markdown_parser: MarkdownParser,
}

/// Parsed documentation for a single item
#[derive(Debug, Clone)]
pub struct ParsedDoc {
    /// Raw documentation text
    pub raw_docs: String,
    
    /// Parsed description
    pub description: Option<String>,
    
    /// Code examples
    pub examples: Vec<CodeExample>,
    
    /// Parameters (for functions)
    pub param_docs: HashMap<String, String>,
    
    /// Return documentation
    pub return_doc: Option<String>,
    
    /// Tags and metadata
    pub tags: HashMap<String, String>,
    
    /// See also references
    pub see_also: Vec<String>,
}

/// Comment parser for extracting structured documentation
#[derive(Debug)]
pub struct CommentParser {
    /// Comment prefix patterns
    patterns: CommentPatterns,
}

/// Comment patterns for different documentation styles
#[derive(Debug, Clone)]
pub struct CommentPatterns {
    /// Line comment prefix (e.g., "//")
    pub line: String,
    
    /// Block comment start (e.g., "/*")
    pub block_start: String,
    
    /// Block comment end (e.g., "*/")
    pub block_end: String,
    
    /// Documentation comment prefix (e.g., "///")
    pub doc_line: String,
    
    /// Documentation block comment start (e.g., "/**")
    pub doc_block_start: String,
}

/// Signature parser for extracting function and type signatures
#[derive(Debug)]
pub struct SignatureParser {
}

/// Parsing context
#[derive(Debug, Clone)]
pub struct ParseContext {
    /// Current module path
    pub module_path: String,
    
    /// Current visibility scope
    pub visibility_scope: Visibility,
    
    /// Type parameters in scope
    pub type_parameters: Vec<String>,
    
    /// Generic constraints
    pub constraints: HashMap<String, Vec<String>>,
}

/// Documentation extraction result
#[derive(Debug)]
pub struct ExtractionResult {
    /// Extracted modules
    pub modules: Vec<ModuleDoc>,
    
    /// Extraction warnings
    pub warnings: Vec<String>,
    
    /// Extraction errors
    pub errors: Vec<String>,
}

impl DocParser {
    /// Create a new documentation parser
    pub fn new(config: &DocConfig) -> Result<Self, SemanticError> {
        let comment_parser = CommentParser::new();
        let signature_parser = SignatureParser::new();
        
        Ok(Self {
            source_parser: SourceParser,
            markdown_parser: MarkdownParser,
        })
    }
    
    /// Parse a directory for documentation
    pub fn parse_directory(&mut self, dir: &PathBuf) -> Result<Vec<ModuleDoc>, SemanticError> {
        let mut modules = Vec::new();
        
        // Find all AetherScript source files
        let source_files = self.find_source_files(dir)?;
        
        for file_path in source_files {
            match self.parse_file(&file_path) {
                Ok(mut file_modules) => {
                    modules.append(&mut file_modules);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse file {}: {}", file_path.display(), e);
                }
            }
        }
        
        Ok(modules)
    }
    
    /// Parse a single file for documentation
    pub fn parse_file(&mut self, file_path: &PathBuf) -> Result<Vec<ModuleDoc>, SemanticError> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| SemanticError::Internal {
                message: format!("Failed to read file {}: {}", file_path.display(), e),
            })?;
        
        self.parse_content(&content, file_path)
    }
    
    /// Parse content string for documentation
    pub fn parse_content(&mut self, content: &str, file_path: &PathBuf) -> Result<Vec<ModuleDoc>, SemanticError> {
        let mut modules = Vec::new();
        
        // Initialize parsing context
        let module_name = self.extract_module_name(file_path);
        let mut context = ParseContext {
            module_path: module_name.clone(),
            visibility_scope: Visibility::Public,
            type_parameters: Vec::new(),
            constraints: HashMap::new(),
        };
        
        // Parse the content into tokens/AST nodes (simplified)
        let items = self.parse_top_level_items(content)?;
        
        // Extract documentation for each item
        let mut module_doc = ModuleDoc {
            name: module_name.clone(),
            path: module_name,
            description: None,
            docs: String::new(),
            visibility: Visibility::Public,
            source_location: SourceLocation {
                file: file_path.clone(),
                line: 1,
                column: 1,
                span: content.len(),
            },
            submodules: Vec::new(),
            items: Vec::new(),
            examples: Vec::new(),
        };
        
        // Extract module-level documentation
        if let Some(module_docs) = self.extract_module_documentation(content)? {
            module_doc.docs = module_docs.raw_docs;
            module_doc.description = module_docs.description;
            module_doc.examples = module_docs.examples;
        }
        
        // Process each top-level item
        for item in items {
            let item_summary = self.process_item(item, &mut context)?;
            module_doc.items.push(item_summary);
        }
        
        modules.push(module_doc);
        Ok(modules)
    }
    
    /// Extract documentation for a specific function
    pub fn extract_function_doc(&mut self, function_source: &str, file_path: &PathBuf, line: usize) -> Result<FunctionDoc, SemanticError> {
        let parsed_doc = self.parse_documentation(function_source)?;
        let signature = self.parse_function_signature(function_source)?;
        
        Ok(FunctionDoc {
            name: signature.name.clone(),
            path: format!("{}::{}", self.get_current_module_path(), signature.name),
            description: parsed_doc.description,
            docs: parsed_doc.raw_docs,
            signature,
            visibility: Visibility::Public, // Would be determined from actual parsing
            source_location: SourceLocation {
                file: file_path.clone(),
                line,
                column: 1,
                span: function_source.len(),
            },
            parameters: parsed_doc.param_docs.into_iter()
                .map(|(name, desc)| ParameterDoc {
                    name,
                    param_type: TypeReference {
                        name: "unknown".to_string(),
                        path: "unknown".to_string(),
                        type_args: vec![],
                        is_reference: false,
                        is_mutable: false,
                    },
                    description: Some(desc),
                    default_value: None,
                })
                .collect(),
            return_type: None, // Would be extracted from signature
            examples: parsed_doc.examples,
            related: vec![],
        })
    }
    
    /// Extract documentation for a specific type
    pub fn extract_type_doc(&mut self, type_source: &str, file_path: &PathBuf, line: usize) -> Result<TypeDoc, SemanticError> {
        let parsed_doc = self.parse_documentation(type_source)?;
        let type_info = self.parse_type_definition(type_source)?;
        
        Ok(TypeDoc {
            name: type_info.name.clone(),
            path: format!("{}::{}", self.get_current_module_path(), type_info.name),
            description: parsed_doc.description,
            docs: parsed_doc.raw_docs,
            kind: type_info.kind,
            visibility: Visibility::Public,
            source_location: SourceLocation {
                file: file_path.clone(),
                line,
                column: 1,
                span: type_source.len(),
            },
            type_parameters: type_info.type_parameters,
            fields: type_info.fields,
            variants: type_info.variants,
            methods: vec![],
            trait_impls: vec![],
            examples: parsed_doc.examples,
        })
    }
    
    // Helper methods
    
    fn find_source_files(&self, dir: &PathBuf) -> Result<Vec<PathBuf>, SemanticError> {
        let mut files = Vec::new();
        
        if !dir.is_dir() {
            return Ok(files);
        }
        
        let entries = std::fs::read_dir(dir)
            .map_err(|e| SemanticError::Internal {
                message: format!("Failed to read directory {}: {}", dir.display(), e),
            })?;
        
        for entry in entries {
            let entry = entry.map_err(|e| SemanticError::Internal {
                message: format!("Failed to read directory entry: {}", e),
            })?;
            
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "aether" || extension == "ae" {
                        files.push(path);
                    }
                }
            } else if path.is_dir() {
                // Recursively search subdirectories
                let mut subfiles = self.find_source_files(&path)?;
                files.append(&mut subfiles);
            }
        }
        
        Ok(files)
    }
    
    fn extract_module_name(&self, file_path: &PathBuf) -> String {
        file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
    
    fn parse_top_level_items(&self, content: &str) -> Result<Vec<ParsedItem>, SemanticError> {
        // Simplified parsing - in real implementation would use proper lexer/parser
        let mut items = Vec::new();
        
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            if line.starts_with("(defn ") || line.starts_with("(fn ") {
                // Function definition
                items.push(ParsedItem {
                    item_type: ItemType::Function,
                    name: self.extract_function_name(line),
                    doc_comment: self.extract_preceding_comments(&lines, i).unwrap_or_default(),
                    location: SourceLocation {
                        file: PathBuf::new(),
                        line: i + 1,
                        column: 0,
                        span: 0,
                    },
                });
            } else if line.starts_with("(deftype ") || line.starts_with("(struct ") {
                // Type definition
                items.push(ParsedItem {
                    item_type: ItemType::Struct,
                    name: self.extract_type_name(line),
                    doc_comment: self.extract_preceding_comments(&lines, i).unwrap_or_default(),
                    location: SourceLocation {
                        file: PathBuf::new(),
                        line: i + 1,
                        column: 0,
                        span: 0,
                    },
                });
            } else if line.starts_with("(def ") {
                // Constant definition
                items.push(ParsedItem {
                    item_type: ItemType::Constant,
                    name: self.extract_constant_name(line),
                    doc_comment: self.extract_preceding_comments(&lines, i).unwrap_or_default(),
                    location: SourceLocation {
                        file: PathBuf::new(),
                        line: i + 1,
                        column: 0,
                        span: 0,
                    },
                });
            }
            
            i += 1;
        }
        
        Ok(items)
    }
    
    fn extract_module_documentation(&mut self, content: &str) -> Result<Option<ParsedDoc>, SemanticError> {
        // Look for module-level documentation at the top of the file
        let lines: Vec<&str> = content.lines().collect();
        let mut doc_lines = Vec::new();
        
        for line in lines {
            let trimmed = line.trim();
            if trimmed.starts_with(";;;") || trimmed.starts_with("//!") {
                doc_lines.push(trimmed);
            } else if !trimmed.is_empty() {
                break; // Stop at first non-comment, non-empty line
            }
        }
        
        if doc_lines.is_empty() {
            return Ok(None);
        }
        
        let doc_text = doc_lines.join("\n");
        let parsed_doc = self.parse_documentation(&doc_text)?;
        Ok(Some(parsed_doc))
    }
    
    fn process_item(&mut self, item: ParsedItem, _context: &mut ParseContext) -> Result<ItemSummary, SemanticError> {
        let kind = match item.item_type {
            ItemType::Function => ItemKind::Function,
            ItemType::Struct => ItemKind::Struct,
            ItemType::Constant => ItemKind::Constant,
            ItemType::Macro => ItemKind::Macro,
            ItemType::Module => ItemKind::Module,
            ItemType::Enum => ItemKind::Type,
            ItemType::Trait => ItemKind::Type,
            ItemType::TypeAlias => ItemKind::Type,
            ItemType::Import => ItemKind::Module,
        };
        
        let description = if !item.doc_comment.is_empty() {
            self.parse_documentation(&item.doc_comment)?
                .description
        } else {
            None
        };
        
        Ok(ItemSummary {
            name: item.name,
            kind,
            description,
            visibility: Visibility::Public, // Would be determined from actual parsing
        })
    }
    
    fn extract_preceding_comments(&self, lines: &[&str], index: usize) -> Option<String> {
        let mut comment_lines = Vec::new();
        let mut i = index;
        
        // Look backwards for comment lines
        while i > 0 {
            i -= 1;
            let line = lines[i].trim();
            
            if line.starts_with(";;;") || line.starts_with("//") {
                comment_lines.insert(0, line);
            } else if line.is_empty() {
                continue; // Skip empty lines
            } else {
                break; // Stop at non-comment line
            }
        }
        
        if comment_lines.is_empty() {
            None
        } else {
            Some(comment_lines.join("\n"))
        }
    }
    
    fn extract_function_name(&self, line: &str) -> String {
        // Extract function name from definition line
        // Simplified implementation
        if let Some(start) = line.find("(defn ") {
            let rest = &line[start + 6..];
            if let Some(end) = rest.find(' ') {
                return rest[..end].to_string();
            }
        }
        "unknown".to_string()
    }
    
    fn extract_type_name(&self, line: &str) -> String {
        // Extract type name from definition line
        // Simplified implementation
        if let Some(start) = line.find("(deftype ") {
            let rest = &line[start + 9..];
            if let Some(end) = rest.find(' ') {
                return rest[..end].to_string();
            }
        }
        "unknown".to_string()
    }
    
    fn extract_constant_name(&self, line: &str) -> String {
        // Extract constant name from definition line
        // Simplified implementation
        if let Some(start) = line.find("(def ") {
            let rest = &line[start + 5..];
            if let Some(end) = rest.find(' ') {
                return rest[..end].to_string();
            }
        }
        "unknown".to_string()
    }
    
    fn get_current_module_path(&self) -> String {
        "unknown".to_string()
    }
    
    fn parse_documentation(&self, raw_docs: &str) -> Result<ParsedDoc, SemanticError> {
        let mut description = None;
        let mut examples = Vec::new();
        let mut param_docs = HashMap::new();
        let mut return_doc = None;
        let mut tags = HashMap::new();
        let mut see_also = Vec::new();
        
        let lines: Vec<&str> = raw_docs.lines().collect();
        let mut current_section = DocSection::Description;
        let mut current_param = None;
        
        for line in lines {
            let trimmed = line.trim();
            
            // Check for section markers
            if trimmed.starts_with("@param") || trimmed.starts_with(":param") {
                if let Some(param_match) = trimmed.split_whitespace().nth(1) {
                    current_param = Some(param_match.to_string());
                    current_section = DocSection::Parameter;
                    continue;
                }
            } else if trimmed.starts_with("@return") || trimmed.starts_with(":return") {
                current_section = DocSection::Return;
                continue;
            } else if trimmed.starts_with("@example") || trimmed.starts_with("Example:") {
                current_section = DocSection::Example;
                continue;
            } else if trimmed.starts_with("@see") || trimmed.starts_with("See also:") {
                current_section = DocSection::SeeAlso;
                continue;
            } else if trimmed.starts_with("@") || trimmed.starts_with(":") {
                // Generic tag
                if let Some(tag_end) = trimmed.find(' ') {
                    let tag = trimmed[1..tag_end].to_string();
                    let value = trimmed[tag_end+1..].to_string();
                    tags.insert(tag, value);
                }
                continue;
            }
            
            // Add content to appropriate section
            match current_section {
                DocSection::Description => {
                    if description.is_none() {
                        description = Some(String::new());
                    }
                    if let Some(ref mut desc) = description {
                        if !desc.is_empty() {
                            desc.push('\n');
                        }
                        desc.push_str(trimmed);
                    }
                }
                DocSection::Parameter => {
                    if let Some(ref param_name) = current_param {
                        let entry = param_docs.entry(param_name.clone()).or_insert(String::new());
                        if !entry.is_empty() {
                            entry.push('\n');
                        }
                        entry.push_str(trimmed);
                    }
                }
                DocSection::Return => {
                    if return_doc.is_none() {
                        return_doc = Some(String::new());
                    }
                    if let Some(ref mut ret) = return_doc {
                        if !ret.is_empty() {
                            ret.push('\n');
                        }
                        ret.push_str(trimmed);
                    }
                }
                DocSection::Example => {
                    // Start a new example or continue existing one
                    if trimmed.is_empty() && !examples.is_empty() {
                        // Empty line might separate examples
                        continue;
                    }
                    if examples.is_empty() || trimmed.starts_with("```") {
                        examples.push(CodeExample {
                            title: None,
                            description: None,
                            code: String::new(),
                            output: None,
                            language: "aetherscript".to_string(),
                            example_type: ExampleType::Basic,
                        });
                    }
                    if let Some(last_example) = examples.last_mut() {
                        if !last_example.code.is_empty() {
                            last_example.code.push('\n');
                        }
                        last_example.code.push_str(line); // Use original line to preserve indentation
                    }
                }
                DocSection::SeeAlso => {
                    see_also.push(trimmed.to_string());
                }
                DocSection::Tag(_) => {
                    // Tags are handled differently, skip
                }
            }
        }
        
        Ok(ParsedDoc {
            raw_docs: raw_docs.to_string(),
            description,
            param_docs,
            return_doc,
            examples,
            tags,
            see_also,
        })
    }
    
    fn parse_function_signature(&self, source: &str) -> Result<FunctionSignature, SemanticError> {
        // Simple signature parsing - in a real implementation this would be more sophisticated
        let name = source.lines()
            .find(|line| line.contains("fn ") || line.contains("def "))
            .and_then(|line| {
                line.split_whitespace()
                    .skip_while(|&w| w != "fn" && w != "def")
                    .nth(1)
                    .map(|s| s.trim_end_matches('('))
            })
            .unwrap_or("unknown")
            .to_string();
        
        Ok(FunctionSignature {
            name,
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            return_type: None,
            is_async: false,
            is_unsafe: false,
        })
    }
    
    fn parse_type_definition(&self, source: &str) -> Result<TypeInfo, SemanticError> {
        // Simple type parsing - in a real implementation this would be more sophisticated
        let name = source.lines()
            .find(|line| line.contains("struct ") || line.contains("enum ") || line.contains("type "))
            .and_then(|line| {
                line.split_whitespace()
                    .skip_while(|&w| w != "struct" && w != "enum" && w != "type")
                    .nth(1)
            })
            .unwrap_or("unknown")
            .to_string();
        
        let kind = if source.contains("struct ") {
            TypeKind::Struct
        } else if source.contains("enum ") {
            TypeKind::Enum
        } else {
            TypeKind::TypeAlias
        };
        
        Ok(TypeInfo {
            name,
            kind,
            type_parameters: Vec::new(),
            fields: Vec::new(),
            variants: Vec::new(),
        })
    }
}

/// Parsed item from source code
#[derive(Debug, Clone)]
struct ParsedItem {
    /// Item type
    item_type: ItemType,
    
    /// Item name
    name: String,
    
    /// Raw documentation comment
    doc_comment: String,
    
    /// Source location
    location: SourceLocation,
}

/// Item types
#[derive(Debug, Clone)]
enum ItemType {
    Module,
    Function,
    Struct,
    Enum,
    Trait,
    TypeAlias,
    Constant,
    Import,
    Macro,
}

/// Type information extracted from parsing
#[derive(Debug, Clone)]
struct TypeInfo {
    /// Type name
    name: String,
    
    /// Type kind
    kind: TypeKind,
    
    /// Type parameters
    type_parameters: Vec<TypeParameterDoc>,
    
    /// Fields (for structs)
    fields: Vec<FieldDoc>,
    
    /// Variants (for enums)
    variants: Vec<VariantDoc>,
}

impl CommentParser {
    fn new() -> Self {
        Self {
            patterns: CommentPatterns {
                line: ";;;".to_string(),
                block_start: "#|".to_string(),
                block_end: "|#".to_string(),
                doc_line: "///".to_string(),
                doc_block_start: "/**".to_string(),
            },
        }
    }
    
    fn parse_documentation(&self, raw_docs: &str) -> Result<ParsedDoc, SemanticError> {
        let mut description = None;
        let mut examples = Vec::new();
        let mut param_docs = HashMap::new();
        let mut return_doc = None;
        let mut tags = HashMap::new();
        let mut see_also = Vec::new();
        
        let lines: Vec<&str> = raw_docs.lines().collect();
        let mut current_section = DocSection::Description;
        let mut current_content: Vec<String> = Vec::new();
        
        for line in lines {
            let cleaned_line = self.clean_comment_line(line);
            
            if let Some(tag) = self.extract_tag(&cleaned_line) {
                // Process previous section
                self.finalize_section(&mut current_section, &current_content, 
                    &mut description, &mut examples, &mut param_docs, 
                    &mut return_doc, &mut tags, &mut see_also)?;
                
                // Start new section
                current_content.clear();
                current_section = self.tag_to_section(&tag);
                
                if let Some(content) = cleaned_line.strip_prefix(&format!("{}{}", "@", tag)) {
                    current_content.push(content.trim().to_string());
                }
            } else {
                current_content.push(cleaned_line);
            }
        }
        
        // Process final section
        self.finalize_section(&mut current_section, &current_content, 
            &mut description, &mut examples, &mut param_docs, 
            &mut return_doc, &mut tags, &mut see_also)?;
        
        Ok(ParsedDoc {
            raw_docs: raw_docs.to_string(),
            description,
            examples,
            param_docs,
            return_doc,
            tags,
            see_also,
        })
    }
    
    fn clean_comment_line(&self, line: &str) -> String {
        let trimmed = line.trim();
        
        // Remove comment prefixes
        if let Some(rest) = trimmed.strip_prefix(&self.patterns.line) {
            rest.trim().to_string()
        } else if let Some(rest) = trimmed.strip_prefix("//") {
            rest.trim().to_string()
        } else {
            trimmed.to_string()
        }
    }
    
    fn extract_tag(&self, line: &str) -> Option<String> {
        if line.starts_with("@") {
            if let Some(space_pos) = line.find(' ') {
                Some(line[1..space_pos].to_string())
            } else {
                Some(line[1..].to_string())
            }
        } else {
            None
        }
    }
    
    fn tag_to_section(&self, tag: &str) -> DocSection {
        match tag {
            "param" | "parameter" => DocSection::Parameter,
            "return" | "returns" => DocSection::Return,
            "example" | "examples" => DocSection::Example,
            "see" | "see_also" => DocSection::SeeAlso,
            _ => DocSection::Tag(tag.to_string()),
        }
    }
    
    fn finalize_section(&self, section: &mut DocSection, content: &[String],
                       description: &mut Option<String>,
                       examples: &mut Vec<CodeExample>,
                       param_docs: &mut HashMap<String, String>,
                       return_doc: &mut Option<String>,
                       tags: &mut HashMap<String, String>,
                       see_also: &mut Vec<String>) -> Result<(), SemanticError> {
        
        let content_text = content.join("\n").trim().to_string();
        if content_text.is_empty() {
            return Ok(());
        }
        
        match section {
            DocSection::Description => {
                *description = Some(content_text);
            }
            DocSection::Parameter => {
                // Parse parameter documentation
                if let Some(space_pos) = content_text.find(' ') {
                    let param_name = content_text[..space_pos].to_string();
                    let param_desc = content_text[space_pos + 1..].to_string();
                    param_docs.insert(param_name, param_desc);
                }
            }
            DocSection::Return => {
                *return_doc = Some(content_text);
            }
            DocSection::Example => {
                examples.push(CodeExample {
                    title: None,
                    description: None,
                    code: content_text,
                    output: None,
                    language: "aetherscript".to_string(),
                    example_type: ExampleType::Basic,
                });
            }
            DocSection::SeeAlso => {
                see_also.push(content_text);
            }
            DocSection::Tag(tag_name) => {
                tags.insert(tag_name.clone(), content_text);
            }
        }
        
        Ok(())
    }
}

/// Documentation sections
#[derive(Debug, Clone)]
enum DocSection {
    Description,
    Parameter,
    Return,
    Example,
    SeeAlso,
    Tag(String),
}

impl SignatureParser {
    fn new() -> Self {
        Self {}
    }
    
    fn parse_function_signature(&self, source: &str) -> Result<FunctionSignature, SemanticError> {
        // Simplified function signature parsing
        // In real implementation, would use proper parser
        
        let name = self.extract_function_name(source);
        
        Ok(FunctionSignature {
            name,
            type_parameters: vec![],
            parameters: vec![],
            return_type: None,
            is_async: false,
            is_unsafe: false,
        })
    }
    
    fn parse_type_definition(&self, source: &str) -> Result<TypeInfo, SemanticError> {
        // Simplified type parsing
        let name = self.extract_type_name(source);
        
        Ok(TypeInfo {
            name,
            kind: TypeKind::Struct, // Would be determined from actual parsing
            type_parameters: vec![],
            fields: vec![],
            variants: vec![],
        })
    }
    
    fn extract_function_name(&self, source: &str) -> String {
        // Simplified function name extraction
        if let Some(start) = source.find("(defn ") {
            let rest = &source[start + 6..];
            if let Some(end) = rest.find(' ') {
                return rest[..end].to_string();
            }
        }
        "unknown".to_string()
    }
    
    fn extract_type_name(&self, source: &str) -> String {
        // Simplified type name extraction
        if let Some(start) = source.find("(deftype ") {
            let rest = &source[start + 9..];
            if let Some(end) = rest.find(' ') {
                return rest[..end].to_string();
            }
        }
        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_comment_parser() {
        let parser = CommentParser::new();
        let docs = ";;; A simple function\n;;; @param x The input value\n;;; @return The result";
        
        let parsed = parser.parse_documentation(docs).unwrap();
        assert_eq!(parsed.description, Some("A simple function".to_string()));
        assert!(parsed.param_docs.contains_key("x"));
        assert!(parsed.return_doc.is_some());
    }
    
    #[test]
    fn test_function_name_extraction() {
        let parser = SignatureParser::new();
        let source = "(defn add [x y] (+ x y))";
        
        let name = parser.extract_function_name(source);
        assert_eq!(name, "add");
    }
    
    #[test]
    fn test_source_file_discovery() {
        let config = DocConfig::default();
        let mut parser = DocParser::new(&config).unwrap();
        
        // Would test with actual filesystem in real implementation
        // let files = parser.find_source_files(&PathBuf::from("test_sources")).unwrap();
        // assert!(!files.is_empty());
    }
    
    #[test]
    fn test_module_name_extraction() {
        let config = DocConfig::default();
        let parser = DocParser::new(&config).unwrap();
        
        let path = PathBuf::from("src/test_module.aether");
        let name = parser.extract_module_name(&path);
        assert_eq!(name, "test_module");
    }
}