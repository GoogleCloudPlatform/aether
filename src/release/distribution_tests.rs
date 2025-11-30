use super::*;
use crate::release::packaging::*;
use crate::release::{
    AnnouncementConfig, AudienceConfig, AuthConfig, ChangelogConfig, ChangelogFormat,
    ChannelMetadata, ChannelType, ChannelVisibility, CommitParsingConfig, FailoverConfig,
    FailoverThreshold, HealthCheckConfig, LoadBalancingConfig, LoadBalancingStrategy,
    MirrorConfig, OptOutConfig, RecoveryConfig, ReleaseNotesConfig, ReleaseNotesFormat,
    ReleaseNotesSection, SchedulingConfig, SyncConfig, SyncFrequency, SyncMethod,
};
use std::path::PathBuf;
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
                templates.insert(
                    "default".to_string(),
                    "New release: {{version}}".to_string(),
                );
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

    assert_eq!(
        manager.get_content_type(&PackageFormat::Zip),
        "application/zip"
    );
    assert_eq!(
        manager.get_content_type(&PackageFormat::TarGz),
        "application/gzip"
    );
    assert_eq!(
        manager.get_content_type(&PackageFormat::Deb),
        "application/vnd.debian.binary-package"
    );
}

#[test]
fn test_release_metadata_update() {
    let config = create_test_distribution_config();

    let mut manager = DistributionManager::new(config).unwrap();

    let packages = vec![PackageInfo {
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
    }];

    manager.update_release_metadata(&packages).unwrap();
    assert_eq!(manager.metadata.assets.len(), 1);
    assert_eq!(manager.metadata.assets[0].name, "test-package.zip");
}
