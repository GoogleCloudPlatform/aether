//! HTTP protocol support functions for AetherScript
//! 
//! Provides HTTP server functionality with request routing and response generation

use std::ffi::{c_char, c_int, c_void, CStr};
use std::ptr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::io::{Read, Write};

// HTTP server context for request handling
#[repr(C)]
pub struct HttpRequestContext {
    method: *mut c_char,
    path: *mut c_char, 
    headers: *mut c_char,
    body: *mut c_char,
    socket_fd: c_int,
}

// Global HTTP server manager
lazy_static::lazy_static! {
    static ref HTTP_SERVERS: Mutex<HashMap<c_int, HttpServerInstance>> = Mutex::new(HashMap::new());
}

struct HttpServerInstance {
    port: c_int,
    handler: Option<extern "C" fn(*mut HttpRequestContext)>,
    running: Arc<Mutex<bool>>,
}

/// Create and start an HTTP server on the specified port
/// Returns server handle ID on success, -1 on failure
#[no_mangle]
pub unsafe extern "C" fn http_create_server(
    port: c_int,
    request_handler: extern "C" fn(*mut HttpRequestContext)
) -> c_int {
    let server_id = {
        match crate::network::tcp_listen(port) {
            -1 => return -1, // Failed to bind
            listener_fd => listener_fd,
        }
    };
    
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();
    
    let server_instance = HttpServerInstance {
        port,
        handler: Some(request_handler),
        running: running.clone(),
    };
    
    // Store server instance
    if let Ok(mut servers) = HTTP_SERVERS.lock() {
        servers.insert(server_id, server_instance);
    }
    
    // Spawn server thread
    thread::spawn(move || {
        http_server_loop(server_id, request_handler, running_clone);
    });
    
    server_id
}

/// Stop an HTTP server
#[no_mangle]
pub unsafe extern "C" fn http_stop_server(server_handle: c_int) {
    if let Ok(mut servers) = HTTP_SERVERS.lock() {
        if let Some(server) = servers.get(&server_handle) {
            if let Ok(mut running) = server.running.lock() {
                *running = false;
            }
        }
        servers.remove(&server_handle);
    }
    crate::network::tcp_close(server_handle);
}

/// Get request path from HTTP request context
#[no_mangle]
pub unsafe extern "C" fn http_get_request_path(request_ctx: *mut c_void) -> *mut c_char {
    if request_ctx.is_null() {
        return ptr::null_mut();
    }
    
    let ctx = request_ctx as *mut HttpRequestContext;
    let path = (*ctx).path;
    
    if path.is_null() {
        return ptr::null_mut();
    }
    
    // Clone the path string
    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let result = format!("{}\0", path_str);
    let len = result.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        std::ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Get request method from HTTP request context
#[no_mangle]
pub unsafe extern "C" fn http_get_request_method(request_ctx: *mut c_void) -> *mut c_char {
    if request_ctx.is_null() {
        return ptr::null_mut();
    }
    
    let ctx = request_ctx as *mut HttpRequestContext;
    let method = (*ctx).method;
    
    if method.is_null() {
        return ptr::null_mut();
    }
    
    // Clone the method string
    let method_str = match CStr::from_ptr(method).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    let result = format!("{}\0", method_str);
    let len = result.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        std::ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Send HTTP response
#[no_mangle]
pub unsafe extern "C" fn http_send_response(
    request_ctx: *mut c_void,
    status_code: c_int,
    content_type: *const c_char,
    body: *const c_char
) {
    if request_ctx.is_null() {
        return;
    }
    
    let ctx = request_ctx as *mut HttpRequestContext;
    let socket_fd = (*ctx).socket_fd;
    
    let content_type_str = if content_type.is_null() {
        "text/plain"
    } else {
        match CStr::from_ptr(content_type).to_str() {
            Ok(s) => s,
            Err(_) => "text/plain"
        }
    };
    
    let body_str = if body.is_null() {
        ""
    } else {
        match CStr::from_ptr(body).to_str() {
            Ok(s) => s,
            Err(_) => ""
        }
    };
    
    let status_text = match status_code {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "Unknown"
    };
    
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status_code, status_text, content_type_str, body_str.len(), body_str
    );
    
    crate::network::tcp_write(socket_fd, response.as_ptr() as *const c_char, response.len() as c_int);
}

/// HTTP server main loop (internal function)
fn http_server_loop(
    listener_fd: c_int,
    handler: extern "C" fn(*mut HttpRequestContext),
    running: Arc<Mutex<bool>>
) {
    loop {
        // Check if server should stop
        if let Ok(is_running) = running.lock() {
            if !*is_running {
                break;
            }
        }
        
        // Accept connection
        let client_fd = unsafe { crate::network::tcp_accept(listener_fd) };
        if client_fd < 0 {
            continue;
        }
        
        // Read HTTP request
        let buffer = unsafe { crate::network::tcp_read_string(client_fd, 4096) };
        if buffer.is_null() {
            unsafe { crate::network::tcp_close(client_fd) };
            continue;
        }
        
        unsafe {
            // Parse HTTP request
            let request_str = match CStr::from_ptr(buffer).to_str() {
                Ok(s) => s,
                Err(_) => {
                    crate::memory::aether_free(buffer as *mut c_void);
                    crate::network::tcp_close(client_fd);
                    continue;
                }
            };
            
            let (method, path) = parse_http_request_line(request_str);
            
            // Create request context
            let mut ctx = HttpRequestContext {
                method: create_c_string(&method),
                path: create_c_string(&path),
                headers: ptr::null_mut(),
                body: ptr::null_mut(),
                socket_fd: client_fd,
            };
            
            // Call user handler
            handler(&mut ctx);
            
            // Cleanup
            cleanup_request_context(&mut ctx);
            crate::memory::aether_free(buffer as *mut c_void);
            crate::network::tcp_close(client_fd);
        }
    }
}

/// Parse HTTP request line to extract method and path
fn parse_http_request_line(request: &str) -> (String, String) {
    let first_line = request.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    
    let method = parts.get(0).unwrap_or(&"GET").to_string();
    let path = parts.get(1).unwrap_or(&"/").to_string();
    
    (method, path)
}

/// Create C string from Rust string
unsafe fn create_c_string(s: &str) -> *mut c_char {
    let c_str = format!("{}\0", s);
    let len = c_str.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        std::ptr::copy_nonoverlapping(c_str.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Cleanup request context memory
unsafe fn cleanup_request_context(ctx: *mut HttpRequestContext) {
    if !(*ctx).method.is_null() {
        crate::memory::aether_free((*ctx).method as *mut c_void);
    }
    if !(*ctx).path.is_null() {
        crate::memory::aether_free((*ctx).path as *mut c_void);
    }
    if !(*ctx).headers.is_null() {
        crate::memory::aether_free((*ctx).headers as *mut c_void);
    }
    if !(*ctx).body.is_null() {
        crate::memory::aether_free((*ctx).body as *mut c_void);
    }
}

/// Legacy: Parse HTTP request and return method type (simplified)
/// Returns 1 for GET, 2 for POST, 0 for unknown/error
#[no_mangle]
pub unsafe extern "C" fn parse_request(request_data: *const c_char) -> *mut c_char {
    if request_data.is_null() {
        return ptr::null_mut();
    }
    
    let request = match CStr::from_ptr(request_data).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    
    // Extract method from first line
    let first_line = request.lines().next().unwrap_or("");
    let method = first_line.split_whitespace().next().unwrap_or("");
    
    let result = format!("{}\0", method);
    let len = result.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(result.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Check if request method is GET
#[no_mangle]
pub unsafe extern "C" fn is_get(method: *const c_char) -> c_int {
    if method.is_null() {
        return 0;
    }
    
    let method_str = match CStr::from_ptr(method).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    if method_str.eq_ignore_ascii_case("GET") { 1 } else { 0 }
}

/// Create HTTP response with status code and body
#[no_mangle]
pub unsafe extern "C" fn create_response(status_code: c_int, body: *const c_char) -> *mut c_char {
    let body_str = if body.is_null() {
        ""
    } else {
        match CStr::from_ptr(body).to_str() {
            Ok(s) => s,
            Err(_) => ""
        }
    };
    
    let status_text = match status_code {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "Unknown"
    };
    
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}\0",
        status_code, status_text, body_str.len(), body_str
    );
    
    let len = response.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(response.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Create JSON HTTP response
#[no_mangle]
pub unsafe extern "C" fn json_response(json_body: *const c_char, status_code: c_int) -> *mut c_char {
    let body_str = if json_body.is_null() {
        "{}"
    } else {
        match CStr::from_ptr(json_body).to_str() {
            Ok(s) => s,
            Err(_) => "{}"
        }
    };
    
    let status_text = match status_code {
        200 => "OK",
        404 => "Not Found", 
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "Unknown"
    };
    
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}\0",
        status_code, status_text, body_str.len(), body_str
    );
    
    let len = response.len();
    let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
    
    if !ptr.is_null() {
        ptr::copy_nonoverlapping(response.as_ptr() as *const c_char, ptr, len);
    }
    
    ptr
}

/// Network function aliases to match the example
#[no_mangle]
pub unsafe extern "C" fn tcp_server(_host: *const c_char, port: c_int) -> c_int {
    // For now, ignore host and just use port
    crate::network::tcp_listen(port)
}

#[no_mangle]
pub unsafe extern "C" fn socket_accept(server_fd: c_int) -> c_int {
    crate::network::tcp_accept(server_fd)
}

#[no_mangle]
pub unsafe extern "C" fn socket_receive(socket_fd: c_int, buffer_size: c_int) -> *mut c_char {
    // Allocate buffer
    let buffer = crate::memory::aether_malloc(buffer_size) as *mut c_char;
    if buffer.is_null() {
        return ptr::null_mut();
    }
    
    // Read data
    let bytes_read = crate::network::tcp_read(socket_fd, buffer, buffer_size);
    if bytes_read <= 0 {
        crate::memory::aether_free(buffer as *mut std::ffi::c_void);
        return ptr::null_mut();
    }
    
    buffer
}

#[no_mangle]
pub unsafe extern "C" fn socket_send(socket_fd: c_int, data: *const c_char) -> c_int {
    if data.is_null() {
        return -1;
    }
    
    let data_str = match CStr::from_ptr(data).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    crate::network::tcp_write(socket_fd, data, data_str.len() as c_int)
}

#[no_mangle]
pub unsafe extern "C" fn socket_close(socket_fd: c_int) {
    crate::network::tcp_close(socket_fd);
}