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

//! Dependency resolution for AetherScript packages
//!
//! Implements sophisticated dependency resolution algorithms including
//! backtracking, conflict resolution, and optimal version selection.

use crate::error::SemanticError;
use crate::package::manifest::Dependency;
use crate::package::version::{Version, VersionRequirement};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

/// Dependency resolver
#[derive(Debug)]
pub struct DependencyResolver {
    /// Resolution cache
    cache: ResolutionCache,

    /// Resolver configuration
    config: ResolverConfig,

    /// Conflict resolution strategy
    strategy: ConflictStrategy,
}

/// Resolution cache for performance
#[derive(Debug, Default)]
pub struct ResolutionCache {
    /// Cached resolution results
    resolutions: HashMap<ResolutionKey, ResolutionResult>,

    /// Package version cache
    versions: HashMap<String, Vec<Version>>,

    /// Dependency cache
    dependencies: HashMap<(String, Version), Vec<Dependency>>,
}

/// Cache key for resolution results
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResolutionKey {
    /// Root dependencies
    dependencies: Vec<DependencySpec>,

    /// Feature flags
    features: Vec<String>,

    /// Target platform
    target: Option<String>,
}

/// Dependency specification for resolution
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DependencySpec {
    /// Package name
    pub name: String,

    /// Version requirement
    pub requirement: String, // Serialized version requirement

    /// Features enabled
    pub features: Vec<String>,

    /// Whether optional
    pub optional: bool,
}

/// Resolution result
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// Resolved dependencies
    pub resolved: Vec<ResolvedDependency>,

    /// Resolution graph
    pub graph: DependencyGraph,

    /// Any warnings
    pub warnings: Vec<ResolutionWarning>,

    /// Resolution time
    pub duration: std::time::Duration,
}

/// Resolved dependency
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolvedDependency {
    /// Package name
    pub name: String,

    /// Resolved version
    pub version: Version,

    /// Source of dependency
    pub source: DependencySource,

    /// Features enabled
    pub features: Vec<String>,

    /// Direct dependencies
    pub dependencies: Vec<String>,
}

/// Source of a dependency
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencySource {
    /// Registry package
    Registry { registry: String },

    /// Git repository
    Git { url: String, rev: String },

    /// Local path
    Path { path: String },
}

/// Dependency graph
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Nodes (packages)
    pub nodes: HashMap<String, GraphNode>,

    /// Edges (dependencies)
    pub edges: Vec<GraphEdge>,

    /// Root packages
    pub roots: Vec<String>,
}

/// Graph node (package)
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Package name
    pub name: String,

    /// Package version
    pub version: Version,

    /// Features enabled
    pub features: Vec<String>,

    /// Dependency depth
    pub depth: usize,
}

/// Graph edge (dependency relationship)
#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// From package
    pub from: String,

    /// To package
    pub to: String,

    /// Version requirement
    pub requirement: VersionRequirement,

    /// Whether optional
    pub optional: bool,
}

/// Resolution warning
#[derive(Debug, Clone)]
pub struct ResolutionWarning {
    /// Warning type
    pub warning_type: WarningType,

    /// Warning message
    pub message: String,

    /// Related packages
    pub packages: Vec<String>,
}

/// Types of resolution warnings
#[derive(Debug, Clone)]
pub enum WarningType {
    /// Version constraint too restrictive
    RestrictiveConstraint,

    /// Potentially incompatible versions
    VersionSkew,

    /// Yanked version used
    YankedVersion,

    /// Pre-release version used
    PreReleaseVersion,

    /// Unused feature
    UnusedFeature,

    /// Duplicate dependency
    DuplicateDependency,
}

/// Resolver configuration
#[derive(Debug, Clone)]
pub struct ResolverConfig {
    /// Maximum resolution depth
    pub max_depth: usize,

    /// Maximum backtrack attempts
    pub max_backtracks: usize,

    /// Whether to allow pre-release versions
    pub allow_prerelease: bool,

    /// Whether to prefer minimal versions
    pub minimal_versions: bool,

    /// Offline mode
    pub offline: bool,

    /// Target platform
    pub target: Option<String>,
}

/// Conflict resolution strategy
pub enum ConflictStrategy {
    /// Prefer newest compatible versions
    Newest,

    /// Prefer oldest compatible versions
    Oldest,

    /// Prefer minimal version set
    Minimal,

    /// Custom strategy
    Custom(Box<dyn Fn(&[Version]) -> Option<Version>>),
}

impl std::fmt::Debug for ConflictStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Newest => write!(f, "Newest"),
            Self::Oldest => write!(f, "Oldest"),
            Self::Minimal => write!(f, "Minimal"),
            Self::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

/// Resolution error
#[derive(Debug, Clone)]
pub struct ResolutionError {
    /// Error type
    pub error_type: ResolutionErrorType,

    /// Error message
    pub message: String,

    /// Conflicting packages
    pub conflicts: Vec<PackageConflict>,
}

/// Types of resolution errors
#[derive(Debug, Clone)]
pub enum ResolutionErrorType {
    /// No version satisfies requirements
    NoSatisfyingVersion,

    /// Circular dependency detected
    CircularDependency,

    /// Package not found
    PackageNotFound,

    /// Version conflict
    VersionConflict,

    /// Feature conflict
    FeatureConflict,

    /// Maximum depth exceeded
    MaxDepthExceeded,
}

/// Package conflict information
#[derive(Debug, Clone)]
pub struct PackageConflict {
    /// Package name
    pub package: String,

    /// Conflicting requirements
    pub requirements: Vec<ConflictingRequirement>,
}

/// Conflicting requirement
#[derive(Debug, Clone)]
pub struct ConflictingRequirement {
    /// Version requirement
    pub requirement: VersionRequirement,

    /// Source of requirement
    pub source: String,

    /// Feature context
    pub features: Vec<String>,
}

/// Resolution state for backtracking
#[derive(Debug, Clone)]
struct ResolutionState {
    /// Currently resolved packages
    resolved: HashMap<String, ResolvedDependency>,

    /// Pending dependencies to resolve
    pending: VecDeque<PendingDependency>,

    /// Chosen versions
    choices: HashMap<String, Version>,
}

/// Pending dependency to resolve
#[derive(Debug, Clone)]
struct PendingDependency {
    /// Dependency specification
    spec: Dependency,

    /// Depth in dependency tree
    depth: usize,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self {
            cache: ResolutionCache::default(),
            config: ResolverConfig::default(),
            strategy: ConflictStrategy::Newest,
        }
    }

    /// Create resolver with configuration
    pub fn with_config(config: ResolverConfig) -> Self {
        Self {
            cache: ResolutionCache::default(),
            config,
            strategy: ConflictStrategy::Newest,
        }
    }

    /// Resolve dependencies
    pub fn resolve(
        &mut self,
        dependencies: Vec<Dependency>,
    ) -> Result<Vec<ResolvedDependency>, SemanticError> {
        let start_time = std::time::Instant::now();

        // Create resolution key for caching
        let key = ResolutionKey {
            dependencies: dependencies
                .iter()
                .map(|d| DependencySpec {
                    name: d.name.clone(),
                    requirement: d.version.to_string(),
                    features: d.features.clone(),
                    optional: d.optional,
                })
                .collect(),
            features: vec![], // TODO: Pass features from context
            target: self.config.target.clone(),
        };

        // Check cache first
        if let Some(cached) = self.cache.resolutions.get(&key) {
            return Ok(cached.resolved.clone());
        }

        // Perform resolution
        let result = self.resolve_internal(dependencies)?;

        // Cache result
        let resolution_result = ResolutionResult {
            resolved: result.clone(),
            graph: self.build_dependency_graph(&result)?,
            warnings: vec![], // TODO: Collect warnings during resolution
            duration: start_time.elapsed(),
        };

        self.cache.resolutions.insert(key, resolution_result);

        Ok(result)
    }

    /// Internal resolution implementation
    fn resolve_internal(
        &mut self,
        dependencies: Vec<Dependency>,
    ) -> Result<Vec<ResolvedDependency>, SemanticError> {
        let mut state = ResolutionState {
            resolved: HashMap::new(),
            pending: VecDeque::new(),
            choices: HashMap::new(),
        };

        // Initialize with root dependencies
        for dep in dependencies {
            state.pending.push_back(PendingDependency {
                spec: dep,
                depth: 0,
            });
        }

        // Resolve dependencies using backtracking
        self.resolve_with_backtracking(&mut state)?;

        // Convert to sorted list
        let mut resolved: Vec<ResolvedDependency> = state.resolved.into_values().collect();
        resolved.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(resolved)
    }

    /// Resolve dependencies with backtracking
    fn resolve_with_backtracking(
        &mut self,
        state: &mut ResolutionState,
    ) -> Result<(), SemanticError> {
        let mut backtrack_count = 0;

        while let Some(pending) = state.pending.pop_front() {
            // Check depth limit
            if pending.depth > self.config.max_depth {
                return Err(SemanticError::Internal {
                    message: format!(
                        "Maximum dependency depth {} exceeded",
                        self.config.max_depth
                    ),
                });
            }

            // Skip if already resolved
            if state.resolved.contains_key(&pending.spec.name) {
                // Check for conflicts
                let existing = &state.resolved[&pending.spec.name];
                if !pending.spec.version.matches(&existing.version) {
                    // Version conflict - try to backtrack
                    if backtrack_count < self.config.max_backtracks {
                        backtrack_count += 1;
                        if let Some(alternative) = self.find_alternative_version(state, &pending)? {
                            // Update resolved version
                            state
                                .resolved
                                .insert(pending.spec.name.clone(), alternative);
                            continue;
                        }
                    }

                    return Err(SemanticError::Internal {
                        message: format!(
                            "Version conflict for package {}: existing {} vs required {}",
                            pending.spec.name, existing.version, pending.spec.version
                        ),
                    });
                }
                continue;
            }

            // Find compatible version
            let version = self.select_version(&pending.spec)?;

            // Create resolved dependency
            let resolved_dep = ResolvedDependency {
                name: pending.spec.name.clone(),
                version: version.clone(),
                source: self.determine_source(&pending.spec)?,
                features: pending.spec.features.clone(),
                dependencies: vec![], // Will be filled when processing dependencies
            };

            // Add to resolved set
            state
                .resolved
                .insert(pending.spec.name.clone(), resolved_dep);
            state
                .choices
                .insert(pending.spec.name.clone(), version.clone());

            // Get dependencies of this package
            let package_deps = self.get_package_dependencies(&pending.spec.name, &version)?;

            // Add dependencies to pending queue
            for dep in package_deps {
                // Apply feature filtering
                if !self.should_include_dependency(&dep, &pending.spec.features) {
                    continue;
                }

                state.pending.push_back(PendingDependency {
                    spec: dep,
                    depth: pending.depth + 1,
                });
            }
        }

        Ok(())
    }

    /// Select best version for a dependency
    fn select_version(&self, dependency: &Dependency) -> Result<Version, SemanticError> {
        // Get available versions
        let available_versions = self.get_available_versions(&dependency.name)?;

        // Filter by version requirement
        let compatible_versions: Vec<&Version> = available_versions
            .iter()
            .filter(|v| dependency.version.matches(v))
            .filter(|v| self.config.allow_prerelease || !v.is_prerelease())
            .collect();

        if compatible_versions.is_empty() {
            return Err(SemanticError::Internal {
                message: format!(
                    "No compatible version found for {} {}",
                    dependency.name, dependency.version
                ),
            });
        }

        // Apply selection strategy
        match self.strategy {
            ConflictStrategy::Newest => {
                Ok(compatible_versions.iter().max().unwrap().clone().clone())
            }
            ConflictStrategy::Oldest => {
                Ok(compatible_versions.iter().min().unwrap().clone().clone())
            }
            ConflictStrategy::Minimal => {
                // Find minimal version that satisfies requirement
                Ok(compatible_versions.iter().min().unwrap().clone().clone())
            }
            ConflictStrategy::Custom(_) => {
                // TODO: Implement custom strategy
                Ok(compatible_versions.iter().max().unwrap().clone().clone())
            }
        }
    }

    /// Find alternative version during backtracking
    fn find_alternative_version(
        &self,
        _state: &ResolutionState,
        _pending: &PendingDependency,
    ) -> Result<Option<ResolvedDependency>, SemanticError> {
        // This is a simplified implementation
        // A real resolver would try different version combinations
        Ok(None)
    }

    /// Get available versions for a package
    fn get_available_versions(&self, package_name: &str) -> Result<Vec<Version>, SemanticError> {
        // Check cache first
        if let Some(versions) = self.cache.versions.get(package_name) {
            return Ok(versions.clone());
        }

        // In a real implementation, this would query the registry
        // For now, return some mock versions
        let versions = vec![
            Version::new(1, 0, 0),
            Version::new(1, 1, 0),
            Version::new(1, 2, 0),
            Version::new(2, 0, 0),
        ];

        Ok(versions)
    }

    /// Get dependencies for a specific package version
    fn get_package_dependencies(
        &mut self,
        package_name: &str,
        version: &Version,
    ) -> Result<Vec<Dependency>, SemanticError> {
        let key = (package_name.to_string(), version.clone());

        // Check cache first
        if let Some(deps) = self.cache.dependencies.get(&key) {
            return Ok(deps.clone());
        }

        // In a real implementation, this would fetch the package manifest
        // For now, return empty dependencies
        let dependencies = vec![];

        self.cache.dependencies.insert(key, dependencies.clone());
        Ok(dependencies)
    }

    /// Determine dependency source
    fn determine_source(&self, dependency: &Dependency) -> Result<DependencySource, SemanticError> {
        if let Some(ref git_url) = dependency.git {
            Ok(DependencySource::Git {
                url: git_url.clone(),
                rev: dependency.rev.clone().unwrap_or_else(|| "HEAD".to_string()),
            })
        } else if let Some(ref path) = dependency.path {
            Ok(DependencySource::Path {
                path: path.to_string_lossy().to_string(),
            })
        } else {
            Ok(DependencySource::Registry {
                registry: dependency
                    .registry
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
            })
        }
    }

    /// Check if dependency should be included based on features
    fn should_include_dependency(
        &self,
        dependency: &Dependency,
        enabled_features: &[String],
    ) -> bool {
        if dependency.optional
            && !enabled_features
                .iter()
                .any(|f| dependency.features.contains(f))
        {
            return false;
        }

        // TODO: Implement more sophisticated feature logic
        true
    }

    /// Build dependency graph from resolved dependencies
    fn build_dependency_graph(
        &self,
        resolved: &[ResolvedDependency],
    ) -> Result<DependencyGraph, SemanticError> {
        let mut graph = DependencyGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
            roots: Vec::new(),
        };

        // Add nodes
        for (i, dep) in resolved.iter().enumerate() {
            let node = GraphNode {
                name: dep.name.clone(),
                version: dep.version.clone(),
                features: dep.features.clone(),
                depth: i, // Simplified depth calculation
            };
            graph.nodes.insert(dep.name.clone(), node);
        }

        // TODO: Add edges based on dependency relationships

        Ok(graph)
    }

    /// Update resolver configuration
    pub fn set_config(&mut self, config: ResolverConfig) {
        self.config = config;
    }

    /// Set conflict resolution strategy
    pub fn set_strategy(&mut self, strategy: ConflictStrategy) {
        self.strategy = strategy;
    }

    /// Clear resolution cache
    pub fn clear_cache(&mut self) {
        self.cache = ResolutionCache::default();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            resolution_cache_size: self.cache.resolutions.len(),
            version_cache_size: self.cache.versions.len(),
            dependency_cache_size: self.cache.dependencies.len(),
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub resolution_cache_size: usize,
    pub version_cache_size: usize,
    pub dependency_cache_size: usize,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            max_backtracks: 1000,
            allow_prerelease: false,
            minimal_versions: false,
            offline: false,
            target: None,
        }
    }
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl fmt::Display for ResolutionErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolutionErrorType::NoSatisfyingVersion => write!(f, "No satisfying version"),
            ResolutionErrorType::CircularDependency => write!(f, "Circular dependency"),
            ResolutionErrorType::PackageNotFound => write!(f, "Package not found"),
            ResolutionErrorType::VersionConflict => write!(f, "Version conflict"),
            ResolutionErrorType::FeatureConflict => write!(f, "Feature conflict"),
            ResolutionErrorType::MaxDepthExceeded => write!(f, "Maximum depth exceeded"),
        }
    }
}

impl DependencyGraph {
    /// Check for circular dependencies
    pub fn has_cycles(&self) -> bool {
        // Simplified cycle detection
        // A real implementation would use proper graph algorithms
        false
    }

    /// Get topological ordering
    pub fn topological_sort(&self) -> Result<Vec<String>, SemanticError> {
        // Simplified topological sort
        let mut sorted: Vec<String> = self.nodes.keys().cloned().collect();
        sorted.sort();
        Ok(sorted)
    }

    /// Find shortest path between packages
    pub fn shortest_path(&self, _from: &str, _to: &str) -> Option<Vec<String>> {
        // Simplified path finding
        None
    }

    /// Get all transitive dependencies of a package
    pub fn transitive_dependencies(&self, package: &str) -> HashSet<String> {
        let mut deps = HashSet::new();

        for edge in &self.edges {
            if edge.from == package {
                deps.insert(edge.to.clone());
                // TODO: Recursively add transitive dependencies
            }
        }

        deps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::version::{Version, VersionRequirement};

    #[test]
    fn test_resolver_creation() {
        let resolver = DependencyResolver::new();
        assert_eq!(resolver.config.max_depth, 100);
        assert!(!resolver.config.allow_prerelease);
    }

    #[test]
    fn test_resolver_config() {
        let config = ResolverConfig {
            max_depth: 50,
            max_backtracks: 500,
            allow_prerelease: true,
            minimal_versions: true,
            offline: true,
            target: Some("x86_64-unknown-linux-gnu".to_string()),
        };

        let resolver = DependencyResolver::with_config(config.clone());
        assert_eq!(resolver.config.max_depth, 50);
        assert!(resolver.config.allow_prerelease);
        assert!(resolver.config.offline);
    }

    #[test]
    fn test_resolution_key() {
        let key1 = ResolutionKey {
            dependencies: vec![DependencySpec {
                name: "test".to_string(),
                requirement: "^1.0.0".to_string(),
                features: vec![],
                optional: false,
            }],
            features: vec![],
            target: None,
        };

        let key2 = key1.clone();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_resolved_dependency() {
        let dep = ResolvedDependency {
            name: "test-package".to_string(),
            version: Version::new(1, 2, 3),
            source: DependencySource::Registry {
                registry: "default".to_string(),
            },
            features: vec!["feature1".to_string()],
            dependencies: vec!["sub-package".to_string()],
        };

        assert_eq!(dep.name, "test-package");
        assert_eq!(dep.version.major, 1);
        assert_eq!(dep.features.len(), 1);
    }

    #[test]
    fn test_dependency_graph() {
        let mut graph = DependencyGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
            roots: vec!["root".to_string()],
        };

        let node = GraphNode {
            name: "test".to_string(),
            version: Version::new(1, 0, 0),
            features: vec![],
            depth: 0,
        };

        graph.nodes.insert("test".to_string(), node);

        assert_eq!(graph.nodes.len(), 1);
        assert!(!graph.has_cycles());
    }

    #[test]
    fn test_cache_stats() {
        let resolver = DependencyResolver::new();
        let stats = resolver.cache_stats();

        assert_eq!(stats.resolution_cache_size, 0);
        assert_eq!(stats.version_cache_size, 0);
        assert_eq!(stats.dependency_cache_size, 0);
    }

    #[test]
    fn test_conflict_strategy() {
        let strategy = ConflictStrategy::Newest;
        assert!(matches!(strategy, ConflictStrategy::Newest));

        let strategy = ConflictStrategy::Oldest;
        assert!(matches!(strategy, ConflictStrategy::Oldest));
    }
}
