use super::*;

#[test]
fn test_package_builder_creation() {
    let config = PackageBuilderConfig {
        build_dir: PathBuf::from("target/build"),
        output_dir: PathBuf::from("target/packages"),
        temp_dir: PathBuf::from("target/temp"),
        signing: None,
        verification: VerificationConfig::default(),
        formats: vec![],
        archive: ArchiveConfig::default(),
        installer: InstallerConfig {
            installer_type: InstallerType::ShellScript,
            install_dir: "/usr/local".to_string(),
            desktop_shortcuts: false,
            start_menu: false,
            add_to_path: true,
            license_agreement: None,
            install_scripts: vec![],
            uninstall_scripts: vec![],
            branding: InstallerBranding {
                company: "AetherScript".to_string(),
                product: "AetherScript Compiler".to_string(),
                logo: None,
                banner: None,
                icon: None,
                colors: HashMap::new(),
            },
        },
    };

    // Create directories for test
    std::fs::create_dir_all(&config.build_dir).unwrap();
    std::fs::create_dir_all(&config.output_dir).unwrap();
    std::fs::create_dir_all(&config.temp_dir).unwrap();

    let builder = PackageBuilder::new(config).unwrap();
    assert_eq!(builder.metadata.name, "aetherscript");
}

#[test]
fn test_package_name_generation() {
    let config = PackageBuilderConfig {
        build_dir: PathBuf::from("target/build"),
        output_dir: PathBuf::from("target/packages"),
        temp_dir: PathBuf::from("target/temp"),
        signing: None,
        verification: VerificationConfig::default(),
        formats: vec![],
        archive: ArchiveConfig::default(),
        installer: InstallerConfig {
            installer_type: InstallerType::ShellScript,
            install_dir: "/usr/local".to_string(),
            desktop_shortcuts: false,
            start_menu: false,
            add_to_path: true,
            license_agreement: None,
            install_scripts: vec![],
            uninstall_scripts: vec![],
            branding: InstallerBranding {
                company: "AetherScript".to_string(),
                product: "AetherScript Compiler".to_string(),
                logo: None,
                banner: None,
                icon: None,
                colors: HashMap::new(),
            },
        },
    };

    std::fs::create_dir_all(&config.build_dir).unwrap();
    std::fs::create_dir_all(&config.output_dir).unwrap();
    std::fs::create_dir_all(&config.temp_dir).unwrap();

    let builder = PackageBuilder::new(config).unwrap();

    let platform = PlatformTarget {
        os: "linux".to_string(),
        arch: "x86_64".to_string(),
        variant: None,
        min_version: None,
        options: HashMap::new(),
    };

    let name = builder.generate_package_name(&PackageFormat::Zip, &platform);
    assert_eq!(name, "aetherscript-1.0.0-linux-x86_64.zip");
}

#[test]
fn test_checksum_calculation() {
    let config = PackageBuilderConfig {
        build_dir: PathBuf::from("target/build"),
        output_dir: PathBuf::from("target/packages"),
        temp_dir: PathBuf::from("target/temp"),
        signing: None,
        verification: VerificationConfig::default(),
        formats: vec![],
        archive: ArchiveConfig::default(),
        installer: InstallerConfig {
            installer_type: InstallerType::ShellScript,
            install_dir: "/usr/local".to_string(),
            desktop_shortcuts: false,
            start_menu: false,
            add_to_path: true,
            license_agreement: None,
            install_scripts: vec![],
            uninstall_scripts: vec![],
            branding: InstallerBranding {
                company: "AetherScript".to_string(),
                product: "AetherScript Compiler".to_string(),
                logo: None,
                banner: None,
                icon: None,
                colors: HashMap::new(),
            },
        },
    };

    std::fs::create_dir_all(&config.build_dir).unwrap();
    std::fs::create_dir_all(&config.output_dir).unwrap();
    std::fs::create_dir_all(&config.temp_dir).unwrap();

    let builder = PackageBuilder::new(config).unwrap();

    // Create a test file
    let test_file = PathBuf::from("target/test_file.txt");
    std::fs::write(&test_file, b"test content").unwrap();

    let checksum = builder
        .calculate_checksum(&test_file, ChecksumAlgorithm::Sha256)
        .unwrap();
    assert!(!checksum.is_empty());

    // Clean up
    std::fs::remove_file(test_file).unwrap();
}

#[test]
fn test_deb_control_generation() {
    let config = PackageBuilderConfig {
        build_dir: PathBuf::from("target/build"),
        output_dir: PathBuf::from("target/packages"),
        temp_dir: PathBuf::from("target/temp"),
        signing: None,
        verification: VerificationConfig::default(),
        formats: vec![],
        archive: ArchiveConfig::default(),
        installer: InstallerConfig {
            installer_type: InstallerType::ShellScript,
            install_dir: "/usr/local".to_string(),
            desktop_shortcuts: false,
            start_menu: false,
            add_to_path: true,
            license_agreement: None,
            install_scripts: vec![],
            uninstall_scripts: vec![],
            branding: InstallerBranding {
                company: "AetherScript".to_string(),
                product: "AetherScript Compiler".to_string(),
                logo: None,
                banner: None,
                icon: None,
                colors: HashMap::new(),
            },
        },
    };

    std::fs::create_dir_all(&config.build_dir).unwrap();
    std::fs::create_dir_all(&config.output_dir).unwrap();
    std::fs::create_dir_all(&config.temp_dir).unwrap();

    let builder = PackageBuilder::new(config).unwrap();

    let platform = PlatformTarget {
        os: "linux".to_string(),
        arch: "x86_64".to_string(),
        variant: None,
        min_version: None,
        options: HashMap::new(),
    };

    let control = builder.generate_deb_control(&platform).unwrap();
    assert!(control.contains("Package: aetherscript"));
    assert!(control.contains("Version: 1.0.0"));
    assert!(control.contains("Architecture: amd64"));
}
