//! Resource Management Module for AetherScript
//! 
//! Provides deterministic resource management with explicit scopes
//! and automatic cleanup guarantees.

pub mod analysis;

pub use analysis::{
    ResourceAnalyzer,
    ResourceAnalysisResults,
    ResourceLeak,
    DoubleRelease,
    UseAfterRelease,
    ContractViolation,
    UsagePattern,
    ResourceOptimization,
    OptimizationType,
    OptimizationBenefit,
};

use crate::error::{CompilerError, SemanticError};

/// High-level resource management validator
pub struct ResourceManager {
    analyzer: ResourceAnalyzer,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            analyzer: ResourceAnalyzer::new(),
        }
    }
    
    /// Validate resource usage in a module
    pub fn validate_module(&mut self, module: &crate::ast::Module) -> Result<(), CompilerError> {
        // Analyze all functions in the module
        for function in &module.function_definitions {
            self.analyzer.analyze_function(function)
                .map_err(CompilerError::SemanticError)?;
        }
        
        // Generate optimization suggestions
        self.analyzer.generate_optimizations();
        
        // Check results and report errors
        let results = self.analyzer.get_results();
        
        // Report leaks as errors
        for leak in &results.leaks {
            return Err(CompilerError::SemanticError(
                SemanticError::ResourceLeak {
                    resource_type: leak.resource_type.clone(),
                    binding: leak.binding.clone(),
                    location: leak.acquisition_location.clone(),
                }
            ));
        }
        
        // Report double releases
        for double_release in &results.double_releases {
            return Err(CompilerError::SemanticError(
                SemanticError::InvalidOperation {
                    operation: "double release".to_string(),
                    reason: format!("Resource '{}' was released twice", double_release.binding),
                    location: double_release.second_release.clone(),
                }
            ));
        }
        
        // Report use after release
        for use_after_release in &results.use_after_release {
            return Err(CompilerError::SemanticError(
                SemanticError::InvalidOperation {
                    operation: "use after release".to_string(),
                    reason: format!("Resource '{}' used after being released", use_after_release.binding),
                    location: use_after_release.use_location.clone(),
                }
            ));
        }
        
        Ok(())
    }
    
    /// Get analysis results
    pub fn get_results(&self) -> &ResourceAnalysisResults {
        self.analyzer.get_results()
    }
    
    /// Get optimization suggestions
    pub fn get_optimizations(&self) -> Vec<ResourceOptimization> {
        self.analyzer.get_results().optimizations.clone()
    }
}

/// Resource validation pass for the compiler pipeline
pub struct ResourceValidationPass {
    manager: ResourceManager,
}

impl ResourceValidationPass {
    pub fn new() -> Self {
        Self {
            manager: ResourceManager::new(),
        }
    }
    
    pub fn run(&mut self, program: &crate::ast::Program) -> Result<ResourceReport, CompilerError> {
        let mut report = ResourceReport::default();
        
        for module in &program.modules {
            match self.manager.validate_module(module) {
                Ok(()) => {
                    let results = self.manager.get_results();
                    report.max_concurrent_resources = results.max_concurrent_resources.max(report.max_concurrent_resources);
                    report.optimizations.extend(results.optimizations.clone());
                    report.usage_patterns.extend(results.usage_patterns.clone());
                }
                Err(e) => {
                    report.errors.push(e);
                }
            }
        }
        
        if report.errors.is_empty() {
            Ok(report)
        } else {
            Err(report.errors[0].clone())
        }
    }
}

/// Resource analysis report
#[derive(Debug, Clone, Default)]
pub struct ResourceReport {
    pub max_concurrent_resources: usize,
    pub optimizations: Vec<ResourceOptimization>,
    pub usage_patterns: std::collections::HashMap<String, UsagePattern>,
    pub errors: Vec<CompilerError>,
}

impl ResourceReport {
    /// Format report as S-expression for LLM consumption
    pub fn to_sexp(&self) -> String {
        let mut result = String::from("(RESOURCE_ANALYSIS_REPORT\n");
        
        result.push_str(&format!("  (MAX_CONCURRENT_RESOURCES {})\n", self.max_concurrent_resources));
        
        if !self.optimizations.is_empty() {
            result.push_str("  (OPTIMIZATIONS\n");
            for opt in &self.optimizations {
                result.push_str(&format!("    (OPTIMIZATION\n"));
                result.push_str(&format!("      (TYPE {:?})\n", opt.optimization_type));
                result.push_str(&format!("      (RESOURCE \"{}\")\n", opt.resource_type));
                result.push_str(&format!("      (DESCRIPTION \"{}\")\n", opt.description));
                
                if let Some(memory) = opt.estimated_benefit.memory_saved_mb {
                    result.push_str(&format!("      (MEMORY_SAVED_MB {})\n", memory));
                }
                if let Some(latency) = opt.estimated_benefit.latency_reduced_ms {
                    result.push_str(&format!("      (LATENCY_REDUCED_MS {})\n", latency));
                }
                
                result.push_str("    )\n");
            }
            result.push_str("  )\n");
        }
        
        if !self.usage_patterns.is_empty() {
            result.push_str("  (USAGE_PATTERNS\n");
            for (resource_type, pattern) in &self.usage_patterns {
                result.push_str(&format!("    (PATTERN\n"));
                result.push_str(&format!("      (RESOURCE_TYPE \"{}\")\n", resource_type));
                result.push_str(&format!("      (AVG_HOLD_TIME_MS {})\n", pattern.avg_hold_time));
                result.push_str(&format!("      (MAX_HOLD_TIME_MS {})\n", pattern.max_hold_time));
                result.push_str(&format!("      (ACCESS_FREQUENCY {})\n", pattern.access_frequency));
                result.push_str(&format!("      (TYPICAL_COUNT {})\n", pattern.typical_count));
                result.push_str("    )\n");
            }
            result.push_str("  )\n");
        }
        
        result.push_str(")\n");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Module, Identifier};
    use crate::error::SourceLocation;
    
    #[test]
    fn test_resource_manager_creation() {
        let manager = ResourceManager::new();
        let results = manager.get_results();
        assert_eq!(results.max_concurrent_resources, 0);
        assert!(results.leaks.is_empty());
    }
    
    #[test]
    fn test_resource_report_sexp() {
        let mut report = ResourceReport::default();
        report.max_concurrent_resources = 5;
        
        report.optimizations.push(ResourceOptimization {
            optimization_type: OptimizationType::UsePool,
            resource_type: "file_handle".to_string(),
            location: SourceLocation::unknown(),
            description: "Consider pooling file handles".to_string(),
            estimated_benefit: OptimizationBenefit {
                memory_saved_mb: Some(2.5),
                latency_reduced_ms: Some(10.0),
                resource_count_reduced: Some(3),
            },
        });
        
        let sexp = report.to_sexp();
        assert!(sexp.contains("(MAX_CONCURRENT_RESOURCES 5)"));
        assert!(sexp.contains("(TYPE UsePool)"));
        assert!(sexp.contains("(MEMORY_SAVED_MB 2.5)"));
    }
}