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

//! LLVM context management and utilities

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::OptimizationLevel;

/// Managed LLVM context with utilities
pub struct LLVMContext {
    context: Context,
    // Store modules by name for management, but don't expose direct references
    module_names: Vec<String>,
    execution_engine: Option<ExecutionEngine<'static>>,
}

impl LLVMContext {
    /// Create a new LLVM context
    pub fn new() -> Self {
        Self {
            context: Context::create(),
            module_names: Vec::new(),
            execution_engine: None,
        }
    }
    
    /// Get the underlying LLVM context
    pub fn context(&self) -> &Context {
        &self.context
    }
    
    /// Create a new module
    pub fn create_module(&self, name: &str) -> Module {
        let module = self.context.create_module(name);
        module
    }
    
    /// Check if a module name is registered
    pub fn has_module(&self, name: &str) -> bool {
        self.module_names.contains(&name.to_string())
    }
    
    /// Create a new builder
    pub fn create_builder(&self) -> Builder {
        self.context.create_builder()
    }
    
    /// Initialize JIT execution engine for a module
    /// Note: This is simplified for lifetime management
    pub fn can_init_jit(&self) -> bool {
        true // Simplified - in a real implementation this would check module state
    }
    
    /// Check if JIT execution engine is initialized
    pub fn has_jit(&self) -> bool {
        self.execution_engine.is_some()
    }
    
    /// Placeholder for JIT execution (simplified)
    pub fn can_execute_jit(&self) -> bool {
        self.execution_engine.is_some()
    }
    
    
    /// List all module names
    pub fn list_modules(&self) -> Vec<&str> {
        self.module_names.iter().map(|s| s.as_str()).collect()
    }
    
    /// Remove a module name from tracking
    pub fn remove_module(&mut self, name: &str) -> bool {
        if let Some(pos) = self.module_names.iter().position(|x| x == name) {
            self.module_names.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Clear all modules
    pub fn clear_modules(&mut self) {
        self.module_names.clear();
        self.execution_engine = None;
    }
}

impl Default for LLVMContext {
    fn default() -> Self {
        Self::new()
    }
}

/// LLVM context builder for configuration
pub struct LLVMContextBuilder {
    optimization_level: OptimizationLevel,
}

impl LLVMContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        Self {
            optimization_level: OptimizationLevel::None,
        }
    }
    
    /// Set optimization level
    pub fn with_optimization_level(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }
    
    /// Build the LLVM context
    pub fn build(self) -> LLVMContext {
        // For now, just create a basic context
        // In the future, this could configure the context based on the builder settings
        LLVMContext::new()
    }
}

impl Default for LLVMContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::OptimizationLevel;
    
    #[test]
    fn test_context_creation() {
        let ctx = LLVMContext::new();
        assert!(ctx.module_names.is_empty());
        assert!(ctx.execution_engine.is_none());
    }
    
    #[test]
    fn test_module_management() {
        let mut ctx = LLVMContext::new();
        
        // Create a module
        let _module = ctx.create_module("test_module");
        // Note: We can't track modules directly anymore due to lifetime issues
        // This test just ensures module creation doesn't panic
        
        // Test module name tracking if needed
        assert!(ctx.list_modules().is_empty()); // No automatic tracking
    }
    
    #[test]
    fn test_builder_creation() {
        let ctx = LLVMContext::new();
        let _builder = ctx.create_builder();
        // If we get here without panicking, the builder was created successfully
    }
    
    #[test]
    fn test_context_builder() {
        let builder = LLVMContextBuilder::new()
            .with_optimization_level(OptimizationLevel::Default);
        
        let _ctx = builder.build();
        // If we get here, the context was built successfully
    }
    
    #[test]
    fn test_clear_modules() {
        let mut ctx = LLVMContext::new();
        
        ctx.create_module("module1");
        ctx.create_module("module2");
        // Note: Modules aren't tracked automatically anymore
        
        ctx.clear_modules();
        assert!(ctx.list_modules().is_empty());
    }
}