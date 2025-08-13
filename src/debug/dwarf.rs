//! DWARF debug information generation for AetherScript
//!
//! Generates DWARF debugging information that can be consumed by debuggers like GDB and LLDB.
//! Includes support for line numbers, variable locations, and function information.

use crate::error::SemanticError;
use crate::mir::{Program, Function, BasicBlock};
use crate::llvm_backend::LLVMBackend;
use std::collections::HashMap;
use std::path::PathBuf;

/// DWARF debug information generator
#[derive(Debug)]
pub struct DwarfGenerator {
    /// Debug information level (0=none, 1=minimal, 2=default, 3=full)
    debug_level: u8,
    
    /// Compilation unit information
    compilation_units: Vec<CompilationUnit>,
    
    /// Debug information entries
    debug_entries: Vec<DebugInfoEntry>,
    
    /// Line number program
    line_program: LineProgram,
    
    /// Source file mapping
    source_files: HashMap<String, u32>,
}

/// Compilation unit in DWARF format
#[derive(Debug, Clone)]
pub struct CompilationUnit {
    /// Producer (compiler name and version)
    pub producer: String,
    
    /// Language identifier
    pub language: DwarfLanguage,
    
    /// Compilation directory
    pub comp_dir: PathBuf,
    
    /// Name of the source file
    pub name: String,
    
    /// Low and high program counter addresses
    pub low_pc: u64,
    pub high_pc: u64,
}

/// DWARF language codes
#[derive(Debug, Clone, Copy)]
pub enum DwarfLanguage {
    AetherScript = 0x8001, // Custom language code
}

/// Debug information entry (DIE)
#[derive(Debug, Clone)]
pub struct DebugInfoEntry {
    /// DIE tag (function, variable, etc.)
    pub tag: DieTag,
    
    /// Attributes for this DIE
    pub attributes: Vec<DieAttribute>,
    
    /// Child DIEs
    pub children: Vec<DebugInfoEntry>,
}

/// DWARF DIE tags
#[derive(Debug, Clone, PartialEq)]
pub enum DieTag {
    CompileUnit,
    Subprogram,
    Variable,
    Parameter,
    LexicalBlock,
    BaseType,
    PointerType,
    ArrayType,
}

/// DWARF DIE attributes
#[derive(Debug, Clone)]
pub struct DieAttribute {
    pub name: DieAttributeName,
    pub value: DieAttributeValue,
}

/// DWARF attribute names
#[derive(Debug, Clone, PartialEq)]
pub enum DieAttributeName {
    Name,
    Type,
    LowPc,
    HighPc,
    DeclFile,
    DeclLine,
    Location,
    ByteSize,
    Encoding,
    External,
}

/// DWARF attribute values
#[derive(Debug, Clone)]
pub enum DieAttributeValue {
    String(String),
    Address(u64),
    UnsignedInt(u64),
    Block(Vec<u8>),
    Reference(u32),
}

/// Line number program for source-level debugging
#[derive(Debug, Default)]
pub struct LineProgram {
    /// Line number entries
    entries: Vec<LineEntry>,
    
    /// Source file table
    file_table: Vec<SourceFile>,
}

/// Single line number entry
#[derive(Debug, Clone)]
pub struct LineEntry {
    /// Program counter address
    pub address: u64,
    
    /// Source file index
    pub file: u32,
    
    /// Line number
    pub line: u32,
    
    /// Column number
    pub column: u32,
    
    /// Is this the beginning of a statement?
    pub is_stmt: bool,
    
    /// Is this the beginning of a basic block?
    pub basic_block: bool,
    
    /// Is this the end of a sequence?
    pub end_sequence: bool,
}

/// Source file information
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// File name
    pub name: String,
    
    /// Directory index
    pub dir_index: u32,
    
    /// Last modification time
    pub mtime: u64,
    
    /// File size
    pub size: u64,
}

/// Variable location information
#[derive(Debug, Clone)]
pub enum VariableLocation {
    /// Variable is in a register
    Register(u8),
    
    /// Variable is on the stack at offset
    Stack(i32),
    
    /// Variable is at a memory address
    Memory(u64),
    
    /// Variable location is computed by expression
    Expression(Vec<u8>),
}

impl DwarfGenerator {
    pub fn new(debug_level: u8) -> Self {
        Self {
            debug_level,
            compilation_units: Vec::new(),
            debug_entries: Vec::new(),
            line_program: LineProgram::default(),
            source_files: HashMap::new(),
        }
    }
    
    /// Generate DWARF debug information for a program
    pub fn generate_for_program(&mut self, program: &Program, backend: &mut LLVMBackend) -> Result<(), SemanticError> {
        // Create compilation unit
        let comp_unit = self.create_compilation_unit()?;
        self.compilation_units.push(comp_unit);
        
        // Generate debug info for all functions
        for (name, function) in &program.functions {
            self.generate_function_debug_info(name, function, backend)?;
        }
        
        // Generate line number information
        self.generate_line_number_info(program)?;
        
        // Emit DWARF sections to LLVM module
        self.emit_dwarf_sections(backend)?;
        
        Ok(())
    }
    
    /// Create a compilation unit DIE
    fn create_compilation_unit(&mut self) -> Result<CompilationUnit, SemanticError> {
        Ok(CompilationUnit {
            producer: "AetherScript Compiler 1.0".to_string(),
            language: DwarfLanguage::AetherScript,
            comp_dir: std::env::current_dir().unwrap_or_default(),
            name: "main.aether".to_string(),
            low_pc: 0,
            high_pc: 0,
        })
    }
    
    /// Generate debug information for a function
    fn generate_function_debug_info(&mut self, name: &str, function: &Function, _backend: &LLVMBackend) -> Result<(), SemanticError> {
        let mut function_die = DebugInfoEntry {
            tag: DieTag::Subprogram,
            attributes: vec![
                DieAttribute {
                    name: DieAttributeName::Name,
                    value: DieAttributeValue::String(name.to_string()),
                },
                DieAttribute {
                    name: DieAttributeName::External,
                    value: DieAttributeValue::UnsignedInt(1),
                },
            ],
            children: Vec::new(),
        };
        
        // Add function parameters
        for (param_idx, param) in function.parameters.iter().enumerate() {
            if let Some(local) = function.locals.get(&param.local_id) {
                let param_die = self.create_parameter_die(param_idx, local)?;
                function_die.children.push(param_die);
            }
        }
        
        // Add local variables  
        for (idx, (_local_id, local)) in function.locals.iter().enumerate() {
            let var_die = self.create_variable_die(idx, local)?;
            function_die.children.push(var_die);
        }
        
        // Add lexical blocks for basic blocks
        for (block_id, block) in &function.basic_blocks {
            let block_die = self.create_lexical_block_die(*block_id as usize, block)?;
            function_die.children.push(block_die);
        }
        
        self.debug_entries.push(function_die);
        Ok(())
    }
    
    /// Create a parameter DIE
    fn create_parameter_die(&self, index: usize, param: &crate::mir::Local) -> Result<DebugInfoEntry, SemanticError> {
        Ok(DebugInfoEntry {
            tag: DieTag::Parameter,
            attributes: vec![
                DieAttribute {
                    name: DieAttributeName::Name,
                    value: DieAttributeValue::String(format!("param_{}", index)),
                },
                DieAttribute {
                    name: DieAttributeName::Type,
                    value: DieAttributeValue::Reference(self.get_type_reference(&param.ty)),
                },
                DieAttribute {
                    name: DieAttributeName::Location,
                    value: DieAttributeValue::Block(self.encode_parameter_location(index)),
                },
            ],
            children: Vec::new(),
        })
    }
    
    /// Create a variable DIE
    fn create_variable_die(&self, index: usize, local: &crate::mir::Local) -> Result<DebugInfoEntry, SemanticError> {
        Ok(DebugInfoEntry {
            tag: DieTag::Variable,
            attributes: vec![
                DieAttribute {
                    name: DieAttributeName::Name,
                    value: DieAttributeValue::String(format!("local_{}", index)),
                },
                DieAttribute {
                    name: DieAttributeName::Type,
                    value: DieAttributeValue::Reference(self.get_type_reference(&local.ty)),
                },
                DieAttribute {
                    name: DieAttributeName::Location,
                    value: DieAttributeValue::Block(self.encode_variable_location(index)),
                },
            ],
            children: Vec::new(),
        })
    }
    
    /// Create a lexical block DIE
    fn create_lexical_block_die(&self, block_id: usize, _block: &BasicBlock) -> Result<DebugInfoEntry, SemanticError> {
        Ok(DebugInfoEntry {
            tag: DieTag::LexicalBlock,
            attributes: vec![
                DieAttribute {
                    name: DieAttributeName::LowPc,
                    value: DieAttributeValue::Address(block_id as u64 * 0x1000), // Placeholder
                },
                DieAttribute {
                    name: DieAttributeName::HighPc,
                    value: DieAttributeValue::Address((block_id as u64 + 1) * 0x1000), // Placeholder
                },
            ],
            children: Vec::new(),
        })
    }
    
    /// Get type reference for a type
    fn get_type_reference(&self, ty: &crate::types::Type) -> u32 {
        // Simplified type mapping - in a real implementation this would
        // maintain a proper type table
        match ty {
            crate::types::Type::Primitive(prim) => {
                match prim {
                    crate::ast::PrimitiveType::Integer => 1,
                    crate::ast::PrimitiveType::Integer32 => 1,
                    crate::ast::PrimitiveType::Integer64 => 1,
                    crate::ast::PrimitiveType::Float => 2,
                    crate::ast::PrimitiveType::Float32 => 2,
                    crate::ast::PrimitiveType::Float64 => 2,
                    crate::ast::PrimitiveType::Boolean => 3,
                    crate::ast::PrimitiveType::String => 4,
                    crate::ast::PrimitiveType::Char => 6,
                    crate::ast::PrimitiveType::Void => 5,
                    crate::ast::PrimitiveType::SizeT => 1, // Treat as integer
                    crate::ast::PrimitiveType::UIntPtrT => 1, // Treat as integer
                }
            }
            _ => 0, // Unknown type
        }
    }
    
    /// Encode parameter location in DWARF expression format
    fn encode_parameter_location(&self, index: usize) -> Vec<u8> {
        // Simplified encoding - parameters are typically in registers or on stack
        // DW_OP_fbreg (frame base register) + offset
        vec![0x91, (index as u8 * 8)] // Stack offset for parameter
    }
    
    /// Encode variable location in DWARF expression format
    fn encode_variable_location(&self, index: usize) -> Vec<u8> {
        // Simplified encoding - locals are typically on stack
        // DW_OP_fbreg (frame base register) + negative offset
        vec![0x91, (0x80 | (index as u8 * 8))] // Negative stack offset for local
    }
    
    /// Generate line number information
    fn generate_line_number_info(&mut self, program: &Program) -> Result<(), SemanticError> {
        // Add source files to file table
        self.add_source_file("main.aether".to_string());
        
        // Generate line entries for each function
        for (name, function) in &program.functions {
            self.generate_function_line_info(name, function)?;
        }
        
        Ok(())
    }
    
    /// Add a source file to the file table
    fn add_source_file(&mut self, filename: String) -> u32 {
        if let Some(&index) = self.source_files.get(&filename) {
            return index;
        }
        
        let index = self.line_program.file_table.len() as u32;
        self.line_program.file_table.push(SourceFile {
            name: filename.clone(),
            dir_index: 0,
            mtime: 0,
            size: 0,
        });
        
        self.source_files.insert(filename, index);
        index
    }
    
    /// Generate line number information for a function
    fn generate_function_line_info(&mut self, _name: &str, function: &Function) -> Result<(), SemanticError> {
        let file_index = 0; // Main source file
        
        for (block_id, block) in &function.basic_blocks {
            // Add line entry for block start
            self.line_program.entries.push(LineEntry {
                address: *block_id as u64 * 0x1000, // Placeholder address
                file: file_index,
                line: (*block_id + 1) as u32, // Placeholder line number
                column: 1,
                is_stmt: true,
                basic_block: true,
                end_sequence: false,
            });
            
            // Add line entries for each statement
            for (stmt_idx, _statement) in block.statements.iter().enumerate() {
                self.line_program.entries.push(LineEntry {
                    address: (*block_id as u64 * 0x1000) + (stmt_idx as u64 * 4),
                    file: file_index,
                    line: (*block_id + 1) as u32,
                    column: (stmt_idx + 1) as u32,
                    is_stmt: true,
                    basic_block: false,
                    end_sequence: false,
                });
            }
        }
        
        Ok(())
    }
    
    /// Emit DWARF sections to LLVM module
    fn emit_dwarf_sections(&self, _backend: &mut LLVMBackend) -> Result<(), SemanticError> {
        // This would emit the actual DWARF sections to the LLVM module
        // For now, we'll just log what we would do
        eprintln!("Would emit DWARF debug information:");
        eprintln!("  {} compilation units", self.compilation_units.len());
        eprintln!("  {} debug info entries", self.debug_entries.len());
        eprintln!("  {} line program entries", self.line_program.entries.len());
        eprintln!("  {} source files", self.line_program.file_table.len());
        
        // In a real implementation, this would:
        // 1. Create .debug_info section with DIE tree
        // 2. Create .debug_line section with line number program
        // 3. Create .debug_abbrev section with abbreviation declarations
        // 4. Create .debug_str section with string table
        // 5. Create .debug_loc section with location lists
        // 6. Create .debug_ranges section with address ranges
        
        Ok(())
    }
    
    /// Get debug level
    pub fn debug_level(&self) -> u8 {
        self.debug_level
    }
    
    /// Set debug level
    pub fn set_debug_level(&mut self, level: u8) {
        self.debug_level = level;
    }
    
    /// Get compilation units
    pub fn compilation_units(&self) -> &[CompilationUnit] {
        &self.compilation_units
    }
    
    /// Get debug entries
    pub fn debug_entries(&self) -> &[DebugInfoEntry] {
        &self.debug_entries
    }
    
    /// Get line program
    pub fn line_program(&self) -> &LineProgram {
        &self.line_program
    }
}

impl LineProgram {
    /// Get line entries
    pub fn entries(&self) -> &[LineEntry] {
        &self.entries
    }
    
    /// Get file table
    pub fn file_table(&self) -> &[SourceFile] {
        &self.file_table
    }
    
    /// Add line entry
    pub fn add_entry(&mut self, entry: LineEntry) {
        self.entries.push(entry);
    }
    
    /// Find line entry for address
    pub fn find_line_for_address(&self, address: u64) -> Option<&LineEntry> {
        self.entries.iter()
            .filter(|entry| entry.address <= address)
            .max_by_key(|entry| entry.address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{Builder, Program};
    use crate::types::Type;
    use crate::ast::PrimitiveType;
    use std::collections::HashMap;
    
    #[test]
    fn test_dwarf_generator_creation() {
        let generator = DwarfGenerator::new(2);
        assert_eq!(generator.debug_level(), 2);
        assert!(generator.compilation_units().is_empty());
        assert!(generator.debug_entries().is_empty());
    }
    
    #[test]
    fn test_compilation_unit_creation() {
        let mut generator = DwarfGenerator::new(2);
        let comp_unit = generator.create_compilation_unit().unwrap();
        
        assert_eq!(comp_unit.producer, "AetherScript Compiler 1.0");
        assert_eq!(comp_unit.name, "main.aether");
    }
    
    #[test]
    fn test_line_program() {
        let mut line_program = LineProgram::default();
        
        let entry = LineEntry {
            address: 0x1000,
            file: 0,
            line: 10,
            column: 5,
            is_stmt: true,
            basic_block: true,
            end_sequence: false,
        };
        
        line_program.add_entry(entry);
        assert_eq!(line_program.entries().len(), 1);
        
        let found = line_program.find_line_for_address(0x1000);
        assert!(found.is_some());
        assert_eq!(found.unwrap().line, 10);
    }
    
    #[test]
    fn test_die_creation() {
        let generator = DwarfGenerator::new(2);
        
        // Create a mock local for testing
        let local = crate::mir::Local {
            ty: Type::primitive(PrimitiveType::Integer),
            is_mutable: true,
            source_info: None,
        };
        let param_die = generator.create_parameter_die(0, &local).unwrap();
        
        assert_eq!(param_die.tag, DieTag::Parameter);
        assert!(!param_die.attributes.is_empty());
    }
    
    #[test]
    fn test_type_reference() {
        let generator = DwarfGenerator::new(2);
        
        let int_type = Type::primitive(PrimitiveType::Integer);
        let float_type = Type::primitive(PrimitiveType::Float);
        
        assert_eq!(generator.get_type_reference(&int_type), 1);
        assert_eq!(generator.get_type_reference(&float_type), 2);
    }
    
    #[test]
    fn test_source_file_management() {
        let mut generator = DwarfGenerator::new(2);
        
        let index1 = generator.add_source_file("test1.aether".to_string());
        let index2 = generator.add_source_file("test2.aether".to_string());
        let index3 = generator.add_source_file("test1.aether".to_string()); // Duplicate
        
        assert_eq!(index1, 0);
        assert_eq!(index2, 1);
        assert_eq!(index3, 0); // Should return existing index
        assert_eq!(generator.line_program.file_table.len(), 2);
    }
}