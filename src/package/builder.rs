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

//! Build script support for AetherScript packages
//!
//! Provides comprehensive build system integration including custom build scripts,
//! dependency compilation, artifact generation, and build caching.

#![allow(dead_code)]

use crate::error::SemanticError;
use crate::package::manifest::PackageManifest;
use crate::package::BuildConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Package builder for AetherScript
#[derive(Debug)]
pub struct PackageBuilder {
    /// Target directory
    target_dir: PathBuf,

    /// Build artifacts
    artifacts: Vec<BuildArtifact>,

    /// Build script runner
    script_runner: BuildScriptRunner,

    /// Build environment
    env: BuildEnvironment,

    /// Active build processes
    active_builds: HashMap<String, BuildProcess>,
}

/// Build cache for incremental compilation
#[derive(Debug, Default)]
pub struct BuildCache {
    /// Cached build artifacts
    artifacts: HashMap<String, CachedArtifact>,

    /// Source file timestamps
    source_timestamps: HashMap<PathBuf, std::time::SystemTime>,

    /// Dependency timestamps
    dependency_timestamps: HashMap<String, std::time::SystemTime>,

    /// Build configuration hash
    config_hash: Option<String>,

    /// Cache statistics
    stats: BuildCacheStats,
}

/// Cached build artifact
#[derive(Debug, Clone)]
pub struct CachedArtifact {
    /// Artifact path
    pub path: PathBuf,

    /// Source files used
    pub sources: Vec<PathBuf>,

    /// Dependencies used
    pub dependencies: Vec<String>,

    /// Build timestamp
    pub built_at: std::time::SystemTime,

    /// Artifact hash
    pub hash: String,

    /// Artifact size
    pub size: u64,
}

/// Build cache statistics
#[derive(Debug, Default, Clone)]
pub struct BuildCacheStats {
    /// Total artifacts cached
    pub total_artifacts: usize,

    /// Cache hits
    pub hits: u64,

    /// Cache misses
    pub misses: u64,

    /// Total cache size in bytes
    pub total_size: u64,
}

/// Build environment configuration
#[derive(Debug, Clone)]
pub struct BuildEnvironment {
    /// Environment variables
    pub env_vars: HashMap<String, String>,

    /// Include directories
    pub include_dirs: Vec<PathBuf>,

    /// Library directories
    pub lib_dirs: Vec<PathBuf>,

    /// Linked libraries
    pub libs: Vec<String>,

    /// Build flags
    pub flags: Vec<String>,

    /// Target triple
    pub target: Option<String>,

    /// Working directory
    pub work_dir: PathBuf,
}

/// Build process tracking
#[derive(Debug)]
pub struct BuildProcess {
    /// Package being built
    pub package_name: String,

    /// Build start time
    pub started_at: std::time::SystemTime,

    /// Build stages
    pub stages: Vec<BuildStage>,

    /// Current stage
    pub current_stage: usize,

    /// Build output
    pub output: Vec<String>,

    /// Build warnings
    pub warnings: Vec<String>,

    /// Build errors
    pub errors: Vec<String>,
}

/// Build stage
#[derive(Debug, Clone)]
pub struct BuildStage {
    /// Stage name
    pub name: String,

    /// Stage description
    pub description: String,

    /// Stage start time
    pub started_at: Option<std::time::SystemTime>,

    /// Stage end time
    pub finished_at: Option<std::time::SystemTime>,

    /// Stage status
    pub status: BuildStageStatus,

    /// Stage output
    pub output: Vec<String>,
}

/// Build stage status
#[derive(Debug, Clone)]
pub enum BuildStageStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Skipped(String),
}

/// Build result
#[derive(Debug)]
pub struct BuildResult {
    /// Build success
    pub success: bool,

    /// Generated artifacts
    pub artifacts: Vec<BuildArtifact>,

    /// Build duration
    pub duration: std::time::Duration,

    /// Build warnings
    pub warnings: Vec<String>,

    /// Build errors
    pub errors: Vec<String>,

    /// Cache statistics
    pub cache_stats: BuildCacheStats,
}

/// Build artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    /// Artifact type
    pub artifact_type: ArtifactType,

    /// Artifact path
    pub path: PathBuf,

    /// Artifact size
    pub size: u64,

    /// Artifact hash
    pub hash: String,

    /// Target platform
    pub target: Option<String>,
}

/// Types of build artifacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Executable binary
    Executable,

    /// Dynamic library
    DynamicLibrary,

    /// Static library
    StaticLibrary,

    /// Object file
    ObjectFile,

    /// Documentation
    Documentation,

    /// Examples
    Example,

    /// Test executable
    TestExecutable,

    /// Benchmark executable
    BenchmarkExecutable,
}

/// Build script execution context
#[derive(Debug)]
pub struct BuildScriptContext {
    /// Package manifest
    pub manifest: PackageManifest,

    /// Build environment
    pub environment: BuildEnvironment,

    /// Output directory
    pub out_dir: PathBuf,

    /// Source directory
    pub src_dir: PathBuf,

    /// Available dependencies
    pub dependencies: HashMap<String, PathBuf>,

    /// Build features
    pub features: Vec<String>,
}

/// Custom build script runner
#[derive(Debug)]
pub struct SandboxConfig;

#[derive(Debug)]
pub struct BuildScriptRunner {
    /// Sandbox configuration
    sandbox_config: SandboxConfig,
}

/// Script interpreter types
#[derive(Debug, Clone)]
pub enum ScriptInterpreter {
    /// AetherScript interpreter
    AetherScript,

    /// Shell script
    Shell(String), // shell path

    /// Python script
    Python(String), // python path

    /// Node.js script
    NodeJs(String), // node path

    /// Custom interpreter
    Custom { command: String, args: Vec<String> },
}

impl PackageBuilder {
    /// Create a new package builder
    pub fn new(config: BuildConfig) -> Self {
        let work_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let env = BuildEnvironment {
            env_vars: std::env::vars().collect(),
            include_dirs: Vec::new(),
            lib_dirs: Vec::new(),
            libs: Vec::new(),
            flags: Vec::new(),
            target: config.target_triple.clone(),
            work_dir: work_dir.clone(),
        };

        Self {
            target_dir: work_dir.join("target"),
            artifacts: Vec::new(),
            script_runner: BuildScriptRunner::new().unwrap(),
            env,
            active_builds: HashMap::new(),
        }
    }

    /// Build a package
    pub fn build_package(
        &mut self,
        manifest: &PackageManifest,
    ) -> Result<BuildResult, SemanticError> {
        let start_time = std::time::Instant::now();
        let package_name = manifest.name().to_string();

        // In a real implementation, this would be populated from the build process
        let process = BuildProcess {
            package_name: package_name.clone(),
            started_at: std::time::SystemTime::now(),
            stages: self.create_build_stages(manifest),
            current_stage: 0,
            output: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        let mut artifacts = Vec::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut success = true;

        // Execute build stages
        for stage_idx in 0..process.stages.len() {
            let stage_name = process.stages[stage_idx].name.clone();

            match self.execute_build_stage(manifest, stage_idx) {
                Ok(stage_artifacts) => {
                    artifacts.extend(stage_artifacts);
                }
                Err(e) => {
                    errors.push(format!("Stage '{}' failed: {}", stage_name, e));
                    success = false;
                    break;
                }
            }
        }

        // Collect final results
        warnings.extend(process.warnings);
        errors.extend(process.errors);

        Ok(BuildResult {
            success,
            artifacts,
            duration: start_time.elapsed(),
            warnings,
            errors,
            cache_stats: self.get_cache_stats(),
        })
    }

    /// Create package archive
    pub fn create_package_archive(
        &self,
        manifest: &PackageManifest,
    ) -> Result<PathBuf, SemanticError> {
        let archive_name = format!("{}-{}.tar.gz", manifest.name(), manifest.version());
        let archive_path = self.target_dir.join("package").join(archive_name);

        // Create target directory
        if let Some(parent) = archive_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create archive (simplified implementation)
        let archive_data = self.create_archive_data(manifest)?;
        std::fs::write(&archive_path, archive_data)?;

        Ok(archive_path)
    }

    /// Clean build artifacts
    pub fn clean(&mut self, _manifest: &PackageManifest) -> Result<(), SemanticError> {
        if self.target_dir.exists() {
            std::fs::remove_dir_all(&self.target_dir)?;
        }

        Ok(())
    }

    /// Get build cache statistics
    pub fn cache_stats(&self) -> BuildCacheStats {
        BuildCacheStats::default()
    }

    /// Clear build cache
    pub fn clear_cache(&mut self) {}

    // Private implementation methods

    fn create_build_stages(&self, manifest: &PackageManifest) -> Vec<BuildStage> {
        let mut stages = Vec::new();

        // Pre-build stage
        stages.push(BuildStage {
            name: "pre-build".to_string(),
            description: "Prepare build environment".to_string(),
            started_at: None,
            finished_at: None,
            status: BuildStageStatus::Pending,
            output: Vec::new(),
        });

        // Dependencies stage
        if !manifest.dependencies.is_empty() {
            stages.push(BuildStage {
                name: "dependencies".to_string(),
                description: "Build dependencies".to_string(),
                started_at: None,
                finished_at: None,
                status: BuildStageStatus::Pending,
                output: Vec::new(),
            });
        }

        // Build script stage
        if manifest.build.script.is_some() {
            stages.push(BuildStage {
                name: "build-script".to_string(),
                description: "Execute build script".to_string(),
                started_at: None,
                finished_at: None,
                status: BuildStageStatus::Pending,
                output: Vec::new(),
            });
        }

        // Library stage
        if manifest.is_library() {
            stages.push(BuildStage {
                name: "library".to_string(),
                description: "Build library".to_string(),
                started_at: None,
                finished_at: None,
                status: BuildStageStatus::Pending,
                output: Vec::new(),
            });
        }

        // Binaries stage
        if manifest.has_binaries() {
            stages.push(BuildStage {
                name: "binaries".to_string(),
                description: "Build binaries".to_string(),
                started_at: None,
                finished_at: None,
                status: BuildStageStatus::Pending,
                output: Vec::new(),
            });
        }

        // Examples stage
        if !manifest.example.is_empty() {
            stages.push(BuildStage {
                name: "examples".to_string(),
                description: "Build examples".to_string(),
                started_at: None,
                finished_at: None,
                status: BuildStageStatus::Pending,
                output: Vec::new(),
            });
        }

        // Tests stage
        if !manifest.test.is_empty() {
            stages.push(BuildStage {
                name: "tests".to_string(),
                description: "Build tests".to_string(),
                started_at: None,
                finished_at: None,
                status: BuildStageStatus::Pending,
                output: Vec::new(),
            });
        }

        // Documentation stage
        stages.push(BuildStage {
            name: "documentation".to_string(),
            description: "Generate documentation".to_string(),
            started_at: None,
            finished_at: None,
            status: BuildStageStatus::Pending,
            output: Vec::new(),
        });

        stages
    }

    fn execute_build_stage(
        &mut self,
        manifest: &PackageManifest,
        stage_idx: usize,
    ) -> Result<Vec<BuildArtifact>, SemanticError> {
        let package_name = manifest.name().to_string();
        let stage_name = self.active_builds[&package_name].stages[stage_idx]
            .name
            .clone();

        // Update stage status
        if let Some(process) = self.active_builds.get_mut(&package_name) {
            process.stages[stage_idx].status = BuildStageStatus::Running;
            process.stages[stage_idx].started_at = Some(std::time::SystemTime::now());
            process.current_stage = stage_idx;
        }

        let artifacts = match stage_name.as_str() {
            "pre-build" => self.execute_pre_build_stage(manifest)?,
            "dependencies" => self.execute_dependencies_stage(manifest)?,
            "build-script" => self.execute_build_script_stage(manifest)?,
            "library" => self.execute_library_stage(manifest)?,
            "binaries" => self.execute_binaries_stage(manifest)?,
            "examples" => self.execute_examples_stage(manifest)?,
            "tests" => self.execute_tests_stage(manifest)?,
            "documentation" => self.execute_documentation_stage(manifest)?,
            _ => Vec::new(),
        };

        // Update stage completion
        if let Some(process) = self.active_builds.get_mut(&package_name) {
            process.stages[stage_idx].status = BuildStageStatus::Completed;
            process.stages[stage_idx].finished_at = Some(std::time::SystemTime::now());
        }

        Ok(artifacts)
    }

    fn execute_pre_build_stage(
        &mut self,
        manifest: &PackageManifest,
    ) -> Result<Vec<BuildArtifact>, SemanticError> {
        // Setup build environment
        self.setup_build_environment(manifest)?;

        // Create output directories
        self.create_output_directories()?;

        // Check for incremental build
        if self.can_skip_build(manifest)? {
            return Ok(Vec::new());
        }

        Ok(Vec::new())
    }

    fn execute_dependencies_stage(
        &mut self,
        manifest: &PackageManifest,
    ) -> Result<Vec<BuildArtifact>, SemanticError> {
        // Build dependencies in dependency order
        for dependency in &manifest.dependencies {
            self.build_dependency(&dependency.name)?;
        }

        Ok(Vec::new())
    }

    fn execute_build_script_stage(
        &mut self,
        manifest: &PackageManifest,
    ) -> Result<Vec<BuildArtifact>, SemanticError> {
        if let Some(ref script_path) = manifest.build.script {
            let context = BuildScriptContext {
                manifest: manifest.clone(),
                environment: self.get_build_environment().clone(),
                out_dir: self.target_dir.join("build"),
                src_dir: self.target_dir.join("src"),
                dependencies: HashMap::new(), // TODO: Populate with actual dependencies
                features: Vec::new(),         // TODO: Pass enabled features
            };

            self.script_runner.execute_script(script_path, &context)?;
        }

        Ok(Vec::new())
    }

    fn execute_library_stage(
        &mut self,
        manifest: &PackageManifest,
    ) -> Result<Vec<BuildArtifact>, SemanticError> {
        if let Some(ref lib_target) = manifest.lib {
            let default_lib_path = PathBuf::from("src/lib.aether");
            let lib_path = lib_target.path.as_ref().unwrap_or(&default_lib_path);

            let default_name = manifest.package.name.clone();
            let lib_name = lib_target.name.as_ref().unwrap_or(&default_name);
            let artifact = self.compile_library(lib_path, lib_name)?;

            return Ok(vec![artifact]);
        }

        Ok(Vec::new())
    }

    fn execute_binaries_stage(
        &mut self,
        manifest: &PackageManifest,
    ) -> Result<Vec<BuildArtifact>, SemanticError> {
        let mut artifacts = Vec::new();

        for bin_target in &manifest.bin {
            let default_bin_path = PathBuf::from(format!("src/bin/{}.aether", bin_target.name));
            let bin_path = bin_target.path.as_ref().unwrap_or(&default_bin_path);

            let artifact = self.compile_binary(bin_path, &bin_target.name)?;
            artifacts.push(artifact);
        }

        Ok(artifacts)
    }

    fn execute_examples_stage(
        &mut self,
        manifest: &PackageManifest,
    ) -> Result<Vec<BuildArtifact>, SemanticError> {
        let mut artifacts = Vec::new();

        for example_target in &manifest.example {
            let default_example_path =
                PathBuf::from(format!("examples/{}.aether", example_target.name));
            let example_path = example_target
                .path
                .as_ref()
                .unwrap_or(&default_example_path);

            let artifact = self.compile_example(example_path, &example_target.name)?;
            artifacts.push(artifact);
        }

        Ok(artifacts)
    }

    fn execute_tests_stage(
        &mut self,
        manifest: &PackageManifest,
    ) -> Result<Vec<BuildArtifact>, SemanticError> {
        let mut artifacts = Vec::new();

        for test_target in &manifest.test {
            let default_test_path = PathBuf::from(format!("tests/{}.aether", test_target.name));
            let test_path = test_target.path.as_ref().unwrap_or(&default_test_path);

            let artifact = self.compile_test(test_path, &test_target.name)?;
            artifacts.push(artifact);
        }

        Ok(artifacts)
    }

    fn execute_documentation_stage(
        &mut self,
        _manifest: &PackageManifest,
    ) -> Result<Vec<BuildArtifact>, SemanticError> {
        let doc_dir = self.env.work_dir.join("target").join("doc");
        std::fs::create_dir_all(&doc_dir)?;

        // Generate documentation
        let artifact = BuildArtifact {
            artifact_type: ArtifactType::Documentation,
            path: doc_dir.join("index.html"),
            size: 1024,                   // Placeholder
            hash: "doc_hash".to_string(), // Placeholder
            target: None,
        };

        Ok(vec![artifact])
    }

    fn setup_build_environment(&mut self, manifest: &PackageManifest) -> Result<(), SemanticError> {
        // Add manifest build configuration to environment
        for (key, value) in &manifest.build.env {
            self.env.env_vars.insert(key.clone(), value.clone());
        }

        // Add include directories
        self.env.include_dirs.extend(manifest.build.include.clone());

        // Add library directories
        self.env.lib_dirs.extend(manifest.build.lib_dirs.clone());

        // Add libraries
        self.env.libs.extend(manifest.build.libs.clone());

        // Add build flags
        self.env.flags.extend(manifest.build.flags.clone());

        Ok(())
    }

    fn create_output_directories(&self) -> Result<(), SemanticError> {
        let target_dir = self.env.work_dir.join("target");
        std::fs::create_dir_all(&target_dir)?;

        let build_dir = target_dir.join("build");
        std::fs::create_dir_all(&build_dir)?;

        let debug_dir = target_dir.join("debug");
        std::fs::create_dir_all(&debug_dir)?;

        let release_dir = target_dir.join("release");
        std::fs::create_dir_all(&release_dir)?;

        Ok(())
    }

    fn can_skip_build(&self, _manifest: &PackageManifest) -> Result<bool, SemanticError> {
        // Check if build can be skipped based on cache
        // This is a simplified implementation
        Ok(false)
    }

    fn build_dependency(&mut self, _dep_name: &str) -> Result<(), SemanticError> {
        // Build a dependency package
        // This would involve loading the dependency's manifest and building it
        Ok(())
    }

    /// Get current target
    fn get_target(&self) -> Option<String> {
        self.env.target.clone()
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> BuildCacheStats {
        BuildCacheStats {
            total_artifacts: 0,
            hits: 0,
            misses: 0,
            total_size: 0,
        }
    }

    /// Get build environment
    pub fn get_build_environment(&mut self) -> &BuildEnvironment {
        &self.env
    }

    fn compile_library(
        &self,
        source_path: &PathBuf,
        name: &str,
    ) -> Result<BuildArtifact, SemanticError> {
        let output_path = self.target_dir.join("debug").join(format!("lib{}.a", name));

        // Compile library (simplified)
        self.run_compiler(source_path, &output_path)?;

        Ok(BuildArtifact {
            artifact_type: ArtifactType::StaticLibrary,
            path: output_path,
            size: 1024,                   // Placeholder
            hash: "lib_hash".to_string(), // Placeholder
            target: self.get_target(),
        })
    }

    fn compile_binary(
        &self,
        source_path: &PathBuf,
        name: &str,
    ) -> Result<BuildArtifact, SemanticError> {
        let output_path = self.target_dir.join("debug").join(name);

        // Compile binary (simplified)
        self.run_compiler(source_path, &output_path)?;

        Ok(BuildArtifact {
            artifact_type: ArtifactType::Executable,
            path: output_path,
            size: 2048,                   // Placeholder
            hash: "bin_hash".to_string(), // Placeholder
            target: self.get_target(),
        })
    }

    fn compile_example(
        &self,
        source_path: &PathBuf,
        name: &str,
    ) -> Result<BuildArtifact, SemanticError> {
        let output_path = self.target_dir.join("debug").join("examples").join(name);

        // Compile example (simplified)
        self.run_compiler(source_path, &output_path)?;

        Ok(BuildArtifact {
            artifact_type: ArtifactType::Example,
            path: output_path,
            size: 1536,                       // Placeholder
            hash: "example_hash".to_string(), // Placeholder
            target: self.get_target(),
        })
    }

    fn compile_test(
        &self,
        source_path: &PathBuf,
        name: &str,
    ) -> Result<BuildArtifact, SemanticError> {
        let output_path = self.target_dir.join("debug").join("tests").join(name);

        // Compile test (simplified)
        self.run_compiler(source_path, &output_path)?;

        Ok(BuildArtifact {
            artifact_type: ArtifactType::TestExecutable,
            path: output_path,
            size: 1792,                    // Placeholder
            hash: "test_hash".to_string(), // Placeholder
            target: self.get_target(),
        })
    }

    fn run_compiler(&self, source: &PathBuf, output: &PathBuf) -> Result<(), SemanticError> {
        // Create output directory
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Run AetherScript compiler (simplified)
        let mut cmd = Command::new("aether");
        cmd.arg("compile").arg(source).arg("-o").arg(output);

        // Execute compiler
        let output = cmd.output().map_err(|e| SemanticError::Internal {
            message: format!("Failed to execute compiler: {}", e),
        })?;

        if !output.status.success() {
            return Err(SemanticError::Internal {
                message: format!(
                    "Compilation failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    fn create_archive_data(&self, manifest: &PackageManifest) -> Result<Vec<u8>, SemanticError> {
        // Create a simplified archive (in real implementation, use tar/zip)
        let manifest_data = format!("# Package archive for {}\n", manifest.name());

        Ok(manifest_data.into_bytes())
    }
}

impl BuildCache {
    fn clear(&mut self) {
        self.artifacts.clear();
        self.source_timestamps.clear();
        self.dependency_timestamps.clear();
        self.config_hash = None;
        self.stats = BuildCacheStats::default();
    }
}

impl BuildScriptRunner {
    fn new() -> Result<Self, SemanticError> {
        Ok(Self {
            sandbox_config: SandboxConfig,
        })
    }

    fn execute_script(
        &self,
        script_path: &PathBuf,
        context: &BuildScriptContext,
    ) -> Result<(), SemanticError> {
        self.execute_aether_script(script_path, context)
    }

    fn execute_aether_script(
        &self,
        script_path: &PathBuf,
        context: &BuildScriptContext,
    ) -> Result<(), SemanticError> {
        // Execute AetherScript build script
        let mut cmd = Command::new("aether");
        cmd.arg("run")
            .arg(script_path)
            .current_dir(&context.environment.work_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &context.environment.env_vars {
            cmd.env(key, value);
        }

        // Set build-specific environment variables
        cmd.env("AETHER_OUT_DIR", &context.out_dir);
        cmd.env("AETHER_SRC_DIR", &context.src_dir);
        cmd.env("AETHER_PACKAGE_NAME", &context.manifest.package.name);
        cmd.env(
            "AETHER_PACKAGE_VERSION",
            context.manifest.package.version.to_string(),
        );

        let output = cmd.output().map_err(|e| SemanticError::Internal {
            message: format!("Failed to execute build script: {}", e),
        })?;

        if !output.status.success() {
            return Err(SemanticError::Internal {
                message: format!(
                    "Build script failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    fn execute_shell_script(
        &self,
        script_path: &PathBuf,
        context: &BuildScriptContext,
        shell: &str,
    ) -> Result<(), SemanticError> {
        let mut cmd = Command::new(shell);
        cmd.arg(script_path);
        self.execute_external_script(cmd, context)
    }

    fn execute_python_script(
        &self,
        script_path: &PathBuf,
        context: &BuildScriptContext,
        python: &str,
    ) -> Result<(), SemanticError> {
        let mut cmd = Command::new(python);
        cmd.arg(script_path);
        self.execute_external_script(cmd, context)
    }

    fn execute_node_script(
        &self,
        script_path: &PathBuf,
        context: &BuildScriptContext,
        node: &str,
    ) -> Result<(), SemanticError> {
        let mut cmd = Command::new(node);
        cmd.arg(script_path);
        self.execute_external_script(cmd, context)
    }

    fn execute_custom_script(
        &self,
        script_path: &PathBuf,
        context: &BuildScriptContext,
        command: &str,
        args: &[String],
    ) -> Result<(), SemanticError> {
        let mut cmd = Command::new(command);
        cmd.args(args).arg(script_path);
        self.execute_external_script(cmd, context)
    }

    fn execute_external_script(
        &self,
        mut cmd: Command,
        context: &BuildScriptContext,
    ) -> Result<(), SemanticError> {
        cmd.current_dir(&context.environment.work_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        for (key, value) in &context.environment.env_vars {
            cmd.env(key, value);
        }

        let output = cmd.output().map_err(|e| SemanticError::Internal {
            message: format!("Failed to execute build script: {}", e),
        })?;

        if !output.status.success() {
            return Err(SemanticError::Internal {
                message: format!(
                    "Build script failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }
}


#[cfg(test)]
#[path = "builder_tests.rs"]
mod builder_tests;
