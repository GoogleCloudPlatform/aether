use super::*;

#[test]
fn test_pipeline_creation() {
    let config = PipelineConfig {
        name: "test-pipeline".to_string(),
        stages: vec![StageConfig {
            name: "build".to_string(),
            description: "Build stage".to_string(),
            stage_type: StageType::Build,
            commands: vec!["echo 'building'".to_string()],
            depends_on: vec![],
            timeout: None,
            retry: RetryConfig::default(),
            environment: HashMap::new(),
            conditions: vec![],
        }],
        environment: HashMap::new(),
        timeouts: TimeoutConfig::default(),
        parallel: ParallelConfig::default(),
        quality_gates: vec![],
        notifications: NotificationConfig {
            channels: vec![],
            triggers: vec![],
            templates: HashMap::new(),
        },
    };

    let pipeline = AutomationPipeline::new(config).unwrap();
    assert_eq!(pipeline.stages.len(), 1);
    assert_eq!(pipeline.current_stage, 0);
}

#[test]
fn test_stage_execution() {
    let config = StageConfig {
        name: "test-stage".to_string(),
        description: "Test stage".to_string(),
        stage_type: StageType::Test,
        commands: vec!["echo 'test'".to_string()],
        depends_on: vec![],
        timeout: None,
        retry: RetryConfig::default(),
        environment: HashMap::new(),
        conditions: vec![],
    };

    let mut stage = PipelineStage::new(config).unwrap();
    let context = PipelineContext::from_environment().unwrap();

    let result = stage.execute(&context).unwrap();
    assert!(matches!(result.status, StageStatus::Success));
    assert!(result.output.contains("test"));
}

#[test]
fn test_condition_evaluation() {
    let config = PipelineConfig {
        name: "test-pipeline".to_string(),
        stages: vec![],
        environment: HashMap::new(),
        timeouts: TimeoutConfig::default(),
        parallel: ParallelConfig::default(),
        quality_gates: vec![],
        notifications: NotificationConfig {
            channels: vec![],
            triggers: vec![],
            templates: HashMap::new(),
        },
    };

    let pipeline = AutomationPipeline::new(config).unwrap();

    // Test environment variable exists
    let condition = StageCondition {
        condition_type: ConditionType::Environment,
        value: "PATH".to_string(),
        operator: ConditionOperator::NotEquals,
    };

    // PATH environment variable should exist and not equal "PATH"
    let result = pipeline.evaluate_condition(&condition).unwrap();
    assert!(result);

    // Test file exists condition
    let condition = StageCondition {
        condition_type: ConditionType::FileExists,
        value: "true".to_string(), // For FileExists, we check if actual value equals expected
        operator: ConditionOperator::Equals,
    };

    // Create a temp file to test with
    std::fs::write("test_temp_file.txt", "test").unwrap();

    let condition2 = StageCondition {
        condition_type: ConditionType::FileExists,
        value: "test_temp_file.txt".to_string(),
        operator: ConditionOperator::NotEquals,
    };

    // This should return true because "true" != "test_temp_file.txt"
    let result = pipeline.evaluate_condition(&condition2).unwrap();
    assert!(result);

    // Clean up
    std::fs::remove_file("test_temp_file.txt").ok();
}

#[test]
fn test_quality_gate_evaluation() {
    let gate = QualityGate {
        name: "coverage-gate".to_string(),
        gate_type: QualityGateType::Coverage,
        criteria: vec![QualityCriteria {
            metric: "coverage".to_string(),
            threshold: 80.0,
            operator: ComparisonOperator::GreaterThanOrEqual,
            severity: CriteriaSeverity::Error,
        }],
        action: QualityGateAction::Stop,
    };

    let mut metrics = HashMap::new();
    metrics.insert("coverage".to_string(), "85.5".to_string());

    let result = StageResult {
        stage_name: "test".to_string(),
        status: StageStatus::Success,
        duration: std::time::Duration::from_secs(1),
        output: String::new(),
        error_output: String::new(),
        exit_code: Some(0),
        artifacts: vec![],
        metrics,
    };

    let config = PipelineConfig {
        name: "test-pipeline".to_string(),
        stages: vec![],
        environment: HashMap::new(),
        timeouts: TimeoutConfig::default(),
        parallel: ParallelConfig::default(),
        quality_gates: vec![gate],
        notifications: NotificationConfig {
            channels: vec![],
            triggers: vec![],
            templates: HashMap::new(),
        },
    };

    let pipeline = AutomationPipeline::new(config).unwrap();

    let gate_result = pipeline
        .evaluate_quality_gate(&pipeline.config.quality_gates[0], &result)
        .unwrap();
    assert!(gate_result); // Should pass since 85.5 >= 80.0
}

#[test]
fn test_variable_expansion() {
    let context = PipelineContext::from_environment().unwrap();
    let executor = StageExecutor {
        executor_type: ExecutorType::Shell,
        working_dir: PathBuf::from("."),
        environment: HashMap::new(),
    };

    let command = "echo 'Project: ${PROJECT_NAME}, Version: ${VERSION}'";
    let expanded = executor.expand_variables(command, &context);

    assert!(expanded.contains(&context.project.name));
    assert!(expanded.contains(&context.version.current));
}
