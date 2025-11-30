use super::*;

#[test]
fn test_valid_package_names() {
    assert!(is_valid_package_name("my-package"));
    assert!(is_valid_package_name("my_package"));
    assert!(is_valid_package_name("package123"));
    assert!(is_valid_package_name("a"));
}

#[test]
fn test_invalid_package_names() {
    assert!(!is_valid_package_name(""));
    assert!(!is_valid_package_name("123package"));
    assert!(!is_valid_package_name("-package"));
    assert!(!is_valid_package_name("package.name"));
    assert!(!is_valid_package_name("package name"));
}

#[test]
fn test_manifest_parsing() {
    let toml_content = r#"
[package]
name = "test-package"
version = "1.0.0"
description = "A test package"
authors = ["Test Author <test@example.com>"]
license = "MIT"
edition = "2024"

[[dependencies]]
name = "serde"
version = "^1.0"

[build]
script = "build.aether"
opt-level = 2
debug = true
    "#;

    let manifest = PackageManifest::from_str(toml_content);
    assert!(manifest.is_ok());

    let manifest = manifest.unwrap();
    assert_eq!(manifest.package.name, "test-package");
    assert_eq!(manifest.package.version.major, 1);
    assert_eq!(manifest.dependencies.len(), 1);
    assert_eq!(manifest.dependencies[0].name, "serde");
}

#[test]
fn test_default_edition() {
    let edition = Edition::default();
    assert_eq!(edition, Edition::Edition2024);
}

#[test]
fn test_build_config_default() {
    let config = BuildConfiguration::default();
    assert!(config.sources.is_empty());
    assert!(config.libs.is_empty());
    assert!(config.env.is_empty());
}

#[test]
fn test_dependency_validation() {
    let manifest = PackageManifest {
        package: PackageMetadata {
            name: "test".to_string(),
            version: Version::new(1, 0, 0),
            description: None,
            authors: vec![],
            license: None,
            license_file: None,
            homepage: None,
            repository: None,
            documentation: None,
            keywords: vec![],
            categories: vec![],
            readme: None,
            include: None,
            exclude: None,
            edition: Edition::Edition2024,
            aether_version: None,
            build: None,
            publish: None,
            metadata: TomlValue::Table(std::collections::HashMap::new()),
        },
        dependencies: vec![],
        dev_dependencies: vec![],
        optional_dependencies: vec![],
        build_dependencies: vec![],
        build: BuildConfiguration::default(),
        features: HashMap::new(),
        target: HashMap::new(),
        workspace: None,
        bin: vec![],
        lib: None,
        example: vec![],
        test: vec![],
        bench: vec![],
    };

    assert!(manifest.validate().is_ok());
}
