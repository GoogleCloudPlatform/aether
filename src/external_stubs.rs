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

//! Stubs for external dependencies that are not available
//! 
//! This module provides minimal implementations for external crates
//! to allow compilation to succeed.

/// URL encoding stub
pub mod urlencoding {
    pub fn encode(s: &str) -> String {
        // Simple URL encoding - replace spaces with %20
        s.replace(' ', "%20")
    }
    
    pub fn decode(s: &str) -> Result<String, &'static str> {
        // Simple URL decoding - replace %20 with spaces
        Ok(s.replace("%20", " "))
    }
}

/// TOML parsing stub
pub mod toml {
    use serde::{Deserialize, Serialize};
    use std::error::Error;
    use std::fmt;
    
    #[derive(Debug)]
    pub struct TomlError {
        message: String,
    }
    
    impl fmt::Display for TomlError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "TOML error: {}", self.message)
        }
    }
    
    impl Error for TomlError {}
    
    pub fn from_str<T>(_s: &str) -> Result<T, TomlError> 
    where
        T: for<'de> Deserialize<'de>,
    {
        Err(TomlError {
            message: "TOML parsing not implemented".to_string(),
        })
    }
    
    pub fn to_string<T>(_value: &T) -> Result<String, TomlError>
    where
        T: Serialize,
    {
        Err(TomlError {
            message: "TOML serialization not implemented".to_string(),
        })
    }
    
    pub fn to_string_pretty<T>(_value: &T) -> Result<String, TomlError>
    where
        T: Serialize,
    {
        Err(TomlError {
            message: "TOML serialization not implemented".to_string(),
        })
    }
}

/// YAML parsing stub
pub mod serde_yaml {
    use serde::{Deserialize, Serialize};
    use std::error::Error;
    use std::fmt;
    
    #[derive(Debug)]
    pub struct YamlError {
        message: String,
    }
    
    impl fmt::Display for YamlError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "YAML error: {}", self.message)
        }
    }
    
    impl Error for YamlError {}
    
    pub fn from_str<T>(_s: &str) -> Result<T, YamlError>
    where
        T: for<'de> Deserialize<'de>,
    {
        Err(YamlError {
            message: "YAML parsing not implemented".to_string(),
        })
    }
    
    pub fn to_string<T>(_value: &T) -> Result<String, YamlError>
    where
        T: Serialize,
    {
        Err(YamlError {
            message: "YAML serialization not implemented".to_string(),
        })
    }
}