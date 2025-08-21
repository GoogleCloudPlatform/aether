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

//! Concurrency primitives runtime support

use std::ffi::c_int;
use std::sync::{Arc, Mutex, mpsc, atomic::{AtomicI32, Ordering}};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Mutex as StdMutex;

// Global thread registry for tracking threads
lazy_static::lazy_static! {
    static ref THREAD_REGISTRY: StdMutex<HashMap<i32, thread::JoinHandle<()>>> = StdMutex::new(HashMap::new());
    static ref NEXT_THREAD_ID: AtomicI32 = AtomicI32::new(1);
    static ref NEXT_MUTEX_ID: AtomicI32 = AtomicI32::new(1);
    static ref MUTEX_REGISTRY: StdMutex<HashMap<i32, Arc<Mutex<MutexState>>>> = StdMutex::new(HashMap::new());
    static ref NEXT_CHANNEL_ID: AtomicI32 = AtomicI32::new(1);
    static ref CHANNEL_REGISTRY: StdMutex<HashMap<i32, ChannelPair>> = StdMutex::new(HashMap::new());
}

struct MutexState {
    locked: bool,
    owner_thread: i32,
}

enum ChannelSender {
    Bounded(mpsc::SyncSender<i32>),
    Unbounded(mpsc::Sender<i32>),
}

impl ChannelSender {
    fn send(&self, value: i32) -> Result<(), mpsc::SendError<i32>> {
        match self {
            ChannelSender::Bounded(s) => s.send(value),
            ChannelSender::Unbounded(s) => s.send(value),
        }
    }
}

struct ChannelPair {
    sender: ChannelSender,
    receiver: Arc<Mutex<mpsc::Receiver<i32>>>,
}

/// Create a new thread
#[no_mangle]
pub unsafe extern "C" fn aether_thread_create(
    function: extern "C" fn(),
    stack_size: c_int
) -> c_int {
    let stack_size = if stack_size > 0 { stack_size as usize } else { 2 * 1024 * 1024 };
    
    let builder = thread::Builder::new()
        .stack_size(stack_size);
    
    match builder.spawn(move || {
        function();
    }) {
        Ok(handle) => {
            let thread_id = NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst);
            THREAD_REGISTRY.lock().unwrap().insert(thread_id, handle);
            thread_id
        },
        Err(_) => -1,
    }
}

/// Join a thread
#[no_mangle]
pub extern "C" fn aether_thread_join(handle: c_int, timeout_ms: c_int) -> c_int {
    let handle_opt = THREAD_REGISTRY.lock().unwrap().remove(&handle);
    
    match handle_opt {
        Some(thread_handle) => {
            if timeout_ms < 0 {
                // Infinite wait
                match thread_handle.join() {
                    Ok(_) => 1,
                    Err(_) => 0,
                }
            } else {
                // For simplicity, we'll do a blocking join since Rust doesn't have timed join
                // In a real implementation, you'd use a different approach
                match thread_handle.join() {
                    Ok(_) => 1,
                    Err(_) => 0,
                }
            }
        },
        None => 0,
    }
}

/// Get current thread ID
#[no_mangle]
pub extern "C" fn aether_thread_current_id() -> c_int {
    // Simple thread ID based on thread local storage
    thread_local! {
        static THREAD_ID: std::cell::Cell<i32> = std::cell::Cell::new(0);
    }
    
    THREAD_ID.with(|id| {
        if id.get() == 0 {
            id.set(NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst));
        }
        id.get()
    })
}

/// Yield current thread
#[no_mangle]
pub extern "C" fn aether_thread_yield() {
    thread::yield_now();
}

/// Create a mutex
#[no_mangle]
pub extern "C" fn aether_mutex_create() -> c_int {
    let mutex_id = NEXT_MUTEX_ID.fetch_add(1, Ordering::SeqCst);
    let mutex_state = Arc::new(Mutex::new(MutexState {
        locked: false,
        owner_thread: -1,
    }));
    
    MUTEX_REGISTRY.lock().unwrap().insert(mutex_id, mutex_state);
    mutex_id
}

/// Lock a mutex
#[no_mangle]
pub extern "C" fn aether_mutex_lock(handle: c_int, timeout_ms: c_int) -> c_int {
    let mutex_opt = MUTEX_REGISTRY.lock().unwrap().get(&handle).cloned();
    
    match mutex_opt {
        Some(mutex) => {
            if timeout_ms < 0 {
                // Infinite wait
                match mutex.lock() {
                    Ok(mut state) => {
                        state.locked = true;
                        state.owner_thread = aether_thread_current_id();
                        1
                    },
                    Err(_) => 0,
                }
            } else {
                // Try lock with timeout
                let timeout = Duration::from_millis(timeout_ms as u64);
                let start = std::time::Instant::now();
                
                loop {
                    match mutex.try_lock() {
                        Ok(mut state) => {
                            state.locked = true;
                            state.owner_thread = aether_thread_current_id();
                            return 1;
                        },
                        Err(_) => {
                            if start.elapsed() >= timeout {
                                return 0;
                            }
                            thread::sleep(Duration::from_millis(1));
                        }
                    }
                }
            }
        },
        None => 0,
    }
}

/// Unlock a mutex
#[no_mangle]
pub extern "C" fn aether_mutex_unlock(handle: c_int) {
    let mutex_opt = MUTEX_REGISTRY.lock().unwrap().get(&handle).cloned();
    
    if let Some(mutex) = mutex_opt {
        if let Ok(mut state) = mutex.lock() {
            state.locked = false;
            state.owner_thread = -1;
        }
    }
}

/// Destroy a mutex
#[no_mangle]
pub extern "C" fn aether_mutex_destroy(handle: c_int) {
    MUTEX_REGISTRY.lock().unwrap().remove(&handle);
}

/// Create a channel
#[no_mangle]
pub extern "C" fn aether_channel_create(capacity: c_int) -> c_int {
    let channel_id = NEXT_CHANNEL_ID.fetch_add(1, Ordering::SeqCst);
    
    let channel_pair = if capacity == 0 {
        let (sender, receiver) = mpsc::channel();
        ChannelPair {
            sender: ChannelSender::Unbounded(sender),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    } else {
        let (sender, receiver) = mpsc::sync_channel(capacity as usize);
        ChannelPair {
            sender: ChannelSender::Bounded(sender),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    };
    
    CHANNEL_REGISTRY.lock().unwrap().insert(channel_id, channel_pair);
    channel_id
}

/// Send on a channel
#[no_mangle]
pub extern "C" fn aether_channel_send(handle: c_int, value: c_int, timeout_ms: c_int) -> c_int {
    let sender_opt = CHANNEL_REGISTRY.lock().unwrap().get(&handle).map(|c| match &c.sender {
        ChannelSender::Bounded(s) => ChannelSender::Bounded(s.clone()),
        ChannelSender::Unbounded(s) => ChannelSender::Unbounded(s.clone()),
    });
    
    match sender_opt {
        Some(sender) => {
            if timeout_ms < 0 {
                // Blocking send
                match sender.send(value) {
                    Ok(_) => 1,
                    Err(_) => 0,
                }
            } else {
                // Rust's channels don't have timeout on send, so we simulate
                match sender.send(value) {
                    Ok(_) => 1,
                    Err(_) => 0,
                }
            }
        },
        None => 0,
    }
}

/// Receive from a channel
#[no_mangle]
pub unsafe extern "C" fn aether_channel_receive(
    handle: c_int,
    value: *mut c_int,
    timeout_ms: c_int
) -> c_int {
    if value.is_null() {
        return 0;
    }
    
    let receiver_opt = CHANNEL_REGISTRY.lock().unwrap()
        .get(&handle)
        .map(|c| c.receiver.clone());
    
    match receiver_opt {
        Some(receiver) => {
            let receiver = match receiver.lock() {
                Ok(r) => r,
                Err(_) => return 0,
            };
            
            if timeout_ms < 0 {
                // Blocking receive
                match receiver.recv() {
                    Ok(v) => {
                        *value = v;
                        1
                    },
                    Err(_) => 0,
                }
            } else {
                // Timed receive
                match receiver.recv_timeout(Duration::from_millis(timeout_ms as u64)) {
                    Ok(v) => {
                        *value = v;
                        1
                    },
                    Err(_) => 0,
                }
            }
        },
        None => 0,
    }
}

/// Close a channel
#[no_mangle]
pub extern "C" fn aether_channel_close(handle: c_int) {
    CHANNEL_REGISTRY.lock().unwrap().remove(&handle);
}

/// Atomic load
#[no_mangle]
pub unsafe extern "C" fn aether_atomic_load(ptr: *mut c_int) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    
    let atomic = &*(ptr as *const AtomicI32);
    atomic.load(Ordering::SeqCst)
}

/// Atomic store
#[no_mangle]
pub unsafe extern "C" fn aether_atomic_store(ptr: *mut c_int, value: c_int) {
    if ptr.is_null() {
        return;
    }
    
    let atomic = &*(ptr as *const AtomicI32);
    atomic.store(value, Ordering::SeqCst);
}

/// Atomic fetch and add
#[no_mangle]
pub unsafe extern "C" fn aether_atomic_fetch_add(ptr: *mut c_int, delta: c_int) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    
    let atomic = &*(ptr as *const AtomicI32);
    atomic.fetch_add(delta, Ordering::SeqCst)
}

/// Atomic compare and swap
#[no_mangle]
pub unsafe extern "C" fn aether_atomic_compare_swap(
    ptr: *mut c_int,
    expected: c_int,
    desired: c_int
) -> c_int {
    if ptr.is_null() {
        return 0;
    }
    
    let atomic = &*(ptr as *const AtomicI32);
    match atomic.compare_exchange(expected, desired, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(v) => v,
        Err(v) => v,
    }
}