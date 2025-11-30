use super::*;
use crate::package::manifest::{BuildConfiguration, Edition, PackageMetadata};
use crate::package::version::Version;

#[test]
fn test_package_builder_creation() {
    let config = BuildConfig::default();
    let builder = PackageBuilder::new(config);

    // Builder created successfully with custom config
}

#[test]
fn test_build_stages_creation() {
    let manifest = create_test_manifest();
    let builder = PackageBuilder::new(BuildConfig::default());
    let stages = builder.create_build_stages(&manifest);

    assert!(stages.iter().any(|s| s.name == "pre-build"));
    assert!(stages.iter().any(|s| s.name == "documentation"));
}

#[test]
fn test_build_artifact_types() {
    let executable = ArtifactType::Executable;
    assert!(matches!(executable, ArtifactType::Executable));

    let library = ArtifactType::StaticLibrary;
    assert!(matches!(library, ArtifactType::StaticLibrary));
}

#[test]
fn test_build_stage_status() {
    let pending = BuildStageStatus::Pending;
    assert!(matches!(pending, BuildStageStatus::Pending));

    let failed = BuildStageStatus::Failed("test error".to_string());
    assert!(matches!(failed, BuildStageStatus::Failed(_)));
}

#[test]
fn test_script_interpreter_types() {
    let aether = ScriptInterpreter::AetherScript;
    assert!(matches!(aether, ScriptInterpreter::AetherScript));

    let shell = ScriptInterpreter::Shell("/bin/bash".to_string());
    assert!(matches!(shell, ScriptInterpreter::Shell(_)));
}

fn create_test_manifest() -> PackageManifest {
    use std::collections::HashMap;

    PackageManifest {
        package: PackageMetadata {
            name: "test-package".to_string(),
            version: Version::new(1, 0, 0),
            description: Some("Test package".to_string()),
            authors: vec!["Test Author".to_string()],
            license: Some("MIT".to_string()),
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
            metadata: crate::package::manifest::TomlValue::Table(
                std::collections::HashMap::new(),
            ),
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
    }
}
