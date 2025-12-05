//! ABI (Application Binary Interface) for Aether separate compilation
//!
//! This module defines the metadata format for pre-compiled Aether modules.
//! When a module is compiled, it produces:
//! - `.o` file: Object code (machine code)
//! - `.abi` file: Module interface (this format)
//!
//! The ABI file contains everything needed to compile against a module
//! without having the source code.

pub mod generator;

use crate::error::SourceLocation;
use crate::types::Type;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

pub use generator::{generate_abi_file, AbiGenerator};

/// Current ABI format version
pub const ABI_VERSION: &str = "1.0.0";

// =============================================================================
// Top-Level ABI Structure
// =============================================================================

/// Complete ABI for a module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiModule {
    /// ABI format version (for compatibility checking)
    pub abi_version: String,

    /// Aether compiler version that produced this ABI
    pub aether_version: String,

    /// Module metadata
    pub module: ModuleInfo,

    /// Module dependencies
    pub dependencies: Vec<Dependency>,

    /// Exported functions
    pub functions: Vec<AbiFunction>,

    /// Exported types (structs, enums, type aliases)
    pub types: AbiTypes,

    /// Trait definitions
    pub traits: Vec<AbiTrait>,

    /// Trait implementations
    pub impls: Vec<AbiImpl>,

    /// MIR data for generic functions (JSON-serialized MIR::Function structs)
    #[serde(default)]
    pub mir_data: Vec<SerializedMir>,
}

impl AbiModule {
    /// Create a new empty ABI module
    pub fn new(module_name: String, source_path: String) -> Self {
        AbiModule {
            abi_version: ABI_VERSION.to_string(),
            aether_version: env!("CARGO_PKG_VERSION").to_string(),
            module: ModuleInfo {
                name: module_name,
                path: source_path,
                checksum: None,
            },
            dependencies: Vec::new(),
            functions: Vec::new(),
            types: AbiTypes::default(),
            traits: Vec::new(),
            impls: Vec::new(),
            mir_data: Vec::new(),
        }
    }

    /// Load an ABI from a file
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let abi: AbiModule = serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Version compatibility check
        if !Self::is_compatible_version(&abi.abi_version) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Incompatible ABI version: {} (expected {})",
                    abi.abi_version, ABI_VERSION
                ),
            ));
        }

        Ok(abi)
    }

    /// Save an ABI to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }

    /// Check if an ABI version is compatible with the current version
    fn is_compatible_version(version: &str) -> bool {
        // For now, require exact major version match
        let current_major = ABI_VERSION.split('.').next().unwrap_or("0");
        let other_major = version.split('.').next().unwrap_or("0");
        current_major == other_major
    }

    /// Find a function by name
    pub fn find_function(&self, name: &str) -> Option<&AbiFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Find a struct by name
    pub fn find_struct(&self, name: &str) -> Option<&AbiStruct> {
        self.types.structs.iter().find(|s| s.name == name)
    }

    /// Find an enum by name
    pub fn find_enum(&self, name: &str) -> Option<&AbiEnum> {
        self.types.enums.iter().find(|e| e.name == name)
    }

    /// Find a trait by name
    pub fn find_trait(&self, name: &str) -> Option<&AbiTrait> {
        self.traits.iter().find(|t| t.name == name)
    }

    /// Find MIR for a generic function
    pub fn find_mir(&self, function_name: &str) -> Option<&SerializedMir> {
        self.mir_data
            .iter()
            .find(|mir| mir.function_name == function_name)
    }
}

/// Serialized MIR for a generic function
///
/// Generic functions can't be compiled to object code directly - they need
/// to be monomorphized first. This structure stores the MIR representation
/// so that importing modules can instantiate the generic function with
/// concrete types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMir {
    /// Name of the generic function
    pub function_name: String,

    /// JSON-serialized MIR::Function
    pub mir_json: String,

    /// Generic parameters (for validation during instantiation)
    pub generic_params: Vec<String>,
}

impl SerializedMir {
    /// Create a new SerializedMir from a MIR function
    pub fn from_mir_function(
        function_name: String,
        mir_func: &crate::mir::Function,
        generic_params: Vec<String>,
    ) -> Result<Self, serde_json::Error> {
        let mir_json = serde_json::to_string(mir_func)?;
        Ok(SerializedMir {
            function_name,
            mir_json,
            generic_params,
        })
    }

    /// Deserialize back to a MIR function
    pub fn to_mir_function(&self) -> Result<crate::mir::Function, serde_json::Error> {
        serde_json::from_str(&self.mir_json)
    }
}

/// Module identification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Fully qualified module name (e.g., "std.io")
    pub name: String,

    /// Original source file path
    pub path: String,

    /// SHA-256 checksum of source file (for cache invalidation)
    pub checksum: Option<String>,
}

/// Module dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Module name being imported
    pub module: String,

    /// Version constraint (for future package management)
    pub version_constraint: Option<String>,

    /// Specific items imported
    pub imports: Vec<ImportItem>,
}

/// Single imported item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportItem {
    /// Original name in the source module
    pub name: String,

    /// Alias in the importing module (None if same as name)
    pub alias: Option<String>,
}

// =============================================================================
// Function ABI
// =============================================================================

/// Function declaration in ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFunction {
    /// Function name
    pub name: String,

    /// Function signature
    pub signature: FunctionSignature,

    /// Function kind (native Aether, extern FFI, or generic)
    pub kind: FunctionKind,

    /// Pre/post conditions and verification status
    pub contracts: FunctionContracts,

    /// Function attributes (export, inline, pure, etc.)
    pub attributes: Vec<String>,

    /// Source location for error messages
    pub source_location: AbiSourceLocation,
}

/// Function signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Generic type parameters (e.g., T, U in func<T, U>)
    pub generic_params: Vec<GenericParam>,

    /// Where clause constraints
    pub where_clauses: Vec<WhereClause>,

    /// Function parameters
    pub parameters: Vec<FunctionParameter>,

    /// Return type
    pub return_type: AbiType,

    /// Whether function is variadic
    pub is_variadic: bool,
}

/// Generic type parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericParam {
    /// Parameter name (e.g., "T")
    pub name: String,

    /// Parameter kind (type, lifetime, const)
    pub kind: GenericParamKind,

    /// Default type (if any)
    pub default: Option<AbiType>,
}

/// Kind of generic parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenericParamKind {
    /// Type parameter
    Type,
    /// Lifetime parameter (future)
    Lifetime,
    /// Const parameter (future)
    Const { ty: AbiType },
}

/// Where clause constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereClause {
    /// Type being constrained
    pub constrained_type: AbiType,

    /// Trait that must be implemented
    pub trait_bound: String,

    /// Module containing the trait
    pub trait_module: Option<String>,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameter {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub ty: AbiType,

    /// Parameter passing mode
    pub mode: ParameterMode,
}

/// How a parameter is passed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterMode {
    /// Value is moved (default for non-Copy types)
    Owned,
    /// Immutable borrow
    Borrowed,
    /// Mutable borrow
    MutableBorrow,
}

/// Function kind
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FunctionKind {
    /// Native Aether function (compiled to object code)
    Native {
        /// Symbol name in object file
        symbol: String,
    },

    /// External FFI function
    Extern {
        /// Library containing the function
        library: String,
        /// Symbol name in library
        symbol: String,
        /// Calling convention
        calling_convention: CallingConvention,
    },

    /// Generic function (requires MIR for monomorphization)
    Generic {
        /// Symbol prefix (monomorphized versions get suffix)
        symbol_prefix: String,
        /// Whether MIR is stored in this ABI
        has_mir: bool,
        /// Offset into MIR section (if has_mir)
        mir_offset: Option<usize>,
        /// Length of MIR data
        mir_length: Option<usize>,
    },
}

/// Calling convention for extern functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallingConvention {
    /// C calling convention (default)
    C,
    /// Rust calling convention
    Rust,
    /// System default
    System,
}

/// Function contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionContracts {
    /// Preconditions (@pre)
    pub preconditions: Vec<Contract>,

    /// Postconditions (@post)
    pub postconditions: Vec<Contract>,

    /// Whether contracts have been verified
    pub verified: bool,

    /// Axioms assumed for verification
    pub assumes_axioms: Vec<String>,
}

impl Default for FunctionContracts {
    fn default() -> Self {
        FunctionContracts {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            verified: false,
            assumes_axioms: Vec::new(),
        }
    }
}

/// Single contract (pre/post condition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Contract expression (as string, parseable)
    pub expr: String,

    /// Human-readable message
    pub message: Option<String>,
}

// =============================================================================
// Type ABI
// =============================================================================

/// All type definitions in a module
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbiTypes {
    /// Struct definitions
    pub structs: Vec<AbiStruct>,

    /// Enum definitions
    pub enums: Vec<AbiEnum>,

    /// Type aliases
    pub type_aliases: Vec<AbiTypeAlias>,
}

/// Struct definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiStruct {
    /// Struct name
    pub name: String,

    /// Generic parameters
    pub generic_params: Vec<GenericParam>,

    /// Where clause constraints
    pub where_clauses: Vec<WhereClause>,

    /// Fields
    pub fields: Vec<StructField>,

    /// Struct attributes (derive, repr, etc.)
    pub attributes: Vec<String>,

    /// Invariants (@invariant contracts)
    pub invariants: Vec<Contract>,

    /// Source location
    pub source_location: AbiSourceLocation,
}

/// Struct field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    /// Field name
    pub name: String,

    /// Field type
    pub ty: AbiType,

    /// Field visibility
    pub visibility: Visibility,
}

/// Enum definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiEnum {
    /// Enum name
    pub name: String,

    /// Generic parameters
    pub generic_params: Vec<GenericParam>,

    /// Where clause constraints
    pub where_clauses: Vec<WhereClause>,

    /// Variants
    pub variants: Vec<EnumVariant>,

    /// Enum attributes
    pub attributes: Vec<String>,

    /// Source location
    pub source_location: AbiSourceLocation,
}

/// Enum variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    /// Variant name
    pub name: String,

    /// Variant fields (empty for unit variants)
    pub fields: Vec<VariantField>,

    /// Discriminant value (if explicitly specified)
    pub discriminant: Option<i64>,
}

/// Enum variant field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantField {
    /// Field name (None for tuple variants)
    pub name: Option<String>,

    /// Field type
    pub ty: AbiType,
}

/// Type alias definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiTypeAlias {
    /// Alias name
    pub name: String,

    /// Generic parameters
    pub generic_params: Vec<GenericParam>,

    /// Target type
    pub target: AbiType,

    /// Source location
    pub source_location: AbiSourceLocation,
}

/// Visibility level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    /// Public (exported from module)
    Public,
    /// Private (module-internal)
    Private,
    /// Visible within crate/package
    Crate,
}

// =============================================================================
// Trait ABI
// =============================================================================

/// Trait definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiTrait {
    /// Trait name
    pub name: String,

    /// Generic parameters
    pub generic_params: Vec<GenericParam>,

    /// Super traits (traits this trait extends)
    pub super_traits: Vec<TraitRef>,

    /// Associated types
    pub associated_types: Vec<AssociatedType>,

    /// Trait methods
    pub methods: Vec<TraitMethod>,

    /// Trait axioms (for verification)
    pub axioms: Vec<TraitAxiom>,

    /// Source location
    pub source_location: AbiSourceLocation,
}

/// Reference to a trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitRef {
    /// Trait name
    pub name: String,

    /// Module containing trait
    pub module: Option<String>,

    /// Type arguments (for generic traits)
    pub type_args: Vec<AbiType>,
}

/// Associated type in a trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedType {
    /// Type name
    pub name: String,

    /// Bounds on the associated type
    pub bounds: Vec<TraitRef>,

    /// Default type (if any)
    pub default: Option<AbiType>,
}

/// Trait method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitMethod {
    /// Method name
    pub name: String,

    /// Method signature
    pub signature: FunctionSignature,

    /// Whether method has a default implementation
    pub has_default: bool,

    /// Contracts on the method
    pub contracts: FunctionContracts,
}

/// Trait axiom (for verification)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitAxiom {
    /// Axiom name
    pub name: String,

    /// Quantified variables
    pub quantifiers: Vec<Quantifier>,

    /// Axiom expression (as parseable string)
    pub expr: String,

    /// Source location
    pub source_location: AbiSourceLocation,
}

/// Quantifier for axioms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantifier {
    /// Variable name
    pub var: String,

    /// Variable type
    pub ty: AbiType,

    /// Quantifier kind
    pub kind: QuantifierKind,
}

/// Kind of quantifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuantifierKind {
    /// Universal quantifier (forall)
    ForAll,
    /// Existential quantifier (exists)
    Exists,
}

// =============================================================================
// Trait Implementation ABI
// =============================================================================

/// Trait implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiImpl {
    /// Trait being implemented
    pub trait_ref: TraitRef,

    /// Type implementing the trait
    pub for_type: AbiType,

    /// Generic parameters on the impl
    pub generic_params: Vec<GenericParam>,

    /// Where clause constraints
    pub where_clauses: Vec<WhereClause>,

    /// Associated type assignments
    pub associated_types: Vec<AssociatedTypeImpl>,

    /// Method implementations
    pub methods: Vec<MethodImpl>,

    /// Source location
    pub source_location: AbiSourceLocation,
}

/// Associated type implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedTypeImpl {
    /// Associated type name
    pub name: String,

    /// Concrete type
    pub ty: AbiType,
}

/// Method implementation reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodImpl {
    /// Method name
    pub name: String,

    /// Symbol in object file
    pub symbol: String,
}

// =============================================================================
// ABI Type Representation
// =============================================================================

/// Type representation in ABI (serializable version of Type)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AbiType {
    /// Primitive type
    Primitive { name: String },

    /// Named type (struct, enum, or alias)
    Named { name: String, module: Option<String> },

    /// Array type
    Array {
        element: Box<AbiType>,
        size: Option<usize>,
    },

    /// Map type
    Map {
        key: Box<AbiType>,
        value: Box<AbiType>,
    },

    /// Function type
    Function {
        params: Vec<AbiType>,
        ret: Box<AbiType>,
    },

    /// Reference type
    Reference { target: Box<AbiType>, mutable: bool },

    /// Pointer type
    Pointer { target: Box<AbiType>, mutable: bool },

    /// Generic instance (e.g., Vec<Int>)
    GenericInstance {
        base: String,
        module: Option<String>,
        args: Vec<AbiType>,
    },

    /// Generic parameter (e.g., T in func<T>)
    GenericParam { name: String },

    /// Tuple type
    Tuple { elements: Vec<AbiType> },

    /// Unit/void type
    Unit,

    /// Never type (for functions that don't return)
    Never,
}

impl AbiType {
    /// Convert from internal Type to AbiType
    pub fn from_type(ty: &Type) -> Self {
        use crate::ast::PrimitiveType as PT;

        match ty {
            Type::Primitive(p) => AbiType::Primitive {
                name: match p {
                    PT::Integer => "Int".to_string(),
                    PT::Integer32 => "Int32".to_string(),
                    PT::Integer64 => "Int64".to_string(),
                    PT::Float => "Float".to_string(),
                    PT::Float32 => "Float32".to_string(),
                    PT::Float64 => "Float64".to_string(),
                    PT::Boolean => "Bool".to_string(),
                    PT::String => "String".to_string(),
                    PT::Char => "Char".to_string(),
                    PT::Void => "Void".to_string(),
                    PT::SizeT => "SizeT".to_string(),
                    PT::UIntPtrT => "UIntPtrT".to_string(),
                    PT::UInt8 => "UInt8".to_string(),
                    PT::Int8 => "Int8".to_string(),
                    PT::UInt16 => "UInt16".to_string(),
                    PT::Int16 => "Int16".to_string(),
                    PT::UInt32 => "UInt32".to_string(),
                    PT::Int32 => "Int32".to_string(),
                    PT::UInt64 => "UInt64".to_string(),
                },
            },

            Type::Named { name, module } => AbiType::Named {
                name: name.clone(),
                module: module.clone(),
            },

            Type::Array { element_type, size } => AbiType::Array {
                element: Box::new(AbiType::from_type(element_type)),
                size: *size,
            },

            Type::Map {
                key_type,
                value_type,
            } => AbiType::Map {
                key: Box::new(AbiType::from_type(key_type)),
                value: Box::new(AbiType::from_type(value_type)),
            },

            Type::Function {
                parameter_types,
                return_type,
                ..
            } => AbiType::Function {
                params: parameter_types.iter().map(AbiType::from_type).collect(),
                ret: Box::new(AbiType::from_type(return_type)),
            },

            Type::Pointer {
                target_type,
                is_mutable,
            } => AbiType::Pointer {
                target: Box::new(AbiType::from_type(target_type)),
                mutable: *is_mutable,
            },

            Type::Generic { name, .. } => AbiType::GenericParam { name: name.clone() },

            Type::GenericInstance {
                base_type,
                type_arguments,
                module,
            } => AbiType::GenericInstance {
                base: base_type.clone(),
                module: module.clone(),
                args: type_arguments.iter().map(AbiType::from_type).collect(),
            },

            Type::Owned { base_type, .. } => AbiType::from_type(base_type),

            Type::Module(_) => AbiType::Unit, // Modules aren't types in ABI

            Type::Variable(_) => AbiType::Unit, // Should be resolved before ABI generation

            Type::Error => AbiType::Unit,
        }
    }

    /// Convert from AbiType to internal Type
    pub fn to_type(&self) -> Type {
        use crate::ast::PrimitiveType as PT;

        match self {
            AbiType::Primitive { name } => {
                let prim = match name.as_str() {
                    "Int" => PT::Integer,
                    "Int32" => PT::Integer32,
                    "Int64" => PT::Integer64,
                    "Float" => PT::Float,
                    "Float32" => PT::Float32,
                    "Float64" => PT::Float64,
                    "Bool" => PT::Boolean,
                    "String" => PT::String,
                    "Char" => PT::Char,
                    "Void" => PT::Void,
                    "SizeT" => PT::SizeT,
                    "UIntPtrT" => PT::UIntPtrT,
                    "UInt8" => PT::UInt8,
                    "Int8" => PT::Int8,
                    "UInt16" => PT::UInt16,
                    "Int16" => PT::Int16,
                    "UInt32" => PT::UInt32,
                    "UInt64" => PT::UInt64,
                    _ => PT::Void, // Fallback
                };
                Type::Primitive(prim)
            }

            AbiType::Named { name, module } => Type::Named {
                name: name.clone(),
                module: module.clone(),
            },

            AbiType::Array { element, size } => Type::Array {
                element_type: Box::new(element.to_type()),
                size: *size,
            },

            AbiType::Map { key, value } => Type::Map {
                key_type: Box::new(key.to_type()),
                value_type: Box::new(value.to_type()),
            },

            AbiType::Function { params, ret } => Type::Function {
                parameter_types: params.iter().map(|p| p.to_type()).collect(),
                return_type: Box::new(ret.to_type()),
                is_variadic: false,
            },

            AbiType::Reference { target, .. } | AbiType::Pointer { target, mutable: _ } => {
                Type::Pointer {
                    target_type: Box::new(target.to_type()),
                    is_mutable: matches!(self, AbiType::Pointer { mutable: true, .. }),
                }
            }

            AbiType::GenericInstance { base, module, args } => Type::GenericInstance {
                base_type: base.clone(),
                type_arguments: args.iter().map(|a| a.to_type()).collect(),
                module: module.clone(),
            },

            AbiType::GenericParam { name } => Type::Generic {
                name: name.clone(),
                constraints: Vec::new(),
            },

            AbiType::Tuple { elements } => {
                // TODO: Proper tuple type support
                if elements.is_empty() {
                    Type::Primitive(PT::Void)
                } else {
                    elements[0].to_type()
                }
            }

            AbiType::Unit => Type::Primitive(PT::Void),

            AbiType::Never => Type::Primitive(PT::Void), // TODO: Proper never type
        }
    }
}

// =============================================================================
// Source Location
// =============================================================================

/// Source location for error messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiSourceLocation {
    /// Line number (1-indexed)
    pub line: usize,

    /// Column number (1-indexed)
    pub column: usize,
}

impl From<&SourceLocation> for AbiSourceLocation {
    fn from(loc: &SourceLocation) -> Self {
        AbiSourceLocation {
            line: loc.line,
            column: loc.column,
        }
    }
}

impl AbiSourceLocation {
    pub fn unknown() -> Self {
        AbiSourceLocation { line: 0, column: 0 }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_module_creation() {
        let abi = AbiModule::new("test.module".to_string(), "test/module.aether".to_string());
        assert_eq!(abi.module.name, "test.module");
        assert_eq!(abi.abi_version, ABI_VERSION);
    }

    #[test]
    fn test_abi_serialization_roundtrip() {
        let mut abi = AbiModule::new("std.io".to_string(), "stdlib/io.aether".to_string());

        // Add a function
        abi.functions.push(AbiFunction {
            name: "print".to_string(),
            signature: FunctionSignature {
                generic_params: Vec::new(),
                where_clauses: Vec::new(),
                parameters: vec![FunctionParameter {
                    name: "message".to_string(),
                    ty: AbiType::Primitive {
                        name: "String".to_string(),
                    },
                    mode: ParameterMode::Owned,
                }],
                return_type: AbiType::Unit,
                is_variadic: false,
            },
            kind: FunctionKind::Extern {
                library: "aether_runtime".to_string(),
                symbol: "aether_print".to_string(),
                calling_convention: CallingConvention::C,
            },
            contracts: FunctionContracts::default(),
            attributes: vec!["export".to_string()],
            source_location: AbiSourceLocation { line: 10, column: 1 },
        });

        // Serialize
        let json = serde_json::to_string_pretty(&abi).unwrap();
        assert!(json.contains("print"));
        assert!(json.contains("aether_print"));

        // Deserialize
        let loaded: AbiModule = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.module.name, "std.io");
        assert_eq!(loaded.functions.len(), 1);
        assert_eq!(loaded.functions[0].name, "print");
    }

    #[test]
    fn test_abi_type_conversion() {
        use crate::ast::PrimitiveType;

        // Test primitive type conversion
        let int_type = Type::Primitive(PrimitiveType::Integer);
        let abi_type = AbiType::from_type(&int_type);
        assert_eq!(
            abi_type,
            AbiType::Primitive {
                name: "Int".to_string()
            }
        );

        // Test round-trip
        let back = abi_type.to_type();
        assert_eq!(back, int_type);

        // Test generic instance
        let vec_int = Type::GenericInstance {
            base_type: "Vec".to_string(),
            type_arguments: vec![Type::Primitive(PrimitiveType::Integer)],
            module: Some("std.collections".to_string()),
        };
        let abi_vec = AbiType::from_type(&vec_int);
        match &abi_vec {
            AbiType::GenericInstance { base, args, .. } => {
                assert_eq!(base, "Vec");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected GenericInstance"),
        }
    }

    #[test]
    fn test_find_function() {
        let mut abi = AbiModule::new("test".to_string(), "test.aether".to_string());
        abi.functions.push(AbiFunction {
            name: "foo".to_string(),
            signature: FunctionSignature {
                generic_params: Vec::new(),
                where_clauses: Vec::new(),
                parameters: Vec::new(),
                return_type: AbiType::Unit,
                is_variadic: false,
            },
            kind: FunctionKind::Native {
                symbol: "test_foo".to_string(),
            },
            contracts: FunctionContracts::default(),
            attributes: Vec::new(),
            source_location: AbiSourceLocation::unknown(),
        });

        assert!(abi.find_function("foo").is_some());
        assert!(abi.find_function("bar").is_none());
    }

    #[test]
    fn test_version_compatibility() {
        assert!(AbiModule::is_compatible_version("1.0.0"));
        assert!(AbiModule::is_compatible_version("1.5.0"));
        assert!(!AbiModule::is_compatible_version("2.0.0"));
        assert!(!AbiModule::is_compatible_version("0.9.0"));
    }

    #[test]
    fn test_mir_serialization_roundtrip() {
        use crate::mir::{Function, Parameter, BasicBlock, Terminator};
        use crate::types::Type;
        use crate::ast::PrimitiveType;
        use std::collections::HashMap;

        // Create a simple MIR function
        let mir_func = Function {
            name: "add".to_string(),
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    ty: Type::Primitive(PrimitiveType::Integer),
                    local_id: 0,
                },
                Parameter {
                    name: "b".to_string(),
                    ty: Type::Primitive(PrimitiveType::Integer),
                    local_id: 1,
                },
            ],
            return_type: Type::Primitive(PrimitiveType::Integer),
            locals: HashMap::new(),
            basic_blocks: {
                let mut blocks = HashMap::new();
                blocks.insert(
                    0,
                    BasicBlock {
                        id: 0,
                        statements: vec![],
                        terminator: Terminator::Return,
                    },
                );
                blocks
            },
            entry_block: 0,
            return_local: Some(2),
        };

        // Serialize to SerializedMir
        let serialized = SerializedMir::from_mir_function(
            "add".to_string(),
            &mir_func,
            vec!["T".to_string()],
        )
        .expect("Failed to serialize MIR");

        assert_eq!(serialized.function_name, "add");
        assert_eq!(serialized.generic_params, vec!["T".to_string()]);

        // Deserialize back
        let deserialized = serialized
            .to_mir_function()
            .expect("Failed to deserialize MIR");

        assert_eq!(deserialized.name, "add");
        assert_eq!(deserialized.parameters.len(), 2);
        assert_eq!(deserialized.parameters[0].name, "a");
        assert_eq!(deserialized.parameters[1].name, "b");
    }

    #[test]
    fn test_find_mir() {
        let mut abi = AbiModule::new("test".to_string(), "test.aether".to_string());

        // Add a SerializedMir
        abi.mir_data.push(SerializedMir {
            function_name: "generic_func".to_string(),
            mir_json: "{}".to_string(),
            generic_params: vec!["T".to_string()],
        });

        assert!(abi.find_mir("generic_func").is_some());
        assert!(abi.find_mir("nonexistent").is_none());
    }
}
