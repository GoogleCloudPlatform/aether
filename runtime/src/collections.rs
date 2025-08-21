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

//! Collections runtime support for AetherScript
//! 
//! Provides runtime implementations for map and array operations

use std::collections::HashMap;
use std::os::raw::c_void;
use std::ptr;

/// Simple map structure for runtime
#[repr(C)]
pub struct AetherMap {
    // For simplicity, we'll use a HashMap internally
    // In a real implementation, this would be more sophisticated
    data: *mut HashMap<i32, i32>,
}

/// Create a new map
#[no_mangle]
pub extern "C" fn map_new() -> *mut c_void {
    let map = Box::new(AetherMap {
        data: Box::into_raw(Box::new(HashMap::new())),
    });
    Box::into_raw(map) as *mut c_void
}

/// Insert a key-value pair into the map (generic version)
#[no_mangle]
pub extern "C" fn map_insert(map: *mut c_void, key: *const c_void, value: *const c_void) {
    if map.is_null() || key.is_null() || value.is_null() {
        return;
    }
    
    // For now, we'll assume keys and values are i32
    // In a real implementation, we'd need type information
    unsafe {
        let map_ref = &mut *(map as *mut AetherMap);
        if let Some(hashmap) = map_ref.data.as_mut() {
            let key_val = *(key as *const i32);
            let value_val = *(value as *const i32);
            (*hashmap).insert(key_val, value_val);
        }
    }
}

/// Get a value from the map (generic version)
#[no_mangle]
pub extern "C" fn map_get(map: *const c_void, key: *const c_void) -> *mut c_void {
    if map.is_null() || key.is_null() {
        return std::ptr::null_mut();
    }
    
    unsafe {
        let map_ref = &*(map as *const AetherMap);
        if let Some(hashmap) = map_ref.data.as_ref() {
            let key_val = *(key as *const i32);
            if let Some(value) = (*hashmap).get(&key_val) {
                // Allocate memory for the result and return it
                let result = Box::new(*value);
                Box::into_raw(result) as *mut c_void
            } else {
                std::ptr::null_mut()
            }
        } else {
            std::ptr::null_mut()
        }
    }
}

/// Delete a map and free its memory
#[no_mangle]
pub extern "C" fn map_delete(map: *mut AetherMap) {
    if !map.is_null() {
        unsafe {
            let map_box = Box::from_raw(map);
            if !map_box.data.is_null() {
                let _ = Box::from_raw(map_box.data);
            }
        }
    }
}

/// More sophisticated map operations for std.collections

/// Create a map with size hints
#[no_mangle]
pub extern "C" fn aether_collections_map_create(key_size: usize, value_size: usize) -> *mut c_void {
    // For now, ignore size hints and create a simple map
    // In a real implementation, we'd use these for optimization
    let _ = (key_size, value_size);
    map_new() as *mut c_void
}

/// Destroy a map
#[no_mangle]
pub extern "C" fn aether_collections_map_destroy(map: *mut c_void) {
    if !map.is_null() {
        map_delete(map as *mut AetherMap);
    }
}

/// Insert with generic key/value pointers
#[no_mangle]
pub extern "C" fn aether_collections_map_insert(
    map: *mut c_void,
    key: *const c_void,
    value: *const c_void,
) -> bool {
    if map.is_null() || key.is_null() || value.is_null() {
        return false;
    }
    
    // For now, assume i32 key/value types
    unsafe {
        let key_val = *(key as *const i32);
        let value_val = *(value as *const i32);
        map_insert(map, &key_val as *const i32 as *const c_void, &value_val as *const i32 as *const c_void);
    }
    
    true
}

/// Get with generic key/value pointers
#[no_mangle]
pub extern "C" fn aether_collections_map_get(
    map: *const c_void,
    key: *const c_void,
    result: *mut c_void,
) -> bool {
    if map.is_null() || key.is_null() || result.is_null() {
        return false;
    }
    
    // For now, assume i32 key/value types
    unsafe {
        let key_val = *(key as *const i32);
        let value_ptr = map_get(map, &key_val as *const i32 as *const c_void);
        if !value_ptr.is_null() {
            *(result as *mut i32) = *(value_ptr as *const i32);
            // Free the allocated memory
            let _ = Box::from_raw(value_ptr as *mut i32);
        } else {
            *(result as *mut i32) = 0;
        }
    }
    
    true
}

/// Check if map contains a key
#[no_mangle]
pub extern "C" fn aether_collections_map_contains(
    map: *const c_void,
    key: *const c_void,
) -> bool {
    if map.is_null() || key.is_null() {
        return false;
    }
    
    unsafe {
        let map_ref = &*(map as *const AetherMap);
        if let Some(hashmap) = map_ref.data.as_ref() {
            let key_val = *(key as *const i32);
            (*hashmap).contains_key(&key_val)
        } else {
            false
        }
    }
}

/// Remove a key from the map
#[no_mangle]
pub extern "C" fn aether_collections_map_remove(
    map: *mut c_void,
    key: *const c_void,
) -> bool {
    if map.is_null() || key.is_null() {
        return false;
    }
    
    unsafe {
        let map_ref = &mut *(map as *mut AetherMap);
        if let Some(hashmap) = map_ref.data.as_mut() {
            let key_val = *(key as *const i32);
            (*hashmap).remove(&key_val).is_some()
        } else {
            false
        }
    }
}

/// Get the size of the map
#[no_mangle]
pub extern "C" fn aether_collections_map_size(map: *const c_void) -> usize {
    if map.is_null() {
        return 0;
    }
    
    unsafe {
        let map_ref = &*(map as *const AetherMap);
        if let Some(hashmap) = map_ref.data.as_ref() {
            (*hashmap).len()
        } else {
            0
        }
    }
}