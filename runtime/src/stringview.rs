// StringView and Arena - Zero-copy string operations for high-performance string processing
//
// Design:
// - StringView is a (ptr, len) pair that references existing string data
// - Views don't own their data - the parent string must outlive all views
// - Arena provides bump-pointer allocation for fast string creation with bulk deallocation

use std::ffi::{c_char, c_int, c_void, CStr};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

// =============================================================================
// StringView - Zero-copy substring views
// =============================================================================

/// StringView structure - a borrowed reference into a string
/// This is NOT null-terminated - it's a (ptr, len) slice
#[repr(C)]
pub struct StringView {
    /// Pointer to the first character (not owned)
    ptr: *const c_char,
    /// Length in bytes
    len: usize,
    /// Parent string pointer (for lifetime tracking - not used at runtime, but enables verification)
    parent: *const c_char,
}

// Global storage for string views (simple approach - could use arena per scope later)
static VIEW_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Create a StringView from a String and range
/// Returns an opaque handle (pointer to StringView)
#[no_mangle]
pub unsafe extern "C" fn stringview_create(
    parent: *const c_char,
    start: c_int,
    len: c_int,
) -> *mut c_void {
    if parent.is_null() || start < 0 || len < 0 {
        return ptr::null_mut();
    }

    let parent_len = CStr::from_ptr(parent).to_bytes().len();
    let start = start as usize;
    let len = len as usize;

    // Bounds check
    if start > parent_len || start + len > parent_len {
        return ptr::null_mut();
    }

    let view = Box::new(StringView {
        ptr: parent.add(start),
        len,
        parent,
    });

    VIEW_COUNTER.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(view) as *mut c_void
}

/// Create a StringView from an entire String
#[no_mangle]
pub unsafe extern "C" fn stringview_from_string(s: *const c_char) -> *mut c_void {
    if s.is_null() {
        return ptr::null_mut();
    }

    let len = CStr::from_ptr(s).to_bytes().len();
    stringview_create(s, 0, len as c_int)
}

/// Get the length of a StringView
#[no_mangle]
pub unsafe extern "C" fn stringview_length(view: *mut c_void) -> c_int {
    if view.is_null() {
        return 0;
    }
    let view = &*(view as *const StringView);
    view.len as c_int
}

/// Get character at index in StringView (returns byte value)
#[no_mangle]
pub unsafe extern "C" fn stringview_char_at(view: *mut c_void, index: c_int) -> c_int {
    if view.is_null() || index < 0 {
        return 0;
    }

    let view = &*(view as *const StringView);
    let index = index as usize;

    if index >= view.len {
        return 0;
    }

    *view.ptr.add(index) as u8 as c_int
}

/// Create a sub-view from a StringView (zero-copy)
#[no_mangle]
pub unsafe extern "C" fn stringview_slice(
    view: *mut c_void,
    start: c_int,
    len: c_int,
) -> *mut c_void {
    if view.is_null() || start < 0 || len < 0 {
        return ptr::null_mut();
    }

    let view = &*(view as *const StringView);
    let start = start as usize;
    let len = len as usize;

    // Bounds check
    if start > view.len || start + len > view.len {
        return ptr::null_mut();
    }

    let sub_view = Box::new(StringView {
        ptr: view.ptr.add(start),
        len,
        parent: view.parent, // Keep original parent for lifetime tracking
    });

    VIEW_COUNTER.fetch_add(1, Ordering::Relaxed);
    Box::into_raw(sub_view) as *mut c_void
}

/// Check if two StringViews are equal (byte-by-byte comparison)
#[no_mangle]
pub unsafe extern "C" fn stringview_equals(a: *mut c_void, b: *mut c_void) -> c_int {
    if a.is_null() && b.is_null() {
        return 1;
    }
    if a.is_null() || b.is_null() {
        return 0;
    }

    let a = &*(a as *const StringView);
    let b = &*(b as *const StringView);

    if a.len != b.len {
        return 0;
    }

    // Compare bytes
    let a_slice = std::slice::from_raw_parts(a.ptr as *const u8, a.len);
    let b_slice = std::slice::from_raw_parts(b.ptr as *const u8, b.len);

    if a_slice == b_slice { 1 } else { 0 }
}

/// Check if StringView equals a C string
#[no_mangle]
pub unsafe extern "C" fn stringview_equals_str(view: *mut c_void, s: *const c_char) -> c_int {
    if view.is_null() && s.is_null() {
        return 1;
    }
    if view.is_null() || s.is_null() {
        return 0;
    }

    let view = &*(view as *const StringView);
    let s_bytes = CStr::from_ptr(s).to_bytes();

    if view.len != s_bytes.len() {
        return 0;
    }

    let view_slice = std::slice::from_raw_parts(view.ptr as *const u8, view.len);
    if view_slice == s_bytes { 1 } else { 0 }
}

/// Materialize a StringView to an owned String (allocates)
/// Use this when you need to return a String from a function
#[no_mangle]
pub unsafe extern "C" fn stringview_to_string(view: *mut c_void) -> *mut c_char {
    if view.is_null() {
        return ptr::null_mut();
    }

    let view = &*(view as *const StringView);

    // Allocate space for string + null terminator
    let ptr = crate::memory_alloc::aether_safe_malloc(view.len + 1) as *mut c_char;
    if ptr.is_null() {
        return ptr::null_mut();
    }

    // Copy data
    ptr::copy_nonoverlapping(view.ptr, ptr, view.len);
    // Null terminate
    *ptr.add(view.len) = 0;

    ptr
}

/// Free a StringView handle (does NOT free the underlying string data)
#[no_mangle]
pub unsafe extern "C" fn stringview_free(view: *mut c_void) {
    if !view.is_null() {
        let _ = Box::from_raw(view as *mut StringView);
        VIEW_COUNTER.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Find first occurrence of byte in StringView, returns -1 if not found
#[no_mangle]
pub unsafe extern "C" fn stringview_find_byte(view: *mut c_void, byte: c_int) -> c_int {
    if view.is_null() {
        return -1;
    }

    let view = &*(view as *const StringView);
    let byte = byte as u8;

    for i in 0..view.len {
        if *view.ptr.add(i) as u8 == byte {
            return i as c_int;
        }
    }

    -1
}

/// Trim whitespace from both ends (returns new view, zero-copy)
#[no_mangle]
pub unsafe extern "C" fn stringview_trim(view: *mut c_void) -> *mut c_void {
    if view.is_null() {
        return ptr::null_mut();
    }

    let view = &*(view as *const StringView);

    if view.len == 0 {
        return stringview_slice(view as *const StringView as *mut c_void, 0, 0);
    }

    let bytes = std::slice::from_raw_parts(view.ptr as *const u8, view.len);

    // Find first non-whitespace
    let start = bytes.iter()
        .position(|&b| !b.is_ascii_whitespace())
        .unwrap_or(view.len);

    // Find last non-whitespace
    let end = bytes.iter()
        .rposition(|&b| !b.is_ascii_whitespace())
        .map(|i| i + 1)
        .unwrap_or(start);

    stringview_slice(view as *const StringView as *mut c_void, start as c_int, (end - start) as c_int)
}

/// Check if StringView starts with a prefix (C string)
#[no_mangle]
pub unsafe extern "C" fn stringview_starts_with(view: *mut c_void, prefix: *const c_char) -> c_int {
    if view.is_null() || prefix.is_null() {
        return 0;
    }

    let view = &*(view as *const StringView);
    let prefix_bytes = CStr::from_ptr(prefix).to_bytes();

    if prefix_bytes.len() > view.len {
        return 0;
    }

    let view_prefix = std::slice::from_raw_parts(view.ptr as *const u8, prefix_bytes.len());
    if view_prefix == prefix_bytes { 1 } else { 0 }
}

// =============================================================================
// Arena - Bump-pointer allocator for fast allocation with bulk deallocation
// =============================================================================

/// Arena structure - a simple bump allocator
#[repr(C)]
pub struct Arena {
    /// Base pointer to allocated memory
    base: *mut u8,
    /// Current offset (next allocation starts here)
    offset: usize,
    /// Total capacity
    capacity: usize,
    /// Number of allocations (for debugging/verification)
    alloc_count: usize,
}

/// Create a new Arena with given capacity
#[no_mangle]
pub unsafe extern "C" fn arena_create(capacity: c_int) -> *mut c_void {
    let capacity = if capacity <= 0 { 4096 } else { capacity as usize };

    let base = crate::memory_alloc::aether_safe_malloc(capacity) as *mut u8;
    if base.is_null() {
        return ptr::null_mut();
    }

    let arena = Box::new(Arena {
        base,
        offset: 0,
        capacity,
        alloc_count: 0,
    });

    Box::into_raw(arena) as *mut c_void
}

/// Allocate memory from arena (returns pointer, NOT null-terminated)
/// Alignment is 8 bytes
#[no_mangle]
pub unsafe extern "C" fn arena_alloc(arena: *mut c_void, size: c_int) -> *mut c_void {
    if arena.is_null() || size <= 0 {
        return ptr::null_mut();
    }

    let arena = &mut *(arena as *mut Arena);
    let size = size as usize;

    // Align to 8 bytes
    let aligned_offset = (arena.offset + 7) & !7;

    if aligned_offset + size > arena.capacity {
        // Arena is full - return null (could grow, but keeping it simple)
        return ptr::null_mut();
    }

    let ptr = arena.base.add(aligned_offset);
    arena.offset = aligned_offset + size;
    arena.alloc_count += 1;

    ptr as *mut c_void
}

/// Allocate a string in the arena (copies from source, adds null terminator)
#[no_mangle]
pub unsafe extern "C" fn arena_alloc_string(arena: *mut c_void, src: *const c_char) -> *mut c_char {
    if arena.is_null() || src.is_null() {
        return ptr::null_mut();
    }

    let len = CStr::from_ptr(src).to_bytes().len();
    let ptr = arena_alloc(arena, (len + 1) as c_int) as *mut c_char;

    if ptr.is_null() {
        return ptr::null_mut();
    }

    ptr::copy_nonoverlapping(src, ptr, len);
    *ptr.add(len) = 0;

    ptr
}

/// Allocate a string from a StringView in the arena
#[no_mangle]
pub unsafe extern "C" fn arena_alloc_from_view(arena: *mut c_void, view: *mut c_void) -> *mut c_char {
    if arena.is_null() || view.is_null() {
        return ptr::null_mut();
    }

    let view = &*(view as *const StringView);
    let ptr = arena_alloc(arena, (view.len + 1) as c_int) as *mut c_char;

    if ptr.is_null() {
        return ptr::null_mut();
    }

    ptr::copy_nonoverlapping(view.ptr, ptr, view.len);
    *ptr.add(view.len) = 0;

    ptr
}

/// Reset arena (free all allocations but keep capacity)
#[no_mangle]
pub unsafe extern "C" fn arena_reset(arena: *mut c_void) {
    if arena.is_null() {
        return;
    }

    let arena = &mut *(arena as *mut Arena);
    arena.offset = 0;
    arena.alloc_count = 0;
}

/// Get number of bytes used in arena
#[no_mangle]
pub unsafe extern "C" fn arena_bytes_used(arena: *mut c_void) -> c_int {
    if arena.is_null() {
        return 0;
    }
    let arena = &*(arena as *const Arena);
    arena.offset as c_int
}

/// Get remaining capacity in arena
#[no_mangle]
pub unsafe extern "C" fn arena_bytes_remaining(arena: *mut c_void) -> c_int {
    if arena.is_null() {
        return 0;
    }
    let arena = &*(arena as *const Arena);
    (arena.capacity - arena.offset) as c_int
}

/// Destroy arena and free all memory
#[no_mangle]
pub unsafe extern "C" fn arena_destroy(arena: *mut c_void) {
    if arena.is_null() {
        return;
    }

    let arena = Box::from_raw(arena as *mut Arena);
    crate::memory_alloc::aether_safe_free(arena.base as *mut c_void);
    // arena is dropped here, freeing the Arena struct
}

// =============================================================================
// Pure functions that can be implemented in Aether but provided here for speed
// These are candidates for replacement with pure Aether + verification
// =============================================================================

/// Check if byte is ASCII lowercase (a-z)
#[no_mangle]
pub extern "C" fn byte_is_lower(b: c_int) -> c_int {
    let b = b as u8;
    if b >= b'a' && b <= b'z' { 1 } else { 0 }
}

/// Check if byte is ASCII uppercase (A-Z)
#[no_mangle]
pub extern "C" fn byte_is_upper(b: c_int) -> c_int {
    let b = b as u8;
    if b >= b'A' && b <= b'Z' { 1 } else { 0 }
}

/// Check if byte is ASCII digit (0-9)
#[no_mangle]
pub extern "C" fn byte_is_digit(b: c_int) -> c_int {
    let b = b as u8;
    if b >= b'0' && b <= b'9' { 1 } else { 0 }
}

/// Check if byte is ASCII whitespace
#[no_mangle]
pub extern "C" fn byte_is_whitespace(b: c_int) -> c_int {
    let b = b as u8;
    if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' { 1 } else { 0 }
}

/// Convert byte to uppercase (ASCII only)
#[no_mangle]
pub extern "C" fn byte_to_upper(b: c_int) -> c_int {
    let b = b as u8;
    if b >= b'a' && b <= b'z' {
        (b - 32) as c_int
    } else {
        b as c_int
    }
}

/// Convert byte to lowercase (ASCII only)
#[no_mangle]
pub extern "C" fn byte_to_lower(b: c_int) -> c_int {
    let b = b as u8;
    if b >= b'A' && b <= b'Z' {
        (b + 32) as c_int
    } else {
        b as c_int
    }
}

/// Create a single-character string from a byte (allocates)
#[no_mangle]
pub unsafe extern "C" fn byte_to_string(b: c_int) -> *mut c_char {
    let ptr = crate::memory_alloc::aether_safe_malloc(2) as *mut c_char;
    if ptr.is_null() {
        return ptr::null_mut();
    }
    *ptr = b as c_char;
    *ptr.add(1) = 0;
    ptr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stringview_basic() {
        unsafe {
            crate::memory_alloc::aether_memory_init();
            let s = b"Hello, World!\0".as_ptr() as *const c_char;

            // Create view of entire string
            let view = stringview_from_string(s);
            assert!(!view.is_null());
            assert_eq!(stringview_length(view), 13);

            // Test char_at
            assert_eq!(stringview_char_at(view, 0), b'H' as c_int);
            assert_eq!(stringview_char_at(view, 7), b'W' as c_int);

            stringview_free(view);
        }
    }

    #[test]
    fn test_stringview_slice() {
        unsafe {
            crate::memory_alloc::aether_memory_init();
            let s = b"Hello, World!\0".as_ptr() as *const c_char;
            let view = stringview_from_string(s);

            // Get "World"
            let sub = stringview_slice(view, 7, 5);
            assert!(!sub.is_null());
            assert_eq!(stringview_length(sub), 5);

            // Materialize to string
            let str_ptr = stringview_to_string(sub);
            assert!(!str_ptr.is_null());
            let materialized = CStr::from_ptr(str_ptr).to_str().unwrap();
            assert_eq!(materialized, "World");

            crate::memory_alloc::aether_safe_free(str_ptr as *mut c_void);
            stringview_free(sub);
            stringview_free(view);
        }
    }

    #[test]
    fn test_stringview_trim() {
        unsafe {
            crate::memory_alloc::aether_memory_init();
            let s = b"  Hello  \0".as_ptr() as *const c_char;
            let view = stringview_from_string(s);

            let trimmed = stringview_trim(view);
            assert!(!trimmed.is_null());
            assert_eq!(stringview_length(trimmed), 5);

            let str_ptr = stringview_to_string(trimmed);
            let materialized = CStr::from_ptr(str_ptr).to_str().unwrap();
            assert_eq!(materialized, "Hello");

            crate::memory_alloc::aether_safe_free(str_ptr as *mut c_void);
            stringview_free(trimmed);
            stringview_free(view);
        }
    }

    #[test]
    fn test_arena_basic() {
        unsafe {
            crate::memory_alloc::aether_memory_init();
            let arena = arena_create(1024);
            assert!(!arena.is_null());

            // Allocate some strings
            let s1 = b"Hello\0".as_ptr() as *const c_char;
            let s2 = b"World\0".as_ptr() as *const c_char;

            let p1 = arena_alloc_string(arena, s1);
            let p2 = arena_alloc_string(arena, s2);

            assert!(!p1.is_null());
            assert!(!p2.is_null());

            assert_eq!(CStr::from_ptr(p1).to_str().unwrap(), "Hello");
            assert_eq!(CStr::from_ptr(p2).to_str().unwrap(), "World");

            // Check bytes used
            assert!(arena_bytes_used(arena) > 0);

            // Reset and verify
            arena_reset(arena);
            assert_eq!(arena_bytes_used(arena), 0);

            arena_destroy(arena);
        }
    }

    #[test]
    fn test_byte_operations() {
        assert_eq!(byte_is_lower(b'a' as c_int), 1);
        assert_eq!(byte_is_lower(b'z' as c_int), 1);
        assert_eq!(byte_is_lower(b'A' as c_int), 0);

        assert_eq!(byte_is_upper(b'A' as c_int), 1);
        assert_eq!(byte_is_upper(b'Z' as c_int), 1);
        assert_eq!(byte_is_upper(b'a' as c_int), 0);

        assert_eq!(byte_to_upper(b'a' as c_int), b'A' as c_int);
        assert_eq!(byte_to_lower(b'A' as c_int), b'a' as c_int);
    }
}
