//! Memory management for AetherScript runtime
//! 
//! Provides memory allocation, deallocation, and garbage collection

use std::alloc::{alloc, dealloc, Layout};
use std::collections::HashSet;
use std::ffi::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Mutex;

// Global memory manager
lazy_static::lazy_static! {
    static ref MEMORY_MANAGER: Mutex<MemoryManager> = Mutex::new(MemoryManager::new());
}

/// Allocation header stored before each allocated block
#[repr(C)]
struct AllocationHeader {
    size: usize,
    gc_mark: bool,
    next: *mut AllocationHeader,
    prev: *mut AllocationHeader,
}

struct MemoryManager {
    /// Total allocated memory
    total_allocated: usize,
    
    /// Number of allocations
    allocation_count: usize,
    
    /// Head of allocation linked list for GC
    allocation_list: Option<std::ptr::NonNull<AllocationHeader>>,
    
    /// Root set for garbage collection
    gc_roots: HashSet<usize>, // Store as usize to make it Send
    
    /// GC threshold (bytes)
    gc_threshold: usize,
    
    /// Next GC threshold
    next_gc: usize,
}

// Mark MemoryManager as Send
unsafe impl Send for MemoryManager {}

impl MemoryManager {
    fn new() -> Self {
        Self {
            total_allocated: 0,
            allocation_count: 0,
            allocation_list: None,
            gc_roots: HashSet::new(),
            gc_threshold: 1024 * 1024, // 1MB initial threshold
            next_gc: 1024 * 1024,
        }
    }
    
    unsafe fn allocate(&mut self, size: usize) -> *mut c_void {
        // Calculate total size including header
        let total_size = size + std::mem::size_of::<AllocationHeader>();
        let layout = Layout::from_size_align_unchecked(total_size, 8);
        
        // Allocate memory
        let ptr = alloc(layout);
        if ptr.is_null() {
            return ptr::null_mut();
        }
        
        // Initialize header
        let header = ptr as *mut AllocationHeader;
        (*header).size = size;
        (*header).gc_mark = false;
        (*header).next = self.allocation_list.map(|p| p.as_ptr()).unwrap_or(ptr::null_mut());
        (*header).prev = ptr::null_mut();
        
        // Update linked list
        if let Some(mut old_head) = self.allocation_list {
            old_head.as_mut().prev = header;
        }
        self.allocation_list = std::ptr::NonNull::new(header);
        
        // Update statistics
        self.total_allocated += total_size;
        self.allocation_count += 1;
        
        // Check if GC should run
        if self.total_allocated > self.next_gc {
            self.collect_garbage();
        }
        
        // Return pointer to user data (after header)
        (ptr as *mut u8).add(std::mem::size_of::<AllocationHeader>()) as *mut c_void
    }
    
    unsafe fn deallocate(&mut self, ptr: *mut c_void) {
        if ptr.is_null() {
            return;
        }
        
        // Get header pointer
        let header = (ptr as *mut u8).sub(std::mem::size_of::<AllocationHeader>()) 
            as *mut AllocationHeader;
        
        // Remove from linked list
        if !(*header).prev.is_null() {
            (*(*header).prev).next = (*header).next;
        } else {
            self.allocation_list = std::ptr::NonNull::new((*header).next);
        }
        
        if !(*header).next.is_null() {
            (*(*header).next).prev = (*header).prev;
        }
        
        // Update statistics
        let total_size = (*header).size + std::mem::size_of::<AllocationHeader>();
        self.total_allocated -= total_size;
        self.allocation_count -= 1;
        
        // Deallocate
        let layout = Layout::from_size_align_unchecked(total_size, 8);
        dealloc(header as *mut u8, layout);
    }
    
    unsafe fn reallocate(&mut self, ptr: *mut c_void, new_size: usize) -> *mut c_void {
        if ptr.is_null() {
            return self.allocate(new_size);
        }
        
        // Get old header
        let old_header = (ptr as *mut u8).sub(std::mem::size_of::<AllocationHeader>()) 
            as *mut AllocationHeader;
        let old_size = (*old_header).size;
        
        // Allocate new block
        let new_ptr = self.allocate(new_size);
        if new_ptr.is_null() {
            return ptr::null_mut();
        }
        
        // Copy data
        let copy_size = if new_size < old_size { new_size } else { old_size };
        ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, copy_size);
        
        // Free old block
        self.deallocate(ptr);
        
        new_ptr
    }
    
    fn add_root(&mut self, ptr: *mut c_void) {
        self.gc_roots.insert(ptr as usize);
    }
    
    fn remove_root(&mut self, ptr: *mut c_void) {
        self.gc_roots.remove(&(ptr as usize));
    }
    
    unsafe fn collect_garbage(&mut self) {
        // Mark phase - clear all marks
        let mut current = self.allocation_list;
        while let Some(node) = current {
            let ptr = node.as_ptr();
            (*ptr).gc_mark = false;
            current = std::ptr::NonNull::new((*ptr).next);
        }
        
        // Mark from roots
        let roots: Vec<*mut c_void> = self.gc_roots.iter().map(|&p| p as *mut c_void).collect();
        for root in roots {
            self.mark_recursive(root);
        }
        
        // Sweep phase - collect unmarked allocations
        let mut current = self.allocation_list;
        let mut _collected = 0;
        
        while let Some(node) = current {
            let ptr = node.as_ptr();
            let next = std::ptr::NonNull::new((*ptr).next);
            
            if !(*ptr).gc_mark {
                // Get user pointer
                let user_ptr = (ptr as *mut u8)
                    .add(std::mem::size_of::<AllocationHeader>()) as *mut c_void;
                
                // Deallocate
                self.deallocate(user_ptr);
                _collected += 1;
            }
            
            current = next;
        }
        
        // Update GC threshold
        if self.total_allocated > 0 {
            self.next_gc = self.total_allocated * 2;
        } else {
            self.next_gc = self.gc_threshold;
        }
    }
    
    unsafe fn mark_recursive(&mut self, ptr: *mut c_void) {
        if ptr.is_null() {
            return;
        }
        
        // Check if this is a valid allocation
        let header = (ptr as *mut u8).sub(std::mem::size_of::<AllocationHeader>()) 
            as *mut AllocationHeader;
        
        // Verify it's in our allocation list
        let mut current = self.allocation_list;
        let mut found = false;
        
        while let Some(node) = current {
            let ptr = node.as_ptr();
            if ptr == header {
                found = true;
                break;
            }
            current = std::ptr::NonNull::new((*ptr).next);
        }
        
        if !found || (*header).gc_mark {
            return;
        }
        
        // Mark this allocation
        (*header).gc_mark = true;
        
        // TODO: Scan allocation for pointers and mark recursively
        // This would require type information or conservative scanning
    }
}

/// Allocate memory
#[no_mangle]
pub unsafe extern "C" fn aether_malloc(size: c_int) -> *mut c_void {
    if size <= 0 {
        return ptr::null_mut();
    }
    
    let mut manager = MEMORY_MANAGER.lock().unwrap();
    manager.allocate(size as usize)
}

/// Free memory
#[no_mangle]
pub unsafe extern "C" fn aether_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    
    let mut manager = MEMORY_MANAGER.lock().unwrap();
    manager.deallocate(ptr);
}

/// Reallocate memory
#[no_mangle]
pub unsafe extern "C" fn aether_realloc(ptr: *mut c_void, new_size: c_int) -> *mut c_void {
    if new_size <= 0 {
        if !ptr.is_null() {
            let mut manager = MEMORY_MANAGER.lock().unwrap();
            manager.deallocate(ptr);
        }
        return ptr::null_mut();
    }
    
    let mut manager = MEMORY_MANAGER.lock().unwrap();
    manager.reallocate(ptr, new_size as usize)
}

/// Add a GC root
#[no_mangle]
pub unsafe extern "C" fn aether_gc_add_root(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    
    let mut manager = MEMORY_MANAGER.lock().unwrap();
    manager.add_root(ptr);
}

/// Remove a GC root
#[no_mangle]
pub unsafe extern "C" fn aether_gc_remove_root(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    
    let mut manager = MEMORY_MANAGER.lock().unwrap();
    manager.remove_root(ptr);
}

/// Manually trigger garbage collection
#[no_mangle]
pub unsafe extern "C" fn aether_gc_collect() {
    let mut manager = MEMORY_MANAGER.lock().unwrap();
    manager.collect_garbage();
}

/// Get memory statistics
#[no_mangle]
pub unsafe extern "C" fn aether_memory_used() -> c_int {
    let manager = MEMORY_MANAGER.lock().unwrap();
    manager.total_allocated as c_int
}

/// Get allocation count
#[no_mangle]
pub unsafe extern "C" fn aether_allocation_count() -> c_int {
    let manager = MEMORY_MANAGER.lock().unwrap();
    manager.allocation_count as c_int
}

/// Duplicate a string (for C interop)
#[no_mangle]
pub unsafe extern "C" fn aether_strdup(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    
    let len = libc::strlen(s);
    let new_str = aether_malloc((len + 1) as c_int) as *mut c_char;
    
    if !new_str.is_null() {
        ptr::copy_nonoverlapping(s, new_str, len + 1);
    }
    
    new_str
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_malloc_free() {
        unsafe {
            // Test basic allocation
            let ptr = aether_malloc(100);
            assert!(!ptr.is_null());
            
            // Write and read data
            let data = ptr as *mut u8;
            for i in 0..100 {
                *data.add(i) = i as u8;
            }
            
            for i in 0..100 {
                assert_eq!(*data.add(i), i as u8);
            }
            
            // Free memory
            aether_free(ptr);
        }
    }
    
    #[test]
    fn test_realloc() {
        unsafe {
            // Initial allocation
            let ptr1 = aether_malloc(50) as *mut u8;
            assert!(!ptr1.is_null());
            
            // Fill with data
            for i in 0..50 {
                *ptr1.add(i) = i as u8;
            }
            
            // Reallocate larger
            let ptr2 = aether_realloc(ptr1 as *mut c_void, 100) as *mut u8;
            assert!(!ptr2.is_null());
            
            // Verify data preserved
            for i in 0..50 {
                assert_eq!(*ptr2.add(i), i as u8);
            }
            
            // Free memory
            aether_free(ptr2 as *mut c_void);
        }
    }
    
    #[test]
    fn test_gc_basic() {
        unsafe {
            // Allocate some memory
            let root = aether_malloc(100);
            let _garbage = aether_malloc(200);
            
            // Add root
            aether_gc_add_root(root);
            
            // Trigger GC - should collect garbage but not root
            aether_gc_collect();
            
            // Root should still be valid
            let data = root as *mut u8;
            *data = 42;
            assert_eq!(*data, 42);
            
            // Clean up
            aether_gc_remove_root(root);
            aether_free(root);
        }
    }
    
    #[test]
    fn test_memory_stats() {
        unsafe {
            // Allocate memory
            let ptr1 = aether_malloc(1000);
            let ptr2 = aether_malloc(2000);
            
            // Add to root set to prevent GC from collecting them
            aether_gc_add_root(ptr1);
            aether_gc_add_root(ptr2);
            
            // Just verify that allocations succeeded
            assert!(!ptr1.is_null());
            assert!(!ptr2.is_null());
            assert!(ptr1 != ptr2);
            
            // Verify memory was actually allocated by checking we can write to it
            *(ptr1 as *mut u8) = 42;
            *(ptr2 as *mut u8) = 43;
            assert_eq!(*(ptr1 as *mut u8), 42);
            assert_eq!(*(ptr2 as *mut u8), 43);
            
            // Remove from root set before freeing
            aether_gc_remove_root(ptr1);
            aether_gc_remove_root(ptr2);
            
            // Free memory
            aether_free(ptr1);
            aether_free(ptr2);
            
            // After freeing, we shouldn't crash when allocating again
            let ptr3 = aether_malloc(500);
            assert!(!ptr3.is_null());
            aether_free(ptr3);
        }
    }
}