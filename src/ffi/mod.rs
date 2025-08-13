//! Foreign Function Interface (FFI) framework for AetherScript
//! 
//! Provides FFI support for C/C++, Rust, and Go interoperability

use crate::ast::*;
use crate::types::{Type, TypeChecker};
use crate::error::SemanticError;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

/// FFI type mapping between AetherScript and external languages
#[derive(Debug, Clone)]
pub struct FFITypeMapper {
    /// Mapping from AetherScript types to C types
    c_type_map: HashMap<String, String>,
    /// Mapping from AetherScript types to Rust types
    rust_type_map: HashMap<String, String>,
    /// Mapping from AetherScript types to Go types
    go_type_map: HashMap<String, String>,
}

impl FFITypeMapper {
    pub fn new() -> Self {
        let mut mapper = Self {
            c_type_map: HashMap::new(),
            rust_type_map: HashMap::new(),
            go_type_map: HashMap::new(),
        };
        
        // Initialize primitive type mappings
        mapper.init_primitive_mappings();
        mapper
    }
    
    fn init_primitive_mappings(&mut self) {
        // C mappings
        self.c_type_map.insert("Integer".to_string(), "int64_t".to_string());
        self.c_type_map.insert("Integer32".to_string(), "int32_t".to_string());
        self.c_type_map.insert("Integer64".to_string(), "int64_t".to_string());
        self.c_type_map.insert("Float".to_string(), "double".to_string());
        self.c_type_map.insert("Float32".to_string(), "float".to_string());
        self.c_type_map.insert("Float64".to_string(), "double".to_string());
        self.c_type_map.insert("Boolean".to_string(), "bool".to_string());
        self.c_type_map.insert("String".to_string(), "const char*".to_string());
        self.c_type_map.insert("Void".to_string(), "void".to_string());
        
        // Rust mappings
        self.rust_type_map.insert("Integer".to_string(), "i64".to_string());
        self.rust_type_map.insert("Integer32".to_string(), "i32".to_string());
        self.rust_type_map.insert("Integer64".to_string(), "i64".to_string());
        self.rust_type_map.insert("Float".to_string(), "f64".to_string());
        self.rust_type_map.insert("Float32".to_string(), "f32".to_string());
        self.rust_type_map.insert("Float64".to_string(), "f64".to_string());
        self.rust_type_map.insert("Boolean".to_string(), "bool".to_string());
        self.rust_type_map.insert("String".to_string(), "*const c_char".to_string());
        self.rust_type_map.insert("Void".to_string(), "()".to_string());
        
        // Go mappings
        self.go_type_map.insert("Integer".to_string(), "int64".to_string());
        self.go_type_map.insert("Integer32".to_string(), "int32".to_string());
        self.go_type_map.insert("Integer64".to_string(), "int64".to_string());
        self.go_type_map.insert("Float".to_string(), "float64".to_string());
        self.go_type_map.insert("Float32".to_string(), "float32".to_string());
        self.go_type_map.insert("Float64".to_string(), "float64".to_string());
        self.go_type_map.insert("Boolean".to_string(), "bool".to_string());
        self.go_type_map.insert("String".to_string(), "*C.char".to_string());
        self.go_type_map.insert("Void".to_string(), "".to_string());
    }
    
    /// Map AetherScript type to C type
    pub fn map_to_c_type(&self, aether_type: &Type) -> Result<String, String> {
        match aether_type {
            Type::Primitive(prim) => {
                let type_name = format!("{:?}", prim);
                self.c_type_map.get(&type_name)
                    .cloned()
                    .ok_or_else(|| format!("No C mapping for type: {}", type_name))
            }
            Type::Pointer { target_type, is_mutable } => {
                let base_type = self.map_to_c_type(target_type)?;
                if *is_mutable {
                    Ok(format!("{}*", base_type))
                } else {
                    Ok(format!("const {}*", base_type))
                }
            }
            Type::Array { element_type, .. } => {
                let elem_type = self.map_to_c_type(element_type)?;
                Ok(format!("{}*", elem_type))
            }
            _ => Err(format!("Unsupported type for C FFI: {:?}", aether_type)),
        }
    }
    
    /// Map AetherScript type to Rust type
    pub fn map_to_rust_type(&self, aether_type: &Type) -> Result<String, String> {
        match aether_type {
            Type::Primitive(prim) => {
                let type_name = format!("{:?}", prim);
                self.rust_type_map.get(&type_name)
                    .cloned()
                    .ok_or_else(|| format!("No Rust mapping for type: {}", type_name))
            }
            Type::Pointer { target_type, is_mutable } => {
                let base_type = self.map_to_rust_type(target_type)?;
                if *is_mutable {
                    Ok(format!("*mut {}", base_type))
                } else {
                    Ok(format!("*const {}", base_type))
                }
            }
            Type::Array { element_type, .. } => {
                let elem_type = self.map_to_rust_type(element_type)?;
                Ok(format!("*const {}", elem_type))
            }
            _ => Err(format!("Unsupported type for Rust FFI: {:?}", aether_type)),
        }
    }
    
    /// Map AetherScript type to Go type
    pub fn map_to_go_type(&self, aether_type: &Type) -> Result<String, String> {
        match aether_type {
            Type::Primitive(prim) => {
                let type_name = format!("{:?}", prim);
                self.go_type_map.get(&type_name)
                    .cloned()
                    .ok_or_else(|| format!("No Go mapping for type: {}", type_name))
            }
            Type::Pointer { target_type, is_mutable } => {
                let base_type = self.map_to_go_type(target_type)?;
                if *is_mutable {
                    Ok(format!("*{}", base_type))
                } else {
                    Ok(format!("*{}", base_type)) // Go doesn't have const pointers
                }
            }
            Type::Array { element_type, .. } => {
                let elem_type = self.map_to_go_type(element_type)?;
                Ok(format!("*{}", elem_type))
            }
            _ => Err(format!("Unsupported type for Go FFI: {:?}", aether_type)),
        }
    }
}

/// FFI analyzer for external function declarations
#[derive(Debug, Clone)]
pub struct FFIAnalyzer {
    type_mapper: FFITypeMapper,
    type_checker: Rc<RefCell<TypeChecker>>,
    external_functions: HashMap<String, ExternalFunction>,
}

impl FFIAnalyzer {
    pub fn new(type_checker: Rc<RefCell<TypeChecker>>) -> Self {
        Self {
            type_mapper: FFITypeMapper::new(),
            type_checker,
            external_functions: HashMap::new(),
        }
    }
    
    /// Analyze an external function declaration
    pub fn analyze_external_function(&mut self, ext_func: &ExternalFunction) -> Result<(), SemanticError> {
        // Validate library name
        if ext_func.library.is_empty() {
            return Err(SemanticError::InvalidFFI {
                message: "External function must specify a library".to_string(),
                location: ext_func.source_location.clone(),
            });
        }
        
        // Validate parameters
        for param in &ext_func.parameters {
            let param_type = self.type_checker.borrow().ast_type_to_type(&param.param_type)?;
            
            // Check if type is FFI-compatible
            if !self.is_ffi_compatible(&param_type) {
                return Err(SemanticError::InvalidFFI {
                    message: format!("Parameter '{}' has non-FFI-compatible type: {}", 
                                   param.name.name, param_type),
                    location: param.source_location.clone(),
                });
            }
        }
        
        // Validate return type
        let return_type = self.type_checker.borrow().ast_type_to_type(&ext_func.return_type)?;
        if !self.is_ffi_compatible(&return_type) {
            return Err(SemanticError::InvalidFFI {
                message: format!("Return type is not FFI-compatible: {}", return_type),
                location: ext_func.source_location.clone(),
            });
        }
        
        // Validate ownership info for pointer types (skip for C calling convention)
        if self.has_pointer_types(ext_func) && ext_func.ownership_info.is_none() 
            && ext_func.calling_convention != CallingConvention::C {
            return Err(SemanticError::InvalidFFI {
                message: "External function with pointer parameters must specify ownership info (not required for C calling convention)".to_string(),
                location: ext_func.source_location.clone(),
            });
        }
        
        // Store the external function
        self.external_functions.insert(
            ext_func.name.name.clone(),
            ext_func.clone()
        );
        
        Ok(())
    }
    
    /// Check if a type is FFI-compatible
    fn is_ffi_compatible(&self, aether_type: &Type) -> bool {
        match aether_type {
            Type::Primitive(_) => true,
            Type::Pointer { target_type, .. } => self.is_ffi_compatible(target_type),
            Type::Array { element_type, .. } => self.is_ffi_compatible(element_type),
            Type::Function { .. } => false, // Function pointers need special handling
            Type::Named { .. } => false,    // Named types need special handling
            _ => false,
        }
    }
    
    /// Check if external function has pointer types
    fn has_pointer_types(&self, ext_func: &ExternalFunction) -> bool {
        // Check parameters
        for param in &ext_func.parameters {
            if let Ok(param_type) = self.type_checker.borrow().ast_type_to_type(&param.param_type) {
                if matches!(param_type, Type::Pointer { .. }) {
                    return true;
                }
            }
        }
        
        // Check return type
        if let Ok(return_type) = self.type_checker.borrow().ast_type_to_type(&ext_func.return_type) {
            if matches!(return_type, Type::Pointer { .. }) {
                return true;
            }
        }
        
        false
    }
    
    /// Generate C header for external functions
    pub fn generate_c_header(&self, module_name: &str) -> String {
        let mut header = String::new();
        
        // Header guard
        let guard_name = format!("{}_H", module_name.to_uppercase());
        header.push_str(&format!("#ifndef {}\n", guard_name));
        header.push_str(&format!("#define {}\n\n", guard_name));
        
        // Standard includes
        header.push_str("#include <stdint.h>\n");
        header.push_str("#include <stdbool.h>\n\n");
        
        // Function declarations
        for (name, ext_func) in &self.external_functions {
            if let Ok(decl) = self.generate_c_function_declaration(ext_func) {
                header.push_str(&format!("// {}\n", name));
                header.push_str(&format!("{}\n\n", decl));
            }
        }
        
        // Close header guard
        header.push_str(&format!("#endif // {}\n", guard_name));
        
        header
    }
    
    /// Generate C function declaration
    fn generate_c_function_declaration(&self, ext_func: &ExternalFunction) -> Result<String, String> {
        let return_type = self.type_checker.borrow().ast_type_to_type(&ext_func.return_type)
            .map_err(|e| e.to_string())?;
        let c_return_type = self.type_mapper.map_to_c_type(&return_type)?;
        
        let mut param_list = Vec::new();
        for param in &ext_func.parameters {
            let param_type = self.type_checker.borrow().ast_type_to_type(&param.param_type)
                .map_err(|e| e.to_string())?;
            let c_param_type = self.type_mapper.map_to_c_type(&param_type)?;
            param_list.push(format!("{} {}", c_param_type, param.name.name));
        }
        
        let params = if param_list.is_empty() {
            "void".to_string()
        } else {
            param_list.join(", ")
        };
        
        // Add calling convention if not C
        let convention = match ext_func.calling_convention {
            CallingConvention::StdCall => "__stdcall ",
            CallingConvention::FastCall => "__fastcall ",
            _ => "",
        };
        
        Ok(format!("{} {}{} {}({})", 
                   c_return_type,
                   convention,
                   ext_func.symbol.as_ref().unwrap_or(&ext_func.name.name),
                   ext_func.name.name,
                   params))
    }
    
    /// Get all analyzed external functions
    pub fn get_external_functions(&self) -> &HashMap<String, ExternalFunction> {
        &self.external_functions
    }
}

/// FFI code generator for creating bindings
pub struct FFIGenerator {
    analyzer: FFIAnalyzer,
}

impl FFIGenerator {
    pub fn new(analyzer: FFIAnalyzer) -> Self {
        Self { analyzer }
    }
    
    /// Generate Rust FFI bindings
    pub fn generate_rust_bindings(&self, module_name: &str) -> String {
        let mut bindings = String::new();
        
        // Use statements
        bindings.push_str("use std::os::raw::{c_char, c_void};\n");
        bindings.push_str("use std::ffi::CString;\n\n");
        
        // Extern block
        bindings.push_str("#[link(name = \"");
        bindings.push_str(module_name);
        bindings.push_str("\")]\n");
        bindings.push_str("extern \"C\" {\n");
        
        // Function declarations
        for ext_func in self.analyzer.get_external_functions().values() {
            if let Ok(decl) = self.generate_rust_function_declaration(ext_func) {
                bindings.push_str("    ");
                bindings.push_str(&decl);
                bindings.push('\n');
            }
        }
        
        bindings.push_str("}\n");
        
        bindings
    }
    
    /// Generate Rust function declaration
    fn generate_rust_function_declaration(&self, ext_func: &ExternalFunction) -> Result<String, String> {
        let return_type = self.analyzer.type_checker.borrow().ast_type_to_type(&ext_func.return_type)
            .map_err(|e| e.to_string())?;
        let rust_return_type = self.analyzer.type_mapper.map_to_rust_type(&return_type)?;
        
        let mut param_list = Vec::new();
        for param in &ext_func.parameters {
            let param_type = self.analyzer.type_checker.borrow().ast_type_to_type(&param.param_type)
                .map_err(|e| e.to_string())?;
            let rust_param_type = self.analyzer.type_mapper.map_to_rust_type(&param_type)?;
            param_list.push(format!("{}: {}", param.name.name, rust_param_type));
        }
        
        let params = param_list.join(", ");
        let return_part = if rust_return_type == "()" {
            String::new()
        } else {
            format!(" -> {}", rust_return_type)
        };
        
        Ok(format!("pub fn {}({}){};", 
                   ext_func.symbol.as_ref().unwrap_or(&ext_func.name.name),
                   params,
                   return_part))
    }
    
    /// Generate Go FFI bindings
    pub fn generate_go_bindings(&self, module_name: &str, package_name: &str) -> String {
        let mut bindings = String::new();
        
        // Package declaration
        bindings.push_str(&format!("package {}\n\n", package_name));
        
        // CGO directives
        bindings.push_str("/*\n");
        bindings.push_str(&format!("#cgo LDFLAGS: -l{}\n", module_name));
        bindings.push_str("#include <stdlib.h>\n");
        bindings.push_str(&format!("#include \"{}.h\"\n", module_name));
        bindings.push_str("*/\n");
        bindings.push_str("import \"C\"\n");
        bindings.push_str("import \"unsafe\"\n\n");
        
        // Function wrappers
        for ext_func in self.analyzer.get_external_functions().values() {
            if let Ok(wrapper) = self.generate_go_function_wrapper(ext_func) {
                bindings.push_str(&wrapper);
                bindings.push_str("\n\n");
            }
        }
        
        bindings
    }
    
    /// Generate Go function wrapper
    fn generate_go_function_wrapper(&self, ext_func: &ExternalFunction) -> Result<String, String> {
        let mut wrapper = String::new();
        
        // Function signature
        wrapper.push_str(&format!("func {}(", capitalize(&ext_func.name.name)));
        
        // Parameters
        let mut param_list = Vec::new();
        let mut conversions = Vec::new();
        
        for param in &ext_func.parameters {
            let param_type = self.analyzer.type_checker.borrow().ast_type_to_type(&param.param_type)
                .map_err(|e| e.to_string())?;
            let go_type = self.analyzer.type_mapper.map_to_go_type(&param_type)?;
            
            // Clean up Go type for function signature
            let clean_go_type = go_type.replace("*C.", "");
            param_list.push(format!("{} {}", param.name.name, clean_go_type));
            
            // Add conversion if needed
            if go_type.contains("*C.char") {
                conversions.push(format!("c{} := C.CString({})", param.name.name, param.name.name));
                conversions.push(format!("defer C.free(unsafe.Pointer(c{}))", param.name.name));
            }
        }
        
        wrapper.push_str(&param_list.join(", "));
        wrapper.push(')');
        
        // Return type
        let return_type = self.analyzer.type_checker.borrow().ast_type_to_type(&ext_func.return_type)
            .map_err(|e| e.to_string())?;
        let go_return_type = self.analyzer.type_mapper.map_to_go_type(&return_type)?;
        
        if !go_return_type.is_empty() {
            wrapper.push_str(&format!(" {}", go_return_type.replace("*C.", "")));
        }
        
        wrapper.push_str(" {\n");
        
        // Add conversions
        for conversion in &conversions {
            wrapper.push_str(&format!("    {}\n", conversion));
        }
        
        // Call C function
        wrapper.push_str("    ");
        if !go_return_type.is_empty() {
            wrapper.push_str("return ");
        }
        
        wrapper.push_str(&format!("C.{}(", ext_func.symbol.as_ref().unwrap_or(&ext_func.name.name)));
        
        // C parameters
        let mut c_params = Vec::new();
        for param in &ext_func.parameters {
            let param_type = self.analyzer.type_checker.borrow().ast_type_to_type(&param.param_type)
                .map_err(|e| e.to_string())?;
            if matches!(param_type, Type::Primitive(PrimitiveType::String)) {
                c_params.push(format!("c{}", param.name.name));
            } else {
                c_params.push(format!("C.{}({})", 
                    self.analyzer.type_mapper.map_to_go_type(&param_type)?.replace("*C.", ""),
                    param.name.name));
            }
        }
        
        wrapper.push_str(&c_params.join(", "));
        wrapper.push_str(")\n}");
        
        Ok(wrapper)
    }
}

/// Helper function to capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Callback registry for managing callbacks from external languages
#[derive(Debug, Clone)]
pub struct CallbackRegistry {
    /// Registered AetherScript functions that can be called from external code
    callbacks: HashMap<String, CallbackFunction>,
    /// Next callback ID
    next_id: u32,
}

/// Information about a function that can be called back from external code
#[derive(Debug, Clone)]
pub struct CallbackFunction {
    /// Unique callback ID
    pub id: u32,
    /// Function name in AetherScript
    pub function_name: String,
    /// Function signature
    pub signature: FunctionSignature,
    /// Calling convention for callback
    pub calling_convention: CallingConvention,
    /// Whether the callback is thread-safe
    pub thread_safe: bool,
}

/// Function signature for callbacks
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub parameters: Vec<Type>,
    pub return_type: Type,
}

impl CallbackRegistry {
    pub fn new() -> Self {
        Self {
            callbacks: HashMap::new(),
            next_id: 1,
        }
    }
    
    /// Register a function for callback from external code
    pub fn register_callback(
        &mut self,
        function_name: String,
        signature: FunctionSignature,
        calling_convention: CallingConvention,
        thread_safe: bool,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let callback = CallbackFunction {
            id,
            function_name: function_name.clone(),
            signature,
            calling_convention,
            thread_safe,
        };
        
        self.callbacks.insert(function_name, callback);
        id
    }
    
    /// Get callback by ID
    pub fn get_callback(&self, id: u32) -> Option<&CallbackFunction> {
        self.callbacks.values().find(|cb| cb.id == id)
    }
    
    /// Get callback by function name
    pub fn get_callback_by_name(&self, name: &str) -> Option<&CallbackFunction> {
        self.callbacks.get(name)
    }
    
    /// Generate C callback declarations
    pub fn generate_c_callback_declarations(&self, type_mapper: &FFITypeMapper) -> String {
        let mut declarations = String::new();
        
        declarations.push_str("// Callback function pointers\n");
        for callback in self.callbacks.values() {
            if let Ok(decl) = self.generate_c_callback_declaration(callback, type_mapper) {
                declarations.push_str(&format!("{}\n", decl));
            }
        }
        
        declarations.push_str("\n// Callback registration functions\n");
        for callback in self.callbacks.values() {
            declarations.push_str(&format!("void register_{}({}*);\n", 
                callback.function_name, callback.function_name));
        }
        
        declarations
    }
    
    /// Generate C callback declaration
    fn generate_c_callback_declaration(
        &self,
        callback: &CallbackFunction,
        type_mapper: &FFITypeMapper,
    ) -> Result<String, String> {
        let return_type = type_mapper.map_to_c_type(&callback.signature.return_type)?;
        
        let mut param_types = Vec::new();
        for (i, param_type) in callback.signature.parameters.iter().enumerate() {
            let c_type = type_mapper.map_to_c_type(param_type)?;
            param_types.push(format!("{} param{}", c_type, i));
        }
        
        let params = if param_types.is_empty() {
            "void".to_string()
        } else {
            param_types.join(", ")
        };
        
        Ok(format!("typedef {} (*{})({})", 
                   return_type, callback.function_name, params))
    }
    
    /// Generate Rust callback declarations
    pub fn generate_rust_callback_declarations(&self, type_mapper: &FFITypeMapper) -> String {
        let mut declarations = String::new();
        
        declarations.push_str("// Callback function types\n");
        for callback in self.callbacks.values() {
            if let Ok(decl) = self.generate_rust_callback_declaration(callback, type_mapper) {
                declarations.push_str(&format!("{}\n", decl));
            }
        }
        
        declarations
    }
    
    /// Generate Rust callback declaration
    fn generate_rust_callback_declaration(
        &self,
        callback: &CallbackFunction,
        type_mapper: &FFITypeMapper,
    ) -> Result<String, String> {
        let return_type = type_mapper.map_to_rust_type(&callback.signature.return_type)?;
        
        let mut param_types = Vec::new();
        for param_type in &callback.signature.parameters {
            let rust_type = type_mapper.map_to_rust_type(param_type)?;
            param_types.push(rust_type);
        }
        
        let params = param_types.join(", ");
        
        Ok(format!("pub type {}_callback = extern \"C\" fn({}) -> {}", 
                   callback.function_name, params, return_type))
    }
}

/// Enhanced FFI type support for structs and enums
#[derive(Debug, Clone)]
pub struct FFIStructHandler {
    /// Struct definitions
    structs: HashMap<String, StructType>,
    /// Enum definitions
    enums: HashMap<String, EnumType>,
    type_mapper: FFITypeMapper,
}

/// Struct type information for FFI
#[derive(Debug, Clone)]
pub struct StructType {
    pub name: String,
    pub fields: Vec<StructField>,
    pub packed: bool,
    pub alignment: Option<usize>,
}

/// Struct field information
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub offset: Option<usize>,
}

/// Enum type information for FFI
#[derive(Debug, Clone)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub underlying_type: Type,
}

/// Enum variant information
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i64>,
}

impl FFIStructHandler {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            type_mapper: FFITypeMapper::new(),
        }
    }
    
    /// Register a struct type for FFI
    pub fn register_struct(&mut self, struct_type: StructType) {
        self.structs.insert(struct_type.name.clone(), struct_type);
    }
    
    /// Register an enum type for FFI
    pub fn register_enum(&mut self, enum_type: EnumType) {
        self.enums.insert(enum_type.name.clone(), enum_type);
    }
    
    /// Generate C struct declarations
    pub fn generate_c_struct_declarations(&self) -> String {
        let mut declarations = String::new();
        
        // Generate struct declarations
        for struct_type in self.structs.values() {
            if let Ok(decl) = self.generate_c_struct_declaration(struct_type) {
                declarations.push_str(&format!("{}\n\n", decl));
            }
        }
        
        // Generate enum declarations
        for enum_type in self.enums.values() {
            if let Ok(decl) = self.generate_c_enum_declaration(enum_type) {
                declarations.push_str(&format!("{}\n\n", decl));
            }
        }
        
        declarations
    }
    
    /// Generate C struct declaration
    fn generate_c_struct_declaration(&self, struct_type: &StructType) -> Result<String, String> {
        let mut declaration = String::new();
        
        if struct_type.packed {
            declaration.push_str("#pragma pack(push, 1)\n");
        }
        
        declaration.push_str(&format!("typedef struct {} {{\n", struct_type.name));
        
        for field in &struct_type.fields {
            let c_type = self.type_mapper.map_to_c_type(&field.field_type)?;
            declaration.push_str(&format!("    {} {};\n", c_type, field.name));
        }
        
        declaration.push_str(&format!("}} {};\n", struct_type.name));
        
        if struct_type.packed {
            declaration.push_str("#pragma pack(pop)\n");
        }
        
        Ok(declaration)
    }
    
    /// Generate C enum declaration
    fn generate_c_enum_declaration(&self, enum_type: &EnumType) -> Result<String, String> {
        let mut declaration = String::new();
        
        let underlying_type = self.type_mapper.map_to_c_type(&enum_type.underlying_type)?;
        declaration.push_str(&format!("typedef enum : {} {{\n", underlying_type));
        
        for (i, variant) in enum_type.variants.iter().enumerate() {
            if let Some(value) = variant.value {
                declaration.push_str(&format!("    {}__{} = {},\n", 
                    enum_type.name.to_uppercase(), variant.name.to_uppercase(), value));
            } else {
                declaration.push_str(&format!("    {}__{} = {},\n", 
                    enum_type.name.to_uppercase(), variant.name.to_uppercase(), i));
            }
        }
        
        declaration.push_str(&format!("}} {};\n", enum_type.name));
        
        Ok(declaration)
    }
    
    /// Generate Rust struct declarations
    pub fn generate_rust_struct_declarations(&self) -> String {
        let mut declarations = String::new();
        
        // Generate struct declarations
        for struct_type in self.structs.values() {
            if let Ok(decl) = self.generate_rust_struct_declaration(struct_type) {
                declarations.push_str(&format!("{}\n\n", decl));
            }
        }
        
        // Generate enum declarations
        for enum_type in self.enums.values() {
            if let Ok(decl) = self.generate_rust_enum_declaration(enum_type) {
                declarations.push_str(&format!("{}\n\n", decl));
            }
        }
        
        declarations
    }
    
    /// Generate Rust struct declaration
    fn generate_rust_struct_declaration(&self, struct_type: &StructType) -> Result<String, String> {
        let mut declaration = String::new();
        
        let repr = if struct_type.packed {
            "#[repr(packed)]\n"
        } else {
            "#[repr(C)]\n"
        };
        
        declaration.push_str(repr);
        declaration.push_str("#[derive(Debug, Clone, Copy)]\n");
        declaration.push_str(&format!("pub struct {} {{\n", struct_type.name));
        
        for field in &struct_type.fields {
            let rust_type = self.type_mapper.map_to_rust_type(&field.field_type)?;
            declaration.push_str(&format!("    pub {}: {},\n", field.name, rust_type));
        }
        
        declaration.push_str("}\n");
        
        Ok(declaration)
    }
    
    /// Generate Rust enum declaration
    fn generate_rust_enum_declaration(&self, enum_type: &EnumType) -> Result<String, String> {
        let mut declaration = String::new();
        
        let underlying_type = self.type_mapper.map_to_rust_type(&enum_type.underlying_type)?;
        declaration.push_str(&format!("#[repr({})]\n", underlying_type));
        declaration.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
        declaration.push_str(&format!("pub enum {} {{\n", enum_type.name));
        
        for (i, variant) in enum_type.variants.iter().enumerate() {
            if let Some(value) = variant.value {
                declaration.push_str(&format!("    {} = {},\n", variant.name, value));
            } else {
                declaration.push_str(&format!("    {} = {},\n", variant.name, i));
            }
        }
        
        declaration.push_str("}\n");
        
        Ok(declaration)
    }
}

/// Automatic binding generation tool
#[derive(Debug)]
pub struct BindingGenerator {
    analyzer: FFIAnalyzer,
    callback_registry: CallbackRegistry,
    struct_handler: FFIStructHandler,
}

impl BindingGenerator {
    pub fn new(type_checker: Rc<RefCell<TypeChecker>>) -> Self {
        Self {
            analyzer: FFIAnalyzer::new(type_checker),
            callback_registry: CallbackRegistry::new(),
            struct_handler: FFIStructHandler::new(),
        }
    }
    
    /// Generate complete bindings for a target language
    pub fn generate_complete_bindings(
        &self,
        module_name: &str,
        target_language: TargetLanguage,
    ) -> Result<String, String> {
        match target_language {
            TargetLanguage::C => self.generate_complete_c_bindings(module_name),
            TargetLanguage::Rust => self.generate_complete_rust_bindings(module_name),
            TargetLanguage::Go => self.generate_complete_go_bindings(module_name),
        }
    }
    
    /// Generate complete C bindings
    fn generate_complete_c_bindings(&self, module_name: &str) -> Result<String, String> {
        let mut bindings = String::new();
        
        // Header guard
        let guard_name = format!("{}_BINDINGS_H", module_name.to_uppercase());
        bindings.push_str(&format!("#ifndef {}\n", guard_name));
        bindings.push_str(&format!("#define {}\n\n", guard_name));
        
        // Standard includes
        bindings.push_str("#include <stdint.h>\n");
        bindings.push_str("#include <stdbool.h>\n");
        bindings.push_str("#include <stddef.h>\n\n");
        
        // Extern C block
        bindings.push_str("#ifdef __cplusplus\n");
        bindings.push_str("extern \"C\" {\n");
        bindings.push_str("#endif\n\n");
        
        // Struct and enum declarations
        bindings.push_str("// Type definitions\n");
        bindings.push_str(&self.struct_handler.generate_c_struct_declarations());
        
        // External function declarations
        bindings.push_str("// External function declarations\n");
        bindings.push_str(&self.analyzer.generate_c_header(module_name));
        
        // Callback declarations
        bindings.push_str("// Callback declarations\n");
        bindings.push_str(&self.callback_registry.generate_c_callback_declarations(&self.analyzer.type_mapper));
        
        // Close extern C block
        bindings.push_str("\n#ifdef __cplusplus\n");
        bindings.push_str("}\n");
        bindings.push_str("#endif\n\n");
        
        // Close header guard
        bindings.push_str(&format!("#endif // {}\n", guard_name));
        
        Ok(bindings)
    }
    
    /// Generate complete Rust bindings
    fn generate_complete_rust_bindings(&self, module_name: &str) -> Result<String, String> {
        let mut bindings = String::new();
        
        // Standard imports
        bindings.push_str("use std::os::raw::{c_char, c_void};\n");
        bindings.push_str("use std::ffi::{CString, CStr};\n\n");
        
        // Type definitions
        bindings.push_str("// Type definitions\n");
        bindings.push_str(&self.struct_handler.generate_rust_struct_declarations());
        
        // Callback types
        bindings.push_str("// Callback types\n");
        bindings.push_str(&self.callback_registry.generate_rust_callback_declarations(&self.analyzer.type_mapper));
        
        // External functions
        bindings.push_str("// External functions\n");
        let generator = FFIGenerator::new(self.analyzer.clone());
        bindings.push_str(&generator.generate_rust_bindings(module_name));
        
        Ok(bindings)
    }
    
    /// Generate complete Go bindings
    fn generate_complete_go_bindings(&self, module_name: &str) -> Result<String, String> {
        let generator = FFIGenerator::new(self.analyzer.clone());
        Ok(generator.generate_go_bindings(module_name, &format!("{}_bindings", module_name)))
    }
    
    /// Add external function for binding generation
    pub fn add_external_function(&mut self, ext_func: &ExternalFunction) -> Result<(), SemanticError> {
        self.analyzer.analyze_external_function(ext_func)
    }
    
    /// Add callback function
    pub fn add_callback(
        &mut self,
        function_name: String,
        signature: FunctionSignature,
        calling_convention: CallingConvention,
        thread_safe: bool,
    ) -> u32 {
        self.callback_registry.register_callback(function_name, signature, calling_convention, thread_safe)
    }
    
    /// Add struct type
    pub fn add_struct(&mut self, struct_type: StructType) {
        self.struct_handler.register_struct(struct_type);
    }
    
    /// Add enum type
    pub fn add_enum(&mut self, enum_type: EnumType) {
        self.struct_handler.register_enum(enum_type);
    }
}

/// Target language for binding generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetLanguage {
    C,
    Rust,
    Go,
}

impl fmt::Display for TargetLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetLanguage::C => write!(f, "C"),
            TargetLanguage::Rust => write!(f, "Rust"),
            TargetLanguage::Go => write!(f, "Go"),
        }
    }
}

impl Default for CallbackRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FFIStructHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FFITypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SourceLocation;
    
    fn create_test_external_function() -> ExternalFunction {
        ExternalFunction {
            name: Identifier::new("test_add".to_string(), SourceLocation::unknown()),
            library: "testlib".to_string(),
            symbol: Some("test_add_impl".to_string()),
            parameters: vec![
                Parameter {
                    name: Identifier::new("a".to_string(), SourceLocation::unknown()),
                    param_type: Box::new(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Integer,
                        source_location: SourceLocation::unknown(),
                    }),
                    intent: None,
                    constraint: None,
                    passing_mode: PassingMode::ByValue,
                    source_location: SourceLocation::unknown(),
                },
                Parameter {
                    name: Identifier::new("b".to_string(), SourceLocation::unknown()),
                    param_type: Box::new(TypeSpecifier::Primitive {
                        type_name: PrimitiveType::Integer,
                        source_location: SourceLocation::unknown(),
                    }),
                    intent: None,
                    constraint: None,
                    passing_mode: PassingMode::ByValue,
                    source_location: SourceLocation::unknown(),
                },
            ],
            return_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                source_location: SourceLocation::unknown(),
            }),
            calling_convention: CallingConvention::C,
            thread_safe: true,
            may_block: false,
            variadic: false,
            ownership_info: None,
            source_location: SourceLocation::unknown(),
        }
    }
    
    #[test]
    fn test_ffi_type_mapper() {
        let mapper = FFITypeMapper::new();
        
        // Test C mappings
        let int_type = Type::primitive(PrimitiveType::Integer);
        assert_eq!(mapper.map_to_c_type(&int_type).unwrap(), "int64_t");
        
        let float_type = Type::primitive(PrimitiveType::Float);
        assert_eq!(mapper.map_to_c_type(&float_type).unwrap(), "double");
        
        // Test Rust mappings
        assert_eq!(mapper.map_to_rust_type(&int_type).unwrap(), "i64");
        assert_eq!(mapper.map_to_rust_type(&float_type).unwrap(), "f64");
        
        // Test Go mappings
        assert_eq!(mapper.map_to_go_type(&int_type).unwrap(), "int64");
        assert_eq!(mapper.map_to_go_type(&float_type).unwrap(), "float64");
    }
    
    #[test]
    fn test_pointer_type_mapping() {
        let mapper = FFITypeMapper::new();
        
        let ptr_type = Type::pointer(Type::primitive(PrimitiveType::Integer), false);
        assert_eq!(mapper.map_to_c_type(&ptr_type).unwrap(), "const int64_t*");
        assert_eq!(mapper.map_to_rust_type(&ptr_type).unwrap(), "*const i64");
        
        let mut_ptr_type = Type::pointer(Type::primitive(PrimitiveType::Integer), true);
        assert_eq!(mapper.map_to_c_type(&mut_ptr_type).unwrap(), "int64_t*");
        assert_eq!(mapper.map_to_rust_type(&mut_ptr_type).unwrap(), "*mut i64");
    }
    
    #[test]
    fn test_ffi_analyzer() {
        let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
        let mut analyzer = FFIAnalyzer::new(type_checker);
        
        let ext_func = create_test_external_function();
        assert!(analyzer.analyze_external_function(&ext_func).is_ok());
        
        assert_eq!(analyzer.get_external_functions().len(), 1);
        assert!(analyzer.get_external_functions().contains_key("test_add"));
    }
    
    #[test]
    fn test_c_header_generation() {
        let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
        let mut analyzer = FFIAnalyzer::new(type_checker);
        
        let ext_func = create_test_external_function();
        analyzer.analyze_external_function(&ext_func).unwrap();
        
        let header = analyzer.generate_c_header("test_module");
        assert!(header.contains("#ifndef TEST_MODULE_H"));
        assert!(header.contains("#include <stdint.h>"));
        assert!(header.contains("int64_t test_add_impl test_add(int64_t a, int64_t b)"));
    }
    
    #[test]
    fn test_callback_registry() {
        let mut registry = CallbackRegistry::new();
        
        let signature = FunctionSignature {
            parameters: vec![
                Type::primitive(PrimitiveType::Integer),
                Type::primitive(PrimitiveType::Float),
            ],
            return_type: Type::primitive(PrimitiveType::Boolean),
        };
        
        let id = registry.register_callback(
            "my_callback".to_string(),
            signature,
            CallingConvention::C,
            true,
        );
        
        assert_eq!(id, 1);
        assert!(registry.get_callback(id).is_some());
        assert!(registry.get_callback_by_name("my_callback").is_some());
        
        let callback = registry.get_callback_by_name("my_callback").unwrap();
        assert_eq!(callback.function_name, "my_callback");
        assert!(callback.thread_safe);
        assert_eq!(callback.calling_convention, CallingConvention::C);
    }
    
    #[test]
    fn test_callback_c_generation() {
        let mut registry = CallbackRegistry::new();
        let type_mapper = FFITypeMapper::new();
        
        let signature = FunctionSignature {
            parameters: vec![Type::primitive(PrimitiveType::Integer)],
            return_type: Type::primitive(PrimitiveType::Void),
        };
        
        registry.register_callback(
            "test_callback".to_string(),
            signature,
            CallingConvention::C,
            true,
        );
        
        let declarations = registry.generate_c_callback_declarations(&type_mapper);
        assert!(declarations.contains("typedef void (*test_callback)(int64_t param0)"));
        assert!(declarations.contains("void register_test_callback(test_callback*)"));
    }
    
    #[test]
    fn test_struct_handler() {
        let mut handler = FFIStructHandler::new();
        
        let struct_type = StructType {
            name: "Point".to_string(),
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::primitive(PrimitiveType::Float),
                    offset: None,
                },
                StructField {
                    name: "y".to_string(),
                    field_type: Type::primitive(PrimitiveType::Float),
                    offset: None,
                },
            ],
            packed: false,
            alignment: None,
        };
        
        handler.register_struct(struct_type);
        
        let c_declarations = handler.generate_c_struct_declarations();
        assert!(c_declarations.contains("typedef struct Point {"));
        assert!(c_declarations.contains("double x;"));
        assert!(c_declarations.contains("double y;"));
        assert!(c_declarations.contains("} Point;"));
        
        let rust_declarations = handler.generate_rust_struct_declarations();
        assert!(rust_declarations.contains("#[repr(C)]"));
        assert!(rust_declarations.contains("pub struct Point {"));
        assert!(rust_declarations.contains("pub x: f64,"));
        assert!(rust_declarations.contains("pub y: f64,"));
    }
    
    #[test]
    fn test_enum_handler() {
        let mut handler = FFIStructHandler::new();
        
        let enum_type = EnumType {
            name: "Color".to_string(),
            variants: vec![
                EnumVariant {
                    name: "Red".to_string(),
                    value: Some(0),
                },
                EnumVariant {
                    name: "Green".to_string(),
                    value: Some(1),
                },
                EnumVariant {
                    name: "Blue".to_string(),
                    value: Some(2),
                },
            ],
            underlying_type: Type::primitive(PrimitiveType::Integer),
        };
        
        handler.register_enum(enum_type);
        
        let c_declarations = handler.generate_c_struct_declarations();
        assert!(c_declarations.contains("typedef enum : int64_t {"));
        assert!(c_declarations.contains("COLOR__RED = 0,"));
        assert!(c_declarations.contains("COLOR__GREEN = 1,"));
        assert!(c_declarations.contains("COLOR__BLUE = 2,"));
        assert!(c_declarations.contains("} Color;"));
        
        let rust_declarations = handler.generate_rust_struct_declarations();
        assert!(rust_declarations.contains("#[repr(i64)]"));
        assert!(rust_declarations.contains("pub enum Color {"));
        assert!(rust_declarations.contains("Red = 0,"));
        assert!(rust_declarations.contains("Green = 1,"));
        assert!(rust_declarations.contains("Blue = 2,"));
    }
    
    #[test]
    fn test_binding_generator() {
        let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
        let mut generator = BindingGenerator::new(type_checker);
        
        // Add external function
        let ext_func = create_test_external_function();
        generator.add_external_function(&ext_func).unwrap();
        
        // Add callback
        let signature = FunctionSignature {
            parameters: vec![Type::primitive(PrimitiveType::Integer)],
            return_type: Type::primitive(PrimitiveType::Void),
        };
        generator.add_callback(
            "event_handler".to_string(),
            signature,
            CallingConvention::C,
            true,
        );
        
        // Add struct
        let struct_type = StructType {
            name: "TestStruct".to_string(),
            fields: vec![
                StructField {
                    name: "value".to_string(),
                    field_type: Type::primitive(PrimitiveType::Integer),
                    offset: None,
                },
            ],
            packed: false,
            alignment: None,
        };
        generator.add_struct(struct_type);
        
        // Generate C bindings
        let c_bindings = generator.generate_complete_bindings("test", TargetLanguage::C).unwrap();
        assert!(c_bindings.contains("#ifndef TEST_BINDINGS_H"));
        assert!(c_bindings.contains("typedef struct TestStruct {"));
        assert!(c_bindings.contains("typedef void (*event_handler)"));
        
        // Generate Rust bindings
        let rust_bindings = generator.generate_complete_bindings("test", TargetLanguage::Rust).unwrap();
        assert!(rust_bindings.contains("use std::os::raw"));
        assert!(rust_bindings.contains("#[repr(C)]"));
        assert!(rust_bindings.contains("pub type event_handler_callback"));
        
        // Generate Go bindings
        let go_bindings = generator.generate_complete_bindings("test", TargetLanguage::Go).unwrap();
        assert!(go_bindings.contains("package test_bindings"));
        assert!(go_bindings.contains("import \"C\""));
    }
    
    #[test]
    fn test_target_language_display() {
        assert_eq!(format!("{}", TargetLanguage::C), "C");
        assert_eq!(format!("{}", TargetLanguage::Rust), "Rust");
        assert_eq!(format!("{}", TargetLanguage::Go), "Go");
    }

    #[test]
    fn test_rust_bindings_generation() {
        let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
        let mut analyzer = FFIAnalyzer::new(type_checker);
        
        let ext_func = create_test_external_function();
        analyzer.analyze_external_function(&ext_func).unwrap();
        
        let generator = FFIGenerator::new(analyzer);
        let bindings = generator.generate_rust_bindings("test_module");
        
        assert!(bindings.contains("use std::os::raw"));
        assert!(bindings.contains("#[link(name = \"test_module\")]"));
        assert!(bindings.contains("pub fn test_add_impl(a: i64, b: i64) -> i64;"));
    }
    
    #[test]
    fn test_ownership_validation() {
        let type_checker = Rc::new(RefCell::new(TypeChecker::new()));
        let mut analyzer = FFIAnalyzer::new(type_checker);
        
        // Create function with pointer parameter but no ownership info
        let mut ext_func = create_test_external_function();
        ext_func.parameters[0].param_type = Box::new(TypeSpecifier::Pointer {
            target_type: Box::new(TypeSpecifier::Primitive {
                type_name: PrimitiveType::Integer,
                source_location: SourceLocation::unknown(),
            }),
            is_mutable: false,
            source_location: SourceLocation::unknown(),
        });
        
        // For C calling convention, ownership info is not required
        // Should succeed even without ownership info
        let result = analyzer.analyze_external_function(&ext_func);
        assert!(result.is_ok());
        
        // Change to System calling convention - now it should require ownership info
        ext_func.calling_convention = CallingConvention::System;
        let result = analyzer.analyze_external_function(&ext_func);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ownership info"));
        
        // Add ownership info
        ext_func.ownership_info = Some(OwnershipInfo {
            ownership: Ownership::Borrowed,
            lifetime: Some(Lifetime::CallDuration),
            deallocator: None,
        });
        
        // Should succeed with ownership info
        assert!(analyzer.analyze_external_function(&ext_func).is_ok());
    }
}