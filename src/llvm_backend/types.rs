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

//! Type conversion between MIR types and LLVM types

use crate::types::Type;
use crate::ast::PrimitiveType;
use crate::error::SemanticError;
use inkwell::context::Context;
use inkwell::types::{BasicTypeEnum, IntType, FloatType, PointerType, VoidType, BasicType, BasicMetadataTypeEnum};
use inkwell::AddressSpace;
use std::collections::HashMap;

/// Converts MIR types to LLVM types
pub struct TypeConverter<'ctx> {
    context: &'ctx Context,
    type_cache: HashMap<String, BasicTypeEnum<'ctx>>,
}

impl<'ctx> TypeConverter<'ctx> {
    /// Create a new type converter
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            context,
            type_cache: HashMap::new(),
        }
    }
    
    /// Convert a MIR type to an LLVM type
    pub fn convert_type(&mut self, mir_type: &Type) -> Result<BasicTypeEnum<'ctx>, SemanticError> {
        match mir_type {
            Type::Primitive(prim_type) => self.convert_primitive_type(*prim_type),
            
            Type::Named { name, module } => {
                let full_name = match module {
                    Some(module) => format!("{}::{}", module, name),
                    None => name.clone(),
                };
                
                // Check cache first
                if let Some(cached_type) = self.type_cache.get(&full_name) {
                    return Ok(*cached_type);
                }
                
                // Named types will be resolved by the backend when it has access to type definitions
                // For now, create an opaque struct that will be defined later
                let struct_type = self.context.opaque_struct_type(&full_name);
                let basic_type = BasicTypeEnum::StructType(struct_type);
                self.type_cache.insert(full_name, basic_type);
                Ok(basic_type)
            }
            
            Type::Array { element_type, size } => {
                let element_llvm_type = self.convert_type(element_type)?;
                
                match size {
                    Some(array_size) => {
                        let array_type = element_llvm_type.array_type(*array_size as u32);
                        Ok(BasicTypeEnum::ArrayType(array_type))
                    }
                    None => {
                        // Dynamic array - use a pointer for now
                        let ptr_type = element_llvm_type.ptr_type(AddressSpace::default());
                        Ok(BasicTypeEnum::PointerType(ptr_type))
                    }
                }
            }
            
            Type::Map { key_type, value_type } => {
                // For now, represent maps as opaque structs
                // In a full implementation, this would be a hash table or similar structure
                let _key_llvm_type = self.convert_type(key_type)?;
                let _value_llvm_type = self.convert_type(value_type)?;
                
                let map_name = format!("Map<{:?}, {:?}>", key_type, value_type);
                let struct_type = self.context.opaque_struct_type(&map_name);
                Ok(BasicTypeEnum::StructType(struct_type))
            }
            
            Type::Pointer { target_type, .. } => {
                let target_llvm_type = self.convert_type(target_type)?;
                let ptr_type = target_llvm_type.ptr_type(AddressSpace::default());
                Ok(BasicTypeEnum::PointerType(ptr_type))
            }
            
            Type::Function { parameter_types, return_type } => {
                let param_llvm_types: Result<Vec<_>, _> = parameter_types.iter()
                    .map(|param_type| self.convert_type(param_type))
                    .collect();
                let param_llvm_types = param_llvm_types?;
                
                let _return_llvm_type = self.convert_type(return_type)?;
                
                // Function types are represented as function pointers
                let param_meta_types: Vec<BasicMetadataTypeEnum> = param_llvm_types.into_iter()
                    .map(|t| t.into())
                    .collect();
                let fn_type = self.context.void_type().fn_type(&param_meta_types, false);
                
                let fn_ptr_type = fn_type.ptr_type(AddressSpace::default());
                Ok(BasicTypeEnum::PointerType(fn_ptr_type))
            }
            
            Type::Generic { name, .. } => {
                // Generics should be resolved by this point
                Err(SemanticError::InvalidType {
                    type_name: name.clone(),
                    reason: "Unresolved generic type in code generation".to_string(),
                    location: crate::error::SourceLocation::unknown(),
                })
            }
            
            Type::GenericInstance { base_type, type_arguments, module } => {
                // For now, treat as opaque struct
                let full_name = match module {
                    Some(module) => format!("{}::{}<{}>", module, base_type, type_arguments.len()),
                    None => format!("{}<{}>", base_type, type_arguments.len()),
                };
                
                let struct_type = self.context.opaque_struct_type(&full_name);
                Ok(BasicTypeEnum::StructType(struct_type))
            }
            
            Type::Variable(_) => {
                Err(SemanticError::InvalidType {
                    type_name: "type_variable".to_string(),
                    reason: "Unresolved type variable in code generation".to_string(),
                    location: crate::error::SourceLocation::unknown(),
                })
            }
            
            Type::Error => {
                Err(SemanticError::InvalidType {
                    type_name: "error".to_string(),
                    reason: "Error type in code generation".to_string(),
                    location: crate::error::SourceLocation::unknown(),
                })
            }
            
            Type::Owned { base_type, .. } => {
                // Owned types have the same representation as their base type
                // The ownership is tracked at compile time, not runtime
                self.convert_type(base_type)
            }
        }
    }
    
    /// Convert a primitive type to an LLVM type
    fn convert_primitive_type(&self, prim_type: PrimitiveType) -> Result<BasicTypeEnum<'ctx>, SemanticError> {
        let llvm_type = match prim_type {
            PrimitiveType::Integer => BasicTypeEnum::IntType(self.context.i64_type()),
            PrimitiveType::Integer32 => BasicTypeEnum::IntType(self.context.i32_type()),
            PrimitiveType::Integer64 => BasicTypeEnum::IntType(self.context.i64_type()),
            PrimitiveType::Float => BasicTypeEnum::FloatType(self.context.f64_type()),
            PrimitiveType::Float32 => BasicTypeEnum::FloatType(self.context.f32_type()),
            PrimitiveType::Float64 => BasicTypeEnum::FloatType(self.context.f64_type()),
            PrimitiveType::Boolean => BasicTypeEnum::IntType(self.context.bool_type()),
            PrimitiveType::String => {
                // String as a pointer to i8 (C-style string for now)
                BasicTypeEnum::PointerType(self.context.i8_type().ptr_type(AddressSpace::default()))
            }
            PrimitiveType::Char => {
                // Char as i8 (C-style char)
                BasicTypeEnum::IntType(self.context.i8_type())
            }
            PrimitiveType::Void => {
                // Void type can't be used as BasicTypeEnum, we'll handle this specially
                return Err(SemanticError::InvalidType {
                    type_name: "void".to_string(),
                    reason: "Void type cannot be used as a basic type".to_string(),
                    location: crate::error::SourceLocation::unknown(),
                });
            }
            PrimitiveType::SizeT => {
                // Size type - use pointer-sized integer
                #[cfg(target_pointer_width = "64")]
                return Ok(BasicTypeEnum::IntType(self.context.i64_type()));
                
                #[cfg(target_pointer_width = "32")]
                return Ok(BasicTypeEnum::IntType(self.context.i32_type()));
            }
            PrimitiveType::UIntPtrT => {
                // Unsigned pointer-sized integer
                #[cfg(target_pointer_width = "64")]
                return Ok(BasicTypeEnum::IntType(self.context.i64_type()));
                
                #[cfg(target_pointer_width = "32")]
                return Ok(BasicTypeEnum::IntType(self.context.i32_type()));
            }
        };
        
        Ok(llvm_type)
    }
    
    /// Get an i8 pointer type (useful for generic pointers)
    pub fn i8_ptr_type(&self) -> PointerType<'ctx> {
        self.context.i8_type().ptr_type(AddressSpace::default())
    }
    
    /// Get a void type
    pub fn void_type(&self) -> VoidType<'ctx> {
        self.context.void_type()
    }
    
    /// Get an integer type of specified width
    pub fn int_type(&self, width: u32) -> IntType<'ctx> {
        self.context.custom_width_int_type(width)
    }
    
    /// Get a float type of specified precision
    pub fn float_type(&self, double_precision: bool) -> FloatType<'ctx> {
        if double_precision {
            self.context.f64_type()
        } else {
            self.context.f32_type()
        }
    }
    
    /// Define a struct type with its fields
    pub fn define_struct_type(&mut self, name: &str, fields: &[(String, Type)]) -> Result<inkwell::types::StructType<'ctx>, SemanticError> {
        // First, get or create the opaque struct
        let struct_type = if let Some(BasicTypeEnum::StructType(st)) = self.type_cache.get(name) {
            *st
        } else {
            self.context.opaque_struct_type(name)
        };
        
        // Convert field types
        let mut field_types = Vec::new();
        for (_, field_type) in fields {
            let llvm_field_type = self.convert_type(field_type)?;
            field_types.push(llvm_field_type);
        }
        
        // Set the body of the struct
        struct_type.set_body(&field_types, false);
        
        // Update cache
        self.type_cache.insert(name.to_string(), BasicTypeEnum::StructType(struct_type));
        
        Ok(struct_type)
    }
    
    /// Get a struct type by name (must be already defined)
    pub fn get_struct_type(&self, name: &str) -> Option<inkwell::types::StructType<'ctx>> {
        match self.type_cache.get(name) {
            Some(BasicTypeEnum::StructType(st)) => Some(*st),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;
    
    #[test]
    fn test_primitive_type_conversion() {
        let context = Context::create();
        let mut converter = TypeConverter::new(&context);
        
        // Test integer types
        let int_type = converter.convert_primitive_type(PrimitiveType::Integer).unwrap();
        assert!(matches!(int_type, BasicTypeEnum::IntType(_)));
        
        let int32_type = converter.convert_primitive_type(PrimitiveType::Integer32).unwrap();
        assert!(matches!(int32_type, BasicTypeEnum::IntType(_)));
        
        // Test float types
        let float_type = converter.convert_primitive_type(PrimitiveType::Float).unwrap();
        assert!(matches!(float_type, BasicTypeEnum::FloatType(_)));
        
        // Test boolean type
        let bool_type = converter.convert_primitive_type(PrimitiveType::Boolean).unwrap();
        assert!(matches!(bool_type, BasicTypeEnum::IntType(_)));
        
        // Test string type
        let string_type = converter.convert_primitive_type(PrimitiveType::String).unwrap();
        assert!(matches!(string_type, BasicTypeEnum::PointerType(_)));
        
        // Note: Void type cannot be converted to BasicTypeEnum
        // It's handled specially in function types
    }
    
    #[test]
    fn test_array_type_conversion() {
        let context = Context::create();
        let mut converter = TypeConverter::new(&context);
        
        // Fixed-size array
        let element_type = Type::primitive(PrimitiveType::Integer);
        let array_type = Type::Array {
            element_type: Box::new(element_type),
            size: Some(10),
        };
        
        let llvm_type = converter.convert_type(&array_type).unwrap();
        assert!(matches!(llvm_type, BasicTypeEnum::ArrayType(_)));
        
        // Dynamic array
        let element_type = Type::primitive(PrimitiveType::Integer);
        let dynamic_array_type = Type::Array {
            element_type: Box::new(element_type),
            size: None,
        };
        
        let llvm_type = converter.convert_type(&dynamic_array_type).unwrap();
        assert!(matches!(llvm_type, BasicTypeEnum::PointerType(_)));
    }
    
    #[test]
    fn test_pointer_type_conversion() {
        let context = Context::create();
        let mut converter = TypeConverter::new(&context);
        
        let target_type = Type::primitive(PrimitiveType::Integer);
        let pointer_type = Type::Pointer {
            target_type: Box::new(target_type),
            is_mutable: true,
        };
        
        let llvm_type = converter.convert_type(&pointer_type).unwrap();
        assert!(matches!(llvm_type, BasicTypeEnum::PointerType(_)));
    }
    
    #[test]
    fn test_named_type_conversion() {
        let context = Context::create();
        let mut converter = TypeConverter::new(&context);
        
        let named_type = Type::Named {
            name: "MyStruct".to_string(),
            module: Some("mymodule".to_string()),
        };
        
        let llvm_type = converter.convert_type(&named_type).unwrap();
        assert!(matches!(llvm_type, BasicTypeEnum::StructType(_)));
    }
    
    #[test]
    fn test_function_type_conversion() {
        let context = Context::create();
        let mut converter = TypeConverter::new(&context);
        
        let function_type = Type::Function {
            parameter_types: vec![
                Type::primitive(PrimitiveType::Integer),
                Type::primitive(PrimitiveType::Float),
            ],
            return_type: Box::new(Type::primitive(PrimitiveType::Boolean)),
        };
        
        let llvm_type = converter.convert_type(&function_type).unwrap();
        assert!(matches!(llvm_type, BasicTypeEnum::PointerType(_)));
    }
}