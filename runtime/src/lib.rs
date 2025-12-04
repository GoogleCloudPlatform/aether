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
use std::panic::{self, PanicInfo};
use backtrace::Backtrace;
use unicode_segmentation::UnicodeSegmentation;

/// Initialize the AetherScript runtime
/// This function must be called at the beginning of every AetherScript program
#[no_mangle]
pub extern "C" fn aether_runtime_init() {
    panic::set_hook(Box::new(aether_panic_handler));
    crate::async_runtime::aether_async_init();
}

/// Custom panic handler for AetherScript runtime
fn aether_panic_handler(panic_info: &PanicInfo) {
    // Extract panic message
    let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        *s
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
        s.as_str()
    } else {
        "Unknown panic"
    };
    
    // Extract location information
    let location = panic_info.location()
        .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
        .unwrap_or_else(|| "<unknown location>".to_string());
    
    // Print error header
    eprintln!("\x1b[91;1mError: AetherScript Runtime Panic\x1b[0m");
    eprintln!("\x1b[1mReason:\x1b[0m {}", message);
    eprintln!("\x1b[1mLocation:\x1b[0m {}", location);
    eprintln!();
    
    // Capture and print stack trace
    eprintln!("\x1b[1mStack Trace (most recent call first):\x1b[0m");
    print_stack_trace();
    
    // Exit with standard panic exit code
    std::process::exit(101);
}

/// Print a filtered stack trace
fn print_stack_trace() {
    let backtrace = Backtrace::new();
    let mut frame_count = 0;
    
    for frame in backtrace.frames() {
        for symbol in frame.symbols() {
            if let Some(name) = symbol.name() {
                let name_str = name.to_string();
                
                // Filter out internal frames
                if should_display_frame(&name_str) {
                    let filename = symbol.filename()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "<unknown>".to_string());
                    let lineno = symbol.lineno().unwrap_or(0);
                    
                    eprintln!("  \x1b[36m{}:\x1b[0m \x1b[33m{}\x1b[0m", frame_count, name_str);
                    eprintln!("      at {}:{}", filename, lineno);
                    frame_count += 1;
                    
                    // Limit stack trace depth for readability
                    if frame_count >= 20 {
                        eprintln!("  ... (truncated)");
                        return;
                    }
                }
            }
        }
    }
}

/// Determine if a stack frame should be displayed
fn should_display_frame(name: &str) -> bool {
    // Filter out Rust runtime and backtrace internals
    if name.contains("std::panic::") || 
       name.contains("std::panicking::") ||
       name.contains("rust_panic") ||
       name.contains("rust_begin_unwind") ||
       name.contains("core::panic::") ||
       name.contains("backtrace::") ||
       name.contains("aether_panic_handler") ||
       name.contains("print_stack_trace") {
        return false;
    }
    
    // Filter out some common runtime internals
    if name.contains("__rust_") ||
       name.contains("call_once") ||
       name.contains("lang_start") {
        return false;
    }
    
    true
}

/// Runtime panic function callable from AetherScript
#[no_mangle]
pub extern "C" fn aether_panic(message: *const c_char) {
    let msg = if message.is_null() {
        "AetherScript runtime panic"
    } else {
        unsafe {
            match CStr::from_ptr(message).to_str() {
                Ok(s) => s,
                Err(_) => "AetherScript runtime panic (invalid UTF-8 message)",
            }
        }
    };
    
    panic!("{}", msg);
}

pub mod network;
pub mod memory;
pub mod memory_alloc;
pub mod http;
pub mod json;
pub mod collections;
pub mod io;
pub mod math;
pub mod time;
pub mod concurrency;
pub mod async_runtime;
pub mod ffi;
pub mod ffi_structs;
pub mod gguf;

/// Array structure with length prefix
/// Memory layout: [length: i32][elements...]
#[repr(C)]
struct AetherArray {
    length: i32,
    // Elements follow immediately after in memory
}

/// Create an array with given count and elements
/// This is a simplified version that works with our current setup
#[no_mangle]
pub unsafe extern "C" fn array_create(count: c_int) -> *mut c_void {
    if count <= 0 {
        return ptr::null_mut();
    }
    
    // Calculate size needed with proper alignment
    let header_size = mem::size_of::<AetherArray>();
    let elem_align = mem::align_of::<i32>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let array_size = aligned_offset + (count as usize) * mem::size_of::<i32>();
    
    // Allocate memory using our safe memory allocator
    let array_ptr = crate::memory_alloc::aether_safe_malloc(array_size) as *mut AetherArray;
    
    if array_ptr.is_null() {
        return ptr::null_mut();
    }
    
    // Set length
    (*array_ptr).length = count;
    
    // Initialize elements to zero - use aligned offset
    let elements_ptr = (array_ptr as *mut u8).add(aligned_offset) as *mut i32;
    ptr::write_bytes(elements_ptr, 0, count as usize);
    
    array_ptr as *mut c_void
}

/// Set an element in an array (helper for array creation)
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
    
    // Get pointer to elements with proper alignment
    let header_size = mem::size_of::<AetherArray>();
    let elem_align = mem::align_of::<i32>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut i32;
    
    // Set the element
    *elements_ptr.add(index as usize) = value;
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
    
    // Get pointer to elements with proper alignment
    let header_size = mem::size_of::<AetherArray>();
    let elem_align = mem::align_of::<i32>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut i32;
    
    // Return the element
    *elements_ptr.add(index as usize)
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

/// Free an array
#[no_mangle]
pub unsafe extern "C" fn array_free(array_ptr: *mut c_void) {
    crate::memory_alloc::aether_safe_free(array_ptr);
}

// =============================================================================
// String Array Functions - for dynamic arrays of strings (pointers)
// =============================================================================

/// String array structure
/// Memory layout: [length: i32][capacity: i32][pointers: *mut c_char...]
#[repr(C)]
struct StringArray {
    length: i32,
    capacity: i32,
    // String pointers follow immediately after in memory
}

/// Create a string array with given initial capacity
/// If capacity is 0, creates an empty array with capacity 4
#[no_mangle]
pub unsafe extern "C" fn string_array_create(capacity: c_int) -> *mut c_void {
    let cap = if capacity <= 0 { 4 } else { capacity as usize };

    // Calculate size needed
    let header_size = mem::size_of::<StringArray>();
    let ptr_align = mem::align_of::<*mut c_char>();
    let aligned_offset = (header_size + ptr_align - 1) & !(ptr_align - 1);
    let array_size = aligned_offset + cap * mem::size_of::<*mut c_char>();

    let array_ptr = crate::memory_alloc::aether_safe_malloc(array_size) as *mut StringArray;

    if array_ptr.is_null() {
        return ptr::null_mut();
    }

    (*array_ptr).length = 0;
    (*array_ptr).capacity = cap as i32;

    // Initialize pointers to null
    let elements_ptr = (array_ptr as *mut u8).add(aligned_offset) as *mut *mut c_char;
    ptr::write_bytes(elements_ptr, 0, cap);

    array_ptr as *mut c_void
}

/// Push a string onto the string array (returns new array pointer, may reallocate)
#[no_mangle]
pub unsafe extern "C" fn string_array_push(array_ptr: *mut c_void, value: *const c_char) -> *mut c_void {
    if array_ptr.is_null() {
        // Create new array and push
        let new_array = string_array_create(4);
        if new_array.is_null() {
            return ptr::null_mut();
        }
        return string_array_push(new_array, value);
    }

    let array = array_ptr as *mut StringArray;
    let length = (*array).length;
    let capacity = (*array).capacity;

    let header_size = mem::size_of::<StringArray>();
    let ptr_align = mem::align_of::<*mut c_char>();
    let aligned_offset = (header_size + ptr_align - 1) & !(ptr_align - 1);

    // Check if we need to grow
    if length >= capacity {
        let new_capacity = (capacity * 2) as usize;
        let new_size = aligned_offset + new_capacity * mem::size_of::<*mut c_char>();

        let new_array_ptr = crate::memory_alloc::aether_safe_malloc(new_size) as *mut StringArray;
        if new_array_ptr.is_null() {
            return array_ptr; // Return original on failure
        }

        (*new_array_ptr).length = length;
        (*new_array_ptr).capacity = new_capacity as i32;

        // Copy existing pointers
        let old_elements = (array as *mut u8).add(aligned_offset) as *mut *mut c_char;
        let new_elements = (new_array_ptr as *mut u8).add(aligned_offset) as *mut *mut c_char;
        ptr::copy_nonoverlapping(old_elements, new_elements, length as usize);

        // Initialize remaining to null
        for i in length as usize..new_capacity {
            *new_elements.add(i) = ptr::null_mut();
        }

        // Free old array (but not the strings it contained - they're now in new array)
        crate::memory_alloc::aether_safe_free(array_ptr);

        // Continue with new array
        return string_array_push(new_array_ptr as *mut c_void, value);
    }

    // Duplicate the string and add to array
    let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut *mut c_char;

    // Duplicate the string
    let dup = if value.is_null() {
        ptr::null_mut()
    } else {
        crate::memory::aether_strdup(value)
    };

    *elements_ptr.add(length as usize) = dup;
    (*array).length = length + 1;

    array_ptr
}

/// Get string at index from string array
#[no_mangle]
pub unsafe extern "C" fn string_array_get(array_ptr: *mut c_void, index: c_int) -> *mut c_char {
    if array_ptr.is_null() || index < 0 {
        return ptr::null_mut();
    }

    let array = array_ptr as *mut StringArray;

    if index >= (*array).length {
        return ptr::null_mut();
    }

    let header_size = mem::size_of::<StringArray>();
    let ptr_align = mem::align_of::<*mut c_char>();
    let aligned_offset = (header_size + ptr_align - 1) & !(ptr_align - 1);
    let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut *mut c_char;

    *elements_ptr.add(index as usize)
}

/// Get length of string array
#[no_mangle]
pub unsafe extern "C" fn string_array_length(array_ptr: *mut c_void) -> c_int {
    if array_ptr.is_null() {
        return 0;
    }

    let array = array_ptr as *mut StringArray;
    (*array).length
}

/// Free a string array and all its contained strings
#[no_mangle]
pub unsafe extern "C" fn string_array_free(array_ptr: *mut c_void) {
    if array_ptr.is_null() {
        return;
    }

    let array = array_ptr as *mut StringArray;
    let length = (*array).length;

    let header_size = mem::size_of::<StringArray>();
    let ptr_align = mem::align_of::<*mut c_char>();
    let aligned_offset = (header_size + ptr_align - 1) & !(ptr_align - 1);
    let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut *mut c_char;

    // Free each string
    for i in 0..length as usize {
        let s = *elements_ptr.add(i);
        if !s.is_null() {
            crate::memory_alloc::aether_safe_free(s as *mut c_void);
        }
    }

    // Free the array itself
    crate::memory_alloc::aether_safe_free(array_ptr);
}

// =============================================================================
// Int Array Functions - for dynamic arrays of integers
// =============================================================================

/// Int array structure
/// Memory layout: [length: i32][capacity: i32][elements: i32...]
#[repr(C)]
struct IntArray {
    length: i32,
    capacity: i32,
}

/// Create an int array with given initial capacity
#[no_mangle]
pub unsafe extern "C" fn int_array_create(capacity: c_int) -> *mut c_void {
    let cap = if capacity <= 0 { 4 } else { capacity as usize };

    let header_size = mem::size_of::<IntArray>();
    let elem_align = mem::align_of::<i32>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let array_size = aligned_offset + cap * mem::size_of::<i32>();

    let array_ptr = crate::memory_alloc::aether_safe_malloc(array_size) as *mut IntArray;

    if array_ptr.is_null() {
        return ptr::null_mut();
    }

    (*array_ptr).length = 0;
    (*array_ptr).capacity = cap as i32;

    let elements_ptr = (array_ptr as *mut u8).add(aligned_offset) as *mut i32;
    ptr::write_bytes(elements_ptr, 0, cap);

    array_ptr as *mut c_void
}

/// Push an int onto the int array
#[no_mangle]
pub unsafe extern "C" fn int_array_push(array_ptr: *mut c_void, value: c_int) -> *mut c_void {
    if array_ptr.is_null() {
        let new_array = int_array_create(4);
        if new_array.is_null() {
            return ptr::null_mut();
        }
        return int_array_push(new_array, value);
    }

    let array = array_ptr as *mut IntArray;
    let length = (*array).length;
    let capacity = (*array).capacity;

    let header_size = mem::size_of::<IntArray>();
    let elem_align = mem::align_of::<i32>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);

    if length >= capacity {
        let new_capacity = (capacity * 2) as usize;
        let new_size = aligned_offset + new_capacity * mem::size_of::<i32>();

        let new_array_ptr = crate::memory_alloc::aether_safe_malloc(new_size) as *mut IntArray;
        if new_array_ptr.is_null() {
            return array_ptr;
        }

        (*new_array_ptr).length = length;
        (*new_array_ptr).capacity = new_capacity as i32;

        let old_elements = (array as *mut u8).add(aligned_offset) as *mut i32;
        let new_elements = (new_array_ptr as *mut u8).add(aligned_offset) as *mut i32;
        ptr::copy_nonoverlapping(old_elements, new_elements, length as usize);

        for i in length as usize..new_capacity {
            *new_elements.add(i) = 0;
        }

        crate::memory_alloc::aether_safe_free(array_ptr);

        return int_array_push(new_array_ptr as *mut c_void, value);
    }

    let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut i32;
    *elements_ptr.add(length as usize) = value;
    (*array).length = length + 1;

    array_ptr
}

/// Get int at index from int array
#[no_mangle]
pub unsafe extern "C" fn int_array_get(array_ptr: *mut c_void, index: c_int) -> c_int {
    if array_ptr.is_null() || index < 0 {
        return 0;
    }

    let array = array_ptr as *mut IntArray;

    if index >= (*array).length {
        return 0;
    }

    let header_size = mem::size_of::<IntArray>();
    let elem_align = mem::align_of::<i32>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut i32;

    *elements_ptr.add(index as usize)
}

/// Get length of int array
#[no_mangle]
pub unsafe extern "C" fn int_array_length(array_ptr: *mut c_void) -> c_int {
    if array_ptr.is_null() {
        return 0;
    }

    let array = array_ptr as *mut IntArray;
    (*array).length
}

/// Set int at index in int array
#[no_mangle]
pub unsafe extern "C" fn int_array_set(array_ptr: *mut c_void, index: c_int, value: c_int) -> c_int {
    if array_ptr.is_null() || index < 0 {
        return -1;
    }

    let array = array_ptr as *mut IntArray;
    if index >= (*array).length {
        return -1;
    }

    let header_size = mem::size_of::<IntArray>();
    let elem_align = mem::align_of::<i32>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let elements = (array as *mut u8).add(aligned_offset) as *mut i32;

    *elements.add(index as usize) = value;
    0
}

/// Free an int array
#[no_mangle]
pub unsafe extern "C" fn int_array_free(array_ptr: *mut c_void) {
    crate::memory_alloc::aether_safe_free(array_ptr);
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
    
    // Allocate and copy
    let len = result.len();
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
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

/// Get character at specific index in string
#[no_mangle]
pub unsafe extern "C" fn string_char_at(str: *const c_char, index: c_int) -> c_char {
    if str.is_null() || index < 0 {
        return 0; // Return null character for invalid input
    }
    
    let bytes = CStr::from_ptr(str).to_bytes();
    if (index as usize) >= bytes.len() {
        return 0; // Return null character for out-of-bounds access
    }
    
    bytes[index as usize] as c_char
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
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
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

/// String split - returns an array of strings
#[no_mangle]
pub unsafe extern "C" fn string_split(str: *const c_char, delimiter: *const c_char) -> *mut c_void {
    if str.is_null() || delimiter.is_null() {
        return ptr::null_mut();
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let delim = match CStr::from_ptr(delimiter).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let parts: Vec<&str> = s.split(delim).collect();
    let count = parts.len();
    
    // Create array to hold string pointers with proper alignment
    let header_size = mem::size_of::<AetherArray>();
    let ptr_align = mem::align_of::<*mut c_char>();
    let aligned_offset = (header_size + ptr_align - 1) & !(ptr_align - 1);
    let array_size = aligned_offset + count * mem::size_of::<*mut c_char>();
    let array_ptr = crate::memory_alloc::aether_safe_malloc(array_size) as *mut AetherArray;
    
    if array_ptr.is_null() {
        return ptr::null_mut();
    }
    
    (*array_ptr).length = count as i32;
    
    // Copy strings with aligned pointer
    let strings_ptr = (array_ptr as *mut u8).add(aligned_offset) as *mut *mut c_char;
    for (i, part) in parts.iter().enumerate() {
        let part_with_null = format!("{}\0", part);
        let len = part_with_null.len();
        let str_ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
        
        if !str_ptr.is_null() {
            ptr::copy_nonoverlapping(part_with_null.as_ptr() as *const c_char, str_ptr, len);
            *strings_ptr.add(i) = str_ptr;
        }
    }
    
    array_ptr as *mut c_void
}

/// Convert integer to string
#[no_mangle]
pub unsafe extern "C" fn int_to_string(value: c_int) -> *mut c_char {
    let result = format!("{}\0", value);
    
    let len = result.len();
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Convert string to integer
#[no_mangle]
pub unsafe extern "C" fn string_to_int(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    s.parse::<i32>().unwrap_or(0)
}

/// String trim (remove leading/trailing whitespace)
#[no_mangle]
pub unsafe extern "C" fn string_trim(str: *const c_char) -> *mut c_char {
    if str.is_null() {
        return ptr::null_mut();
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let trimmed = s.trim();
    let result = format!("{}\0", trimmed);
    
    let len = result.len();
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Print a string to stdout (like C's puts)
#[no_mangle]
pub unsafe extern "C" fn puts(str: *const c_char) -> c_int {
    if str.is_null() {
        return -1;
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    println!("{}", s);
    s.len() as c_int
}

/// String to uppercase
#[no_mangle]
pub unsafe extern "C" fn string_to_upper(str: *const c_char) -> *mut c_char {
    if str.is_null() {
        return ptr::null_mut();
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let upper = s.to_uppercase();
    let result = format!("{}\0", upper);
    
    let len = result.len();
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// String to lowercase
#[no_mangle]
pub unsafe extern "C" fn string_to_lower(str: *const c_char) -> *mut c_char {
    if str.is_null() {
        return ptr::null_mut();
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let lower = s.to_lowercase();
    let result = format!("{}\0", lower);
    
    let len = result.len();
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// String replace
#[no_mangle]
pub unsafe extern "C" fn string_replace(str: *const c_char, find: *const c_char, replace: *const c_char) -> *mut c_char {
    if str.is_null() || find.is_null() || replace.is_null() {
        return ptr::null_mut();
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let find_str = match CStr::from_ptr(find).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let replace_str = match CStr::from_ptr(replace).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let replaced = s.replace(find_str, replace_str);
    let result = format!("{}\0", replaced);
    
    let len = result.len();
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Find the index of a substring in a string
#[no_mangle]
pub unsafe extern "C" fn string_index_of(haystack: *const c_char, needle: *const c_char) -> c_int {
    if haystack.is_null() || needle.is_null() {
        return -1;
    }
    
    let haystack_str = match CStr::from_ptr(haystack).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let needle_str = match CStr::from_ptr(needle).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    match haystack_str.find(needle_str) {
        Some(index) => index as c_int,
        None => -1,
    }
}

/// Check if string starts with prefix
#[no_mangle]
pub unsafe extern "C" fn string_starts_with(str: *const c_char, prefix: *const c_char) -> c_int {
    if str.is_null() || prefix.is_null() {
        return 0;
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    let prefix_str = match CStr::from_ptr(prefix).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    if s.starts_with(prefix_str) { 1 } else { 0 }
}

/// Check if string ends with suffix
#[no_mangle]
pub unsafe extern "C" fn string_ends_with(str: *const c_char, suffix: *const c_char) -> c_int {
    if str.is_null() || suffix.is_null() {
        return 0;
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    let suffix_str = match CStr::from_ptr(suffix).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    if s.ends_with(suffix_str) { 1 } else { 0 }
}

/// Join an array of strings with a delimiter
#[no_mangle]
pub unsafe extern "C" fn string_join(strings_array: *mut c_void, delimiter: *const c_char) -> *mut c_char {
    if strings_array.is_null() || delimiter.is_null() {
        return ptr::null_mut();
    }
    
    let array = strings_array as *mut AetherArray;
    let count = (*array).length as usize;
    
    if count == 0 {
        // Return empty string
        let ptr = crate::memory_alloc::aether_safe_malloc(1) as *mut c_char;
        if !ptr.is_null() {
            *ptr = 0;
        }
        return ptr;
    }
    
    let delim = match CStr::from_ptr(delimiter).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    // Get string pointers from array - ensure proper alignment
    // AetherArray has i32 (4 bytes), but pointers need 8-byte alignment on 64-bit systems
    let header_size = mem::size_of::<AetherArray>();
    let ptr_align = mem::align_of::<*mut c_char>();
    let aligned_offset = (header_size + ptr_align - 1) & !(ptr_align - 1); // Round up to alignment
    let strings_ptr = (array as *mut u8).add(aligned_offset) as *mut *mut c_char;
    let mut parts = Vec::new();
    
    for i in 0..count {
        let str_ptr = *strings_ptr.add(i);
        if !str_ptr.is_null() {
            if let Ok(s) = CStr::from_ptr(str_ptr).to_str() {
                parts.push(s);
            }
        }
    }
    
    let joined = parts.join(delim);
    let result = format!("{}\0", joined);
    
    let len = result.len();
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Split a string into an array of UTF-8 characters
/// Each character is returned as a separate string
#[no_mangle]
pub unsafe extern "C" fn string_to_chars(str: *const c_char) -> *mut c_void {
    if str.is_null() {
        return ptr::null_mut();
    }

    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let chars: Vec<&str> = s.graphemes(true).collect();
    let count = chars.len();

    // Create array to hold string pointers with proper alignment
    let header_size = mem::size_of::<AetherArray>();
    let ptr_align = mem::align_of::<*mut c_char>();
    let aligned_offset = (header_size + ptr_align - 1) & !(ptr_align - 1);
    let array_size = aligned_offset + count * mem::size_of::<*mut c_char>();
    let array_ptr = crate::memory_alloc::aether_safe_malloc(array_size) as *mut AetherArray;

    if array_ptr.is_null() {
        return ptr::null_mut();
    }

    (*array_ptr).length = count as i32;

    // Copy each character as a string
    let strings_ptr = (array_ptr as *mut u8).add(aligned_offset) as *mut *mut c_char;
    for (i, ch) in chars.iter().enumerate() {
        let char_with_null = format!("{}\0", ch);
        let len = char_with_null.len();
        let str_ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;

        if !str_ptr.is_null() {
            ptr::copy_nonoverlapping(char_with_null.as_ptr() as *const c_char, str_ptr, len);
            *strings_ptr.add(i) = str_ptr;
        }
    }

    array_ptr as *mut c_void
}

/// Get the number of UTF-8 characters (grapheme clusters) in a string
#[no_mangle]
pub unsafe extern "C" fn string_char_count(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }

    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    s.graphemes(true).count() as c_int
}

/// Get a single UTF-8 character at the given index
/// Returns null if index is out of bounds
#[no_mangle]
pub unsafe extern "C" fn string_grapheme_at(str: *const c_char, index: c_int) -> *mut c_char {
    if str.is_null() || index < 0 {
        return ptr::null_mut();
    }

    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let index = index as usize;

    if let Some(ch) = s.graphemes(true).nth(index) {
        let char_with_null = format!("{}\0", ch);
        let len = char_with_null.len();
        let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;

        if !ptr.is_null() {
            ptr::copy_nonoverlapping(char_with_null.as_ptr() as *const c_char, ptr, len);
        }

        ptr
    } else {
        ptr::null_mut()
    }
}

/// Parse float from string
#[no_mangle]
pub unsafe extern "C" fn parse_float(str: *const c_char) -> f64 {
    if str.is_null() {
        return 0.0;
    }
    
    let s = match CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return 0.0,
    };
    
    s.parse::<f64>().unwrap_or(0.0)
}

/// Convert float to string
#[no_mangle]
pub unsafe extern "C" fn float_to_string(value: f64) -> *mut c_char {
    let result = format!("{}\0", value);
    
    let len = result.len();
    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Alias for substring to match stdlib name
#[no_mangle]
pub unsafe extern "C" fn string_substring(str: *const c_char, start: c_int, length: c_int) -> *mut c_char {
    substring(str, start, length)
}

/// Alias for string_to_int to match stdlib name
#[no_mangle]
pub unsafe extern "C" fn parse_int(str: *const c_char) -> c_int {
    string_to_int(str)
}

/// Generic to_string function (for now just handles integers)
#[no_mangle]
pub unsafe extern "C" fn to_string(value: c_int) -> *mut c_char {
    int_to_string(value)
}

/// Free a string
#[no_mangle]
pub unsafe extern "C" fn string_free(str: *mut c_char) {
    if str.is_null() {
        return;
    }
    
    crate::memory_alloc::aether_safe_free(str as *mut c_void);
}

// ===== Ownership System Cleanup Functions =====

/// Cleanup function for owned strings
#[no_mangle]
pub unsafe extern "C" fn aether_drop_string(str_ptr: *mut c_char) {
    if !str_ptr.is_null() {
        string_free(str_ptr);
    }
}

/// Cleanup function for owned arrays
#[no_mangle]
pub unsafe extern "C" fn aether_drop_array(array_ptr: *mut c_void) {
    if !array_ptr.is_null() {
        array_free(array_ptr);
    }
}

/// Cleanup function for owned maps
#[no_mangle]
pub unsafe extern "C" fn aether_drop_map(map_ptr: *mut c_void) {
    if !map_ptr.is_null() {
        // TODO: Implement proper map cleanup
        // For now, just drop the Box to free memory
        let _ = Box::from_raw(map_ptr as *mut std::collections::HashMap<String, *mut c_void>);
    }
}

/// Reference counting structure for shared ownership (~T)
#[repr(C)]
pub struct AetherRefCount {
    count: std::sync::atomic::AtomicUsize,
    data: *mut c_void,
    drop_fn: Option<unsafe extern "C" fn(*mut c_void)>,
}

/// Create a new reference counted value
#[no_mangle]
pub unsafe extern "C" fn aether_rc_new(data: *mut c_void, drop_fn: Option<unsafe extern "C" fn(*mut c_void)>) -> *mut AetherRefCount {
    let rc = Box::new(AetherRefCount {
        count: std::sync::atomic::AtomicUsize::new(1),
        data,
        drop_fn,
    });
    Box::into_raw(rc)
}

/// Increment reference count
#[no_mangle]
pub unsafe extern "C" fn aether_rc_retain(rc_ptr: *mut AetherRefCount) -> *mut AetherRefCount {
    if !rc_ptr.is_null() {
        let rc = &*rc_ptr;
        rc.count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    rc_ptr
}

/// Decrement reference count and free if zero
#[no_mangle]
pub unsafe extern "C" fn aether_rc_release(rc_ptr: *mut AetherRefCount) {
    if rc_ptr.is_null() {
        return;
    }
    
    let rc = &*rc_ptr;
    let old_count = rc.count.fetch_sub(1, std::sync::atomic::Ordering::Release);
    
    if old_count == 1 {
        // This was the last reference
        std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire);
        
        // Call the drop function if provided
        if let Some(drop_fn) = rc.drop_fn {
            drop_fn(rc.data);
        }
        
        // Free the reference count structure
        Box::from_raw(rc_ptr);
    }
}

/// Get the data pointer from a reference counted value
#[no_mangle]
pub unsafe extern "C" fn aether_rc_get_data(rc_ptr: *mut AetherRefCount) -> *mut c_void {
    if rc_ptr.is_null() {
        return ptr::null_mut();
    }
    (*rc_ptr).data
}

/// Generic cleanup dispatcher based on type
#[no_mangle]
pub unsafe extern "C" fn aether_drop_value(value_ptr: *mut c_void, type_id: c_int) {
    match type_id {
        1 => aether_drop_string(value_ptr as *mut c_char), // String type
        2 => aether_drop_array(value_ptr),                 // Array type
        3 => aether_drop_map(value_ptr),                   // Map type
        _ => {} // Primitive types don't need cleanup
    }
}

// =============================================================================
// Timing Functions - for benchmarking
// =============================================================================

use std::sync::atomic::{AtomicU64, Ordering};

static TIMER_START: AtomicU64 = AtomicU64::new(0);

// =============================================================================
// Random Number Generator - Seeded LCG for deterministic sampling
// =============================================================================

// Simple Linear Congruential Generator state
// Uses constants from Numerical Recipes
static RNG_STATE: AtomicU64 = AtomicU64::new(12345);

/// Seed the random number generator
#[no_mangle]
pub extern "C" fn rng_seed(seed: c_int) -> c_int {
    RNG_STATE.store(seed as u64, Ordering::SeqCst);
    0
}

/// Get next random integer (full range)
#[no_mangle]
pub extern "C" fn rng_next() -> c_int {
    // LCG: state = (a * state + c) mod m
    // Using constants from Numerical Recipes
    let a: u64 = 1664525;
    let c: u64 = 1013904223;
    let state = RNG_STATE.load(Ordering::SeqCst);
    let new_state = state.wrapping_mul(a).wrapping_add(c);
    RNG_STATE.store(new_state, Ordering::SeqCst);
    (new_state >> 16) as c_int  // Use upper bits for better randomness
}

/// Get random integer in range [0, max)
#[no_mangle]
pub extern "C" fn rng_next_range(max: c_int) -> c_int {
    if max <= 0 {
        return 0;
    }
    let r = rng_next();
    // Use modulo (simple approach, slight bias for large ranges)
    ((r as u32) % (max as u32)) as c_int
}

/// Get random float in range [0, 1) - returns f64 for Aether Float
#[no_mangle]
pub extern "C" fn rng_next_float() -> f64 {
    let r = rng_next() as u32;
    (r as f64) / (u32::MAX as f64)
}

// =============================================================================
// Float Array Functions - for logits/probabilities
// =============================================================================

/// Float array structure (uses f64 to match Aether Float type)
#[repr(C)]
struct FloatArray {
    length: i32,
    capacity: i32,
}

/// Create a float array with given initial capacity
#[no_mangle]
pub unsafe extern "C" fn float_array_create(capacity: c_int) -> *mut c_void {
    let cap = if capacity <= 0 { 4 } else { capacity as usize };

    let header_size = mem::size_of::<FloatArray>();
    let elem_align = mem::align_of::<f64>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let array_size = aligned_offset + cap * mem::size_of::<f64>();

    let array_ptr = crate::memory_alloc::aether_safe_malloc(array_size) as *mut FloatArray;

    if array_ptr.is_null() {
        return ptr::null_mut();
    }

    (*array_ptr).length = 0;
    (*array_ptr).capacity = cap as i32;

    let elements_ptr = (array_ptr as *mut u8).add(aligned_offset) as *mut f64;
    ptr::write_bytes(elements_ptr, 0, cap);

    array_ptr as *mut c_void
}

/// Create a float array with given size, pre-filled with zeros (length = size)
#[no_mangle]
pub unsafe extern "C" fn float_array_zeros(size: c_int) -> *mut c_void {
    let cap = if size <= 0 { 4 } else { size as usize };

    let header_size = mem::size_of::<FloatArray>();
    let elem_align = mem::align_of::<f64>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let array_size = aligned_offset + cap * mem::size_of::<f64>();

    let array_ptr = crate::memory_alloc::aether_safe_malloc(array_size) as *mut FloatArray;

    if array_ptr.is_null() {
        return ptr::null_mut();
    }

    (*array_ptr).length = cap as i32; // Set length = capacity
    (*array_ptr).capacity = cap as i32;

    let elements_ptr = (array_ptr as *mut u8).add(aligned_offset) as *mut f64;
    ptr::write_bytes(elements_ptr, 0, cap);

    array_ptr as *mut c_void
}

/// Push a float onto the array (f64 for Aether Float)
#[no_mangle]
pub unsafe extern "C" fn float_array_push(array_ptr: *mut c_void, value: f64) -> *mut c_void {
    if array_ptr.is_null() {
        let new_array = float_array_create(4);
        if new_array.is_null() {
            return ptr::null_mut();
        }
        return float_array_push(new_array, value);
    }

    let array = array_ptr as *mut FloatArray;
    let length = (*array).length;
    let capacity = (*array).capacity;

    let header_size = mem::size_of::<FloatArray>();
    let elem_align = mem::align_of::<f64>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);

    if length >= capacity {
        let new_capacity = (capacity * 2) as usize;
        let new_size = aligned_offset + new_capacity * mem::size_of::<f64>();

        let new_array_ptr = crate::memory_alloc::aether_safe_malloc(new_size) as *mut FloatArray;
        if new_array_ptr.is_null() {
            return array_ptr;
        }

        (*new_array_ptr).length = length;
        (*new_array_ptr).capacity = new_capacity as i32;

        let old_elements = (array as *mut u8).add(aligned_offset) as *mut f64;
        let new_elements = (new_array_ptr as *mut u8).add(aligned_offset) as *mut f64;
        ptr::copy_nonoverlapping(old_elements, new_elements, length as usize);

        crate::memory_alloc::aether_safe_free(array_ptr);

        let elements = (new_array_ptr as *mut u8).add(aligned_offset) as *mut f64;
        *elements.add(length as usize) = value;
        (*new_array_ptr).length = length + 1;

        return new_array_ptr as *mut c_void;
    }

    let elements = (array as *mut u8).add(aligned_offset) as *mut f64;
    *elements.add(length as usize) = value;
    (*array).length = length + 1;

    array_ptr
}

/// Get float at index (f64 for Aether Float)
#[no_mangle]
pub unsafe extern "C" fn float_array_get(array_ptr: *mut c_void, index: c_int) -> f64 {
    if array_ptr.is_null() || index < 0 {
        return 0.0;
    }

    let array = array_ptr as *mut FloatArray;
    if index >= (*array).length {
        return 0.0;
    }

    let header_size = mem::size_of::<FloatArray>();
    let elem_align = mem::align_of::<f64>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let elements = (array as *mut u8).add(aligned_offset) as *mut f64;

    *elements.add(index as usize)
}

/// Set float at index (f64 for Aether Float)
#[no_mangle]
pub unsafe extern "C" fn float_array_set(array_ptr: *mut c_void, index: c_int, value: f64) -> c_int {
    if array_ptr.is_null() || index < 0 {
        return -1;
    }

    let array = array_ptr as *mut FloatArray;
    if index >= (*array).length {
        return -1;
    }

    let header_size = mem::size_of::<FloatArray>();
    let elem_align = mem::align_of::<f64>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let elements = (array as *mut u8).add(aligned_offset) as *mut f64;

    *elements.add(index as usize) = value;
    0
}

/// Get float array length
#[no_mangle]
pub unsafe extern "C" fn float_array_length(array_ptr: *mut c_void) -> c_int {
    if array_ptr.is_null() {
        return 0;
    }
    let array = array_ptr as *mut FloatArray;
    (*array).length
}

/// Free float array
#[no_mangle]
pub unsafe extern "C" fn float_array_free(array_ptr: *mut c_void) {
    if !array_ptr.is_null() {
        crate::memory_alloc::aether_safe_free(array_ptr);
    }
}

// =============================================================================
// Math Functions - for sampling operations (f64 for Aether Float type)
// =============================================================================

/// Exponential function
#[no_mangle]
pub extern "C" fn math_exp(x: f64) -> f64 {
    x.exp()
}

/// Natural logarithm
#[no_mangle]
pub extern "C" fn math_log(x: f64) -> f64 {
    x.ln()
}

/// Maximum of two floats
#[no_mangle]
pub extern "C" fn math_max_f(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// Minimum of two floats
#[no_mangle]
pub extern "C" fn math_min_f(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// Float to int conversion (Aether Float is f64)
#[no_mangle]
pub extern "C" fn float_to_int(x: f64) -> c_int {
    x as c_int
}

/// Int to float conversion (Aether Float is f64)
#[no_mangle]
pub extern "C" fn int_to_float(x: c_int) -> f64 {
    x as f64
}

/// Start a timer for benchmarking
/// Returns 0 on success
#[no_mangle]
pub extern "C" fn timer_start() -> c_int {
    // Store a reference point using the system
    TIMER_START.store(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64, Ordering::SeqCst);
    0
}

/// Get elapsed time in microseconds since timer_start was called
#[no_mangle]
pub extern "C" fn timer_elapsed_us() -> c_int {
    let start = TIMER_START.load(Ordering::SeqCst);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    ((now - start) / 1000) as c_int
}

/// Get elapsed time in milliseconds since timer_start was called
#[no_mangle]
pub extern "C" fn timer_elapsed_ms() -> c_int {
    let start = TIMER_START.load(Ordering::SeqCst);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    ((now - start) / 1_000_000) as c_int
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_array_operations() {
        unsafe {
            crate::memory_alloc::aether_memory_init();
            let array = array_create(5);
            assert!(!array.is_null());
            
            // Set some values
            array_set(array, 0, 10);
            array_set(array, 1, 20);
            array_set(array, 2, 30);
            array_set(array, 3, 40);
            array_set(array, 4, 50);
            
            assert_eq!(array_length(array), 5);
            assert_eq!(array_get(array, 0), 10);
            assert_eq!(array_get(array, 2), 30);
            assert_eq!(array_get(array, 4), 50);
            assert_eq!(array_get(array, 5), 0); // Out of bounds
            
            array_free(array);
            
            // Check no memory leaks
            assert_eq!(crate::memory_alloc::aether_check_leaks(), 0);
        }
    }
    
    #[test]
    fn test_string_index_of() {
        unsafe {
            let haystack = b"Hello, World!\0".as_ptr() as *const c_char;
            let needle = b"World\0".as_ptr() as *const c_char;
            
            assert_eq!(string_index_of(haystack, needle), 7);
            
            let not_found = b"xyz\0".as_ptr() as *const c_char;
            assert_eq!(string_index_of(haystack, not_found), -1);
            
            // Test edge cases
            assert_eq!(string_index_of(ptr::null(), needle), -1);
            assert_eq!(string_index_of(haystack, ptr::null()), -1);
            
            // Test empty needle
            let empty = b"\0".as_ptr() as *const c_char;
            assert_eq!(string_index_of(haystack, empty), 0);
        }
    }
    
    #[test]
    fn test_string_starts_with() {
        unsafe {
            let str = b"Hello, World!\0".as_ptr() as *const c_char;
            let prefix1 = b"Hello\0".as_ptr() as *const c_char;
            let prefix2 = b"World\0".as_ptr() as *const c_char;
            let empty = b"\0".as_ptr() as *const c_char;
            
            assert_eq!(string_starts_with(str, prefix1), 1);
            assert_eq!(string_starts_with(str, prefix2), 0);
            assert_eq!(string_starts_with(str, empty), 1); // Empty prefix always matches
            
            // Test null cases
            assert_eq!(string_starts_with(ptr::null(), prefix1), 0);
            assert_eq!(string_starts_with(str, ptr::null()), 0);
        }
    }
    
    #[test]
    fn test_string_ends_with() {
        unsafe {
            let str = b"Hello, World!\0".as_ptr() as *const c_char;
            let suffix1 = b"World!\0".as_ptr() as *const c_char;
            let suffix2 = b"Hello\0".as_ptr() as *const c_char;
            let empty = b"\0".as_ptr() as *const c_char;
            
            assert_eq!(string_ends_with(str, suffix1), 1);
            assert_eq!(string_ends_with(str, suffix2), 0);
            assert_eq!(string_ends_with(str, empty), 1); // Empty suffix always matches
            
            // Test null cases
            assert_eq!(string_ends_with(ptr::null(), suffix1), 0);
            assert_eq!(string_ends_with(str, ptr::null()), 0);
        }
    }
    
    #[test]
    fn test_parse_float() {
        unsafe {
            let str1 = b"3.14159\0".as_ptr() as *const c_char;
            let str2 = b"-2.5\0".as_ptr() as *const c_char;
            let str3 = b"0.0\0".as_ptr() as *const c_char;
            let invalid = b"abc\0".as_ptr() as *const c_char;
            
            assert!((parse_float(str1) - 3.14159).abs() < 0.00001);
            assert!((parse_float(str2) - (-2.5)).abs() < 0.00001);
            assert!((parse_float(str3) - 0.0).abs() < 0.00001);
            assert!((parse_float(invalid) - 0.0).abs() < 0.00001); // Invalid returns 0.0
            
            // Test null case
            assert!((parse_float(ptr::null()) - 0.0).abs() < 0.00001);
        }
    }
    
    #[test]
    fn test_float_to_string() {
        unsafe {
            let result1 = float_to_string(3.14159);
            assert!(!result1.is_null());
            let str1 = CStr::from_ptr(result1).to_str().unwrap();
            assert!(str1 == "3.14159");
            string_free(result1);
            
            let result2 = float_to_string(-2.5);
            assert!(!result2.is_null());
            let str2 = CStr::from_ptr(result2).to_str().unwrap();
            assert!(str2 == "-2.5");
            string_free(result2);
            
            let result3 = float_to_string(0.0);
            assert!(!result3.is_null());
            let str3 = CStr::from_ptr(result3).to_str().unwrap();
            assert!(str3 == "0");
            string_free(result3);
        }
    }
    
    #[test] 
    fn test_string_join() {
        unsafe {
            // Create array of strings using standard allocation for testing
            let array_size = 3;
            
            // Use a properly aligned struct for testing
            #[repr(C)]
            struct TestArray {
                header: AetherArray,
                _padding: [u8; 4], // Padding to ensure 8-byte alignment for pointers on 64-bit
                strings: [*mut c_char; 3],
            }
            
            // Allocate using Box for proper alignment
            let mut test_array = Box::new(TestArray {
                header: AetherArray { length: array_size },
                _padding: [0; 4],
                strings: [ptr::null_mut(); 3],
            });
            
            // Allocate and set up test strings
            let str1 = b"Hello\0";
            let str2 = b"World\0"; 
            let str3 = b"!\0";
            
            // Use standard allocation for test strings
            let s1 = libc::malloc(str1.len()) as *mut c_char;
            ptr::copy_nonoverlapping(str1.as_ptr() as *const c_char, s1, str1.len());
            test_array.strings[0] = s1;
            
            let s2 = libc::malloc(str2.len()) as *mut c_char;
            ptr::copy_nonoverlapping(str2.as_ptr() as *const c_char, s2, str2.len());
            test_array.strings[1] = s2;
            
            let s3 = libc::malloc(str3.len()) as *mut c_char;
            ptr::copy_nonoverlapping(str3.as_ptr() as *const c_char, s3, str3.len());
            test_array.strings[2] = s3;
            
            // Test join with comma delimiter
            let delimiter = b", \0".as_ptr() as *const c_char;
            let array_ptr = &test_array.header as *const AetherArray as *mut c_void;
            let result = string_join(array_ptr, delimiter);
            assert!(!result.is_null());
            let joined_str = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(joined_str, "Hello, World, !");
            string_free(result);
            
            // Test join with empty delimiter
            let empty_delim = b"\0".as_ptr() as *const c_char;
            let result2 = string_join(array_ptr, empty_delim);
            assert!(!result2.is_null());
            let joined_str2 = CStr::from_ptr(result2).to_str().unwrap();
            assert_eq!(joined_str2, "HelloWorld!");
            string_free(result2);
            
            // Test empty array
            test_array.header.length = 0;
            let result3 = string_join(array_ptr, delimiter);
            assert!(!result3.is_null());
            let joined_str3 = CStr::from_ptr(result3).to_str().unwrap();
            assert_eq!(joined_str3, "");
            string_free(result3);
            
            // Clean up
            libc::free(s1 as *mut c_void);
            libc::free(s2 as *mut c_void);
            libc::free(s3 as *mut c_void);
        }
    }
    
    #[test]
    fn test_string_operations_existing() {
        unsafe {
            // Test string_concat
            let str1 = b"Hello, \0".as_ptr() as *const c_char;
            let str2 = b"World!\0".as_ptr() as *const c_char;
            let result = string_concat(str1, str2);
            assert!(!result.is_null());
            let concat_str = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(concat_str, "Hello, World!");
            string_free(result);
            
            // Test string_length
            assert_eq!(string_length(str1), 7);
            assert_eq!(string_length(str2), 6);
            
            // Test string_char_at
            assert_eq!(string_char_at(str1, 0), b'H' as c_char);
            assert_eq!(string_char_at(str1, 4), b'o' as c_char);
            assert_eq!(string_char_at(str1, 100), 0); // Out of bounds
            
            // Test string_equals
            let same = b"Hello, \0".as_ptr() as *const c_char;
            assert_eq!(string_equals(str1, same), 1);
            assert_eq!(string_equals(str1, str2), 0);
            
            // Test string_contains
            let needle = b"llo\0".as_ptr() as *const c_char;
            assert_eq!(string_contains(str1, needle), 1);
            let not_there = b"xyz\0".as_ptr() as *const c_char;
            assert_eq!(string_contains(str1, not_there), 0);
        }
    }
    
    #[test]
    fn test_string_aliases() {
        unsafe {
            // Test parse_int alias
            let int_str = b"42\0".as_ptr() as *const c_char;
            assert_eq!(parse_int(int_str), 42);
            
            // Test string_substring alias
            let str = b"Hello World\0".as_ptr() as *const c_char;
            let sub = string_substring(str, 6, 5);
            assert!(!sub.is_null());
            let sub_str = CStr::from_ptr(sub).to_str().unwrap();
            assert_eq!(sub_str, "World");
            string_free(sub);
            
            // Test to_string alias
            let result = to_string(123);
            assert!(!result.is_null());
            let str = CStr::from_ptr(result).to_str().unwrap();
            assert_eq!(str, "123");
            string_free(result);
        }
    }
    
    #[test]
    fn test_async_runtime_ffi() {
        use std::ffi::c_void;
        use std::sync::{Arc, Mutex};
        use std::time::Duration;
        
        // Initialize runtime
        crate::async_runtime::aether_async_init();
        
        // Define a task function that mocks an Aether function
        extern "C" fn mock_task(ctx: *mut c_void) -> *mut c_void {
            unsafe {
                // Context is a pointer to an integer
                let val_ptr = ctx as *mut i32;
                let val = *val_ptr;
                
                // Sleep to simulate work
                std::thread::sleep(Duration::from_millis(10));
                
                // Return result (val * 2)
                let result = Box::new(val * 2);
                Box::into_raw(result) as *mut c_void
            }
        }
        
        // Define cleanup function
        extern "C" fn mock_cleanup(ctx: *mut c_void) {
            unsafe {
                let _ = Box::from_raw(ctx as *mut i32);
            }
        }
        
        unsafe {
            // Create context
            let context = Box::new(21);
            let ctx_ptr = Box::into_raw(context) as *mut c_void;
            
            // Spawn task
            let future = crate::async_runtime::aether_spawn(mock_task, ctx_ptr, Some(mock_cleanup));
            
            // Wait for result
            let result_ptr = crate::async_runtime::aether_await(future);
            
            // Verify result
            let result = *(result_ptr as *mut i32);
            assert_eq!(result, 42);
            
            // Cleanup result
            let _ = Box::from_raw(result_ptr as *mut i32);
            
            // Release future
            crate::async_runtime::aether_future_release(future);
        }
    }
}