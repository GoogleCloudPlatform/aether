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

//! Package management system for AetherScript
//!
//! Provides comprehensive package management including dependency resolution,
//! version management, registry integration, and build script support.

pub mod builder;
pub mod manifest;
pub mod registry;
pub mod resolver;
pub mod version;

use crate::error::SemanticError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Package manager for AetherScript
#[derive(Debug)]
pub struct PackageManager {
    /// Package registry client
    registry: registry::RegistryClient,

    /// Dependency resolver
    resolver: resolver::DependencyResolver,

    /// Package builder
    builder: builder::PackageBuilder,

    /// Local package cache
    cache: PackageCache,

    /// Configuration
    config: PackageConfig,
}

/// Package management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    /// Default registry URL
    pub default_registry: String,

    /// Additional registries
    pub registries: HashMap<String, String>,

    /// Cache directory
    pub cache_dir: PathBuf,

    /// Global package directory
    pub global_dir: PathBuf,

    /// Build configuration
    pub build_config: BuildConfig,

    /// Network settings
    pub network: NetworkConfig,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Default optimization level
    pub optimization_level: u8,

    /// Enable debug info by default
    pub debug_info: bool,

    /// Default target triple
    pub target_triple: Option<String>,

    /// Parallel build jobs
    pub jobs: Option<usize>,

    /// Build timeout in seconds
    pub timeout: u64,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// HTTP timeout in seconds
    pub timeout: u64,

    /// Maximum retries
    pub max_retries: u32,

    /// Use proxy
    pub proxy: Option<String>,

    /// SSL verification
    pub ssl_verify: bool,
}

/// Local package cache
#[derive(Debug, Default)]
pub struct PackageCache {
    /// Cached packages by name and version
    packages: HashMap<String, HashMap<version::Version, CachedPackage>>,

    /// Package metadata cache
    metadata_cache: HashMap<String, manifest::PackageManifest>,

    /// Cache statistics
    stats: CacheStats,
}

/// Cached package information
#[derive(Debug, Clone)]
pub struct CachedPackage {
    /// Package manifest
    pub manifest: manifest::PackageManifest,

    /// Local path to package
    pub path: PathBuf,

    /// When package was cached
    pub cached_at: std::time::SystemTime,

    /// Package size in bytes
    pub size: u64,

    /// Package hash for integrity
    pub hash: String,
}

/// Cache statistics
#[derive(Debug, Default)]
pub struct CacheStats {
    /// Total packages cached
    pub total_packages: usize,

    /// Total cache size in bytes
    pub total_size: u64,

    /// Cache hits
    pub hits: u64,

    /// Cache misses
    pub misses: u64,
}

/// Package installation options
#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    /// Install globally
    pub global: bool,

    /// Skip dependency resolution
    pub no_deps: bool,

    /// Force reinstallation
    pub force: bool,

    /// Specific version to install
    pub version: Option<version::VersionRequirement>,

    /// Development dependencies
    pub dev: bool,

    /// Optional dependencies
    pub optional: bool,
}

/// Package search options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Search query
    pub query: String,

    /// Limit results
    pub limit: Option<usize>,

    /// Include pre-releases
    pub include_prerelease: bool,

    /// Sort by relevance/downloads/name
    pub sort_by: SearchSort,
}

/// Search sorting options
#[derive(Debug, Clone)]
pub enum SearchSort {
    Relevance,
    Downloads,
    Name,
    Updated,
}

/// Package operation result
#[derive(Debug)]
pub struct PackageOperation {
    /// Packages that were installed
    pub installed: Vec<PackageInfo>,

    /// Packages that were updated
    pub updated: Vec<PackageInfo>,

    /// Packages that were removed
    pub removed: Vec<PackageInfo>,

    /// Total operation time
    pub duration: std::time::Duration,

    /// Any warnings
    pub warnings: Vec<String>,
}

/// Package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name
    pub name: String,

    /// Package version
    pub version: version::Version,

    /// Package description
    pub description: Option<String>,

    /// Package authors
    pub authors: Vec<String>,

    /// Package license
    pub license: Option<String>,

    /// Package homepage
    pub homepage: Option<String>,

    /// Package repository
    pub repository: Option<String>,

    /// Package keywords
    pub keywords: Vec<String>,

    /// Package categories
    pub categories: Vec<String>,

    /// Download count
    pub downloads: u64,

    /// Last updated
    pub updated_at: std::time::SystemTime,
}

impl PackageManager {
    /// Create a new package manager
    pub fn new(config: PackageConfig) -> Result<Self, SemanticError> {
        let registry = registry::RegistryClient::new(config.clone())?;
        let resolver = resolver::DependencyResolver::new();
        let builder = builder::PackageBuilder::new(config.build_config.clone());
        let cache = PackageCache::new(&config.cache_dir)?;

        Ok(Self {
            registry,
            resolver,
            builder,
            cache,
            config,
        })
    }

    /// Install a package
    pub fn install(
        &mut self,
        package_name: &str,
        options: InstallOptions,
    ) -> Result<PackageOperation, SemanticError> {
        let start_time = std::time::Instant::now();
        let mut operation = PackageOperation {
            installed: Vec::new(),
            updated: Vec::new(),
            removed: Vec::new(),
            duration: std::time::Duration::default(),
            warnings: Vec::new(),
        };

        // Resolve package version
        let version_req = options
            .version
            .clone()
            .unwrap_or(version::VersionRequirement::any());
        let package_info = self.registry.get_package_info(package_name)?;
        let selected_version = self.select_version(&package_info, &version_req)?;

        // Check if already installed
        if !options.force && self.is_installed(package_name, &selected_version)? {
            operation.warnings.push(format!(
                "Package {} {} is already installed",
                package_name, selected_version
            ));
            return Ok(operation);
        }

        // Resolve dependencies
        let dependencies = if options.no_deps {
            Vec::new()
        } else {
            self.resolve_dependencies(package_name, &selected_version, &options)?
        };

        // Install dependencies first
        for dep in dependencies {
            let dep_info = self.install_package_version(&dep.name, &dep.version, &options)?;
            operation.installed.push(dep_info);
        }

        // Install the main package
        let main_package =
            self.install_package_version(package_name, &selected_version, &options)?;
        operation.installed.push(main_package);

        operation.duration = start_time.elapsed();
        Ok(operation)
    }

    /// Uninstall a package
    pub fn uninstall(&mut self, package_name: &str) -> Result<PackageOperation, SemanticError> {
        let start_time = std::time::Instant::now();
        let mut operation = PackageOperation {
            installed: Vec::new(),
            updated: Vec::new(),
            removed: Vec::new(),
            duration: std::time::Duration::default(),
            warnings: Vec::new(),
        };

        // Check if package is installed
        if !self.is_package_installed(package_name)? {
            return Err(SemanticError::Internal {
                message: format!("Package {} is not installed", package_name),
            });
        }

        // Get installed version
        let installed_version = self.get_installed_version(package_name)?;

        // Check for dependent packages
        let dependents = self.find_dependents(package_name)?;
        if !dependents.is_empty() {
            return Err(SemanticError::Internal {
                message: format!(
                    "Cannot uninstall {}: depended on by {}",
                    package_name,
                    dependents.join(", ")
                ),
            });
        }

        // Remove package
        self.remove_package(package_name, &installed_version)?;

        let package_info = PackageInfo {
            name: package_name.to_string(),
            version: installed_version,
            description: None,
            authors: vec![],
            license: None,
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
            downloads: 0,
            updated_at: std::time::SystemTime::now(),
        };

        operation.removed.push(package_info);
        operation.duration = start_time.elapsed();
        Ok(operation)
    }

    /// Update a package
    pub fn update(
        &mut self,
        package_name: Option<&str>,
    ) -> Result<PackageOperation, SemanticError> {
        let start_time = std::time::Instant::now();
        let mut operation = PackageOperation {
            installed: Vec::new(),
            updated: Vec::new(),
            removed: Vec::new(),
            duration: std::time::Duration::default(),
            warnings: Vec::new(),
        };

        let packages_to_update = if let Some(name) = package_name {
            vec![name.to_string()]
        } else {
            self.list_installed_packages()?
        };

        for package in packages_to_update {
            if let Some(updated_info) = self.update_single_package(&package)? {
                operation.updated.push(updated_info);
            }
        }

        operation.duration = start_time.elapsed();
        Ok(operation)
    }

    /// Search for packages
    pub fn search(&mut self, options: SearchOptions) -> Result<Vec<PackageInfo>, SemanticError> {
        self.registry.search_packages(options)
    }

    /// List installed packages
    pub fn list(&self) -> Result<Vec<PackageInfo>, SemanticError> {
        let installed_packages = self.list_installed_packages()?;
        let mut package_infos = Vec::new();

        for package_name in installed_packages {
            if let Ok(version) = self.get_installed_version(&package_name) {
                let package_info = PackageInfo {
                    name: package_name,
                    version,
                    description: None,
                    authors: vec![],
                    license: None,
                    homepage: None,
                    repository: None,
                    keywords: vec![],
                    categories: vec![],
                    downloads: 0,
                    updated_at: std::time::SystemTime::now(),
                };
                package_infos.push(package_info);
            }
        }

        Ok(package_infos)
    }

    /// Build a package
    pub fn build(&mut self, manifest_path: Option<PathBuf>) -> Result<(), SemanticError> {
        let manifest_path = manifest_path.unwrap_or_else(|| PathBuf::from("Package.toml"));
        let manifest = manifest::PackageManifest::load(&manifest_path)?;

        self.builder.build_package(&manifest)?;
        Ok(())
    }

    /// Publish a package
    pub fn publish(&mut self, manifest_path: Option<PathBuf>) -> Result<(), SemanticError> {
        let manifest_path = manifest_path.unwrap_or_else(|| PathBuf::from("Package.toml"));
        let manifest = manifest::PackageManifest::load(&manifest_path)?;

        // Build package first
        self.builder.build_package(&manifest)?;

        // Create package archive
        let archive_path = self.builder.create_package_archive(&manifest)?;

        // Publish to registry
        self.registry.publish_package(&manifest, &archive_path)?;

        Ok(())
    }

    /// Clean cache
    pub fn clean_cache(&mut self) -> Result<(), SemanticError> {
        self.cache.clear()?;
        Ok(())
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> &CacheStats {
        &self.cache.stats
    }

    // Helper methods

    fn select_version(
        &mut self,
        package_info: &PackageInfo,
        version_req: &version::VersionRequirement,
    ) -> Result<version::Version, SemanticError> {
        // Get available versions from registry
        let available_versions = self.registry.get_package_versions(&package_info.name)?;

        // Find best matching version
        for version in available_versions.iter().rev() {
            // Start with newest
            if version_req.matches(version) {
                return Ok(version.clone());
            }
        }

        Err(SemanticError::Internal {
            message: format!(
                "No version of {} matches requirement {}",
                package_info.name, version_req
            ),
        })
    }

    fn resolve_dependencies(
        &mut self,
        package_name: &str,
        version: &version::Version,
        options: &InstallOptions,
    ) -> Result<Vec<resolver::ResolvedDependency>, SemanticError> {
        let manifest = self.registry.get_package_manifest(package_name, version)?;

        let mut dependencies = manifest.dependencies.clone();

        if options.dev {
            dependencies.extend(manifest.dev_dependencies.clone());
        }

        if options.optional {
            dependencies.extend(manifest.optional_dependencies.clone());
        }

        self.resolver.resolve(dependencies)
    }

    fn install_package_version(
        &mut self,
        name: &str,
        version: &version::Version,
        options: &InstallOptions,
    ) -> Result<PackageInfo, SemanticError> {
        // Check cache first
        if let Some(cached) = self.cache.get_package(name, version) {
            return Ok(PackageInfo {
                name: name.to_string(),
                version: version.clone(),
                description: cached.manifest.package.description.clone(),
                authors: cached.manifest.package.authors.clone(),
                license: cached.manifest.package.license.clone(),
                homepage: cached.manifest.package.homepage.clone(),
                repository: cached.manifest.package.repository.clone(),
                keywords: cached.manifest.package.keywords.clone(),
                categories: cached.manifest.package.categories.clone(),
                downloads: 0,
                updated_at: cached.cached_at,
            });
        }

        // Download package
        let package_data = self.registry.download_package(name, version)?;

        // Install package
        let install_path = if options.global {
            self.config.global_dir.join(name).join(version.to_string())
        } else {
            std::env::current_dir()?
                .join("packages")
                .join(name)
                .join(version.to_string())
        };

        std::fs::create_dir_all(&install_path)?;

        // Extract package
        self.extract_package(&package_data, &install_path)?;

        // Cache package
        let manifest = manifest::PackageManifest::load(&install_path.join("Package.toml"))?;
        self.cache
            .add_package(name, version, &manifest, &install_path)?;

        Ok(PackageInfo {
            name: name.to_string(),
            version: version.clone(),
            description: manifest.package.description,
            authors: manifest.package.authors,
            license: manifest.package.license,
            homepage: manifest.package.homepage,
            repository: manifest.package.repository,
            keywords: manifest.package.keywords,
            categories: manifest.package.categories,
            downloads: 0,
            updated_at: std::time::SystemTime::now(),
        })
    }

    fn is_installed(&self, name: &str, version: &version::Version) -> Result<bool, SemanticError> {
        Ok(self.cache.has_package(name, version))
    }

    fn is_package_installed(&self, name: &str) -> Result<bool, SemanticError> {
        Ok(self.cache.has_any_version(name))
    }

    fn get_installed_version(&self, name: &str) -> Result<version::Version, SemanticError> {
        self.cache
            .get_latest_version(name)
            .ok_or_else(|| SemanticError::Internal {
                message: format!("Package {} is not installed", name),
            })
    }

    fn find_dependents(&self, name: &str) -> Result<Vec<String>, SemanticError> {
        // Find packages that depend on this one
        let mut dependents = Vec::new();

        for (package_name, versions) in &self.cache.packages {
            for cached_package in versions.values() {
                if cached_package
                    .manifest
                    .dependencies
                    .iter()
                    .any(|dep| dep.name == name)
                {
                    dependents.push(package_name.clone());
                    break;
                }
            }
        }

        Ok(dependents)
    }

    fn remove_package(
        &mut self,
        name: &str,
        version: &version::Version,
    ) -> Result<(), SemanticError> {
        self.cache.remove_package(name, version)?;

        // Remove from filesystem
        let package_path = self.config.cache_dir.join(name).join(version.to_string());
        if package_path.exists() {
            std::fs::remove_dir_all(package_path)?;
        }

        Ok(())
    }

    fn list_installed_packages(&self) -> Result<Vec<String>, SemanticError> {
        Ok(self.cache.packages.keys().cloned().collect())
    }

    fn update_single_package(&mut self, name: &str) -> Result<Option<PackageInfo>, SemanticError> {
        let current_version = self.get_installed_version(name)?;
        let latest_version = self.registry.get_latest_version(name)?;

        if latest_version > current_version {
            let options = InstallOptions {
                global: false,
                no_deps: false,
                force: true,
                version: Some(version::VersionRequirement::exact(&latest_version)),
                dev: false,
                optional: false,
            };

            self.install_package_version(name, &latest_version, &options)
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn extract_package(
        &self,
        package_data: &[u8],
        install_path: &PathBuf,
    ) -> Result<(), SemanticError> {
        // Simplified package extraction - would use tar/zip in real implementation
        std::fs::write(install_path.join("package.bin"), package_data)?;
        Ok(())
    }
}

impl PackageCache {
    fn new(cache_dir: &PathBuf) -> Result<Self, SemanticError> {
        std::fs::create_dir_all(cache_dir)?;

        Ok(Self {
            packages: HashMap::new(),
            metadata_cache: HashMap::new(),
            stats: CacheStats::default(),
        })
    }

    fn get_package(&mut self, name: &str, version: &version::Version) -> Option<&CachedPackage> {
        if let Some(versions) = self.packages.get(name) {
            if let Some(package) = versions.get(version) {
                self.stats.hits += 1;
                return Some(package);
            }
        }
        self.stats.misses += 1;
        None
    }

    fn has_package(&self, name: &str, version: &version::Version) -> bool {
        self.packages
            .get(name)
            .map(|versions| versions.contains_key(version))
            .unwrap_or(false)
    }

    fn has_any_version(&self, name: &str) -> bool {
        self.packages.contains_key(name)
    }

    fn get_latest_version(&self, name: &str) -> Option<version::Version> {
        self.packages.get(name)?.keys().max().cloned()
    }

    fn add_package(
        &mut self,
        name: &str,
        version: &version::Version,
        manifest: &manifest::PackageManifest,
        path: &PathBuf,
    ) -> Result<(), SemanticError> {
        let size = self.calculate_package_size(path)?;
        let hash = self.calculate_package_hash(path)?;

        let cached_package = CachedPackage {
            manifest: manifest.clone(),
            path: path.clone(),
            cached_at: std::time::SystemTime::now(),
            size,
            hash,
        };

        self.packages
            .entry(name.to_string())
            .or_default()
            .insert(version.clone(), cached_package);

        self.stats.total_packages += 1;
        self.stats.total_size += size;

        Ok(())
    }

    fn remove_package(
        &mut self,
        name: &str,
        version: &version::Version,
    ) -> Result<(), SemanticError> {
        if let Some(versions) = self.packages.get_mut(name) {
            if let Some(package) = versions.remove(version) {
                self.stats.total_packages -= 1;
                self.stats.total_size -= package.size;

                if versions.is_empty() {
                    self.packages.remove(name);
                }
            }
        }
        Ok(())
    }

    fn clear(&mut self) -> Result<(), SemanticError> {
        self.packages.clear();
        self.metadata_cache.clear();
        self.stats = CacheStats::default();
        Ok(())
    }

    fn calculate_package_size(&self, _path: &PathBuf) -> Result<u64, SemanticError> {
        // Simplified size calculation
        Ok(1024) // Placeholder
    }

    fn calculate_package_hash(&self, _path: &PathBuf) -> Result<String, SemanticError> {
        // Simplified hash calculation
        Ok("abc123".to_string()) // Placeholder
    }
}

impl Default for PackageConfig {
    fn default() -> Self {
        Self {
            default_registry: "https://registry.aetherscript.org".to_string(),
            registries: HashMap::new(),
            cache_dir: std::env::current_dir()
                .unwrap_or_default()
                .join(".aether/cache"),
            global_dir: std::env::current_dir()
                .unwrap_or_default()
                .join(".aether/global"),
            build_config: BuildConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            optimization_level: 2,
            debug_info: false,
            target_triple: None,
            jobs: Some(
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(1),
            ),
            timeout: 300, // 5 minutes
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            max_retries: 3,
            proxy: None,
            ssl_verify: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_config_default() {
        let config = PackageConfig::default();
        assert_eq!(config.default_registry, "https://registry.aetherscript.org");
        assert!(config.registries.is_empty());
        assert_eq!(config.build_config.optimization_level, 2);
    }

    #[test]
    fn test_install_options_default() {
        let options = InstallOptions::default();
        assert!(!options.global);
        assert!(!options.no_deps);
        assert!(!options.force);
        assert!(options.version.is_none());
    }

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats::default();
        assert_eq!(stats.total_packages, 0);
        assert_eq!(stats.total_size, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_package_info_creation() {
        let info = PackageInfo {
            name: "test-package".to_string(),
            version: version::Version::new(1, 0, 0),
            description: Some("Test package".to_string()),
            authors: vec!["Test Author".to_string()],
            license: Some("MIT".to_string()),
            homepage: None,
            repository: None,
            keywords: vec!["test".to_string()],
            categories: vec!["development".to_string()],
            downloads: 100,
            updated_at: std::time::SystemTime::now(),
        };

        assert_eq!(info.name, "test-package");
        assert_eq!(info.version.major, 1);
        assert_eq!(info.downloads, 100);
    }
}
