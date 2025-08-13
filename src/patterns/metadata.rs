//! Pattern Metadata for Enhanced Discovery
//! 
//! Provides rich metadata for patterns to enable LLM discovery and selection

use super::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Enhanced metadata for pattern discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMetadata {
    /// Base metadata
    pub base: PatternMetadata,
    
    /// Semantic annotations
    pub semantic: SemanticAnnotations,
    
    /// Usage statistics
    pub usage: UsageStatistics,
    
    /// Quality metrics
    pub quality: QualityMetrics,
    
    /// Learning hints for LLMs
    pub learning_hints: LearningHints,
}

/// Semantic annotations for pattern understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnnotations {
    /// Domain this pattern belongs to
    pub domain: Vec<String>,
    
    /// Problem types this pattern solves
    pub solves: Vec<ProblemType>,
    
    /// Common use cases
    pub use_cases: Vec<UseCase>,
    
    /// Alternative patterns
    pub alternatives: Vec<String>,
    
    /// Pattern relationships
    pub relationships: Vec<PatternRelationship>,
    
    /// Natural language descriptions
    pub descriptions: MultilingualDescriptions,
}

/// Problem types that patterns solve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProblemType {
    /// Data validation problems
    Validation {
        data_type: String,
        constraints: Vec<String>,
    },
    /// Performance optimization
    Performance {
        optimization_type: String,
        improvement_factor: f32,
    },
    /// Security issues
    Security {
        threat_model: String,
        mitigation: String,
    },
    /// Correctness issues
    Correctness {
        bug_category: String,
        prevention_method: String,
    },
    /// Resource management
    ResourceManagement {
        resource_type: String,
        management_strategy: String,
    },
    /// Concurrency issues
    Concurrency {
        issue_type: String,
        synchronization: String,
    },
}

/// Use case for a pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCase {
    /// Use case title
    pub title: String,
    
    /// Detailed description
    pub description: String,
    
    /// When to use
    pub when_to_use: Vec<String>,
    
    /// When NOT to use
    pub when_not_to_use: Vec<String>,
    
    /// Real-world example
    pub example_scenario: String,
}

/// Pattern relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRelationship {
    /// Related pattern ID
    pub pattern_id: String,
    
    /// Relationship type
    pub relationship: RelationshipType,
    
    /// Description of relationship
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// This pattern extends another
    Extends,
    /// This pattern is extended by another
    ExtendedBy,
    /// Works well with
    Complements,
    /// Alternative to
    Alternative,
    /// Often used before
    Precedes,
    /// Often used after
    Follows,
    /// Part of larger pattern
    PartOf,
    /// Contains this pattern
    Contains,
}

/// Multilingual descriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultilingualDescriptions {
    /// Descriptions by language code
    pub descriptions: HashMap<String, LanguageDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDescription {
    /// Short description (one line)
    pub short: String,
    
    /// Medium description (paragraph)
    pub medium: String,
    
    /// Detailed description
    pub detailed: String,
    
    /// Keywords for search
    pub keywords: Vec<String>,
}

/// Usage statistics for patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    /// Number of times used
    pub use_count: u64,
    
    /// Success rate (0.0 - 1.0)
    pub success_rate: f32,
    
    /// Average generation time
    pub avg_generation_time_ms: u64,
    
    /// Common parameter values
    pub common_parameters: HashMap<String, Vec<ParameterFrequency>>,
    
    /// Common error patterns
    pub common_errors: Vec<ErrorPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterFrequency {
    pub value: String,
    pub frequency: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub error_type: String,
    pub frequency: f32,
    pub typical_cause: String,
    pub suggested_fix: String,
}

/// Quality metrics for patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Correctness score (0.0 - 1.0)
    pub correctness_score: f32,
    
    /// Performance score (0.0 - 1.0)
    pub performance_score: f32,
    
    /// Maintainability score (0.0 - 1.0)
    pub maintainability_score: f32,
    
    /// Test coverage
    pub test_coverage: f32,
    
    /// Verification confidence
    pub verification_confidence: f32,
    
    /// Community rating
    pub community_rating: Option<f32>,
    
    /// Expert review status
    pub expert_reviewed: bool,
}

/// Learning hints for LLMs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningHints {
    /// Concept prerequisites
    pub prerequisites: Vec<String>,
    
    /// Learning objectives
    pub objectives: Vec<String>,
    
    /// Common misconceptions
    pub misconceptions: Vec<Misconception>,
    
    /// Best practices
    pub best_practices: Vec<String>,
    
    /// Anti-patterns to avoid
    pub anti_patterns: Vec<AntiPattern>,
    
    /// Gradual complexity examples
    pub complexity_progression: Vec<ComplexityLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Misconception {
    pub misconception: String,
    pub correct_understanding: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    pub name: String,
    pub description: String,
    pub why_bad: String,
    pub better_alternative: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityLevel {
    pub level: u32,
    pub description: String,
    pub example_parameters: HashMap<String, ParameterValue>,
}

/// Metadata index for fast search
pub struct MetadataIndex {
    /// Patterns by domain
    by_domain: HashMap<String, Vec<String>>,
    
    /// Patterns by problem type
    by_problem: HashMap<String, Vec<String>>,
    
    /// Patterns by keyword
    by_keyword: HashMap<String, Vec<String>>,
    
    /// Quality rankings
    quality_rankings: Vec<(String, f32)>,
    
    /// Usage rankings
    usage_rankings: Vec<(String, u64)>,
}

impl MetadataIndex {
    /// Create new metadata index
    pub fn new() -> Self {
        Self {
            by_domain: HashMap::new(),
            by_problem: HashMap::new(),
            by_keyword: HashMap::new(),
            quality_rankings: Vec::new(),
            usage_rankings: Vec::new(),
        }
    }
    
    /// Index pattern metadata
    pub fn index_pattern(&mut self, pattern_id: &str, metadata: &EnhancedMetadata) {
        // Index by domain
        for domain in &metadata.semantic.domain {
            self.by_domain
                .entry(domain.clone())
                .or_insert_with(Vec::new)
                .push(pattern_id.to_string());
        }
        
        // Index by problem type
        for problem in &metadata.semantic.solves {
            let problem_key = format!("{:?}", problem);
            self.by_problem
                .entry(problem_key)
                .or_insert_with(Vec::new)
                .push(pattern_id.to_string());
        }
        
        // Index by keywords
        for (_, lang_desc) in &metadata.semantic.descriptions.descriptions {
            for keyword in &lang_desc.keywords {
                self.by_keyword
                    .entry(keyword.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(pattern_id.to_string());
            }
        }
        
        // Update quality ranking
        let quality_score = metadata.quality.correctness_score * 0.4 +
                           metadata.quality.performance_score * 0.3 +
                           metadata.quality.maintainability_score * 0.3;
        
        self.quality_rankings.push((pattern_id.to_string(), quality_score));
        self.quality_rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Update usage ranking
        self.usage_rankings.push((pattern_id.to_string(), metadata.usage.use_count));
        self.usage_rankings.sort_by(|a, b| b.1.cmp(&a.1));
    }
    
    /// Search patterns by domain
    pub fn search_by_domain(&self, domain: &str) -> Vec<&String> {
        self.by_domain
            .get(domain)
            .map(|ids| ids.iter().collect())
            .unwrap_or_default()
    }
    
    /// Search patterns by keyword
    pub fn search_by_keyword(&self, keyword: &str) -> Vec<&String> {
        self.by_keyword
            .get(&keyword.to_lowercase())
            .map(|ids| ids.iter().collect())
            .unwrap_or_default()
    }
    
    /// Get top patterns by quality
    pub fn top_by_quality(&self, n: usize) -> Vec<(&String, f32)> {
        self.quality_rankings
            .iter()
            .take(n)
            .map(|(id, score)| (id, *score))
            .collect()
    }
    
    /// Get top patterns by usage
    pub fn top_by_usage(&self, n: usize) -> Vec<(&String, u64)> {
        self.usage_rankings
            .iter()
            .take(n)
            .map(|(id, count)| (id, *count))
            .collect()
    }
}

/// Pattern recommendation engine
pub struct PatternRecommender {
    /// Metadata index
    index: MetadataIndex,
    
    /// Pattern similarity matrix
    similarity: HashMap<(String, String), f32>,
}

impl PatternRecommender {
    /// Create new recommender
    pub fn new(index: MetadataIndex) -> Self {
        Self {
            index,
            similarity: HashMap::new(),
        }
    }
    
    /// Recommend patterns for intent
    pub fn recommend_for_intent(
        &self,
        intent: &str,
        context: &RecommendationContext,
    ) -> Vec<PatternRecommendation> {
        let mut recommendations = Vec::new();
        
        // Parse intent keywords
        let keywords = self.extract_keywords(intent);
        
        // Find patterns matching keywords
        let mut pattern_scores: HashMap<String, f32> = HashMap::new();
        
        for keyword in &keywords {
            for pattern_id in self.index.search_by_keyword(keyword) {
                *pattern_scores.entry(pattern_id.clone()).or_insert(0.0) += 1.0;
            }
        }
        
        // Boost by domain if specified
        if let Some(domain) = &context.domain {
            for pattern_id in self.index.search_by_domain(domain) {
                *pattern_scores.entry(pattern_id.clone()).or_insert(0.0) += 2.0;
            }
        }
        
        // Sort by score
        let mut scored_patterns: Vec<_> = pattern_scores.into_iter().collect();
        scored_patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Create recommendations
        for (pattern_id, score) in scored_patterns.into_iter().take(10) {
            recommendations.push(PatternRecommendation {
                pattern_id,
                relevance_score: score / keywords.len() as f32,
                reason: "Keyword match".to_string(),
                confidence: 0.8,
            });
        }
        
        recommendations
    }
    
    /// Recommend similar patterns
    pub fn recommend_similar(
        &self,
        pattern_id: &str,
        n: usize,
    ) -> Vec<PatternRecommendation> {
        let mut recommendations = Vec::new();
        
        // Find patterns with highest similarity
        let mut similarities: Vec<_> = self.similarity
            .iter()
            .filter_map(|((p1, p2), score)| {
                if p1 == pattern_id {
                    Some((p2.clone(), *score))
                } else if p2 == pattern_id {
                    Some((p1.clone(), *score))
                } else {
                    None
                }
            })
            .collect();
        
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        for (similar_id, score) in similarities.into_iter().take(n) {
            recommendations.push(PatternRecommendation {
                pattern_id: similar_id,
                relevance_score: score,
                reason: "Similar pattern".to_string(),
                confidence: score,
            });
        }
        
        recommendations
    }
    
    /// Extract keywords from intent
    fn extract_keywords(&self, intent: &str) -> Vec<String> {
        // Simple keyword extraction - would use NLP in real implementation
        intent.to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .filter(|w| !STOP_WORDS.contains(w))
            .map(String::from)
            .collect()
    }
    
    /// Compute pattern similarity
    pub fn compute_similarity(&mut self, patterns: &[(&String, &EnhancedMetadata)]) {
        for i in 0..patterns.len() {
            for j in i+1..patterns.len() {
                let (id1, meta1) = &patterns[i];
                let (id2, meta2) = &patterns[j];
                
                let similarity = self.calculate_similarity(meta1, meta2);
                self.similarity.insert(((*id1).clone(), (*id2).clone()), similarity);
            }
        }
    }
    
    /// Calculate similarity between patterns
    fn calculate_similarity(&self, meta1: &EnhancedMetadata, meta2: &EnhancedMetadata) -> f32 {
        let mut score = 0.0;
        let mut factors = 0;
        
        // Domain similarity
        let domain_overlap = meta1.semantic.domain.iter()
            .filter(|d| meta2.semantic.domain.contains(d))
            .count() as f32;
        let domain_union = (meta1.semantic.domain.len() + meta2.semantic.domain.len()) as f32 
            - domain_overlap;
        if domain_union > 0.0 {
            score += domain_overlap / domain_union;
            factors += 1;
        }
        
        // Tag similarity
        let tag_overlap = meta1.base.tags.iter()
            .filter(|t| meta2.base.tags.contains(t))
            .count() as f32;
        let tag_union = (meta1.base.tags.len() + meta2.base.tags.len()) as f32 - tag_overlap;
        if tag_union > 0.0 {
            score += tag_overlap / tag_union;
            factors += 1;
        }
        
        // Category match
        if meta1.base.tags == meta2.base.tags {
            score += 1.0;
            factors += 1;
        }
        
        if factors > 0 {
            score / factors as f32
        } else {
            0.0
        }
    }
}

/// Recommendation context
#[derive(Debug, Clone)]
pub struct RecommendationContext {
    /// Domain preference
    pub domain: Option<String>,
    
    /// Performance requirements
    pub performance_critical: bool,
    
    /// Safety requirements
    pub safety_critical: bool,
    
    /// Preferred complexity
    pub complexity_preference: ComplexityPreference,
}

#[derive(Debug, Clone)]
pub enum ComplexityPreference {
    Simple,
    Moderate,
    Complex,
    Any,
}

/// Pattern recommendation
#[derive(Debug, Clone)]
pub struct PatternRecommendation {
    /// Recommended pattern ID
    pub pattern_id: String,
    
    /// Relevance score (0.0 - 1.0)
    pub relevance_score: f32,
    
    /// Reason for recommendation
    pub reason: String,
    
    /// Confidence in recommendation
    pub confidence: f32,
}

// Stop words for keyword extraction
const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "with", "by", "from", "up", "about", "into", "through", "during",
    "before", "after", "above", "below", "between", "under", "over",
];

/// Create default enhanced metadata
pub fn create_enhanced_metadata(base: PatternMetadata) -> EnhancedMetadata {
    EnhancedMetadata {
        base,
        semantic: SemanticAnnotations {
            domain: vec![],
            solves: vec![],
            use_cases: vec![],
            alternatives: vec![],
            relationships: vec![],
            descriptions: MultilingualDescriptions {
                descriptions: HashMap::new(),
            },
        },
        usage: UsageStatistics {
            use_count: 0,
            success_rate: 1.0,
            avg_generation_time_ms: 0,
            common_parameters: HashMap::new(),
            common_errors: vec![],
        },
        quality: QualityMetrics {
            correctness_score: 1.0,
            performance_score: 1.0,
            maintainability_score: 1.0,
            test_coverage: 1.0,
            verification_confidence: 1.0,
            community_rating: None,
            expert_reviewed: false,
        },
        learning_hints: LearningHints {
            prerequisites: vec![],
            objectives: vec![],
            misconceptions: vec![],
            best_practices: vec![],
            anti_patterns: vec![],
            complexity_progression: vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metadata_index() {
        let mut index = MetadataIndex::new();
        
        let metadata = create_enhanced_metadata(PatternMetadata {
            tags: vec!["test".to_string()],
            requires: vec![],
            provides: vec![],
            author: "test".to_string(),
            version: "1.0.0".to_string(),
            stability: StabilityLevel::Stable,
            complexity: ComplexityEstimate {
                time: "O(1)".to_string(),
                space: "O(1)".to_string(),
                io: None,
            },
            safety: SafetyGuarantees {
                memory_safe: true,
                thread_safe: true,
                exception_safe: ExceptionSafety::NoThrow,
                resource_safe: true,
            },
        });
        
        index.index_pattern("test_pattern", &metadata);
        
        // Check that pattern was indexed
        assert_eq!(index.quality_rankings.len(), 1);
        assert_eq!(index.usage_rankings.len(), 1);
    }
}