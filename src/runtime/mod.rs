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

//! AetherScript Runtime Library
//! 
//! Provides runtime support functions for AetherScript programs
//! These functions are exposed with C ABI for LLVM to call

use std::ffi::{c_char, c_int, c_void, CStr};
use std::mem;
use std::ptr;

/// Array structure with length prefix
/// Memory layout: [length: i32][elements...]
#[repr(C)]
struct AetherArray {
    length: i32,
    // Elements follow immediately after in memory
}

/// Create an array with given count
#[no_mangle]
pub unsafe extern "C" fn array_create(count: c_int) -> *mut c_void {
    if count <= 0 {
        return ptr::null_mut();
    }
    
    // Calculate size needed
    let array_size = mem::size_of::<AetherArray>() + (count as usize) * mem::size_of::<i32>();
    
    // Allocate memory
    let layout = std::alloc::Layout::from_size_align_unchecked(array_size, mem::align_of::<i32>());
    let array_ptr = std::alloc::alloc(layout) as *mut AetherArray;
    
    if array_ptr.is_null() {
        return ptr::null_mut();
    }
    
    // Set length
    (*array_ptr).length = count;
    
    // Initialize elements to zero
    let elements_ptr = array_ptr.add(1) as *mut i32;
    ptr::write_bytes(elements_ptr, 0, count as usize);
    
    array_ptr as *mut c_void
}

/// Alternative array creation that takes a pointer to elements
#[no_mangle]
pub unsafe extern "C" fn array_create_from_elements(count: c_int, elements: *const c_int) -> *mut c_void {
    if count <= 0 || elements.is_null() {
        return ptr::null_mut();
    }
    
    // Calculate size needed
    let array_size = mem::size_of::<AetherArray>() + (count as usize) * mem::size_of::<i32>();
    
    // Allocate memory
    let layout = std::alloc::Layout::from_size_align_unchecked(array_size, mem::align_of::<i32>());
    let array_ptr = std::alloc::alloc(layout) as *mut AetherArray;
    
    if array_ptr.is_null() {
        return ptr::null_mut();
    }
    
    // Set length
    (*array_ptr).length = count;
    
    // Get pointer to elements
    let elements_ptr = array_ptr.add(1) as *mut i32;
    
    // Copy elements
    ptr::copy_nonoverlapping(elements, elements_ptr, count as usize);
    
    array_ptr as *mut c_void
}

/// Set an element in an array
#[no_mangle]
pub unsafe extern "C" fn array_set(array_ptr: *mut c_void, index: c_int, value: c_int) {
    if array_ptr.is_null() {
        return;
    }
    
    let array = array_ptr as *mut AetherArray;
    
    // Bounds check
    if index < 0 || index >= (*array).length {
        return;
    }
    
    // Get pointer to elements
    let elements_ptr = array.add(1) as *mut i32;
    
    // Set the element
    *elements_ptr.offset(index as isize) = value;
}

/// Get an element from an array
#[no_mangle]
pub unsafe extern "C" fn array_get(array_ptr: *mut c_void, index: c_int) -> c_int {
    if array_ptr.is_null() {
        return 0;
    }
    
    let array = array_ptr as *mut AetherArray;
    
    // Bounds check
    if index < 0 || index >= (*array).length {
        return 0; // Return 0 for out of bounds
    }
    
    // Get pointer to elements
    let elements_ptr = array.add(1) as *mut i32;
    
    // Return the element
    *elements_ptr.offset(index as isize)
}

/// Get the length of an array
#[no_mangle]
pub unsafe extern "C" fn array_length(array_ptr: *mut c_void) -> c_int {
    if array_ptr.is_null() {
        return 0;
    }
    
    let array = array_ptr as *mut AetherArray;
    (*array).length
}

/// String concatenation
#[no_mangle]
pub unsafe extern "C" fn string_concat(str1: *const c_char, str2: *const c_char) -> *mut c_char {
    let s1 = if str1.is_null() { 
        "" 
    } else { 
        CStr::from_ptr(str1).to_str().unwrap_or("") 
    };
    
    let s2 = if str2.is_null() { 
        "" 
    } else { 
        CStr::from_ptr(str2).to_str().unwrap_or("") 
    };
    
    let result = format!("{}{}\0", s1, s2);
    let c_string = result.as_ptr() as *mut c_char;
    
    // Allocate and copy
    let len = result.len();
    let layout = std::alloc::Layout::from_size_align_unchecked(len, 1);
    let ptr = std::alloc::alloc(layout) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// String length
#[no_mangle]
pub unsafe extern "C" fn string_length(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }
    
    CStr::from_ptr(str).to_bytes().len() as c_int
}

/// Substring extraction
#[no_mangle]
pub unsafe extern "C" fn substring(str: *const c_char, start: c_int, length: c_int) -> *mut c_char {
    if str.is_null() || start < 0 || length <= 0 {
        return ptr::null_mut();
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let start = start as usize;
    let length = length as usize;
    
    if start >= s.len() {
        return ptr::null_mut();
    }
    
    let end = std::cmp::min(start + length, s.len());
    let substr = &s[start..end];
    let result = format!("{}\0", substr);
    
    // Allocate and copy
    let len = result.len();
    let layout = std::alloc::Layout::from_size_align_unchecked(len, 1);
    let ptr = std::alloc::alloc(layout) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// String equality
#[no_mangle]
pub unsafe extern "C" fn string_equals(str1: *const c_char, str2: *const c_char) -> c_int {
    if str1.is_null() && str2.is_null() {
        return 1;
    }
    
    if str1.is_null() || str2.is_null() {
        return 0;
    }
    
    let s1 = CStr::from_ptr(str1).to_bytes();
    let s2 = CStr::from_ptr(str2).to_bytes();
    
    if s1 == s2 { 1 } else { 0 }
}

/// String contains
#[no_mangle]
pub unsafe extern "C" fn string_contains(haystack: *const c_char, needle: *const c_char) -> c_int {
    if haystack.is_null() || needle.is_null() {
        return 0;
    }
    
    let haystack = match CStr::from_ptr(haystack).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    let needle = match CStr::from_ptr(needle).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    if haystack.contains(needle) { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_array_operations() {
        unsafe {
            let elements = vec![10i32, 20, 30, 40, 50];
            let array = array_create_from_elements(5, elements.as_ptr());
            
            assert!(!array.is_null());
            assert_eq!(array_length(array), 5);
            assert_eq!(array_get(array, 0), 10);
            assert_eq!(array_get(array, 2), 30);
            assert_eq!(array_get(array, 4), 50);
            assert_eq!(array_get(array, 5), 0); // Out of bounds
            
            // Clean up
            let array_ptr = array as *mut AetherArray;
            let array_size = mem::size_of::<AetherArray>() + 5 * mem::size_of::<i32>();
            let layout = std::alloc::Layout::from_size_align_unchecked(array_size, mem::align_of::<i32>());
            std::alloc::dealloc(array as *mut u8, layout);
        }
    }
    
    #[test]
    fn test_string_operations() {
        unsafe {
            let s1 = "Hello, \0".as_ptr() as *const c_char;
            let s2 = "World!\0".as_ptr() as *const c_char;
            
            assert_eq!(string_length(s1), 7);
            assert_eq!(string_equals(s1, s1), 1);
            assert_eq!(string_equals(s1, s2), 0);
            assert_eq!(string_contains(s1, "ello\0".as_ptr() as *const c_char), 1);
        }
    }
}