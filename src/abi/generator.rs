//! ABI Generator - Extracts public symbols from AST and generates ABI files
//!
//! After semantic analysis, this module extracts public symbols from a module
//! and generates the ABI metadata file that other modules can import.

use crate::abi::{
    AbiFunction, AbiModule, AbiSourceLocation, AbiStruct, AbiType,
    CallingConvention, Contract, FunctionContracts, FunctionKind, FunctionParameter,
    FunctionSignature, GenericParam, GenericParamKind, ParameterMode, StructField, Visibility,
};
use crate::ast::{
    self, ExternalFunction, Function, Module, Parameter, TypeDefinition, TypeSpecifier,
};
use crate::error::CompilerError;
use crate::symbols::SymbolTable;

use std::path::Path;

/// Generates ABI from a parsed and analyzed module
pub struct AbiGenerator<'a> {
    module: &'a Module,
    #[allow(dead_code)] // Will be used for resolving type info
    symbol_table: &'a SymbolTable,
    source_path: String,
}

impl<'a> AbiGenerator<'a> {
    /// Create a new ABI generator for a module
    pub fn new(module: &'a Module, symbol_table: &'a SymbolTable, source_path: &str) -> Self {
        AbiGenerator {
            module,
            symbol_table,
            source_path: source_path.to_string(),
        }
    }

    /// Generate the complete ABI for the module
    pub fn generate(&self) -> Result<AbiModule, CompilerError> {
        let mut abi = AbiModule::new(self.module.name.name.clone(), self.source_path.clone());

        // Extract dependencies (imports)
        for import in &self.module.imports {
            abi.dependencies.push(crate::abi::Dependency {
                module: import.module_name.name.clone(),
                version_constraint: None,
                imports: Vec::new(), // TODO: Track specific imports
            });
        }

        // Extract functions (both native and external)
        self.generate_functions(&mut abi)?;

        // Extract types (structs, enums, type aliases)
        self.generate_types(&mut abi)?;

        // Extract traits
        self.generate_traits(&mut abi)?;

        // Extract trait implementations
        self.generate_impls(&mut abi)?;

        Ok(abi)
    }

    /// Generate ABI for all functions in the module
    fn generate_functions(&self, abi: &mut AbiModule) -> Result<(), CompilerError> {
        // Generate ABI for native Aether functions
        for func in &self.module.function_definitions {
            if self.is_public_function(func) {
                let abi_func = self.convert_function(func)?;
                abi.functions.push(abi_func);
            }
        }

        // Generate ABI for external (FFI) functions
        for ext_func in &self.module.external_functions {
            let abi_func = self.convert_external_function(ext_func)?;
            abi.functions.push(abi_func);
        }

        Ok(())
    }

    /// Check if a function should be exported in the ABI
    fn is_public_function(&self, func: &Function) -> bool {
        // Functions with @export annotation are public
        if func.export_info.is_some() {
            return true;
        }

        // Check if the function is in the module's exports list
        for export in &self.module.exports {
            if let ast::ExportStatement::Function { name, .. } = export {
                if name.name == func.name.name {
                    return true;
                }
            }
        }

        // By default, all functions in a module are public unless marked private
        // This matches Aether's current behavior for stdlib modules
        true
    }

    /// Convert a native Aether function to ABI
    fn convert_function(&self, func: &Function) -> Result<AbiFunction, CompilerError> {
        let signature = self.convert_function_signature(
            &func.parameters,
            &func.return_type,
            &func.generic_parameters,
            &func.where_clause,
        )?;

        let kind = if func.generic_parameters.is_empty() {
            FunctionKind::Native {
                symbol: self.mangle_symbol(&func.name.name),
            }
        } else {
            FunctionKind::Generic {
                symbol_prefix: self.mangle_symbol(&func.name.name),
                has_mir: true, // We'll store MIR for generics
                mir_offset: None,
                mir_length: None,
            }
        };

        let contracts = self.convert_contracts(&func.metadata)?;

        let mut attributes = Vec::new();
        if func.export_info.is_some() {
            attributes.push("export".to_string());
        }
        if func.is_async {
            attributes.push("async".to_string());
        }

        Ok(AbiFunction {
            name: func.name.name.clone(),
            signature,
            kind,
            contracts,
            attributes,
            source_location: AbiSourceLocation::from(&func.source_location),
        })
    }

    /// Convert an external (FFI) function to ABI
    fn convert_external_function(
        &self,
        func: &ExternalFunction,
    ) -> Result<AbiFunction, CompilerError> {
        let signature = self.convert_function_signature(
            &func.parameters,
            &func.return_type,
            &[], // External functions don't have generic parameters
            &[],
        )?;

        let calling_convention = match func.calling_convention {
            ast::CallingConvention::C => CallingConvention::C,
            ast::CallingConvention::StdCall
            | ast::CallingConvention::FastCall
            | ast::CallingConvention::System => CallingConvention::System,
        };

        let kind = FunctionKind::Extern {
            library: func.library.clone(),
            symbol: func.symbol.clone().unwrap_or_else(|| func.name.name.clone()),
            calling_convention,
        };

        Ok(AbiFunction {
            name: func.name.name.clone(),
            signature: FunctionSignature {
                is_variadic: func.variadic,
                ..signature
            },
            kind,
            contracts: FunctionContracts::default(),
            attributes: vec!["extern".to_string()],
            source_location: AbiSourceLocation::from(&func.source_location),
        })
    }

    /// Convert function signature to ABI format
    fn convert_function_signature(
        &self,
        parameters: &[Parameter],
        return_type: &TypeSpecifier,
        generic_params: &[ast::GenericParameter],
        _where_clause: &[ast::WhereClause],
    ) -> Result<FunctionSignature, CompilerError> {
        let params: Vec<FunctionParameter> = parameters
            .iter()
            .map(|p| {
                let ty = self.convert_type_specifier(&p.param_type);
                FunctionParameter {
                    name: p.name.name.clone(),
                    ty,
                    mode: ParameterMode::Owned, // TODO: Detect borrow modes
                }
            })
            .collect();

        let ret_type = self.convert_type_specifier(return_type);

        let generics: Vec<GenericParam> = generic_params
            .iter()
            .map(|g| GenericParam {
                name: g.name.name.clone(),
                kind: GenericParamKind::Type,
                default: None,
            })
            .collect();

        Ok(FunctionSignature {
            generic_params: generics,
            where_clauses: Vec::new(), // TODO: Convert where clauses
            parameters: params,
            return_type: ret_type,
            is_variadic: false,
        })
    }

    /// Convert TypeSpecifier to AbiType
    fn convert_type_specifier(&self, ts: &TypeSpecifier) -> AbiType {
        match ts {
            TypeSpecifier::Primitive { type_name, .. } => AbiType::Primitive {
                name: primitive_to_string(type_name),
            },

            TypeSpecifier::Named { name, .. } => AbiType::Named {
                name: name.name.clone(),
                module: None, // TODO: Track module for qualified types
            },

            TypeSpecifier::Generic {
                base_type,
                type_arguments,
                ..
            } => AbiType::GenericInstance {
                base: base_type.name.clone(),
                module: None,
                args: type_arguments
                    .iter()
                    .map(|t| self.convert_type_specifier(t))
                    .collect(),
            },

            TypeSpecifier::TypeParameter { name, .. } => AbiType::GenericParam {
                name: name.name.clone(),
            },

            TypeSpecifier::Array {
                element_type,
                size,
                ..
            } => AbiType::Array {
                element: Box::new(self.convert_type_specifier(element_type)),
                size: size.as_ref().and_then(|s| extract_array_size(s)),
            },

            TypeSpecifier::Map {
                key_type,
                value_type,
                ..
            } => AbiType::Map {
                key: Box::new(self.convert_type_specifier(key_type)),
                value: Box::new(self.convert_type_specifier(value_type)),
            },

            TypeSpecifier::Pointer {
                target_type,
                is_mutable,
                ..
            } => AbiType::Pointer {
                target: Box::new(self.convert_type_specifier(target_type)),
                mutable: *is_mutable,
            },

            TypeSpecifier::Function {
                parameter_types,
                return_type,
                ..
            } => AbiType::Function {
                params: parameter_types
                    .iter()
                    .map(|t| self.convert_type_specifier(t))
                    .collect(),
                ret: Box::new(self.convert_type_specifier(return_type)),
            },

            TypeSpecifier::Owned { base_type, .. } => {
                // For ABI purposes, we treat owned types as their base type
                self.convert_type_specifier(base_type)
            }
        }
    }

    /// Convert function contracts to ABI format
    fn convert_contracts(
        &self,
        metadata: &ast::FunctionMetadata,
    ) -> Result<FunctionContracts, CompilerError> {
        let preconditions: Vec<Contract> = metadata
            .preconditions
            .iter()
            .map(|c| Contract {
                expr: format_expression(&c.condition),
                message: c.message.clone(),
            })
            .collect();

        let postconditions: Vec<Contract> = metadata
            .postconditions
            .iter()
            .map(|c| Contract {
                expr: format_expression(&c.condition),
                message: c.message.clone(),
            })
            .collect();

        Ok(FunctionContracts {
            preconditions,
            postconditions,
            verified: false, // Will be updated by verification pass
            assumes_axioms: Vec::new(),
        })
    }

    /// Generate symbol name with module prefix
    /// Uses dot separator to match LLVM backend symbol naming (e.g., "MathUtils.add")
    fn mangle_symbol(&self, name: &str) -> String {
        format!("{}.{}", self.module.name.name, name)
    }

    /// Generate ABI for type definitions
    fn generate_types(&self, abi: &mut AbiModule) -> Result<(), CompilerError> {
        for type_def in &self.module.type_definitions {
            match type_def {
                TypeDefinition::Structured {
                    name,
                    generic_parameters,
                    fields,
                    source_location,
                    ..
                } => {
                    let abi_struct = AbiStruct {
                        name: name.name.clone(),
                        generic_params: generic_parameters
                            .iter()
                            .map(|g| GenericParam {
                                name: g.name.name.clone(),
                                kind: GenericParamKind::Type,
                                default: None,
                            })
                            .collect(),
                        where_clauses: Vec::new(),
                        fields: fields
                            .iter()
                            .map(|f| StructField {
                                name: f.name.name.clone(),
                                ty: self.convert_type_specifier(&f.field_type),
                                visibility: Visibility::Public,
                            })
                            .collect(),
                        attributes: Vec::new(),
                        invariants: Vec::new(),
                        source_location: AbiSourceLocation::from(source_location),
                    };
                    abi.types.structs.push(abi_struct);
                }

                TypeDefinition::Enumeration {
                    name,
                    generic_parameters,
                    variants,
                    source_location,
                    ..
                } => {
                    let abi_enum = crate::abi::AbiEnum {
                        name: name.name.clone(),
                        generic_params: generic_parameters
                            .iter()
                            .map(|g| GenericParam {
                                name: g.name.name.clone(),
                                kind: GenericParamKind::Type,
                                default: None,
                            })
                            .collect(),
                        where_clauses: Vec::new(),
                        variants: variants
                            .iter()
                            .map(|v| crate::abi::EnumVariant {
                                name: v.name.name.clone(),
                                fields: v
                                    .associated_types
                                    .iter()
                                    .enumerate()
                                    .map(|(i, t)| crate::abi::VariantField {
                                        name: Some(format!("_{}", i)),
                                        ty: self.convert_type_specifier(t),
                                    })
                                    .collect(),
                                discriminant: None,
                            })
                            .collect(),
                        attributes: Vec::new(),
                        source_location: AbiSourceLocation::from(source_location),
                    };
                    abi.types.enums.push(abi_enum);
                }

                TypeDefinition::Alias {
                    new_name,
                    original_type,
                    generic_parameters,
                    source_location,
                    ..
                } => {
                    let alias = crate::abi::AbiTypeAlias {
                        name: new_name.name.clone(),
                        generic_params: generic_parameters
                            .iter()
                            .map(|g| GenericParam {
                                name: g.name.name.clone(),
                                kind: GenericParamKind::Type,
                                default: None,
                            })
                            .collect(),
                        target: self.convert_type_specifier(original_type),
                        source_location: AbiSourceLocation::from(source_location),
                    };
                    abi.types.type_aliases.push(alias);
                }
            }
        }
        Ok(())
    }

    /// Generate ABI for trait definitions
    fn generate_traits(&self, abi: &mut AbiModule) -> Result<(), CompilerError> {
        for trait_def in &self.module.trait_definitions {
            let abi_trait = crate::abi::AbiTrait {
                name: trait_def.name.name.clone(),
                generic_params: trait_def
                    .generic_parameters
                    .iter()
                    .map(|g| GenericParam {
                        name: g.name.name.clone(),
                        kind: GenericParamKind::Type,
                        default: None,
                    })
                    .collect(),
                super_traits: Vec::new(), // TODO: Extract super traits
                associated_types: Vec::new(), // TODO: Extract associated types
                methods: trait_def
                    .methods
                    .iter()
                    .map(|m| {
                        crate::abi::TraitMethod {
                            name: m.name.name.clone(),
                            signature: FunctionSignature {
                                generic_params: m
                                    .generic_parameters
                                    .iter()
                                    .map(|g| GenericParam {
                                        name: g.name.name.clone(),
                                        kind: GenericParamKind::Type,
                                        default: None,
                                    })
                                    .collect(),
                                where_clauses: Vec::new(),
                                parameters: m
                                    .parameters
                                    .iter()
                                    .map(|p| FunctionParameter {
                                        name: p.name.name.clone(),
                                        ty: self.convert_type_specifier(&p.param_type),
                                        mode: ParameterMode::Owned,
                                    })
                                    .collect(),
                                return_type: self.convert_type_specifier(&m.return_type),
                                is_variadic: false,
                            },
                            has_default: m.default_body.is_some(),
                            contracts: FunctionContracts::default(),
                        }
                    })
                    .collect(),
                axioms: trait_def
                    .axioms
                    .iter()
                    .map(|a| crate::abi::TraitAxiom {
                        name: a.name.as_ref().map(|n| n.name.clone()).unwrap_or_default(),
                        quantifiers: a
                            .quantifiers
                            .iter()
                            .map(|q| {
                                q.variables
                                    .iter()
                                    .map(|v| crate::abi::Quantifier {
                                        var: v.name.name.clone(),
                                        ty: self.convert_type_specifier(&v.var_type),
                                        kind: match q.kind {
                                            ast::QuantifierKind::ForAll => {
                                                crate::abi::QuantifierKind::ForAll
                                            }
                                            ast::QuantifierKind::Exists => {
                                                crate::abi::QuantifierKind::Exists
                                            }
                                        },
                                    })
                                    .collect::<Vec<_>>()
                            })
                            .flatten()
                            .collect(),
                        expr: format_expression(&a.condition),
                        source_location: AbiSourceLocation::from(&a.source_location),
                    })
                    .collect(),
                source_location: AbiSourceLocation::from(&trait_def.source_location),
            };
            abi.traits.push(abi_trait);
        }
        Ok(())
    }

    /// Generate ABI for trait implementations
    fn generate_impls(&self, abi: &mut AbiModule) -> Result<(), CompilerError> {
        for impl_block in &self.module.impl_blocks {
            if let Some(trait_name) = &impl_block.trait_name {
                let abi_impl = crate::abi::AbiImpl {
                    trait_ref: crate::abi::TraitRef {
                        name: trait_name.name.clone(),
                        module: None,
                        type_args: impl_block
                            .trait_generic_args
                            .iter()
                            .map(|t| self.convert_type_specifier(t))
                            .collect(),
                    },
                    for_type: self.convert_type_specifier(&impl_block.for_type),
                    generic_params: impl_block
                        .generic_parameters
                        .iter()
                        .map(|g| GenericParam {
                            name: g.name.name.clone(),
                            kind: GenericParamKind::Type,
                            default: None,
                        })
                        .collect(),
                    where_clauses: Vec::new(),
                    associated_types: Vec::new(),
                    methods: impl_block
                        .methods
                        .iter()
                        .map(|m| crate::abi::MethodImpl {
                            name: m.name.name.clone(),
                            symbol: self.mangle_symbol(&format!(
                                "{}__{}",
                                trait_name.name, m.name.name
                            )),
                        })
                        .collect(),
                    source_location: AbiSourceLocation::from(&impl_block.source_location),
                };
                abi.impls.push(abi_impl);
            }
        }
        Ok(())
    }
}

/// Convert primitive type to string representation
fn primitive_to_string(prim: &ast::PrimitiveType) -> String {
    match prim {
        ast::PrimitiveType::Integer => "Int".to_string(),
        ast::PrimitiveType::Integer32 => "Int32".to_string(),
        ast::PrimitiveType::Integer64 => "Int64".to_string(),
        ast::PrimitiveType::Float => "Float".to_string(),
        ast::PrimitiveType::Float32 => "Float32".to_string(),
        ast::PrimitiveType::Float64 => "Float64".to_string(),
        ast::PrimitiveType::Boolean => "Bool".to_string(),
        ast::PrimitiveType::String => "String".to_string(),
        ast::PrimitiveType::Char => "Char".to_string(),
        ast::PrimitiveType::Void => "Void".to_string(),
        ast::PrimitiveType::SizeT => "SizeT".to_string(),
        ast::PrimitiveType::UIntPtrT => "UIntPtrT".to_string(),
        ast::PrimitiveType::UInt8 => "UInt8".to_string(),
        ast::PrimitiveType::Int8 => "Int8".to_string(),
        ast::PrimitiveType::UInt16 => "UInt16".to_string(),
        ast::PrimitiveType::Int16 => "Int16".to_string(),
        ast::PrimitiveType::UInt32 => "UInt32".to_string(),
        ast::PrimitiveType::Int32 => "Int32".to_string(),
        ast::PrimitiveType::UInt64 => "UInt64".to_string(),
    }
}

/// Extract array size from constant expression
fn extract_array_size(expr: &ast::Expression) -> Option<usize> {
    match expr {
        ast::Expression::IntegerLiteral { value, .. } => Some(*value as usize),
        _ => None,
    }
}

/// Format expression to string for storage in ABI
fn format_expression(expr: &ast::Expression) -> String {
    // Simplified expression formatting for contract storage
    use ast::Expression::*;
    match expr {
        // Literals
        IntegerLiteral { value, .. } => value.to_string(),
        FloatLiteral { value, .. } => value.to_string(),
        StringLiteral { value, .. } => format!("\"{}\"", value),
        BooleanLiteral { value, .. } => value.to_string(),
        CharacterLiteral { value, .. } => format!("'{}'", value),
        NullLiteral { .. } => "null".to_string(),

        // Variable
        Variable { name, .. } => name.name.clone(),

        // Arithmetic binary operators
        Add { left, right, .. } => format!("({} + {})", format_expression(left), format_expression(right)),
        Subtract { left, right, .. } => format!("({} - {})", format_expression(left), format_expression(right)),
        Multiply { left, right, .. } => format!("({} * {})", format_expression(left), format_expression(right)),
        Divide { left, right, .. } => format!("({} / {})", format_expression(left), format_expression(right)),
        Modulo { left, right, .. } => format!("({} % {})", format_expression(left), format_expression(right)),
        IntegerDivide { left, right, .. } => format!("({} // {})", format_expression(left), format_expression(right)),

        // Comparison operators
        Equals { left, right, .. } => format!("({} == {})", format_expression(left), format_expression(right)),
        NotEquals { left, right, .. } => format!("({} != {})", format_expression(left), format_expression(right)),
        LessThan { left, right, .. } => format!("({} < {})", format_expression(left), format_expression(right)),
        LessThanOrEqual { left, right, .. } => format!("({} <= {})", format_expression(left), format_expression(right)),
        GreaterThan { left, right, .. } => format!("({} > {})", format_expression(left), format_expression(right)),
        GreaterThanOrEqual { left, right, .. } => format!("({} >= {})", format_expression(left), format_expression(right)),

        // Logical operators
        LogicalAnd { operands, .. } => {
            let parts: Vec<String> = operands.iter().map(format_expression).collect();
            format!("({})", parts.join(" && "))
        }
        LogicalOr { operands, .. } => {
            let parts: Vec<String> = operands.iter().map(format_expression).collect();
            format!("({})", parts.join(" || "))
        }
        LogicalNot { operand, .. } => format!("!{}", format_expression(operand)),

        // Unary operators
        Negate { operand, .. } => format!("-{}", format_expression(operand)),

        // Member access
        FieldAccess { instance, field_name, .. } => {
            format!("{}.{}", format_expression(instance), field_name.name)
        }

        // Index
        ArrayAccess { array, index, .. } => {
            format!("{}[{}]", format_expression(array), format_expression(index))
        }

        // Function call
        FunctionCall { call, .. } => {
            let func_name = match &call.function_reference {
                ast::FunctionReference::Local { name } => name.name.clone(),
                ast::FunctionReference::Qualified { module, name } => format!("{}.{}", module.name, name.name),
                ast::FunctionReference::External { name } => name.name.clone(),
            };
            let args: Vec<String> = call.arguments.iter().map(|a| format_expression(&a.value)).collect();
            format!("{}({})", func_name, args.join(", "))
        }

        // Default for complex expressions
        _ => "<expr>".to_string(),
    }
}

/// Generate ABI file from a module
pub fn generate_abi_file(
    module: &Module,
    symbol_table: &SymbolTable,
    source_path: &Path,
    output_path: &Path,
) -> Result<(), CompilerError> {
    let generator = AbiGenerator::new(
        module,
        symbol_table,
        &source_path.to_string_lossy(),
    );

    let abi = generator.generate()?;

    abi.save(output_path).map_err(|e| CompilerError::IoError {
        message: format!("Failed to write ABI file {}: {}", output_path.display(), e),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Identifier;
    use crate::error::SourceLocation;
    use crate::symbols::SymbolTable;

    fn make_test_module() -> Module {
        Module {
            name: Identifier {
                name: "test".to_string(),
                source_location: SourceLocation::unknown(),
            },
            intent: None,
            imports: Vec::new(),
            exports: Vec::new(),
            type_definitions: Vec::new(),
            trait_definitions: Vec::new(),
            impl_blocks: Vec::new(),
            constant_declarations: Vec::new(),
            function_definitions: Vec::new(),
            external_functions: Vec::new(),
            source_location: SourceLocation::unknown(),
        }
    }

    #[test]
    fn test_generate_empty_module() {
        let module = make_test_module();
        let symbol_table = SymbolTable::new();
        let generator = AbiGenerator::new(&module, &symbol_table, "test.aether");

        let abi = generator.generate().unwrap();
        assert_eq!(abi.module.name, "test");
        assert!(abi.functions.is_empty());
        assert!(abi.types.structs.is_empty());
    }

    #[test]
    fn test_primitive_to_string() {
        assert_eq!(primitive_to_string(&ast::PrimitiveType::Integer), "Int");
        assert_eq!(primitive_to_string(&ast::PrimitiveType::Boolean), "Bool");
        assert_eq!(primitive_to_string(&ast::PrimitiveType::String), "String");
    }

    #[test]
    fn test_convert_type_specifier() {
        let module = make_test_module();
        let symbol_table = SymbolTable::new();
        let generator = AbiGenerator::new(&module, &symbol_table, "test.aether");

        let ts = TypeSpecifier::Primitive {
            type_name: ast::PrimitiveType::Integer,
            source_location: SourceLocation::unknown(),
        };

        let abi_type = generator.convert_type_specifier(&ts);
        assert_eq!(abi_type, AbiType::Primitive { name: "Int".to_string() });
    }
}
