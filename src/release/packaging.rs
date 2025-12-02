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

//! Package creation and management for AetherScript releases
//!
//! Handles creation of distribution packages in various formats including
//! archives, installers, containers, and platform-specific packages.

#![allow(dead_code)]

use crate::error::SemanticError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Package builder for creating distribution packages
#[derive(Debug)]
pub struct PackageBuilder {
    /// Builder configuration
    config: PackageBuilderConfig,

    /// Build artifacts
    artifacts: HashMap<String, ArtifactInfo>,

    /// Package metadata
    metadata: PackageMetadata,

    /// Supported package formats
    formats: Vec<PackageFormat>,

    /// Compression settings
    compression: CompressionSettings,
}

/// Compression settings
#[derive(Debug, Clone)]
pub struct CompressionSettings {
    /// Compression level (0-9)
    pub level: u32,

    /// Compression algorithm
    pub algorithm: CompressionAlgorithm,
}

impl Default for CompressionSettings {
    fn default() -> Self {
        Self {
            level: 6,
            algorithm: CompressionAlgorithm::Gzip,
        }
    }
}

/// Compression algorithm
#[derive(Debug, Clone)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Bzip2,
    Xz,
    Zstd,
    Lz4,
    Brotli,
}

/// Package builder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageBuilderConfig {
    /// Build directory
    pub build_dir: PathBuf,

    /// Output directory
    pub output_dir: PathBuf,

    /// Temporary directory
    pub temp_dir: PathBuf,

    /// Signing configuration
    pub signing: Option<SigningConfig>,

    /// Verification settings
    pub verification: VerificationConfig,

    /// Package formats to create
    pub formats: Vec<PackageFormatConfig>,

    /// Archive settings
    pub archive: ArchiveConfig,

    /// Installer settings
    pub installer: InstallerConfig,
}

/// Package formats
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PackageFormat {
    /// ZIP archive
    Zip,

    /// TAR.GZ archive
    TarGz,

    /// TAR.XZ archive
    TarXz,

    /// DEB package (Debian/Ubuntu)
    Deb,

    /// RPM package (Red Hat/SUSE)
    Rpm,

    /// MSI installer (Windows)
    Msi,

    /// PKG installer (macOS)
    Pkg,

    /// DMG disk image (macOS)
    Dmg,

    /// AppImage (Linux)
    AppImage,

    /// Snap package (Linux)
    Snap,

    /// Flatpak (Linux)
    Flatpak,

    /// Docker image
    Docker,

    /// OCI container
    Oci,

    /// Homebrew formula
    Homebrew,

    /// Chocolatey package
    Chocolatey,

    /// NPM package
    Npm,

    /// PyPI package
    PyPi,

    /// Custom format
    Custom(String),
}

/// Package format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageFormatConfig {
    /// Format type
    pub format: PackageFormat,

    /// Platform targets
    pub platforms: Vec<PlatformTarget>,

    /// Format-specific options
    pub options: HashMap<String, String>,

    /// Include patterns
    pub includes: Vec<String>,

    /// Exclude patterns
    pub excludes: Vec<String>,

    /// Post-processing steps
    pub post_process: Vec<String>,
}

/// Platform target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformTarget {
    /// Operating system
    pub os: String,

    /// Architecture
    pub arch: String,

    /// ABI/variant
    pub variant: Option<String>,

    /// Minimum OS version
    pub min_version: Option<String>,

    /// Platform-specific options
    pub options: HashMap<String, String>,
}

/// Artifact information
#[derive(Debug, Clone)]
pub struct ArtifactInfo {
    /// Artifact path
    pub path: PathBuf,

    /// Artifact type
    pub artifact_type: ArtifactType,

    /// Target platform
    pub platform: Option<PlatformTarget>,

    /// File size
    pub size: u64,

    /// Checksum
    pub checksum: String,

    /// Signature
    pub signature: Option<String>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Artifact types
#[derive(Debug, Clone)]
pub enum ArtifactType {
    /// Executable binary
    Executable,

    /// Shared library
    Library,

    /// Static library
    StaticLibrary,

    /// Documentation
    Documentation,

    /// Configuration file
    Configuration,

    /// Resource file
    Resource,

    /// Script
    Script,

    /// Archive
    Archive,

    /// Installer
    Installer,

    /// Container image
    Container,

    /// Metadata file
    Metadata,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,

    /// Package version
    pub version: String,

    /// Package description
    pub description: String,

    /// Package maintainer
    pub maintainer: String,

    /// Package homepage
    pub homepage: String,

    /// Package license
    pub license: String,

    /// Package dependencies
    pub dependencies: Vec<PackageDependency>,

    /// Package categories
    pub categories: Vec<String>,

    /// Package keywords
    pub keywords: Vec<String>,

    /// Installation size
    pub installed_size: Option<u64>,

    /// Download size
    pub download_size: Option<u64>,

    /// Package priority
    pub priority: PackagePriority,

    /// Custom fields
    pub custom_fields: HashMap<String, String>,
}

/// Package dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    /// Dependency name
    pub name: String,

    /// Version requirement
    pub version: Option<String>,

    /// Dependency type
    pub dependency_type: DependencyType,

    /// Platform constraints
    pub platforms: Vec<PlatformTarget>,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Required dependency
    Required,

    /// Optional dependency
    Optional,

    /// Recommended dependency
    Recommended,

    /// Suggested dependency
    Suggested,

    /// Build dependency
    Build,

    /// Test dependency
    Test,

    /// Runtime dependency
    Runtime,
}

/// Package priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackagePriority {
    Required,
    Important,
    Standard,
    Optional,
    Extra,
}

/// Signing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    /// Signing key path
    pub key_path: PathBuf,

    /// Key passphrase
    pub passphrase: Option<String>,

    /// Signing algorithm
    pub algorithm: SigningAlgorithm,

    /// Certificate chain
    pub cert_chain: Option<PathBuf>,

    /// Timestamp server
    pub timestamp_server: Option<String>,

    /// Sign all artifacts
    pub sign_all: bool,
}

/// Signing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SigningAlgorithm {
    Rsa2048,
    Rsa4096,
    EcdsaP256,
    EcdsaP384,
    Ed25519,
}

/// Verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Generate checksums
    pub checksums: bool,

    /// Checksum algorithms
    pub checksum_algorithms: Vec<ChecksumAlgorithm>,

    /// Verify signatures
    pub verify_signatures: bool,

    /// Trusted certificates
    pub trusted_certs: Vec<PathBuf>,

    /// Verification strict mode
    pub strict_mode: bool,
}

/// Checksum algorithms
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChecksumAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Blake2b,
    Blake3,
}

/// Archive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    /// Compression level (0-9)
    pub compression_level: u8,

    /// Include hidden files
    pub include_hidden: bool,

    /// Preserve permissions
    pub preserve_permissions: bool,

    /// Preserve timestamps
    pub preserve_timestamps: bool,

    /// Archive format options
    pub format_options: HashMap<String, String>,
}

/// Installer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerConfig {
    /// Installer type
    pub installer_type: InstallerType,

    /// Installation directory
    pub install_dir: String,

    /// Create desktop shortcuts
    pub desktop_shortcuts: bool,

    /// Create start menu entries
    pub start_menu: bool,

    /// Add to PATH
    pub add_to_path: bool,

    /// License agreement
    pub license_agreement: Option<PathBuf>,

    /// Custom install scripts
    pub install_scripts: Vec<PathBuf>,

    /// Uninstall scripts
    pub uninstall_scripts: Vec<PathBuf>,

    /// Installer branding
    pub branding: InstallerBranding,
}

/// Installer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallerType {
    /// NSIS installer (Windows)
    Nsis,

    /// WiX installer (Windows)
    Wix,

    /// InstallShield (Windows)
    InstallShield,

    /// PackageMaker (macOS)
    PackageMaker,

    /// productbuild (macOS)
    ProductBuild,

    /// makeself (Linux)
    MakeSelf,

    /// Generic shell script
    ShellScript,
}

/// Installer branding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerBranding {
    /// Company name
    pub company: String,

    /// Product name
    pub product: String,

    /// Logo image
    pub logo: Option<PathBuf>,

    /// Banner image
    pub banner: Option<PathBuf>,

    /// Icon file
    pub icon: Option<PathBuf>,

    /// Color scheme
    pub colors: HashMap<String, String>,
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Default compression algorithm
    pub default_algorithm: CompressionAlgorithm,

    /// Compression level
    pub level: u8,

    /// Algorithm-specific options
    pub options: HashMap<CompressionAlgorithm, HashMap<String, String>>,
}

impl PackageBuilder {
    /// Create a new package builder
    pub fn new(config: PackageBuilderConfig) -> Result<Self, SemanticError> {
        // Create directories if they don't exist
        std::fs::create_dir_all(&config.build_dir).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create build directory: {}", e),
        })?;

        std::fs::create_dir_all(&config.output_dir).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create output directory: {}", e),
        })?;

        std::fs::create_dir_all(&config.temp_dir).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create temp directory: {}", e),
        })?;

        let formats = config.formats.iter().map(|f| f.format.clone()).collect();

        let metadata = PackageMetadata {
            name: "aetherscript".to_string(),
            version: "1.0.0".to_string(),
            description: "AetherScript Programming Language".to_string(),
            maintainer: "AetherScript Team <team@aetherscript.dev>".to_string(),
            homepage: "https://aetherscript.dev".to_string(),
            license: "MIT".to_string(),
            dependencies: vec![],
            categories: vec!["development".to_string(), "programming".to_string()],
            keywords: vec![
                "compiler".to_string(),
                "language".to_string(),
                "llm".to_string(),
            ],
            installed_size: None,
            download_size: None,
            priority: PackagePriority::Standard,
            custom_fields: HashMap::new(),
        };

        Ok(Self {
            config,
            formats,
            artifacts: HashMap::new(),
            metadata,
            compression: CompressionSettings::default(),
        })
    }

    /// Add an artifact to be packaged
    pub fn add_artifact(
        &mut self,
        path: PathBuf,
        artifact_type: ArtifactType,
        platform: Option<PlatformTarget>,
    ) -> Result<(), SemanticError> {
        if !path.exists() {
            return Err(SemanticError::Internal {
                message: format!("Artifact not found: {}", path.display()),
            });
        }

        let metadata = std::fs::metadata(&path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to read artifact metadata: {}", e),
        })?;

        let checksum = self.calculate_checksum(&path, ChecksumAlgorithm::Sha256)?;

        let artifact = ArtifactInfo {
            path: path.clone(),
            artifact_type,
            platform,
            size: metadata.len(),
            checksum,
            signature: None,
            metadata: HashMap::new(),
        };

        let key = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        self.artifacts.insert(key, artifact);
        Ok(())
    }

    /// Build all packages
    pub fn build_packages(&mut self) -> Result<Vec<PackageInfo>, SemanticError> {
        let mut packages = Vec::new();

        // Sign artifacts if configured
        if self.config.signing.is_some() {
            self.sign_artifacts()?;
        }

        // Build packages for each format
        for format_config in &self.config.formats.clone() {
            for platform in &format_config.platforms {
                let package = self.build_package(&format_config.format, platform)?;
                packages.push(package);
            }
        }

        // Verify packages
        if self.config.verification.verify_signatures {
            self.verify_packages(&packages)?;
        }

        Ok(packages)
    }

    /// Build a package for a specific format and platform
    fn build_package(
        &self,
        format: &PackageFormat,
        platform: &PlatformTarget,
    ) -> Result<PackageInfo, SemanticError> {
        println!(
            "Building {} package for {}-{}",
            format.to_string(),
            platform.os,
            platform.arch
        );

        let package_name = self.generate_package_name(format, platform);
        let package_path = self.config.output_dir.join(&package_name);

        match format {
            PackageFormat::Zip => self.build_zip_package(&package_path, platform)?,
            PackageFormat::TarGz => {
                self.build_tar_package(&package_path, platform, CompressionAlgorithm::Gzip)?
            }
            PackageFormat::TarXz => {
                self.build_tar_package(&package_path, platform, CompressionAlgorithm::Xz)?
            }
            PackageFormat::Deb => self.build_deb_package(&package_path, platform)?,
            PackageFormat::Rpm => self.build_rpm_package(&package_path, platform)?,
            PackageFormat::Msi => self.build_msi_package(&package_path, platform)?,
            PackageFormat::Pkg => self.build_pkg_package(&package_path, platform)?,
            PackageFormat::Dmg => self.build_dmg_package(&package_path, platform)?,
            PackageFormat::AppImage => self.build_appimage_package(&package_path, platform)?,
            PackageFormat::Docker => self.build_docker_package(&package_path, platform)?,
            PackageFormat::Homebrew => self.build_homebrew_package(&package_path, platform)?,
            _ => {
                return Err(SemanticError::Internal {
                    message: format!("Unsupported package format: {:?}", format),
                })
            }
        }

        let package_metadata =
            std::fs::metadata(&package_path).map_err(|e| SemanticError::Internal {
                message: format!("Failed to read package metadata: {}", e),
            })?;

        let checksum = self.calculate_checksum(&package_path, ChecksumAlgorithm::Sha256)?;

        Ok(PackageInfo {
            name: package_name,
            path: package_path,
            format: format.clone(),
            platform: platform.clone(),
            size: package_metadata.len(),
            checksum,
            signature: None,
            created_at: std::time::SystemTime::now(),
            metadata: self.metadata.clone(),
        })
    }

    /// Build ZIP package
    fn build_zip_package(
        &self,
        output_path: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        use std::io::Write;

        let file = std::fs::File::create(output_path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create ZIP file: {}", e),
        })?;

        let mut zip = zip::ZipWriter::new(file);

        // Add artifacts to ZIP
        for (name, artifact) in &self.artifacts {
            if self.should_include_artifact(artifact, platform) {
                let file_contents =
                    std::fs::read(&artifact.path).map_err(|e| SemanticError::Internal {
                        message: format!("Failed to read artifact: {}", e),
                    })?;

                let options = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated)
                    .unix_permissions(0o755);

                zip.start_file(name, options)
                    .map_err(|e| SemanticError::Internal {
                        message: format!("Failed to start ZIP file entry: {}", e),
                    })?;

                zip.write_all(&file_contents)
                    .map_err(|e| SemanticError::Internal {
                        message: format!("Failed to write ZIP file entry: {}", e),
                    })?;
            }
        }

        zip.finish().map_err(|e| SemanticError::Internal {
            message: format!("Failed to finish ZIP file: {}", e),
        })?;

        Ok(())
    }

    /// Build TAR package with compression
    fn build_tar_package(
        &self,
        output_path: &PathBuf,
        _platform: &PlatformTarget,
        compression: CompressionAlgorithm,
    ) -> Result<(), SemanticError> {
        // This would use the tar crate in a real implementation
        println!("Building TAR package with {:?} compression", compression);

        // Create a dummy tar file for this example
        std::fs::write(output_path, b"TAR package content").map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to create TAR file: {}", e),
            }
        })?;

        Ok(())
    }

    /// Build DEB package
    fn build_deb_package(
        &self,
        output_path: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        // This would use dpkg-deb or similar tools in a real implementation
        println!("Building DEB package for {}-{}", platform.os, platform.arch);

        // Create package structure
        let temp_dir = self.config.temp_dir.join("deb_build");
        std::fs::create_dir_all(&temp_dir).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create DEB build directory: {}", e),
        })?;

        // Create DEBIAN directory
        let debian_dir = temp_dir.join("DEBIAN");
        std::fs::create_dir_all(&debian_dir).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create DEBIAN directory: {}", e),
        })?;

        // Create control file
        let control_content = self.generate_deb_control(platform)?;
        std::fs::write(debian_dir.join("control"), control_content).map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to write control file: {}", e),
            }
        })?;

        // Copy artifacts
        self.copy_artifacts_to_deb_structure(&temp_dir, platform)?;

        // Create a dummy DEB file
        std::fs::write(output_path, b"DEB package content").map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to create DEB file: {}", e),
            }
        })?;

        Ok(())
    }

    /// Build RPM package
    fn build_rpm_package(
        &self,
        output_path: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        println!("Building RPM package for {}-{}", platform.os, platform.arch);

        // Create a dummy RPM file
        std::fs::write(output_path, b"RPM package content").map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to create RPM file: {}", e),
            }
        })?;

        Ok(())
    }

    /// Build MSI package  
    fn build_msi_package(
        &self,
        output_path: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        println!("Building MSI package for {}-{}", platform.os, platform.arch);

        // Create a dummy MSI file
        std::fs::write(output_path, b"MSI package content").map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to create MSI file: {}", e),
            }
        })?;

        Ok(())
    }

    /// Build PKG package (macOS)
    fn build_pkg_package(
        &self,
        output_path: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        println!("Building PKG package for {}-{}", platform.os, platform.arch);

        // Create a dummy PKG file
        std::fs::write(output_path, b"PKG package content").map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to create PKG file: {}", e),
            }
        })?;

        Ok(())
    }

    /// Build DMG package (macOS)
    fn build_dmg_package(
        &self,
        output_path: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        println!("Building DMG package for {}-{}", platform.os, platform.arch);

        // Create a dummy DMG file
        std::fs::write(output_path, b"DMG package content").map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to create DMG file: {}", e),
            }
        })?;

        Ok(())
    }

    /// Build AppImage package
    fn build_appimage_package(
        &self,
        output_path: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        println!(
            "Building AppImage package for {}-{}",
            platform.os, platform.arch
        );

        // Create a dummy AppImage file
        std::fs::write(output_path, b"AppImage package content").map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to create AppImage file: {}", e),
            }
        })?;

        Ok(())
    }

    /// Build Docker package
    fn build_docker_package(
        &self,
        output_path: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        println!(
            "Building Docker image for {}-{}",
            platform.os, platform.arch
        );

        // Generate Dockerfile
        let dockerfile_content = self.generate_dockerfile(platform)?;
        let dockerfile_path = self.config.temp_dir.join("Dockerfile");
        std::fs::write(&dockerfile_path, dockerfile_content).map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to write Dockerfile: {}", e),
            }
        })?;

        // Create a dummy Docker image file
        std::fs::write(output_path, b"Docker image content").map_err(|e| {
            SemanticError::Internal {
                message: format!("Failed to create Docker image file: {}", e),
            }
        })?;

        Ok(())
    }

    /// Build Homebrew package
    fn build_homebrew_package(
        &self,
        output_path: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        println!(
            "Building Homebrew formula for {}-{}",
            platform.os, platform.arch
        );

        let formula_content = self.generate_homebrew_formula(platform)?;
        std::fs::write(output_path, formula_content).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create Homebrew formula: {}", e),
        })?;

        Ok(())
    }

    /// Generate package name
    fn generate_package_name(&self, format: &PackageFormat, platform: &PlatformTarget) -> String {
        let extension = match format {
            PackageFormat::Zip => "zip",
            PackageFormat::TarGz => "tar.gz",
            PackageFormat::TarXz => "tar.xz",
            PackageFormat::Deb => "deb",
            PackageFormat::Rpm => "rpm",
            PackageFormat::Msi => "msi",
            PackageFormat::Pkg => "pkg",
            PackageFormat::Dmg => "dmg",
            PackageFormat::AppImage => "AppImage",
            PackageFormat::Docker => "tar",
            PackageFormat::Homebrew => "rb",
            _ => "pkg",
        };

        format!(
            "{}-{}-{}-{}.{}",
            self.metadata.name, self.metadata.version, platform.os, platform.arch, extension
        )
    }

    /// Check if artifact should be included for platform
    fn should_include_artifact(&self, artifact: &ArtifactInfo, platform: &PlatformTarget) -> bool {
        match &artifact.platform {
            Some(artifact_platform) => {
                artifact_platform.os == platform.os && artifact_platform.arch == platform.arch
            }
            None => true, // Platform-agnostic artifacts
        }
    }

    /// Generate DEB control file content
    fn generate_deb_control(&self, platform: &PlatformTarget) -> Result<String, SemanticError> {
        let installed_size = self
            .artifacts
            .values()
            .filter(|a| self.should_include_artifact(a, platform))
            .map(|a| a.size)
            .sum::<u64>()
            / 1024; // Convert to KB

        Ok(format!(
            r#"Package: {}
Version: {}
Architecture: {}
Maintainer: {}
Installed-Size: {}
Depends: libc6
Section: devel
Priority: {}
Homepage: {}
Description: {}
 AetherScript is a modern programming language designed for optimal
 generation by Large Language Models (LLMs).
"#,
            self.metadata.name,
            self.metadata.version,
            if platform.arch == "x86_64" {
                "amd64"
            } else {
                &platform.arch
            },
            self.metadata.maintainer,
            installed_size,
            match self.metadata.priority {
                PackagePriority::Required => "required",
                PackagePriority::Important => "important",
                PackagePriority::Standard => "standard",
                PackagePriority::Optional => "optional",
                PackagePriority::Extra => "extra",
            },
            self.metadata.homepage,
            self.metadata.description
        ))
    }

    /// Copy artifacts to DEB package structure
    fn copy_artifacts_to_deb_structure(
        &self,
        temp_dir: &PathBuf,
        platform: &PlatformTarget,
    ) -> Result<(), SemanticError> {
        let usr_bin = temp_dir.join("usr/bin");
        std::fs::create_dir_all(&usr_bin).map_err(|e| SemanticError::Internal {
            message: format!("Failed to create usr/bin directory: {}", e),
        })?;

        for artifact in self.artifacts.values() {
            if self.should_include_artifact(artifact, platform)
                && matches!(artifact.artifact_type, ArtifactType::Executable)
            {
                let dest = usr_bin.join(artifact.path.file_name().unwrap());
                std::fs::copy(&artifact.path, dest).map_err(|e| SemanticError::Internal {
                    message: format!("Failed to copy artifact: {}", e),
                })?;
            }
        }

        Ok(())
    }

    /// Generate Dockerfile content
    fn generate_dockerfile(&self, platform: &PlatformTarget) -> Result<String, SemanticError> {
        let base_image = match platform.os.as_str() {
            "linux" => match platform.arch.as_str() {
                "x86_64" => "ubuntu:22.04",
                "aarch64" => "arm64v8/ubuntu:22.04",
                _ => "ubuntu:22.04",
            },
            _ => "ubuntu:22.04",
        };

        Ok(format!(
            r#"FROM {}

# Install dependencies
RUN apt-get update && apt-get install -y \
    libc6 \
    && rm -rf /var/lib/apt/lists/*

# Create user
RUN useradd -m -s /bin/bash aether

# Copy binaries
COPY --chown=aether:aether . /opt/aetherscript/

# Set permissions
RUN chmod +x /opt/aetherscript/bin/*

# Add to PATH
ENV PATH="/opt/aetherscript/bin:${{PATH}}"

# Switch to user
USER aether
WORKDIR /home/aether

# Default command
CMD ["aether", "--help"]
"#,
            base_image
        ))
    }

    /// Generate Homebrew formula
    fn generate_homebrew_formula(
        &self,
        _platform: &PlatformTarget,
    ) -> Result<String, SemanticError> {
        let formula = format!("class Aetherscript < Formula
  desc \"{}\"
  homepage \"{}\"
  url \"https://github.com/aetherscript/aetherscript/archive/v{}.tar.gz\"
  sha256 \"0000000000000000000000000000000000000000000000000000000000000000\"  # Update with actual SHA256
  license \"{}\"

  depends_on \"llvm\"

  def install
    bin.install \"aether\"
    lib.install Dir[\"lib/*\"]
    include.install Dir[\"include/*\"]
  end

  test do
    system \"#{{bin}}/aether\", \"--version\"
  end
end
",
            self.metadata.description,
            self.metadata.homepage,
            self.metadata.version,
            self.metadata.license);
        Ok(formula)
    }

    /// Calculate checksum for a file
    fn calculate_checksum(
        &self,
        path: &PathBuf,
        algorithm: ChecksumAlgorithm,
    ) -> Result<String, SemanticError> {
        use std::io::Read;

        let mut file = std::fs::File::open(path).map_err(|e| SemanticError::Internal {
            message: format!("Failed to open file for checksum: {}", e),
        })?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| SemanticError::Internal {
                message: format!("Failed to read file for checksum: {}", e),
            })?;

        match algorithm {
            ChecksumAlgorithm::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                Ok(format!("{:x}", hasher.finalize()))
            }
            ChecksumAlgorithm::Sha512 => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(&buffer);
                Ok(format!("{:x}", hasher.finalize()))
            }
            _ => {
                // Simple hash for other algorithms in this example
                Ok(format!("{:08x}", buffer.len()))
            }
        }
    }

    /// Sign artifacts
    fn sign_artifacts(&mut self) -> Result<(), SemanticError> {
        if let Some(signing_config) = &self.config.signing {
            println!("Signing artifacts with {:?}", signing_config.algorithm);

            for artifact in self.artifacts.values_mut() {
                // In a real implementation, would use actual signing libraries
                let signature = format!("signature_for_{}", artifact.checksum);
                artifact.signature = Some(signature);
            }
        }

        Ok(())
    }

    /// Verify packages
    fn verify_packages(&self, packages: &[PackageInfo]) -> Result<(), SemanticError> {
        println!("Verifying {} packages", packages.len());

        for package in packages {
            // Verify checksum
            let calculated_checksum =
                self.calculate_checksum(&package.path, ChecksumAlgorithm::Sha256)?;
            if calculated_checksum != package.checksum {
                return Err(SemanticError::Internal {
                    message: format!("Checksum mismatch for package: {}", package.name),
                });
            }

            // Verify signature if present
            if let Some(_signature) = &package.signature {
                // In a real implementation, would verify the signature
                println!("Verifying signature for package: {}", package.name);
            }
        }

        Ok(())
    }
}

/// Package information
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// Package name
    pub name: String,

    /// Package file path
    pub path: PathBuf,

    /// Package format
    pub format: PackageFormat,

    /// Target platform
    pub platform: PlatformTarget,

    /// Package size
    pub size: u64,

    /// Package checksum
    pub checksum: String,

    /// Package signature
    pub signature: Option<String>,

    /// Creation timestamp
    pub created_at: std::time::SystemTime,

    /// Package metadata
    pub metadata: PackageMetadata,
}

impl PackageFormat {
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            PackageFormat::Zip => "ZIP",
            PackageFormat::TarGz => "TAR.GZ",
            PackageFormat::TarXz => "TAR.XZ",
            PackageFormat::Deb => "DEB",
            PackageFormat::Rpm => "RPM",
            PackageFormat::Msi => "MSI",
            PackageFormat::Pkg => "PKG",
            PackageFormat::Dmg => "DMG",
            PackageFormat::AppImage => "AppImage",
            PackageFormat::Snap => "Snap",
            PackageFormat::Flatpak => "Flatpak",
            PackageFormat::Docker => "Docker",
            PackageFormat::Oci => "OCI",
            PackageFormat::Homebrew => "Homebrew",
            PackageFormat::Chocolatey => "Chocolatey",
            PackageFormat::Npm => "NPM",
            PackageFormat::PyPi => "PyPI",
            PackageFormat::Custom(name) => name,
        }
        .to_string()
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            default_algorithm: CompressionAlgorithm::Gzip,
            level: 6,
            options: HashMap::new(),
        }
    }
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            compression_level: 6,
            include_hidden: false,
            preserve_permissions: true,
            preserve_timestamps: true,
            format_options: HashMap::new(),
        }
    }
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            checksums: true,
            checksum_algorithms: vec![ChecksumAlgorithm::Sha256],
            verify_signatures: false,
            trusted_certs: Vec::new(),
            strict_mode: false,
        }
    }
}

#[cfg(test)]
#[path = "packaging_tests.rs"]
mod packaging_tests;
