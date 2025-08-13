//! Debugging and tooling support for AetherScript
//!
//! This module provides comprehensive debugging infrastructure including:
//! - DWARF debug information generation
//! - Debugger integration (GDB/LLDB)
//! - Language Server Protocol (LSP) support
//! - Development tooling utilities

pub mod dwarf;
pub mod debugger;
pub mod lsp;
pub mod source_map;
pub mod breakpoints;

use crate::error::SemanticError;
use crate::mir::Program;
use crate::llvm_backend::LLVMBackend;

/// Debug information configuration
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// Generate DWARF debug information
    pub generate_debug_info: bool,
    
    /// Debug information level (0-3)
    pub debug_level: u8,
    
    /// Include source line information
    pub include_line_info: bool,
    
    /// Include variable information
    pub include_variable_info: bool,
    
    /// Enable debugger integration
    pub enable_debugger: bool,
    
    /// LSP server configuration
    pub lsp_config: LspConfig,
}

/// Language Server Protocol configuration
#[derive(Debug, Clone)]
pub struct LspConfig {
    /// Enable LSP server
    pub enabled: bool,
    
    /// Server port
    pub port: u16,
    
    /// Enable hover information
    pub enable_hover: bool,
    
    /// Enable auto-completion
    pub enable_completion: bool,
    
    /// Enable diagnostics
    pub enable_diagnostics: bool,
    
    /// Enable go-to-definition
    pub enable_goto_definition: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            generate_debug_info: true,
            debug_level: 2,
            include_line_info: true,
            include_variable_info: true,
            enable_debugger: true,
            lsp_config: LspConfig::default(),
        }
    }
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 7878,
            enable_hover: true,
            enable_completion: true,
            enable_diagnostics: true,
            enable_goto_definition: true,
        }
    }
}

/// Main debugging support interface
#[derive(Debug)]
pub struct DebugSupport {
    config: DebugConfig,
    dwarf_generator: dwarf::DwarfGenerator,
    debugger_interface: debugger::DebuggerInterface,
    lsp_server: Option<lsp::LanguageServer>,
    source_map: source_map::SourceMapGenerator,
}

impl DebugSupport {
    pub fn new(config: DebugConfig) -> Self {
        Self {
            dwarf_generator: dwarf::DwarfGenerator::new(config.debug_level),
            debugger_interface: debugger::DebuggerInterface::new(),
            lsp_server: if config.lsp_config.enabled {
                // Convert from debug::LspConfig to lsp::LspConfig
                let lsp_config = lsp::LspConfig {
                    capabilities: lsp::ServerCapabilities::default(),
                    port: config.lsp_config.port,
                    max_cached_documents: 100,
                    real_time_diagnostics: config.lsp_config.enable_diagnostics,
                    completion_triggers: vec![".".to_string(), "(".to_string()],
                };
                Some(lsp::LanguageServer::new(lsp_config))
            } else {
                None
            },
            source_map: source_map::SourceMapGenerator::new(),
            config,
        }
    }
    
    /// Generate debug information for a program
    pub fn generate_debug_info(&mut self, program: &Program, backend: &mut LLVMBackend) -> Result<(), SemanticError> {
        if self.config.generate_debug_info {
            self.dwarf_generator.generate_for_program(program, backend)?;
        }
        
        if self.config.include_line_info {
            self.source_map.generate_for_program(program)?;
        }
        
        Ok(())
    }
    
    /// Start LSP server if enabled
    pub fn start_lsp_server(&mut self) -> Result<(), SemanticError> {
        if let Some(ref mut server) = self.lsp_server {
            server.start()?;
        }
        Ok(())
    }
    
    /// Initialize debugger integration
    pub fn initialize_debugger(&mut self, program: &Program) -> Result<(), SemanticError> {
        if self.config.enable_debugger {
            self.debugger_interface.initialize(program)?;
        }
        Ok(())
    }
    
    /// Set breakpoint at source location
    pub fn set_breakpoint(&mut self, file: &str, line: u32) -> Result<(), SemanticError> {
        self.debugger_interface.set_breakpoint(file, line)
    }
    
    /// Get debugging configuration
    pub fn config(&self) -> &DebugConfig {
        &self.config
    }
    
    /// Update debugging configuration
    pub fn update_config(&mut self, config: DebugConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_debug_config_default() {
        let config = DebugConfig::default();
        assert!(config.generate_debug_info);
        assert_eq!(config.debug_level, 2);
        assert!(config.include_line_info);
        assert!(config.include_variable_info);
        assert!(config.enable_debugger);
    }
    
    #[test]
    fn test_lsp_config_default() {
        let config = LspConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.port, 7878);
        assert!(config.enable_hover);
        assert!(config.enable_completion);
        assert!(config.enable_diagnostics);
        assert!(config.enable_goto_definition);
    }
    
    #[test]
    fn test_debug_support_creation() {
        let config = DebugConfig::default();
        let debug_support = DebugSupport::new(config);
        
        assert_eq!(debug_support.config.debug_level, 2);
        assert!(debug_support.lsp_server.is_none()); // LSP disabled by default
    }
    
    #[test]
    fn test_debug_support_with_lsp() {
        let mut config = DebugConfig::default();
        config.lsp_config.enabled = true;
        
        let debug_support = DebugSupport::new(config);
        assert!(debug_support.lsp_server.is_some());
    }
}