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

//! Typed Array Implementation for AetherScript
//!
//! Provides specialized array implementations for primitive types to ensure
//! efficient memory usage and high performance.

use std::ffi::{c_int, c_void};
use std::mem;
use std::ptr;

/// Helper macro to generate array implementations for primitive types
macro_rules! impl_typed_array {
    ($name:ident, $type:ty, $create_fn:ident, $push_fn:ident, $get_fn:ident, $set_fn:ident, $len_fn:ident, $free_fn:ident) => {
        /// Array structure with length prefix
        #[repr(C)]
        pub struct $name {
            pub length: i32,
            pub capacity: i32,
            // Elements follow immediately after in memory
        }

        /// Create an array with given capacity
        #[no_mangle]
        pub unsafe extern "C" fn $create_fn(capacity: c_int) -> *mut c_void {
            let cap = if capacity <= 0 { 4 } else { capacity as usize };

            // Calculate size needed with proper alignment
            let header_size = mem::size_of::<$name>();
            let elem_align = mem::align_of::<$type>();
            let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
            let array_size = aligned_offset + cap * mem::size_of::<$type>();

            // Allocate memory
            let array_ptr = crate::memory_alloc::aether_safe_malloc(array_size) as *mut $name;

            if array_ptr.is_null() {
                return ptr::null_mut();
            }

            // Initialize header
            (*array_ptr).length = 0;
            (*array_ptr).capacity = cap as i32;

            // Initialize elements to zero
            let elements_ptr = (array_ptr as *mut u8).add(aligned_offset) as *mut $type;
            ptr::write_bytes(elements_ptr, 0, cap);

            array_ptr as *mut c_void
        }

        /// Push an element onto the array
        #[no_mangle]
        pub unsafe extern "C" fn $push_fn(array_ptr: *mut c_void, value: $type) -> *mut c_void {
            if array_ptr.is_null() {
                let new_array = $create_fn(4);
                if new_array.is_null() {
                    return ptr::null_mut();
                }
                return $push_fn(new_array, value);
            }

            let array = array_ptr as *mut $name;
            let length = (*array).length;
            let capacity = (*array).capacity;

            let header_size = mem::size_of::<$name>();
            let elem_align = mem::align_of::<$type>();
            let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);

            // Check if we need to grow
            if length >= capacity {
                let new_capacity = (capacity * 2) as usize;
                let new_size = aligned_offset + new_capacity * mem::size_of::<$type>();

                let new_array_ptr = crate::memory_alloc::aether_safe_malloc(new_size) as *mut $name;
                if new_array_ptr.is_null() {
                    return array_ptr; // Return original on failure
                }

                (*new_array_ptr).length = length;
                (*new_array_ptr).capacity = new_capacity as i32;

                // Copy existing elements
                let old_elements = (array as *mut u8).add(aligned_offset) as *mut $type;
                let new_elements = (new_array_ptr as *mut u8).add(aligned_offset) as *mut $type;
                ptr::copy_nonoverlapping(old_elements, new_elements, length as usize);

                // Free old array
                crate::memory_alloc::aether_safe_free(array_ptr);

                // Add new element
                *new_elements.add(length as usize) = value;
                (*new_array_ptr).length = length + 1;

                return new_array_ptr as *mut c_void;
            }

            // Add element to existing array
            let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut $type;
            *elements_ptr.add(length as usize) = value;
            (*array).length = length + 1;

            array_ptr
        }

        /// Get an element from the array
        /// Note: Verification system should ensure bounds safety, so this can be unchecked in optimized builds
        #[no_mangle]
        pub unsafe extern "C" fn $get_fn(array_ptr: *mut c_void, index: c_int) -> $type {
            if array_ptr.is_null() {
                return Default::default();
            }

            let array = array_ptr as *mut $name;
            
            // Bounds check (runtime safety)
            if index < 0 || index >= (*array).length {
                return Default::default();
            }

            let header_size = mem::size_of::<$name>();
            let elem_align = mem::align_of::<$type>();
            let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
            let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut $type;

            *elements_ptr.add(index as usize)
        }

        /// Set an element in the array
        #[no_mangle]
        pub unsafe extern "C" fn $set_fn(array_ptr: *mut c_void, index: c_int, value: $type) {
            if array_ptr.is_null() {
                return;
            }

            let array = array_ptr as *mut $name;
            
            if index < 0 || index >= (*array).length {
                return;
            }

            let header_size = mem::size_of::<$name>();
            let elem_align = mem::align_of::<$type>();
            let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
            let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut $type;

            *elements_ptr.add(index as usize) = value;
        }

        /// Get array length
        #[no_mangle]
        pub unsafe extern "C" fn $len_fn(array_ptr: *mut c_void) -> c_int {
            if array_ptr.is_null() {
                return 0;
            }
            let array = array_ptr as *mut $name;
            (*array).length
        }

        /// Free the array
        #[no_mangle]
        pub unsafe extern "C" fn $free_fn(array_ptr: *mut c_void) {
            if !array_ptr.is_null() {
                crate::memory_alloc::aether_safe_free(array_ptr);
            }
        }
    };
}

// Implement arrays for all primitive types
impl_typed_array!(ArrayU8, u8, array_u8_create, array_u8_push, array_u8_get, array_u8_set, array_u8_length, array_u8_free);
impl_typed_array!(ArrayI8, i8, array_i8_create, array_i8_push, array_i8_get, array_i8_set, array_i8_length, array_i8_free);
impl_typed_array!(ArrayU16, u16, array_u16_create, array_u16_push, array_u16_get, array_u16_set, array_u16_length, array_u16_free);
impl_typed_array!(ArrayI16, i16, array_i16_create, array_i16_push, array_i16_get, array_i16_set, array_i16_length, array_i16_free);
impl_typed_array!(ArrayU32, u32, array_u32_create, array_u32_push, array_u32_get, array_u32_set, array_u32_length, array_u32_free);
impl_typed_array!(ArrayI32, i32, array_i32_create, array_i32_push, array_i32_get, array_i32_set, array_i32_length, array_i32_free);
impl_typed_array!(ArrayU64, u64, array_u64_create, array_u64_push, array_u64_get, array_u64_set, array_u64_length, array_u64_free);
impl_typed_array!(ArrayI64, i64, array_i64_create, array_i64_push, array_i64_get, array_i64_set, array_i64_length, array_i64_free);
impl_typed_array!(ArrayF32, f32, array_f32_create, array_f32_push, array_f32_get, array_f32_set, array_f32_length, array_f32_free);
impl_typed_array!(ArrayF64, f64, array_f64_create, array_f64_push, array_f64_get, array_f64_set, array_f64_length, array_f64_free);

// Unchecked accessors for optimization (verification-driven)
// These skip bounds checks when the compiler can prove safety via contracts

#[no_mangle]
pub unsafe extern "C" fn array_u8_get_unchecked(array_ptr: *mut c_void, index: c_int) -> u8 {
    let array = array_ptr as *mut ArrayU8;
    let header_size = mem::size_of::<ArrayU8>();
    let elem_align = mem::align_of::<u8>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let elements_ptr = (array as *mut u8).add(aligned_offset);
    *elements_ptr.add(index as usize)
}

#[no_mangle]
pub unsafe extern "C" fn array_f32_get_unchecked(array_ptr: *mut c_void, index: c_int) -> f32 {
    let array = array_ptr as *mut ArrayF32;
    let header_size = mem::size_of::<ArrayF32>();
    let elem_align = mem::align_of::<f32>();
    let aligned_offset = (header_size + elem_align - 1) & !(elem_align - 1);
    let elements_ptr = (array as *mut u8).add(aligned_offset) as *mut f32;
    *elements_ptr.add(index as usize)
}
