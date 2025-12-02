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

//! Package manifest format for AetherScript packages
//!
//! Defines the structure and parsing of Package.toml files that describe
//! AetherScript packages, their dependencies, and build configuration.

use crate::error::SemanticError;
use crate::package::version::{Version, VersionRequirement};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// Type alias for convenience
type VersionReq = VersionRequirement;

/// Placeholder for TOML value until toml crate is added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TomlValue {
    Table(std::collections::HashMap<String, TomlValue>),
    Array(Vec<TomlValue>),
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

/// Package manifest (Package.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Package metadata
    pub package: PackageMetadata,

    /// Dependencies
    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    /// Development dependencies
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: Vec<Dependency>,

    /// Optional dependencies
    #[serde(default, rename = "optional-dependencies")]
    pub optional_dependencies: Vec<Dependency>,

    /// Build dependencies
    #[serde(default, rename = "build-dependencies")]
    pub build_dependencies: Vec<Dependency>,

    /// Build configuration
    #[serde(default)]
    pub build: BuildConfiguration,

    /// Features
    #[serde(default)]
    pub features: HashMap<String, Feature>,

    /// Target-specific configuration
    #[serde(default)]
    pub target: HashMap<String, TargetConfiguration>,

    /// Workspace configuration
    pub workspace: Option<WorkspaceConfiguration>,

    /// Binary targets
    #[serde(default)]
    pub bin: Vec<BinaryTarget>,

    /// Library target
    pub lib: Option<LibraryTarget>,

    /// Example targets
    #[serde(default)]
    pub example: Vec<ExampleTarget>,

    /// Test targets
    #[serde(default)]
    pub test: Vec<TestTarget>,

    /// Benchmark targets
    #[serde(default)]
    pub bench: Vec<BenchmarkTarget>,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,

    /// Package version
    pub version: Version,

    /// Package description
    pub description: Option<String>,

    /// Package authors
    #[serde(default)]
    pub authors: Vec<String>,

    /// Package license
    pub license: Option<String>,

    /// License file path
    #[serde(rename = "license-file")]
    pub license_file: Option<PathBuf>,

    /// Package homepage
    pub homepage: Option<String>,

    /// Package repository
    pub repository: Option<String>,

    /// Package documentation URL
    pub documentation: Option<String>,

    /// Package keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Package categories
    #[serde(default)]
    pub categories: Vec<String>,

    /// README file path
    pub readme: Option<PathBuf>,

    /// Include/exclude patterns
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,

    /// Package edition
    #[serde(default)]
    pub edition: Edition,

    /// Minimum AetherScript version required
    #[serde(rename = "aether-version")]
    pub aether_version: Option<VersionRequirement>,

    /// Build script path
    pub build: Option<PathBuf>,

    /// Publish configuration
    pub publish: Option<PublishConfig>,

    /// Metadata table
    #[serde(default)]
    pub metadata: TomlValue,
}

/// Package edition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Edition {
    #[serde(rename = "2024")]
    Edition2024,
    #[serde(rename = "2025")]
    Edition2025,
}

/// Publishing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    /// Registries to publish to
    pub registry: Option<Vec<String>>,

    /// Whether to publish at all
    pub publish: Option<bool>,
}

/// Package dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name
    pub name: String,

    /// Version requirement
    pub version: VersionRequirement,

    /// Git repository (alternative to version)
    pub git: Option<String>,

    /// Git branch
    pub branch: Option<String>,

    /// Git tag
    pub tag: Option<String>,

    /// Git revision
    pub rev: Option<String>,

    /// Local path (alternative to version)
    pub path: Option<PathBuf>,

    /// Registry to use
    pub registry: Option<String>,

    /// Features to enable
    #[serde(default)]
    pub features: Vec<String>,

    /// Whether this is optional
    #[serde(default)]
    pub optional: bool,

    /// Default features flag
    #[serde(default = "default_true", rename = "default-features")]
    pub default_features: bool,

    /// Package name (if different from dependency key)
    pub package: Option<String>,
}

/// Package feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Description of the feature
    pub description: Option<String>,

    /// Dependencies enabled by this feature
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Other features enabled by this feature
    #[serde(default)]
    pub features: Vec<String>,

    /// Whether this is a default feature
    #[serde(default)]
    pub default: bool,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfiguration {
    /// Build script path
    pub script: Option<PathBuf>,

    /// Additional source directories
    #[serde(default)]
    pub sources: Vec<PathBuf>,

    /// Include directories
    #[serde(default)]
    pub include: Vec<PathBuf>,

    /// Library directories
    #[serde(default, rename = "lib-dirs")]
    pub lib_dirs: Vec<PathBuf>,

    /// Libraries to link
    #[serde(default)]
    pub libs: Vec<String>,

    /// Build flags
    #[serde(default)]
    pub flags: Vec<String>,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Optimization level
    #[serde(rename = "opt-level")]
    pub opt_level: Option<u8>,

    /// Debug information
    pub debug: Option<bool>,

    /// Link-time optimization
    pub lto: Option<bool>,

    /// Code generation units
    pub codegen_units: Option<u32>,

    /// Panic strategy
    pub panic: Option<PanicStrategy>,

    /// Overflow checks
    pub overflow_checks: Option<bool>,

    /// Debug assertions
    pub debug_assertions: Option<bool>,

    /// Incremental compilation
    pub incremental: Option<bool>,
}

/// Panic strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanicStrategy {
    #[serde(rename = "unwind")]
    Unwind,
    #[serde(rename = "abort")]
    Abort,
}

/// Target-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfiguration {
    /// Target-specific dependencies
    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    /// Target-specific build configuration
    pub build: Option<BuildConfiguration>,

    /// Target-specific features
    #[serde(default)]
    pub features: HashMap<String, Feature>,
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfiguration {
    /// Workspace members
    pub members: Vec<String>,

    /// Workspace exclusions
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Default workspace members
    #[serde(default, rename = "default-members")]
    pub default_members: Vec<String>,

    /// Workspace-level dependencies
    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    /// Package-level metadata inheritance
    #[serde(default)]
    pub package: Option<WorkspacePackageConfig>,
}

/// Workspace package configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePackageConfig {
    /// Workspace version
    pub version: Option<Version>,

    /// Workspace authors
    pub authors: Option<Vec<String>>,

    /// Workspace license
    pub license: Option<String>,

    /// Workspace homepage
    pub homepage: Option<String>,

    /// Workspace repository
    pub repository: Option<String>,

    /// Workspace edition
    pub edition: Option<Edition>,
}

/// Binary target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryTarget {
    /// Binary name
    pub name: String,

    /// Source file path
    pub path: Option<PathBuf>,

    /// Required features
    #[serde(default, rename = "required-features")]
    pub required_features: Vec<String>,

    /// Test binary
    #[serde(default)]
    pub test: bool,

    /// Benchmark binary
    #[serde(default)]
    pub bench: bool,

    /// Documentation
    #[serde(default)]
    pub doc: bool,

    /// Harness
    #[serde(default = "default_true")]
    pub harness: bool,

    /// Edition
    pub edition: Option<Edition>,
}

/// Library target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryTarget {
    /// Library name
    pub name: Option<String>,

    /// Source file path
    pub path: Option<PathBuf>,

    /// Library types
    #[serde(default)]
    pub crate_type: Vec<CrateType>,

    /// Required features
    #[serde(default, rename = "required-features")]
    pub required_features: Vec<String>,

    /// Documentation
    #[serde(default = "default_true")]
    pub doc: bool,

    /// Doctests
    #[serde(default = "default_true")]
    pub doctest: bool,

    /// Harness
    #[serde(default = "default_true")]
    pub harness: bool,

    /// Edition
    pub edition: Option<Edition>,
}

/// Library crate types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrateType {
    #[serde(rename = "lib")]
    Lib,
    #[serde(rename = "dylib")]
    DynamicLib,
    #[serde(rename = "staticlib")]
    StaticLib,
    #[serde(rename = "cdylib")]
    CDynamicLib,
}

/// Example target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleTarget {
    /// Example name
    pub name: String,

    /// Source file path
    pub path: Option<PathBuf>,

    /// Required features
    #[serde(default, rename = "required-features")]
    pub required_features: Vec<String>,

    /// Test example
    #[serde(default)]
    pub test: bool,

    /// Benchmark example
    #[serde(default)]
    pub bench: bool,

    /// Documentation
    #[serde(default)]
    pub doc: bool,

    /// Harness
    #[serde(default = "default_true")]
    pub harness: bool,

    /// Edition
    pub edition: Option<Edition>,
}

/// Test target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTarget {
    /// Test name
    pub name: String,

    /// Source file path
    pub path: Option<PathBuf>,

    /// Required features
    #[serde(default, rename = "required-features")]
    pub required_features: Vec<String>,

    /// Harness
    #[serde(default = "default_true")]
    pub harness: bool,

    /// Edition
    pub edition: Option<Edition>,
}

/// Benchmark target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTarget {
    /// Benchmark name
    pub name: String,

    /// Source file path
    pub path: Option<PathBuf>,

    /// Required features
    #[serde(default, rename = "required-features")]
    pub required_features: Vec<String>,

    /// Harness
    #[serde(default = "default_true")]
    pub harness: bool,

    /// Edition
    pub edition: Option<Edition>,
}

// Helper function for default true values
fn default_true() -> bool {
    true
}

impl PackageManifest {
    /// Load manifest from file
    pub fn load(path: &PathBuf) -> Result<Self, SemanticError> {
        let content = std::fs::read_to_string(path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read manifest file {}: {}", path.display(), e),
        })?;

        Self::from_str(&content)
    }

    /// Parse manifest from string
    pub fn from_str(content: &str) -> Result<Self, SemanticError> {
        // Simple parser for test purposes - in real implementation would use toml crate
        let mut package = PackageMetadata {
            name: String::new(),
            version: Version {
                major: 0,
                minor: 0,
                patch: 0,
                pre: vec![],
                build: vec![],
            },
            description: None,
            authors: Vec::new(),
            license: None,
            license_file: None,
            edition: Edition::default(),
            repository: None,
            homepage: None,
            documentation: None,
            readme: None,
            keywords: Vec::new(),
            categories: Vec::new(),
            include: None,
            exclude: None,
            aether_version: None,
            build: None,
            publish: None,
            metadata: TomlValue::Table(HashMap::new()),
        };

        let mut dependencies = Vec::new();
        let mut in_package = false;
        let mut in_dependencies = false;

        for line in content.lines() {
            let line = line.trim();

            if line == "[package]" {
                in_package = true;
                in_dependencies = false;
            } else if line == "[[dependencies]]" {
                in_package = false;
                in_dependencies = true;
                dependencies.push(Dependency {
                    name: String::new(),
                    version: VersionReq::any(),
                    path: None,
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    features: Vec::new(),
                    optional: false,
                    default_features: true,
                    package: None,
                    registry: None,
                });
            } else if line.starts_with('[') {
                in_package = false;
                in_dependencies = false;
            } else if line.is_empty() || line.starts_with('#') {
                continue;
            } else if let Some((key, value)) = line.split_once(" = ") {
                let value = value.trim_matches('"');

                if in_package {
                    match key {
                        "name" => package.name = value.to_string(),
                        "version" => {
                            let parts: Vec<&str> = value.split('.').collect();
                            if parts.len() == 3 {
                                package.version = Version {
                                    major: parts[0].parse().unwrap_or(0),
                                    minor: parts[1].parse().unwrap_or(0),
                                    patch: parts[2].parse().unwrap_or(0),
                                    pre: vec![],
                                    build: vec![],
                                };
                            }
                        }
                        "description" => package.description = Some(value.to_string()),
                        "license" => package.license = Some(value.to_string()),
                        "edition" => {
                            package.edition = match value {
                                "2024" => Edition::Edition2024,
                                _ => Edition::default(),
                            }
                        }
                        _ => {}
                    }
                } else if in_dependencies && !dependencies.is_empty() {
                    let dep = dependencies.last_mut().unwrap();
                    match key {
                        "name" => dep.name = value.to_string(),
                        "version" => {
                            if value == "*" {
                                dep.version = VersionReq::any();
                            } else if let Some(version_str) = value.strip_prefix('^') {
                                // Parse caret version
                                let parts: Vec<&str> = version_str.split('.').collect();
                                if parts.len() == 3 {
                                    let v = Version {
                                        major: parts[0].parse().unwrap_or(0),
                                        minor: parts[1].parse().unwrap_or(0),
                                        patch: parts[2].parse().unwrap_or(0),
                                        pre: vec![],
                                        build: vec![],
                                    };
                                    dep.version = VersionReq::caret(&v);
                                }
                            } else if let Some(version_str) = value.strip_prefix('~') {
                                // Parse tilde version
                                let parts: Vec<&str> = version_str.split('.').collect();
                                if parts.len() == 3 {
                                    let v = Version {
                                        major: parts[0].parse().unwrap_or(0),
                                        minor: parts[1].parse().unwrap_or(0),
                                        patch: parts[2].parse().unwrap_or(0),
                                        pre: vec![],
                                        build: vec![],
                                    };
                                    dep.version = VersionReq::tilde(&v);
                                }
                            } else {
                                // Parse exact version
                                let parts: Vec<&str> = value.split('.').collect();
                                if parts.len() == 3 {
                                    let v = Version {
                                        major: parts[0].parse().unwrap_or(0),
                                        minor: parts[1].parse().unwrap_or(0),
                                        patch: parts[2].parse().unwrap_or(0),
                                        pre: vec![],
                                        build: vec![],
                                    };
                                    dep.version = VersionReq::exact(&v);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(Self {
            package,
            dependencies,
            dev_dependencies: Vec::new(),
            optional_dependencies: Vec::new(),
            build_dependencies: Vec::new(),
            build: BuildConfiguration::default(),
            features: HashMap::new(),
            target: HashMap::new(),
            workspace: None,
            bin: Vec::new(),
            lib: None,
            example: Vec::new(),
            test: Vec::new(),
            bench: Vec::new(),
        })
    }

    /// Save manifest to file
    pub fn save(&self, path: &PathBuf) -> Result<(), SemanticError> {
        // Simplified serialization - in real implementation would use toml crate
        let content = format!("# Package manifest for {}\n", self.package.name);

        std::fs::write(path, content).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write manifest file {}: {}", path.display(), e),
        })
    }

    /// Get package name
    pub fn name(&self) -> &str {
        &self.package.name
    }

    /// Get package version
    pub fn version(&self) -> &Version {
        &self.package.version
    }

    /// Check if package is a library
    pub fn is_library(&self) -> bool {
        self.lib.is_some()
    }

    /// Check if package has binaries
    pub fn has_binaries(&self) -> bool {
        !self.bin.is_empty()
    }

    /// Get all dependencies (regular + dev + optional)
    pub fn all_dependencies(&self) -> Vec<&Dependency> {
        let mut deps = Vec::new();
        deps.extend(&self.dependencies);
        deps.extend(&self.dev_dependencies);
        deps.extend(&self.optional_dependencies);
        deps
    }

    /// Get dependencies for a specific target
    pub fn dependencies_for_target(&self, target: &str) -> Vec<&Dependency> {
        let mut deps = self.dependencies.iter().collect::<Vec<_>>();

        if let Some(target_config) = self.target.get(target) {
            deps.extend(&target_config.dependencies);
        }

        deps
    }

    /// Check if a feature is enabled by default
    pub fn is_default_feature(&self, feature_name: &str) -> bool {
        self.features
            .get(feature_name)
            .map(|f| f.default)
            .unwrap_or(false)
    }

    /// Get all default features
    pub fn default_features(&self) -> Vec<&str> {
        self.features
            .iter()
            .filter(|(_, feature)| feature.default)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Validate manifest
    pub fn validate(&self) -> Result<(), SemanticError> {
        // Validate package name
        if self.package.name.is_empty() {
            return Err(SemanticError::Internal {
                message: "Package name cannot be empty".to_string(),
            });
        }

        if !is_valid_package_name(&self.package.name) {
            return Err(SemanticError::Internal {
                message: format!("Invalid package name: {}", self.package.name),
            });
        }

        // Validate dependencies
        for dep in &self.dependencies {
            self.validate_dependency(dep)?;
        }

        for dep in &self.dev_dependencies {
            self.validate_dependency(dep)?;
        }

        for dep in &self.optional_dependencies {
            self.validate_dependency(dep)?;
        }

        // Validate features
        for (name, feature) in &self.features {
            self.validate_feature(name, feature)?;
        }

        // Validate targets
        if let Some(ref lib) = self.lib {
            self.validate_library_target(lib)?;
        }

        for bin in &self.bin {
            self.validate_binary_target(bin)?;
        }

        Ok(())
    }

    fn validate_dependency(&self, dep: &Dependency) -> Result<(), SemanticError> {
        if dep.name.is_empty() {
            return Err(SemanticError::Internal {
                message: "Dependency name cannot be empty".to_string(),
            });
        }

        // Check that version, git, or path is specified
        let has_version = !dep.version.is_any();
        let has_git = dep.git.is_some();
        let has_path = dep.path.is_some();

        if !has_version && !has_git && !has_path {
            return Err(SemanticError::Internal {
                message: format!("Dependency {} must specify version, git, or path", dep.name),
            });
        }

        Ok(())
    }

    fn validate_feature(&self, name: &str, feature: &Feature) -> Result<(), SemanticError> {
        if name.is_empty() {
            return Err(SemanticError::Internal {
                message: "Feature name cannot be empty".to_string(),
            });
        }

        // Validate feature dependencies
        for dep_name in &feature.dependencies {
            if !self.dependencies.iter().any(|d| &d.name == dep_name)
                && !self
                    .optional_dependencies
                    .iter()
                    .any(|d| &d.name == dep_name)
            {
                return Err(SemanticError::Internal {
                    message: format!(
                        "Feature {} references unknown dependency {}",
                        name, dep_name
                    ),
                });
            }
        }

        Ok(())
    }

    fn validate_library_target(&self, lib: &LibraryTarget) -> Result<(), SemanticError> {
        if let Some(ref path) = lib.path {
            if !path.exists() {
                return Err(SemanticError::Internal {
                    message: format!("Library source file does not exist: {}", path.display()),
                });
            }
        }

        Ok(())
    }

    fn validate_binary_target(&self, bin: &BinaryTarget) -> Result<(), SemanticError> {
        if bin.name.is_empty() {
            return Err(SemanticError::Internal {
                message: "Binary name cannot be empty".to_string(),
            });
        }

        if let Some(ref path) = bin.path {
            if !path.exists() {
                return Err(SemanticError::Internal {
                    message: format!("Binary source file does not exist: {}", path.display()),
                });
            }
        }

        Ok(())
    }
}

impl Default for Edition {
    fn default() -> Self {
        Edition::Edition2024
    }
}

impl Default for BuildConfiguration {
    fn default() -> Self {
        Self {
            script: None,
            sources: vec![],
            include: vec![],
            lib_dirs: vec![],
            libs: vec![],
            flags: vec![],
            env: HashMap::new(),
            opt_level: None,
            debug: None,
            lto: None,
            codegen_units: None,
            panic: None,
            overflow_checks: None,
            debug_assertions: None,
            incremental: None,
        }
    }
}

impl Default for TomlValue {
    fn default() -> Self {
        TomlValue::Table(HashMap::new())
    }
}

/// Check if package name is valid
fn is_valid_package_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }

    // Must start with alphabetic character
    if !name.chars().next().unwrap_or('0').is_alphabetic() {
        return false;
    }

    // Can only contain alphanumeric, hyphens, and underscores
    name.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
#[path = "manifest_tests.rs"]
mod manifest_tests;
