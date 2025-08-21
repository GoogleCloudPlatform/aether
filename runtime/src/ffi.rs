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

//! FFI Runtime Support
//! 
//! Provides runtime support for Foreign Function Interface operations

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

#[cfg(unix)]
use libc::{dlopen, dlsym, dlclose, RTLD_LAZY};

#[cfg(windows)]
use winapi::um::libloaderapi::{LoadLibraryA, GetProcAddress, FreeLibrary};

/// Handle for a dynamically loaded library
pub struct LibraryHandle {
    #[cfg(unix)]
    handle: *mut c_void,
    #[cfg(windows)]
    handle: winapi::shared::minwindef::HMODULE,
}

// Mark as Send and Sync since we're handling thread safety ourselves
unsafe impl Send for LibraryHandle {}
unsafe impl Sync for LibraryHandle {}

/// Global registry of loaded libraries
lazy_static! {
    static ref LOADED_LIBRARIES: Mutex<HashMap<String, LibraryHandle>> = Mutex::new(HashMap::new());
}

/// Load a dynamic library
#[no_mangle]
pub unsafe extern "C" fn aether_load_library(name: *const c_char) -> *mut c_void {
    if name.is_null() {
        return ptr::null_mut();
    }
    
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    // Check if already loaded
    let mut libraries = LOADED_LIBRARIES.lock().unwrap();
    if libraries.contains_key(name_str) {
        return name_str.as_ptr() as *mut c_void; // Return a non-null sentinel
    }
    
    #[cfg(unix)]
    {
        let handle = dlopen(name, RTLD_LAZY);
        if handle.is_null() {
            return ptr::null_mut();
        }
        
        libraries.insert(name_str.to_string(), LibraryHandle { handle });
        handle
    }
    
    #[cfg(windows)]
    {
        let handle = LoadLibraryA(name);
        if handle.is_null() {
            return ptr::null_mut();
        }
        
        libraries.insert(name_str.to_string(), LibraryHandle { handle });
        handle as *mut c_void
    }
}

/// Get a symbol from a loaded library
#[no_mangle]
pub unsafe extern "C" fn aether_get_symbol(library: *const c_char, symbol: *const c_char) -> *mut c_void {
    if library.is_null() || symbol.is_null() {
        return ptr::null_mut();
    }
    
    let library_str = match CStr::from_ptr(library).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let symbol_str = match CStr::from_ptr(symbol).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let libraries = LOADED_LIBRARIES.lock().unwrap();
    let lib_handle = match libraries.get(library_str) {
        Some(handle) => handle,
        None => return ptr::null_mut(),
    };
    
    #[cfg(unix)]
    {
        let symbol_cstr = match CString::new(symbol_str) {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        dlsym(lib_handle.handle, symbol_cstr.as_ptr())
    }
    
    #[cfg(windows)]
    {
        let symbol_cstr = match CString::new(symbol_str) {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        GetProcAddress(lib_handle.handle, symbol_cstr.as_ptr() as *const i8) as *mut c_void
    }
}

/// Unload a dynamic library
#[no_mangle]
pub unsafe extern "C" fn aether_unload_library(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }
    
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let mut libraries = LOADED_LIBRARIES.lock().unwrap();
    if let Some(lib_handle) = libraries.remove(name_str) {
        #[cfg(unix)]
        {
            if dlclose(lib_handle.handle) == 0 {
                0
            } else {
                -1
            }
        }
        
        #[cfg(windows)]
        {
            if FreeLibrary(lib_handle.handle) != 0 {
                0
            } else {
                -1
            }
        }
    } else {
        -1
    }
}

/// Convert an AetherScript string to a C string
#[no_mangle]
pub unsafe extern "C" fn aether_string_to_cstr(s: *const c_char, len: c_int) -> *mut c_char {
    if s.is_null() || len < 0 {
        return ptr::null_mut();
    }
    
    // Create a new C string with null terminator
    
    // Allocate memory for the C string (length + 1 for null terminator)
    let cstr_ptr = crate::memory::aether_malloc((len + 1) as c_int) as *mut c_char;
    if cstr_ptr.is_null() {
        return ptr::null_mut();
    }
    
    // Copy the bytes
    ptr::copy_nonoverlapping(s, cstr_ptr, len as usize);
    
    // Add null terminator
    *cstr_ptr.add(len as usize) = 0;
    
    cstr_ptr
}

/// Convert a C string to an AetherScript string (duplicates the C string)
#[no_mangle]
pub unsafe extern "C" fn aether_cstr_to_string(cstr: *const c_char) -> *mut c_char {
    if cstr.is_null() {
        return ptr::null_mut();
    }
    
    crate::memory::aether_strdup(cstr)
}

/// Wrapper for callback function pointers
struct CallbackPtr(*mut c_void);

// Mark as Send and Sync since we're handling thread safety ourselves
unsafe impl Send for CallbackPtr {}
unsafe impl Sync for CallbackPtr {}

/// Register a callback function
lazy_static! {
    static ref CALLBACKS: Mutex<HashMap<String, CallbackPtr>> = Mutex::new(HashMap::new());
}

#[no_mangle]
pub unsafe extern "C" fn aether_register_callback(name: *const c_char, func_ptr: *mut c_void) -> c_int {
    if name.is_null() || func_ptr.is_null() {
        return -1;
    }
    
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let mut callbacks = CALLBACKS.lock().unwrap();
    callbacks.insert(name_str.to_string(), CallbackPtr(func_ptr));
    0
}

/// Get a registered callback
#[no_mangle]
pub unsafe extern "C" fn aether_get_callback(name: *const c_char) -> *mut c_void {
    if name.is_null() {
        return ptr::null_mut();
    }
    
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let callbacks = CALLBACKS.lock().unwrap();
    callbacks.get(name_str).map(|cb| cb.0).unwrap_or(ptr::null_mut())
}

/// Unregister a callback
#[no_mangle]
pub unsafe extern "C" fn aether_unregister_callback(name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }
    
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let mut callbacks = CALLBACKS.lock().unwrap();
    if callbacks.remove(name_str).is_some() {
        0
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    
    #[test]
    fn test_string_conversion() {
        unsafe {
            // Test string to C string
            let aether_str = b"Hello, FFI!";
            let cstr = aether_string_to_cstr(aether_str.as_ptr() as *const c_char, aether_str.len() as c_int);
            assert!(!cstr.is_null());
            
            // Verify the content
            let cstr_rust = CStr::from_ptr(cstr);
            assert_eq!(cstr_rust.to_str().unwrap(), "Hello, FFI!");
            
            // Test C string to string
            let test_cstr = CString::new("Test String").unwrap();
            let result = aether_cstr_to_string(test_cstr.as_ptr());
            assert!(!result.is_null());
            
            // Verify the content
            let result_str = CStr::from_ptr(result);
            assert_eq!(result_str.to_str().unwrap(), "Test String");
            
            // Clean up
            crate::memory::aether_free(cstr as *mut c_void);
            crate::memory::aether_free(result as *mut c_void);
        }
    }
    
    #[test]
    fn test_callback_registration() {
        unsafe {
            let callback_name = CString::new("test_callback").unwrap();
            let func_ptr = 0x12345678 as *mut c_void;
            
            // Register callback
            let result = aether_register_callback(callback_name.as_ptr(), func_ptr);
            assert_eq!(result, 0);
            
            // Get callback
            let retrieved = aether_get_callback(callback_name.as_ptr());
            assert_eq!(retrieved, func_ptr);
            
            // Unregister callback
            let result = aether_unregister_callback(callback_name.as_ptr());
            assert_eq!(result, 0);
            
            // Should be null after unregistering
            let retrieved = aether_get_callback(callback_name.as_ptr());
            assert!(retrieved.is_null());
        }
    }
}