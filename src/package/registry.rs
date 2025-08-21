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

//! Package registry integration for AetherScript packages
//!
//! Provides registry client functionality for publishing, downloading,
//! and searching packages from remote registries.

use crate::error::SemanticError;
use crate::package::{PackageConfig, PackageInfo, SearchOptions, SearchSort};
use crate::package::manifest::PackageManifest;
use crate::package::version::Version;
use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Registry client for package operations
#[derive(Debug)]
pub struct RegistryClient {
    /// HTTP client
    client: HttpClient,
    
    /// Registry configuration
    config: RegistryConfig,
    
    /// Authentication tokens
    auth_tokens: HashMap<String, String>,
    
    /// Request cache
    cache: RequestCache,
}

/// Registry configuration
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Default registry URL
    pub default_url: String,
    
    /// Registry endpoints
    pub registries: HashMap<String, RegistryEndpoint>,
    
    /// Request timeout
    pub timeout: std::time::Duration,
    
    /// Maximum retries
    pub max_retries: u32,
    
    /// User agent string
    pub user_agent: String,
}

/// Registry endpoint configuration
#[derive(Debug, Clone)]
pub struct RegistryEndpoint {
    /// Base URL
    pub url: String,
    
    /// API version
    pub api_version: String,
    
    /// Authentication method
    pub auth_method: AuthMethod,
    
    /// SSL verification
    pub ssl_verify: bool,
    
    /// Custom headers
    pub headers: HashMap<String, String>,
}

/// Authentication methods
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// No authentication
    None,
    
    /// API key authentication
    ApiKey { header: String },
    
    /// Bearer token authentication
    Bearer,
    
    /// Basic authentication
    Basic,
    
    /// OAuth 2.0
    OAuth2 { 
        client_id: String,
        client_secret: String,
        token_url: String,
    },
}

/// HTTP client wrapper
#[derive(Debug)]
pub struct HttpClient {
    /// HTTP client
    client: reqwest::Client,
}

/// Request cache for performance
#[derive(Debug, Default)]
pub struct RequestCache {
    /// Package metadata cache
    package_metadata: HashMap<String, CachedResponse<PackageMetadata>>,
    
    /// Version list cache
    version_lists: HashMap<String, CachedResponse<Vec<Version>>>,
    
    /// Search result cache
    search_results: HashMap<String, CachedResponse<Vec<PackageInfo>>>,
    
    /// Download URL cache
    download_urls: HashMap<(String, Version), CachedResponse<String>>,
}

/// Cached response with TTL
#[derive(Debug, Clone)]
pub struct CachedResponse<T> {
    /// Cached data
    pub data: T,
    
    /// Cache timestamp
    pub cached_at: std::time::SystemTime,
    
    /// Time to live
    pub ttl: std::time::Duration,
}

/// Package metadata from registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Package name
    pub name: String,
    
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
    
    /// Available versions
    pub versions: Vec<VersionMetadata>,
    
    /// Download statistics
    pub downloads: DownloadStats,
    
    /// Package creation date
    pub created_at: std::time::SystemTime,
    
    /// Last update date
    pub updated_at: std::time::SystemTime,
}

/// Version metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    /// Version number
    pub version: Version,
    
    /// Version description
    pub description: Option<String>,
    
    /// Download URL
    pub download_url: String,
    
    /// Package size in bytes
    pub size: u64,
    
    /// SHA256 checksum
    pub checksum: String,
    
    /// Whether version is yanked
    pub yanked: bool,
    
    /// Yanked reason
    pub yanked_reason: Option<String>,
    
    /// Version creation date
    pub created_at: std::time::SystemTime,
    
    /// Dependencies for this version
    pub dependencies: Vec<DependencyInfo>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Dependency name
    pub name: String,
    
    /// Version requirement
    pub version_req: String,
    
    /// Dependency type
    pub dep_type: DependencyType,
    
    /// Whether optional
    pub optional: bool,
    
    /// Target platform
    pub target: Option<String>,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Normal,
    Development,
    Build,
}

/// Download statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStats {
    /// Total downloads
    pub total: u64,
    
    /// Downloads in last 30 days
    pub recent: u64,
    
    /// Downloads by version
    pub by_version: HashMap<String, u64>,
}

/// Package publish request
#[derive(Debug, Serialize)]
pub struct PublishRequest {
    /// Package manifest
    pub manifest: PackageManifest,
    
    /// Package archive data
    pub archive_data: Vec<u8>,
    
    /// Archive checksum
    pub checksum: String,
    
    /// Signature (if required)
    pub signature: Option<String>,
}

/// Registry API response
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    /// Success flag
    pub success: bool,
    
    /// Response data
    pub data: Option<T>,
    
    /// Error message
    pub error: Option<String>,
    
    /// Warnings
    pub warnings: Vec<String>,
    
    /// Request metadata
    pub metadata: ResponseMetadata,
}

/// Response metadata
#[derive(Debug, Deserialize)]
pub struct ResponseMetadata {
    /// Request ID
    pub request_id: String,
    
    /// Response time
    pub response_time_ms: u64,
    
    /// Rate limit information
    pub rate_limit: Option<RateLimitInfo>,
}

/// Rate limiting information
#[derive(Debug, Deserialize)]
pub struct RateLimitInfo {
    /// Requests remaining
    pub remaining: u32,
    
    /// Rate limit reset time
    pub reset_at: std::time::SystemTime,
    
    /// Total limit
    pub limit: u32,
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(config: PackageConfig) -> Result<Self, SemanticError> {
        let registry_config = RegistryConfig {
            default_url: config.default_registry.clone(),
            registries: config.registries.iter().map(|(name, url)| {
                (name.clone(), RegistryEndpoint {
                    url: url.clone(),
                    api_version: "v1".to_string(),
                    auth_method: AuthMethod::None,
                    ssl_verify: config.network.ssl_verify,
                    headers: HashMap::new(),
                })
            }).collect(),
            timeout: std::time::Duration::from_secs(config.network.timeout),
            max_retries: config.network.max_retries,
            user_agent: format!("aether-package-manager/{}", env!("CARGO_PKG_VERSION")),
        };
        
        let client = HttpClient {
            client: reqwest::Client::new(),
        };
        
        Ok(Self {
            client,
            config: registry_config,
            auth_tokens: HashMap::new(),
            cache: RequestCache::default(),
        })
    }
    
    /// Get package information
    pub fn get_package_info(&mut self, package_name: &str) -> Result<PackageInfo, SemanticError> {
        // Check cache first
        if let Some(cached) = self.cache.get_package_metadata(package_name) {
            return Ok(self.convert_metadata_to_info(&cached.data));
        }
        
        // Fetch from registry
        let url = format!("{}/api/v1/packages/{}", self.config.default_url, package_name);
        let response: ApiResponse<PackageMetadata> = self.client.get(&url, &self.auth_tokens)?;
        
        if !response.success {
            return Err(SemanticError::Internal {
                message: response.error.unwrap_or_else(|| "Unknown registry error".to_string()),
            });
        }
        
        let metadata = response.data.ok_or_else(|| SemanticError::Internal {
            message: "Empty response from registry".to_string(),
        })?;
        
        // Cache the result
        self.cache.cache_package_metadata(package_name, metadata.clone());
        
        Ok(self.convert_metadata_to_info(&metadata))
    }
    
    /// Get available versions for a package
    pub fn get_package_versions(&mut self, package_name: &str) -> Result<Vec<Version>, SemanticError> {
        // Check cache first
        if let Some(cached) = self.cache.get_version_list(package_name) {
            return Ok(cached.data.clone());
        }
        
        // Fetch from registry
        let url = format!("{}/api/v1/packages/{}/versions", self.config.default_url, package_name);
        let response: ApiResponse<Vec<VersionMetadata>> = self.client.get(&url, &self.auth_tokens)?;
        
        if !response.success {
            return Err(SemanticError::Internal {
                message: response.error.unwrap_or_else(|| "Unknown registry error".to_string()),
            });
        }
        
        let version_metadata = response.data.ok_or_else(|| SemanticError::Internal {
            message: "Empty response from registry".to_string(),
        })?;
        
        let versions: Vec<Version> = version_metadata.iter()
            .filter(|v| !v.yanked)
            .map(|v| v.version.clone())
            .collect();
        
        // Cache the result
        self.cache.cache_version_list(package_name, versions.clone());
        
        Ok(versions)
    }
    
    /// Get latest version of a package
    pub fn get_latest_version(&mut self, package_name: &str) -> Result<Version, SemanticError> {
        let versions = self.get_package_versions(package_name)?;
        
        versions.into_iter()
            .max()
            .ok_or_else(|| SemanticError::Internal {
                message: format!("No versions available for package {}", package_name),
            })
    }
    
    /// Get package manifest for a specific version
    pub fn get_package_manifest(&mut self, package_name: &str, version: &Version) -> Result<PackageManifest, SemanticError> {
        let url = format!("{}/api/v1/packages/{}/{}/manifest", 
            self.config.default_url, package_name, version);
        
        let response: ApiResponse<PackageManifest> = self.client.get(&url, &self.auth_tokens)?;
        
        if !response.success {
            return Err(SemanticError::Internal {
                message: response.error.unwrap_or_else(|| "Unknown registry error".to_string()),
            });
        }
        
        response.data.ok_or_else(|| SemanticError::Internal {
            message: "Empty manifest response from registry".to_string(),
        })
    }
    
    /// Download a package
    pub fn download_package(&mut self, package_name: &str, version: &Version) -> Result<Vec<u8>, SemanticError> {
        // Get download URL
        let download_url = self.get_download_url(package_name, version)?;
        
        // Download package data
        let package_data = self.client.download(&download_url)?;
        
        // Verify checksum
        self.verify_package_checksum(package_name, version, &package_data)?;
        
        Ok(package_data)
    }
    
    /// Search for packages
    pub fn search_packages(&mut self, options: SearchOptions) -> Result<Vec<PackageInfo>, SemanticError> {
        let cache_key = format!("{}:{:?}:{:?}", options.query, options.limit, options.sort_by);
        
        // Check cache first
        if let Some(cached) = self.cache.get_search_results(&cache_key) {
            return Ok(cached.data.clone());
        }
        
        // Build search URL
        let mut url = format!("{}/api/v1/search?q={}", self.config.default_url, 
            crate::external_stubs::urlencoding::encode(&options.query));
        
        if let Some(limit) = options.limit {
            url.push_str(&format!("&limit={}", limit));
        }
        
        match options.sort_by {
            SearchSort::Relevance => url.push_str("&sort=relevance"),
            SearchSort::Downloads => url.push_str("&sort=downloads"),
            SearchSort::Name => url.push_str("&sort=name"),
            SearchSort::Updated => url.push_str("&sort=updated"),
        }
        
        if options.include_prerelease {
            url.push_str("&include_prerelease=true");
        }
        
        let response: ApiResponse<Vec<PackageMetadata>> = self.client.get(&url, &self.auth_tokens)?;
        
        if !response.success {
            return Err(SemanticError::Internal {
                message: response.error.unwrap_or_else(|| "Search failed".to_string()),
            });
        }
        
        let search_results = response.data.unwrap_or_default();
        let package_infos: Vec<PackageInfo> = search_results.iter()
            .map(|metadata| self.convert_metadata_to_info(metadata))
            .collect();
        
        // Cache the result
        self.cache.cache_search_results(&cache_key, package_infos.clone());
        
        Ok(package_infos)
    }
    
    /// Publish a package
    pub fn publish_package(&mut self, manifest: &PackageManifest, archive_path: &PathBuf) -> Result<(), SemanticError> {
        // Read archive data
        let archive_data = std::fs::read(archive_path)
            .map_err(|e| SemanticError::Internal {
                message: format!("Failed to read package archive: {}", e),
            })?;
        
        // Calculate checksum
        let checksum = self.calculate_checksum(&archive_data)?;
        
        // Create publish request
        let publish_request = PublishRequest {
            manifest: manifest.clone(),
            archive_data,
            checksum,
            signature: None, // TODO: Implement package signing
        };
        
        // Send publish request
        let url = format!("{}/api/v1/packages", self.config.default_url);
        let response: ApiResponse<()> = self.client.post(&url, &publish_request, &self.auth_tokens)?;
        
        if !response.success {
            return Err(SemanticError::Internal {
                message: response.error.unwrap_or_else(|| "Publish failed".to_string()),
            });
        }
        
        Ok(())
    }
    
    /// Set authentication token for a registry
    pub fn set_auth_token(&mut self, registry: &str, token: String) {
        self.auth_tokens.insert(registry.to_string(), token);
    }
    
    /// Clear authentication tokens
    pub fn clear_auth_tokens(&mut self) {
        self.auth_tokens.clear();
    }
    
    // Helper methods
    
    fn get_download_url(&mut self, package_name: &str, version: &Version) -> Result<String, SemanticError> {
        let cache_key = (package_name.to_string(), version.clone());
        
        // Check cache first
        if let Some(cached) = self.cache.get_download_url(&cache_key) {
            return Ok(cached.data.clone());
        }
        
        let url = format!("{}/api/v1/packages/{}/{}/download", 
            self.config.default_url, package_name, version);
        
        let response: ApiResponse<String> = self.client.get(&url, &self.auth_tokens)?;
        
        if !response.success {
            return Err(SemanticError::Internal {
                message: response.error.unwrap_or_else(|| "Failed to get download URL".to_string()),
            });
        }
        
        let download_url = response.data.ok_or_else(|| SemanticError::Internal {
            message: "Empty download URL response".to_string(),
        })?;
        
        // Cache the result
        self.cache.cache_download_url(&cache_key, download_url.clone());
        
        Ok(download_url)
    }
    
    fn verify_package_checksum(&self, package_name: &str, version: &Version, data: &[u8]) -> Result<(), SemanticError> {
        let computed_checksum = self.calculate_checksum(data)?;
        
        // Get expected checksum from registry
        let url = format!("{}/api/v1/packages/{}/{}/checksum", 
            self.config.default_url, package_name, version);
        
        let response: ApiResponse<String> = self.client.get(&url, &self.auth_tokens)?;
        
        if !response.success {
            return Err(SemanticError::Internal {
                message: "Failed to get package checksum".to_string(),
            });
        }
        
        let expected_checksum = response.data.ok_or_else(|| SemanticError::Internal {
            message: "Empty checksum response".to_string(),
        })?;
        
        if computed_checksum != expected_checksum {
            return Err(SemanticError::Internal {
                message: format!("Package checksum mismatch for {} {}", package_name, version),
            });
        }
        
        Ok(())
    }
    
    fn calculate_checksum(&self, data: &[u8]) -> Result<String, SemanticError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        Ok(format!("{:x}", hasher.finish()))
    }
    
    fn convert_metadata_to_info(&self, metadata: &PackageMetadata) -> PackageInfo {
        PackageInfo {
            name: metadata.name.clone(),
            version: metadata.versions.first()
                .map(|v| v.version.clone())
                .unwrap_or_else(|| Version::new(0, 0, 0)),
            description: metadata.description.clone(),
            authors: metadata.authors.clone(),
            license: metadata.license.clone(),
            homepage: metadata.homepage.clone(),
            repository: metadata.repository.clone(),
            keywords: metadata.keywords.clone(),
            categories: metadata.categories.clone(),
            downloads: metadata.downloads.total,
            updated_at: metadata.updated_at,
        }
    }
}

impl HttpClient {
    fn get<T>(&self, _url: &str, _auth_tokens: &HashMap<String, String>) -> Result<ApiResponse<T>, SemanticError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Simplified HTTP GET implementation
        // In a real implementation, this would use reqwest or similar
        Err(SemanticError::Internal {
            message: "HTTP client not implemented".to_string(),
        })
    }
    
    fn post<T, U>(&self, _url: &str, _body: &T, _auth_tokens: &HashMap<String, String>) -> Result<ApiResponse<U>, SemanticError>
    where
        T: Serialize,
        U: for<'de> Deserialize<'de>,
    {
        // Simplified HTTP POST implementation
        Err(SemanticError::Internal {
            message: "HTTP client not implemented".to_string(),
        })
    }
    
    fn download(&self, _url: &str) -> Result<Vec<u8>, SemanticError> {
        // Simplified download implementation
        Err(SemanticError::Internal {
            message: "Download not implemented".to_string(),
        })
    }
}

impl RequestCache {
    fn get_package_metadata(&self, package_name: &str) -> Option<&CachedResponse<PackageMetadata>> {
        let cached = self.package_metadata.get(package_name)?;
        if cached.is_expired() {
            return None;
        }
        Some(cached)
    }
    
    fn cache_package_metadata(&mut self, package_name: &str, metadata: PackageMetadata) {
        let cached = CachedResponse {
            data: metadata,
            cached_at: std::time::SystemTime::now(),
            ttl: std::time::Duration::from_secs(300), // 5 minutes
        };
        self.package_metadata.insert(package_name.to_string(), cached);
    }
    
    fn get_version_list(&self, package_name: &str) -> Option<&CachedResponse<Vec<Version>>> {
        let cached = self.version_lists.get(package_name)?;
        if cached.is_expired() {
            return None;
        }
        Some(cached)
    }
    
    fn cache_version_list(&mut self, package_name: &str, versions: Vec<Version>) {
        let cached = CachedResponse {
            data: versions,
            cached_at: std::time::SystemTime::now(),
            ttl: std::time::Duration::from_secs(300), // 5 minutes
        };
        self.version_lists.insert(package_name.to_string(), cached);
    }
    
    fn get_search_results(&self, cache_key: &str) -> Option<&CachedResponse<Vec<PackageInfo>>> {
        let cached = self.search_results.get(cache_key)?;
        if cached.is_expired() {
            return None;
        }
        Some(cached)
    }
    
    fn cache_search_results(&mut self, cache_key: &str, results: Vec<PackageInfo>) {
        let cached = CachedResponse {
            data: results,
            cached_at: std::time::SystemTime::now(),
            ttl: std::time::Duration::from_secs(600), // 10 minutes
        };
        self.search_results.insert(cache_key.to_string(), cached);
    }
    
    fn get_download_url(&self, cache_key: &(String, Version)) -> Option<&CachedResponse<String>> {
        let cached = self.download_urls.get(cache_key)?;
        if cached.is_expired() {
            return None;
        }
        Some(cached)
    }
    
    fn cache_download_url(&mut self, cache_key: &(String, Version), url: String) {
        let cached = CachedResponse {
            data: url,
            cached_at: std::time::SystemTime::now(),
            ttl: std::time::Duration::from_secs(3600), // 1 hour
        };
        self.download_urls.insert(cache_key.clone(), cached);
    }
}

impl<T> CachedResponse<T> {
    fn is_expired(&self) -> bool {
        if let Ok(elapsed) = self.cached_at.elapsed() {
            elapsed > self.ttl
        } else {
            true // If we can't determine elapsed time, consider it expired
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::PackageConfig;
    
    #[test]
    fn test_registry_config_creation() {
        let package_config = PackageConfig::default();
        let registry_config = RegistryConfig {
            default_url: package_config.default_registry.clone(),
            registries: HashMap::new(),
            timeout: std::time::Duration::from_secs(30),
            max_retries: 3,
            user_agent: "test-agent".to_string(),
        };
        
        assert_eq!(registry_config.default_url, "https://registry.aetherscript.org");
        assert_eq!(registry_config.max_retries, 3);
    }
    
    #[test]
    fn test_cached_response_expiry() {
        let cached = CachedResponse {
            data: "test".to_string(),
            cached_at: std::time::SystemTime::now() - std::time::Duration::from_secs(3600),
            ttl: std::time::Duration::from_secs(300),
        };
        
        assert!(cached.is_expired());
        
        let fresh = CachedResponse {
            data: "test".to_string(),
            cached_at: std::time::SystemTime::now(),
            ttl: std::time::Duration::from_secs(300),
        };
        
        assert!(!fresh.is_expired());
    }
    
    #[test]
    fn test_auth_methods() {
        let api_key = AuthMethod::ApiKey {
            header: "X-API-Key".to_string(),
        };
        
        assert!(matches!(api_key, AuthMethod::ApiKey { .. }));
        
        let bearer = AuthMethod::Bearer;
        assert!(matches!(bearer, AuthMethod::Bearer));
    }
    
    #[test]
    fn test_dependency_info() {
        let dep = DependencyInfo {
            name: "test-dep".to_string(),
            version_req: "^1.0.0".to_string(),
            dep_type: DependencyType::Normal,
            optional: false,
            target: None,
        };
        
        assert_eq!(dep.name, "test-dep");
        assert!(!dep.optional);
        assert!(matches!(dep.dep_type, DependencyType::Normal));
    }
}