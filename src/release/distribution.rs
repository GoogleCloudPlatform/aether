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

//! Distribution system for AetherScript releases
//!
//! Manages distribution channels, release deployment, and content delivery
//! for AetherScript packages across multiple platforms and repositories.

use crate::error::SemanticError;
use crate::release::packaging::ArtifactInfo;
use crate::release::{self, DistributionConfig, FailoverConfig, FailoverThreshold, RecoveryConfig};
use crate::release::packaging::{PackageInfo, PackageFormat};
use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Distribution manager for handling release deployment
#[derive(Debug)]
pub struct DistributionManager {
    /// Distribution channels
    channels: Vec<DistributionChannel>,
    
    /// CDN manager
    cdn_manager: Option<CdnManager>,
    f
    /// Distribution history
    history: Vec<DistributionRecord>,
    
    /// Distribution metadata
    metadata: DistributionMetadata,
}

/// Distribution metadata
#[derive(Debug, Default)]
pub struct DistributionMetadata {
    /// Version being distributed
    pub version: String,
    
    /// Distribution assets
    pub assets: Vec<DistributionAsset>,
}

/// Distribution asset
#[derive(Debug, Clone)]
pub struct DistributionAsset {
    /// Asset name
    pub name: String,
    
    /// Asset path
    pub path: PathBuf,
    
    /// Asset size
    pub size: u64,
    
    /// Asset checksum
    pub checksum: String,
}

/// Distribution channel
#[derive(Debug, Clone)]
pub struct DistributionChannel {
    /// Channel name
    pub name: String,
    
    /// Channel type
    pub channel_type: ChannelType,
    
    /// Channel URL or endpoint
    pub url: String,
    
    /// Channel statistics
    pub stats: ChannelStats,
    
    /// Channel status
    pub status: ChannelStatus,
    
    /// Channel-specific options
    pub options: HashMap<String, String>,
    
    /// Whether the channel is enabled
    pub enabled: bool,
    
    /// Channel configuration
    pub config: ChannelConfig,
    
    /// Channel credentials
    pub credentials: Option<ChannelCredentials>,
}

/// Channel credentials
#[derive(Debug, Clone)]
pub struct ChannelCredentials {
    /// API key
    pub api_key: Option<String>,
    
    /// Username
    pub username: Option<String>,
    
    /// Password (encrypted)
    pub password: Option<String>,
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Channel name
    pub name: String,
    
    /// Channel description
    pub description: String,
    
    /// Channel endpoint URL
    pub endpoint: String,
    
    /// Supported package formats
    pub supported_formats: Vec<PackageFormat>,
    
    /// Platform targets
    pub platforms: Vec<String>,
    
    /// Release stages
    pub stages: Vec<ReleaseStage>,
    
    /// Upload configuration
    pub upload: UploadConfig,
    
    /// Validation rules
    pub validation: ValidationConfig,
    
    /// Retry configuration
    pub retry: RetryConfig,
}

/// Distribution channel types
#[derive(Debug, Clone)]
pub enum ChannelType {
    /// GitHub Releases
    GitHub {
        owner: String,
        repo: String,
    },
    
    /// GitLab Releases
    GitLab {
        project_id: String,
    },
    
    /// Package repositories
    PackageRepository {
        repository_type: RepositoryType,
        repository_url: String,
    },
    
    /// Container registries
    ContainerRegistry {
        registry_url: String,
        namespace: String,
    },
    
    /// CDN distribution
    Cdn {
        provider: CdnProvider,
        distribution_id: String,
    },
    
    /// S3-compatible storage
    S3Compatible {
        endpoint: String,
        bucket: String,
        region: String,
    },
    
    /// FTP/SFTP server
    Ftp {
        server: String,
        path: String,
        secure: bool,
    },
    
    /// Custom distribution endpoint
    Custom {
        upload_method: String,
        parameters: HashMap<String, String>,
    },
}

/// Repository types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RepositoryType {
    /// APT repository (Debian/Ubuntu)
    Apt,
    
    /// YUM repository (Red Hat/CentOS)
    Yum,
    
    /// DNF repository (Fedora)
    Dnf,
    
    /// Zypper repository (openSUSE)
    Zypper,
    
    /// Pacman repository (Arch Linux)
    Pacman,
    
    /// Homebrew tap
    Homebrew,
    
    /// Chocolatey repository
    Chocolatey,
    
    /// npm registry
    Npm,
    
    /// PyPI repository
    PyPI,
    
    /// NuGet repository
    NuGet,
    
    /// Maven repository
    Maven,
    
    /// Custom repository
    Custom(String),
}

/// CDN providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CdnProvider {
    CloudFront,
    CloudFlare,
    KeyCDN,
    MaxCDN,
    Fastly,
    Azure,
    Google,
    Custom(String),
}

/// Release stages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReleaseStage {
    /// Development releases
    Development,
    
    /// Alpha releases
    Alpha,
    
    /// Beta releases
    Beta,
    
    /// Release candidate
    ReleaseCandidate,
    
    /// Stable release
    Stable,
    
    /// Long-term support
    LongTermSupport,
    
    /// Security update
    SecurityUpdate,
    
    /// Hotfix
    Hotfix,
}


/// Authentication types
#[derive(Debug)]
pub enum AuthenticationType {
    /// API token
    ApiToken,
    
    /// OAuth2
    OAuth2,
    
    /// Basic authentication
    BasicAuth,
    
    /// SSH key
    SshKey,
    
    /// AWS credentials
    AwsCredentials,
    
    /// Google Cloud credentials
    GcpCredentials,
    
    /// Azure credentials
    AzureCredentials,
    
    /// Custom authentication
    Custom(String),
}

/// Channel status
#[derive(Debug, Clone)]
pub enum ChannelStatus {
    /// Channel is active
    Active,
    
    /// Channel is inactive
    Inactive,
    
    /// Channel has errors
    Error(String),
    
    /// Channel is under maintenance
    Maintenance,
    
    /// Channel is rate limited
    RateLimited {
        reset_at: std::time::SystemTime,
    },
}

/// Channel statistics
#[derive(Debug, Clone)]
pub struct ChannelStats {
    /// Total uploads
    pub total_uploads: u64,
    
    /// Successful uploads
    pub successful_uploads: u64,
    
    /// Failed uploads
    pub failed_uploads: u64,
    
    /// Total bytes uploaded
    pub bytes_uploaded: u64,
    
    /// Average upload time
    pub avg_upload_time: std::time::Duration,
    
    /// Last upload time
    pub last_upload: Option<std::time::SystemTime>,
    
    /// Upload rate (bytes per second)
    pub upload_rate: f64,
}

/// Upload configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    /// Maximum file size
    pub max_file_size: u64,
    
    /// Chunk size for multipart uploads
    pub chunk_size: u64,
    
    /// Concurrent uploads
    pub max_concurrent: usize,
    
    /// Upload timeout
    pub timeout: u64,
    
    /// Resume failed uploads
    pub resume_uploads: bool,
    
    /// Verify uploads
    pub verify_uploads: bool,
    
    /// Compression
    pub compression: bool,
    
    /// Upload metadata
    pub include_metadata: bool,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Validate checksums
    pub validate_checksums: bool,
    
    /// Validate signatures
    pub validate_signatures: bool,
    
    /// Scan for viruses
    pub virus_scan: bool,
    
    /// Check file format
    pub format_validation: bool,
    
    /// Custom validation scripts
    pub custom_validators: Vec<String>,
    
    /// Validation timeout
    pub timeout: u64,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    
    /// Retry delay (seconds)
    pub delay: u64,
    
    /// Backoff strategy
    pub backoff: BackoffStrategy,
    
    /// Retry conditions
    pub retry_on: Vec<RetryCondition>,
}

/// Backoff strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed,
    Linear,
    Exponential,
    Jittered,
}

/// Retry conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCondition {
    NetworkError,
    Timeout,
    ServerError,
    RateLimit,
    AuthenticationError,
    Custom(String),
}

/// CDN manager
#[derive(Debug)]
pub struct CdnManager {
    /// CDN configuration
    pub config: CdnConfig,
    
    /// CDN provider
    pub provider: CdnProvider,
    
    /// Active distributions
    pub distributions: Vec<CdnDistribution>,
    
    /// CDN metrics
    pub metrics: CdnMetrics,
}





/// CDN configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnConfig {
    /// CDN provider settings
    pub provider_config: HashMap<String, String>,
    
    /// Custom domains
    pub custom_domains: Vec<String>,
    
    /// SSL/TLS configuration
    pub ssl_config: SslConfig,
    
    /// Geographic restrictions
    pub geo_restrictions: Option<GeoRestrictions>,
    
    /// Access logging
    pub access_logging: bool,
    
    /// Real-time logs
    pub real_time_logs: bool,
}

/// CDN distribution
#[derive(Debug, Clone)]
pub struct CdnDistribution {
    /// Distribution ID
    pub id: String,
    
    /// Distribution domain
    pub domain: String,
    
    /// Origin configuration
    pub origin: OriginConfig,
    
    /// Cache behaviors
    pub behaviors: Vec<CacheBehavior>,
    
    /// Distribution status
    pub status: DistributionStatus,
}

/// Origin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginConfig {
    /// Origin domain
    pub domain: String,
    
    /// Origin path
    pub path: String,
    
    /// Origin protocol
    pub protocol: OriginProtocol,
    
    /// Custom headers
    pub custom_headers: HashMap<String, String>,
    
    /// Connection timeout
    pub connect_timeout: u64,
    
    /// Read timeout
    pub read_timeout: u64,
}

/// Origin protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OriginProtocol {
    Http,
    Https,
    MatchViewer,
}

/// Cache behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheBehavior {
    /// Path pattern
    pub path_pattern: String,
    
    /// Cache policy
    pub cache_policy: CachePolicy,
    
    /// Compression
    pub compression: bool,
    
    /// Viewer protocol policy
    pub viewer_protocol: ViewerProtocolPolicy,
    
    /// Allowed HTTP methods
    pub allowed_methods: Vec<HttpMethod>,
}

/// Cache policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CachePolicy {
    /// Cache everything
    CacheAll {
        ttl: u64,
    },
    
    /// Cache based on headers
    CacheByHeaders {
        headers: Vec<String>,
        ttl: u64,
    },
    
    /// No caching
    NoCache,
    
    /// Custom policy
    Custom {
        rules: HashMap<String, String>,
    },
}

/// Viewer protocol policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViewerProtocolPolicy {
    AllowAll,
    RedirectToHttps,
    HttpsOnly,
}

/// HTTP methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Head,
    Options,
    Put,
    Post,
    Patch,
    Delete,
}

/// Distribution status
#[derive(Debug, Clone)]
pub enum DistributionStatus {
    InProgress,
    Deployed,
    Completed,
    Error(String),
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Default TTL
    pub default_ttl: u64,
    
    /// Maximum TTL
    pub max_ttl: u64,
    
    /// Cache key policy
    pub cache_key_policy: CacheKeyPolicy,
    
    /// Compression settings
    pub compression: CompressionSettings,
    
    /// Invalidation settings
    pub invalidation: InvalidationConfig,
}

/// Cache key policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheKeyPolicy {
    /// Include all query parameters
    IncludeAll,
    
    /// Include specific parameters
    IncludeSpecific(Vec<String>),
    
    /// Exclude specific parameters
    ExcludeSpecific(Vec<String>),
    
    /// No query parameters
    None,
}

/// Compression settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionSettings {
    /// Enable compression
    pub enabled: bool,
    
    /// Compression algorithms
    pub algorithms: Vec<CompressionAlgorithm>,
    
    /// Minimum file size
    pub min_size: u64,
    
    /// File types to compress
    pub file_types: Vec<String>,
}

/// Compression algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Brotli,
    Deflate,
}

/// Invalidation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationConfig {
    /// Auto-invalidate on deployment
    pub auto_invalidate: bool,
    
    /// Invalidation patterns
    pub patterns: Vec<String>,
    
    /// Maximum invalidations per hour
    pub max_per_hour: u32,
}

/// SSL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    /// SSL certificate source
    pub certificate_source: CertificateSource,
    
    /// Minimum TLS version
    pub min_tls_version: TlsVersion,
    
    /// Security policy
    pub security_policy: SecurityPolicy,
    
    /// HSTS settings
    pub hsts: Option<HstsConfig>,
}

/// Certificate sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateSource {
    /// Provider-managed certificate
    ProviderManaged,
    
    /// Custom certificate
    Custom {
        certificate: PathBuf,
        private_key: PathBuf,
        chain: Option<PathBuf>,
    },
    
    /// Let's Encrypt
    LetsEncrypt,
}

/// TLS versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TlsVersion {
    Tls10,
    Tls11,
    Tls12,
    Tls13,
}

/// Security policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityPolicy {
    Modern,
    Intermediate,
    Legacy,
    Custom(HashMap<String, String>),
}

/// HSTS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HstsConfig {
    /// Max age in seconds
    pub max_age: u64,
    
    /// Include subdomains
    pub include_subdomains: bool,
    
    /// Preload
    pub preload: bool,
}

/// Geographic restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoRestrictions {
    /// Restriction type
    pub restriction_type: GeoRestrictionType,
    
    /// Country codes
    pub locations: Vec<String>,
}

/// Geographic restriction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeoRestrictionType {
    Allowlist,
    Blocklist,
    None,
}

/// CDN metrics
#[derive(Debug, Clone)]
pub struct CdnMetrics {
    /// Request statistics
    pub requests: RequestStats,
    
    /// Bandwidth statistics
    pub bandwidth: BandwidthStats,
    
    /// Cache statistics
    pub cache: CacheStats,
    
    /// Error statistics
    pub errors: ErrorStats,
    
    /// Performance metrics
    pub performance: PerformanceStats,
}

/// Request statistics
#[derive(Debug, Clone)]
pub struct RequestStats {
    /// Total requests
    pub total: u64,
    
    /// Requests per second
    pub rps: f64,
    
    /// Requests by status code
    pub by_status: HashMap<u16, u64>,
    
    /// Requests by country
    pub by_country: HashMap<String, u64>,
}

/// Bandwidth statistics
#[derive(Debug, Clone)]
pub struct BandwidthStats {
    /// Total bandwidth
    pub total_bytes: u64,
    
    /// Bandwidth per second
    pub bytes_per_second: f64,
    
    /// Peak bandwidth
    pub peak_bandwidth: f64,
    
    /// Bandwidth by country
    pub by_country: HashMap<String, u64>,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Cache hit ratio
    pub hit_ratio: f64,
    
    /// Cache hits
    pub hits: u64,
    
    /// Cache misses
    pub misses: u64,
    
    /// Edge hits
    pub edge_hits: u64,
    
    /// Origin hits
    pub origin_hits: u64,
}

/// Error statistics
#[derive(Debug, Clone)]
pub struct ErrorStats {
    /// Total errors
    pub total: u64,
    
    /// Error rate
    pub error_rate: f64,
    
    /// Errors by type
    pub by_type: HashMap<u16, u64>,
    
    /// 4xx errors
    pub client_errors: u64,
    
    /// 5xx errors
    pub server_errors: u64,
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    /// Average response time
    pub avg_response_time: f64,
    
    /// 95th percentile response time
    pub p95_response_time: f64,
    
    /// 99th percentile response time
    pub p99_response_time: f64,
    
    /// Time to first byte
    pub ttfb: f64,
}

/// Release metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseMetadata {
    /// Release version
    pub version: String,
    
    /// Release name
    pub name: String,
    
    /// Release description
    pub description: String,
    
    /// Release notes
    pub notes: String,
    
    /// Release timestamp
    pub timestamp: String,
    
    /// Git tag
    pub tag: String,
    
    /// Git commit hash
    pub commit: String,
    
    /// Release assets
    pub assets: Vec<ReleaseAsset>,
    
    /// Release stage
    pub stage: ReleaseStage,
    
    /// Release channels
    pub channels: Vec<String>,
}

/// Release asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    /// Asset name
    pub name: String,
    
    /// Asset URL
    pub url: String,
    
    /// Asset size
    pub size: u64,
    
    /// Asset content type
    pub content_type: String,
    
    /// Asset checksum
    pub checksum: String,
    
    /// Download count
    pub download_count: u64,
}

/// Distribution record
#[derive(Debug, Clone)]
pub struct DistributionRecord {
    /// Distribution timestamp
    pub timestamp: std::time::SystemTime,
    
    /// Release version
    pub version: String,
    
    /// Distribution channel
    pub channel: String,
    
    /// Distribution status
    pub status: DistributionStatus,
    
    /// Assets distributed
    pub assets: Vec<String>,
    
    /// Distribution metrics
    pub metrics: DistributionMetrics,
    
    /// Error details
    pub error: Option<String>,
}


/// Distribution metrics
#[derive(Debug, Clone)]
pub struct DistributionMetrics {
    /// Start time
    pub start_time: std::time::SystemTime,
    
    /// End time
    pub end_time: Option<std::time::SystemTime>,
    
    /// Duration
    pub duration: Option<std::time::Duration>,
    
    /// Total bytes transferred
    pub bytes_transferred: u64,
    
    /// Transfer rate
    pub transfer_rate: f64,
    
    /// Retry count
    pub retry_count: u32,
}

impl DistributionManager {
    /// Create a new distribution manager
    pub fn new(config: DistributionConfig) -> Result<Self, SemanticError> {
        let channels = Vec::new();
        let cdn = None;
        
        let metadata = DistributionMetadata {
            version: "1.0.0".to_string(),
            assets: Vec::new(),
        };
        
        Ok(Self {
            channels,
            cdn_manager: cdn,
            metadata,
            history: Vec::new(),
        })
    }
    
    /// Add a distribution channel
    pub fn add_channel(&mut self, channel_config: ChannelConfig, channel_type: ChannelType) -> Result<(), SemanticError> {
        let channel = DistributionChannel {
            name: channel_config.name.clone(),
            channel_type,
            url: channel_config.endpoint.clone(),
            stats: ChannelStats::default(),
            status: ChannelStatus::Active,
            options: HashMap::new(),
            enabled: true,
            config: channel_config,
            credentials: None,
        };
        
        self.channels.push(channel);
        Ok(())
    }
    
    /// Distribute packages to all channels
    pub fn distribute_packages(&mut self, packages: &[PackageInfo]) -> Result<Vec<DistributionRecord>, SemanticError> {
        let mut records = Vec::new();
        
        // Collect active channel indices
        let active_channels: Vec<usize> = self.channels.iter()
            .enumerate()
            .filter(|(_, ch)| matches!(ch.status, ChannelStatus::Active))
            .map(|(i, _)| i)
            .collect();
        
        // Distribute to each active channel
        for channel_idx in active_channels {
            let record = self.distribute_to_channel_by_index(channel_idx, packages)?;
            records.push(record);
        }
        
        // Update release metadata with assets
        self.update_release_metadata(packages)?;
        
        self.history.extend(records.clone());
        Ok(records)
    }
    
    /// Distribute packages to a channel by index
    fn distribute_to_channel_by_index(&mut self, channel_idx: usize, packages: &[PackageInfo]) -> Result<DistributionRecord, SemanticError> {
        // Extract necessary data to avoid borrow conflicts
        let channel_name = self.channels[channel_idx].config.name.clone();
        let _channel_type = self.channels[channel_idx].channel_type.clone();
        
        let start_time = std::time::SystemTime::now();
        
        println!("Distributing to channel: {}", channel_name);
        
        // In a real implementation, would perform actual distribution
        let record = DistributionRecord {
            timestamp: start_time,
            version: "1.0.0".to_string(), // TODO: Get actual version
            channel: channel_name,
            status: DistributionStatus::Completed,
            assets: packages.iter().map(|p| p.name.clone()).collect(),
            metrics: DistributionMetrics {
                start_time,
                end_time: Some(start_time + std::time::Duration::from_secs(1)),
                duration: Some(std::time::Duration::from_secs(1)),
                bytes_transferred: packages.iter().map(|p| p.size).sum(),
                transfer_rate: 1024.0 * 1024.0, // 1 MB/s
                retry_count: 0,
            },
            error: None,
        };
        
        // Update channel stats
        self.channels[channel_idx].stats.total_uploads += 1;
        self.channels[channel_idx].stats.successful_uploads += 1;
        self.channels[channel_idx].stats.bytes_uploaded += record.metrics.bytes_transferred;
        self.channels[channel_idx].stats.last_upload = Some(start_time);
        
        Ok(record)
    }
    
    /// Distribute packages to a specific channel
    
    
    /// Update release metadata with package information
    fn update_release_metadata(&mut self, packages: &[PackageInfo]) -> Result<(), SemanticError> {
        self.metadata.assets.clear();
        
        for package in packages {
            let asset = DistributionAsset {
                name: package.name.clone(),
                path: PathBuf::from(&package.name),
                size: package.size,
                checksum: package.checksum.clone(),
            };
            
            self.metadata.assets.push(asset);
        }
        
        Ok(())
    }
    
    /// Get content type for package format
    fn get_content_type(&self, format: &PackageFormat) -> String {
        match format {
            PackageFormat::Zip => "application/zip".to_string(),
            PackageFormat::TarGz => "application/gzip".to_string(),
            PackageFormat::TarXz => "application/x-xz".to_string(),
            PackageFormat::Deb => "application/vnd.debian.binary-package".to_string(),
            PackageFormat::Rpm => "application/x-rpm".to_string(),
            PackageFormat::Msi => "application/x-msi".to_string(),
            PackageFormat::Dmg => "application/x-apple-diskimage".to_string(),
            PackageFormat::AppImage => "application/x-executable".to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }
    
    /// Get distribution statistics
    pub fn get_distribution_stats(&self) -> DistributionStats {
        let total_distributions = self.history.len();
        let successful_distributions = self.history.iter()
            .filter(|r| matches!(r.status, DistributionStatus::Completed))
            .count();
        let failed_distributions = self.history.iter()
            .filter(|r| matches!(r.status, DistributionStatus::Error(_)))
            .count();
        
        let total_bytes = self.history.iter()
            .map(|r| r.metrics.bytes_transferred)
            .sum();
        
        let avg_duration = if !self.history.is_empty() {
            let total_duration: std::time::Duration = self.history.iter()
                .filter_map(|r| r.metrics.duration)
                .sum();
            total_duration / self.history.len() as u32
        } else {
            std::time::Duration::from_secs(0)
        };
        
        DistributionStats {
            total_distributions,
            successful_distributions,
            failed_distributions,
            total_bytes,
            avg_duration,
            channel_stats: self.channels.iter().map(|c| c.stats.clone()).collect(),
        }
    }
    
    /// Setup CDN distribution
    pub fn setup_cdn(&mut self, cdn_config: CdnConfig) -> Result<(), SemanticError> {
        println!("Setting up CDN distribution");
        
        let provider = CdnProvider::CloudFront; // Example provider
        let distributions = vec![]; // Would create distributions based on config
        let cache_config = CacheConfig {
            default_ttl: 3600,
            max_ttl: 86400,
            cache_key_policy: CacheKeyPolicy::IncludeAll,
            compression: CompressionSettings {
                enabled: true,
                algorithms: vec![CompressionAlgorithm::Gzip, CompressionAlgorithm::Brotli],
                min_size: 1024,
                file_types: vec!["text/*".to_string(), "application/javascript".to_string()],
            },
            invalidation: InvalidationConfig {
                auto_invalidate: true,
                patterns: vec!["/*".to_string()],
                max_per_hour: 100,
            },
        };
        
        let metrics = CdnMetrics {
            requests: RequestStats {
                total: 0,
                rps: 0.0,
                by_status: HashMap::new(),
                by_country: HashMap::new(),
            },
            bandwidth: BandwidthStats {
                total_bytes: 0,
                bytes_per_second: 0.0,
                peak_bandwidth: 0.0,
                by_country: HashMap::new(),
            },
            cache: CacheStats {
                hit_ratio: 0.0,
                hits: 0,
                misses: 0,
                edge_hits: 0,
                origin_hits: 0,
            },
            errors: ErrorStats {
                total: 0,
                error_rate: 0.0,
                by_type: HashMap::new(),
                client_errors: 0,
                server_errors: 0,
            },
            performance: PerformanceStats {
                avg_response_time: 0.0,
                p95_response_time: 0.0,
                p99_response_time: 0.0,
                ttfb: 0.0,
            },
        };
        
        self.cdn_manager = Some(CdnManager {
            config: cdn_config,
            provider,
            distributions,
            metrics,
        });
        
        Ok(())
    }
    
    /// Distribute artifacts through configured channels
    pub fn distribute_artifacts(&mut self, artifacts: &[ArtifactInfo]) -> Result<(), SemanticError> {
        println!("Distributing {} artifacts", artifacts.len());
        
        for channel in &mut self.channels {
            match &channel.channel_type {
                ChannelType::GitHub { owner, repo } => {
                    println!("Distributing to GitHub: {}/{}", owner, repo);
                    // In a real implementation, would upload to GitHub releases
                }
                ChannelType::GitLab { project_id } => {
                    println!("Distributing to GitLab project: {}", project_id);
                    // In a real implementation, would upload to GitLab releases
                }
                ChannelType::PackageRepository { repository_type, repository_url } => {
                    println!("Distributing to {:?} repository: {}", repository_type, repository_url);
                    // In a real implementation, would upload to package repository
                }
                _ => {
                    println!("Distributing to custom channel: {}", channel.config.name);
                }
            }
        }
        
        Ok(())
    }
}

/// Distribution statistics
#[derive(Debug, Clone)]
pub struct DistributionStats {
    /// Total number of distributions
    pub total_distributions: usize,
    
    /// Successful distributions
    pub successful_distributions: usize,
    
    /// Failed distributions
    pub failed_distributions: usize,
    
    /// Total bytes distributed
    pub total_bytes: u64,
    
    /// Average distribution duration
    pub avg_duration: std::time::Duration,
    
    /// Channel statistics
    pub channel_stats: Vec<ChannelStats>,
}

impl Default for ChannelStats {
    fn default() -> Self {
        Self {
            total_uploads: 0,
            successful_uploads: 0,
            failed_uploads: 0,
            bytes_uploaded: 0,
            avg_upload_time: std::time::Duration::from_secs(0),
            last_upload: None,
            upload_rate: 0.0,
        }
    }
}

impl Default for UploadConfig {
    fn default() -> Self {
        Self {
            max_file_size: 2 * 1024 * 1024 * 1024, // 2GB
            chunk_size: 64 * 1024 * 1024,          // 64MB
            max_concurrent: 4,
            timeout: 300,                           // 5 minutes
            resume_uploads: true,
            verify_uploads: true,
            compression: false,
            include_metadata: true,
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            validate_checksums: true,
            validate_signatures: true,
            virus_scan: false,
            format_validation: true,
            custom_validators: Vec::new(),
            timeout: 60,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay: 5,
            backoff: BackoffStrategy::Exponential,
            retry_on: vec![
                RetryCondition::NetworkError,
                RetryCondition::Timeout,
                RetryCondition::ServerError,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::release::packaging::*;
    use std::path::PathBuf;
    use crate::release::{
        ReleaseNotesConfig, ReleaseNotesFormat, ReleaseNotesSection,
        AnnouncementConfig, AudienceConfig, OptOutConfig,
        MirrorConfig, SyncConfig, SyncMethod, SyncFrequency, LoadBalancingConfig,
        LoadBalancingStrategy, HealthCheckConfig, ChannelVisibility,
        ChannelType, AuthConfig, ChannelMetadata,
        ChangelogConfig, ChangelogFormat, CommitParsingConfig,
        SchedulingConfig
    };
    // Import conflicting types with aliases to avoid conflicts
    use crate::release::{RetryConfig as ReleaseRetryConfig, UploadConfig as ReleaseUploadConfig};
    
    fn create_test_distribution_config() -> DistributionConfig {
        DistributionConfig {
            channels: vec![],
            release_notes: ReleaseNotesConfig {
                template: PathBuf::from("release-notes.md"),
                format: ReleaseNotesFormat::Markdown,
                sections: vec![
                    ReleaseNotesSection::Summary,
                    ReleaseNotesSection::NewFeatures,
                    ReleaseNotesSection::BugFixes,
                ],
                changelog: ChangelogConfig {
                    file: PathBuf::from("CHANGELOG.md"),
                    format: ChangelogFormat::KeepAChangelog,
                    auto_generate: true,
                    commit_parsing: CommitParsingConfig {
                        pattern: r"^(\w+)(?:\(([^)]+)\))?: (.+)$".to_string(),
                        type_mapping: HashMap::new(),
                        breaking_change_patterns: vec!["BREAKING CHANGE:".to_string()],
                    },
                },
            },
            announcements: AnnouncementConfig {
                channels: vec![],
                templates: {
                    let mut templates = HashMap::new();
                    templates.insert("default".to_string(), "New release: {{version}}".to_string());
                    templates
                },
                scheduling: SchedulingConfig {
                    immediate: true,
                    delay: None,
                    schedule: vec![],
                },
            },
            mirrors: MirrorConfig {
                mirrors: vec![],
                sync: SyncConfig {
                    method: SyncMethod::Push,
                    frequency: SyncFrequency::Immediate,
                    retry_policy: ReleaseRetryConfig {
                        max_attempts: 3,
                        initial_delay: 1,
                        max_delay: 60,
                        backoff_factor: 2.0,
                    },
                },
                load_balancing: LoadBalancingConfig {
                    strategy: LoadBalancingStrategy::RoundRobin,
                    health_monitoring: true,
                    failover: FailoverConfig {
                        enabled: true,
                        threshold: FailoverThreshold {
                            error_rate: 0.5,
                            response_time: 5000,
                            availability: 0.95,
                        },
                        recovery: RecoveryConfig {
                            check_interval: 30,
                            recovery_threshold: 3,
                            gradual_recovery: true,
                        },
                    },
                },
            },
        }
    }
    
    #[test]
    fn test_distribution_manager_creation() {
        let config = create_test_distribution_config();
        
        let manager = DistributionManager::new(config).unwrap();
        assert_eq!(manager.channels.len(), 0);
        assert_eq!(manager.metadata.version, "1.0.0");
    }
    
    #[test]
    fn test_channel_addition() {
        let config = create_test_distribution_config();
        
        let mut manager = DistributionManager::new(config).unwrap();
        
        let channel_config = ChannelConfig {
            name: "github".to_string(),
            description: "GitHub Releases".to_string(),
            endpoint: "https://api.github.com".to_string(),
            supported_formats: vec![PackageFormat::Zip, PackageFormat::TarGz],
            platforms: vec!["linux".to_string(), "darwin".to_string()],
            stages: vec![ReleaseStage::Stable],
            upload: UploadConfig {
                max_file_size: 100 * 1024 * 1024,
                chunk_size: 5 * 1024 * 1024,
                max_concurrent: 4,
                timeout: 300,
                resume_uploads: true,
                verify_uploads: true,
                compression: true,
                include_metadata: true,
            },
            validation: ValidationConfig::default(),
            retry: RetryConfig {
                max_attempts: 3,
                delay: 1,
                backoff: BackoffStrategy::Exponential,
                retry_on: vec![RetryCondition::NetworkError],
            },
        };
        
        // Use the local ChannelType which has GitHub variant with fields
        // Note: Using the local distribution::ChannelType, not the imported one
        let channel_type = crate::release::distribution::ChannelType::GitHub {
            owner: "aetherscript".to_string(),
            repo: "aetherscript".to_string(),
        };
        
        manager.add_channel(channel_config, channel_type).unwrap();
        assert_eq!(manager.channels.len(), 1);
    }
    
    #[test]
    fn test_content_type_detection() {
        let config = create_test_distribution_config();
        
        let manager = DistributionManager::new(config).unwrap();
        
        assert_eq!(manager.get_content_type(&PackageFormat::Zip), "application/zip");
        assert_eq!(manager.get_content_type(&PackageFormat::TarGz), "application/gzip");
        assert_eq!(manager.get_content_type(&PackageFormat::Deb), "application/vnd.debian.binary-package");
    }
    
    #[test]
    fn test_release_metadata_update() {
        let config = create_test_distribution_config();
        
        let mut manager = DistributionManager::new(config).unwrap();
        
        let packages = vec![
            PackageInfo {
                name: "test-package.zip".to_string(),
                path: PathBuf::from("test-package.zip"),
                format: PackageFormat::Zip,
                platform: PlatformTarget {
                    os: "linux".to_string(),
                    arch: "x86_64".to_string(),
                    variant: None,
                    min_version: None,
                    options: HashMap::new(),
                },
                size: 1024,
                checksum: "abcd1234".to_string(),
                signature: None,
                created_at: std::time::SystemTime::now(),
                metadata: PackageMetadata {
                    name: "test".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Test package".to_string(),
                    maintainer: "Test".to_string(),
                    homepage: "https://test.com".to_string(),
                    license: "MIT".to_string(),
                    dependencies: vec![],
                    categories: vec![],
                    keywords: vec![],
                    installed_size: None,
                    download_size: None,
                    priority: PackagePriority::Standard,
                    custom_fields: HashMap::new(),
                },
            }
        ];
        
        manager.update_release_metadata(&packages).unwrap();
        assert_eq!(manager.metadata.assets.len(), 1);
        assert_eq!(manager.metadata.assets[0].name, "test-package.zip");
    }
}