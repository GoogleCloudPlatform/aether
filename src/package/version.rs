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

//! Version management for AetherScript packages
//!
//! Implements semantic versioning (SemVer) with support for version requirements,
//! constraints, and compatibility checking.

use crate::error::SemanticError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Semantic version (SemVer)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    /// Major version number
    pub major: u64,

    /// Minor version number
    pub minor: u64,

    /// Patch version number
    pub patch: u64,

    /// Pre-release identifier
    pub pre: Vec<Identifier>,

    /// Build metadata
    pub build: Vec<Identifier>,
}

/// Version identifier component
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Identifier {
    /// Numeric identifier
    Numeric(u64),

    /// Alphanumeric identifier
    AlphaNumeric(String),
}

/// Version requirement specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRequirement {
    /// Set of version predicates
    predicates: Vec<Predicate>,
}

/// Version predicate
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Predicate {
    /// Comparison operator
    pub op: Op,

    /// Version to compare against
    pub version: Version,

    /// Whether to allow pre-release versions
    pub allow_prerelease: bool,
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Exact version (=)
    Exact,

    /// Greater than (>)
    Greater,

    /// Greater than or equal (>=)
    GreaterEq,

    /// Less than (<)
    Less,

    /// Less than or equal (<=)
    LessEq,

    /// Tilde requirement (~)
    Tilde,

    /// Caret requirement (^)
    Caret,

    /// Wildcard requirement (*)
    Wildcard,
}

/// Version comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionDiff {
    /// Major version difference
    Major,

    /// Minor version difference
    Minor,

    /// Patch version difference
    Patch,

    /// Pre-release difference
    Prerelease,

    /// Build metadata difference
    Build,
}

impl Version {
    /// Create a new version
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: Vec::new(),
            build: Vec::new(),
        }
    }

    /// Create a version with pre-release
    pub fn with_prerelease(major: u64, minor: u64, patch: u64, pre: Vec<Identifier>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre,
            build: Vec::new(),
        }
    }

    /// Create a version with build metadata
    pub fn with_build(major: u64, minor: u64, patch: u64, build: Vec<Identifier>) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: Vec::new(),
            build,
        }
    }

    /// Check if this is a pre-release version
    pub fn is_prerelease(&self) -> bool {
        !self.pre.is_empty()
    }

    /// Check if this is a stable version
    pub fn is_stable(&self) -> bool {
        self.pre.is_empty()
    }

    /// Compare versions ignoring build metadata
    pub fn cmp_precedence(&self, other: &Version) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            result => return result,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            result => return result,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            result => return result,
        }

        // Pre-release versions have lower precedence than normal versions
        match (self.pre.is_empty(), other.pre.is_empty()) {
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (true, true) => Ordering::Equal,
            (false, false) => {
                // Compare pre-release identifiers
                for (a, b) in self.pre.iter().zip(other.pre.iter()) {
                    match a.cmp(b) {
                        Ordering::Equal => continue,
                        result => return result,
                    }
                }
                self.pre.len().cmp(&other.pre.len())
            }
        }
    }

    /// Get the difference type between two versions
    pub fn diff(&self, other: &Version) -> Option<VersionDiff> {
        if self.major != other.major {
            Some(VersionDiff::Major)
        } else if self.minor != other.minor {
            Some(VersionDiff::Minor)
        } else if self.patch != other.patch {
            Some(VersionDiff::Patch)
        } else if self.pre != other.pre {
            Some(VersionDiff::Prerelease)
        } else if self.build != other.build {
            Some(VersionDiff::Build)
        } else {
            None
        }
    }

    /// Check if this version is compatible with another (same major version)
    pub fn is_compatible(&self, other: &Version) -> bool {
        self.major == other.major && self.major != 0
    }

    /// Increment major version
    pub fn increment_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
        self.pre.clear();
        self.build.clear();
    }

    /// Increment minor version
    pub fn increment_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
        self.pre.clear();
        self.build.clear();
    }

    /// Increment patch version
    pub fn increment_patch(&mut self) {
        self.patch += 1;
        self.pre.clear();
        self.build.clear();
    }
}

impl Identifier {
    /// Create a numeric identifier
    pub fn numeric(value: u64) -> Self {
        Identifier::Numeric(value)
    }

    /// Create an alphanumeric identifier
    pub fn alpha(value: String) -> Self {
        Identifier::AlphaNumeric(value)
    }

    /// Check if identifier is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(self, Identifier::Numeric(_))
    }

    /// Get numeric value if this is a numeric identifier
    pub fn as_numeric(&self) -> Option<u64> {
        match self {
            Identifier::Numeric(n) => Some(*n),
            _ => None,
        }
    }

    /// Get string value
    pub fn as_str(&self) -> String {
        match self {
            Identifier::Numeric(n) => n.to_string(),
            Identifier::AlphaNumeric(s) => s.clone(),
        }
    }
}

impl VersionRequirement {
    /// Create an exact version requirement
    pub fn exact(version: &Version) -> Self {
        Self {
            predicates: vec![Predicate {
                op: Op::Exact,
                version: version.clone(),
                allow_prerelease: version.is_prerelease(),
            }],
        }
    }

    /// Create a caret requirement (^1.2.3)
    pub fn caret(version: &Version) -> Self {
        Self {
            predicates: vec![Predicate {
                op: Op::Caret,
                version: version.clone(),
                allow_prerelease: version.is_prerelease(),
            }],
        }
    }

    /// Create a tilde requirement (~1.2.3)
    pub fn tilde(version: &Version) -> Self {
        Self {
            predicates: vec![Predicate {
                op: Op::Tilde,
                version: version.clone(),
                allow_prerelease: version.is_prerelease(),
            }],
        }
    }

    /// Create a wildcard requirement (*)
    pub fn any() -> Self {
        Self {
            predicates: vec![Predicate {
                op: Op::Wildcard,
                version: Version::new(0, 0, 0),
                allow_prerelease: true,
            }],
        }
    }

    /// Create a range requirement (>=1.0.0, <2.0.0)
    pub fn range(min: &Version, max: &Version, include_max: bool) -> Self {
        let max_op = if include_max { Op::LessEq } else { Op::Less };

        Self {
            predicates: vec![
                Predicate {
                    op: Op::GreaterEq,
                    version: min.clone(),
                    allow_prerelease: min.is_prerelease(),
                },
                Predicate {
                    op: max_op,
                    version: max.clone(),
                    allow_prerelease: false,
                },
            ],
        }
    }

    /// Check if this requirement is satisfied by a version
    pub fn matches(&self, version: &Version) -> bool {
        if self.predicates.is_empty() {
            return true;
        }

        // All predicates must be satisfied
        for predicate in &self.predicates {
            if !predicate.matches(version) {
                return false;
            }
        }

        true
    }

    /// Check if this is an "any" requirement
    pub fn is_any(&self) -> bool {
        self.predicates.len() == 1 && self.predicates[0].op == Op::Wildcard
    }

    /// Get the most permissive version that satisfies this requirement
    pub fn max_version(&self) -> Option<Version> {
        // This is a simplified implementation
        for predicate in &self.predicates {
            match predicate.op {
                Op::Exact => return Some(predicate.version.clone()),
                Op::LessEq => return Some(predicate.version.clone()),
                Op::Less => {
                    let mut v = predicate.version.clone();
                    if v.patch > 0 {
                        v.patch -= 1;
                    } else if v.minor > 0 {
                        v.minor -= 1;
                        v.patch = u64::MAX;
                    } else if v.major > 0 {
                        v.major -= 1;
                        v.minor = u64::MAX;
                        v.patch = u64::MAX;
                    }
                    return Some(v);
                }
                _ => continue,
            }
        }
        None
    }
}

impl Predicate {
    /// Check if this predicate matches a version
    pub fn matches(&self, version: &Version) -> bool {
        // Check pre-release allowance
        if version.is_prerelease() && !self.allow_prerelease {
            // Only allow pre-release if the requirement version is also pre-release
            // and they have the same major.minor.patch
            if !(self.version.is_prerelease()
                && version.major == self.version.major
                && version.minor == self.version.minor
                && version.patch == self.version.patch)
            {
                return false;
            }
        }

        match self.op {
            Op::Exact => version.cmp_precedence(&self.version) == Ordering::Equal,
            Op::Greater => version.cmp_precedence(&self.version) == Ordering::Greater,
            Op::GreaterEq => version.cmp_precedence(&self.version) != Ordering::Less,
            Op::Less => version.cmp_precedence(&self.version) == Ordering::Less,
            Op::LessEq => version.cmp_precedence(&self.version) != Ordering::Greater,
            Op::Wildcard => true,
            Op::Tilde => self.matches_tilde(version),
            Op::Caret => self.matches_caret(version),
        }
    }

    fn matches_tilde(&self, version: &Version) -> bool {
        // ~1.2.3 := >=1.2.3, <1.3.0 (reasonably close to 1.2.3)
        // ~1.2 := >=1.2.0, <1.3.0
        // ~1 := >=1.0.0, <2.0.0
        // ~0.2.3 := >=0.2.3, <0.3.0
        // ~0.2 := >=0.2.0, <0.3.0
        // ~0 := >=0.0.0, <1.0.0

        if version.major != self.version.major {
            return false;
        }

        if version.minor != self.version.minor {
            return false;
        }

        version.cmp_precedence(&self.version) != Ordering::Less
    }

    fn matches_caret(&self, version: &Version) -> bool {
        // ^1.2.3 := >=1.2.3, <2.0.0 (compatible within same major version)
        // ^0.2.3 := >=0.2.3, <0.3.0 (compatible within same minor version if major is 0)
        // ^0.0.3 := >=0.0.3, <0.0.4 (compatible within same patch version if major and minor are 0)

        if version.cmp_precedence(&self.version) == Ordering::Less {
            return false;
        }

        if self.version.major > 0 {
            version.major == self.version.major
        } else if self.version.minor > 0 {
            version.major == 0 && version.minor == self.version.minor
        } else {
            version.major == 0 && version.minor == 0 && version.patch == self.version.patch
        }
    }
}

impl FromStr for Version {
    type Err = SemanticError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('+');
        let version_part = parts.next().unwrap();
        let build_part = parts.next();

        let mut version_build_parts = version_part.split('-');
        let version_core = version_build_parts.next().unwrap();
        let pre_parts: Vec<&str> = version_build_parts.collect();

        // Parse core version (major.minor.patch)
        let core_parts: Vec<&str> = version_core.split('.').collect();
        if core_parts.len() != 3 {
            return Err(SemanticError::Internal {
                message: format!("Invalid version format: {}", s),
            });
        }

        let major = core_parts[0]
            .parse::<u64>()
            .map_err(|_| SemanticError::Internal {
                message: format!("Invalid major version: {}", core_parts[0]),
            })?;

        let minor = core_parts[1]
            .parse::<u64>()
            .map_err(|_| SemanticError::Internal {
                message: format!("Invalid minor version: {}", core_parts[1]),
            })?;

        let patch = core_parts[2]
            .parse::<u64>()
            .map_err(|_| SemanticError::Internal {
                message: format!("Invalid patch version: {}", core_parts[2]),
            })?;

        // Parse pre-release identifiers
        let mut pre = Vec::new();
        for part in pre_parts {
            for identifier in part.split('.') {
                if identifier.is_empty() {
                    continue;
                }

                if let Ok(num) = identifier.parse::<u64>() {
                    pre.push(Identifier::Numeric(num));
                } else {
                    pre.push(Identifier::AlphaNumeric(identifier.to_string()));
                }
            }
        }

        // Parse build metadata
        let mut build = Vec::new();
        if let Some(build_str) = build_part {
            for identifier in build_str.split('.') {
                if identifier.is_empty() {
                    continue;
                }

                if let Ok(num) = identifier.parse::<u64>() {
                    build.push(Identifier::Numeric(num));
                } else {
                    build.push(Identifier::AlphaNumeric(identifier.to_string()));
                }
            }
        }

        Ok(Version {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }
}

impl FromStr for VersionRequirement {
    type Err = SemanticError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() || s == "*" {
            return Ok(VersionRequirement::any());
        }

        let s = s.trim();

        // Handle different operators
        if s.starts_with("^") {
            let version = Version::from_str(&s[1..])?;
            Ok(VersionRequirement::caret(&version))
        } else if s.starts_with("~") {
            let version = Version::from_str(&s[1..])?;
            Ok(VersionRequirement::tilde(&version))
        } else if s.starts_with(">=") {
            let version = Version::from_str(&s[2..])?;
            Ok(VersionRequirement {
                predicates: vec![Predicate {
                    op: Op::GreaterEq,
                    version,
                    allow_prerelease: false,
                }],
            })
        } else if s.starts_with("<=") {
            let version = Version::from_str(&s[2..])?;
            Ok(VersionRequirement {
                predicates: vec![Predicate {
                    op: Op::LessEq,
                    version,
                    allow_prerelease: false,
                }],
            })
        } else if s.starts_with(">") {
            let version = Version::from_str(&s[1..])?;
            Ok(VersionRequirement {
                predicates: vec![Predicate {
                    op: Op::Greater,
                    version,
                    allow_prerelease: false,
                }],
            })
        } else if s.starts_with("<") {
            let version = Version::from_str(&s[1..])?;
            Ok(VersionRequirement {
                predicates: vec![Predicate {
                    op: Op::Less,
                    version,
                    allow_prerelease: false,
                }],
            })
        } else if s.starts_with("=") {
            let version = Version::from_str(&s[1..])?;
            Ok(VersionRequirement::exact(&version))
        } else {
            // Default to exact match
            let version = Version::from_str(s)?;
            Ok(VersionRequirement::exact(&version))
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if !self.pre.is_empty() {
            write!(f, "-")?;
            for (i, identifier) in self.pre.iter().enumerate() {
                if i > 0 {
                    write!(f, ".")?;
                }
                write!(f, "{}", identifier)?;
            }
        }

        if !self.build.is_empty() {
            write!(f, "+")?;
            for (i, identifier) in self.build.iter().enumerate() {
                if i > 0 {
                    write!(f, ".")?;
                }
                write!(f, "{}", identifier)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Identifier::Numeric(n) => write!(f, "{}", n),
            Identifier::AlphaNumeric(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Display for VersionRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.predicates.is_empty() {
            return write!(f, "*");
        }

        for (i, predicate) in self.predicates.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", predicate)?;
        }

        Ok(())
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.op {
            Op::Exact => write!(f, "={}", self.version),
            Op::Greater => write!(f, ">{}", self.version),
            Op::GreaterEq => write!(f, ">={}", self.version),
            Op::Less => write!(f, "<{}", self.version),
            Op::LessEq => write!(f, "<={}", self.version),
            Op::Tilde => write!(f, "~{}", self.version),
            Op::Caret => write!(f, "^{}", self.version),
            Op::Wildcard => write!(f, "*"),
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_precedence(other)
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Identifier::Numeric(a), Identifier::Numeric(b)) => a.cmp(b),
            (Identifier::Numeric(_), Identifier::AlphaNumeric(_)) => Ordering::Less,
            (Identifier::AlphaNumeric(_), Identifier::Numeric(_)) => Ordering::Greater,
            (Identifier::AlphaNumeric(a), Identifier::AlphaNumeric(b)) => a.cmp(b),
        }
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for VersionRequirement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for VersionRequirement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        VersionRequirement::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = Version::from_str("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre.is_empty());
        assert!(v.build.is_empty());
    }

    #[test]
    fn test_version_with_prerelease() {
        let v = Version::from_str("1.2.3-alpha.1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.pre.len(), 2);
        assert!(v.is_prerelease());
    }

    #[test]
    fn test_version_with_build() {
        let v = Version::from_str("1.2.3+build.1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.build.len(), 2);
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 2, 4);
        let v3 = Version::new(1, 3, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v1 < v3);
    }

    #[test]
    fn test_prerelease_precedence() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::from_str("1.0.0-alpha").unwrap();

        assert!(v2 < v1);
        assert!(v2.is_prerelease());
        assert!(!v1.is_prerelease());
    }

    #[test]
    fn test_version_requirement_exact() {
        let v = Version::new(1, 2, 3);
        let req = VersionRequirement::exact(&v);

        assert!(req.matches(&v));
        assert!(!req.matches(&Version::new(1, 2, 4)));
    }

    #[test]
    fn test_version_requirement_caret() {
        let v = Version::new(1, 2, 3);
        let req = VersionRequirement::caret(&v);

        assert!(req.matches(&Version::new(1, 2, 3)));
        assert!(req.matches(&Version::new(1, 2, 4)));
        assert!(req.matches(&Version::new(1, 3, 0)));
        assert!(!req.matches(&Version::new(2, 0, 0)));
        assert!(!req.matches(&Version::new(1, 2, 2)));
    }

    #[test]
    fn test_version_requirement_tilde() {
        let v = Version::new(1, 2, 3);
        let req = VersionRequirement::tilde(&v);

        assert!(req.matches(&Version::new(1, 2, 3)));
        assert!(req.matches(&Version::new(1, 2, 4)));
        assert!(!req.matches(&Version::new(1, 3, 0)));
        assert!(!req.matches(&Version::new(1, 2, 2)));
    }

    #[test]
    fn test_version_requirement_parsing() {
        let req = VersionRequirement::from_str("^1.2.3").unwrap();
        assert!(req.matches(&Version::new(1, 2, 3)));
        assert!(req.matches(&Version::new(1, 3, 0)));
        assert!(!req.matches(&Version::new(2, 0, 0)));

        let req = VersionRequirement::from_str("~1.2.3").unwrap();
        assert!(req.matches(&Version::new(1, 2, 3)));
        assert!(req.matches(&Version::new(1, 2, 4)));
        assert!(!req.matches(&Version::new(1, 3, 0)));

        let req = VersionRequirement::from_str(">=1.2.0").unwrap();
        assert!(req.matches(&Version::new(1, 2, 0)));
        assert!(req.matches(&Version::new(1, 3, 0)));
        assert!(req.matches(&Version::new(2, 0, 0)));
        assert!(!req.matches(&Version::new(1, 1, 9)));
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");

        let v = Version::from_str("1.2.3-alpha.1+build.2").unwrap();
        assert_eq!(v.to_string(), "1.2.3-alpha.1+build.2");
    }

    #[test]
    fn test_version_compatibility() {
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 3, 0);
        let v3 = Version::new(2, 0, 0);
        let v4 = Version::new(0, 1, 0);

        assert!(v1.is_compatible(&v2));
        assert!(!v1.is_compatible(&v3));
        assert!(!v1.is_compatible(&v4));
    }

    #[test]
    fn test_version_increment() {
        let mut v = Version::new(1, 2, 3);

        v.increment_patch();
        assert_eq!(v, Version::new(1, 2, 4));

        v.increment_minor();
        assert_eq!(v, Version::new(1, 3, 0));

        v.increment_major();
        assert_eq!(v, Version::new(2, 0, 0));
    }
}
