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

use std::ffi::c_void;
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// ============================================================================
// Future Implementation
// ============================================================================

/// Status of an asynchronous task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum FutureStatus {
    Pending = 0,
    Completed = 1,
    Failed = 2,
}

/// Shared state for a future
struct SharedState {
    status: FutureStatus,
    result: *mut c_void,
    waker: Option<thread::Thread>,
}

// Safety: The raw pointer 'result' is protected by the Mutex in AetherFuture.
// Access is synchronized, so it's safe to send/share across threads.
unsafe impl Send for SharedState {}
unsafe impl Sync for SharedState {}

/// A future that represents an asynchronous computation
#[derive(Clone)]
#[repr(C)]
pub struct AetherFuture {
    shared: Arc<(Mutex<SharedState>, Condvar)>,
}

impl AetherFuture {
    /// Create a new pending future
    pub fn new() -> Self {
        AetherFuture {
            shared: Arc::new((
                Mutex::new(SharedState {
                    status: FutureStatus::Pending,
                    result: std::ptr::null_mut(),
                    waker: None,
                }),
                Condvar::new(),
            )),
        }
    }

    /// Complete the future with a result
    pub fn complete(&self, result: *mut c_void) {
        let (lock, cvar) = &*self.shared;
        let mut state = lock.lock().unwrap();
        state.status = FutureStatus::Completed;
        state.result = result;
        
        // Notify any waiters
        cvar.notify_all();
        
        // Wake up specific waker if registered
        if let Some(waker) = &state.waker {
            waker.unpark();
        }
    }

    /// Mark the future as failed
    pub fn fail(&self) {
        let (lock, cvar) = &*self.shared;
        let mut state = lock.lock().unwrap();
        state.status = FutureStatus::Failed;
        
        // Notify any waiters
        cvar.notify_all();
    }

    /// Wait for the future to complete and return the result
    pub fn wait(&self) -> *mut c_void {
        let (lock, cvar) = &*self.shared;
        let mut state = lock.lock().unwrap();
        
        while state.status == FutureStatus::Pending {
            state = cvar.wait(state).unwrap();
        }
        
        state.result
    }
    
    /// Check if completed
    pub fn is_completed(&self) -> bool {
        let (lock, _) = &*self.shared;
        let state = lock.lock().unwrap();
        state.status != FutureStatus::Pending
    }
}

// ============================================================================
// Thread Pool Implementation
// ============================================================================

type Task = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    tasks: Arc<Mutex<VecDeque<Task>>>,
    cond: Arc<Condvar>,
    workers: Vec<thread::JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let tasks = Arc::new(Mutex::new(VecDeque::<Task>::new()));
        let cond = Arc::new(Condvar::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            let tasks_clone = Arc::clone(&tasks);
            let cond_clone = Arc::clone(&cond);
            let shutdown_clone = Arc::clone(&shutdown);

            workers.push(thread::spawn(move || {
                loop {
                    let task = {
                        let mut tasks_guard = tasks_clone.lock().unwrap();
                        
                        while tasks_guard.is_empty() {
                            if shutdown_clone.load(Ordering::SeqCst) {
                                return;
                            }
                            tasks_guard = cond_clone.wait(tasks_guard).unwrap();
                        }
                        
                        tasks_guard.pop_front()
                    };

                    if let Some(task) = task {
                        task();
                    }
                }
            }));
        }

        ThreadPool {
            tasks,
            cond,
            workers,
            shutdown,
        }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Box::new(f);
        {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.push_back(task);
        }
        self.cond.notify_one();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.cond.notify_all();

        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

// Global runtime state
static mut RUNTIME: Option<AsyncRuntime> = None;
static RUNTIME_INIT: std::sync::Once = std::sync::Once::new();

struct AsyncRuntime {
    pool: ThreadPool,
    active_tasks: AtomicUsize,
}

impl AsyncRuntime {
    fn global() -> &'static AsyncRuntime {
        unsafe {
            RUNTIME_INIT.call_once(|| {
                // Create thread pool with number of cores
                let threads = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4);
                
                RUNTIME = Some(AsyncRuntime {
                    pool: ThreadPool::new(threads),
                    active_tasks: AtomicUsize::new(0),
                });
            });
            
            RUNTIME.as_ref().unwrap()
        }
    }
}

// ============================================================================
// FFI Interface
// ============================================================================

/// Initialize the async runtime (idempotent)
#[no_mangle]
pub extern "C" fn aether_async_init() {
    AsyncRuntime::global();
}

/// Spawn an asynchronous task
/// 
/// # Arguments
/// * `task_fn` - Function pointer to the task to execute
/// * `context` - Void pointer to context data (arguments/captures)
/// * `cleanup_fn` - Optional function to clean up context after execution
/// 
/// # Returns
/// Pointer to an `AetherFuture` that will hold the result
#[no_mangle]
pub extern "C" fn aether_spawn(
    task_fn: extern "C" fn(*mut c_void) -> *mut c_void,
    context: *mut c_void,
    cleanup_fn: Option<extern "C" fn(*mut c_void)>
) -> *mut AetherFuture {
    let runtime = AsyncRuntime::global();
    runtime.active_tasks.fetch_add(1, Ordering::SeqCst);
    
    // Create future to hold result
    let future = AetherFuture::new();
    
    // Clone future for the worker thread
    // This keeps the shared state alive even if the main thread drops the future handle
    let future_clone = future.clone();
    
    // Create pointer for the caller
    let future_ptr = Box::into_raw(Box::new(future));
    
    // Convert raw pointers to usize for safe transport across threads
    let ctx_addr = context as usize;
    let task_fn_addr = task_fn as usize;
    let cleanup_fn_addr = cleanup_fn.map(|f| f as usize).unwrap_or(0);
    
    runtime.pool.execute(move || {
        // Convert back to pointers inside the thread
        let ctx = ctx_addr as *mut c_void;
        
        let task: extern "C" fn(*mut c_void) -> *mut c_void = unsafe { 
            std::mem::transmute(task_fn_addr) 
        };
        
        let cleanup: Option<extern "C" fn(*mut c_void)> = if cleanup_fn_addr == 0 {
            None
        } else {
            unsafe { Some(std::mem::transmute(cleanup_fn_addr)) }
        };
        
        // Execute the task
        let result = task(ctx);
        
        // Complete the future using the owned clone
        future_clone.complete(result);
            
        // Cleanup context if needed
        if let Some(cleanup_func) = cleanup {
            cleanup_func(ctx);
        }
        
        AsyncRuntime::global().active_tasks.fetch_sub(1, Ordering::SeqCst);
    });
    
    future_ptr
}

/// Wait for a future to complete
#[no_mangle]
pub extern "C" fn aether_await(future_ptr: *mut AetherFuture) -> *mut c_void {
    if future_ptr.is_null() {
        return std::ptr::null_mut();
    }
    
    unsafe {
        let future = &*future_ptr;
        future.wait()
    }
}

/// Release a future (decrement refcount/free)
#[no_mangle]
pub extern "C" fn aether_future_release(future_ptr: *mut AetherFuture) {
    if !future_ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(future_ptr);
        }
    }
}

/// Shutdown the runtime and wait for all tasks
#[no_mangle]
pub extern "C" fn aether_async_shutdown() {
    unsafe {
        if let Some(_) = RUNTIME.take() {
            // Thread pool drops here, waiting for workers
        }
    }
}

// Helper for tests
pub fn active_task_count() -> usize {
    if unsafe { RUNTIME.is_some() } {
        AsyncRuntime::global().active_tasks.load(Ordering::SeqCst)
    } else {
        0
    }
}