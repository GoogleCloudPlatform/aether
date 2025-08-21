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

//! Pattern Generator Module
//! 
//! Generates code from patterns by instantiating templates with parameters

use super::*;
use crate::ast::{Statement, Expression, Block, Function, Identifier, TypeSpecifier};
use crate::error::SourceLocation;
use crate::parser::Parser;
use std::collections::HashMap;

/// Pattern generator for instantiating patterns
pub struct PatternGenerator {
    /// Pattern library reference
    library: PatternLibrary,
    
    /// Template engine
    template_engine: TemplateEngine,
    
    /// Generation cache
    cache: HashMap<String, GeneratedCode>,
}

/// Generated code from pattern
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    /// Pattern ID used
    pub pattern_id: String,
    
    /// Parameter values used
    pub parameters: HashMap<String, ParameterValue>,
    
    /// Generated AST
    pub ast: GeneratedAST,
    
    /// Verification status
    pub verified: bool,
    
    /// Generation metadata
    pub metadata: GenerationMetadata,
}

/// Generated AST types
#[derive(Debug, Clone)]
pub enum GeneratedAST {
    Function(Box<Function>),
    Statement(Box<Statement>),
    Expression(Box<Expression>),
    Module(GeneratedModule),
}

/// Generated module
#[derive(Debug, Clone)]
pub struct GeneratedModule {
    pub name: String,
    pub imports: Vec<String>,
    pub types: Vec<crate::ast::TypeDefinition>,
    pub functions: Vec<Function>,
}

/// Generation metadata
#[derive(Debug, Clone)]
pub struct GenerationMetadata {
    /// Time taken to generate
    pub generation_time_ms: u64,
    
    /// Template expansions performed
    pub expansions: Vec<TemplateExpansion>,
    
    /// Warnings during generation
    pub warnings: Vec<String>,
}

/// Template expansion record
#[derive(Debug, Clone)]
pub struct TemplateExpansion {
    pub template_var: String,
    pub value: String,
    pub location: String,
}

impl PatternGenerator {
    /// Create new pattern generator
    pub fn new(library: PatternLibrary) -> Self {
        Self {
            library,
            template_engine: TemplateEngine::new(),
            cache: HashMap::new(),
        }
    }
    
    /// Generate code from pattern with parameters
    pub fn generate(
        &mut self,
        pattern_id: &str,
        parameters: HashMap<String, ParameterValue>,
    ) -> Result<GeneratedCode, GenerationError> {
        // Get pattern from library
        let pattern = self.library.get_pattern(pattern_id)
            .ok_or(GenerationError::PatternNotFound(pattern_id.to_string()))?;
        
        // Validate parameters
        self.validate_parameters(pattern, &parameters)?;
        
        // Apply defaults for missing parameters
        let parameters = self.apply_defaults(pattern, parameters);
        
        // Generate based on template type
        let start_time = std::time::Instant::now();
        let (ast, expansions) = match &pattern.template {
            PatternTemplate::Function(func_template) => {
                let (func, exp) = self.generate_function(func_template, &parameters)?;
                (GeneratedAST::Function(Box::new(func)), exp)
            }
            PatternTemplate::Statement(stmt_template) => {
                let (stmt, exp) = self.generate_statement(stmt_template, &parameters)?;
                (GeneratedAST::Statement(Box::new(stmt)), exp)
            }
            PatternTemplate::Expression(expr_template) => {
                let (expr, exp) = self.generate_expression(expr_template, &parameters)?;
                (GeneratedAST::Expression(Box::new(expr)), exp)
            }
            PatternTemplate::Module(mod_template) => {
                let (module, exp) = self.generate_module(mod_template, &parameters)?;
                (GeneratedAST::Module(module), exp)
            }
        };
        
        let generation_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Create generated code
        let generated = GeneratedCode {
            pattern_id: pattern_id.to_string(),
            parameters: parameters.clone(),
            ast,
            verified: false, // Will be verified separately
            metadata: GenerationMetadata {
                generation_time_ms,
                expansions,
                warnings: vec![],
            },
        };
        
        Ok(generated)
    }
    
    /// Generate from intent using LLM-friendly interface
    pub fn generate_from_intent(
        &mut self,
        intent: &str,
        context: &GenerationContext,
    ) -> Result<Vec<GeneratedCode>, GenerationError> {
        // Find matching patterns
        let patterns = self.library.find_by_intent(intent);
        
        if patterns.is_empty() {
            return Err(GenerationError::NoMatchingPattern(intent.to_string()));
        }
        
        let mut results = Vec::new();
        
        // Try each matching pattern
        for pattern in patterns {
            // Infer parameters from context
            match self.infer_parameters(pattern, context) {
                Ok(parameters) => {
                    match self.generate(&pattern.id, parameters) {
                        Ok(generated) => results.push(generated),
                        Err(_) => continue, // Try next pattern
                    }
                }
                Err(_) => continue,
            }
        }
        
        if results.is_empty() {
            Err(GenerationError::GenerationFailed("No patterns could be instantiated".to_string()))
        } else {
            Ok(results)
        }
    }
    
    /// Validate parameters against pattern requirements
    fn validate_parameters(
        &self,
        pattern: &Pattern,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(), GenerationError> {
        for param_spec in &pattern.parameters {
            // Check if required parameter is provided
            if param_spec.default.is_none() && !parameters.contains_key(&param_spec.name) {
                return Err(GenerationError::MissingParameter(param_spec.name.clone()));
            }
            
            // Check parameter type
            if let Some(value) = parameters.get(&param_spec.name) {
                if !self.check_parameter_type(&param_spec.param_type, value) {
                    return Err(GenerationError::TypeMismatch {
                        parameter: param_spec.name.clone(),
                        expected: format!("{:?}", param_spec.param_type),
                        actual: format!("{:?}", value),
                    });
                }
                
                // Check constraints
                for constraint in &param_spec.constraints {
                    if !self.check_constraint(constraint, value) {
                        return Err(GenerationError::ConstraintViolation {
                            parameter: param_spec.name.clone(),
                            constraint: format!("{:?}", constraint),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply default values for missing parameters
    fn apply_defaults(
        &self,
        pattern: &Pattern,
        mut parameters: HashMap<String, ParameterValue>,
    ) -> HashMap<String, ParameterValue> {
        for param_spec in &pattern.parameters {
            if !parameters.contains_key(&param_spec.name) {
                if let Some(default) = &param_spec.default {
                    parameters.insert(param_spec.name.clone(), default.clone());
                }
            }
        }
        
        parameters
    }
    
    /// Generate function from template
    fn generate_function(
        &self,
        template: &FunctionTemplate,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(Function, Vec<TemplateExpansion>), GenerationError> {
        let mut expansions = Vec::new();
        
        // Expand name
        let (name, exp) = self.template_engine.expand(&template.name_template, parameters)?;
        expansions.extend(exp);
        
        // Expand parameters
        let mut func_params = Vec::new();
        for param_template in &template.parameters {
            let (param_name, exp1) = self.template_engine.expand(&param_template.name_template, parameters)?;
            let (param_type, exp2) = self.template_engine.expand(&param_template.type_template, parameters)?;
            
            expansions.extend(exp1);
            expansions.extend(exp2);
            
            func_params.push(crate::ast::Parameter {
                name: Identifier::new(param_name, SourceLocation::unknown()),
                type_spec: self.parse_type(&param_type)?,
                metadata: None,
                source_location: SourceLocation::unknown(),
            });
        }
        
        // Expand return type
        let (return_type_str, exp) = self.template_engine.expand(&template.return_type_template, parameters)?;
        expansions.extend(exp);
        let return_type = self.parse_type(&return_type_str)?;
        
        // Expand body
        let (body_str, exp) = self.template_engine.expand(&template.body_template, parameters)?;
        expansions.extend(exp);
        let body = self.parse_block(&body_str)?;
        
        // Create function
        let function = Function {
            name: Identifier::new(name, SourceLocation::unknown()),
            metadata: None,
            parameters: func_params,
            return_type: Box::new(return_type),
            body,
            source_location: SourceLocation::unknown(),
        };
        
        Ok((function, expansions))
    }
    
    /// Generate statement from template
    fn generate_statement(
        &self,
        template: &StatementTemplate,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(Statement, Vec<TemplateExpansion>), GenerationError> {
        let (expanded, expansions) = self.template_engine.expand(&template.template, parameters)?;
        let statement = self.parse_statement(&expanded)?;
        Ok((statement, expansions))
    }
    
    /// Generate expression from template
    fn generate_expression(
        &self,
        template: &ExpressionTemplate,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(Expression, Vec<TemplateExpansion>), GenerationError> {
        let (expanded, expansions) = self.template_engine.expand(&template.template, parameters)?;
        let expression = self.parse_expression(&expanded)?;
        Ok((expression, expansions))
    }
    
    /// Generate module from template
    fn generate_module(
        &self,
        template: &ModuleTemplate,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(GeneratedModule, Vec<TemplateExpansion>), GenerationError> {
        let mut all_expansions = Vec::new();
        
        // Expand module name
        let (name, exp) = self.template_engine.expand(&template.name_template, parameters)?;
        all_expansions.extend(exp);
        
        // Expand imports
        let mut imports = Vec::new();
        for import_template in &template.imports {
            let (import, exp) = self.template_engine.expand(import_template, parameters)?;
            all_expansions.extend(exp);
            imports.push(import);
        }
        
        // Generate functions
        let mut functions = Vec::new();
        for func_template in &template.functions {
            let (func, exp) = self.generate_function(func_template, parameters)?;
            all_expansions.extend(exp);
            functions.push(func);
        }
        
        Ok((GeneratedModule {
            name,
            imports,
            types: vec![], // TODO: Generate types
            functions,
        }, all_expansions))
    }
    
    /// Check if parameter value matches expected type
    fn check_parameter_type(&self, expected: &ParameterType, value: &ParameterValue) -> bool {
        match (expected, value) {
            (ParameterType::TypeName, ParameterValue::Type(_)) => true,
            (ParameterType::Expression, ParameterValue::Expression(_)) => true,
            (ParameterType::Statement, ParameterValue::Statement(_)) => true,
            (ParameterType::Block, ParameterValue::Block(_)) => true,
            (ParameterType::Identifier, ParameterValue::Identifier(_)) => true,
            (ParameterType::IntegerConstant, ParameterValue::Integer(_)) => true,
            (ParameterType::StringConstant, ParameterValue::String(_)) => true,
            (ParameterType::BooleanFlag, ParameterValue::Boolean(_)) => true,
            (ParameterType::Choice { options }, ParameterValue::Choice(choice)) => {
                options.contains(choice)
            }
            _ => false,
        }
    }
    
    /// Check if parameter value satisfies constraint
    fn check_constraint(&self, constraint: &ParameterConstraint, value: &ParameterValue) -> bool {
        match constraint {
            ParameterConstraint::Range { min, max } => {
                if let ParameterValue::Integer(n) = value {
                    n >= min && n <= max
                } else {
                    false
                }
            }
            ParameterConstraint::MinLength(min) => {
                match value {
                    ParameterValue::String(s) => s.len() >= *min,
                    ParameterValue::Identifier(s) => s.len() >= *min,
                    _ => false,
                }
            }
            ParameterConstraint::MaxLength(max) => {
                match value {
                    ParameterValue::String(s) => s.len() <= *max,
                    ParameterValue::Identifier(s) => s.len() <= *max,
                    _ => false,
                }
            }
            ParameterConstraint::Regex(pattern) => {
                // TODO: Implement regex matching
                true
            }
            ParameterConstraint::Custom(_) => {
                // TODO: Implement custom validation
                true
            }
        }
    }
    
    /// Infer parameters from generation context
    fn infer_parameters(
        &self,
        pattern: &Pattern,
        context: &GenerationContext,
    ) -> Result<HashMap<String, ParameterValue>, GenerationError> {
        let mut parameters = HashMap::new();
        
        for param_spec in &pattern.parameters {
            // Try to infer from context
            if let Some(value) = context.get_parameter(&param_spec.name) {
                parameters.insert(param_spec.name.clone(), value);
            } else if let Some(default) = &param_spec.default {
                parameters.insert(param_spec.name.clone(), default.clone());
            } else {
                // Try to infer from parameter type
                match &param_spec.param_type {
                    ParameterType::Identifier => {
                        if let Some(name) = context.suggested_name.clone() {
                            parameters.insert(param_spec.name.clone(), ParameterValue::Identifier(name));
                        }
                    }
                    ParameterType::TypeName => {
                        if let Some(type_name) = context.suggested_type.clone() {
                            parameters.insert(param_spec.name.clone(), ParameterValue::Type(type_name));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(parameters)
    }
    
    // Parsing helpers (simplified - would use real parser)
    
    fn parse_type(&self, type_str: &str) -> Result<TypeSpecifier, GenerationError> {
        // Simplified type parsing
        Ok(TypeSpecifier::Primitive {
            type_name: crate::ast::PrimitiveType::Int,
            source_location: SourceLocation::unknown(),
        })
    }
    
    fn parse_block(&self, block_str: &str) -> Result<Block, GenerationError> {
        // Simplified block parsing
        Ok(Block {
            statements: vec![],
            source_location: SourceLocation::unknown(),
        })
    }
    
    fn parse_statement(&self, stmt_str: &str) -> Result<Statement, GenerationError> {
        // Simplified statement parsing
        Ok(Statement::Block {
            block: Block {
                statements: vec![],
                source_location: SourceLocation::unknown(),
            },
            source_location: SourceLocation::unknown(),
        })
    }
    
    fn parse_expression(&self, expr_str: &str) -> Result<Expression, GenerationError> {
        // Simplified expression parsing
        Ok(Expression::IntegerLiteral {
            value: 0,
            source_location: SourceLocation::unknown(),
        })
    }
}

/// Template engine for expanding templates
struct TemplateEngine {
    /// Registered template functions
    functions: HashMap<String, Box<dyn TemplateFn>>,
}

trait TemplateFn {
    fn apply(&self, args: &[String]) -> String;
}

impl TemplateEngine {
    fn new() -> Self {
        let mut engine = Self {
            functions: HashMap::new(),
        };
        
        // Register built-in functions
        engine.register_builtin_functions();
        
        engine
    }
    
    fn register_builtin_functions(&mut self) {
        // TODO: Add template functions like uppercase, lowercase, etc.
    }
    
    /// Expand template with parameters
    fn expand(
        &self,
        template: &str,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(String, Vec<TemplateExpansion>), GenerationError> {
        let mut result = template.to_string();
        let mut expansions = Vec::new();
        
        // Simple template expansion using {{param_name}}
        for (name, value) in parameters {
            let placeholder = format!("{{{{{}}}}}", name);
            if result.contains(&placeholder) {
                let value_str = self.parameter_value_to_string(value);
                result = result.replace(&placeholder, &value_str);
                
                expansions.push(TemplateExpansion {
                    template_var: name.clone(),
                    value: value_str,
                    location: template.to_string(),
                });
            }
        }
        
        // Check for unexpanded placeholders
        if result.contains("{{") && result.contains("}}") {
            return Err(GenerationError::UnexpandedTemplate(result));
        }
        
        Ok((result, expansions))
    }
    
    fn parameter_value_to_string(&self, value: &ParameterValue) -> String {
        match value {
            ParameterValue::Type(s) => s.clone(),
            ParameterValue::Identifier(s) => s.clone(),
            ParameterValue::String(s) => format!("\"{}\"", s),
            ParameterValue::Integer(n) => n.to_string(),
            ParameterValue::Boolean(b) => b.to_string().to_uppercase(),
            ParameterValue::Choice(s) => s.clone(),
            ParameterValue::Expression(expr) => format!("{:?}", expr), // Simplified
            ParameterValue::Statement(stmt) => format!("{:?}", stmt), // Simplified
            ParameterValue::Block(block) => format!("{:?}", block), // Simplified
        }
    }
}

/// Generation context for LLM-friendly generation
#[derive(Debug, Clone)]
pub struct GenerationContext {
    /// Suggested function/variable name
    pub suggested_name: Option<String>,
    
    /// Suggested type
    pub suggested_type: Option<String>,
    
    /// Available variables in scope
    pub variables: HashMap<String, String>, // name -> type
    
    /// Custom parameters
    pub custom_parameters: HashMap<String, ParameterValue>,
}

impl GenerationContext {
    pub fn new() -> Self {
        Self {
            suggested_name: None,
            suggested_type: None,
            variables: HashMap::new(),
            custom_parameters: HashMap::new(),
        }
    }
    
    fn get_parameter(&self, name: &str) -> Option<ParameterValue> {
        self.custom_parameters.get(name).cloned()
    }
}

/// Generation errors
#[derive(Debug, Clone)]
pub enum GenerationError {
    PatternNotFound(String),
    MissingParameter(String),
    TypeMismatch {
        parameter: String,
        expected: String,
        actual: String,
    },
    ConstraintViolation {
        parameter: String,
        constraint: String,
    },
    UnexpandedTemplate(String),
    ParseError(String),
    NoMatchingPattern(String),
    GenerationFailed(String),
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationError::PatternNotFound(id) => write!(f, "Pattern not found: {}", id),
            GenerationError::MissingParameter(name) => write!(f, "Missing required parameter: {}", name),
            GenerationError::TypeMismatch { parameter, expected, actual } => {
                write!(f, "Type mismatch for parameter '{}': expected {}, got {}", parameter, expected, actual)
            }
            GenerationError::ConstraintViolation { parameter, constraint } => {
                write!(f, "Constraint violation for parameter '{}': {}", parameter, constraint)
            }
            GenerationError::UnexpandedTemplate(template) => write!(f, "Unexpanded template variables in: {}", template),
            GenerationError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            GenerationError::NoMatchingPattern(intent) => write!(f, "No pattern matches intent: {}", intent),
            GenerationError::GenerationFailed(msg) => write!(f, "Generation failed: {}", msg),
        }
    }
}

impl std::error::Error for GenerationError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_generation() {
        let library = PatternLibrary::new();
        let mut generator = PatternGenerator::new(library);
        
        // Test template expansion
        let engine = TemplateEngine::new();
        let mut params = HashMap::new();
        params.insert("name".to_string(), ParameterValue::Identifier("test_func".to_string()));
        params.insert("type".to_string(), ParameterValue::Type("INT".to_string()));
        
        let (expanded, expansions) = engine.expand(
            "(FUNCTION {{name}} RETURNS {{type}})",
            &params
        ).unwrap();
        
        assert_eq!(expanded, "(FUNCTION test_func RETURNS INT)");
        assert_eq!(expansions.len(), 2);
    }
}