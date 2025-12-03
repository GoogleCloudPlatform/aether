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

//! I/O operations runtime support

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::ptr;

/// File handle structure
#[repr(C)]
pub struct FileHandle {
    file: *mut File,
    mode: i32, // 0 = read, 1 = write, 2 = append
}

/// Open a file
#[no_mangle]
pub unsafe extern "C" fn aether_open_file(path: *const c_char, mode: *const c_char) -> *mut FileHandle {
    if path.is_null() || mode.is_null() {
        return ptr::null_mut();
    }
    
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let mode_str = match CStr::from_ptr(mode).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let (file, mode_num) = match mode_str {
        "r" => {
            match File::open(path_str) {
                Ok(f) => (f, 0),
                Err(_) => return ptr::null_mut(),
            }
        },
        "w" => {
            match File::create(path_str) {
                Ok(f) => (f, 1),
                Err(_) => return ptr::null_mut(),
            }
        },
        "a" => {
            match OpenOptions::new().append(true).create(true).open(path_str) {
                Ok(f) => (f, 2),
                Err(_) => return ptr::null_mut(),
            }
        },
        _ => return ptr::null_mut(),
    };
    
    // Internal structs use memory allocator (GC or standard)
    let handle = crate::memory::aether_malloc(std::mem::size_of::<FileHandle>() as c_int) as *mut FileHandle;
    if handle.is_null() {
        return ptr::null_mut();
    }
    
    let file_box = Box::new(file);
    (*handle).file = Box::into_raw(file_box);
    (*handle).mode = mode_num;
    
    handle
}

/// Close a file
#[no_mangle]
pub unsafe extern "C" fn aether_close_file(handle: *mut FileHandle) {
    if handle.is_null() {
        return;
    }
    
    if !(*handle).file.is_null() {
        let _ = Box::from_raw((*handle).file);
    }
    
    crate::memory::aether_free(handle as *mut c_void);
}

/// Get file size
#[no_mangle]
pub unsafe extern "C" fn aether_file_size(handle: *mut FileHandle) -> c_int {
    if handle.is_null() || (*handle).file.is_null() {
        return -1;
    }
    
    let file = &mut *(*handle).file;
    match file.metadata() {
        Ok(metadata) => metadata.len() as c_int,
        Err(_) => -1,
    }
}

/// Read from file
#[no_mangle]
pub unsafe extern "C" fn aether_read_file(handle: *mut FileHandle, buffer: *mut c_char, size: c_int) -> c_int {
    if handle.is_null() || (*handle).file.is_null() || buffer.is_null() || size <= 0 {
        return -1;
    }
    
    if (*handle).mode != 0 {
        return -1; // Not open for reading
    }
    
    let file = &mut *(*handle).file;
    let mut vec = vec![0u8; size as usize];
    
    match file.read(&mut vec) {
        Ok(bytes_read) => {
            ptr::copy_nonoverlapping(vec.as_ptr(), buffer as *mut u8, bytes_read);
            bytes_read as c_int
        },
        Err(_) => -1,
    }
}

/// Write to file
#[no_mangle]
pub unsafe extern "C" fn aether_write_file(handle: *mut FileHandle, data: *const c_char, size: c_int) -> c_int {
    if handle.is_null() || (*handle).file.is_null() || data.is_null() || size <= 0 {
        return -1;
    }
    
    if (*handle).mode == 0 {
        return -1; // Not open for writing
    }
    
    let file = &mut *(*handle).file;
    let slice = std::slice::from_raw_parts(data as *const u8, size as usize);
    
    match file.write(slice) {
        Ok(bytes_written) => bytes_written as c_int,
        Err(_) => -1,
    }
}

/// Allocate a string buffer
#[no_mangle]
pub unsafe extern "C" fn aether_allocate_string(size: c_int) -> *mut c_char {
    if size <= 0 {
        return ptr::null_mut();
    }
    
    // UPDATED: Use memory_alloc to match lib.rs string_free
    let ptr = crate::memory_alloc::aether_safe_malloc((size + 1) as usize) as *mut c_char;
    if !ptr.is_null() {
        ptr::write_bytes(ptr, 0, (size + 1) as usize);
    }
    ptr
}

/// Print to stdout
#[no_mangle]
pub unsafe extern "C" fn aether_print(text: *const c_char) {
    if text.is_null() {
        return;
    }
    
    if let Ok(s) = CStr::from_ptr(text).to_str() {
        print!("{}", s);
        let _ = std::io::stdout().flush();
    }
}

/// Print an integer to stdout
#[no_mangle]
pub extern "C" fn print_int(value: c_int) {
    println!("{}", value);
}

/// Read line from stdin
#[no_mangle]
pub unsafe extern "C" fn aether_read_line(buffer: *mut c_char) -> c_int {
    if buffer.is_null() {
        return -1;
    }
    
    let stdin = std::io::stdin();
    let mut line = String::new();
    
    match stdin.read_line(&mut line) {
        Ok(_) => {
            // Remove newline if present
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            
            let c_string = match CString::new(line) {
                Ok(s) => s,
                Err(_) => return -1,
            };
            
            let bytes = c_string.as_bytes_with_nul();
            // Note: Caller is responsible for ensuring buffer is large enough
            // This function is unsafe and potentially dangerous if buffer is too small
            ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, buffer, bytes.len());
            
            (bytes.len() - 1) as c_int
        },
        Err(_) => -1,
    }
}

/// List directory contents
#[no_mangle]
pub unsafe extern "C" fn aether_list_directory(path: *const c_char, entries: *mut c_void) -> c_int {
    if path.is_null() || entries.is_null() {
        return -1;
    }
    
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    match std::fs::read_dir(path_str) {
        Ok(dir) => {
            let mut count = 0;
            let array_ptr = entries as *mut *mut c_char;
            
            for (i, entry) in dir.enumerate() {
                if i >= 1000 { break; } // Limit to 1000 entries
                
                if let Ok(entry) = entry {
                    if let Some(name) = entry.file_name().to_str() {
                        let c_string = match CString::new(name) {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        
                        let bytes = c_string.as_bytes_with_nul();
                        // Use memory_alloc for strings
                        let str_ptr = crate::memory_alloc::aether_safe_malloc(bytes.len()) as *mut c_char;
                        
                        if !str_ptr.is_null() {
                            ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, str_ptr, bytes.len());
                            *array_ptr.offset(i as isize) = str_ptr;
                            count += 1;
                        }
                    }
                }
            }
            
            count
        },
        Err(_) => -1,
    }
}

// Safely read file content to string with max size limit
// Matches stdlib/io.aether: read_file_safe(path: String, max_size: Int) -> String
#[no_mangle]
pub unsafe extern "C" fn read_file_safe(path: *const c_char, max_size: c_int) -> *mut c_char {
    if path.is_null() || max_size <= 0 {
        return ptr::null_mut();
    }
    
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let mut file = match File::open(path_str) {
        Ok(f) => f,
        Err(_) => return ptr::null_mut(),
    };
    
    // Check file size first
    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let read_len = std::cmp::min(file_len, max_size as u64) as usize;
    
    // Create vector of correct size
    let mut buffer = Vec::with_capacity(read_len);
    
    // Read up to max_size bytes
    // Use take() to limit read
    match file.take(max_size as u64).read_to_end(&mut buffer) {
        Ok(_) => {
            // Validate UTF-8
            match String::from_utf8(buffer) {
                Ok(s) => {
                    let result = format!("{}\0", s);
                    let len = result.len();
                    // Use memory_alloc for string return
                    let ptr = crate::memory_alloc::aether_safe_malloc(len) as *mut c_char;
                    if !ptr.is_null() {
                        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
                    }
                    ptr
                },
                Err(_) => ptr::null_mut(), // Invalid UTF-8
            }
        },
        Err(_) => ptr::null_mut(),
    }
}

// Write file safe
// Matches stdlib/io.aether: write_file_safe(path: String, content: String, append: Bool) -> Bool
#[no_mangle]
pub unsafe extern "C" fn write_file_safe(path: *const c_char, content: *const c_char, append: c_int) -> c_int {
    if path.is_null() || content.is_null() {
        return 0; // False
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    let content_slice = CStr::from_ptr(content).to_bytes();
    
    let mut options = OpenOptions::new();
    options.write(true).create(true);
    
    if append != 0 {
        options.append(true);
    } else {
        options.truncate(true);
    }
    
    match options.open(path_str) {
        Ok(mut file) => {
            match file.write_all(content_slice) {
                Ok(_) => 1, // True
                Err(_) => 0,
            }
        },
        Err(_) => 0,
    }
}
