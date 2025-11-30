use super::*;

#[test]
fn test_example_manager_creation() {
    let config = DocConfig::default();
    let mut manager = ExampleManager::new(&config).unwrap();

    // Generate examples after creation
    manager.generate_examples().unwrap();

    assert!(!manager.examples.is_empty());
}

#[test]
fn test_example_categories() {
    let categories = ExampleManager::create_example_categories();

    assert_eq!(categories.len(), 6);
    assert!(categories.contains_key("Basic Syntax"));
    assert!(categories.contains_key("Data Structures"));
    assert!(categories.contains_key("Algorithms"));
}

#[test]
fn test_syntax_examples_generation() {
    let config = DocConfig::default();
    let mut manager = ExampleManager::new(&config).unwrap();

    manager.generate_syntax_examples().unwrap();

    let syntax_examples: Vec<_> = manager
        .examples
        .iter()
        .filter(|e| e.category == "Basic Syntax")
        .collect();

    assert!(!syntax_examples.is_empty());
    assert!(syntax_examples.iter().any(|e| e.name == "Hello World"));
}

#[test]
fn test_example_difficulty_levels() {
    let beginner = ExampleDifficulty::Beginner;
    let intermediate = ExampleDifficulty::Intermediate;
    let advanced = ExampleDifficulty::Advanced;
    let expert = ExampleDifficulty::Expert;

    assert!(matches!(beginner, ExampleDifficulty::Beginner));
    assert!(matches!(intermediate, ExampleDifficulty::Intermediate));
    assert!(matches!(advanced, ExampleDifficulty::Advanced));
    assert!(matches!(expert, ExampleDifficulty::Expert));
}
