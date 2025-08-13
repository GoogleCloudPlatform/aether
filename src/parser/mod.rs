//! Parser for AetherScript S-expressions
//! 
//! Converts token stream to Abstract Syntax Tree

use crate::ast::*;
use crate::ast::CastFailureBehavior;
use crate::error::{ParserError, SourceLocation};
use crate::lexer::{Token, TokenType};
use std::collections::HashMap;

/// Parser for AetherScript source code
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    keywords: HashMap<String, KeywordType>,
    errors: Vec<ParserError>,
    recovery_mode: bool,
}

/// Keyword types for parsing
#[derive(Debug, Clone, PartialEq)]
pub enum KeywordType {
    // Module keywords
    DefineModule,
    ImportModule,
    ExportsFunction,
    ExportsType,
    ExportsConstant,
    
    // Declaration keywords
    DeclareVariable,
    DeclareConstant,
    DeclareExternalFunction,
    
    // Type definition keywords
    DefineStructuredType,
    DefineEnumerationType,
    DefineTypeAlias,
    DefineFunction,
    
    // Enumeration and pattern matching keywords
    Variants,
    Variant,
    Holds,
    MatchExpression,
    Case,
    
    // Type keywords
    Integer,
    Float,
    String,
    Char,
    Boolean,
    Void,
    ArrayOfType,
    MapFromTypeToType,
    PointerTo,
    FunctionType,
    
    // Calling convention alias
    Convention,
    
    // Metadata keywords
    Precondition,
    Postcondition,
    Invariant,
    AlgorithmHint,
    PerformanceExpectation,
    ComplexityExpectation,
    
    // Performance metric keywords
    LatencyMs,
    ThroughputOps,
    MemoryBytes,
    CpuPercent,
    
    // Complexity keywords
    Time,
    Space,
    BigO,
    BigTheta,
    BigOmega,
    
    // Failure action keywords
    AssertFail,
    LogWarning,
    ThrowException,
    
    // Function keywords
    AcceptsParameter,
    Returns,
    Body,
    CallFunction,
    ReturnValue,
    ReturnVoid,
    
    // Expression keywords
    ExpressionAdd,
    ExpressionSubtract,
    ExpressionMultiply,
    ExpressionDivide,
    ExpressionIntegerDivide,
    ExpressionModulo,
    ExpressionNegate,
    
    // Predicate keywords
    PredicateEquals,
    PredicateNotEquals,
    PredicateLessThan,
    PredicateLessThanOrEqualTo,
    PredicateGreaterThan,
    PredicateGreaterThanOrEqualTo,
    
    // Logical keywords
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    
    // String operations
    StringConcat,
    StringLength,
    StringCharAt,
    Substring,
    StringEquals,
    StringContains,
    
    // Type conversion
    CastToType,
    ToString,
    ToInteger,
    ToFloat,
    
    // Control flow keywords
    IfCondition,
    ThenExecute,
    ElseIfCondition,
    ElseExecute,
    LoopWhileCondition,
    LoopForEachElement,
    LoopFixedIterations,
    Counter,
    From,
    To,
    Step,
    Do,
    BreakLoop,
    ContinueLoop,
    
    // Assignment and access keywords
    Assign,
    TargetVariable,
    SourceExpression,
    GetFieldValue,
    GetArrayElement,
    SetArrayElement,
    GetMapValue,
    SetMapValue,
    
    // Statement keywords
    ExpressionStatement,
    
    // Error handling keywords
    TryExecute,
    CatchException,
    FinallyExecute,
    
    // Resource management keywords
    ResourceScope,
    AcquireResource,
    ResourceType,
    ResourceBinding,
    Cleanup,
    CleanupOrder,
    
    // Metadata keywords
    Intent,
    
    // Pointer operations
    AddressOf,
    Dereference,
    PointerAdd,
    
    // FFI keywords
    Library,
    Symbol,
    CallingConvention,
    ThreadSafe,
    MayBlock,
    Variadic,
    
    // Construction keywords
    Construct,
    FieldValue,
    ArrayLiteral,
    ArrayLength,
    MapLiteral,
    
    // Misc keywords
    Name,
    Type,
    Value,
    Mutability,
    Mutable,
    Immutable,
    Field,
    Parameter,
    Argument,
    Elements,
    Entry,
    Key,
    Ownership,
    Lifetime,
    Passing,
    ByValue,
    ByReference,
    ExportAs,
    GenericParameters,
    Constraints,
    Param,
    
    // Content and container keywords
    Content,
    Arguments,
    Condition,
    BooleanExpression,
    IterationBody,
    ElementVariable,
    IndexVariable,
    Collection,
    ProtectedBlock,
    HandlerBlock,
    CleanupBlock,
    
    // Literal keywords
    StringLiteral,
}

impl Parser {
    /// Create a new parser with the given tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut parser = Self {
            tokens,
            position: 0,
            keywords: HashMap::new(),
            errors: Vec::new(),
            recovery_mode: false,
        };
        parser.initialize_keywords();
        // Skip any initial comments
        parser.skip_comments();
        parser
    }

    /// Initialize the keyword mapping
    fn initialize_keywords(&mut self) {
        let keywords = [
            ("DEFINE_MODULE", KeywordType::DefineModule),
            ("IMPORT_MODULE", KeywordType::ImportModule),
            ("EXPORTS_FUNCTION", KeywordType::ExportsFunction),
            ("EXPORTS_TYPE", KeywordType::ExportsType),
            ("EXPORTS_CONSTANT", KeywordType::ExportsConstant),
            ("DECLARE_VARIABLE", KeywordType::DeclareVariable),
            ("DECLARE_CONSTANT", KeywordType::DeclareConstant),
            ("DECLARE_EXTERNAL_FUNCTION", KeywordType::DeclareExternalFunction),
            ("DEFINE_STRUCTURED_TYPE", KeywordType::DefineStructuredType),
            ("DEFINE_ENUMERATION_TYPE", KeywordType::DefineEnumerationType),
            ("DEFINE_TYPE_ALIAS", KeywordType::DefineTypeAlias),
            ("DEFINE_FUNCTION", KeywordType::DefineFunction),
            ("VARIANTS", KeywordType::Variants),
            ("VARIANT", KeywordType::Variant),
            ("HOLDS", KeywordType::Holds),
            ("MATCH_EXPRESSION", KeywordType::MatchExpression),
            ("CASE", KeywordType::Case),
            ("INTEGER", KeywordType::Integer),
            ("FLOAT", KeywordType::Float),
            ("STRING", KeywordType::String),
            ("CHAR", KeywordType::Char),
            ("BOOLEAN", KeywordType::Boolean),
            ("VOID", KeywordType::Void),
            ("ARRAY_OF_TYPE", KeywordType::ArrayOfType),
            ("MAP_FROM_TYPE_TO_TYPE", KeywordType::MapFromTypeToType),
            ("POINTER_TO", KeywordType::PointerTo),
            ("FUNCTION_TYPE", KeywordType::FunctionType),
            ("ACCEPTS_PARAMETER", KeywordType::AcceptsParameter),
            ("RETURNS", KeywordType::Returns),
            ("BODY", KeywordType::Body),
            ("CALL_FUNCTION", KeywordType::CallFunction),
            ("RETURN_VALUE", KeywordType::ReturnValue),
            ("RETURN_VOID", KeywordType::ReturnVoid),
            ("EXPRESSION_ADD", KeywordType::ExpressionAdd),
            ("EXPRESSION_SUBTRACT", KeywordType::ExpressionSubtract),
            ("EXPRESSION_MULTIPLY", KeywordType::ExpressionMultiply),
            ("EXPRESSION_DIVIDE", KeywordType::ExpressionDivide),
            ("EXPRESSION_INTEGER_DIVIDE", KeywordType::ExpressionIntegerDivide),
            ("EXPRESSION_MODULO", KeywordType::ExpressionModulo),
            ("EXPRESSION_NEGATE", KeywordType::ExpressionNegate),
            ("PREDICATE_EQUALS", KeywordType::PredicateEquals),
            ("PREDICATE_NOT_EQUALS", KeywordType::PredicateNotEquals),
            ("PREDICATE_LESS_THAN", KeywordType::PredicateLessThan),
            ("PREDICATE_LESS_THAN_OR_EQUAL_TO", KeywordType::PredicateLessThanOrEqualTo),
            ("PREDICATE_GREATER_THAN", KeywordType::PredicateGreaterThan),
            ("PREDICATE_GREATER_THAN_OR_EQUAL_TO", KeywordType::PredicateGreaterThanOrEqualTo),
            ("LOGICAL_AND", KeywordType::LogicalAnd),
            ("LOGICAL_OR", KeywordType::LogicalOr),
            ("LOGICAL_NOT", KeywordType::LogicalNot),
            ("STRING_CONCAT", KeywordType::StringConcat),
            ("STRING_LENGTH", KeywordType::StringLength),
            ("STRING_CHAR_AT", KeywordType::StringCharAt),
            ("SUBSTRING", KeywordType::Substring),
            ("STRING_EQUALS", KeywordType::StringEquals),
            ("STRING_CONTAINS", KeywordType::StringContains),
            ("CAST_TO_TYPE", KeywordType::CastToType),
            ("TO_STRING", KeywordType::ToString),
            ("TO_INTEGER", KeywordType::ToInteger),
            ("TO_FLOAT", KeywordType::ToFloat),
            ("IF_CONDITION", KeywordType::IfCondition),
            ("THEN_EXECUTE", KeywordType::ThenExecute),
            ("ELSE_IF_CONDITION", KeywordType::ElseIfCondition),
            ("ELSE_EXECUTE", KeywordType::ElseExecute),
            ("LOOP_WHILE_CONDITION", KeywordType::LoopWhileCondition),
            ("LOOP_FOR_EACH_ELEMENT", KeywordType::LoopForEachElement),
            ("LOOP_FIXED_ITERATIONS", KeywordType::LoopFixedIterations),
            ("COUNTER", KeywordType::Counter),
            ("FROM", KeywordType::From),
            ("TO", KeywordType::To),
            ("STEP", KeywordType::Step),
            ("DO", KeywordType::Do),
            ("BREAK_LOOP", KeywordType::BreakLoop),
            ("CONTINUE_LOOP", KeywordType::ContinueLoop),
            ("ASSIGN", KeywordType::Assign),
            ("TARGET_VARIABLE", KeywordType::TargetVariable),
            ("SOURCE_EXPRESSION", KeywordType::SourceExpression),
            ("GET_FIELD_VALUE", KeywordType::GetFieldValue),
            ("GET_ARRAY_ELEMENT", KeywordType::GetArrayElement),
            ("SET_ARRAY_ELEMENT", KeywordType::SetArrayElement),
            ("GET_MAP_VALUE", KeywordType::GetMapValue),
            ("SET_MAP_VALUE", KeywordType::SetMapValue),
            ("EXPRESSION_STATEMENT", KeywordType::ExpressionStatement),
            ("TRY_EXECUTE", KeywordType::TryExecute),
            ("CATCH_EXCEPTION", KeywordType::CatchException),
            ("FINALLY_EXECUTE", KeywordType::FinallyExecute),
            ("THROW_EXCEPTION", KeywordType::ThrowException),
            ("RESOURCE_SCOPE", KeywordType::ResourceScope),
            ("ACQUIRE_RESOURCE", KeywordType::AcquireResource),
            ("RESOURCE_TYPE", KeywordType::ResourceType),
            ("RESOURCE_BINDING", KeywordType::ResourceBinding),
            ("CLEANUP", KeywordType::Cleanup),
            ("CLEANUP_ORDER", KeywordType::CleanupOrder),
            ("INTENT", KeywordType::Intent),
            ("PRECONDITION", KeywordType::Precondition),
            ("POSTCONDITION", KeywordType::Postcondition),
            ("INVARIANT", KeywordType::Invariant),
            ("ALGORITHM_HINT", KeywordType::AlgorithmHint),
            ("PERFORMANCE_EXPECTATION", KeywordType::PerformanceExpectation),
            ("COMPLEXITY_EXPECTATION", KeywordType::ComplexityExpectation),
            ("LIBRARY", KeywordType::Library),
            ("SYMBOL", KeywordType::Symbol),
            ("CALLING_CONVENTION", KeywordType::CallingConvention),
            ("CONVENTION", KeywordType::Convention),
            ("THREAD_SAFE", KeywordType::ThreadSafe),
            ("MAY_BLOCK", KeywordType::MayBlock),
            ("VARIADIC", KeywordType::Variadic),
            ("CONSTRUCT", KeywordType::Construct),
            ("FIELD_VALUE", KeywordType::FieldValue),
            ("ARRAY_LITERAL", KeywordType::ArrayLiteral),
            ("ARRAY_LENGTH", KeywordType::ArrayLength),
            ("MAP_LITERAL", KeywordType::MapLiteral),
            ("NAME", KeywordType::Name),
            ("TYPE", KeywordType::Type),
            ("VALUE", KeywordType::Value),
            ("MUTABILITY", KeywordType::Mutability),
            ("MUTABLE", KeywordType::Mutable),
            ("IMMUTABLE", KeywordType::Immutable),
            ("FIELD", KeywordType::Field),
            ("PARAMETER", KeywordType::Parameter),
            ("ARGUMENT", KeywordType::Argument),
            ("ELEMENTS", KeywordType::Elements),
            ("ENTRY", KeywordType::Entry),
            ("KEY", KeywordType::Key),
            ("OWNERSHIP", KeywordType::Ownership),
            ("LIFETIME", KeywordType::Lifetime),
            ("PASSING", KeywordType::Passing),
            ("BY_VALUE", KeywordType::ByValue),
            ("BY_REFERENCE", KeywordType::ByReference),
            ("EXPORT_AS", KeywordType::ExportAs),
            ("GENERIC_PARAMETERS", KeywordType::GenericParameters),
            ("CONSTRAINTS", KeywordType::Constraints),
            ("PARAM", KeywordType::Param),
            ("CONTENT", KeywordType::Content),
            ("ARGUMENTS", KeywordType::Arguments),
            ("CONDITION", KeywordType::Condition),
            ("BOOLEAN_EXPRESSION", KeywordType::BooleanExpression),
            ("ITERATION_BODY", KeywordType::IterationBody),
            ("ELEMENT_VARIABLE", KeywordType::ElementVariable),
            ("INDEX_VARIABLE", KeywordType::IndexVariable),
            ("COLLECTION", KeywordType::Collection),
            ("PROTECTED_BLOCK", KeywordType::ProtectedBlock),
            ("HANDLER_BLOCK", KeywordType::HandlerBlock),
            ("CLEANUP_BLOCK", KeywordType::CleanupBlock),
            ("ADDRESS_OF", KeywordType::AddressOf),
            ("DEREFERENCE", KeywordType::Dereference),
            ("POINTER_ADD", KeywordType::PointerAdd),
            // Metadata keywords
            ("PRECONDITION", KeywordType::Precondition),
            ("POSTCONDITION", KeywordType::Postcondition),
            ("INVARIANT", KeywordType::Invariant),
            ("ALGORITHM_HINT", KeywordType::AlgorithmHint),
            ("PERFORMANCE_EXPECTATION", KeywordType::PerformanceExpectation),
            ("COMPLEXITY_EXPECTATION", KeywordType::ComplexityExpectation),
            // Performance metric keywords
            ("LATENCY_MS", KeywordType::LatencyMs),
            ("THROUGHPUT_OPS", KeywordType::ThroughputOps),
            ("MEMORY_BYTES", KeywordType::MemoryBytes),
            ("CPU_PERCENT", KeywordType::CpuPercent),
            // Complexity keywords
            ("TIME", KeywordType::Time),
            ("SPACE", KeywordType::Space),
            ("BIG_O", KeywordType::BigO),
            ("BIG_THETA", KeywordType::BigTheta),
            ("BIG_OMEGA", KeywordType::BigOmega),
            // Failure action keywords
            ("ASSERT_FAIL", KeywordType::AssertFail),
            ("LOG_WARNING", KeywordType::LogWarning),
            ("THROW_EXCEPTION", KeywordType::ThrowException),
            // Literal keywords
            ("STRING_LITERAL", KeywordType::StringLiteral),
        ];

        for (keyword, keyword_type) in keywords {
            self.keywords.insert(keyword.to_string(), keyword_type);
        }
    }

    /// Get the current token
    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    /// Advance to the next token
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
            // Skip comments
            self.skip_comments();
        }
    }
    
    /// Skip comment tokens
    fn skip_comments(&mut self) {
        while let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::Comment(_) => {
                    if self.position < self.tokens.len() {
                        self.position += 1;
                    }
                }
                _ => break,
            }
        }
    }

    /// Check if we're at the end of tokens
    fn is_at_end(&self) -> bool {
        match self.current_token() {
            Some(token) => matches!(token.token_type, TokenType::Eof),
            None => true,
        }
    }

    /// Consume a left parenthesis
    fn consume_left_paren(&mut self) -> Result<SourceLocation, ParserError> {
        match self.current_token() {
            Some(token) if matches!(token.token_type, TokenType::LeftParen) => {
                let location = token.location.clone();
                self.advance();
                Ok(location)
            }
            Some(token) => Err(ParserError::UnexpectedToken {
                found: format!("{:?}", token.token_type),
                expected: "left parenthesis '('".to_string(),
                location: token.location.clone(),
            }),
            None => Err(ParserError::UnexpectedEof {
                expected: "left parenthesis '('".to_string(),
            }),
        }
    }

    /// Consume a right parenthesis
    fn consume_right_paren(&mut self) -> Result<SourceLocation, ParserError> {
        match self.current_token() {
            Some(token) if matches!(token.token_type, TokenType::RightParen) => {
                let location = token.location.clone();
                self.advance();
                Ok(location)
            }
            Some(token) => Err(ParserError::UnexpectedToken {
                found: format!("{:?}", token.token_type),
                expected: "right parenthesis ')'".to_string(),
                location: token.location.clone(),
            }),
            None => Err(ParserError::UnexpectedEof {
                expected: "right parenthesis ')'".to_string(),
            }),
        }
    }

    /// Consume a specific keyword
    fn peek_keyword(&self, expected: KeywordType) -> bool {
        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Keyword(keyword) => {
                    if let Some(keyword_type) = self.keywords.get(keyword) {
                        *keyword_type == expected
                    } else {
                        false
                    }
                }
                _ => false,
            },
            None => false,
        }
    }
    
    fn consume_keyword(&mut self, expected: KeywordType) -> Result<SourceLocation, ParserError> {
        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Keyword(keyword) => {
                    if let Some(keyword_type) = self.keywords.get(keyword) {
                        if *keyword_type == expected {
                            let location = token.location.clone();
                            self.advance();
                            return Ok(location);
                        }
                    }
                    Err(ParserError::UnexpectedToken {
                        found: keyword.clone(),
                        expected: format!("{:?}", expected),
                        location: token.location.clone(),
                    })
                }
                _ => Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: format!("{:?}", expected),
                    location: token.location.clone(),
                }),
            },
            None => Err(ParserError::UnexpectedEof {
                expected: format!("{:?}", expected),
            }),
        }
    }

    /// Consume an identifier
    fn consume_identifier(&mut self) -> Result<Identifier, ParserError> {
        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Identifier(name) => {
                    let identifier = Identifier::new(name.clone(), token.location.clone());
                    self.advance();
                    Ok(identifier)
                }
                _ => Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: "identifier".to_string(),
                    location: token.location.clone(),
                }),
            },
            None => Err(ParserError::UnexpectedEof {
                expected: "identifier".to_string(),
            }),
        }
    }

    /// Expect a specific keyword
    fn expect_keyword(&mut self, expected: &str) -> Result<(), ParserError> {
        match self.current_token() {
            Some(token) => {
                if let TokenType::Keyword(keyword) = &token.token_type {
                    if keyword == expected {
                        self.advance();
                        return Ok(());
                    }
                }
                Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: format!("keyword {}", expected),
                    location: token.location.clone(),
                })
            }
            None => Err(ParserError::UnexpectedEof {
                expected: format!("keyword {}", expected),
            }),
        }
    }
    
    /// Consume a string literal
    fn consume_float(&mut self) -> Result<f64, ParserError> {
        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Float(value) => {
                    let float_value = *value;
                    self.advance();
                    Ok(float_value)
                }
                TokenType::Integer(value) => {
                    // Allow integers to be used as floats
                    let float_value = *value as f64;
                    self.advance();
                    Ok(float_value)
                }
                _ => Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: "float value".to_string(),
                    location: token.location.clone(),
                }),
            },
            None => Err(ParserError::UnexpectedEof {
                expected: "float value".to_string(),
            }),
        }
    }
    
    fn consume_boolean(&mut self) -> Result<bool, ParserError> {
        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Boolean(value) => {
                    let bool_value = *value;
                    self.advance();
                    Ok(bool_value)
                }
                _ => Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: "boolean value".to_string(),
                    location: token.location.clone(),
                }),
            },
            None => Err(ParserError::UnexpectedEof {
                expected: "boolean value".to_string(),
            }),
        }
    }
    
    fn consume_string(&mut self) -> Result<String, ParserError> {
        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::String(value) => {
                    let string = value.clone();
                    self.advance();
                    Ok(string)
                }
                _ => Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: "string literal".to_string(),
                    location: token.location.clone(),
                }),
            },
            None => Err(ParserError::UnexpectedEof {
                expected: "string literal".to_string(),
            }),
        }
    }

    /// Parse a boolean value
    fn parse_boolean(&mut self) -> Result<bool, ParserError> {
        match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Keyword(keyword) => {
                    if keyword == "true" {
                        self.advance();
                        Ok(true)
                    } else if keyword == "false" {
                        self.advance();
                        Ok(false)
                    } else {
                        Err(ParserError::UnexpectedToken {
                            found: keyword.clone(),
                            expected: "boolean value (true or false)".to_string(),
                            location: token.location.clone(),
                        })
                    }
                }
                TokenType::Identifier(identifier) => {
                    if identifier == "true" {
                        self.advance();
                        Ok(true)
                    } else if identifier == "false" {
                        self.advance();
                        Ok(false)
                    } else {
                        Err(ParserError::UnexpectedToken {
                            found: identifier.clone(),
                            expected: "boolean value (true or false)".to_string(),
                            location: token.location.clone(),
                        })
                    }
                }
                _ => Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: "boolean value".to_string(),
                    location: token.location.clone(),
                }),
            },
            None => Err(ParserError::UnexpectedEof {
                expected: "boolean value".to_string(),
            }),
        }
    }

    /// Record an error and enter recovery mode
    fn record_error(&mut self, error: ParserError) {
        self.errors.push(error);
        self.recovery_mode = true;
    }

    /// Synchronize to the next recovery point (typically a closing parenthesis)
    fn synchronize(&mut self) {
        let mut paren_depth = 0;
        let start_position = self.position;
        
        while !self.is_at_end() {
            if let Some(token) = self.current_token() {
                match &token.token_type {
                    TokenType::LeftParen => {
                        paren_depth += 1;
                    }
                    TokenType::RightParen => {
                        if paren_depth == 0 {
                            // Found a closing paren at the same level, consume it and exit
                            self.advance();
                            self.recovery_mode = false;
                            return;
                        }
                        paren_depth -= 1;
                    }
                    _ => {}
                }
            }
            self.advance();
            
            // Prevent infinite loops - if we've advanced more than 100 tokens, stop
            if self.position > start_position && self.position - start_position > 100 {
                eprintln!("Error recovery: Exceeded maximum token lookahead");
                break;
            }
        }
        self.recovery_mode = false;
    }

    /// Get all accumulated errors
    pub fn get_errors(&self) -> &[ParserError] {
        &self.errors
    }

    /// Check if any errors were encountered
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Parse a complete program (collection of modules)
    pub fn parse_program(&mut self) -> Result<Program, ParserError> {
        let start_location = self.current_token()
            .map(|t| t.location.clone())
            .unwrap_or_else(SourceLocation::unknown);

        let mut modules = Vec::new();
        
        while !self.is_at_end() {
            // Skip comments
            if let Some(token) = self.current_token() {
                if matches!(token.token_type, TokenType::Comment(_)) {
                    self.advance();
                    continue;
                }
            }
            
            match self.parse_module() {
                Ok(module) => modules.push(module),
                Err(error) => {
                    self.record_error(error);
                    self.synchronize();
                    // Continue parsing after error
                }
            }
        }

        // If we have errors, return the first one
        if self.has_errors() {
            return Err(self.errors[0].clone());
        }

        Ok(Program {
            modules,
            source_location: start_location,
        })
    }

    /// Parse a module definition
    pub fn parse_module(&mut self) -> Result<Module, ParserError> {
        let start_location = self.consume_left_paren()?;
        self.consume_keyword(KeywordType::DefineModule)?;

        let mut name = None;
        let mut intent = None;
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut type_definitions = Vec::new();
        let mut constant_declarations = Vec::new();
        let mut function_definitions = Vec::new();
        let mut external_functions = Vec::new();

        // Parse module fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }

            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "module field keyword".to_string(),
                })?;

            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Name) => {
                            self.advance(); // consume NAME keyword
                            name = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::Intent) => {
                            self.advance(); // consume INTENT keyword
                            intent = Some(self.consume_string()?);
                        }
                        Some(KeywordType::Content) => {
                            self.advance(); // consume CONTENT keyword
                            eprintln!("Parser: Entering CONTENT block");
                            // Parse content items
                            while let Some(token) = self.current_token() {
                                if matches!(token.token_type, TokenType::RightParen) {
                                    eprintln!("Parser: Found right paren, exiting CONTENT block");
                                    break;
                                }
                                
                                // Skip comments
                                if matches!(token.token_type, TokenType::Comment(_)) {
                                    self.advance();
                                    continue;
                                }
                                
                                // Parse each content item
                                if let Some(content_token) = self.current_token() {
                                    if matches!(content_token.token_type, TokenType::LeftParen) {
                                        match self.parse_module_content_item() {
                                            Ok(content_item) => {
                                                match content_item {
                                                    ModuleContent::Import(import) => imports.push(import),
                                                    ModuleContent::Export(export) => exports.push(export),
                                                    ModuleContent::TypeDefinition(type_def) => {
                                type_definitions.push(type_def);
                            }
                                                    ModuleContent::ConstantDeclaration(const_decl) => constant_declarations.push(const_decl),
                                                    ModuleContent::FunctionDefinition(func_def) => {
                                                        function_definitions.push(*func_def);
                                                    }
                                                    ModuleContent::ExternalFunction(ext_func) => {
                                                        external_functions.push(ext_func);
                                                    }
                                                }
                                            }
                                            Err(error) => {
                                                // Record error and synchronize
                                                eprintln!("Parser: Error parsing module content: {:?}", error);
                                                self.record_error(error);
                                                self.synchronize();
                                            }
                                        }
                                    } else {
                                        let err = ParserError::UnexpectedToken {
                                            found: format!("{:?}", content_token.token_type),
                                            expected: "left parenthesis for content item".to_string(),
                                            location: content_token.location.clone(),
                                        };
                                        self.record_error(err);
                                        self.advance(); // Skip problematic token
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "module field keyword (NAME, INTENT, CONTENT)".to_string(),
                                location: field_keyword.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", field_keyword.token_type),
                        expected: "module field keyword".to_string(),
                        location: field_keyword.location.clone(),
                    });
                }
            }

            self.consume_right_paren()?; // Close field
        }

        self.consume_right_paren()?; // Close module

        let name = name.ok_or_else(|| ParserError::MissingRequiredField {
            field: "NAME".to_string(),
            construct: "DEFINE_MODULE".to_string(),
            location: start_location.clone(),
        })?;

        Ok(Module {
            name,
            intent,
            imports,
            exports,
            type_definitions,
            constant_declarations,
            function_definitions,
            external_functions,
            source_location: start_location,
        })
    }

    /// Parse a module content item
    fn parse_module_content_item(&mut self) -> Result<ModuleContent, ParserError> {
        self.consume_left_paren()?;
        
        let keyword_token = self.current_token()
            .ok_or_else(|| ParserError::UnexpectedEof {
                expected: "module content keyword".to_string(),
            })?;

        match &keyword_token.token_type {
            TokenType::Keyword(keyword) => {
                match self.keywords.get(keyword) {
                    Some(KeywordType::ImportModule) => {
                        let import = self.parse_import_statement()?;
                        self.consume_right_paren()?;
                        Ok(ModuleContent::Import(import))
                    }
                    Some(KeywordType::ExportsFunction) | 
                    Some(KeywordType::ExportsType) | 
                    Some(KeywordType::ExportsConstant) => {
                        let export = self.parse_export_statement()?;
                        self.consume_right_paren()?;
                        Ok(ModuleContent::Export(export))
                    }
                    Some(KeywordType::DefineStructuredType) |
                    Some(KeywordType::DefineEnumerationType) |
                    Some(KeywordType::DefineTypeAlias) => {
                        eprintln!("Parser: Found type definition keyword: {:?}", keyword);
                        let type_def = self.parse_type_definition()?;
                        eprintln!("Parser: Finished parsing type definition, consuming right paren");
                        self.consume_right_paren()?;
                        Ok(ModuleContent::TypeDefinition(type_def))
                    }
                    Some(KeywordType::DeclareConstant) => {
                        let const_decl = self.parse_constant_declaration()?;
                        self.consume_right_paren()?;
                        Ok(ModuleContent::ConstantDeclaration(const_decl))
                    }
                    Some(KeywordType::DefineFunction) => {
                        eprintln!("Parser: Found DEFINE_FUNCTION keyword");
                        let func_def = self.parse_function_definition()?;
                        eprintln!("Parser: Parsed function: {}", func_def.name.name);
                        self.consume_right_paren()?;
                        Ok(ModuleContent::FunctionDefinition(Box::new(func_def)))
                    }
                    Some(KeywordType::DeclareExternalFunction) => {
                        eprintln!("Parser: Found DECLARE_EXTERNAL_FUNCTION keyword");
                        let ext_func = self.parse_external_function_declaration()?;
                        eprintln!("Parser: Parsed external function: {}", ext_func.name.name);
                        self.consume_right_paren()?;
                        Ok(ModuleContent::ExternalFunction(ext_func))
                    }
                    _ => Err(ParserError::UnexpectedToken {
                        found: keyword.clone(),
                        expected: "module content keyword".to_string(),
                        location: keyword_token.location.clone(),
                    })
                }
            }
            _ => Err(ParserError::UnexpectedToken {
                found: format!("{:?}", keyword_token.token_type),
                expected: "module content keyword".to_string(),
                location: keyword_token.location.clone(),
            })
        }
    }

    /// Parse an import statement
    fn parse_import_statement(&mut self) -> Result<ImportStatement, ParserError> {
        let start_location = self.consume_keyword(KeywordType::ImportModule)?;
        
        let mut module_name = None;
        let mut alias = None;

        // Parse import fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }

            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "import field keyword".to_string(),
                })?;

            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Name) => {
                            self.advance(); // consume NAME keyword
                            module_name = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::Value) => { // Use VALUE for alias
                            self.advance(); // consume VALUE keyword  
                            alias = Some(self.consume_identifier()?);
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "import field keyword (NAME, VALUE)".to_string(),
                                location: field_keyword.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", field_keyword.token_type),
                        expected: "import field keyword".to_string(),
                        location: field_keyword.location.clone(),
                    });
                }
            }

            self.consume_right_paren()?; // Close field
        }

        let module_name = module_name.ok_or_else(|| ParserError::MissingRequiredField {
            field: "NAME".to_string(),
            construct: "IMPORT_MODULE".to_string(),
            location: start_location.clone(),
        })?;

        Ok(ImportStatement {
            module_name,
            alias,
            source_location: start_location,
        })
    }

    /// Parse an export statement
    fn parse_export_statement(&mut self) -> Result<ExportStatement, ParserError> {
        let export_keyword = self.current_token()
            .ok_or_else(|| ParserError::UnexpectedEof {
                expected: "export keyword".to_string(),
            })?;

        let start_location = export_keyword.location.clone();
        let export_type = match &export_keyword.token_type {
            TokenType::Keyword(keyword) => {
                let keyword_clone = keyword.clone();
                match self.keywords.get(&keyword_clone) {
                    Some(KeywordType::ExportsFunction) => {
                        self.advance();
                        "function"
                    }
                    Some(KeywordType::ExportsType) => {
                        self.advance();
                        "type"
                    }
                    Some(KeywordType::ExportsConstant) => {
                        self.advance();
                        "constant"
                    }
                    _ => {
                        return Err(ParserError::UnexpectedToken {
                            found: keyword_clone,
                            expected: "export keyword (EXPORTS_FUNCTION, EXPORTS_TYPE, EXPORTS_CONSTANT)".to_string(),
                            location: start_location,
                        });
                    }
                }
            }
            _ => {
                return Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", export_keyword.token_type),
                    expected: "export keyword".to_string(),
                    location: start_location,
                });
            }
        };
        let mut name = None;

        // Parse export fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }

            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "export field keyword".to_string(),
                })?;

            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Name) => {
                            self.advance(); // consume NAME keyword
                            name = Some(self.consume_identifier()?);
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "export field keyword (NAME)".to_string(),
                                location: field_keyword.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", field_keyword.token_type),
                        expected: "export field keyword".to_string(),
                        location: field_keyword.location.clone(),
                    });
                }
            }

            self.consume_right_paren()?; // Close field
        }

        let name = name.ok_or_else(|| ParserError::MissingRequiredField {
            field: "NAME".to_string(),
            construct: format!("EXPORTS_{}", export_type.to_uppercase()),
            location: start_location.clone(),
        })?;

        match export_type {
            "function" => Ok(ExportStatement::Function {
                name,
                source_location: start_location,
            }),
            "type" => Ok(ExportStatement::Type {
                name,
                source_location: start_location,
            }),
            "constant" => Ok(ExportStatement::Constant {
                name,
                source_location: start_location,
            }),
            _ => unreachable!(),
        }
    }

    /// Parse a type definition (stub implementation)
    fn parse_type_definition(&mut self) -> Result<TypeDefinition, ParserError> {
        let start_location = self.current_token().unwrap().location.clone();
        eprintln!("Parser: Entering parse_type_definition at position {}", self.position);
        
        // Check which type definition keyword we have
        let type_def_keyword = self.current_token()
            .ok_or_else(|| ParserError::UnexpectedEof {
                expected: "type definition keyword".to_string(),
            })?;
        eprintln!("Parser: Type def keyword: {:?}", type_def_keyword.token_type);
            
        match &type_def_keyword.token_type {
            TokenType::Keyword(keyword) => {
                match self.keywords.get(keyword) {
                    Some(KeywordType::DefineStructuredType) => {
                        self.advance(); // consume DEFINE_STRUCTURED_TYPE keyword
                        self.parse_structured_type(start_location)
                    }
                    Some(KeywordType::DefineEnumerationType) => {
                        self.advance(); // consume DEFINE_ENUMERATION_TYPE keyword
                        self.parse_enumeration_type(start_location)
                    }
                    Some(KeywordType::DefineTypeAlias) => {
                        self.advance(); // consume DEFINE_TYPE_ALIAS keyword
                        // TODO: Implement parse_type_alias
                        Err(ParserError::Unimplemented {
                            feature: "Type aliases".to_string(),
                            location: start_location,
                        })
                    }
                    _ => Err(ParserError::UnexpectedToken {
                        found: keyword.clone(),
                        expected: "type definition keyword".to_string(),
                        location: type_def_keyword.location.clone(),
                    })
                }
            }
            _ => Err(ParserError::UnexpectedToken {
                found: format!("{:?}", type_def_keyword.token_type),
                expected: "type definition keyword".to_string(),
                location: type_def_keyword.location.clone(),
            })
        }
    }
    
    /// Parse a structured type definition
    fn parse_structured_type(&mut self, start_location: SourceLocation) -> Result<TypeDefinition, ParserError> {
        
        let mut name = None;
        let mut intent = None;
        let mut fields = Vec::new();
        let mut export_as = None;
        let mut generic_parameters = Vec::new();
        
        // Parse struct fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            
            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "struct field keyword".to_string(),
                })?;
            let field_location = field_keyword.location.clone();
            
            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Name) => {
                            self.advance(); // consume NAME keyword
                            name = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::Intent) => {
                            self.advance(); // consume INTENT keyword
                            intent = Some(self.consume_string()?);
                        }
                        Some(KeywordType::Field) => {
                            self.advance(); // consume FIELD keyword
                            
                            let field_name = self.consume_identifier()?;
                            let field_type = Box::new(self.parse_type_specifier()?);
                            
                            eprintln!("Parser: Added field '{}' with type", field_name.name);
                            fields.push(StructField {
                                name: field_name,
                                field_type,
                                source_location: field_location.clone(),
                            });
                        }
                        Some(KeywordType::ExportAs) => {
                            self.advance(); // consume EXPORT_AS keyword
                            export_as = Some(self.consume_string()?);
                        }
                        Some(KeywordType::GenericParameters) => {
                            self.advance(); // consume GENERIC_PARAMETERS keyword
                            
                            // Parse list of generic parameters
                            while let Some(token) = self.current_token() {
                                if matches!(token.token_type, TokenType::RightParen) {
                                    break;
                                }
                                
                                let param_location = token.location.clone();
                                self.consume_left_paren()?;
                                let param_name = self.consume_identifier()?;
                                
                                // Parse optional constraints
                                let mut constraints = Vec::new();
                                if self.peek_keyword(KeywordType::Constraints) {
                                    self.advance(); // consume CONSTRAINTS keyword
                                    
                                    while let Some(token) = self.current_token() {
                                        if matches!(token.token_type, TokenType::RightParen) {
                                            break;
                                        }
                                        // For now, skip type constraints
                                        // TODO: Implement parse_type_constraint
                                        self.advance();
                                    }
                                }
                                
                                generic_parameters.push(GenericParameter {
                                    name: param_name,
                                    constraints,
                                    default_type: None, // TODO: Parse default types
                                    source_location: param_location,
                                });
                                
                                self.consume_right_paren()?; // Close generic parameter
                            }
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "struct field keyword (NAME, INTENT, FIELD, EXPORT_AS, GENERIC_PARAMETERS)".to_string(),
                                location: field_keyword.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", field_keyword.token_type),
                        expected: "struct field keyword".to_string(),
                        location: field_keyword.location.clone(),
                    });
                }
            }
            
            self.consume_right_paren()?; // Close field
        }
        
        let name = name.ok_or_else(|| ParserError::MissingRequiredField {
            field: "NAME".to_string(),
            construct: "DEFINE_STRUCTURED_TYPE".to_string(),
            location: start_location.clone(),
        })?;
        
        if fields.is_empty() {
            return Err(ParserError::MissingRequiredField {
                field: "FIELD".to_string(),
                construct: "DEFINE_STRUCTURED_TYPE".to_string(),
                location: start_location.clone(),
            });
        }
        
        eprintln!("Parser: Creating struct '{}' with {} fields", name.name, fields.len());
        Ok(TypeDefinition::Structured {
            name,
            intent,
            generic_parameters,
            fields,
            export_as,
            source_location: start_location,
        })
    }
    
    /// Parse an enumeration type definition
    fn parse_enumeration_type(&mut self, start_location: SourceLocation) -> Result<TypeDefinition, ParserError> {
        eprintln!("Parser: Entering parse_enumeration_type");
        let mut name = None;
        let mut intent = None;
        let mut variants = Vec::new();
        let mut generic_parameters = Vec::new();
        
        // Parse enum fields
        while let Some(token) = self.current_token() {
            eprintln!("Parser: Current token in enum: {:?}", token.token_type);
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            
            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "enum field keyword".to_string(),
                })?;
            let field_location = field_keyword.location.clone();
            
            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Name) => {
                            self.advance(); // consume NAME keyword
                            name = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::Intent) => {
                            self.advance(); // consume INTENT keyword
                            intent = Some(self.consume_string()?);
                        }
                        Some(KeywordType::Variants) => {
                            self.advance(); // consume VARIANTS keyword
                            
                            // Parse variant list
                            while let Some(token) = self.current_token() {
                                if matches!(token.token_type, TokenType::RightParen) {
                                    break;
                                }
                                
                                self.consume_left_paren()?;
                                self.consume_keyword(KeywordType::Variant)?;
                                
                                let variant_name = self.consume_identifier()?;
                                let mut associated_type = None;
                                
                                // Check for HOLDS clause
                                if let Some(token) = self.current_token() {
                                    if matches!(token.token_type, TokenType::LeftParen) {
                                        self.consume_left_paren()?;
                                        self.consume_keyword(KeywordType::Holds)?;
                                        associated_type = Some(Box::new(self.parse_type_specifier()?));
                                        self.consume_right_paren()?;
                                    }
                                }
                                
                                variants.push(EnumVariant {
                                    name: variant_name,
                                    associated_type,
                                    source_location: field_location.clone(),
                                });
                                
                                self.consume_right_paren()?; // Close variant
                            }
                        }
                        Some(KeywordType::GenericParameters) => {
                            self.advance(); // consume GENERIC_PARAMETERS keyword
                            
                            while let Some(token) = self.current_token() {
                                if matches!(token.token_type, TokenType::RightParen) {
                                    break;
                                }
                                
                                self.consume_left_paren()?;
                                self.consume_keyword(KeywordType::Parameter)?;
                                
                                let param_name = self.consume_identifier()?;
                                let param_location = param_name.source_location.clone();
                                
                                // Parse optional constraints
                                let mut constraints = Vec::new();
                                if self.peek_keyword(KeywordType::Constraints) {
                                    self.advance(); // consume CONSTRAINTS keyword
                                    
                                    while let Some(token) = self.current_token() {
                                        if matches!(token.token_type, TokenType::RightParen) {
                                            break;
                                        }
                                        // For now, skip type constraints
                                        // TODO: Implement parse_type_constraint
                                        self.advance();
                                    }
                                }
                                
                                generic_parameters.push(GenericParameter {
                                    name: param_name,
                                    constraints,
                                    default_type: None, // TODO: Parse default types
                                    source_location: param_location,
                                });
                                
                                self.consume_right_paren()?; // Close generic parameter
                            }
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "enum field keyword (NAME, INTENT, VARIANTS, GENERIC_PARAMETERS)".to_string(),
                                location: field_keyword.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", field_keyword.token_type),
                        expected: "enum field keyword".to_string(),
                        location: field_keyword.location.clone(),
                    });
                }
            }
            
            self.consume_right_paren()?; // Close field
        }
        
        let name = name.ok_or_else(|| ParserError::MissingRequiredField {
            field: "NAME".to_string(),
            construct: "DEFINE_ENUMERATION_TYPE".to_string(),
            location: start_location.clone(),
        })?;
        
        if variants.is_empty() {
            return Err(ParserError::MissingRequiredField {
                field: "VARIANTS".to_string(),
                construct: "DEFINE_ENUMERATION_TYPE".to_string(),
                location: start_location.clone(),
            });
        }
        
        Ok(TypeDefinition::Enumeration {
            name,
            intent,
            generic_parameters,
            variants,
            source_location: start_location,
        })
    }

    /// Parse a constant declaration
    fn parse_constant_declaration(&mut self) -> Result<ConstantDeclaration, ParserError> {
        let start_location = self.consume_keyword(KeywordType::DeclareConstant)?;
        
        let mut name = None;
        let mut type_spec = None;
        let mut value = None;
        let mut intent = None;

        // Parse constant fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }

            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "constant field keyword".to_string(),
                })?;

            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Name) => {
                            self.advance(); // consume NAME keyword
                            name = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::Type) => {
                            self.advance(); // consume TYPE keyword
                            type_spec = Some(Box::new(self.parse_type_specifier()?));
                        }
                        Some(KeywordType::Value) => {
                            self.advance(); // consume VALUE keyword
                            value = Some(Box::new(self.parse_expression()?));
                        }
                        Some(KeywordType::Intent) => {
                            self.advance(); // consume INTENT keyword
                            intent = Some(self.consume_string()?);
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "constant field keyword (NAME, TYPE, VALUE, INTENT)".to_string(),
                                location: field_keyword.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", field_keyword.token_type),
                        expected: "constant field keyword".to_string(),
                        location: field_keyword.location.clone(),
                    });
                }
            }

            self.consume_right_paren()?; // Close field
        }

        let name = name.ok_or_else(|| ParserError::MissingRequiredField {
            field: "NAME".to_string(),
            construct: "DECLARE_CONSTANT".to_string(),
            location: start_location.clone(),
        })?;

        let type_spec = type_spec.ok_or_else(|| ParserError::MissingRequiredField {
            field: "TYPE".to_string(),
            construct: "DECLARE_CONSTANT".to_string(),
            location: start_location.clone(),
        })?;

        let value = value.ok_or_else(|| ParserError::MissingRequiredField {
            field: "VALUE".to_string(),
            construct: "DECLARE_CONSTANT".to_string(),
            location: start_location.clone(),
        })?;

        Ok(ConstantDeclaration {
            name,
            type_spec,
            value,
            intent,
            source_location: start_location,
        })
    }

    /// Parse an external function declaration
    fn parse_external_function_declaration(&mut self) -> Result<ExternalFunction, ParserError> {
        let start_location = self.consume_keyword(KeywordType::DeclareExternalFunction)?;
        
        let mut name = None;
        let mut library = None;
        let mut symbol = None;
        let mut parameters = Vec::new();
        let mut return_type = None;
        let mut calling_convention = CallingConvention::C;
        let mut thread_safe = true;
        let mut may_block = false;
        let mut variadic = false;
        
        // Parse fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            
            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "external function field keyword".to_string(),
                })?;
            
            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Name) => {
                            self.advance(); // consume NAME
                            name = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::Library) => {
                            self.advance(); // consume LIBRARY
                            library = Some(self.consume_string()?);
                        }
                        Some(KeywordType::Symbol) => {
                            self.advance(); // consume SYMBOL
                            symbol = Some(self.consume_string()?);
                        }
                        Some(KeywordType::AcceptsParameter) | Some(KeywordType::Param) => {
                            self.advance(); // consume ACCEPTS_PARAMETER or PARAM
                            parameters.push(self.parse_parameter()?);
                        }
                        Some(KeywordType::Returns) => {
                            self.advance(); // consume RETURNS
                            return_type = Some(self.parse_type_specifier()?);
                        }
                        Some(KeywordType::CallingConvention) | Some(KeywordType::Convention) => {
                            self.advance(); // consume CALLING_CONVENTION or CONVENTION
                            // Accept either string or identifier
                            let convention_str = {
                                let token = self.current_token()
                                    .ok_or_else(|| ParserError::UnexpectedEof {
                                        expected: "calling convention".to_string(),
                                    })?;
                                
                                let result = match &token.token_type {
                                    TokenType::String(s) => s.clone(),
                                    TokenType::Identifier(s) => s.clone(),
                                    _ => {
                                        return Err(ParserError::UnexpectedToken {
                                            found: format!("{:?}", token.token_type),
                                            expected: "calling convention (string or identifier)".to_string(),
                                            location: token.location.clone(),
                                        });
                                    }
                                };
                                self.advance();
                                result
                            };
                            calling_convention = match convention_str.as_str() {
                                "C" => CallingConvention::C,
                                "FAST" => CallingConvention::FastCall,
                                "STDCALL" => CallingConvention::StdCall,
                                "SYSTEM" => CallingConvention::System,
                                _ => CallingConvention::C, // Default to C
                            };
                        }
                        Some(KeywordType::ThreadSafe) => {
                            self.advance(); // consume THREAD_SAFE
                            thread_safe = self.parse_boolean()?;
                        }
                        Some(KeywordType::MayBlock) => {
                            self.advance(); // consume MAY_BLOCK
                            may_block = self.parse_boolean()?;
                        }
                        Some(KeywordType::Variadic) => {
                            self.advance(); // consume VARIADIC
                            variadic = self.parse_boolean()?;
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "external function field keyword".to_string(),
                                location: field_keyword.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", field_keyword.token_type),
                        expected: "external function field keyword".to_string(),
                        location: field_keyword.location.clone(),
                    });
                }
            }
            
            self.consume_right_paren()?; // Close field
        }
        
        let name = name.ok_or_else(|| ParserError::MissingRequiredField {
            field: "NAME".to_string(),
            construct: "DECLARE_EXTERNAL_FUNCTION".to_string(),
            location: start_location.clone(),
        })?;
        
        // Library is optional - defaults to standard C library
        let library = library.unwrap_or_else(|| "libc".to_string());
        
        let return_type = return_type.ok_or_else(|| ParserError::MissingRequiredField {
            field: "RETURNS".to_string(),
            construct: "DECLARE_EXTERNAL_FUNCTION".to_string(),
            location: start_location.clone(),
        })?;
        
        Ok(ExternalFunction {
            name,
            library,
            symbol,
            parameters,
            return_type: Box::new(return_type),
            calling_convention,
            thread_safe,
            may_block,
            variadic,
            ownership_info: None,
            source_location: start_location,
        })
    }

    /// Parse a function definition (stub implementation)
    fn parse_function_definition(&mut self) -> Result<Function, ParserError> {
        eprintln!("Parser: Entering parse_function_definition");
        let start_location = self.current_token().unwrap().location.clone();
        self.advance(); // consume DEFINE_FUNCTION
        
        let mut name = None;
        let mut intent = None;
        let mut parameters = Vec::new();
        let mut return_type = None;
        let mut body = None;
        let mut metadata = FunctionMetadata {
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            invariants: Vec::new(),
            algorithm_hint: None,
            performance_expectation: None,
            complexity_expectation: None,
            throws_exceptions: Vec::new(),
            thread_safe: None,
            may_block: None,
        };
        
        // Parse function fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            if matches!(token.token_type, TokenType::Eof) {
                return Err(ParserError::UnexpectedEof {
                    expected: "closing parenthesis for DEFINE_FUNCTION".to_string(),
                });
            }
            
            // Skip comments
            if matches!(token.token_type, TokenType::Comment(_)) {
                self.advance();
                continue;
            }
            
            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "function field keyword".to_string(),
                })?;
            
            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Name) => {
                            self.advance(); // consume NAME
                            name = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::Intent) => {
                            self.advance(); // consume INTENT
                            intent = Some(self.consume_string()?);
                        }
                        Some(KeywordType::AcceptsParameter) | Some(KeywordType::Param) => {
                            self.advance(); // consume ACCEPTS_PARAMETER or PARAM
                            parameters.push(self.parse_parameter()?);
                        }
                        Some(KeywordType::Returns) => {
                            self.advance(); // consume RETURNS
                            return_type = Some(self.parse_type_specifier()?);
                        }
                        Some(KeywordType::Body) => {
                            self.advance(); // consume BODY
                            body = Some(self.parse_function_body()?);
                        }
                        Some(KeywordType::Precondition) => {
                            self.advance(); // consume PRECONDITION
                            metadata.preconditions.push(self.parse_contract_assertion()?);
                        }
                        Some(KeywordType::Postcondition) => {
                            self.advance(); // consume POSTCONDITION
                            metadata.postconditions.push(self.parse_contract_assertion()?);
                        }
                        Some(KeywordType::Invariant) => {
                            self.advance(); // consume INVARIANT
                            metadata.invariants.push(self.parse_contract_assertion()?);
                        }
                        Some(KeywordType::AlgorithmHint) => {
                            self.advance(); // consume ALGORITHM_HINT
                            metadata.algorithm_hint = Some(self.consume_string()?);
                        }
                        Some(KeywordType::PerformanceExpectation) => {
                            self.advance(); // consume PERFORMANCE_EXPECTATION
                            metadata.performance_expectation = Some(self.parse_performance_expectation()?);
                        }
                        Some(KeywordType::ComplexityExpectation) => {
                            self.advance(); // consume COMPLEXITY_EXPECTATION
                            metadata.complexity_expectation = Some(self.parse_complexity_expectation()?);
                        }
                        Some(KeywordType::ThreadSafe) => {
                            self.advance(); // consume THREAD_SAFE
                            metadata.thread_safe = Some(self.consume_boolean()?);
                        }
                        Some(KeywordType::MayBlock) => {
                            self.advance(); // consume MAY_BLOCK
                            metadata.may_block = Some(self.consume_boolean()?);
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "function field keyword".to_string(),
                                location: field_keyword.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", field_keyword.token_type),
                        expected: "function field keyword".to_string(),
                        location: field_keyword.location.clone(),
                    });
                }
            }
            
            self.consume_right_paren()?;
        }
        
        // Validate required fields
        let name = name.ok_or_else(|| ParserError::MissingRequiredField {
            field: "NAME".to_string(),
            construct: "DEFINE_FUNCTION".to_string(),
            location: start_location.clone(),
        })?;
        
        let return_type = return_type.unwrap_or_else(|| TypeSpecifier::Primitive {
            type_name: PrimitiveType::Void,
            source_location: start_location.clone(),
        });
        
        let body = body.unwrap_or_else(|| Block {
            statements: Vec::new(),
            source_location: start_location.clone(),
        });
        
        Ok(Function {
            name,
            intent,
            generic_parameters: Vec::new(),
            parameters,
            return_type: Box::new(return_type),
            metadata,
            body,
            export_info: None,
            source_location: start_location,
        })
    }
    
    /// Parse a function parameter
    fn parse_parameter(&mut self) -> Result<Parameter, ParserError> {
        let start_location = self.current_token()
            .map(|t| t.location.clone())
            .unwrap_or_else(SourceLocation::unknown);
        
        self.consume_left_paren()?;
        
        // Parse NAME field
        self.expect_keyword("NAME")?;
        let name = self.consume_string()?;
        self.consume_right_paren()?;
        
        // Parse TYPE field
        self.consume_left_paren()?;
        self.expect_keyword("TYPE")?;
        let param_type = self.parse_type_specifier()?;
        self.consume_right_paren()?;
        
        Ok(Parameter {
            name: Identifier::new(name, start_location.clone()),
            param_type: Box::new(param_type),
            intent: None,
            constraint: None,
            passing_mode: PassingMode::ByValue,
            source_location: start_location,
        })
    }
    
    /// Parse a function body
    fn parse_function_body(&mut self) -> Result<Block, ParserError> {
        // Function body is just a block of statements
        self.parse_block()
    }

    /// Parse a type specifier
    fn parse_type_specifier(&mut self) -> Result<TypeSpecifier, ParserError> {
        // Check for ownership annotations first
        let (ownership_kind, ownership_location) = self.parse_ownership_annotation()?;
        
        let token = self.current_token()
            .ok_or_else(|| ParserError::UnexpectedEof {
                expected: "type specifier".to_string(),
            })?;

        let base_type = match &token.token_type {
            TokenType::Keyword(keyword) => {
                let location = token.location.clone();
                match self.keywords.get(keyword) {
                    Some(KeywordType::Integer) => {
                        self.advance();
                        Ok(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Integer,
                            source_location: location,
                        })
                    }
                    Some(KeywordType::Float) => {
                        self.advance();
                        Ok(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Float,
                            source_location: location,
                        })
                    }
                    Some(KeywordType::String) => {
                        self.advance();
                        Ok(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::String,
                            source_location: location,
                        })
                    }
                    Some(KeywordType::Char) => {
                        self.advance();
                        Ok(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Char,
                            source_location: location,
                        })
                    }
                    Some(KeywordType::Boolean) => {
                        self.advance();
                        Ok(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Boolean,
                            source_location: location,
                        })
                    }
                    Some(KeywordType::Void) => {
                        self.advance();
                        Ok(TypeSpecifier::Primitive {
                            type_name: PrimitiveType::Void,
                            source_location: location,
                        })
                    }
                    _ => {
                        Err(ParserError::UnexpectedToken {
                            found: keyword.clone(),
                            expected: "type keyword".to_string(),
                            location,
                        })
                    }
                }
            }
            TokenType::Identifier(_) => {
                let identifier = self.consume_identifier()?;
                Ok(TypeSpecifier::Named {
                    name: identifier.clone(),
                    source_location: identifier.source_location,
                })
            }
            TokenType::LeftParen => {
                // Complex type (ARRAY_OF_TYPE, MAP_FROM_TYPE_TO_TYPE, etc.)
                self.parse_complex_type_specifier()
            }
            _ => {
                Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: "type specifier".to_string(),
                    location: token.location.clone(),
                })
            }
        }?;
        
        // If we have an ownership annotation, wrap the base type
        if let Some(ownership) = ownership_kind {
            Ok(TypeSpecifier::Owned {
                base_type: Box::new(base_type),
                ownership,
                source_location: ownership_location.unwrap(),
            })
        } else {
            Ok(base_type)
        }
    }
    
    /// Parse ownership annotations (^, &, &mut, ~)
    fn parse_ownership_annotation(&mut self) -> Result<(Option<OwnershipKind>, Option<SourceLocation>), ParserError> {
        let token = match self.current_token() {
            Some(token) => token,
            None => return Ok((None, None)),
        };
        
        match &token.token_type {
            TokenType::Caret => {
                let location = token.location.clone();
                self.advance();
                Ok((Some(OwnershipKind::Owned), Some(location)))
            }
            TokenType::Ampersand => {
                let location = token.location.clone();
                self.advance();
                
                // Check if followed by 'mut' keyword
                if let Some(next_token) = self.current_token() {
                    if let TokenType::Keyword(keyword) = &next_token.token_type {
                        if keyword == "mut" {
                            self.advance(); // consume 'mut'
                            return Ok((Some(OwnershipKind::BorrowedMut), Some(location)));
                        }
                    }
                }
                
                Ok((Some(OwnershipKind::Borrowed), Some(location)))
            }
            TokenType::Tilde => {
                let location = token.location.clone();
                self.advance();
                Ok((Some(OwnershipKind::Shared), Some(location)))
            }
            _ => Ok((None, None)),
        }
    }

    /// Parse complex type specifiers (arrays, maps, pointers, functions)
    fn parse_complex_type_specifier(&mut self) -> Result<TypeSpecifier, ParserError> {
        let start_location = self.consume_left_paren()?;
        
        let keyword_token = self.current_token()
            .ok_or_else(|| ParserError::UnexpectedEof {
                expected: "type keyword".to_string(),
            })?;

        let result = match &keyword_token.token_type {
            TokenType::Keyword(keyword) => {
                match self.keywords.get(keyword) {
                    Some(KeywordType::ArrayOfType) => {
                        self.advance(); // consume ARRAY_OF_TYPE
                        let element_type = Box::new(self.parse_type_specifier()?);
                        
                        // TODO: Parse optional size expression
                        
                        self.consume_right_paren()?;
                        Ok(TypeSpecifier::Array {
                            element_type,
                            size: None, // For now, no size support
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::MapFromTypeToType) => {
                        self.advance(); // consume MAP_FROM_TYPE_TO_TYPE
                        let key_type = Box::new(self.parse_type_specifier()?);
                        let value_type = Box::new(self.parse_type_specifier()?);
                        
                        self.consume_right_paren()?;
                        Ok(TypeSpecifier::Map {
                            key_type,
                            value_type,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::PointerTo) => {
                        self.advance(); // consume POINTER_TO
                        let target_type = Box::new(self.parse_type_specifier()?);
                        
                        self.consume_right_paren()?;
                        Ok(TypeSpecifier::Pointer {
                            target_type,
                            is_mutable: false, // Default to immutable
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::FunctionType) => {
                        self.advance(); // consume FUNCTION_TYPE
                        
                        // Parse parameter types
                        let mut parameter_types = Vec::new();
                        
                        // Expect (PARAMETERS type1 type2 ...)
                        self.consume_left_paren()?;
                        let params_token = self.current_token()
                            .ok_or_else(|| ParserError::UnexpectedEof {
                                expected: "PARAMETERS keyword".to_string(),
                            })?;
                        
                        if let TokenType::Identifier(s) = &params_token.token_type {
                            if s == "PARAMETERS" {
                                self.advance(); // consume PARAMETERS
                                
                                // Parse parameter types until we hit a right paren
                                while let Some(token) = self.current_token() {
                                    if matches!(token.token_type, TokenType::RightParen) {
                                        break;
                                    }
                                    parameter_types.push(Box::new(self.parse_type_specifier()?));
                                }
                            }
                        }
                        self.consume_right_paren()?;
                        
                        // Expect (RETURNS type)
                        self.consume_left_paren()?;
                        let returns_token = self.current_token()
                            .ok_or_else(|| ParserError::UnexpectedEof {
                                expected: "RETURNS keyword".to_string(),
                            })?;
                        
                        let return_type = if let TokenType::Keyword(keyword) = &returns_token.token_type {
                            if self.keywords.get(keyword) == Some(&KeywordType::Returns) {
                                self.advance(); // consume RETURNS
                                Box::new(self.parse_type_specifier()?)
                            } else {
                                return Err(ParserError::UnexpectedToken {
                                    found: keyword.clone(),
                                    expected: "RETURNS keyword".to_string(),
                                    location: returns_token.location.clone(),
                                });
                            }
                        } else {
                            return Err(ParserError::UnexpectedToken {
                                found: format!("{:?}", returns_token.token_type),
                                expected: "RETURNS keyword".to_string(),
                                location: returns_token.location.clone(),
                            });
                        };
                        
                        self.consume_right_paren()?;
                        self.consume_right_paren()?; // Close the FUNCTION_TYPE paren
                        
                        Ok(TypeSpecifier::Function {
                            parameter_types,
                            return_type,
                            source_location: start_location,
                        })
                    }
                    _ => {
                        Err(ParserError::UnexpectedToken {
                            found: keyword.clone(),
                            expected: "complex type keyword".to_string(),
                            location: keyword_token.location.clone(),
                        })
                    }
                }
            }
            _ => {
                Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", keyword_token.token_type),
                    expected: "type keyword".to_string(),
                    location: keyword_token.location.clone(),
                })
            }
        };

        result
    }

    /// Parse an expression (basic implementation)
    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        let token = self.current_token()
            .ok_or_else(|| ParserError::UnexpectedEof {
                expected: "expression".to_string(),
            })?;

        let location = token.location.clone();
        match &token.token_type {
            TokenType::Integer(value) => {
                let int_value = *value;
                self.advance();
                Ok(Expression::IntegerLiteral {
                    value: int_value,
                    source_location: location,
                })
            }
            TokenType::Float(value) => {
                let float_value = *value;
                self.advance();
                Ok(Expression::FloatLiteral {
                    value: float_value,
                    source_location: location,
                })
            }
            TokenType::String(value) => {
                let string_value = value.clone();
                self.advance();
                Ok(Expression::StringLiteral {
                    value: string_value,
                    source_location: location,
                })
            }
            TokenType::Character(value) => {
                let char_value = *value;
                self.advance();
                Ok(Expression::CharacterLiteral {
                    value: char_value,
                    source_location: location,
                })
            }
            TokenType::Boolean(value) => {
                let bool_value = *value;
                self.advance();
                Ok(Expression::BooleanLiteral {
                    value: bool_value,
                    source_location: location,
                })
            }
            TokenType::Identifier(ident) => {
                // Check for special identifiers that represent literals
                if ident == "STRING_LITERAL" {
                    self.advance(); // consume STRING_LITERAL
                    // Next token should be the actual string
                    if let Some(token) = self.current_token() {
                        if let TokenType::String(value) = &token.token_type {
                            let string_value = value.clone();
                            self.advance();
                            return Ok(Expression::StringLiteral {
                                value: string_value,
                                source_location: location,
                            });
                        }
                    }
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", self.current_token().map(|t| &t.token_type)),
                        expected: "string value after STRING_LITERAL".to_string(),
                        location,
                    });
                }
                
                let identifier = self.consume_identifier()?;
                Ok(Expression::Variable {
                    name: identifier.clone(),
                    source_location: identifier.source_location,
                })
            }
            TokenType::LeftParen => {
                // Complex expression (function calls, arithmetic, etc.)
                self.parse_complex_expression()
            }
            _ => {
                Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: "expression".to_string(),
                    location: token.location.clone(),
                })
            }
        }
    }

    /// Parse complex expressions (function calls, arithmetic operations, etc.)
    fn parse_complex_expression(&mut self) -> Result<Expression, ParserError> {
        let start_location = self.consume_left_paren()?;
        
        let keyword_token = self.current_token()
            .ok_or_else(|| ParserError::UnexpectedEof {
                expected: "expression keyword".to_string(),
            })?;

        // Try to get keyword from either Keyword or Identifier token types
        let keyword_str = match &keyword_token.token_type {
            TokenType::Keyword(keyword) => Some(keyword.clone()),
            TokenType::Identifier(ident) => Some(ident.clone()),
            _ => None,
        };
        
        let result = if let Some(keyword) = keyword_str {
            match self.keywords.get(&keyword) {
                    Some(KeywordType::ExpressionAdd) => {
                        self.advance(); // consume EXPRESSION_ADD
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::Add {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::ExpressionSubtract) => {
                        self.advance(); // consume EXPRESSION_SUBTRACT
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::Subtract {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::ExpressionMultiply) => {
                        self.advance(); // consume EXPRESSION_MULTIPLY
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::Multiply {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::ExpressionDivide) => {
                        self.advance(); // consume EXPRESSION_DIVIDE
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::Divide {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::ExpressionModulo) => {
                        self.advance(); // consume EXPRESSION_MODULO
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::Modulo {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::CallFunction) => {
                        self.advance(); // consume CALL_FUNCTION
                        self.parse_function_call_expression(start_location)
                    }
                    Some(KeywordType::PredicateEquals) => {
                        self.advance(); // consume PREDICATE_EQUALS
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::Equals {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::PredicateNotEquals) => {
                        self.advance(); // consume PREDICATE_NOT_EQUALS
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::NotEquals {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::PredicateLessThan) => {
                        self.advance(); // consume PREDICATE_LESS_THAN
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::LessThan {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::PredicateGreaterThan) => {
                        self.advance(); // consume PREDICATE_GREATER_THAN
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::GreaterThan {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::PredicateLessThanOrEqualTo) => {
                        self.advance(); // consume PREDICATE_LESS_THAN_OR_EQUAL_TO
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::LessThanOrEqual {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::StringConcat) => {
                        self.advance(); // consume STRING_CONCAT
                        let mut operands = Vec::new();
                        // Parse multiple operands for concatenation
                        while let Some(token) = self.current_token() {
                            if matches!(token.token_type, TokenType::RightParen) {
                                break;
                            }
                            operands.push(self.parse_expression()?);
                        }
                        if operands.len() < 2 {
                            return Err(ParserError::MalformedSExpression {
                                reason: "STRING_CONCAT requires at least 2 operands".to_string(),
                                location: start_location,
                            });
                        }
                        self.consume_right_paren()?;
                        Ok(Expression::StringConcat {
                            operands,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::StringLength) => {
                        self.advance(); // consume STRING_LENGTH
                        let string = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::StringLength {
                            string,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::StringCharAt) => {
                        self.advance(); // consume STRING_CHAR_AT
                        let string = Box::new(self.parse_expression()?);
                        let index = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::StringCharAt {
                            string,
                            index,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::Substring) => {
                        self.advance(); // consume SUBSTRING
                        let string = Box::new(self.parse_expression()?);
                        let start_index = Box::new(self.parse_expression()?);
                        let length = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::Substring {
                            string,
                            start_index,
                            length,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::StringEquals) => {
                        self.advance(); // consume STRING_EQUALS
                        let left = Box::new(self.parse_expression()?);
                        let right = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::StringEquals {
                            left,
                            right,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::StringContains) => {
                        self.advance(); // consume STRING_CONTAINS
                        let haystack = Box::new(self.parse_expression()?);
                        let needle = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::StringContains {
                            haystack,
                            needle,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::ArrayLiteral) => {
                        self.advance(); // consume ARRAY_LITERAL
                        let mut elements = Vec::new();
                        // Parse array elements
                        while let Some(token) = self.current_token() {
                            if matches!(token.token_type, TokenType::RightParen) {
                                break;
                            }
                            elements.push(Box::new(self.parse_expression()?));
                        }
                        self.consume_right_paren()?;
                        // Infer element type from first element or default to INTEGER
                        let element_type = if elements.is_empty() {
                            Box::new(TypeSpecifier::Primitive {
                                type_name: PrimitiveType::Integer,
                                source_location: start_location.clone(),
                            })
                        } else {
                            // For simplicity, assume INTEGER type
                            // In a full implementation, we'd infer from the first element
                            Box::new(TypeSpecifier::Primitive {
                                type_name: PrimitiveType::Integer,
                                source_location: start_location.clone(),
                            })
                        };
                        Ok(Expression::ArrayLiteral {
                            element_type,
                            elements,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::GetArrayElement) => {
                        self.advance(); // consume GET_ARRAY_ELEMENT
                        let array = Box::new(self.parse_expression()?);
                        let index = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::ArrayAccess {
                            array,
                            index,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::ArrayLength) => {
                        self.advance(); // consume ARRAY_LENGTH
                        let array = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::ArrayLength {
                            array,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::MapLiteral) => {
                        self.advance(); // consume MAP_LITERAL
                        
                        // Check if there are type annotations
                        let (key_type, value_type) = if self.current_token()
                            .map(|t| matches!(&t.token_type, TokenType::Identifier(s) if s == "KEY_TYPE"))
                            .unwrap_or(false) {
                            // Parse typed map literal: (MAP_LITERAL KEY_TYPE <type> VALUE_TYPE <type> ...)
                            self.advance(); // consume KEY_TYPE
                            let key_type = Box::new(self.parse_type_specifier()?);
                            
                            let value_type_id = self.consume_identifier()?;
                            if value_type_id.name != "VALUE_TYPE" {
                                return Err(ParserError::UnexpectedToken {
                                    found: format!("identifier '{}'", value_type_id.name),
                                    expected: "VALUE_TYPE".to_string(),
                                    location: value_type_id.source_location,
                                });
                            }
                            let value_type = Box::new(self.parse_type_specifier()?);
                            
                            (key_type, value_type)
                        } else {
                            // Default to INTEGER -> INTEGER for empty maps when no type specified
                            (
                                Box::new(TypeSpecifier::Primitive {
                                    type_name: PrimitiveType::Integer,
                                    source_location: start_location.clone(),
                                }),
                                Box::new(TypeSpecifier::Primitive {
                                    type_name: PrimitiveType::Integer,
                                    source_location: start_location.clone(),
                                })
                            )
                        };
                        
                        let mut entries = Vec::new();
                        
                        // Parse map entries
                        while let Some(token) = self.current_token() {
                            if matches!(token.token_type, TokenType::RightParen) {
                                break;
                            }
                            
                            // Parse (ENTRY (KEY key_expr) (VALUE value_expr))
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::Entry)?;
                            
                            // Parse key
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::Key)?;
                            let key = Box::new(self.parse_expression()?);
                            self.consume_right_paren()?;
                            
                            // Parse value
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::Value)?;
                            let value = Box::new(self.parse_expression()?);
                            self.consume_right_paren()?;
                            
                            self.consume_right_paren()?; // Close ENTRY
                            
                            entries.push(MapEntry {
                                key,
                                value,
                                source_location: self.current_token()
                                    .map(|t| t.location.clone())
                                    .unwrap_or_else(SourceLocation::unknown),
                            });
                        }
                        
                        self.consume_right_paren()?;
                        
                        Ok(Expression::MapLiteral {
                            key_type,
                            value_type,
                            entries,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::GetMapValue) => {
                        self.advance(); // consume GET_MAP_VALUE
                        let map = Box::new(self.parse_expression()?);
                        let key = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::MapAccess {
                            map,
                            key,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::MatchExpression) => {
                        self.advance(); // consume MATCH_EXPRESSION
                        self.parse_match_expression(start_location)
                    }
                    Some(KeywordType::GetFieldValue) => {
                        self.advance(); // consume GET_FIELD_VALUE
                        let object = Box::new(self.parse_expression()?);
                        let field = self.consume_identifier()?;
                        self.consume_right_paren()?;
                        Ok(Expression::FieldAccess {
                            instance: object,
                            field_name: field,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::ToString) => {
                        self.advance(); // consume TO_STRING
                        let value = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::TypeCast {
                            value,
                            target_type: Box::new(TypeSpecifier::Primitive {
                                type_name: PrimitiveType::String,
                                source_location: start_location.clone(),
                            }),
                            failure_behavior: CastFailureBehavior::ThrowException,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::ToInteger) => {
                        self.advance(); // consume TO_INTEGER
                        let value = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::TypeCast {
                            value,
                            target_type: Box::new(TypeSpecifier::Primitive {
                                type_name: PrimitiveType::Integer,
                                source_location: start_location.clone(),
                            }),
                            failure_behavior: CastFailureBehavior::ThrowException,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::ToFloat) => {
                        self.advance(); // consume TO_FLOAT
                        let value = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::TypeCast {
                            value,
                            target_type: Box::new(TypeSpecifier::Primitive {
                                type_name: PrimitiveType::Float,
                                source_location: start_location.clone(),
                            }),
                            failure_behavior: CastFailureBehavior::ThrowException,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::Construct) => {
                        self.advance(); // consume CONSTRUCT
                        let type_name = self.consume_identifier()?;
                        
                        let mut field_values = Vec::new();
                        // Parse field values
                        while let Some(token) = self.current_token() {
                            if matches!(token.token_type, TokenType::RightParen) {
                                break;
                            }
                            
                            let field_location = token.location.clone();
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::FieldValue)?;
                            let field_name = self.consume_identifier()?;
                            let value = Box::new(self.parse_expression()?);
                            self.consume_right_paren()?;
                            
                            field_values.push(FieldValue {
                                field_name,
                                value,
                                source_location: field_location,
                            });
                        }
                        
                        self.consume_right_paren()?;
                        Ok(Expression::StructConstruct {
                            type_name,
                            field_values,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::CastToType) => {
                        self.advance(); // consume CAST_TO_TYPE
                        let value = Box::new(self.parse_expression()?);
                        let target_type = Box::new(self.parse_type_specifier()?);
                        
                        // Parse failure behavior (TRUNCATE, ZERO_EXTEND, etc.)
                        let failure_behavior = if let Some(token) = self.current_token() {
                            if !matches!(token.token_type, TokenType::RightParen) {
                                // For now, ignore the failure behavior and use default
                                self.advance();
                                CastFailureBehavior::ThrowException
                            } else {
                                CastFailureBehavior::ThrowException
                            }
                        } else {
                            CastFailureBehavior::ThrowException
                        };
                        
                        self.consume_right_paren()?;
                        Ok(Expression::TypeCast {
                            value,
                            target_type,
                            failure_behavior,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::AddressOf) => {
                        self.advance(); // consume ADDRESS_OF
                        let operand = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::AddressOf {
                            operand,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::Dereference) => {
                        self.advance(); // consume DEREFERENCE
                        let pointer = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::Dereference {
                            pointer,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::PointerAdd) => {
                        self.advance(); // consume POINTER_ADD
                        let pointer = Box::new(self.parse_expression()?);
                        let offset = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Expression::PointerArithmetic {
                            pointer,
                            offset,
                            operation: crate::ast::PointerOp::Add,
                            source_location: start_location,
                        })
                    }
                    Some(KeywordType::StringLiteral) => {
                        self.advance(); // consume STRING_LITERAL
                        // Next token should be the actual string
                        if let Some(token) = self.current_token() {
                            if let TokenType::String(value) = &token.token_type {
                                let string_value = value.clone();
                                self.advance();
                                self.consume_right_paren()?;
                                return Ok(Expression::StringLiteral {
                                    value: string_value,
                                    source_location: start_location,
                                });
                            }
                        }
                        return Err(ParserError::UnexpectedToken {
                            found: format!("{:?}", self.current_token().map(|t| &t.token_type)),
                            expected: "string value after STRING_LITERAL".to_string(),
                            location: start_location,
                        });
                    }
                    _ => {
                        Err(ParserError::UnexpectedToken {
                            found: keyword.clone(),
                            expected: "expression keyword".to_string(),
                            location: keyword_token.location.clone(),
                        })
                    }
                }
        } else {
            match &keyword_token.token_type {
                TokenType::Identifier(name) => {
                // Could be an enum variant constructor
                let variant_name = Identifier::new(name.clone(), keyword_token.location.clone());
                self.advance(); // consume variant name
                
                // Check if there's an associated value
                let value = if let Some(token) = self.current_token() {
                    if !matches!(token.token_type, TokenType::RightParen) {
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                self.consume_right_paren()?;
                
                Ok(Expression::EnumVariant {
                    enum_name: Identifier::new("".to_string(), start_location.clone()), // Will be resolved during type checking
                    variant_name,
                    value,
                    source_location: start_location,
                })
                }
                _ => {
                    Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", keyword_token.token_type),
                        expected: "expression keyword or enum variant".to_string(),
                        location: keyword_token.location.clone(),
                    })
                }
            }
        };

        result
    }

    /// Parse a function call expression
    fn parse_function_call_expression(&mut self, start_location: SourceLocation) -> Result<Expression, ParserError> {
        // Function name can be either an identifier or a string
        let function_name = match self.current_token() {
            Some(token) => match &token.token_type {
                TokenType::Identifier(name) => {
                    let identifier = Identifier::new(name.clone(), token.location.clone());
                    self.advance();
                    identifier
                }
                TokenType::String(name) => {
                    let identifier = Identifier::new(name.clone(), token.location.clone());
                    self.advance();
                    identifier
                }
                _ => return Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", token.token_type),
                    expected: "function name (identifier or string)".to_string(),
                    location: token.location.clone(),
                }),
            },
            None => return Err(ParserError::UnexpectedEof {
                expected: "function name".to_string(),
            }),
        };
        
        // Parse arguments
        let mut arguments = Vec::new();
        let mut arg_index = 0;
        
        // Check if there's an ARGUMENTS wrapper
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::LeftParen) {
                let next_pos = self.position + 1;
                if next_pos < self.tokens.len() {
                    if let TokenType::Keyword(keyword) = &self.tokens[next_pos].token_type {
                        if self.keywords.get(keyword) == Some(&KeywordType::Arguments) {
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::Arguments)?;
                            
                            // Parse arguments inside ARGUMENTS wrapper
                            while let Some(token) = self.current_token() {
                                if matches!(token.token_type, TokenType::RightParen) {
                                    break;
                                }
                                
                                let arg_expr = self.parse_expression()?;
                                let arg = Argument {
                                    parameter_name: Identifier::new(
                                        format!("arg_{}", arg_index), 
                                        start_location.clone()
                                    ),
                                    value: Box::new(arg_expr),
                                    source_location: start_location.clone(),
                                };
                                arguments.push(arg);
                                arg_index += 1;
                            }
                            
                            self.consume_right_paren()?; // Close ARGUMENTS
                            self.consume_right_paren()?; // Close CALL_FUNCTION
                            
                            return Ok(Expression::FunctionCall {
                                call: FunctionCall {
                                    function_reference: FunctionReference::Local {
                                        name: function_name,
                                    },
                                    arguments,
                                    variadic_arguments: Vec::new(),
                                },
                                source_location: start_location,
                            });
                        }
                    } else if let TokenType::Identifier(ident) = &self.tokens[next_pos].token_type {
                        if ident == "ARGUMENTS" {
                            self.consume_left_paren()?;
                            self.advance(); // consume ARGUMENTS identifier
                            
                            // Parse arguments inside ARGUMENTS wrapper
                            while let Some(token) = self.current_token() {
                                if matches!(token.token_type, TokenType::RightParen) {
                                    break;
                                }
                                
                                let arg_expr = self.parse_expression()?;
                                let arg = Argument {
                                    parameter_name: Identifier::new(
                                        format!("arg_{}", arg_index), 
                                        start_location.clone()
                                    ),
                                    value: Box::new(arg_expr),
                                    source_location: start_location.clone(),
                                };
                                arguments.push(arg);
                                arg_index += 1;
                            }
                            
                            self.consume_right_paren()?; // Close ARGUMENTS
                            self.consume_right_paren()?; // Close CALL_FUNCTION
                            
                            return Ok(Expression::FunctionCall {
                                call: FunctionCall {
                                    function_reference: FunctionReference::Local {
                                        name: function_name,
                                    },
                                    arguments,
                                    variadic_arguments: Vec::new(),
                                },
                                source_location: start_location,
                            });
                        }
                    }
                }
            }
        }
        
        // Parse all arguments until we hit the closing paren (for non-ARGUMENTS style)
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            
            // Check if it's an ARGUMENT keyword
            if let TokenType::LeftParen = token.token_type {
                self.advance(); // consume left paren
                
                if let Some(keyword_token) = self.current_token() {
                    if let TokenType::Keyword(keyword) = &keyword_token.token_type {
                        if self.keywords.get(keyword) == Some(&KeywordType::Argument) {
                            self.advance(); // consume ARGUMENT
                            let arg_expr = self.parse_expression()?;
                            self.consume_right_paren()?;
                            
                            let arg = Argument {
                                parameter_name: Identifier::new(
                                    format!("arg_{}", arg_index), 
                                    start_location.clone()
                                ),
                                value: Box::new(arg_expr),
                                source_location: start_location.clone(),
                            };
                            arguments.push(arg);
                            arg_index += 1;
                            continue;
                        }
                    }
                }
                // If not ARGUMENT keyword, backtrack
                self.position = self.position.saturating_sub(1);
            }
            
            // Otherwise, just parse an expression directly
            let arg_expr = self.parse_expression()?;
            let arg = Argument {
                parameter_name: Identifier::new(
                    format!("arg_{}", arg_index), 
                    start_location.clone()
                ),
                value: Box::new(arg_expr),
                source_location: start_location.clone(),
            };
            arguments.push(arg);
            arg_index += 1;
        }
        
        self.consume_right_paren()?;
        
        Ok(Expression::FunctionCall {
            call: FunctionCall {
                function_reference: FunctionReference::Local {
                    name: function_name,
                },
                arguments,
                variadic_arguments: Vec::new(),
            },
            source_location: start_location,
        })
    }
    
    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<Block, ParserError> {
        let start_location = self.current_token()
            .map(|t| t.location.clone())
            .unwrap_or_else(SourceLocation::unknown);
            
        let mut statements = Vec::new();
        
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            
            statements.push(self.parse_statement()?);
        }
        
        Ok(Block {
            statements,
            source_location: start_location,
        })
    }
    
    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        self.consume_left_paren()?;
        
        let keyword_token = self.current_token()
            .ok_or_else(|| ParserError::UnexpectedEof {
                expected: "statement keyword".to_string(),
            })?;
            
        let location = keyword_token.location.clone();
        
        // Try to get keyword from either Keyword or Identifier token types
        let keyword_str = match &keyword_token.token_type {
            TokenType::Keyword(keyword) => Some(keyword.clone()),
            TokenType::Identifier(ident) => Some(ident.clone()),
            _ => None,
        };
        
        let result = if let Some(keyword) = keyword_str {
            match self.keywords.get(&keyword) {
                    Some(KeywordType::DeclareVariable) => {
                        self.advance();
                        self.parse_variable_declaration(location)
                    }
                    Some(KeywordType::Assign) => {
                        self.advance();
                        self.parse_assignment(location)
                    }
                    Some(KeywordType::ReturnValue) => {
                        self.advance();
                        let value = Some(Box::new(self.parse_expression()?));
                        self.consume_right_paren()?;
                        Ok(Statement::Return { value, source_location: location })
                    }
                    Some(KeywordType::ReturnVoid) => {
                        self.advance();
                        self.consume_right_paren()?;
                        Ok(Statement::Return { value: None, source_location: location })
                    }
                    Some(KeywordType::IfCondition) => {
                        self.advance();
                        self.parse_if_statement(location)
                    }
                    Some(KeywordType::LoopWhileCondition) => {
                        self.advance();
                        self.parse_while_loop(location)
                    }
                    Some(KeywordType::LoopForEachElement) => {
                        self.advance();
                        self.parse_for_each_loop(location)
                    }
                    Some(KeywordType::LoopFixedIterations) => {
                        self.advance();
                        self.parse_fixed_iteration_loop(location)
                    }
                    Some(KeywordType::BreakLoop) => {
                        self.advance();
                        let target_label = if self.current_token()
                            .map(|t| !matches!(t.token_type, TokenType::RightParen))
                            .unwrap_or(false) {
                            Some(self.consume_identifier()?)
                        } else {
                            None
                        };
                        self.consume_right_paren()?;
                        Ok(Statement::Break { target_label, source_location: location })
                    }
                    Some(KeywordType::ContinueLoop) => {
                        self.advance();
                        let target_label = if self.current_token()
                            .map(|t| !matches!(t.token_type, TokenType::RightParen))
                            .unwrap_or(false) {
                            Some(self.consume_identifier()?)
                        } else {
                            None
                        };
                        self.consume_right_paren()?;
                        Ok(Statement::Continue { target_label, source_location: location })
                    }
                    Some(KeywordType::TryExecute) => {
                        self.advance();
                        self.parse_try_block(location)
                    }
                    Some(KeywordType::ThrowException) => {
                        self.advance();
                        let exception = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Statement::Throw { exception, source_location: location })
                    }
                    Some(KeywordType::ResourceScope) => {
                        self.advance();
                        let scope = self.parse_resource_scope(location.clone())?;
                        Ok(Statement::ResourceScope { scope, source_location: location })
                    }
                    Some(KeywordType::CallFunction) => {
                        self.advance();
                        let call = self.parse_function_call_inner()?;
                        self.consume_right_paren()?;
                        Ok(Statement::FunctionCall { call, source_location: location })
                    }
                    Some(KeywordType::ExpressionStatement) => {
                        self.advance(); // consume EXPRESSION_STATEMENT
                        let expr = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Statement::Expression { expr, source_location: location })
                    }
                    Some(KeywordType::SetMapValue) => {
                        self.advance(); // consume SET_MAP_VALUE
                        let map = Box::new(self.parse_expression()?);
                        let key = Box::new(self.parse_expression()?);
                        let value = Box::new(self.parse_expression()?);
                        self.consume_right_paren()?;
                        Ok(Statement::Assignment {
                            target: AssignmentTarget::MapValue { map, key },
                            value,
                            source_location: location,
                        })
                    }
                    _ => Err(ParserError::UnexpectedToken {
                        found: keyword.clone(),
                        expected: "statement keyword".to_string(),
                        location,
                    })
                }
        } else {
            Err(ParserError::UnexpectedToken {
                found: format!("{:?}", keyword_token.token_type),
                expected: "statement keyword".to_string(),
                location,
            })
        };
        
        result
    }
    
    /// Parse variable declaration
    fn parse_variable_declaration(&mut self, start_location: SourceLocation) -> Result<Statement, ParserError> {
        // DECLARE_VARIABLE has already been consumed
        let mut name = None;
        let mut type_spec = None;
        let mut value = None;
        let mut mutability = Mutability::Mutable;
        
        // Parse fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            
            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "variable declaration field".to_string(),
                })?;
            
            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Name) => {
                            self.advance(); // consume NAME
                            // Name can be a string or identifier
                            name = Some(match self.current_token() {
                                Some(token) => match &token.token_type {
                                    TokenType::String(s) => {
                                        let id = Identifier::new(s.clone(), token.location.clone());
                                        self.advance();
                                        id
                                    }
                                    TokenType::Identifier(_) => {
                                        self.consume_identifier()?
                                    }
                                    _ => return Err(ParserError::UnexpectedToken {
                                        found: format!("{:?}", token.token_type),
                                        expected: "variable name".to_string(),
                                        location: token.location.clone(),
                                    })
                                },
                                None => return Err(ParserError::UnexpectedEof {
                                    expected: "variable name".to_string(),
                                })
                            });
                        }
                        Some(KeywordType::Type) => {
                            self.advance(); // consume TYPE
                            type_spec = Some(self.parse_type_specifier()?);
                        }
                        Some(KeywordType::Value) => {
                            self.advance(); // consume VALUE
                            value = Some(Box::new(self.parse_expression()?));
                        }
                        Some(KeywordType::Mutability) => {
                            self.advance(); // consume MUTABILITY
                            let mutability_keyword = self.current_token()
                                .ok_or_else(|| ParserError::UnexpectedEof {
                                    expected: "mutability value".to_string(),
                                })?;
                            match &mutability_keyword.token_type {
                                TokenType::Keyword(k) => {
                                    match self.keywords.get(k) {
                                        Some(KeywordType::Mutable) => {
                                            mutability = Mutability::Mutable;
                                            self.advance();
                                        }
                                        Some(KeywordType::Immutable) => {
                                            mutability = Mutability::Immutable;
                                            self.advance();
                                        }
                                        _ => return Err(ParserError::UnexpectedToken {
                                            found: k.clone(),
                                            expected: "MUTABLE or IMMUTABLE".to_string(),
                                            location: mutability_keyword.location.clone(),
                                        })
                                    }
                                }
                                _ => return Err(ParserError::UnexpectedToken {
                                    found: format!("{:?}", mutability_keyword.token_type),
                                    expected: "mutability keyword".to_string(),
                                    location: mutability_keyword.location.clone(),
                                })
                            }
                        }
                        _ => return Err(ParserError::UnexpectedToken {
                            found: keyword.clone(),
                            expected: "variable declaration field".to_string(),
                            location: field_keyword.location.clone(),
                        })
                    }
                }
                _ => return Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", field_keyword.token_type),
                    expected: "field keyword".to_string(),
                    location: field_keyword.location.clone(),
                })
            }
            
            self.consume_right_paren()?;
        }
        
        self.consume_right_paren()?;
        
        let name = name.ok_or_else(|| ParserError::MissingRequiredField {
            field: "NAME".to_string(),
            construct: "DECLARE_VARIABLE".to_string(),
            location: start_location.clone(),
        })?;
        
        let type_spec = type_spec.ok_or_else(|| ParserError::MissingRequiredField {
            field: "TYPE".to_string(),
            construct: "DECLARE_VARIABLE".to_string(),
            location: start_location.clone(),
        })?;
        
        Ok(Statement::VariableDeclaration {
            name,
            type_spec: Box::new(type_spec),
            mutability,
            initial_value: value,
            intent: None,
            source_location: start_location,
        })
    }
    
    /// Parse assignment statement
    fn parse_assignment(&mut self, start_location: SourceLocation) -> Result<Statement, ParserError> {
        // ASSIGN has already been consumed
        let mut target = None;
        let mut value = None;
        
        // Parse fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            
            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "assignment field".to_string(),
                })?;
            
            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::TargetVariable) => {
                            self.advance(); // consume TARGET_VARIABLE
                            let var_name = self.consume_identifier()?;
                            target = Some(AssignmentTarget::Variable { name: var_name });
                        }
                        Some(KeywordType::SourceExpression) => {
                            self.advance(); // consume SOURCE_EXPRESSION
                            value = Some(Box::new(self.parse_expression()?));
                        }
                        _ => return Err(ParserError::UnexpectedToken {
                            found: keyword.clone(),
                            expected: "assignment field (TARGET_VARIABLE or SOURCE_EXPRESSION)".to_string(),
                            location: field_keyword.location.clone(),
                        })
                    }
                }
                _ => return Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", field_keyword.token_type),
                    expected: "field keyword".to_string(),
                    location: field_keyword.location.clone(),
                })
            }
            
            self.consume_right_paren()?;
        }
        
        self.consume_right_paren()?;
        
        let target = target.ok_or_else(|| ParserError::MissingRequiredField {
            field: "TARGET_VARIABLE".to_string(),
            construct: "ASSIGN".to_string(),
            location: start_location.clone(),
        })?;
        
        let value = value.ok_or_else(|| ParserError::MissingRequiredField {
            field: "SOURCE_EXPRESSION".to_string(),
            construct: "ASSIGN".to_string(),
            location: start_location.clone(),
        })?;
        
        Ok(Statement::Assignment {
            target,
            value,
            source_location: start_location,
        })
    }
    
    /// Parse if statement
    fn parse_if_statement(&mut self, start_location: SourceLocation) -> Result<Statement, ParserError> {
        // IF_CONDITION has already been consumed by the caller
        let condition = Box::new(self.parse_expression()?);
        
        // Parse THEN_EXECUTE block
        self.consume_left_paren()?;
        self.consume_keyword(KeywordType::ThenExecute)?;
        let then_block = self.parse_block()?;
        self.consume_right_paren()?;
        
        // Parse optional ELSE_IF_CONDITION blocks
        let mut else_ifs = Vec::new();
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::LeftParen) {
                let next_pos = self.position + 1;
                if next_pos < self.tokens.len() {
                    if let TokenType::Keyword(keyword) = &self.tokens[next_pos].token_type {
                        if self.keywords.get(keyword) == Some(&KeywordType::ElseIfCondition) {
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::ElseIfCondition)?;
                            let else_if_condition = Box::new(self.parse_expression()?);
                            
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::ThenExecute)?;
                            let else_if_block = self.parse_block()?;
                            self.consume_right_paren()?;
                            
                            else_ifs.push(ElseIf {
                                condition: else_if_condition,
                                block: else_if_block,
                                source_location: self.tokens[next_pos].location.clone(),
                            });
                            
                            self.consume_right_paren()?;
                            continue;
                        }
                    }
                }
            }
            break;
        }
        
        // Parse optional ELSE_EXECUTE block
        let else_block = if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::LeftParen) {
                let next_pos = self.position + 1;
                if next_pos < self.tokens.len() {
                    if let TokenType::Keyword(keyword) = &self.tokens[next_pos].token_type {
                        if self.keywords.get(keyword) == Some(&KeywordType::ElseExecute) {
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::ElseExecute)?;
                            let block = self.parse_block()?;
                            self.consume_right_paren()?;
                            Some(block)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        self.consume_right_paren()?;
        
        Ok(Statement::If {
            condition,
            then_block,
            else_ifs,
            else_block,
            source_location: start_location,
        })
    }
    
    /// Parse while loop
    fn parse_while_loop(&mut self, start_location: SourceLocation) -> Result<Statement, ParserError> {
        let condition = Box::new(self.parse_expression()?);
        
        // TODO: Parse optional invariant and label
        let invariant = None;
        let label = None;
        
        // Parse loop body
        self.consume_left_paren()?;
        self.consume_keyword(KeywordType::IterationBody)?;
        let body = self.parse_block()?;
        self.consume_right_paren()?;
        
        self.consume_right_paren()?;
        
        Ok(Statement::WhileLoop {
            condition,
            invariant,
            body,
            label,
            source_location: start_location,
        })
    }
    
    /// Parse for-each loop
    fn parse_for_each_loop(&mut self, start_location: SourceLocation) -> Result<Statement, ParserError> {
        // LOOP_FOR_EACH_ELEMENT has already been consumed
        let mut element_binding = None;
        let mut index_binding = None;
        let mut collection = None;
        let mut body = None;
        
        // Parse fields
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            
            self.consume_left_paren()?;
            let field_keyword = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "for-each field keyword".to_string(),
                })?;
            
            match &field_keyword.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::ElementVariable) => {
                            self.advance(); // consume ELEMENT_VARIABLE
                            element_binding = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::IndexVariable) => {
                            self.advance(); // consume INDEX_VARIABLE
                            index_binding = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::Collection) => {
                            self.advance(); // consume COLLECTION
                            collection = Some(Box::new(self.parse_expression()?));
                        }
                        Some(KeywordType::IterationBody) => {
                            self.advance(); // consume ITERATION_BODY
                            body = Some(self.parse_block()?);
                        }
                        _ => return Err(ParserError::UnexpectedToken {
                            found: keyword.clone(),
                            expected: "for-each field keyword (ELEMENT_VARIABLE, INDEX_VARIABLE, COLLECTION, ITERATION_BODY)".to_string(),
                            location: field_keyword.location.clone(),
                        })
                    }
                }
                _ => return Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", field_keyword.token_type),
                    expected: "field keyword".to_string(),
                    location: field_keyword.location.clone(),
                })
            }
            
            self.consume_right_paren()?;
        }
        
        self.consume_right_paren()?;
        
        let element_binding = element_binding.ok_or_else(|| ParserError::MissingRequiredField {
            field: "ELEMENT_VARIABLE".to_string(),
            construct: "LOOP_FOR_EACH_ELEMENT".to_string(),
            location: start_location.clone(),
        })?;
        
        let collection = collection.ok_or_else(|| ParserError::MissingRequiredField {
            field: "COLLECTION".to_string(),
            construct: "LOOP_FOR_EACH_ELEMENT".to_string(),
            location: start_location.clone(),
        })?;
        
        let body = body.ok_or_else(|| ParserError::MissingRequiredField {
            field: "ITERATION_BODY".to_string(),
            construct: "LOOP_FOR_EACH_ELEMENT".to_string(),
            location: start_location.clone(),
        })?;
        
        // For now, infer element type as INTEGER
        // In a full implementation, we'd get this from the collection type
        let element_type = Box::new(TypeSpecifier::Primitive {
            type_name: PrimitiveType::Integer,
            source_location: start_location.clone(),
        });
        
        Ok(Statement::ForEachLoop {
            collection,
            element_binding,
            element_type,
            index_binding,
            body,
            label: None,
            source_location: start_location,
        })
    }
    
    /// Parse fixed iteration loop (FOR loop)
    fn parse_fixed_iteration_loop(&mut self, start_location: SourceLocation) -> Result<Statement, ParserError> {
        // LOOP_FIXED_ITERATIONS has already been consumed
        let mut counter = None;
        let mut from_value = None;
        let mut to_value = None;
        let mut step_value = None;
        let mut body = None;
        
        // Parse fields until we hit the right paren
        while self.current_token()
            .map(|t| !matches!(t.token_type, TokenType::RightParen))
            .unwrap_or(false) {
            
            self.consume_left_paren()?;
            let keyword_token = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "loop field keyword".to_string(),
                })?;
            
            match &keyword_token.token_type {
                TokenType::Keyword(keyword) => {
                    match self.keywords.get(keyword) {
                        Some(KeywordType::Counter) => {
                            self.advance();
                            counter = Some(self.consume_identifier()?);
                        }
                        Some(KeywordType::From) => {
                            self.advance();
                            from_value = Some(Box::new(self.parse_expression()?));
                        }
                        Some(KeywordType::To) => {
                            self.advance();
                            to_value = Some(Box::new(self.parse_expression()?));
                        }
                        Some(KeywordType::Step) => {
                            self.advance();
                            step_value = Some(Box::new(self.parse_expression()?));
                        }
                        Some(KeywordType::Do) => {
                            self.advance();
                            body = Some(self.parse_block()?);
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "loop field keyword (COUNTER, FROM, TO, STEP, DO)".to_string(),
                                location: keyword_token.location.clone(),
                            });
                        }
                    }
                }
                TokenType::Identifier(ident) => {
                    // Handle identifiers that should be keywords
                    match ident.as_str() {
                        "COUNTER" => {
                            self.advance();
                            counter = Some(self.consume_identifier()?);
                        }
                        "FROM" => {
                            self.advance();
                            from_value = Some(Box::new(self.parse_expression()?));
                        }
                        "TO" => {
                            self.advance();
                            to_value = Some(Box::new(self.parse_expression()?));
                        }
                        "STEP" => {
                            self.advance();
                            step_value = Some(Box::new(self.parse_expression()?));
                        }
                        "DO" => {
                            self.advance();
                            body = Some(self.parse_block()?);
                        }
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: ident.clone(),
                                expected: "loop field keyword (COUNTER, FROM, TO, STEP, DO)".to_string(),
                                location: keyword_token.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", keyword_token.token_type),
                        expected: "loop field keyword".to_string(),
                        location: keyword_token.location.clone(),
                    });
                }
            }
            self.consume_right_paren()?;
        }
        
        self.consume_right_paren()?;
        
        // Validate required fields
        let counter = counter.ok_or_else(|| ParserError::MissingRequiredField {
            field: "COUNTER".to_string(),
            construct: "LOOP_FIXED_ITERATIONS".to_string(),
            location: start_location.clone(),
        })?;
        
        let from_value = from_value.ok_or_else(|| ParserError::MissingRequiredField {
            field: "FROM".to_string(),
            construct: "LOOP_FIXED_ITERATIONS".to_string(),
            location: start_location.clone(),
        })?;
        
        let to_value = to_value.ok_or_else(|| ParserError::MissingRequiredField {
            field: "TO".to_string(),
            construct: "LOOP_FIXED_ITERATIONS".to_string(),
            location: start_location.clone(),
        })?;
        
        let body = body.ok_or_else(|| ParserError::MissingRequiredField {
            field: "DO".to_string(),
            construct: "LOOP_FIXED_ITERATIONS".to_string(),
            location: start_location.clone(),
        })?;
        
        Ok(Statement::FixedIterationLoop {
            counter,
            from_value,
            to_value,
            step_value,
            inclusive: true, // Default to inclusive
            body,
            label: None,
            source_location: start_location,
        })
    }
    
    /// Parse try block
    fn parse_try_block(&mut self, start_location: SourceLocation) -> Result<Statement, ParserError> {
        // Parse protected block
        let protected_block = self.parse_block()?;
        
        // Parse catch clauses
        let mut catch_clauses = Vec::new();
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::LeftParen) {
                let next_pos = self.position + 1;
                if next_pos < self.tokens.len() {
                    if let TokenType::Keyword(keyword) = &self.tokens[next_pos].token_type {
                        if self.keywords.get(keyword) == Some(&KeywordType::CatchException) {
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::CatchException)?;
                            
                            let exception_type = Box::new(self.parse_type_specifier()?);
                            let binding_variable = if self.current_token()
                                .map(|t| matches!(t.token_type, TokenType::Identifier(_)))
                                .unwrap_or(false) {
                                Some(self.consume_identifier()?)
                            } else {
                                None
                            };
                            
                            let handler_block = self.parse_block()?;
                            
                            catch_clauses.push(CatchClause {
                                exception_type,
                                binding_variable,
                                handler_block,
                                source_location: self.tokens[next_pos].location.clone(),
                            });
                            
                            self.consume_right_paren()?;
                            continue;
                        }
                    }
                }
            }
            break;
        }
        
        // Parse optional finally block
        let finally_block = if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::LeftParen) {
                let next_pos = self.position + 1;
                if next_pos < self.tokens.len() {
                    if let TokenType::Keyword(keyword) = &self.tokens[next_pos].token_type {
                        if self.keywords.get(keyword) == Some(&KeywordType::FinallyExecute) {
                            self.consume_left_paren()?;
                            self.consume_keyword(KeywordType::FinallyExecute)?;
                            let block = self.parse_block()?;
                            self.consume_right_paren()?;
                            Some(block)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        
        self.consume_right_paren()?;
        
        Ok(Statement::TryBlock {
            protected_block,
            catch_clauses,
            finally_block,
            source_location: start_location,
        })
    }
    
    /// Parse function call (inner part after CALL_FUNCTION)
    fn parse_function_call_inner(&mut self) -> Result<FunctionCall, ParserError> {
        // Parse function name
        let function_name = self.consume_identifier()?;
        
        // For now, assume all function calls are local
        let function_reference = FunctionReference::Local {
            name: function_name,
        };
        
        // Parse arguments - for now, all arguments are treated as variadic
        // since we don't have proper parameter name parsing yet
        let mut variadic_arguments = Vec::new();
        
        // Parse all remaining arguments until right paren
        while self.current_token()
            .map(|t| !matches!(t.token_type, TokenType::RightParen))
            .unwrap_or(false) {
            let arg_expr = self.parse_expression()?;
            variadic_arguments.push(Box::new(arg_expr));
        }
        
        Ok(FunctionCall {
            function_reference,
            arguments: Vec::new(), // Named arguments not supported yet
            variadic_arguments,
        })
    }
    
    /// Parse a match expression
    fn parse_match_expression(&mut self, start_location: SourceLocation) -> Result<Expression, ParserError> {
        // Parse the value to match on
        let value = Box::new(self.parse_expression()?);
        
        // Parse cases
        let mut cases = Vec::new();
        
        while let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::RightParen) {
                break;
            }
            
            self.consume_left_paren()?;
            self.consume_keyword(KeywordType::Case)?;
            
            // Parse pattern
            let pattern = self.parse_pattern()?;
            
            // Parse body expression
            let body = Box::new(self.parse_expression()?);
            
            cases.push(MatchCase {
                pattern,
                body,
                source_location: self.current_token().unwrap().location.clone(),
            });
            
            self.consume_right_paren()?; // Close case
        }
        
        self.consume_right_paren()?; // Close match expression
        
        if cases.is_empty() {
            return Err(ParserError::MissingRequiredField {
                field: "CASE".to_string(),
                construct: "MATCH_EXPRESSION".to_string(),
                location: start_location.clone(),
            });
        }
        
        Ok(Expression::Match {
            value,
            cases,
            source_location: start_location,
        })
    }
    
    /// Parse a pattern for pattern matching
    fn parse_pattern(&mut self) -> Result<Pattern, ParserError> {
        self.consume_left_paren()?;
        
        let first_token = self.current_token()
            .ok_or_else(|| ParserError::UnexpectedEof {
                expected: "pattern".to_string(),
            })?
            .clone(); // Clone to avoid borrow issues
        
        // Check if it's an identifier (for enum variant patterns)
        match &first_token.token_type {
            TokenType::Identifier(name) => {
                let variant_name = Identifier::new(name.clone(), first_token.location.clone());
                self.advance(); // consume variant name
                
                // Check for nested pattern or binding variable
                let (nested_pattern, binding) = if let Some(token) = self.current_token() {
                    match &token.token_type {
                        TokenType::LeftParen => {
                            // Nested pattern like (Some (Ok x))
                            let nested = Box::new(self.parse_pattern()?);
                            (Some(nested), None)
                        }
                        TokenType::Identifier(binding_name) => {
                            // Simple binding like (Some x)
                            let binding_id = Identifier::new(binding_name.clone(), token.location.clone());
                            self.advance(); // consume binding
                            (None, Some(binding_id))
                        }
                        _ => (None, None)
                    }
                } else {
                    (None, None)
                };
                
                self.consume_right_paren()?;
                
                Ok(Pattern::EnumVariant {
                    enum_name: None, // Unqualified for now
                    variant_name,
                    binding,
                    nested_pattern,
                    source_location: first_token.location.clone(),
                })
            }
            TokenType::Keyword(keyword) if keyword == "_" => {
                // Wildcard pattern
                self.advance(); // consume _
                
                // Check for binding variable
                let binding = if let Some(token) = self.current_token() {
                    if let TokenType::Identifier(binding_name) = &token.token_type {
                        let binding_id = Identifier::new(binding_name.clone(), token.location.clone());
                        self.advance(); // consume binding
                        Some(binding_id)
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                self.consume_right_paren()?;
                
                Ok(Pattern::Wildcard {
                    binding,
                    source_location: first_token.location.clone(),
                })
            }
            _ => {
                // Literal pattern
                let start_loc = first_token.location.clone();
                let value = Box::new(self.parse_expression()?);
                self.consume_right_paren()?;
                
                Ok(Pattern::Literal {
                    value,
                    source_location: start_loc,
                })
            }
        }
    }
}

/// Module content items
enum ModuleContent {
    Import(ImportStatement),
    Export(ExportStatement),
    TypeDefinition(TypeDefinition),
    ConstantDeclaration(ConstantDeclaration),
    FunctionDefinition(Box<Function>),
    ExternalFunction(ExternalFunction),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Lexer, Token, TokenType};
    
    #[test]
    fn test_parser_creation() {
        let tokens = vec![
            Token::new(TokenType::LeftParen, SourceLocation::unknown(), "(".to_string()),
            Token::new(TokenType::RightParen, SourceLocation::unknown(), ")".to_string()),
            Token::new(TokenType::Eof, SourceLocation::unknown(), "".to_string()),
        ];
        
        let parser = Parser::new(tokens);
        assert_eq!(parser.position, 0);
    }

    #[test]
    fn test_simple_module_parsing() {
        let source = r#"
        (DEFINE_MODULE
          (NAME 'test_module')
          (INTENT "A test module")
          (CONTENT)
        )
        "#;

        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let program = parser.parse_program().unwrap();
        assert_eq!(program.modules.len(), 1);
        assert_eq!(program.modules[0].name.name, "test_module");
        assert_eq!(program.modules[0].intent, Some("A test module".to_string()));
    }

    #[test]
    fn test_module_with_constant_parsing() {
        let source = r#"
        (DEFINE_MODULE
          (NAME 'math_module')
          (INTENT "A module with constants")
          (CONTENT
            (DECLARE_CONSTANT
              (NAME 'PI')
              (TYPE FLOAT)
              (VALUE 3.14159)
              (INTENT "Mathematical constant PI")
            )
          )
        )
        "#;

        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let program = parser.parse_program().unwrap();
        assert_eq!(program.modules.len(), 1);
        assert_eq!(program.modules[0].name.name, "math_module");
        assert_eq!(program.modules[0].constant_declarations.len(), 1);
        assert_eq!(program.modules[0].constant_declarations[0].name.name, "PI");
    }

    #[test]
    fn test_expression_parsing() {
        let source = r#"
        (DEFINE_MODULE
          (NAME 'expr_test')
          (CONTENT
            (DECLARE_CONSTANT
              (NAME 'result')
              (TYPE INTEGER)
              (VALUE (EXPRESSION_ADD 5 10))
            )
          )
        )
        "#;

        let mut lexer = Lexer::new(source, "test.aether".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let program = parser.parse_program().unwrap();
        assert_eq!(program.modules.len(), 1);
        assert_eq!(program.modules[0].constant_declarations.len(), 1);
        
        let constant = &program.modules[0].constant_declarations[0];
        assert_eq!(constant.name.name, "result");
        
        // Check that the value is an addition expression
        if let Expression::Add { .. } = constant.value.as_ref() {
            // Success
        } else {
            panic!("Expected Add expression, got {:?}", constant.value);
        }
    }

    #[test]
    fn test_parser_error_handling() {
        let tokens = vec![
            Token::new(TokenType::Identifier("invalid".to_string()), SourceLocation::unknown(), "invalid".to_string()),
            Token::new(TokenType::Eof, SourceLocation::unknown(), "".to_string()),
        ];
        
        let mut parser = Parser::new(tokens);
        let result = parser.parse_program();
        assert!(result.is_err());
    }

    #[test]
    fn test_keyword_mapping() {
        let parser = Parser::new(vec![]);
        assert!(parser.keywords.contains_key("DEFINE_MODULE"));
        assert!(parser.keywords.contains_key("DEFINE_FUNCTION"));
        assert_eq!(parser.keywords.get("DEFINE_MODULE"), Some(&KeywordType::DefineModule));
    }
}

impl Parser {
    /// Parse a contract assertion (precondition, postcondition, or invariant)
    fn parse_contract_assertion(&mut self) -> Result<ContractAssertion, ParserError> {
        let start_location = self.current_token().unwrap().location.clone();
        
        // Parse the condition expression
        let condition = Box::new(self.parse_expression()?);
        
        // Default failure action
        let mut failure_action = FailureAction::AssertFail;
        let mut message = None;
        
        // Check for optional failure action
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::Keyword(_)) {
                if let TokenType::Keyword(keyword) = &token.token_type {
                    // Parse failure action if present
                    match keyword.as_str() {
                        "ASSERT_FAIL" => {
                            self.advance();
                            failure_action = FailureAction::AssertFail;
                        }
                        "LOG_WARNING" => {
                            self.advance();
                            failure_action = FailureAction::LogWarning;
                        }
                        "THROW_EXCEPTION" => {
                            self.advance();
                            failure_action = FailureAction::ThrowException;
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Check for optional message
        if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::String(_)) {
                message = Some(self.consume_string()?);
            }
        }
        
        Ok(ContractAssertion {
            condition,
            failure_action,
            message,
            source_location: start_location,
        })
    }
    
    /// Parse performance expectation
    fn parse_performance_expectation(&mut self) -> Result<PerformanceExpectation, ParserError> {
        // Parse metric type
        let metric = if let Some(token) = self.current_token().cloned() {
            match &token.token_type {
                TokenType::Keyword(keyword) => {
                    self.advance();
                    match keyword.as_str() {
                        "LATENCY_MS" => PerformanceMetric::LatencyMs,
                        "THROUGHPUT_OPS" => PerformanceMetric::ThroughputOpsPerSec,
                        "MEMORY_BYTES" => PerformanceMetric::MemoryUsageBytes,
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "performance metric".to_string(),
                                location: token.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", token),
                        expected: "performance metric".to_string(),
                        location: token.location.clone(),
                    });
                }
            }
        } else {
            return Err(ParserError::UnexpectedEof {
                expected: "performance metric".to_string(),
            });
        };
        
        // Parse target value
        let target_value = self.consume_float()?;
        
        // Parse optional context
        let context = if let Some(token) = self.current_token() {
            if matches!(token.token_type, TokenType::String(_)) {
                Some(self.consume_string()?)
            } else {
                None
            }
        } else {
            None
        };
        
        Ok(PerformanceExpectation {
            metric,
            target_value,
            context,
        })
    }
    
    /// Parse complexity expectation
    fn parse_complexity_expectation(&mut self) -> Result<ComplexityExpectation, ParserError> {
        // Parse complexity type
        let complexity_type = if let Some(token) = self.current_token().cloned() {
            match &token.token_type {
                TokenType::Keyword(keyword) => {
                    self.advance();
                    match keyword.as_str() {
                        "TIME" => ComplexityType::Time,
                        "SPACE" => ComplexityType::Space,
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "complexity type (TIME or SPACE)".to_string(),
                                location: token.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", token),
                        expected: "complexity type".to_string(),
                        location: token.location.clone(),
                    });
                }
            }
        } else {
            return Err(ParserError::UnexpectedEof {
                expected: "complexity type".to_string(),
            });
        };
        
        // Parse notation
        let notation = if let Some(token) = self.current_token().cloned() {
            match &token.token_type {
                TokenType::Keyword(keyword) => {
                    self.advance();
                    match keyword.as_str() {
                        "BIG_O" => ComplexityNotation::BigO,
                        "BIG_THETA" => ComplexityNotation::BigTheta,
                        "BIG_OMEGA" => ComplexityNotation::BigOmega,
                        _ => {
                            return Err(ParserError::UnexpectedToken {
                                found: keyword.clone(),
                                expected: "complexity notation".to_string(),
                                location: token.location.clone(),
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        found: format!("{:?}", token),
                        expected: "complexity notation".to_string(),
                        location: token.location.clone(),
                    });
                }
            }
        } else {
            return Err(ParserError::UnexpectedEof {
                expected: "complexity notation".to_string(),
            });
        };
        
        // Parse value string (e.g., "O(n log n)")
        let value = self.consume_string()?;
        
        Ok(ComplexityExpectation {
            complexity_type,
            notation,
            value,
        })
    }
    
    /// Parse RESOURCE_SCOPE construct
    /// (RESOURCE_SCOPE
    ///   (SCOPE_ID "unique_id")
    ///   (RESOURCES
    ///     (ACQUIRE_RESOURCE
    ///       (RESOURCE_TYPE "file_handle")
    ///       (RESOURCE_BINDING "file")
    ///       (ACQUISITION (CALL_FUNCTION (NAME "open_file") ...))
    ///       (CLEANUP "file_close")))
    ///   (INVARIANTS "file != NULL")
    ///   (CLEANUP_ORDER "REVERSE_ACQUISITION")
    ///   (BODY ...))
    fn parse_resource_scope(&mut self, location: SourceLocation) -> Result<resource::ResourceScope, ParserError> {
        use crate::ast::resource::*;
        
        let mut scope_id = String::from("resource_scope_auto");
        let mut resources = Vec::new();
        let mut invariants = Vec::new();
        let mut cleanup_order = CleanupOrder::ReverseAcquisition;
        let mut body = None;
        
        while self.current_token().map(|t| &t.token_type) != Some(&TokenType::RightParen) {
            self.consume_left_paren()?;
            let field_token = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "field keyword".to_string(),
                })?;
            
            let field = match &field_token.token_type {
                TokenType::Keyword(k) => k.clone(),
                _ => return Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", field_token.token_type),
                    expected: "field keyword".to_string(),
                    location: field_token.location.clone(),
                }),
            };
            self.advance();
            
            match self.keywords.get(&field).cloned() {
                Some(KeywordType::Name) => {
                    if let Some(token) = self.current_token() {
                        match &token.token_type {
                            TokenType::String(s) => {
                                scope_id = s.clone();
                                self.advance();
                            }
                            _ => return Err(ParserError::UnexpectedToken {
                                found: format!("{:?}", token.token_type),
                                expected: "string literal".to_string(),
                                location: token.location.clone(),
                            }),
                        }
                    }
                    self.consume_right_paren()?;
                }
                Some(KeywordType::AcquireResource) => {
                    resources.push(self.parse_resource_acquisition()?);
                    self.consume_right_paren()?;
                }
                Some(KeywordType::Body) => {
                    body = Some(self.parse_block()?);
                    self.consume_right_paren()?;
                }
                Some(KeywordType::CleanupOrder) => {
                    let order_str = if let Some(token) = self.current_token() {
                        match &token.token_type {
                            TokenType::String(s) => {
                                let str = s.clone();
                                self.advance();
                                str
                            }
                            _ => return Err(ParserError::UnexpectedToken {
                                found: format!("{:?}", token.token_type),
                                expected: "string literal".to_string(),
                                location: token.location.clone(),
                            }),
                        }
                    } else {
                        return Err(ParserError::UnexpectedEof {
                            expected: "string literal".to_string(),
                        });
                    };
                    cleanup_order = match order_str.as_str() {
                        "REVERSE_ACQUISITION" => CleanupOrder::ReverseAcquisition,
                        "FORWARD_ACQUISITION" => CleanupOrder::ForwardAcquisition,
                        "DEPENDENCY_BASED" => CleanupOrder::DependencyBased,
                        "PARALLEL" => CleanupOrder::Parallel,
                        _ => CleanupOrder::ReverseAcquisition,
                    };
                    self.consume_right_paren()?;
                }
                _ => {
                    return Err(ParserError::InvalidConstruct {
                        construct: format!("Unknown resource scope field: {}", field),
                        location: location.clone(),
                    });
                }
            }
        }
        
        self.consume_right_paren()?;
        
        let body = body.ok_or_else(|| ParserError::MissingRequiredField {
            field: "BODY".to_string(),
            construct: "RESOURCE_SCOPE".to_string(),
            location: location.clone(),
        })?;
        
        Ok(ResourceScope {
            scope_id,
            resources,
            invariants,
            body,
            cleanup_guaranteed: true,
            cleanup_order,
            source_location: location,
        })
    }
    
    /// Parse resource acquisition
    fn parse_resource_acquisition(&mut self) -> Result<resource::ResourceAcquisition, ParserError> {
        use crate::ast::resource::*;
        
        let location = self.current_token()
            .map(|t| t.location.clone())
            .unwrap_or_else(SourceLocation::unknown);
        
        let mut resource_type = String::new();
        let mut binding = None;
        let mut acquisition = None;
        let mut cleanup = CleanupSpecification::Automatic;
        let mut type_spec = None;
        let parameters = Vec::new();
        
        while self.current_token().map(|t| &t.token_type) != Some(&TokenType::RightParen) {
            self.consume_left_paren()?;
            let field_token = self.current_token()
                .ok_or_else(|| ParserError::UnexpectedEof {
                    expected: "field keyword".to_string(),
                })?;
            
            let field = match &field_token.token_type {
                TokenType::Keyword(k) => k.clone(),
                _ => return Err(ParserError::UnexpectedToken {
                    found: format!("{:?}", field_token.token_type),
                    expected: "field keyword".to_string(),
                    location: field_token.location.clone(),
                }),
            };
            self.advance();
            
            match self.keywords.get(&field).cloned() {
                Some(KeywordType::ResourceType) => {
                    if let Some(token) = self.current_token() {
                        match &token.token_type {
                            TokenType::String(s) => {
                                resource_type = s.clone();
                                self.advance();
                            }
                            _ => return Err(ParserError::UnexpectedToken {
                                found: format!("{:?}", token.token_type),
                                expected: "string literal".to_string(),
                                location: token.location.clone(),
                            }),
                        }
                    }
                    self.consume_right_paren()?;
                }
                Some(KeywordType::ResourceBinding) => {
                    if let Some(token) = self.current_token() {
                        match &token.token_type {
                            TokenType::Identifier(name) => {
                                binding = Some(Identifier::new(name.clone(), token.location.clone()));
                                self.advance();
                            }
                            _ => return Err(ParserError::UnexpectedToken {
                                found: format!("{:?}", token.token_type),
                                expected: "identifier".to_string(),
                                location: token.location.clone(),
                            }),
                        }
                    }
                    self.consume_right_paren()?;
                }
                Some(KeywordType::Value) => {
                    acquisition = Some(self.parse_expression()?);
                    self.consume_right_paren()?;
                }
                Some(KeywordType::Cleanup) => {
                    cleanup = self.parse_cleanup_spec()?;
                    self.consume_right_paren()?;
                }
                Some(KeywordType::Type) => {
                    type_spec = Some(self.parse_type_specifier()?);
                    self.consume_right_paren()?;
                }
                _ => {
                    return Err(ParserError::InvalidConstruct {
                        construct: format!("Unknown resource acquisition field: {}", field),
                        location: location.clone(),
                    });
                }
            }
        }
        
        let binding = binding.ok_or_else(|| ParserError::MissingRequiredField {
            field: "RESOURCE_BINDING".to_string(),
            construct: "ACQUIRE_RESOURCE".to_string(),
            location: location.clone(),
        })?;
        
        let acquisition = acquisition.unwrap_or_else(|| Expression::NullLiteral {
            source_location: location.clone(),
        });
        
        Ok(ResourceAcquisition {
            resource_type,
            binding,
            acquisition,
            cleanup,
            type_spec,
            parameters,
        })
    }
    
    /// Parse cleanup specification
    fn parse_cleanup_spec(&mut self) -> Result<resource::CleanupSpecification, ParserError> {
        use crate::ast::resource::*;
        
        match self.current_token().map(|t| t.token_type.clone()) {
            Some(TokenType::String(s)) => {
                self.advance();
                Ok(CleanupSpecification::Function {
                    name: s,
                    pass_resource: true,
                })
            }
            Some(TokenType::Keyword(k)) if k == "AUTOMATIC" => {
                self.advance();
                Ok(CleanupSpecification::Automatic)
            }
            Some(_) => {
                let expr = self.parse_expression()?;
                Ok(CleanupSpecification::Expression(expr))
            }
            None => Ok(CleanupSpecification::Automatic)
        }
    }
}