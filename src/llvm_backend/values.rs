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

//! Value conversion between MIR values and LLVM values

use crate::error::SemanticError;
use crate::mir::ConstantValue;
use inkwell::context::Context;
use inkwell::values::{BasicValueEnum, FloatValue, IntValue, PointerValue};
use inkwell::AddressSpace;

/// Converts MIR values to LLVM values
pub struct ValueConverter<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> ValueConverter<'ctx> {
    /// Create a new value converter
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Convert a MIR constant value to an LLVM constant value
    pub fn convert_constant_value(
        &self,
        value: &ConstantValue,
    ) -> Result<BasicValueEnum<'ctx>, SemanticError> {
        let llvm_value = match value {
            ConstantValue::Bool(b) => {
                let bool_val = self.context.bool_type().const_int(*b as u64, false);
                BasicValueEnum::IntValue(bool_val)
            }

            ConstantValue::Integer(i) => {
                // Use i64 for all integers by default
                let int_val = self.context.i64_type().const_int(*i as u64, true);
                BasicValueEnum::IntValue(int_val)
            }

            ConstantValue::Float(f) => {
                let float_val = self.context.f64_type().const_float(*f);
                BasicValueEnum::FloatValue(float_val)
            }

            ConstantValue::String(s) => {
                // Create a global string constant
                let string_val = self.context.const_string(s.as_bytes(), true);
                // Return as array for now, caller needs to create global and get pointer
                BasicValueEnum::ArrayValue(string_val)
            }

            ConstantValue::Char(c) => {
                let char_val = self.context.i8_type().const_int(*c as u64, false);
                BasicValueEnum::IntValue(char_val)
            }

            ConstantValue::Null => {
                // Null pointer
                let null_ptr = self
                    .context
                    .i8_type()
                    .ptr_type(AddressSpace::default())
                    .const_null();
                BasicValueEnum::PointerValue(null_ptr)
            }
        };

        Ok(llvm_value)
    }

    /// Create an integer constant of specified width
    pub fn const_int(&self, value: i64, width: u32, signed: bool) -> IntValue<'ctx> {
        let int_type = self.context.custom_width_int_type(width);
        int_type.const_int(value as u64, signed)
    }

    /// Create a float constant
    pub fn const_float(&self, value: f64, double_precision: bool) -> FloatValue<'ctx> {
        if double_precision {
            self.context.f64_type().const_float(value)
        } else {
            self.context.f32_type().const_float(value)
        }
    }

    /// Create a boolean constant
    pub fn const_bool(&self, value: bool) -> IntValue<'ctx> {
        self.context.bool_type().const_int(value as u64, false)
    }

    /// Create a null pointer constant
    pub fn const_null_ptr(&self) -> PointerValue<'ctx> {
        self.context
            .i8_type()
            .ptr_type(AddressSpace::default())
            .const_null()
    }

    /// Create a string constant
    pub fn const_string(&self, value: &str, null_terminated: bool) -> BasicValueEnum<'ctx> {
        let string_val = self.context.const_string(value.as_bytes(), null_terminated);
        BasicValueEnum::ArrayValue(string_val)
    }

    /// Create an array constant
    pub fn const_array(
        &self,
        values: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, SemanticError> {
        if values.is_empty() {
            return Err(SemanticError::InvalidType {
                type_name: "array".to_string(),
                reason: "Cannot create empty array constant".to_string(),
                location: crate::error::SourceLocation::unknown(),
            });
        }

        // All elements must be the same type
        let element_type = values[0].get_type();
        for value in values.iter().skip(1) {
            if value.get_type() != element_type {
                return Err(SemanticError::InvalidType {
                    type_name: "array".to_string(),
                    reason: "Array elements must have the same type".to_string(),
                    location: crate::error::SourceLocation::unknown(),
                });
            }
        }

        match values[0] {
            BasicValueEnum::IntValue(_) => {
                let int_values: Vec<IntValue> = values.iter().map(|v| v.into_int_value()).collect();
                let array_val = element_type.into_int_type().const_array(&int_values);
                Ok(BasicValueEnum::ArrayValue(array_val))
            }

            BasicValueEnum::FloatValue(_) => {
                let float_values: Vec<FloatValue> =
                    values.iter().map(|v| v.into_float_value()).collect();
                let array_val = element_type.into_float_type().const_array(&float_values);
                Ok(BasicValueEnum::ArrayValue(array_val))
            }

            BasicValueEnum::PointerValue(_) => {
                let ptr_values: Vec<PointerValue> =
                    values.iter().map(|v| v.into_pointer_value()).collect();
                let array_val = element_type.into_pointer_type().const_array(&ptr_values);
                Ok(BasicValueEnum::ArrayValue(array_val))
            }

            _ => Err(SemanticError::UnsupportedFeature {
                feature: "Array constant type not supported".to_string(),
                location: crate::error::SourceLocation::unknown(),
            }),
        }
    }

    /// Create a struct constant
    pub fn const_struct(
        &self,
        values: &[BasicValueEnum<'ctx>],
        packed: bool,
    ) -> BasicValueEnum<'ctx> {
        let struct_val = self.context.const_struct(values, packed);
        BasicValueEnum::StructValue(struct_val)
    }

    /// Get the LLVM context
    pub fn context(&self) -> &'ctx Context {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::context::Context;

    #[test]
    fn test_convert_boolean_constant() {
        let context = Context::create();
        let converter = ValueConverter::new(&context);

        let true_val = converter
            .convert_constant_value(&ConstantValue::Bool(true))
            .unwrap();
        assert!(matches!(true_val, BasicValueEnum::IntValue(_)));

        let false_val = converter
            .convert_constant_value(&ConstantValue::Bool(false))
            .unwrap();
        assert!(matches!(false_val, BasicValueEnum::IntValue(_)));
    }

    #[test]
    fn test_convert_integer_constant() {
        let context = Context::create();
        let converter = ValueConverter::new(&context);

        let int_val = converter
            .convert_constant_value(&ConstantValue::Integer(42))
            .unwrap();
        assert!(matches!(int_val, BasicValueEnum::IntValue(_)));

        let negative_val = converter
            .convert_constant_value(&ConstantValue::Integer(-123))
            .unwrap();
        assert!(matches!(negative_val, BasicValueEnum::IntValue(_)));
    }

    #[test]
    fn test_convert_float_constant() {
        let context = Context::create();
        let converter = ValueConverter::new(&context);

        let float_val = converter
            .convert_constant_value(&ConstantValue::Float(3.14))
            .unwrap();
        assert!(matches!(float_val, BasicValueEnum::FloatValue(_)));

        let negative_float = converter
            .convert_constant_value(&ConstantValue::Float(-2.71))
            .unwrap();
        assert!(matches!(negative_float, BasicValueEnum::FloatValue(_)));
    }

    #[test]
    fn test_convert_string_constant() {
        let context = Context::create();
        let converter = ValueConverter::new(&context);

        // String constants are returned as ArrayValue, not PointerValue
        // The caller needs to create a global and get its pointer
        let string_val = converter
            .convert_constant_value(&ConstantValue::String("hello".to_string()))
            .unwrap();
        assert!(matches!(string_val, BasicValueEnum::ArrayValue(_)));

        let empty_string = converter
            .convert_constant_value(&ConstantValue::String("".to_string()))
            .unwrap();
        assert!(matches!(empty_string, BasicValueEnum::ArrayValue(_)));
    }

    #[test]
    fn test_convert_null_constant() {
        let context = Context::create();
        let converter = ValueConverter::new(&context);

        let null_val = converter
            .convert_constant_value(&ConstantValue::Null)
            .unwrap();
        assert!(matches!(null_val, BasicValueEnum::PointerValue(_)));
    }

    #[test]
    fn test_const_helpers() {
        let context = Context::create();
        let converter = ValueConverter::new(&context);

        // Test integer constants
        let int_val = converter.const_int(42, 32, true);
        assert_eq!(int_val.get_type().get_bit_width(), 32);

        // Test float constants
        let _float_val = converter.const_float(3.14, true);
        let _float32_val = converter.const_float(2.71, false);

        // Test boolean constants
        let _true_val = converter.const_bool(true);

        // Test null pointer
        let _null_ptr = converter.const_null_ptr();

        // Note: Type checking methods not available in this LLVM version
    }

    #[test]
    fn test_const_array_integers() {
        let context = Context::create();
        let converter = ValueConverter::new(&context);

        let values = vec![
            BasicValueEnum::IntValue(context.i32_type().const_int(1, false)),
            BasicValueEnum::IntValue(context.i32_type().const_int(2, false)),
            BasicValueEnum::IntValue(context.i32_type().const_int(3, false)),
        ];

        let array_val = converter.const_array(&values).unwrap();
        assert!(matches!(array_val, BasicValueEnum::ArrayValue(_)));
    }

    #[test]
    fn test_const_struct() {
        let context = Context::create();
        let converter = ValueConverter::new(&context);

        let values = vec![
            BasicValueEnum::IntValue(context.i32_type().const_int(42, false)),
            BasicValueEnum::FloatValue(context.f64_type().const_float(3.14)),
        ];

        let struct_val = converter.const_struct(&values, false);
        assert!(matches!(struct_val, BasicValueEnum::StructValue(_)));
    }
}
