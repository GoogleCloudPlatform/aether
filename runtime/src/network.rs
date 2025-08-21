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

//! Network operations for AetherScript runtime
//! 
//! Provides TCP socket operations with C FFI

use std::ffi::{c_char, c_int, CStr};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::ptr;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

// Global socket manager
lazy_static::lazy_static! {
    static ref SOCKET_MANAGER: Mutex<SocketManager> = Mutex::new(SocketManager::new());
}

struct SocketManager {
    next_id: i32,
    tcp_streams: HashMap<i32, TcpStream>,
    tcp_listeners: HashMap<i32, TcpListener>,
}

impl SocketManager {
    fn new() -> Self {
        Self {
            next_id: 1,
            tcp_streams: HashMap::new(),
            tcp_listeners: HashMap::new(),
        }
    }
    
    fn add_stream(&mut self, stream: TcpStream) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.tcp_streams.insert(id, stream);
        id
    }
    
    fn add_listener(&mut self, listener: TcpListener) -> i32 {
        let id = self.next_id;
        self.next_id += 1;
        self.tcp_listeners.insert(id, listener);
        id
    }
    
    fn get_stream(&mut self, id: i32) -> Option<&mut TcpStream> {
        self.tcp_streams.get_mut(&id)
    }
    
    fn get_listener(&self, id: i32) -> Option<&TcpListener> {
        self.tcp_listeners.get(&id)
    }
    
    fn remove_stream(&mut self, id: i32) -> Option<TcpStream> {
        self.tcp_streams.remove(&id)
    }
    
    fn remove_listener(&mut self, id: i32) -> Option<TcpListener> {
        self.tcp_listeners.remove(&id)
    }
}

/// Create a TCP listener on the specified port
/// Returns socket ID on success, -1 on error
#[no_mangle]
pub unsafe extern "C" fn tcp_listen(port: c_int) -> c_int {
    let addr = format!("0.0.0.0:{}", port);
    
    match TcpListener::bind(&addr) {
        Ok(listener) => {
            let mut manager = SOCKET_MANAGER.lock().unwrap();
            manager.add_listener(listener)
        }
        Err(_) => -1
    }
}

/// Accept a connection on a listener socket
/// Returns new socket ID on success, -1 on error
#[no_mangle]
pub unsafe extern "C" fn tcp_accept(listener_id: c_int) -> c_int {
    let manager = SOCKET_MANAGER.lock().unwrap();
    
    if let Some(listener) = manager.get_listener(listener_id) {
        // Clone the listener to avoid holding the lock during accept
        let listener_clone = listener.try_clone();
        drop(manager);
        
        if let Ok(listener) = listener_clone {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let mut manager = SOCKET_MANAGER.lock().unwrap();
                    manager.add_stream(stream)
                }
                Err(_) => -1
            }
        } else {
            -1
        }
    } else {
        -1
    }
}

/// Connect to a TCP server
/// Returns socket ID on success, -1 on error
#[no_mangle]
pub unsafe extern "C" fn tcp_connect(host: *const c_char, port: c_int) -> c_int {
    if host.is_null() {
        return -1;
    }
    
    let host_str = match CStr::from_ptr(host).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    let addr = format!("{}:{}", host_str, port);
    
    match TcpStream::connect(&addr) {
        Ok(stream) => {
            let mut manager = SOCKET_MANAGER.lock().unwrap();
            manager.add_stream(stream)
        }
        Err(_) => -1
    }
}

/// Read data from a socket
/// Returns number of bytes read, or -1 on error
#[no_mangle]
pub unsafe extern "C" fn tcp_read(socket_id: c_int, buffer: *mut c_char, buffer_size: c_int) -> c_int {
    if buffer.is_null() || buffer_size <= 0 {
        return -1;
    }
    
    let mut manager = SOCKET_MANAGER.lock().unwrap();
    
    if let Some(stream) = manager.get_stream(socket_id) {
        let mut vec_buffer = vec![0u8; buffer_size as usize];
        
        match stream.read(&mut vec_buffer) {
            Ok(n) => {
                ptr::copy_nonoverlapping(vec_buffer.as_ptr(), buffer as *mut u8, n);
                n as c_int
            }
            Err(_) => -1
        }
    } else {
        -1
    }
}

/// Write data to a socket
/// Returns number of bytes written, or -1 on error
#[no_mangle]
pub unsafe extern "C" fn tcp_write(socket_id: c_int, data: *const c_char, data_size: c_int) -> c_int {
    if data.is_null() || data_size <= 0 {
        return -1;
    }
    
    let mut manager = SOCKET_MANAGER.lock().unwrap();
    
    if let Some(stream) = manager.get_stream(socket_id) {
        let slice = std::slice::from_raw_parts(data as *const u8, data_size as usize);
        
        match stream.write(slice) {
            Ok(n) => n as c_int,
            Err(_) => -1
        }
    } else {
        -1
    }
}

/// Read data from socket into allocated string buffer
/// Returns allocated C string, or null on error
/// Caller must free the returned pointer
#[no_mangle]
pub unsafe extern "C" fn tcp_read_string(socket_id: c_int, max_size: c_int) -> *mut c_char {
    if max_size <= 0 {
        return ptr::null_mut();
    }
    
    let mut manager = SOCKET_MANAGER.lock().unwrap();
    
    if let Some(stream) = manager.get_stream(socket_id) {
        let mut buffer = vec![0u8; max_size as usize];
        
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                // Truncate buffer to actual size read
                buffer.truncate(bytes_read);
                
                // Ensure null termination
                buffer.push(0);
                
                // Allocate C string
                let len = buffer.len();
                let ptr = crate::memory::aether_malloc(len as c_int) as *mut c_char;
                
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(buffer.as_ptr() as *const c_char, ptr, len);
                }
                
                ptr
            }
            _ => ptr::null_mut()
        }
    } else {
        ptr::null_mut()
    }
}

/// Close a socket
#[no_mangle]
pub unsafe extern "C" fn tcp_close(socket_id: c_int) {
    let mut manager = SOCKET_MANAGER.lock().unwrap();
    manager.remove_stream(socket_id);
    manager.remove_listener(socket_id);
}

/// Set socket timeout (in milliseconds)
/// Returns 0 on success, -1 on error
#[no_mangle]
pub unsafe extern "C" fn tcp_set_timeout(socket_id: c_int, timeout_ms: c_int) -> c_int {
    let mut manager = SOCKET_MANAGER.lock().unwrap();
    
    if let Some(stream) = manager.get_stream(socket_id) {
        let duration = if timeout_ms > 0 {
            Some(Duration::from_millis(timeout_ms as u64))
        } else {
            None
        };
        
        if stream.set_read_timeout(duration).is_ok() && 
           stream.set_write_timeout(duration).is_ok() {
            0
        } else {
            -1
        }
    } else {
        -1
    }
}

/// Get the local address of a socket
/// Returns port number on success, -1 on error
#[no_mangle]
pub unsafe extern "C" fn tcp_get_local_port(socket_id: c_int) -> c_int {
    let manager = SOCKET_MANAGER.lock().unwrap();
    
    if let Some(stream) = manager.tcp_streams.get(&socket_id) {
        match stream.local_addr() {
            Ok(addr) => addr.port() as c_int,
            Err(_) => -1
        }
    } else if let Some(listener) = manager.tcp_listeners.get(&socket_id) {
        match listener.local_addr() {
            Ok(addr) => addr.port() as c_int,
            Err(_) => -1
        }
    } else {
        -1
    }
}

/// Get the peer address of a socket
/// Writes the address string to the buffer
/// Returns 0 on success, -1 on error
#[no_mangle]
pub unsafe extern "C" fn tcp_get_peer_address(socket_id: c_int, buffer: *mut c_char, buffer_size: c_int) -> c_int {
    if buffer.is_null() || buffer_size <= 0 {
        return -1;
    }
    
    let manager = SOCKET_MANAGER.lock().unwrap();
    
    if let Some(stream) = manager.tcp_streams.get(&socket_id) {
        match stream.peer_addr() {
            Ok(addr) => {
                let addr_str = format!("{}\0", addr);
                let len = std::cmp::min(addr_str.len(), buffer_size as usize);
                ptr::copy_nonoverlapping(addr_str.as_ptr() as *const c_char, buffer, len);
                0
            }
            Err(_) => -1
        }
    } else {
        -1
    }
}

/// Check if data is available to read (non-blocking)
/// Returns 1 if data available, 0 if not, -1 on error
#[no_mangle]
pub unsafe extern "C" fn tcp_data_available(socket_id: c_int) -> c_int {
    let mut manager = SOCKET_MANAGER.lock().unwrap();
    
    if let Some(stream) = manager.get_stream(socket_id) {
        // Set non-blocking temporarily
        if let Ok(()) = stream.set_nonblocking(true) {
            let mut buffer = [0u8; 1];
            match stream.peek(&mut buffer) {
                Ok(n) if n > 0 => {
                    let _ = stream.set_nonblocking(false);
                    1
                }
                Ok(_) => {
                    let _ = stream.set_nonblocking(false);
                    0
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    let _ = stream.set_nonblocking(false);
                    0
                }
                Err(_) => {
                    let _ = stream.set_nonblocking(false);
                    -1
                }
            }
        } else {
            -1
        }
    } else {
        -1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_tcp_listen_and_accept() {
        unsafe {
            // Create listener
            let listener_id = tcp_listen(0); // Use port 0 for automatic assignment
            assert!(listener_id > 0);
            
            // Get the actual port
            let port = tcp_get_local_port(listener_id);
            assert!(port > 0);
            
            // Spawn client thread
            let client_thread = thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                let client_id = tcp_connect("127.0.0.1\0".as_ptr() as *const c_char, port);
                assert!(client_id > 0);
                tcp_close(client_id);
            });
            
            // Accept connection
            let server_socket = tcp_accept(listener_id);
            assert!(server_socket > 0);
            
            // Clean up
            tcp_close(server_socket);
            tcp_close(listener_id);
            
            client_thread.join().unwrap();
        }
    }
    
    #[test]
    fn test_tcp_read_write() {
        unsafe {
            // Setup server
            let listener_id = tcp_listen(0);
            let port = tcp_get_local_port(listener_id);
            
            // Client thread
            let client_thread = thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                let client_id = tcp_connect("127.0.0.1\0".as_ptr() as *const c_char, port);
                
                let msg = "Hello, server!\0";
                let written = tcp_write(client_id, msg.as_ptr() as *const c_char, msg.len() as c_int - 1);
                assert_eq!(written, 14);
                
                tcp_close(client_id);
            });
            
            // Server accepts and reads
            let server_socket = tcp_accept(listener_id);
            let mut buffer = vec![0u8; 256];
            let read = tcp_read(server_socket, buffer.as_mut_ptr() as *mut c_char, 256);
            assert_eq!(read, 14);
            assert_eq!(&buffer[..14], b"Hello, server!");
            
            // Clean up
            tcp_close(server_socket);
            tcp_close(listener_id);
            
            client_thread.join().unwrap();
        }
    }
}