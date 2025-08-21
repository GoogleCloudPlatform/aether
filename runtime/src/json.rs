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
//! Provides basic JSON object and array creation/manipulation

use std::ffi::{c_char, c_int, CStr};
use std::ptr;
use std::collections::HashMap;

/// Simple JSON value representation
#[derive(Debug, Clone)]
enum JsonValue {
    String(String),
    Number(i32),
    Object(HashMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    Null,
}

impl JsonValue {
    fn to_string(&self) -> String {
        match self {
            JsonValue::String(s) => format!("\"{}\"", s.replace("\"", "\\\"")),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Object(obj) => {
                let pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k.replace("\"", "\\\""), v.to_string()))
                    .collect();
                format!("{{{}}}", pairs.join(","))
            },
            JsonValue::Array(arr) => {
                let items: Vec<String> = arr.iter()
                    .map(|v| v.to_string())
                    .collect();
                format!("[{}]", items.join(","))
            },
            JsonValue::Null => "null".to_string(),
        }
    }
}

/// Create empty JSON object
#[no_mangle]
pub unsafe extern "C" fn create_object() -> *mut c_char {
    let empty_obj = JsonValue::Object(HashMap::new());
    let json_str = format!("{}\0", empty_obj.to_string());
    
    let len = json_str.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(json_str.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Create empty JSON array
#[no_mangle]
pub unsafe extern "C" fn create_array() -> *mut c_char {
    let empty_array = JsonValue::Array(Vec::new());
    let json_str = format!("{}\0", empty_array.to_string());
    
    let len = json_str.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(json_str.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Set field in JSON object (simplified version)
#[no_mangle]
pub unsafe extern "C" fn json_set_field(json_obj: *const c_char, field: *const c_char, value: *const c_char) -> *mut c_char {
    if json_obj.is_null() || field.is_null() || value.is_null() {
        return ptr::null_mut();
    }
    
    let field_str = match CStr::from_ptr(field).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let value_str = match CStr::from_ptr(value).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    // For simplicity, parse object as string and reconstruct
    let obj_str = match CStr::from_ptr(json_obj).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    // Simple JSON object manipulation (not a full parser)
    let result = if obj_str == "{}" {
        format!("{{\"{}\":\"{}\"}}", field_str, value_str)
    } else {
        // Remove closing brace and add field
        let without_close = obj_str.trim_end_matches('}');
        if without_close.ends_with('{') {
            format!("{{\"{}\":\"{}\"}}", field_str, value_str)
        } else {
            format!("{},\"{}\":\"{}\"}}", without_close, field_str, value_str)
        }
    };
    
    let result_with_null = format!("{}\0", result);
    let len = result_with_null.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result_with_null.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Convert JSON to string (passthrough since it's already a string)
#[no_mangle]
pub unsafe extern "C" fn stringify_json(json: *const c_char) -> *mut c_char {
    if json.is_null() {
        return ptr::null_mut();
    }
    
    let json_str = match CStr::from_ptr(json).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let result = format!("{}\0", json_str);
    let len = result.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Push item to JSON array (simplified)
#[no_mangle]
pub unsafe extern "C" fn json_array_push(json_array: *const c_char, item: *const c_char) -> *mut c_char {
    if json_array.is_null() || item.is_null() {
        return ptr::null_mut();
    }
    
    let array_str = match CStr::from_ptr(json_array).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let item_str = match CStr::from_ptr(item).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let result = if array_str == "[]" {
        format!("[{}]", item_str)
    } else {
        let without_close = array_str.trim_end_matches(']');
        if without_close.ends_with('[') {
            format!("[{}]", item_str)
        } else {
            format!("{},{}]", without_close, item_str)
        }
    };
    
    let result_with_null = format!("{}\0", result);
    let len = result_with_null.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result_with_null.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Get array length (simplified)
#[no_mangle]
pub unsafe extern "C" fn json_array_length(json_array: *const c_char) -> c_int {
    if json_array.is_null() {
        return 0;
    }
    
    let array_str = match CStr::from_ptr(json_array).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    if array_str == "[]" {
        return 0;
    }
    
    // Simple count of commas + 1 (not a full parser)
    let content = array_str.trim_start_matches('[').trim_end_matches(']');
    if content.is_empty() {
        0
    } else {
        content.matches(',').count() as c_int + 1
    }
}

/// Convert string to JSON string value
#[no_mangle]
pub unsafe extern "C" fn from_string(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    
    let string_val = match CStr::from_ptr(s).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let result = format!("\"{}\"\0", string_val.replace("\"", "\\\""));
    let len = result.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Convert integer to JSON number value
#[no_mangle]
pub unsafe extern "C" fn from_integer(n: c_int) -> *mut c_char {
    let result = format!("{}\0", n);
    let len = result.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}