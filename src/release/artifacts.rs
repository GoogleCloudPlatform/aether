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

//! Release artifact generation and management for AetherScript
//! 
//! Handles creation, validation, and management of release artifacts including
//! binaries, documentation, checksums, signatures, and metadata files.

use crate::error::SemanticError;
use crate::release::{ProjectInfo, VersionInfo};
use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Artifact manager for generating and organizing release artifacts
#[derive(Debug)]
pub struct ArtifactManager {
    /// Manager configuration
    config: ArtifactConfig,
    
    /// Project information
    project: ProjectInfo,
    
    /// Version information
    version: VersionInfo,
    
    /// Generated artifacts
    artifacts: HashMap<String, Artifact>,
    
    /// Artifact sets
    artifact_sets: Vec<ArtifactSet>,
    
    /// Validation rules
    validation_rules: Vec<ValidationRule>,
    
    /// Signing configuration
    signing: Option<SigningConfig>,
}

/// Artifact configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactConfig {
    /// Output directory for artifacts
    pub output_dir: PathBuf,
    
    /// Temporary directory for building
    pub temp_dir: PathBuf,
    
    /// Artifact types to generate
    pub artifact_types: Vec<ArtifactType>,
    
    /// Platform targets
    pub platforms: Vec<PlatformConfig>,
    
    /// Build configurations
    pub build_configs: Vec<BuildConfiguration>,
    
    /// Compression settings
    pub compression: CompressionConfig,
    
    /// Metadata settings
    pub metadata: MetadataConfig,
    
    /// Naming conventions
    pub naming: NamingConfig,
    
    /// Validation settings
    pub validation: ValidationConfig,
    
    /// Publishing settings
    pub publishing: PublishingConfig,
}

/// Artifact types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Executable binary
    Binary {
        name: String,
        target: String,
    },
    
    /// Shared library
    Library {
        name: String,
        target: String,
        version: String,
    },
    
    /// Static library
    StaticLibrary {
        name: String,
        target: String,
    },
    
    /// Documentation
    Documentation {
        format: DocumentationFormat,
        output_name: String,
    },
    
    /// Source code archive
    SourceArchive {
        format: ArchiveFormat,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
    },
    
    /// Distribution package
    Package {
        format: PackageFormat,
        platform: String,
    },
    
    /// Container image
    ContainerImage {
        registry: String,
        repository: String,
        tags: Vec<String>,
    },
    
    /// Checksum file
    Checksum {
        algorithms: Vec<ChecksumAlgorithm>,
        format: ChecksumFormat,
    },
    
    /// Digital signature
    Signature {
        algorithm: SignatureAlgorithm,
        format: SignatureFormat,
    },
    
    /// Metadata file
    Metadata {
        format: MetadataFormat,
        content: MetadataContent,
    },
    
    /// Release notes
    ReleaseNotes {
        format: DocumentationFormat,
        template: Option<String>,
    },
    
    /// License file
    License {
        license_type: String,
        format: DocumentationFormat,
    },
    
    /// Installation script
    InstallScript {
        platform: String,
        script_type: ScriptType,
    },
    
    /// Configuration template
    ConfigTemplate {
        name: String,
        format: ConfigFormat,
    },
    
    /// Custom artifact
    Custom {
        name: String,
        generator: String,
        parameters: HashMap<String, String>,
    },
}

/// Documentation formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentationFormat {
    Html,
    Pdf,
    Markdown,
    RestructuredText,
    AsciiDoc,
    ManPage,
    Epub,
}

/// Archive formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiveFormat {
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    Zip,
    SevenZip,
}

/// Package formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageFormat {
    Deb,
    Rpm,
    Msi,
    Pkg,
    Dmg,
    AppImage,
    Snap,
    Flatpak,
}

/// Checksum algorithms
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    MD5,
    SHA1,
    SHA256,
    SHA512,
    Blake2b,
    Blake3,
}

/// Checksum formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChecksumFormat {
    /// Simple text format (algorithm: checksum filename)
    Simple,
    
    /// BSD format
    BSD,
    
    /// GNU format
    GNU,
    
    /// JSON format
    JSON,
    
    /// XML format
    XML,
}

/// Signature algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    RSA2048,
    RSA4096,
    ECDSA256,
    ECDSA384,
    Ed25519,
}

/// Signature formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureFormat {
    PEM,
    DER,
    PKCS7,
    GPG,
    Detached,
}

/// Metadata formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataFormat {
    JSON,
    YAML,
    TOML,
    XML,
    Plist,
}

/// Metadata content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetadataContent {
    ProjectInfo,
    BuildInfo,
    ReleaseInfo,
    DependencyInfo,
    SecurityInfo,
    PerformanceMetrics,
    Custom(String),
}

/// Script types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptType {
    Shell,
    PowerShell,
    Batch,
    Python,
    JavaScript,
    Ruby,
}

/// Configuration formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFormat {
    JSON,
    YAML,
    TOML,
    XML,
    INI,
    Properties,
}

/// Platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Platform identifier
    pub name: String,
    
    /// Operating system
    pub os: String,
    
    /// Architecture
    pub arch: String,
    
    /// ABI variant
    pub abi: Option<String>,
    
    /// Minimum OS version
    pub min_os_version: Option<String>,
    
    /// Platform-specific options
    pub options: HashMap<String, String>,
    
    /// Cross-compilation settings
    pub cross_compile: Option<CrossCompileConfig>,
}

/// Cross-compilation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCompileConfig {
    /// Target triple
    pub target: String,
    
    /// Toolchain path
    pub toolchain: Option<PathBuf>,
    
    /// Sysroot path
    pub sysroot: Option<PathBuf>,
    
    /// Environment variables
    pub env: HashMap<String, String>,
    
    /// Linker settings
    pub linker: Option<LinkerConfig>,
}

/// Linker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkerConfig {
    /// Linker executable
    pub linker: String,
    
    /// Linker flags
    pub flags: Vec<String>,
    
    /// Library paths
    pub library_paths: Vec<PathBuf>,
    
    /// Libraries to link
    pub libraries: Vec<String>,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfiguration {
    /// Configuration name
    pub name: String,
    
    /// Build profile (debug, release, etc.)
    pub profile: String,
    
    /// Optimization level
    pub optimization: OptimizationLevel,
    
    /// Debug information
    pub debug_info: DebugInfoLevel,
    
    /// Feature flags
    pub features: Vec<String>,
    
    /// Environment variables
    pub env: HashMap<String, String>,
    
    /// Compiler flags
    pub compiler_flags: Vec<String>,
    
    /// Linker flags
    pub linker_flags: Vec<String>,
    
    /// Target-specific settings
    pub target_settings: HashMap<String, String>,
}

/// Optimization levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Size,
    Speed,
    Aggressive,
    Custom(String),
}

/// Debug information levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugInfoLevel {
    None,
    Minimal,
    Full,
    Custom(String),
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    
    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,
    
    /// Compression level
    pub level: u8,
    
    /// Minimum file size for compression
    pub min_size: u64,
    
    /// File patterns to compress
    pub include_patterns: Vec<String>,
    
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
}

/// Compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Bzip2,
    Xz,
    Zstd,
    Lz4,
    Brotli,
}

/// Metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConfig {
    /// Include build metadata
    pub include_build_info: bool,
    
    /// Include dependency information
    pub include_dependencies: bool,
    
    /// Include performance metrics
    pub include_metrics: bool,
    
    /// Include security information
    pub include_security: bool,
    
    /// Custom metadata fields
    pub custom_fields: HashMap<String, String>,
    
    /// Metadata format
    pub format: MetadataFormat,
    
    /// Generate schema
    pub generate_schema: bool,
}

/// Naming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConfig {
    /// Naming template
    pub template: String,
    
    /// Date format
    pub date_format: String,
    
    /// Version format
    pub version_format: String,
    
    /// Platform format
    pub platform_format: String,
    
    /// Custom variables
    pub variables: HashMap<String, String>,
    
    /// Case transformation
    pub case_transform: CaseTransform,
}

/// Case transformation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaseTransform {
    None,
    Lower,
    Upper,
    Camel,
    Snake,
    Kebab,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enable validation
    pub enabled: bool,
    
    /// Validation rules
    pub rules: Vec<ValidationRuleConfig>,
    
    /// Fail on validation errors
    pub fail_on_error: bool,
    
    /// Generate validation report
    pub generate_report: bool,
    
    /// Validation timeout
    pub timeout: u64,
}

/// Validation rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRuleConfig {
    /// Rule name
    pub name: String,
    
    /// Rule type
    pub rule_type: ValidationRuleType,
    
    /// Rule parameters
    pub parameters: HashMap<String, String>,
    
    /// Rule severity
    pub severity: ValidationSeverity,
    
    /// Apply to artifact types
    pub apply_to: Vec<String>,
}

/// Validation rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRuleType {
    FileSize,
    FileExists,
    Checksum,
    Signature,
    Executable,
    Library,
    Archive,
    Custom(String),
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Publishing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingConfig {
    /// Enable publishing
    pub enabled: bool,
    
    /// Publishing targets
    pub targets: Vec<PublishingTarget>,
    
    /// Publishing metadata
    pub metadata: HashMap<String, String>,
    
    /// Pre-publish validation
    pub validate_before_publish: bool,
    
    /// Post-publish verification
    pub verify_after_publish: bool,
}

/// Publishing target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingTarget {
    /// Target name
    pub name: String,
    
    /// Target type
    pub target_type: PublishingTargetType,
    
    /// Target configuration
    pub config: HashMap<String, String>,
    
    /// Authentication
    pub auth: Option<PublishingAuth>,
    
    /// Include patterns
    pub include_patterns: Vec<String>,
    
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
}

/// Publishing target types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublishingTargetType {
    GitHub,
    GitLab,
    Registry,
    S3,
    CDN,
    Custom(String),
}

/// Publishing authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingAuth {
    /// Authentication type
    pub auth_type: AuthType,
    
    /// Credentials
    pub credentials: HashMap<String, String>,
}

/// Authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    Token,
    OAuth2,
    BasicAuth,
    Certificate,
    Custom(String),
}

/// Signing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    /// Signing key path
    pub key_path: PathBuf,
    
    /// Key passphrase
    pub passphrase: Option<String>,
    
    /// Signing algorithm
    pub algorithm: SignatureAlgorithm,
    
    /// Certificate chain
    pub cert_chain: Option<PathBuf>,
    
    /// Timestamp server
    pub timestamp_server: Option<String>,
    
    /// Sign all artifacts
    pub sign_all: bool,
    
    /// Signature format
    pub format: SignatureFormat,
}

/// Artifact representation
#[derive(Debug, Clone)]
pub struct Artifact {
    /// Artifact identifier
    pub id: String,
    
    /// Artifact name
    pub name: String,
    
    /// Artifact type
    pub artifact_type: ArtifactType,
    
    /// File path
    pub path: PathBuf,
    
    /// File size
    pub size: u64,
    
    /// Target platform
    pub platform: Option<String>,
    
    /// Build configuration
    pub build_config: Option<String>,
    
    /// Checksums
    pub checksums: HashMap<ChecksumAlgorithm, String>,
    
    /// Digital signature
    pub signature: Option<String>,
    
    /// Creation timestamp
    pub created_at: std::time::SystemTime,
    
    /// Metadata
    pub metadata: ArtifactMetadata,
    
    /// Dependencies
    pub dependencies: Vec<String>,
    
    /// Validation results
    pub validation: Option<ValidationResult>,
}

/// Artifact metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Content type
    pub content_type: String,
    
    /// Description
    pub description: String,
    
    /// Tags
    pub tags: Vec<String>,
    
    /// Build information
    pub build_info: Option<BuildInfo>,
    
    /// License information
    pub license: Option<String>,
    
    /// Custom properties
    pub properties: HashMap<String, String>,
}

/// Build information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    /// Build timestamp
    pub timestamp: String,
    
    /// Build host
    pub host: String,
    
    /// Build user
    pub user: String,
    
    /// Compiler version
    pub compiler_version: String,
    
    /// Build flags
    pub build_flags: Vec<String>,
    
    /// Git information
    pub git_info: Option<GitInfo>,
    
    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

/// Git information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    /// Commit hash
    pub commit: String,
    
    /// Branch name
    pub branch: String,
    
    /// Tag name
    pub tag: Option<String>,
    
    /// Repository URL
    pub repository: String,
    
    /// Commit timestamp
    pub commit_timestamp: String,
    
    /// Commit author
    pub author: String,
    
    /// Commit message
    pub message: String,
}

/// Artifact set
#[derive(Debug, Clone)]
pub struct ArtifactSet {
    /// Set name
    pub name: String,
    
    /// Set description
    pub description: String,
    
    /// Artifacts in the set
    pub artifacts: Vec<String>,
    
    /// Set metadata
    pub metadata: HashMap<String, String>,
    
    /// Release stage
    pub stage: ReleaseStage,
    
    /// Target audience
    pub audience: Vec<String>,
}

/// Release stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReleaseStage {
    Development,
    Alpha,
    Beta,
    ReleaseCandidate,
    Stable,
    LongTermSupport,
    Deprecated,
}

/// Validation rule
pub struct ValidationRule {
    /// Rule configuration
    pub config: ValidationRuleConfig,
    
    /// Rule implementation
    pub validator: Box<dyn ArtifactValidator>,
}

impl std::fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValidationRule")
            .field("config", &self.config)
            .field("validator", &"<ArtifactValidator>")
            .finish()
    }
}

/// Artifact validator trait
pub trait ArtifactValidator {
    /// Validate an artifact
    fn validate(&self, artifact: &Artifact) -> Result<ValidationResult, SemanticError>;
}

/// Validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Validation status
    pub status: ValidationStatus,
    
    /// Validation messages
    pub messages: Vec<ValidationMessage>,
    
    /// Validation duration
    pub duration: std::time::Duration,
    
    /// Validation timestamp
    pub timestamp: std::time::SystemTime,
}

/// Validation status
#[derive(Debug, Clone)]
pub enum ValidationStatus {
    Passed,
    Warning,
    Failed,
    Error,
}

/// Validation message
#[derive(Debug, Clone)]
pub struct ValidationMessage {
    /// Message severity
    pub severity: ValidationSeverity,
    
    /// Message text
    pub message: String,
    
    /// Rule name
    pub rule: String,
    
    /// Additional context
    pub context: HashMap<String, String>,
}

impl ArtifactManager {
    /// Create a new artifact manager
    pub fn new(config: ArtifactConfig, project: ProjectInfo, version: VersionInfo) -> Result<Self, SemanticError> {
        // Create output directories
        std::fs::create_dir_all(&config.output_dir).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create output directory: {}", e),
        })?;
        
        std::fs::create_dir_all(&config.temp_dir).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create temp directory: {}", e),
        })?;
        
        Ok(Self {
            config,
            project,
            version,
            artifacts: HashMap::new(),
            artifact_sets: Vec::new(),
            validation_rules: Vec::new(),
            signing: None,
        })
    }
    
    /// Generate all artifacts
    pub fn generate_artifacts(&mut self) -> Result<Vec<Artifact>, SemanticError> {
        println!("Generating release artifacts");
        
        // Generate artifacts for each type and platform
        for artifact_type in &self.config.artifact_types.clone() {
            for platform in &self.config.platforms {
                for build_config in &self.config.build_configs {
                    if let Ok(artifact) = self.generate_artifact(artifact_type, Some(platform), Some(build_config)) {
                        self.artifacts.insert(artifact.id.clone(), artifact);
                    }
                }
            }
        }
        
        // Generate checksums for all artifacts
        self.generate_checksums()?;
        
        // Sign artifacts if configured
        if self.signing.is_some() {
            self.sign_artifacts()?;
        }
        
        // Validate artifacts
        self.validate_artifacts()?;
        
        // Create artifact sets
        self.create_artifact_sets()?;
        
        Ok(self.artifacts.values().cloned().collect())
    }
    
    /// Generate a specific artifact
    fn generate_artifact(&self, artifact_type: &ArtifactType, platform: Option<&PlatformConfig>, build_config: Option<&BuildConfiguration>) -> Result<Artifact, SemanticError> {
        let artifact_name = self.generate_artifact_name(artifact_type, platform, build_config);
        let artifact_path = self.config.output_dir.join(&artifact_name);
        
        println!("Generating artifact: {}", artifact_name);
        
        match artifact_type {
            ArtifactType::Binary { name, target } => {
                self.generate_binary(name, target, &artifact_path, platform, build_config)
            }
            ArtifactType::Library { name, target, version } => {
                self.generate_library(name, target, version, &artifact_path, platform, build_config)
            }
            ArtifactType::Documentation { format, output_name } => {
                self.generate_documentation(format, output_name, &artifact_path)
            }
            ArtifactType::SourceArchive { format, include_patterns, exclude_patterns } => {
                self.generate_source_archive(format, include_patterns, exclude_patterns, &artifact_path)
            }
            ArtifactType::Package { format, platform: pkg_platform } => {
                self.generate_package(format, pkg_platform, &artifact_path, build_config)
            }
            ArtifactType::Metadata { format, content } => {
                self.generate_metadata(format, content, &artifact_path)
            }
            ArtifactType::ReleaseNotes { format, template } => {
                self.generate_release_notes(format, template, &artifact_path)
            }
            ArtifactType::License { license_type, format } => {
                self.generate_license(license_type, format, &artifact_path)
            }
            ArtifactType::InstallScript { platform: script_platform, script_type } => {
                self.generate_install_script(script_platform, script_type, &artifact_path)
            }
            ArtifactType::ConfigTemplate { name, format } => {
                self.generate_config_template(name, format, &artifact_path)
            }
            _ => {
                Err(SemanticError::Internal {
                    message: format!("Unsupported artifact type: {:?}", artifact_type),
                })
            }
        }
    }
    
    /// Generate artifact name
    fn generate_artifact_name(&self, artifact_type: &ArtifactType, platform: Option<&PlatformConfig>, build_config: Option<&BuildConfiguration>) -> String {
        let mut template = self.config.naming.template.clone();
        
        // Replace template variables
        template = template.replace("{name}", &self.project.name);
        template = template.replace("{version}", &self.version.current);
        template = template.replace("{date}", &chrono::Utc::now().format(&self.config.naming.date_format).to_string());
        
        if let Some(platform) = platform {
            template = template.replace("{platform}", &platform.name);
            template = template.replace("{os}", &platform.os);
            template = template.replace("{arch}", &platform.arch);
        }
        
        if let Some(build_config) = build_config {
            template = template.replace("{profile}", &build_config.profile);
        }
        
        // Add appropriate file extension
        let extension = match artifact_type {
            ArtifactType::Binary { .. } => {
                if platform.map(|p| p.os.as_str()) == Some("windows") {
                    ".exe"
                } else {
                    ""
                }
            }
            ArtifactType::Library { .. } => {
                match platform.map(|p| p.os.as_str()) {
                    Some("windows") => ".dll",
                    Some("macos") => ".dylib",
                    _ => ".so",
                }
            }
            ArtifactType::Documentation { format, .. } => {
                match format {
                    DocumentationFormat::Html => ".html",
                    DocumentationFormat::Pdf => ".pdf",
                    DocumentationFormat::Markdown => ".md",
                    _ => ".txt",
                }
            }
            ArtifactType::SourceArchive { format, .. } => {
                match format {
                    ArchiveFormat::Zip => ".zip",
                    ArchiveFormat::TarGz => ".tar.gz",
                    ArchiveFormat::TarXz => ".tar.xz",
                    _ => ".tar",
                }
            }
            _ => "",
        };
        
        format!("{}{}", template, extension)
    }
    
    /// Generate binary artifact
    fn generate_binary(&self, name: &str, target: &str, output_path: &PathBuf, platform: Option<&PlatformConfig>, build_config: Option<&BuildConfiguration>) -> Result<Artifact, SemanticError> {
        // In real implementation, would compile the binary
        println!("Compiling binary: {} for target: {}", name, target);
        
        // Create dummy binary file
        let binary_content = format!("Binary for {} ({})", name, target);
        std::fs::write(output_path, binary_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write binary: {}", e),
        })?;
        
        let metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read binary metadata: {}", e),
        })?;
        
        let artifact = Artifact {
            id: format!("binary-{}-{}", name, target),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::Binary {
                name: name.to_string(),
                target: target.to_string(),
            },
            path: output_path.clone(),
            size: metadata.len(),
            platform: platform.map(|p| p.name.clone()),
            build_config: build_config.map(|b| b.name.clone()),
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: "application/octet-stream".to_string(),
                description: format!("Executable binary for {}", name),
                tags: vec!["binary".to_string(), "executable".to_string()],
                build_info: Some(self.create_build_info(build_config)),
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        Ok(artifact)
    }
    
    /// Generate library artifact
    fn generate_library(&self, name: &str, target: &str, version: &str, output_path: &PathBuf, platform: Option<&PlatformConfig>, build_config: Option<&BuildConfiguration>) -> Result<Artifact, SemanticError> {
        println!("Compiling library: {} version {} for target: {}", name, version, target);
        
        // Create dummy library file
        let library_content = format!("Library {} version {} for {}", name, version, target);
        std::fs::write(output_path, library_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write library: {}", e),
        })?;
        
        let metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read library metadata: {}", e),
        })?;
        
        let artifact = Artifact {
            id: format!("library-{}-{}", name, target),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::Library {
                name: name.to_string(),
                target: target.to_string(),
                version: version.to_string(),
            },
            path: output_path.clone(),
            size: metadata.len(),
            platform: platform.map(|p| p.name.clone()),
            build_config: build_config.map(|b| b.name.clone()),
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: "application/octet-stream".to_string(),
                description: format!("Shared library for {}", name),
                tags: vec!["library".to_string(), "shared".to_string()],
                build_info: Some(self.create_build_info(build_config)),
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        Ok(artifact)
    }
    
    /// Generate documentation artifact
    fn generate_documentation(&self, format: &DocumentationFormat, output_name: &str, output_path: &PathBuf) -> Result<Artifact, SemanticError> {
        println!("Generating documentation: {} in {:?} format", output_name, format);
        
        let doc_content = match format {
            DocumentationFormat::Html => {
                format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>{} Documentation</title>
</head>
<body>
    <h1>{} Documentation</h1>
    <p>Version: {}</p>
    <p>Generated on: {}</p>
</body>
</html>
                "#, self.project.name, self.project.name, self.version.current, chrono::Utc::now().format("%Y-%m-%d"))
            }
            DocumentationFormat::Markdown => {
                format!(r#"
# {} Documentation

Version: {}
Generated: {}

## Overview

{} is a programming language designed for optimal generation by Large Language Models.

## Installation

Instructions for installing {}.

## Usage

Basic usage examples and tutorials.
                "#, self.project.name, self.version.current, chrono::Utc::now().format("%Y-%m-%d"), self.project.name, self.project.name)
            }
            _ => {
                format!("{} Documentation\nVersion: {}\nGenerated: {}", 
                       self.project.name, self.version.current, chrono::Utc::now().format("%Y-%m-%d"))
            }
        };
        
        std::fs::write(output_path, doc_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write documentation: {}", e),
        })?;
        
        let metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read documentation metadata: {}", e),
        })?;
        
        let content_type = match format {
            DocumentationFormat::Html => "text/html",
            DocumentationFormat::Pdf => "application/pdf",
            DocumentationFormat::Markdown => "text/markdown",
            _ => "text/plain",
        };
        
        let artifact = Artifact {
            id: format!("docs-{}", output_name),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::Documentation {
                format: format.clone(),
                output_name: output_name.to_string(),
            },
            path: output_path.clone(),
            size: metadata.len(),
            platform: None,
            build_config: None,
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: content_type.to_string(),
                description: format!("Documentation in {:?} format", format),
                tags: vec!["documentation".to_string()],
                build_info: None,
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        Ok(artifact)
    }
    
    /// Generate source archive artifact
    fn generate_source_archive(&self, format: &ArchiveFormat, include_patterns: &[String], exclude_patterns: &[String], output_path: &PathBuf) -> Result<Artifact, SemanticError> {
        println!("Generating source archive in {:?} format", format);
        
        // In real implementation, would create actual archive with source files
        let archive_content = format!("Source archive for {} version {}", self.project.name, self.version.current);
        std::fs::write(output_path, archive_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write source archive: {}", e),
        })?;
        
        let metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read source archive metadata: {}", e),
        })?;
        
        let content_type = match format {
            ArchiveFormat::Zip => "application/zip",
            ArchiveFormat::TarGz => "application/gzip",
            ArchiveFormat::TarXz => "application/x-xz",
            _ => "application/octet-stream",
        };
        
        let artifact = Artifact {
            id: "source-archive".to_string(),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::SourceArchive {
                format: format.clone(),
                include_patterns: include_patterns.to_vec(),
                exclude_patterns: exclude_patterns.to_vec(),
            },
            path: output_path.clone(),
            size: metadata.len(),
            platform: None,
            build_config: None,
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: content_type.to_string(),
                description: "Source code archive".to_string(),
                tags: vec!["source".to_string(), "archive".to_string()],
                build_info: None,
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        Ok(artifact)
    }
    
    /// Generate package artifact
    fn generate_package(&self, format: &PackageFormat, platform: &str, output_path: &PathBuf, build_config: Option<&BuildConfiguration>) -> Result<Artifact, SemanticError> {
        println!("Generating package in {:?} format for platform: {}", format, platform);
        
        // In real implementation, would create actual package
        let package_content = format!("Package for {} {} on {}", self.project.name, self.version.current, platform);
        std::fs::write(output_path, package_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write package: {}", e),
        })?;
        
        let metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read package metadata: {}", e),
        })?;
        
        let content_type = match format {
            PackageFormat::Deb => "application/vnd.debian.binary-package",
            PackageFormat::Rpm => "application/x-rpm",
            PackageFormat::Msi => "application/x-msi",
            _ => "application/octet-stream",
        };
        
        let artifact = Artifact {
            id: format!("package-{}-{}", format_to_string(format), platform),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::Package {
                format: format.clone(),
                platform: platform.to_string(),
            },
            path: output_path.clone(),
            size: metadata.len(),
            platform: Some(platform.to_string()),
            build_config: build_config.map(|b| b.name.clone()),
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: content_type.to_string(),
                description: format!("Installation package in {:?} format", format),
                tags: vec!["package".to_string(), "installer".to_string()],
                build_info: Some(self.create_build_info(build_config)),
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        Ok(artifact)
    }
    
    /// Generate metadata artifact
    fn generate_metadata(&self, format: &MetadataFormat, content: &MetadataContent, output_path: &PathBuf) -> Result<Artifact, SemanticError> {
        println!("Generating metadata in {:?} format", format);
        
        let metadata_obj = match content {
            MetadataContent::ProjectInfo => serde_json::json!({
                "name": self.project.name,
                "version": self.version.current,
                "description": self.project.description,
                "homepage": self.project.homepage,
                "repository": self.project.repository,
                "license": self.project.license,
                "authors": self.project.authors,
                "keywords": self.project.keywords,
                "categories": self.project.categories
            }),
            MetadataContent::BuildInfo => serde_json::json!({
                "build_timestamp": chrono::Utc::now().to_rfc3339(),
                "build_host": std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
                "build_user": std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
                "compiler_version": "rustc 1.75.0",
                "target_triple": "x86_64-unknown-linux-gnu"
            }),
            MetadataContent::ReleaseInfo => serde_json::json!({
                "version": self.version.current,
                "release_date": chrono::Utc::now().to_rfc3339(),
                "release_notes": "Bug fixes and improvements",
                "breaking_changes": [],
                "security_fixes": []
            }),
            _ => serde_json::json!({}),
        };
        
        let metadata_content = match format {
            MetadataFormat::JSON => serde_json::to_string_pretty(&metadata_obj).unwrap(),
            MetadataFormat::YAML => crate::external_stubs::serde_yaml::to_string(&metadata_obj).unwrap_or_else(|_| "YAML not supported".to_string()),
            MetadataFormat::TOML => crate::external_stubs::toml::to_string_pretty(&metadata_obj).unwrap_or_else(|_| "TOML not supported".to_string()),
            _ => serde_json::to_string(&metadata_obj).unwrap(),
        };
        
        std::fs::write(output_path, metadata_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write metadata: {}", e),
        })?;
        
        let file_metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read metadata file metadata: {}", e),
        })?;
        
        let content_type = match format {
            MetadataFormat::JSON => "application/json",
            MetadataFormat::YAML => "application/x-yaml",
            MetadataFormat::TOML => "application/toml",
            MetadataFormat::XML => "application/xml",
            _ => "text/plain",
        };
        
        let artifact = Artifact {
            id: format!("metadata-{:?}", content),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::Metadata {
                format: format.clone(),
                content: content.clone(),
            },
            path: output_path.clone(),
            size: file_metadata.len(),
            platform: None,
            build_config: None,
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: content_type.to_string(),
                description: format!("Metadata in {:?} format", format),
                tags: vec!["metadata".to_string()],
                build_info: None,
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        Ok(artifact)
    }
    
    /// Generate release notes artifact
    fn generate_release_notes(&self, format: &DocumentationFormat, template: &Option<String>, output_path: &PathBuf) -> Result<Artifact, SemanticError> {
        println!("Generating release notes in {:?} format", format);
        
        let release_notes = template.as_ref().map(|t| t.clone()).unwrap_or_else(|| {
            format!(r#"
# Release Notes - {} {}

## What's New

- Bug fixes and performance improvements
- Updated dependencies
- Enhanced error handling

## Breaking Changes

None

## Security Fixes

None

## Known Issues

None

---

Released on: {}
            "#, self.project.name, self.version.current, chrono::Utc::now().format("%Y-%m-%d"))
        });
        
        std::fs::write(output_path, release_notes.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write release notes: {}", e),
        })?;
        
        let metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read release notes metadata: {}", e),
        })?;
        
        let artifact = Artifact {
            id: "release-notes".to_string(),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::ReleaseNotes {
                format: format.clone(),
                template: template.clone(),
            },
            path: output_path.clone(),
            size: metadata.len(),
            platform: None,
            build_config: None,
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: "text/markdown".to_string(),
                description: "Release notes".to_string(),
                tags: vec!["release-notes".to_string(), "documentation".to_string()],
                build_info: None,
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        Ok(artifact)
    }
    
    /// Generate license artifact
fn generate_license(&self, license_type: &str, format: &DocumentationFormat, output_path: &PathBuf) -> Result<Artifact, SemanticError> {
        println!("Generating license file for: {}", license_type);

        let license_content = include_str!("../../../../LICENSE");

        std::fs::write(output_path, license_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write license: {}", e),
        })?;

        let metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read license metadata: {}", e),
        })?;

        let artifact = Artifact {
            id: "license".to_string(),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::License {
                license_type: license_type.to_string(),
                format: format.clone(),
            },
            path: output_path.clone(),
            size: metadata.len(),
            platform: None,
            build_config: None,
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: "text/plain".to_string(),
                description: format!("License file ({})", license_type),
                tags: vec!["license".to_string(), "legal".to_string()],
                build_info: None,
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };

        Ok(artifact)
    }
    
    /// Generate install script artifact
    fn generate_install_script(&self, platform: &str, script_type: &ScriptType, output_path: &PathBuf) -> Result<Artifact, SemanticError> {
        println!("Generating install script for platform: {} ({:?})", platform, script_type);
        
        let script_content = match script_type {
            ScriptType::Shell => {
                format!(r#"#!/bin/bash
# Installation script for {} {}

set -e

echo "Installing {} {}..."

# Check platform
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "This installer is for Linux only"
    exit 1
fi

# Install binary
sudo cp aether /usr/local/bin/
sudo chmod +x /usr/local/bin/aether

echo "Installation complete!"
echo "Run 'aether --version' to verify"
                "#, self.project.name, self.version.current, self.project.name, self.version.current)
            }
            ScriptType::PowerShell => {
                format!(r#"# Installation script for {} {}

Write-Host "Installing {} {}..."

# Check platform
if ($env:OS -ne "Windows_NT") {{
    Write-Error "This installer is for Windows only"
    exit 1
}}

# Install binary
Copy-Item "aether.exe" "C:\Program Files\{}\aether.exe" -Force

Write-Host "Installation complete!"
Write-Host "Run 'aether --version' to verify"
                "#, self.project.name, self.version.current, self.project.name, self.version.current, self.project.name)
            }
            _ => {
                format!("# Installation script for {} {}\n# Platform: {}\n", self.project.name, self.version.current, platform)
            }
        };
        
        std::fs::write(output_path, script_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write install script: {}", e),
        })?;
        
        let metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read install script metadata: {}", e),
        })?;
        
        let artifact = Artifact {
            id: format!("install-script-{}", platform),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::InstallScript {
                platform: platform.to_string(),
                script_type: script_type.clone(),
            },
            path: output_path.clone(),
            size: metadata.len(),
            platform: Some(platform.to_string()),
            build_config: None,
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: "text/plain".to_string(),
                description: format!("Installation script for {}", platform),
                tags: vec!["install".to_string(), "script".to_string()],
                build_info: None,
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        Ok(artifact)
    }
    
    /// Generate config template artifact
    fn generate_config_template(&self, name: &str, format: &ConfigFormat, output_path: &PathBuf) -> Result<Artifact, SemanticError> {
        println!("Generating config template: {} in {:?} format", name, format);
        
        let config_content = match format {
            ConfigFormat::JSON => {
                serde_json::to_string_pretty(&serde_json::json!({
                    "name": self.project.name,
                    "version": self.version.current,
                    "log_level": "info",
                    "debug": false,
                    "features": {
                        "optimization": true,
                        "debugging": false
                    }
                })).unwrap()
            }
            ConfigFormat::YAML => {
                format!(r"# Configuration for {}
name: {}
version: {}
log_level: info
debug: false
features:
  optimization: true
  debugging: false
                ", self.project.name, self.project.name, self.version.current)
            }
            ConfigFormat::TOML => {
                format!(r#"# Configuration for {}
name = "{}"
version = "{}"
log_level = "info"
debug = false

[features]
optimization = true
debugging = false
                "#, self.project.name, self.project.name, self.version.current)
            }
            _ => {
                format!("# Configuration for {}\nname={}\nversion={}\n", self.project.name, self.project.name, self.version.current)
            }
        };
        
        std::fs::write(output_path, config_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write config template: {}", e),
        })?;
        
        let file_metadata = std::fs::metadata(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read config template metadata: {}", e),
        })?;
        
        let content_type = match format {
            ConfigFormat::JSON => "application/json",
            ConfigFormat::YAML => "application/x-yaml",
            ConfigFormat::TOML => "application/toml",
            _ => "text/plain",
        };
        
        let artifact = Artifact {
            id: format!("config-{}", name),
            name: output_path.file_name().unwrap().to_string_lossy().to_string(),
            artifact_type: ArtifactType::ConfigTemplate {
                name: name.to_string(),
                format: format.clone(),
            },
            path: output_path.clone(),
            size: file_metadata.len(),
            platform: None,
            build_config: None,
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: content_type.to_string(),
                description: format!("Configuration template for {}", name),
                tags: vec!["config".to_string(), "template".to_string()],
                build_info: None,
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        Ok(artifact)
    }
    
    /// Create build information
    fn create_build_info(&self, build_config: Option<&BuildConfiguration>) -> BuildInfo {
        BuildInfo {
            timestamp: chrono::Utc::now().to_rfc3339(),
            host: std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string()),
            user: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            compiler_version: "rustc 1.75.0".to_string(),
            build_flags: build_config.map(|b| b.compiler_flags.clone()).unwrap_or_default(),
            git_info: Some(GitInfo {
                commit: "abcd1234567890".to_string(),
                branch: "main".to_string(),
                tag: Some(format!("v{}", self.version.current)),
                repository: self.project.repository.clone(),
                commit_timestamp: chrono::Utc::now().to_rfc3339(),
                author: "Developer".to_string(),
                message: "Release commit".to_string(),
            }),
            env_vars: std::env::vars().collect(),
        }
    }
    
    /// Generate checksums for all artifacts
    fn generate_checksums(&mut self) -> Result<(), SemanticError> {
        println!("Generating checksums for artifacts");
        
        // Collect artifact ids and paths to avoid borrow conflicts
        let artifact_data: Vec<(String, PathBuf)> = self.artifacts.iter()
            .map(|(id, artifact)| (id.clone(), artifact.path.clone()))
            .collect();
        
        // Calculate checksums and store them temporarily
        let mut checksums_to_update: Vec<(String, ChecksumAlgorithm, String)> = Vec::new();
        
        for (artifact_id, path) in artifact_data {
            for algorithm in &[ChecksumAlgorithm::SHA256, ChecksumAlgorithm::SHA512] {
                let checksum = self.calculate_checksum(&path, algorithm)?;
                checksums_to_update.push((artifact_id.clone(), algorithm.clone(), checksum));
            }
        }
        
        // Update artifacts with calculated checksums
        for (artifact_id, algorithm, checksum) in checksums_to_update {
            if let Some(artifact) = self.artifacts.get_mut(&artifact_id) {
                artifact.checksums.insert(algorithm, checksum);
            }
        }
        
        // Generate checksum files
        for algorithm in &[ChecksumAlgorithm::SHA256, ChecksumAlgorithm::SHA512] {
            self.generate_checksum_file(algorithm)?;
        }
        
        Ok(())
    }
    
    /// Calculate checksum for a file
    fn calculate_checksum(&self, file_path: &PathBuf, algorithm: &ChecksumAlgorithm) -> Result<String, SemanticError> {
        use std::io::Read;
        
        let mut file = std::fs::File::open(file_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to open file for checksum: {}", e),
        })?;
        
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read file for checksum: {}", e),
        })?;
        
        match algorithm {
            ChecksumAlgorithm::SHA256 => {
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                Ok(format!("{:x}", hasher.finalize()))
            }
            ChecksumAlgorithm::SHA512 => {
                use sha2::{Sha512, Digest};
                let mut hasher = Sha512::new();
                hasher.update(&buffer);
                Ok(format!("{:x}", hasher.finalize()))
            }
            _ => {
                // Simple hash for other algorithms
                Ok(format!("{:08x}", buffer.len()))
            }
        }
    }
    
    /// Generate checksum file
    fn generate_checksum_file(&mut self, algorithm: &ChecksumAlgorithm) -> Result<(), SemanticError> {
        let filename = format!("checksums.{}", algorithm_to_string(algorithm).to_lowercase());
        let checksum_path = self.config.output_dir.join(&filename);
        
        let mut checksum_content = String::new();
        
        for artifact in self.artifacts.values() {
            if let Some(checksum) = artifact.checksums.get(algorithm) {
                checksum_content.push_str(&format!("{}  {}\n", checksum, artifact.name));
            }
        }
        
        std::fs::write(&checksum_path, checksum_content.as_bytes()).map_err(|e| SemanticError::Internal {
            message: format!("Failed to write checksum file: {}", e),
        })?;
        
        // Add checksum file as artifact
        let metadata = std::fs::metadata(&checksum_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read checksum file metadata: {}", e),
        })?;
        
        let artifact = Artifact {
            id: format!("checksums-{}", algorithm_to_string(algorithm).to_lowercase()),
            name: filename,
            artifact_type: ArtifactType::Checksum {
                algorithms: vec![algorithm.clone()],
                format: ChecksumFormat::GNU,
            },
            path: checksum_path,
            size: metadata.len(),
            platform: None,
            build_config: None,
            checksums: HashMap::new(),
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: ArtifactMetadata {
                content_type: "text/plain".to_string(),
                description: format!("Checksum file using {:?}", algorithm),
                tags: vec!["checksum".to_string(), "verification".to_string()],
                build_info: None,
                license: Some(self.project.license.clone()),
                properties: HashMap::new(),
            },
            dependencies: vec![],
            validation: None,
        };
        
        self.artifacts.insert(artifact.id.clone(), artifact);
        
        Ok(())
    }
    
    /// Sign artifacts
    fn sign_artifacts(&mut self) -> Result<(), SemanticError> {
        println!("Signing artifacts");
        
        if let Some(_signing_config) = &self.signing {
            for artifact in self.artifacts.values_mut() {
                // In real implementation, would use cryptographic signing
                let signature = format!("signature_for_{}", artifact.id);
                artifact.signature = Some(signature);
            }
        }
        
        Ok(())
    }
    
    /// Validate artifacts
    fn validate_artifacts(&mut self) -> Result<(), SemanticError> {
        if !self.config.validation.enabled {
            return Ok(());
        }
        
        println!("Validating artifacts");
        
        let artifact_keys: Vec<String> = self.artifacts.keys().cloned().collect();
        
        for key in artifact_keys {
            let mut validation_messages = Vec::new();
            let start_time = std::time::Instant::now();
            
            // Run validation rules
            let artifact = self.artifacts.get(&key).unwrap();
            for rule in &self.validation_rules {
                if self.rule_applies_to_artifact(&rule.config, artifact) {
                    match rule.validator.validate(artifact) {
                        Ok(result) => {
                            validation_messages.extend(result.messages);
                        }
                        Err(e) => {
                            validation_messages.push(ValidationMessage {
                                severity: ValidationSeverity::Error,
                                message: e.to_string(),
                                rule: rule.config.name.clone(),
                                context: HashMap::new(),
                            });
                        }
                    }
                }
            }
            
            let duration = start_time.elapsed();
            let status = if validation_messages.iter().any(|m| matches!(m.severity, ValidationSeverity::Error | ValidationSeverity::Critical)) {
                ValidationStatus::Failed
            } else if validation_messages.iter().any(|m| matches!(m.severity, ValidationSeverity::Warning)) {
                ValidationStatus::Warning
            } else {
                ValidationStatus::Passed
            };
            
            let validation_result = ValidationResult {
                status,
                messages: validation_messages,
                duration,
                timestamp: std::time::SystemTime::now(),
            };
            
            if let Some(artifact) = self.artifacts.get_mut(&key) {
                artifact.validation = Some(validation_result);
            }
        }
        
        Ok(())
    }
    
    /// Check if validation rule applies to artifact
    fn rule_applies_to_artifact(&self, rule_config: &ValidationRuleConfig, artifact: &Artifact) -> bool {
        if rule_config.apply_to.is_empty() {
            return true; // Apply to all artifacts
        }
        
        rule_config.apply_to.iter().any(|pattern| {
            artifact.name.contains(pattern) || artifact.id.contains(pattern)
        })
    }
    
    /// Create artifact sets
    fn create_artifact_sets(&mut self) -> Result<(), SemanticError> {
        println!("Creating artifact sets");
        
        // Create platform-specific sets
        let mut platform_sets: HashMap<String, Vec<String>> = HashMap::new();
        
        for artifact in self.artifacts.values() {
            if let Some(platform) = &artifact.platform {
                platform_sets.entry(platform.clone())
                    .or_default()
                    .push(artifact.id.clone());
            }
        }
        
        for (platform, artifacts) in platform_sets {
            let set = ArtifactSet {
                name: format!("{}-{}", platform, self.version.current),
                description: format!("Artifacts for {} platform", platform),
                artifacts,
                metadata: HashMap::new(),
                stage: ReleaseStage::Stable,
                audience: vec!["end-users".to_string()],
            };
            
            self.artifact_sets.push(set);
        }
        
        // Create documentation set
        let doc_artifacts: Vec<String> = self.artifacts.values()
            .filter(|a| matches!(a.artifact_type, ArtifactType::Documentation { .. } | ArtifactType::ReleaseNotes { .. } | ArtifactType::License { .. }))
            .map(|a| a.id.clone())
            .collect();
        
        if !doc_artifacts.is_empty() {
            let doc_set = ArtifactSet {
                name: format!("documentation-{}", self.version.current),
                description: "Documentation and release information".to_string(),
                artifacts: doc_artifacts,
                metadata: HashMap::new(),
                stage: ReleaseStage::Stable,
                audience: vec!["developers".to_string(), "end-users".to_string()],
            };
            
            self.artifact_sets.push(doc_set);
        }
        
        Ok(())
    }
    
    /// Get artifact by ID
    pub fn get_artifact(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.get(id)
    }
    
    /// Get artifacts by type
    pub fn get_artifacts_by_type(&self, artifact_type: &ArtifactType) -> Vec<&Artifact> {
        self.artifacts.values()
            .filter(|a| std::mem::discriminant(&a.artifact_type) == std::mem::discriminant(artifact_type))
            .collect()
    }
    
    /// Get artifacts by platform
    pub fn get_artifacts_by_platform(&self, platform: &str) -> Vec<&Artifact> {
        self.artifacts.values()
            .filter(|a| a.platform.as_ref() == Some(&platform.to_string()))
            .collect()
    }
    
    /// Get artifact sets
    pub fn get_artifact_sets(&self) -> &[ArtifactSet] {
        &self.artifact_sets
    }
    
    /// Generate artifact manifest
    pub fn generate_manifest(&self) -> Result<String, SemanticError> {
        let manifest = serde_json::json!({
            "project": {
                "name": self.project.name,
                "version": self.version.current,
                "description": self.project.description
            },
            "artifacts": self.artifacts.values().map(|a| serde_json::json!({
                "id": a.id,
                "name": a.name,
                "type": format!("{:?}", a.artifact_type),
                "path": a.path.display().to_string(),
                "size": a.size,
                "platform": a.platform,
                "build_config": a.build_config,
                "checksums": a.checksums,
            })).collect::<Vec<_>>(),
        });
        serde_json::to_string_pretty(&manifest).map_err(|e| SemanticError::Internal {
            message: format!("Failed to serialize manifest: {}", e),
        })
    }
}

fn format_to_string(format: &PackageFormat) -> String {
    match format {
        PackageFormat::Deb => "deb".to_string(),
        PackageFormat::Rpm => "rpm".to_string(),
        PackageFormat::Msi => "msi".to_string(),
        PackageFormat::Pkg => "pkg".to_string(),
        PackageFormat::Dmg => "dmg".to_string(),
        PackageFormat::AppImage => "appimage".to_string(),
        PackageFormat::Snap => "snap".to_string(),
        PackageFormat::Flatpak => "flatpak".to_string(),
    }
}

fn algorithm_to_string(algorithm: &ChecksumAlgorithm) -> String {
    match algorithm {
        ChecksumAlgorithm::MD5 => "md5".to_string(),
        ChecksumAlgorithm::SHA1 => "sha1".to_string(),
        ChecksumAlgorithm::SHA256 => "sha256".to_string(),
        ChecksumAlgorithm::SHA512 => "sha512".to_string(),
        ChecksumAlgorithm::Blake2b => "blake2b".to_string(),
        ChecksumAlgorithm::Blake3 => "blake3".to_string(),
    }
}
