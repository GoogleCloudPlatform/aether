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

//! JSON manipulation functions for AetherScript
//! 
//! Provides basic JSON object and array creation/manipulation using serde_json

use std::ffi::{c_char, c_int, CStr};
use std::ptr;
use serde_json::Value;

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_parse_json() {
        unsafe {
            let json = CString::new(r#"{"x": 42}"#).unwrap();
            let result = parse_json(json.as_ptr());
            assert!(!result.is_null());
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            println!("Parsed result: {}", result_str);
            assert!(result_str.contains("42"));
            crate::memory_alloc::aether_safe_free(result as *mut std::ffi::c_void);
        }
    }

    #[test]
    fn test_json_get_field() {
        unsafe {
            let json = CString::new(r#"{"x":42}"#).unwrap();
            let field = CString::new("x").unwrap();
            let result = json_get_field(json.as_ptr(), field.as_ptr());
            assert!(!result.is_null());
            let result_str = CStr::from_ptr(result).to_str().unwrap();
            println!("Field x value: {}", result_str);
            assert_eq!(result_str, "42");
            crate::memory_alloc::aether_safe_free(result as *mut std::ffi::c_void);
        }
    }
}

// Helper to convert C string to Rust String
unsafe fn from_c_str(s: *const c_char) -> String {
    if s.is_null() {
        return String::new();
    }
    CStr::from_ptr(s).to_string_lossy().into_owned()
}

// Helper to convert Rust String to C string (allocated)
unsafe fn to_c_str(s: String) -> *mut c_char {
    let result = format!("{}\0", s);
    let len = result.len();
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Parse JSON string - verifies validity and returns minified string
#[no_mangle]
pub unsafe extern "C" fn parse_json(json_string: *const c_char) -> *mut c_char {
    let s = from_c_str(json_string);
    match serde_json::from_str::<Value>(&s) {
        Ok(v) => to_c_str(v.to_string()),
        Err(_) => ptr::null_mut(), // Return null on parse error
    }
}

/// Convert JSON value to string representation
#[no_mangle]
pub unsafe extern "C" fn stringify_json(json_value: *const c_char) -> *mut c_char {
    // Since JsonValue is already a string in our representation, this might just be a copy
    // But we should validate it's valid JSON
    let s = from_c_str(json_value);
    match serde_json::from_str::<Value>(&s) {
        Ok(v) => to_c_str(v.to_string()),
        Err(_) => ptr::null_mut(),
    }
}

/// Get field from JSON object
#[no_mangle]
pub unsafe extern "C" fn json_get_field(json_value: *const c_char, field_name: *const c_char) -> *mut c_char {
    let json_str = from_c_str(json_value);
    let field = from_c_str(field_name);
    
    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&json_str) {
        if let Some(val) = map.get(&field) {
            return to_c_str(val.to_string());
        }
    }
    
    // Return "null" string if field not found or not an object
    to_c_str("null".to_string())
}

/// Get length of JSON array
#[no_mangle]
pub unsafe extern "C" fn json_array_length(json_array: *const c_char) -> c_int {
    let json_str = from_c_str(json_array);
    
    if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(&json_str) {
        return arr.len() as c_int;
    }
    
    0
}

/// Get element from JSON array at index
#[no_mangle]
pub unsafe extern "C" fn json_array_get(json_array: *const c_char, index: c_int) -> *mut c_char {
    let json_str = from_c_str(json_array);
    
    if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(&json_str) {
        if index >= 0 && (index as usize) < arr.len() {
            return to_c_str(arr[index as usize].to_string());
        }
    }
    
    to_c_str("null".to_string())
}

/// Create a JSON string value from a raw string
#[no_mangle]
pub unsafe extern "C" fn from_string(value: *const c_char) -> *mut c_char {
    let s = from_c_str(value);
    let v = Value::String(s);
    to_c_str(v.to_string())
}

/// Extract string content from JSON string value, or stringify other types
#[no_mangle]
pub unsafe extern "C" fn json_to_string(json_value: *const c_char) -> *mut c_char {
    let json_str = from_c_str(json_value);
    
    if let Ok(v) = serde_json::from_str::<Value>(&json_str) {
        match v {
            Value::String(s) => to_c_str(s),
            _ => to_c_str(v.to_string()),
        }
    } else {
        to_c_str(String::new())
    }
}

/// Extract integer from JSON number value
#[no_mangle]
pub unsafe extern "C" fn to_integer(json_value: *const c_char) -> c_int {
    let json_str = from_c_str(json_value);
    
    if let Ok(v) = serde_json::from_str::<Value>(&json_str) {
        if let Some(i) = v.as_i64() {
            return i as c_int;
        }
        // Try parsing string as int if it's a string
        if let Some(s) = v.as_str() {
            if let Ok(i) = s.parse::<c_int>() {
                return i;
            }
        }
    }
    
    0
}

// Legacy functions kept for compatibility but implemented with serde

#[no_mangle]
pub unsafe extern "C" fn create_object() -> *mut c_char {
    to_c_str("{}".to_string())
}

#[no_mangle]
pub unsafe extern "C" fn create_array() -> *mut c_char {
    to_c_str("[]".to_string())
}

#[no_mangle]
pub unsafe extern "C" fn json_set_field(json_obj: *const c_char, field: *const c_char, value: *const c_char) -> *mut c_char {
    let obj_str = from_c_str(json_obj);
    let field_str = from_c_str(field);
    let value_str = from_c_str(value);
    
    let mut v: Value = serde_json::from_str(&obj_str).unwrap_or(Value::Object(serde_json::Map::new()));
    let val: Value = serde_json::from_str(&value_str).unwrap_or(Value::Null);
    
    if let Value::Object(ref mut map) = v {
        map.insert(field_str, val);
    }
    
    to_c_str(v.to_string())
}

#[no_mangle]
pub unsafe extern "C" fn json_array_push(json_array: *const c_char, item: *const c_char) -> *mut c_char {
    let arr_str = from_c_str(json_array);
    let item_str = from_c_str(item);

    let mut v: Value = serde_json::from_str(&arr_str).unwrap_or(Value::Array(Vec::new()));
    let item_val: Value = serde_json::from_str(&item_str).unwrap_or(Value::Null);

    if let Value::Array(ref mut vec) = v {
        vec.push(item_val);
    }

    to_c_str(v.to_string())
}

/// Get all keys from a JSON object as a JSON array of strings
#[no_mangle]
pub unsafe extern "C" fn json_object_keys(json_value: *const c_char) -> *mut c_char {
    let json_str = from_c_str(json_value);

    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(&json_str) {
        let keys: Vec<Value> = map.keys()
            .map(|k| Value::String(k.clone()))
            .collect();
        return to_c_str(Value::Array(keys).to_string());
    }

    // Return empty array if not an object
    to_c_str("[]".to_string())
}

/// Check if JSON value is an object
#[no_mangle]
pub unsafe extern "C" fn json_is_object(json_value: *const c_char) -> c_int {
    let json_str = from_c_str(json_value);

    if let Ok(Value::Object(_)) = serde_json::from_str::<Value>(&json_str) {
        return 1;
    }
    0
}

/// Check if JSON value is an array
#[no_mangle]
pub unsafe extern "C" fn json_is_array(json_value: *const c_char) -> c_int {
    let json_str = from_c_str(json_value);

    if let Ok(Value::Array(_)) = serde_json::from_str::<Value>(&json_str) {
        return 1;
    }
    0
}

/// Check if JSON value is a string
#[no_mangle]
pub unsafe extern "C" fn json_is_string(json_value: *const c_char) -> c_int {
    let json_str = from_c_str(json_value);

    if let Ok(Value::String(_)) = serde_json::from_str::<Value>(&json_str) {
        return 1;
    }
    0
}

/// Check if JSON value is a number
#[no_mangle]
pub unsafe extern "C" fn json_is_number(json_value: *const c_char) -> c_int {
    let json_str = from_c_str(json_value);

    if let Ok(Value::Number(_)) = serde_json::from_str::<Value>(&json_str) {
        return 1;
    }
    0
}
