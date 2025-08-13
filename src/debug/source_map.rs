//! Source mapping for AetherScript debugging
//!
//! Provides source-level debugging support by mapping between source code
//! locations and generated code addresses.

use crate::error::SemanticError;
use crate::mir::Program;
use crate::debug::debugger::VariableInfo;
use crate::error::SourceLocation;
use std::collections::{HashMap, BTreeMap};
use serde::{Serialize, Deserialize};

/// Source map generator
#[derive(Debug, Default)]
pub struct SourceMapGenerator {
    /// Source mappings
    mappings: Vec<SourceMapping>,
    
    /// Source files
    source_files: Vec<SourceFile>,
    
    /// Names used in the source map
    names: Vec<String>,
    
    /// Generated source map
    source_map: Option<SourceMap>,
}

/// Source mapping entry
#[derive(Debug, Clone)]
pub struct SourceMapping {
    /// Generated code location
    pub generated: GeneratedLocation,
    
    /// Original source location
    pub original: SourceLocation,
    
    /// Symbol name (if applicable)
    pub name: Option<String>,
}

/// Location in generated code
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GeneratedLocation {
    /// Generated line number (0-based)
    pub line: u32,
    
    /// Generated column number (0-based)
    pub column: u32,
    
    /// Generated address (for binary mappings)
    pub address: Option<u64>,
}

/// Source file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFile {
    /// File name/path
    pub name: String,
    
    /// File content (optional, for inline source maps)
    pub content: Option<String>,
    
    /// Source root (optional)
    pub source_root: Option<String>,
}

/// Source map in standard format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    /// Source map version
    pub version: u8,
    
    /// Output file name
    pub file: Option<String>,
    
    /// Source root
    pub source_root: Option<String>,
    
    /// Source files
    pub sources: Vec<String>,
    
    /// Source file contents (optional)
    pub sources_content: Option<Vec<Option<String>>>,
    
    /// Symbol names
    pub names: Vec<String>,
    
    /// VLQ-encoded mappings
    pub mappings: String,
}

/// Address-to-source mapping for binary debugging
#[derive(Debug, Default)]
pub struct AddressMap {
    /// Mappings from address to source location
    address_to_source: BTreeMap<u64, SourceLocation>,
    
    /// Mappings from source location to addresses
    source_to_addresses: HashMap<SourceLocation, Vec<u64>>,
    
    /// Function boundaries
    function_bounds: HashMap<String, (u64, u64)>,
}

/// Line mapping for source-level debugging
#[derive(Debug, Default)]
pub struct LineMap {
    /// Line-to-line mappings
    line_mappings: HashMap<(String, u32), Vec<GeneratedLocation>>,
    
    /// Generated-to-source line mappings
    generated_mappings: HashMap<GeneratedLocation, SourceLocation>,
}

/// Variable location mapping
#[derive(Debug, Default)]
pub struct VariableMap {
    /// Mapping from variable name to its type and location
    variables: HashMap<String, VariableInfo>,
    /// Mapping from scope::name to variable locations
    locations: HashMap<String, VariableLocation>,
}

/// Variable location information
#[derive(Debug, Clone)]
pub struct VariableLocation {
    /// Variable name
    pub name: String,
    
    /// Location type
    pub location_type: VariableLocationType,
    
    /// Valid address range
    pub range: LocationRange,
    
    /// Source location where variable is declared
    pub declaration: SourceLocation,
}

/// Types of variable locations
#[derive(Debug, Clone)]
pub enum VariableLocationType {
    /// In CPU register
    Register { register: String },
    
    /// On stack at offset
    Stack { offset: i32 },
    
    /// At memory address
    Memory { address: u64 },
    
    /// Optimized away
    OptimizedAway,
    
    /// Computed by expression
    Expression { expression: Vec<u8> },
}

/// Address range for variable validity
#[derive(Debug, Clone)]
pub struct LocationRange {
    /// Start address
    pub start: u64,
    
    /// End address
    pub end: u64,
}

impl SourceMapGenerator {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Generate source mappings for a program
    pub fn generate_for_program(&mut self, program: &Program) -> Result<(), SemanticError> {
        // Clear previous mappings
        self.mappings.clear();
        self.source_files.clear();
        self.names.clear();
        
        // Process each function
        for (name, function) in &program.functions {
            self.generate_function_mappings(name, function)?;
        }
        
        // Generate the source map
        self.generate_source_map()?;
        
        Ok(())
    }
    
    /// Generate mappings for a function
    fn generate_function_mappings(&mut self, name: &str, function: &crate::mir::Function) -> Result<(), SemanticError> {
        // Add function name to names list
        if !self.names.iter().any(|n| n == name) {
            self.names.push(name.to_string());
        }
        
        // Process each basic block
        for (block_id, block) in &function.basic_blocks {
            self.generate_block_mappings(*block_id as usize, block)?;
        }
        
        Ok(())
    }
    
    /// Generate mappings for a basic block
    fn generate_block_mappings(&mut self, block_id: usize, block: &crate::mir::BasicBlock) -> Result<(), SemanticError> {
        // Calculate generated location for block start
        let generated_line = block_id as u32 * 10; // Simplified mapping
        let generated_column = 0;
        
        // Create mapping for block start
        let mapping = SourceMapping {
            generated: GeneratedLocation {
                line: generated_line,
                column: generated_column,
                address: Some(block_id as u64 * 0x1000), // Placeholder address
            },
            original: SourceLocation {
                file: "main.aether".to_string(), // Simplified
                line: block_id,
                column: 0,
                offset: 0,
            },
            name: None,
        };
        
        self.mappings.push(mapping);
        
        // Process statements in the block
        for (stmt_idx, _statement) in block.statements.iter().enumerate() {
            let stmt_mapping = SourceMapping {
                generated: GeneratedLocation {
                    line: generated_line,
                    column: (stmt_idx + 1) as u32 * 4,
                    address: Some((block_id as u64 * 0x1000) + (stmt_idx as u64 * 4)),
                },
                original: SourceLocation {
                    file: "main.aether".to_string(),
                    line: block_id,
                    column: (stmt_idx + 1),
                    offset: 0,
                },
                name: None,
            };
            
            self.mappings.push(stmt_mapping);
        }
        
        Ok(())
    }
    
    /// Generate the source map in standard format
    fn generate_source_map(&mut self) -> Result<(), SemanticError> {
        // Collect unique source files
        let mut source_files = std::collections::HashSet::new();
        for mapping in &self.mappings {
            source_files.insert(mapping.original.file.clone());
        }
        
        let sources: Vec<String> = source_files.into_iter().collect();
        
        // Generate VLQ-encoded mappings (simplified)
        let mappings_string = self.encode_mappings()?;
        
        self.source_map = Some(SourceMap {
            version: 3,
            file: Some("output.js".to_string()), // Placeholder
            source_root: None,
            sources,
            sources_content: None,
            names: self.names.clone(),
            mappings: mappings_string,
        });
        
        Ok(())
    }
    
    /// Encode mappings as VLQ string (simplified implementation)
    fn encode_mappings(&self) -> Result<String, SemanticError> {
        // This is a simplified encoding - real VLQ encoding would be more complex
        let mut result = String::new();
        
        for (i, mapping) in self.mappings.iter().enumerate() {
            if i > 0 {
                result.push(',');
            }
            
            // Simplified encoding: just concatenate values
            result.push_str(&format!("{}:{}:{}:{}", 
                mapping.generated.line,
                mapping.generated.column,
                mapping.original.line,
                mapping.original.column
            ));
        }
        
        Ok(result)
    }
    
    /// Get generated source map
    pub fn source_map(&self) -> Option<&SourceMap> {
        self.source_map.as_ref()
    }
    
    /// Find source location for generated location
    pub fn find_source_location(&self, generated: &GeneratedLocation) -> Option<&SourceLocation> {
        self.mappings
            .iter()
            .find(|mapping| mapping.generated == *generated)
            .map(|mapping| &mapping.original)
    }
    
    /// Find generated locations for source location
    pub fn find_generated_locations(&self, source: &SourceLocation) -> Vec<&GeneratedLocation> {
        self.mappings
            .iter()
            .filter(|mapping| mapping.original == *source)
            .map(|mapping| &mapping.generated)
            .collect()
    }
    
    /// Add source file
    pub fn add_source_file(&mut self, name: String, content: Option<String>) {
        let source_file = SourceFile {
            name,
            content,
            source_root: None,
        };
        self.source_files.push(source_file);
    }
    
    /// Get all mappings
    pub fn mappings(&self) -> &[SourceMapping] {
        &self.mappings
    }
    
    /// Get source files
    pub fn source_files(&self) -> &[SourceFile] {
        &self.source_files
    }
}

impl AddressMap {
    /// Add address mapping
    pub fn add_mapping(&mut self, address: u64, source: SourceLocation) {
        self.address_to_source.insert(address, source.clone());
        self.source_to_addresses
            .entry(source)
            .or_insert_with(Vec::new)
            .push(address);
    }
    
    /// Find source location for address
    pub fn find_source_for_address(&self, address: u64) -> Option<&SourceLocation> {
        // Find the closest mapping at or before the address
        self.address_to_source
            .range(..=address)
            .next_back()
            .map(|(_, location)| location)
    }
    
    /// Find addresses for source location
    pub fn find_addresses_for_source(&self, source: &SourceLocation) -> Option<&[u64]> {
        self.source_to_addresses.get(source).map(|v| v.as_slice())
    }
    
    /// Add function boundaries
    pub fn add_function_bounds(&mut self, name: String, start: u64, end: u64) {
        self.function_bounds.insert(name, (start, end));
    }
    
    /// Find function containing address
    pub fn find_function_for_address(&self, address: u64) -> Option<&str> {
        self.function_bounds
            .iter()
            .find(|(_, (start, end))| address >= *start && address < *end)
            .map(|(name, _)| name.as_str())
    }
}

impl LineMap {
    /// Add line mapping
    pub fn add_mapping(&mut self, source_file: String, source_line: u32, generated: GeneratedLocation) {
        let source_location = SourceLocation {
            file: source_file.clone(),
            line: source_line as usize,
            column: 0,
            offset: 0,
        };
        
        self.line_mappings
            .entry((source_file, source_line))
            .or_insert_with(Vec::new)
            .push(generated.clone());
        
        self.generated_mappings.insert(generated, source_location);
    }
    
    /// Find generated locations for source line
    pub fn find_generated_for_line(&self, file: &str, line: u32) -> Option<&[GeneratedLocation]> {
        self.line_mappings
            .get(&(file.to_string(), line))
            .map(|v| v.as_slice())
    }
    
    /// Find source location for generated location
    pub fn find_source_for_generated(&self, generated: &GeneratedLocation) -> Option<&SourceLocation> {
        self.generated_mappings.get(generated)
    }
}

impl VariableMap {
    /// Add variable location
    pub fn add_variable_location(&mut self, scope: String, location: VariableLocation) {
        // Store the variable info using the variable name as key
        let var_info = VariableInfo {
            name: location.name.clone(),
            type_name: "unknown".to_string(), // Would be filled in real implementation
            value: "".to_string(),
            address: match &location.location_type {
                VariableLocationType::Memory { address } => Some(*address),
                _ => None,
            },
            is_mutable: false,
        };
        let key = format!("{}::{}", scope, location.name);
        self.variables.insert(key.clone(), var_info);
        self.locations.insert(key, location);
    }
    
    /// Find variable location at address
    pub fn find_variable_at_address(&self, scope: &str, name: &str, address: u64) -> Option<&VariableLocation> {
        let key = format!("{}::{}", scope, name);
        self.locations.get(&key).filter(|loc| {
            // Check if address is within the variable's range
            address >= loc.range.start && address < loc.range.end
        })
    }
    
    /// Get all variables in scope
    pub fn variables_in_scope(&self, _scope: &str) -> Option<&[VariableLocation]> {
        // In a real implementation, we would return variables in scope
        None
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{Builder, Program};
    use std::collections::HashMap;
    
    #[test]
    fn test_source_map_generator() {
        let mut generator = SourceMapGenerator::new();
        let program = create_test_program();
        
        assert!(generator.generate_for_program(&program).is_ok());
        assert!(!generator.mappings().is_empty());
    }
    
    #[test]
    fn test_source_mapping() {
        let mapping = SourceMapping {
            generated: GeneratedLocation {
                line: 10,
                column: 5,
                address: Some(0x1000),
            },
            original: SourceLocation {
                file: "test.aether".to_string(),
                line: 1,
                column: 0,
                offset: 0,
            },
            name: Some("test_function".to_string()),
        };
        
        assert_eq!(mapping.generated.line, 10);
        assert_eq!(mapping.original.file, "test.aether");
        assert_eq!(mapping.name, Some("test_function".to_string()));
    }
    
    #[test]
    fn test_address_map() {
        let mut address_map = AddressMap::default();
        
        let source = SourceLocation {
            file: "test.aether".to_string(),
            line: 5,
            column: 10,
            offset: 0,
        };
        
        address_map.add_mapping(0x1000, source.clone());
        
        let found = address_map.find_source_for_address(0x1000);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &source);
    }
    
    #[test]
    fn test_line_map() {
        let mut line_map = LineMap::default();
        
        let generated = GeneratedLocation {
            line: 20,
            column: 0,
            address: None,
        };
        
        line_map.add_mapping("test.aether".to_string(), 5, generated.clone());
        
        let found = line_map.find_generated_for_line("test.aether", 5);
        assert!(found.is_some());
        assert_eq!(found.unwrap().len(), 1);
        assert_eq!(found.unwrap()[0], generated);
    }
    
    #[test]
    fn test_variable_map() {
        let mut var_map = VariableMap::default();
        
        let var_location = VariableLocation {
            name: "test_var".to_string(),
            location_type: VariableLocationType::Stack { offset: -16 },
            range: LocationRange { start: 0x1000, end: 0x2000 },
            declaration: SourceLocation {
                file: "test.aether".to_string(),
                line: 10,
                column: 5,
                offset: 0,
            },
        };
        
        var_map.add_variable_location("main".to_string(), var_location);
        
        let found = var_map.find_variable_at_address("main", "test_var", 0x1500);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test_var");
    }
    
    #[test]
    fn test_source_map_json() {
        let source_map = SourceMap {
            version: 3,
            file: Some("output.js".to_string()),
            source_root: None,
            sources: vec!["input.aether".to_string()],
            sources_content: None,
            names: vec!["main".to_string()],
            mappings: "AAAA,SAAS".to_string(),
        };
        
        // Test serialization
        let json = serde_json::to_string(&source_map);
        assert!(json.is_ok());
        
        // Test deserialization
        let deserialized: Result<SourceMap, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
    
    fn create_test_program() -> Program {
        use crate::mir::*;
        use crate::types::Type;
        
        let mut functions = HashMap::new();
        
        // Create a simple test function
        let mut basic_blocks = HashMap::new();
        let block_id: BasicBlockId = 0;
        basic_blocks.insert(block_id, BasicBlock {
            id: block_id,
            statements: vec![],
            terminator: Terminator::Return,
        });
        
        functions.insert("test_function".to_string(), Function {
            name: "test_function".to_string(),
            parameters: vec![],
            return_type: Type::Primitive(crate::ast::PrimitiveType::Void),
            locals: HashMap::new(),
            basic_blocks,
            entry_block: block_id,
            return_local: None,
        });
        
        Program {
            functions,
            global_constants: HashMap::new(),
            external_functions: HashMap::new(),
            type_definitions: HashMap::new(),
        }
    }
}