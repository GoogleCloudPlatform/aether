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

//! Type system for AetherScript
//!
//! Implements type checking, inference, and compatibility checking

use crate::ast::{PrimitiveType, TypeConstraint, TypeConstraintKind, TypeSpecifier};
use crate::error::{SemanticError, SourceLocation};
use std::collections::HashMap;
use std::fmt;

/// Ownership kind for AetherScript's ownership system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OwnershipKind {
    /// ^T - Single owner, value is moved on assignment
    Owned,
    /// &T - Immutable borrow, multiple readers allowed
    Borrowed,
    /// &mut T - Mutable borrow, exclusive access
    MutableBorrow,
    /// ~T - Reference counted, shared ownership
    Shared,
}

/// Type constraint information for generic parameters
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeConstraintInfo {
    /// Type must implement a trait/interface
    TraitBound {
        trait_name: String,
        module: Option<String>,
    },
    /// Type must be a subtype of another type
    SubtypeBound { parent_type: Type },
    /// Type must be a numeric type
    NumericBound,
    /// Type must be an equality-comparable type
    EqualityBound,
    /// Type must be an order-comparable type
    OrderBound,
    /// Type must have a specific size
    SizeBound { size: usize },
}

/// Type representation in the type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Primitive types
    Primitive(PrimitiveType),

    /// Named types (user-defined structs, enums, type aliases)
    Named {
        name: String,
        module: Option<String>,
    },

    /// Array types
    Array {
        element_type: Box<Type>,
        size: Option<usize>, // None for dynamic arrays
    },

    /// Map types
    Map {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },

    /// Pointer types
    Pointer {
        target_type: Box<Type>,
        is_mutable: bool,
    },

    /// Function types
    Function {
        parameter_types: Vec<Type>,
        return_type: Box<Type>,
    },

    /// Generic type parameters (uninstantiated)
    Generic {
        name: String,
        constraints: Vec<TypeConstraintInfo>,
    },

    /// Generic type instantiation (e.g., List<Integer>, Result<String, Error>)
    GenericInstance {
        base_type: String,
        type_arguments: Vec<Type>,
        module: Option<String>,
    },

    /// Type variables for inference
    Variable(TypeVariable),

    /// Owned type with ownership semantics
    Owned {
        ownership: OwnershipKind,
        base_type: Box<Type>,
    },

    /// Error type for recovery
    Error,
}

impl Type {
    /// Create a new primitive type
    pub fn primitive(prim_type: PrimitiveType) -> Self {
        Type::Primitive(prim_type)
    }

    /// Create a new named type
    pub fn named(name: String, module: Option<String>) -> Self {
        Type::Named { name, module }
    }

    /// Create a new array type
    pub fn array(element_type: Type, size: Option<usize>) -> Self {
        Type::Array {
            element_type: Box::new(element_type),
            size,
        }
    }

    /// Create a new map type
    pub fn map(key_type: Type, value_type: Type) -> Self {
        Type::Map {
            key_type: Box::new(key_type),
            value_type: Box::new(value_type),
        }
    }

    /// Create a new pointer type
    pub fn pointer(target_type: Type, is_mutable: bool) -> Self {
        Type::Pointer {
            target_type: Box::new(target_type),
            is_mutable,
        }
    }

    /// Create a new function type
    pub fn function(parameter_types: Vec<Type>, return_type: Type) -> Self {
        Type::Function {
            parameter_types,
            return_type: Box::new(return_type),
        }
    }

    /// Create a new generic type parameter
    pub fn generic(name: String, constraints: Vec<TypeConstraintInfo>) -> Self {
        Type::Generic { name, constraints }
    }

    /// Create a new generic type instance
    pub fn generic_instance(
        base_type: String,
        type_arguments: Vec<Type>,
        module: Option<String>,
    ) -> Self {
        Type::GenericInstance {
            base_type,
            type_arguments,
            module,
        }
    }

    /// Create a new owned type (^T)
    pub fn owned(base_type: Type) -> Self {
        Type::Owned {
            ownership: OwnershipKind::Owned,
            base_type: Box::new(base_type),
        }
    }

    /// Create a new borrowed type (&T)
    pub fn borrowed(base_type: Type) -> Self {
        Type::Owned {
            ownership: OwnershipKind::Borrowed,
            base_type: Box::new(base_type),
        }
    }

    /// Create a new mutable borrow type (&mut T)
    pub fn mutable_borrow(base_type: Type) -> Self {
        Type::Owned {
            ownership: OwnershipKind::MutableBorrow,
            base_type: Box::new(base_type),
        }
    }

    /// Create a new shared type (~T)
    pub fn shared(base_type: Type) -> Self {
        Type::Owned {
            ownership: OwnershipKind::Shared,
            base_type: Box::new(base_type),
        }
    }

    /// Check if this type is a numeric type
    pub fn is_numeric(&self) -> bool {
        match self {
            Type::Primitive(PrimitiveType::Integer)
            | Type::Primitive(PrimitiveType::Integer32)
            | Type::Primitive(PrimitiveType::Integer64)
            | Type::Primitive(PrimitiveType::Float)
            | Type::Primitive(PrimitiveType::Float32)
            | Type::Primitive(PrimitiveType::Float64)
            | Type::Primitive(PrimitiveType::SizeT)
            | Type::Primitive(PrimitiveType::UIntPtrT) => true,
            _ => false,
        }
    }

    /// Check if this type is an integer type
    pub fn is_integer(&self) -> bool {
        match self {
            Type::Primitive(PrimitiveType::Integer)
            | Type::Primitive(PrimitiveType::Integer32)
            | Type::Primitive(PrimitiveType::Integer64)
            | Type::Primitive(PrimitiveType::SizeT)
            | Type::Primitive(PrimitiveType::UIntPtrT) => true,
            _ => false,
        }
    }

    /// Check if this type is a floating point type
    pub fn is_float(&self) -> bool {
        match self {
            Type::Primitive(PrimitiveType::Float)
            | Type::Primitive(PrimitiveType::Float32)
            | Type::Primitive(PrimitiveType::Float64) => true,
            _ => false,
        }
    }

    /// Check if this type is a pointer type
    pub fn is_pointer(&self) -> bool {
        matches!(self, Type::Pointer { .. })
    }

    /// Extract ownership kind if this is an owned type
    pub fn ownership_kind(&self) -> Option<OwnershipKind> {
        match self {
            Type::Owned { ownership, .. } => Some(*ownership),
            _ => None,
        }
    }

    /// Get the base type without ownership wrapper
    pub fn base_type(&self) -> &Type {
        match self {
            Type::Owned { base_type, .. } => base_type,
            _ => self,
        }
    }

    /// Check if this type is owned (^T)
    pub fn is_owned(&self) -> bool {
        matches!(
            self,
            Type::Owned {
                ownership: OwnershipKind::Owned,
                ..
            }
        )
    }

    /// Check if this type is borrowed (&T or &mut T)
    pub fn is_borrowed(&self) -> bool {
        matches!(
            self,
            Type::Owned {
                ownership: OwnershipKind::Borrowed | OwnershipKind::MutableBorrow,
                ..
            }
        )
    }

    /// Check if this type is void
    pub fn is_void(&self) -> bool {
        matches!(self, Type::Primitive(PrimitiveType::Void))
    }

    /// Get the size in bytes of this type (if known at compile time)
    pub fn size_bytes(&self) -> Option<usize> {
        match self {
            Type::Primitive(PrimitiveType::Boolean) => Some(1),
            Type::Primitive(PrimitiveType::Integer32) => Some(4),
            Type::Primitive(PrimitiveType::Integer64) => Some(8),
            Type::Primitive(PrimitiveType::Float32) => Some(4),
            Type::Primitive(PrimitiveType::Float64) => Some(8),
            Type::Primitive(PrimitiveType::SizeT) => Some(8), // Assuming 64-bit target
            Type::Primitive(PrimitiveType::UIntPtrT) => Some(8), // Assuming 64-bit target
            Type::Pointer { .. } => Some(8),                  // Assuming 64-bit target
            Type::Array {
                element_type,
                size: Some(size),
            } => element_type.size_bytes().map(|elem_size| elem_size * size),
            _ => None, // Dynamic size or unknown
        }
    }

    /// Extract ownership information from a type
    pub fn get_ownership(&self) -> Option<OwnershipKind> {
        match self {
            Type::Owned { ownership, .. } => Some(*ownership),
            _ => None,
        }
    }

    /// Check if this type requires ownership tracking
    pub fn requires_ownership(&self) -> bool {
        match self {
            Type::Primitive(PrimitiveType::Void)
            | Type::Primitive(PrimitiveType::Boolean)
            | Type::Primitive(PrimitiveType::Char)
            | Type::Primitive(PrimitiveType::Integer)
            | Type::Primitive(PrimitiveType::Integer32)
            | Type::Primitive(PrimitiveType::Integer64)
            | Type::Primitive(PrimitiveType::Float)
            | Type::Primitive(PrimitiveType::Float32)
            | Type::Primitive(PrimitiveType::Float64)
            | Type::Primitive(PrimitiveType::SizeT)
            | Type::Primitive(PrimitiveType::UIntPtrT) => false,
            Type::Primitive(PrimitiveType::String)
            | Type::Array { .. }
            | Type::Map { .. }
            | Type::Named { .. }
            | Type::Pointer { .. } => true,
            Type::Function { .. } => false, // Functions are not owned
            Type::Owned { .. } => true,     // Owned types always require ownership tracking
            Type::Error
            | Type::Variable(_)
            | Type::Generic { .. }
            | Type::GenericInstance { .. } => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(prim) => write!(f, "{:?}", prim),
            Type::Named {
                name,
                module: Some(module),
            } => write!(f, "{}::{}", module, name),
            Type::Named { name, module: None } => write!(f, "{}", name),
            Type::Array {
                element_type,
                size: Some(size),
            } => {
                write!(f, "[{}; {}]", element_type, size)
            }
            Type::Array {
                element_type,
                size: None,
            } => {
                write!(f, "[{}]", element_type)
            }
            Type::Map {
                key_type,
                value_type,
            } => {
                write!(f, "Map<{}, {}>", key_type, value_type)
            }
            Type::Pointer {
                target_type,
                is_mutable,
            } => {
                if *is_mutable {
                    write!(f, "*mut {}", target_type)
                } else {
                    write!(f, "*const {}", target_type)
                }
            }
            Type::Function {
                parameter_types,
                return_type,
            } => {
                let params: Vec<String> = parameter_types.iter().map(|t| t.to_string()).collect();
                write!(f, "fn({}) -> {}", params.join(", "), return_type)
            }
            Type::Generic { name, constraints } => {
                if constraints.is_empty() {
                    write!(f, "{}", name)
                } else {
                    let constraint_strs: Vec<String> =
                        constraints.iter().map(|c| format!("{:?}", c)).collect();
                    write!(f, "{}: {}", name, constraint_strs.join(" + "))
                }
            }
            Type::GenericInstance {
                base_type,
                type_arguments,
                module,
            } => {
                let type_name = match module {
                    Some(m) => format!("{}::{}", m, base_type),
                    None => base_type.clone(),
                };
                let args: Vec<String> = type_arguments.iter().map(|t| t.to_string()).collect();
                write!(f, "{}<{}>", type_name, args.join(", "))
            }
            Type::Variable(var) => write!(f, "${}", var.id),
            Type::Owned {
                ownership,
                base_type,
            } => {
                let prefix = match ownership {
                    OwnershipKind::Owned => "^",
                    OwnershipKind::Borrowed => "&",
                    OwnershipKind::MutableBorrow => "&mut ",
                    OwnershipKind::Shared => "~",
                };
                write!(f, "{}{}", prefix, base_type)
            }
            Type::Error => write!(f, "<error>"),
        }
    }
}

/// Type variable for type inference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVariable {
    pub id: usize,
    pub constraints: Vec<Type>,
}

/// Type checker for AetherScript
#[derive(Debug, Clone)]
pub struct TypeChecker {
    /// Type environment mapping variable names to types
    type_env: HashMap<String, Type>,

    /// Type definitions mapping type names to their definitions
    type_definitions: HashMap<String, TypeDefinition>,

    /// Current module name for qualified type lookups
    current_module: Option<String>,

    /// Next type variable ID for inference
    next_type_var_id: usize,

    /// Type variable substitutions
    substitutions: HashMap<usize, Type>,
}

/// Enum variant information
#[derive(Debug, Clone)]
pub struct EnumVariantInfo {
    pub name: String,
    pub associated_type: Option<Type>,
    pub discriminant: usize, // Index of the variant in the enum definition
}

/// Enum type information  
#[derive(Debug, Clone)]
pub struct EnumTypeInfo {
    pub name: String,
    pub variants: Vec<EnumVariantInfo>,
    pub source_location: SourceLocation,
}

impl EnumTypeInfo {
    /// Get a variant by name
    pub fn get_variant(&self, name: &str) -> Option<&EnumVariantInfo> {
        self.variants.iter().find(|v| v.name == name)
    }
}

/// Type definition information
#[derive(Debug, Clone)]
pub enum TypeDefinition {
    /// Struct definition
    Struct {
        fields: Vec<(String, Type)>, // Changed from HashMap to preserve field order
        source_location: SourceLocation,
    },

    /// Enum definition
    Enum {
        variants: Vec<EnumVariantInfo>,
        source_location: SourceLocation,
    },

    /// Type alias
    Alias {
        target_type: Type,
        source_location: SourceLocation,
    },
}

impl TypeChecker {
    /// Create a new type checker
    pub fn new() -> Self {
        let mut checker = Self {
            type_env: HashMap::new(),
            type_definitions: HashMap::new(),
            current_module: None,
            next_type_var_id: 0,
            substitutions: HashMap::new(),
        };

        // Initialize built-in types
        checker.initialize_builtin_types();
        checker
    }

    /// Initialize built-in primitive types
    fn initialize_builtin_types(&mut self) {
        // Built-in types are handled by the Type::Primitive enum
        // No additional initialization needed for now
    }

    /// Set the current module for type checking
    pub fn set_current_module(&mut self, module_name: Option<String>) {
        self.current_module = module_name;
    }

    /// Add a variable to the type environment
    pub fn add_variable(&mut self, name: String, var_type: Type) {
        self.type_env.insert(name, var_type);
    }

    /// Look up a variable's type
    pub fn lookup_variable(&self, name: &str) -> Option<&Type> {
        self.type_env.get(name)
    }

    /// Add a type definition
    pub fn add_type_definition(&mut self, name: String, definition: TypeDefinition) {
        eprintln!("TypeChecker: Adding type definition '{}'", name);
        self.type_definitions.insert(name, definition);
        eprintln!(
            "TypeChecker: Now have {} type definitions",
            self.type_definitions.len()
        );
    }

    /// Look up a type definition
    pub fn lookup_type_definition(&self, name: &str) -> Option<&TypeDefinition> {
        self.type_definitions.get(name)
    }

    /// Convert an AST TypeConstraint to a TypeConstraintInfo
    pub fn ast_constraint_to_constraint(
        &self,
        constraint: &TypeConstraint,
    ) -> Result<TypeConstraintInfo, SemanticError> {
        match &constraint.constraint_type {
            TypeConstraintKind::TraitBound { trait_name } => Ok(TypeConstraintInfo::TraitBound {
                trait_name: trait_name.name.clone(),
                module: self.current_module.clone(),
            }),
            TypeConstraintKind::SubtypeBound { parent_type } => {
                let parent = self.ast_type_to_type(parent_type)?;
                Ok(TypeConstraintInfo::SubtypeBound {
                    parent_type: parent,
                })
            }
            TypeConstraintKind::NumericBound => Ok(TypeConstraintInfo::NumericBound),
            TypeConstraintKind::EqualityBound => Ok(TypeConstraintInfo::EqualityBound),
            TypeConstraintKind::OrderBound => Ok(TypeConstraintInfo::OrderBound),
            TypeConstraintKind::SizeBound { size_expr: _ } => {
                // For now, we'll just use a default size. In a full implementation,
                // we'd evaluate the size expression
                Ok(TypeConstraintInfo::SizeBound { size: 8 })
            }
            TypeConstraintKind::CustomBound { constraint_expr: _ } => {
                // Custom bounds are not implemented yet
                Err(SemanticError::UnsupportedFeature {
                    feature: "Custom type constraints".to_string(),
                    location: constraint.source_location.clone(),
                })
            }
        }
    }

    /// Convert an AST TypeSpecifier to a Type
    pub fn ast_type_to_type(&self, type_spec: &TypeSpecifier) -> Result<Type, SemanticError> {
        match type_spec {
            TypeSpecifier::Primitive { type_name, .. } => Ok(Type::Primitive(*type_name)),
            TypeSpecifier::Named {
                name,
                source_location,
            } => {
                // Check if this is a known type
                eprintln!(
                    "TypeChecker: Looking for type '{}', have {} types",
                    name.name,
                    self.type_definitions.len()
                );
                for (key, _) in &self.type_definitions {
                    eprintln!("  - Type: '{}'", key);
                }
                if self.type_definitions.contains_key(&name.name) {
                    Ok(Type::named(name.name.clone(), self.current_module.clone()))
                } else {
                    Err(SemanticError::UndefinedSymbol {
                        symbol: name.name.clone(),
                        location: source_location.clone(),
                    })
                }
            }
            TypeSpecifier::Generic {
                base_type,
                type_arguments,
                source_location,
            } => {
                // Convert type arguments
                let mut args = Vec::new();
                for arg in type_arguments {
                    args.push(self.ast_type_to_type(arg)?);
                }

                // Check if the base type exists
                if self.type_definitions.contains_key(&base_type.name) {
                    Ok(Type::generic_instance(
                        base_type.name.clone(),
                        args,
                        self.current_module.clone(),
                    ))
                } else {
                    Err(SemanticError::UndefinedSymbol {
                        symbol: base_type.name.clone(),
                        location: source_location.clone(),
                    })
                }
            }
            TypeSpecifier::TypeParameter {
                name, constraints, ..
            } => {
                // Convert constraints
                let mut constraint_infos = Vec::new();
                for constraint in constraints {
                    constraint_infos.push(self.ast_constraint_to_constraint(constraint)?);
                }

                Ok(Type::generic(name.name.clone(), constraint_infos))
            }
            TypeSpecifier::Array { element_type, .. } => {
                let elem_type = self.ast_type_to_type(element_type)?;

                // For now, ignore size expressions - they would need expression evaluation
                let array_size = None; // TODO: Evaluate size expression

                Ok(Type::array(elem_type, array_size))
            }
            TypeSpecifier::Map {
                key_type,
                value_type,
                ..
            } => {
                let key = self.ast_type_to_type(key_type)?;
                let value = self.ast_type_to_type(value_type)?;
                Ok(Type::map(key, value))
            }
            TypeSpecifier::Pointer {
                target_type,
                is_mutable,
                ..
            } => {
                let target = self.ast_type_to_type(target_type)?;
                Ok(Type::pointer(target, *is_mutable))
            }
            TypeSpecifier::Function {
                parameter_types,
                return_type,
                ..
            } => {
                let params: Result<Vec<Type>, SemanticError> = parameter_types
                    .iter()
                    .map(|p| self.ast_type_to_type(p))
                    .collect();

                let params = params?;
                let ret_type = self.ast_type_to_type(return_type)?;

                Ok(Type::function(params, ret_type))
            }
            TypeSpecifier::Owned {
                ownership,
                base_type,
                ..
            } => {
                // Convert the base type
                let base = self.ast_type_to_type(base_type)?;

                // Convert ownership annotation to ownership kind
                let ownership_kind = match ownership {
                    crate::ast::OwnershipKind::Owned => OwnershipKind::Owned,
                    crate::ast::OwnershipKind::Borrowed => OwnershipKind::Borrowed,
                    crate::ast::OwnershipKind::BorrowedMut => OwnershipKind::MutableBorrow,
                    crate::ast::OwnershipKind::Shared => OwnershipKind::Shared,
                };

                // Create the owned type with proper ownership semantics
                Ok(Type::Owned {
                    ownership: ownership_kind,
                    base_type: Box::new(base),
                })
            }
        }
    }

    /// Check if two types are compatible (can be assigned/compared)
    pub fn types_compatible(&self, type1: &Type, type2: &Type) -> bool {
        match (type1, type2) {
            // Same types are always compatible
            (a, b) if a == b => true,

            // Numeric type promotions
            (Type::Primitive(PrimitiveType::Integer), Type::Primitive(PrimitiveType::Float))
            | (Type::Primitive(PrimitiveType::Float), Type::Primitive(PrimitiveType::Integer)) => {
                true
            }

            // Integer size coercions (bidirectional for assignment compatibility)
            (
                Type::Primitive(PrimitiveType::Integer32),
                Type::Primitive(PrimitiveType::Integer64),
            )
            | (
                Type::Primitive(PrimitiveType::Integer64),
                Type::Primitive(PrimitiveType::Integer32),
            )
            | (
                Type::Primitive(PrimitiveType::Integer),
                Type::Primitive(PrimitiveType::Integer32),
            )
            | (
                Type::Primitive(PrimitiveType::Integer32),
                Type::Primitive(PrimitiveType::Integer),
            )
            | (
                Type::Primitive(PrimitiveType::Integer),
                Type::Primitive(PrimitiveType::Integer64),
            )
            | (
                Type::Primitive(PrimitiveType::Integer64),
                Type::Primitive(PrimitiveType::Integer),
            ) => true,

            // Float size coercions (bidirectional for assignment compatibility)
            (Type::Primitive(PrimitiveType::Float32), Type::Primitive(PrimitiveType::Float64))
            | (Type::Primitive(PrimitiveType::Float64), Type::Primitive(PrimitiveType::Float32))
            | (Type::Primitive(PrimitiveType::Float), Type::Primitive(PrimitiveType::Float32))
            | (Type::Primitive(PrimitiveType::Float32), Type::Primitive(PrimitiveType::Float))
            | (Type::Primitive(PrimitiveType::Float), Type::Primitive(PrimitiveType::Float64))
            | (Type::Primitive(PrimitiveType::Float64), Type::Primitive(PrimitiveType::Float)) => {
                true
            }

            // Pointer compatibility (with const/mut differences)
            (
                Type::Pointer {
                    target_type: t1,
                    is_mutable: false,
                },
                Type::Pointer {
                    target_type: t2,
                    is_mutable: _,
                },
            ) if t1 == t2 => true,

            // Array compatibility (dynamic vs sized arrays)
            (
                Type::Array {
                    element_type: e1,
                    size: None,
                },
                Type::Array {
                    element_type: e2,
                    size: Some(_),
                },
            ) if e1 == e2 => true,
            (
                Type::Array {
                    element_type: e1,
                    size: Some(_),
                },
                Type::Array {
                    element_type: e2,
                    size: None,
                },
            ) if e1 == e2 => true,

            // Owned type compatibility
            (
                Type::Owned {
                    ownership: o1,
                    base_type: b1,
                },
                Type::Owned {
                    ownership: o2,
                    base_type: b2,
                },
            ) => {
                match (o1, o2) {
                    // Same ownership kind with compatible base types
                    (a, b) if a == b => self.types_compatible(b1, b2),
                    // Mutable borrow can be used where immutable borrow is expected
                    (OwnershipKind::MutableBorrow, OwnershipKind::Borrowed) => {
                        self.types_compatible(b1, b2)
                    }
                    // Owned can be temporarily borrowed
                    (OwnershipKind::Owned, OwnershipKind::Borrowed) => {
                        self.types_compatible(b1, b2)
                    }
                    (OwnershipKind::Owned, OwnershipKind::MutableBorrow) => {
                        self.types_compatible(b1, b2)
                    }
                    // Shared can be borrowed immutably
                    (OwnershipKind::Shared, OwnershipKind::Borrowed) => {
                        self.types_compatible(b1, b2)
                    }
                    _ => false,
                }
            }

            // Owned type with base type (implicit ownership)
            (Type::Owned { base_type, .. }, other) => self.types_compatible(base_type, other),
            (other, Type::Owned { base_type, .. }) => self.types_compatible(other, base_type),

            _ => false,
        }
    }

    /// Generate a fresh type variable
    pub fn fresh_type_var(&mut self) -> Type {
        let id = self.next_type_var_id;
        self.next_type_var_id += 1;
        Type::Variable(TypeVariable {
            id,
            constraints: Vec::new(),
        })
    }

    /// Check if two types are equal
    pub fn are_types_equal(&self, type1: &Type, type2: &Type) -> bool {
        // TODO: Implement proper type equality checking
        match (type1, type2) {
            (Type::Primitive(p1), Type::Primitive(p2)) => p1 == p2,
            (
                Type::Named {
                    name: n1,
                    module: m1,
                },
                Type::Named {
                    name: n2,
                    module: m2,
                },
            ) => n1 == n2 && m1 == m2,
            (
                Type::Pointer {
                    target_type: p1,
                    is_mutable: m1,
                },
                Type::Pointer {
                    target_type: p2,
                    is_mutable: m2,
                },
            ) => m1 == m2 && self.are_types_equal(p1, p2),
            (
                Type::Array {
                    element_type: e1,
                    size: s1,
                },
                Type::Array {
                    element_type: e2,
                    size: s2,
                },
            ) => s1 == s2 && self.are_types_equal(e1, e2),
            (
                Type::Function {
                    parameter_types: p1,
                    return_type: r1,
                },
                Type::Function {
                    parameter_types: p2,
                    return_type: r2,
                },
            ) => {
                if p1.len() != p2.len() {
                    return false;
                }
                for (t1, t2) in p1.iter().zip(p2.iter()) {
                    if !self.are_types_equal(t1, t2) {
                        return false;
                    }
                }
                self.are_types_equal(r1, r2)
            }
            (
                Type::Owned {
                    ownership: o1,
                    base_type: b1,
                },
                Type::Owned {
                    ownership: o2,
                    base_type: b2,
                },
            ) => o1 == o2 && self.are_types_equal(b1, b2),
            (Type::Error, _) | (_, Type::Error) => true, // Error type is equal to anything
            _ => false,
        }
    }

    /// Check if a type is an enum type
    pub fn is_enum_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Named { name, .. } => {
                matches!(
                    self.type_definitions.get(name),
                    Some(TypeDefinition::Enum { .. })
                )
            }
            _ => false,
        }
    }

    /// Find an enum type by variant name
    pub fn find_enum_type_by_variant(
        &self,
        variant_name: &str,
        _module: &str,
    ) -> Option<EnumTypeInfo> {
        for (type_name, definition) in &self.type_definitions {
            if let TypeDefinition::Enum {
                variants,
                source_location,
            } = definition
            {
                if variants.iter().any(|v| v.name == variant_name) {
                    return Some(EnumTypeInfo {
                        name: type_name.clone(),
                        variants: variants.clone(),
                        source_location: source_location.clone(),
                    });
                }
            }
        }
        None
    }

    /// Check type compatibility (for assignments, etc.)
    pub fn check_type_compatibility(
        &self,
        expected: &Type,
        actual: &Type,
        _location: &SourceLocation,
    ) -> Result<(), SemanticError> {
        if self.are_types_equal(expected, actual) {
            Ok(())
        } else {
            Err(SemanticError::TypeMismatch {
                expected: expected.to_string(),
                found: actual.to_string(),
                location: SourceLocation::unknown(), // TODO: Use the provided location
            })
        }
    }

    /// Apply substitutions to a type
    pub fn apply_substitutions(&self, type_to_subst: &Type) -> Type {
        match type_to_subst {
            Type::Variable(var) => {
                if let Some(substituted) = self.substitutions.get(&var.id) {
                    self.apply_substitutions(substituted)
                } else {
                    type_to_subst.clone()
                }
            }
            Type::Array { element_type, size } => Type::Array {
                element_type: Box::new(self.apply_substitutions(element_type)),
                size: *size,
            },
            Type::Map {
                key_type,
                value_type,
            } => Type::Map {
                key_type: Box::new(self.apply_substitutions(key_type)),
                value_type: Box::new(self.apply_substitutions(value_type)),
            },
            Type::Pointer {
                target_type,
                is_mutable,
            } => Type::Pointer {
                target_type: Box::new(self.apply_substitutions(target_type)),
                is_mutable: *is_mutable,
            },
            Type::Function {
                parameter_types,
                return_type,
            } => Type::Function {
                parameter_types: parameter_types
                    .iter()
                    .map(|t| self.apply_substitutions(t))
                    .collect(),
                return_type: Box::new(self.apply_substitutions(return_type)),
            },
            _ => type_to_subst.clone(),
        }
    }

    /// Check if a type satisfies the given constraints
    pub fn check_constraints(
        &self,
        type_to_check: &Type,
        constraints: &[TypeConstraintInfo],
    ) -> Result<(), SemanticError> {
        for constraint in constraints {
            match constraint {
                TypeConstraintInfo::NumericBound => {
                    if !type_to_check.is_numeric() {
                        return Err(SemanticError::ConstraintViolation {
                            constraint: "NumericBound".to_string(),
                            found_type: type_to_check.to_string(),
                            location: SourceLocation::unknown(),
                        });
                    }
                }
                TypeConstraintInfo::EqualityBound => {
                    // Most types support equality, but functions generally don't
                    if matches!(type_to_check, Type::Function { .. }) {
                        return Err(SemanticError::ConstraintViolation {
                            constraint: "EqualityBound".to_string(),
                            found_type: type_to_check.to_string(),
                            location: SourceLocation::unknown(),
                        });
                    }
                }
                TypeConstraintInfo::OrderBound => {
                    // Only numeric types and strings support ordering
                    if !type_to_check.is_numeric()
                        && !matches!(
                            type_to_check,
                            Type::Primitive(crate::ast::PrimitiveType::String)
                        )
                    {
                        return Err(SemanticError::ConstraintViolation {
                            constraint: "OrderBound".to_string(),
                            found_type: type_to_check.to_string(),
                            location: SourceLocation::unknown(),
                        });
                    }
                }
                TypeConstraintInfo::SizeBound { size } => {
                    if let Some(actual_size) = type_to_check.size_bytes() {
                        if actual_size != *size {
                            return Err(SemanticError::ConstraintViolation {
                                constraint: format!("SizeBound({})", size),
                                found_type: format!("{} (size: {})", type_to_check, actual_size),
                                location: SourceLocation::unknown(),
                            });
                        }
                    } else {
                        return Err(SemanticError::ConstraintViolation {
                            constraint: format!("SizeBound({})", size),
                            found_type: format!("{} (unknown size)", type_to_check),
                            location: SourceLocation::unknown(),
                        });
                    }
                }
                TypeConstraintInfo::TraitBound { .. } => {
                    // For now, we don't have a trait system, so we'll just accept all trait bounds
                    // In a full implementation, this would check if the type implements the trait
                    // TODO: Implement proper trait checking
                }
                TypeConstraintInfo::SubtypeBound { parent_type } => {
                    // Check if type_to_check is compatible with parent_type
                    if !self.types_compatible(type_to_check, parent_type) {
                        return Err(SemanticError::ConstraintViolation {
                            constraint: format!("SubtypeBound({})", parent_type),
                            found_type: type_to_check.to_string(),
                            location: SourceLocation::unknown(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Instantiate a generic type with concrete type arguments
    pub fn instantiate_generic(
        &self,
        base_type: &str,
        type_arguments: &[Type],
        constraints: &[Vec<TypeConstraintInfo>],
    ) -> Result<Type, SemanticError> {
        // Check that we have the right number of type arguments
        if type_arguments.len() != constraints.len() {
            return Err(SemanticError::GenericInstantiationError {
                base_type: base_type.to_string(),
                expected_args: constraints.len(),
                found_args: type_arguments.len(),
                location: SourceLocation::unknown(),
            });
        }

        // Check that each type argument satisfies its constraints
        for (i, (arg_type, arg_constraints)) in
            type_arguments.iter().zip(constraints.iter()).enumerate()
        {
            if let Err(mut error) = self.check_constraints(arg_type, arg_constraints) {
                // Add context about which type parameter failed
                if let SemanticError::ConstraintViolation {
                    constraint,
                    found_type: _,
                    location: _,
                } = &mut error
                {
                    *constraint = format!("Type parameter {} requires {}", i, constraint);
                }
                return Err(error);
            }
        }

        // Create the instantiated generic type
        Ok(Type::generic_instance(
            base_type.to_string(),
            type_arguments.to_vec(),
            self.current_module.clone(),
        ))
    }

    /// Unify two types (for type inference)
    pub fn unify(&mut self, type1: &Type, type2: &Type) -> Result<(), SemanticError> {
        let t1 = self.apply_substitutions(type1);
        let t2 = self.apply_substitutions(type2);

        match (&t1, &t2) {
            // Same types unify
            (a, b) if a == b => Ok(()),

            // Type variables unify with anything
            (Type::Variable(var), other) | (other, Type::Variable(var)) => {
                self.substitutions.insert(var.id, other.clone());
                Ok(())
            }

            // Compatible types unify
            (a, b) if self.types_compatible(a, b) => Ok(()),

            // Recursive unification for compound types
            (
                Type::Array {
                    element_type: e1,
                    size: s1,
                },
                Type::Array {
                    element_type: e2,
                    size: s2,
                },
            ) if s1 == s2 => self.unify(e1, e2),

            (
                Type::Map {
                    key_type: k1,
                    value_type: v1,
                },
                Type::Map {
                    key_type: k2,
                    value_type: v2,
                },
            ) => {
                self.unify(k1, k2)?;
                self.unify(v1, v2)
            }

            (
                Type::Pointer {
                    target_type: t1,
                    is_mutable: m1,
                },
                Type::Pointer {
                    target_type: t2,
                    is_mutable: m2,
                },
            ) if m1 == m2 => self.unify(t1, t2),

            (
                Type::Function {
                    parameter_types: p1,
                    return_type: r1,
                },
                Type::Function {
                    parameter_types: p2,
                    return_type: r2,
                },
            ) if p1.len() == p2.len() => {
                for (param1, param2) in p1.iter().zip(p2.iter()) {
                    self.unify(param1, param2)?;
                }
                self.unify(r1, r2)
            }

            // Generic type unification
            (
                Type::GenericInstance {
                    base_type: b1,
                    type_arguments: args1,
                    module: m1,
                },
                Type::GenericInstance {
                    base_type: b2,
                    type_arguments: args2,
                    module: m2,
                },
            ) if b1 == b2 && m1 == m2 && args1.len() == args2.len() => {
                for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                    self.unify(arg1, arg2)?;
                }
                Ok(())
            }

            // Owned type unification
            (
                Type::Owned {
                    ownership: o1,
                    base_type: b1,
                },
                Type::Owned {
                    ownership: o2,
                    base_type: b2,
                },
            ) if o1 == o2 => self.unify(b1, b2),

            _ => Err(SemanticError::TypeMismatch {
                expected: t1.to_string(),
                found: t2.to_string(),
                location: SourceLocation::unknown(), // TODO: Better location tracking
            }),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        let int_type = Type::primitive(PrimitiveType::Integer);
        let float_type = Type::primitive(PrimitiveType::Float);
        let string_type = Type::primitive(PrimitiveType::String);

        assert!(int_type.is_numeric());
        assert!(int_type.is_integer());
        assert!(!int_type.is_float());

        assert!(float_type.is_numeric());
        assert!(!float_type.is_integer());
        assert!(float_type.is_float());

        assert!(!string_type.is_numeric());
        assert!(!string_type.is_integer());
        assert!(!string_type.is_float());
    }

    #[test]
    fn test_compound_types() {
        let int_type = Type::primitive(PrimitiveType::Integer);
        let array_type = Type::array(int_type.clone(), Some(10));
        let _map_type = Type::map(Type::primitive(PrimitiveType::String), int_type.clone());
        let pointer_type = Type::pointer(int_type, false);

        assert_eq!(array_type.size_bytes(), None); // Size depends on element type
        assert_eq!(pointer_type.size_bytes(), Some(8)); // 64-bit pointer
        assert!(pointer_type.is_pointer());
    }

    #[test]
    fn test_type_compatibility() {
        let checker = TypeChecker::new();

        let int_type = Type::primitive(PrimitiveType::Integer);
        let float_type = Type::primitive(PrimitiveType::Float);
        let string_type = Type::primitive(PrimitiveType::String);

        // Same types are compatible
        assert!(checker.types_compatible(&int_type, &int_type));

        // Numeric types are compatible
        assert!(checker.types_compatible(&int_type, &float_type));
        assert!(checker.types_compatible(&float_type, &int_type));

        // Different types are not compatible
        assert!(!checker.types_compatible(&int_type, &string_type));
    }

    #[test]
    fn test_type_unification() {
        let mut checker = TypeChecker::new();

        let int_type = Type::primitive(PrimitiveType::Integer);
        let var_type = checker.fresh_type_var();

        // Unify type variable with concrete type
        assert!(checker.unify(&var_type, &int_type).is_ok());

        // Check that substitution was applied
        let unified = checker.apply_substitutions(&var_type);
        assert_eq!(unified, int_type);
    }

    #[test]
    fn test_type_display() {
        let int_type = Type::primitive(PrimitiveType::Integer);
        let array_type = Type::array(int_type.clone(), Some(5));
        let map_type = Type::map(Type::primitive(PrimitiveType::String), int_type.clone());

        assert_eq!(int_type.to_string(), "Integer");
        assert_eq!(array_type.to_string(), "[Integer; 5]");
        assert_eq!(map_type.to_string(), "Map<String, Integer>");
    }

    #[test]
    fn test_generic_types() {
        let int_type = Type::primitive(PrimitiveType::Integer);
        let string_type = Type::primitive(PrimitiveType::String);

        // Test generic type parameter
        let generic_param = Type::generic(
            "T".to_string(),
            vec![
                TypeConstraintInfo::NumericBound,
                TypeConstraintInfo::EqualityBound,
            ],
        );

        // Test generic instance
        let list_int = Type::generic_instance("List".to_string(), vec![int_type.clone()], None);

        let map_string_int = Type::generic_instance(
            "Map".to_string(),
            vec![string_type.clone(), int_type.clone()],
            None,
        );

        assert_eq!(generic_param.to_string(), "T: NumericBound + EqualityBound");
        assert_eq!(list_int.to_string(), "List<Integer>");
        assert_eq!(map_string_int.to_string(), "Map<String, Integer>");
    }

    #[test]
    fn test_constraint_checking() {
        let checker = TypeChecker::new();
        let int_type = Type::primitive(PrimitiveType::Integer);
        let string_type = Type::primitive(PrimitiveType::String);
        let func_type = Type::function(vec![int_type.clone()], int_type.clone());

        // Test numeric constraint
        let numeric_constraints = vec![TypeConstraintInfo::NumericBound];
        assert!(checker
            .check_constraints(&int_type, &numeric_constraints)
            .is_ok());
        assert!(checker
            .check_constraints(&string_type, &numeric_constraints)
            .is_err());

        // Test equality constraint
        let equality_constraints = vec![TypeConstraintInfo::EqualityBound];
        assert!(checker
            .check_constraints(&int_type, &equality_constraints)
            .is_ok());
        assert!(checker
            .check_constraints(&string_type, &equality_constraints)
            .is_ok());
        assert!(checker
            .check_constraints(&func_type, &equality_constraints)
            .is_err());

        // Test order constraint
        let order_constraints = vec![TypeConstraintInfo::OrderBound];
        assert!(checker
            .check_constraints(&int_type, &order_constraints)
            .is_ok());
        assert!(checker
            .check_constraints(&string_type, &order_constraints)
            .is_ok());
        assert!(checker
            .check_constraints(&func_type, &order_constraints)
            .is_err());
    }

    #[test]
    fn test_generic_instantiation() {
        let checker = TypeChecker::new();
        let int_type = Type::primitive(PrimitiveType::Integer);
        let string_type = Type::primitive(PrimitiveType::String);

        // Test successful instantiation
        let numeric_constraints = vec![vec![TypeConstraintInfo::NumericBound]];
        let result = checker.instantiate_generic("List", &[int_type.clone()], &numeric_constraints);
        assert!(result.is_ok());

        // Test constraint violation
        let result =
            checker.instantiate_generic("List", &[string_type.clone()], &numeric_constraints);
        assert!(result.is_err());

        // Test wrong number of arguments
        let result = checker.instantiate_generic(
            "List",
            &[int_type.clone(), string_type.clone()],
            &numeric_constraints,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_generic_unification() {
        let mut checker = TypeChecker::new();
        let int_type = Type::primitive(PrimitiveType::Integer);
        let string_type = Type::primitive(PrimitiveType::String);

        let list_int1 = Type::generic_instance("List".to_string(), vec![int_type.clone()], None);
        let list_int2 = Type::generic_instance("List".to_string(), vec![int_type.clone()], None);
        let list_string =
            Type::generic_instance("List".to_string(), vec![string_type.clone()], None);

        // Same generic instances should unify
        assert!(checker.unify(&list_int1, &list_int2).is_ok());

        // Different generic instances should not unify
        assert!(checker.unify(&list_int1, &list_string).is_err());
    }

    #[test]
    fn test_ownership_types() {
        let checker = TypeChecker::new();

        let int_type = Type::primitive(PrimitiveType::Integer);
        let owned_int = Type::owned(int_type.clone());
        let borrowed_int = Type::borrowed(int_type.clone());
        let mut_borrowed_int = Type::mutable_borrow(int_type.clone());
        let shared_int = Type::shared(int_type.clone());

        // Test ownership kind extraction
        assert_eq!(owned_int.ownership_kind(), Some(OwnershipKind::Owned));
        assert_eq!(borrowed_int.ownership_kind(), Some(OwnershipKind::Borrowed));
        assert_eq!(
            mut_borrowed_int.ownership_kind(),
            Some(OwnershipKind::MutableBorrow)
        );
        assert_eq!(shared_int.ownership_kind(), Some(OwnershipKind::Shared));
        assert_eq!(int_type.ownership_kind(), None);

        // Test base type extraction
        assert_eq!(owned_int.base_type(), &int_type);
        assert_eq!(int_type.base_type(), &int_type);

        // Test ownership checks
        assert!(owned_int.is_owned());
        assert!(!borrowed_int.is_owned());
        assert!(borrowed_int.is_borrowed());
        assert!(mut_borrowed_int.is_borrowed());
        assert!(!shared_int.is_borrowed());
    }

    #[test]
    fn test_ownership_compatibility() {
        let checker = TypeChecker::new();

        let int_type = Type::primitive(PrimitiveType::Integer);
        let owned_int = Type::owned(int_type.clone());
        let borrowed_int = Type::borrowed(int_type.clone());
        let mut_borrowed_int = Type::mutable_borrow(int_type.clone());
        let shared_int = Type::shared(int_type.clone());

        // Owned can be borrowed
        assert!(checker.types_compatible(&owned_int, &borrowed_int));
        assert!(checker.types_compatible(&owned_int, &mut_borrowed_int));

        // Mutable borrow can be used as immutable borrow
        assert!(checker.types_compatible(&mut_borrowed_int, &borrowed_int));

        // Shared can be borrowed immutably
        assert!(checker.types_compatible(&shared_int, &borrowed_int));

        // But not the other way around
        assert!(!checker.types_compatible(&borrowed_int, &owned_int));
        assert!(!checker.types_compatible(&borrowed_int, &mut_borrowed_int));

        // Different ownership kinds don't unify unless compatible
        assert!(!checker.types_compatible(&owned_int, &shared_int));
    }

    #[test]
    fn test_ownership_display() {
        let int_type = Type::primitive(PrimitiveType::Integer);
        let owned_int = Type::owned(int_type.clone());
        let borrowed_int = Type::borrowed(int_type.clone());
        let mut_borrowed_int = Type::mutable_borrow(int_type.clone());
        let shared_int = Type::shared(int_type.clone());

        assert_eq!(owned_int.to_string(), "^Integer");
        assert_eq!(borrowed_int.to_string(), "&Integer");
        assert_eq!(mut_borrowed_int.to_string(), "&mut Integer");
        assert_eq!(shared_int.to_string(), "~Integer");
    }

    #[test]
    fn test_ast_to_type_ownership() {
        use crate::ast::{Identifier, TypeSpecifier};
        use crate::error::SourceLocation;

        let checker = TypeChecker::new();
        let int_spec = TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: SourceLocation::unknown(),
        };

        // Test owned type conversion
        let owned_spec = TypeSpecifier::Owned {
            ownership: crate::ast::OwnershipKind::Owned,
            base_type: Box::new(int_spec.clone()),
            source_location: SourceLocation::unknown(),
        };

        let owned_type = checker.ast_type_to_type(&owned_spec).unwrap();
        assert!(owned_type.is_owned());
        assert_eq!(owned_type.ownership_kind(), Some(OwnershipKind::Owned));

        // Test borrowed type conversion
        let borrowed_spec = TypeSpecifier::Owned {
            ownership: crate::ast::OwnershipKind::Borrowed,
            base_type: Box::new(int_spec.clone()),
            source_location: SourceLocation::unknown(),
        };

        let borrowed_type = checker.ast_type_to_type(&borrowed_spec).unwrap();
        assert!(borrowed_type.is_borrowed());
        assert_eq!(
            borrowed_type.ownership_kind(),
            Some(OwnershipKind::Borrowed)
        );
    }
}
