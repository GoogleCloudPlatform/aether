use super::*;

#[test]
fn test_doc_config_default() {
    let config = DocConfig::default();
    assert_eq!(config.project_name, "AetherScript");
    assert!(config.generate_examples);
    assert!(config.generate_tutorials);
}

#[test]
fn test_documentation_structure() {
    let config = DocConfig::default();
    let generator = DocumentationGenerator::new(config).unwrap();

    assert_eq!(generator.documentation.metadata.name, "AetherScript");
    assert!(generator.documentation.api.modules.is_empty());
}

#[test]
fn test_search_config() {
    let search_config = SearchConfig::default();
    assert!(search_config.enabled);
    assert_eq!(search_config.max_results, 50);
    assert!(matches!(
        search_config.index_type,
        SearchIndexType::ClientSide
    ));
}

#[test]
fn test_output_formats() {
    let html_format = OutputFormat::Html {
        javascript: true,
        theme: "dark".to_string(),
        search: true,
    };

    assert!(matches!(html_format, OutputFormat::Html { .. }));
}

#[test]
fn test_tutorial_creation() {
    let config = DocConfig::default();
    let generator = DocumentationGenerator::new(config).unwrap();

    let tutorial = generator.create_language_tutorial().unwrap();
    assert_eq!(tutorial.title, "AetherScript Language Tutorial");
    assert!(matches!(tutorial.difficulty, DifficultyLevel::Beginner));
}
