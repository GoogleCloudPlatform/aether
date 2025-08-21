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

//! Safe memory allocation for Aether runtime
//! 
//! Provides malloc/free wrappers with safety checks including:
//! - Allocation tracking to detect leaks
//! - Double-free protection
//! - Buffer overflow detection using guard bytes
//! - Alignment guarantees

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashMap;
use std::ptr;
use std::sync::Mutex;
use libc::{c_void, size_t};

/// Guard bytes to detect buffer overflows
const GUARD_BYTES: [u8; 8] = [0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF];

/// Allocation metadata
#[repr(C)]
struct AllocHeader {
    size: size_t,
    magic: u32,
    guard_start: [u8; 8],
}

const ALLOC_MAGIC: u32 = 0xAE743200;
const FREED_MAGIC: u32 = 0xDEADBEEF;

/// Global allocation tracker
static ALLOCATIONS: Mutex<Option<HashMap<usize, AllocInfo>>> = Mutex::new(None);

#[derive(Debug)]
struct AllocInfo {
    size: usize,
    backtrace: Option<String>,
}

/// Initialize the memory tracking system
#[no_mangle]
pub unsafe extern "C" fn aether_memory_init() {
    let mut allocations = ALLOCATIONS.lock().unwrap();
    if allocations.is_none() {
        *allocations = Some(HashMap::new());
    }
}

/// Allocate memory with safety checks
#[no_mangle]
pub unsafe extern "C" fn aether_safe_malloc(size: size_t) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }
    
    // Calculate total size including header and guard bytes
    let header_size = std::mem::size_of::<AllocHeader>();
    let total_size = header_size + size + GUARD_BYTES.len();
    
    // Ensure proper alignment
    let align = std::mem::align_of::<AllocHeader>().max(8);
    let layout = match Layout::from_size_align(total_size, align) {
        Ok(layout) => layout,
        Err(_) => return ptr::null_mut(),
    };
    
    // Allocate memory
    let raw_ptr = alloc(layout);
    if raw_ptr.is_null() {
        return ptr::null_mut();
    }
    
    // Set up header
    let header_ptr = raw_ptr as *mut AllocHeader;
    (*header_ptr).size = size;
    (*header_ptr).magic = ALLOC_MAGIC;
    (*header_ptr).guard_start.copy_from_slice(&GUARD_BYTES);
    
    // Set up trailing guard bytes
    let user_ptr = raw_ptr.add(header_size);
    let guard_ptr = user_ptr.add(size);
    ptr::copy_nonoverlapping(GUARD_BYTES.as_ptr(), guard_ptr, GUARD_BYTES.len());
    
    // Track allocation
    if let Ok(mut allocations) = ALLOCATIONS.lock() {
        if let Some(ref mut map) = *allocations {
            map.insert(user_ptr as usize, AllocInfo {
                size,
                backtrace: None, // Could capture backtrace here for debugging
            });
        }
    }
    
    user_ptr as *mut c_void
}

/// Free memory with safety checks
#[no_mangle]
pub unsafe extern "C" fn aether_safe_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    
    let user_ptr = ptr as *mut u8;
    let header_size = std::mem::size_of::<AllocHeader>();
    let raw_ptr = user_ptr.sub(header_size);
    let header_ptr = raw_ptr as *mut AllocHeader;
    
    // Check magic number
    if (*header_ptr).magic == FREED_MAGIC {
        // Double free detected
        eprintln!("AETHER MEMORY ERROR: Double free detected at {:p}", ptr);
        return;
    }
    
    if (*header_ptr).magic != ALLOC_MAGIC {
        // Invalid pointer or corrupted header
        eprintln!("AETHER MEMORY ERROR: Invalid free of {:p} (bad magic: 0x{:x})", 
                 ptr, (*header_ptr).magic);
        return;
    }
    
    let size = (*header_ptr).size;
    
    // Check guard bytes
    let header_guard = &(*header_ptr).guard_start;
    if header_guard != &GUARD_BYTES {
        eprintln!("AETHER MEMORY ERROR: Buffer underflow detected at {:p}", ptr);
    }
    
    let guard_ptr = user_ptr.add(size);
    let mut guard_corrupted = false;
    for i in 0..GUARD_BYTES.len() {
        if *guard_ptr.add(i) != GUARD_BYTES[i] {
            guard_corrupted = true;
            break;
        }
    }
    
    if guard_corrupted {
        eprintln!("AETHER MEMORY ERROR: Buffer overflow detected at {:p}", ptr);
    }
    
    // Mark as freed
    (*header_ptr).magic = FREED_MAGIC;
    
    // Remove from tracking
    if let Ok(mut allocations) = ALLOCATIONS.lock() {
        if let Some(ref mut map) = *allocations {
            map.remove(&(user_ptr as usize));
        }
    }
    
    // Free the memory
    let total_size = header_size + size + GUARD_BYTES.len();
    let align = std::mem::align_of::<AllocHeader>().max(8);
    if let Ok(layout) = Layout::from_size_align(total_size, align) {
        dealloc(raw_ptr, layout);
    }
}

/// Reallocate memory with safety checks
#[no_mangle]
pub unsafe extern "C" fn aether_safe_realloc(ptr: *mut c_void, new_size: size_t) -> *mut c_void {
    if ptr.is_null() {
        return aether_safe_malloc(new_size);
    }
    
    if new_size == 0 {
        aether_safe_free(ptr);
        return ptr::null_mut();
    }
    
    // Get old size from header
    let user_ptr = ptr as *mut u8;
    let header_size = std::mem::size_of::<AllocHeader>();
    let raw_ptr = user_ptr.sub(header_size);
    let header_ptr = raw_ptr as *mut AllocHeader;
    
    if (*header_ptr).magic != ALLOC_MAGIC {
        eprintln!("AETHER MEMORY ERROR: Invalid realloc of {:p}", ptr);
        return ptr::null_mut();
    }
    
    let old_size = (*header_ptr).size;
    
    // Allocate new block
    let new_ptr = aether_safe_malloc(new_size);
    if new_ptr.is_null() {
        return ptr::null_mut();
    }
    
    // Copy data
    let copy_size = old_size.min(new_size);
    ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, copy_size);
    
    // Free old block
    aether_safe_free(ptr);
    
    new_ptr
}

/// Check for memory leaks
#[no_mangle]
pub unsafe extern "C" fn aether_check_leaks() -> size_t {
    if let Ok(allocations) = ALLOCATIONS.lock() {
        if let Some(ref map) = *allocations {
            let leak_count = map.len();
            if leak_count > 0 {
                eprintln!("AETHER MEMORY: {} allocation(s) leaked:", leak_count);
                for (addr, info) in map.iter() {
                    eprintln!("  - Address: 0x{:x}, Size: {} bytes", addr, info.size);
                }
            }
            return leak_count;
        }
    }
    0
}

/// Get current memory usage
#[no_mangle]
pub unsafe extern "C" fn aether_memory_usage() -> size_t {
    if let Ok(allocations) = ALLOCATIONS.lock() {
        if let Some(ref map) = *allocations {
            return map.values().map(|info| info.size).sum();
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Helper to reset allocations between tests
    fn reset_allocations() {
        unsafe {
            if let Ok(mut allocations) = ALLOCATIONS.lock() {
                if let Some(ref mut map) = *allocations {
                    map.clear();
                }
            }
        }
    }
    
    #[test]
    fn test_basic_allocation() {
        unsafe {
            reset_allocations();
            aether_memory_init();
            
            // Test allocation
            let ptr = aether_safe_malloc(100);
            assert!(!ptr.is_null());
            
            // Write some data
            let data = ptr as *mut u8;
            for i in 0..100 {
                *data.add(i) = i as u8;
            }
            
            // Read it back
            for i in 0..100 {
                assert_eq!(*data.add(i), i as u8);
            }
            
            // Free
            aether_safe_free(ptr);
            
            // Check no leaks
            assert_eq!(aether_check_leaks(), 0);
        }
    }
    
    #[test]
    fn test_zero_allocation() {
        unsafe {
            reset_allocations();
            aether_memory_init();
            
            let ptr = aether_safe_malloc(0);
            assert!(ptr.is_null());
        }
    }
    
    #[test]
    fn test_double_free_protection() {
        unsafe {
            reset_allocations();
            aether_memory_init();
            
            let ptr = aether_safe_malloc(50);
            assert!(!ptr.is_null());
            
            aether_safe_free(ptr);
            // This should not crash, just print an error
            aether_safe_free(ptr);
            
            assert_eq!(aether_check_leaks(), 0);
        }
    }
    
    #[test]
    fn test_realloc() {
        unsafe {
            reset_allocations();
            aether_memory_init();
            
            // Initial allocation
            let ptr1 = aether_safe_malloc(50);
            assert!(!ptr1.is_null());
            
            // Fill with data
            let data1 = ptr1 as *mut u8;
            for i in 0..50 {
                *data1.add(i) = i as u8;
            }
            
            // Reallocate larger
            let ptr2 = aether_safe_realloc(ptr1, 100);
            assert!(!ptr2.is_null());
            
            // Check old data preserved
            let data2 = ptr2 as *mut u8;
            for i in 0..50 {
                assert_eq!(*data2.add(i), i as u8);
            }
            
            // Write to new area
            for i in 50..100 {
                *data2.add(i) = i as u8;
            }
            
            aether_safe_free(ptr2);
            assert_eq!(aether_check_leaks(), 0);
        }
    }
    
    #[test]
    fn test_memory_tracking() {
        unsafe {
            reset_allocations();
            aether_memory_init();
            
            assert_eq!(aether_memory_usage(), 0);
            
            let ptr1 = aether_safe_malloc(100);
            assert_eq!(aether_memory_usage(), 100);
            
            let ptr2 = aether_safe_malloc(200);
            assert_eq!(aether_memory_usage(), 300);
            
            aether_safe_free(ptr1);
            assert_eq!(aether_memory_usage(), 200);
            
            aether_safe_free(ptr2);
            assert_eq!(aether_memory_usage(), 0);
            assert_eq!(aether_check_leaks(), 0);
        }
    }
    
    #[test]
    fn test_buffer_overflow_detection() {
        unsafe {
            reset_allocations();
            aether_memory_init();
            
            let ptr = aether_safe_malloc(10);
            let data = ptr as *mut u8;
            
            // Write past the end (this will be detected on free)
            *data.add(10) = 0xFF; // This corrupts guard bytes
            
            // Free should detect the overflow
            aether_safe_free(ptr);
            
            // Note: In a real implementation, we might want to check
            // for overflow periodically, not just on free
            
            // Since we freed it, there should be no leaks
            // (The overflow detection doesn't prevent freeing)
        }
    }
}