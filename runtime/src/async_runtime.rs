use std::ffi::c_void;
use std::sync::{Arc, Mutex, Condvar};
use std::collections::HashMap;
use lazy_static::lazy_static;

// Wrapper to allow sending raw pointers across threads
#[derive(Clone, Copy, Debug)]
struct SafePtr(*mut c_void);
unsafe impl Send for SafePtr {}
unsafe impl Sync for SafePtr {}

// Task status
#[derive(PartialEq)]
enum TaskStatus {
    Running,
    Completed,
}

struct Task {
    #[allow(dead_code)]
    id: usize,
    status: Arc<Mutex<TaskStatus>>,
    cond: Arc<Condvar>,
    result: Arc<Mutex<Option<SafePtr>>>,
}

struct AsyncRuntime {
    next_task_id: usize,
    tasks: HashMap<usize, Task>,
}

impl AsyncRuntime {
    fn new() -> Self {
        AsyncRuntime {
            next_task_id: 1,
            tasks: HashMap::new(),
        }
    }
}

lazy_static! {
    static ref RUNTIME: Mutex<AsyncRuntime> = Mutex::new(AsyncRuntime::new());
}

/// Initialize the async runtime
#[no_mangle]
pub extern "C" fn aether_async_init() {
    // Ensure runtime is initialized (lazy_static does this, but we might want explicit setup later)
    drop(RUNTIME.lock().unwrap());
}

/// Shutdown the async runtime
#[no_mangle]
pub extern "C" fn aether_async_shutdown() {
    let mut runtime = RUNTIME.lock().unwrap();
    runtime.tasks.clear();
    runtime.next_task_id = 1;
}

/// Spawn a new asynchronous task
#[no_mangle]
pub extern "C" fn aether_async_spawn(
    func: extern "C" fn(*mut c_void) -> *mut c_void, 
    arg: *mut c_void
) -> *mut c_void {
    let mut runtime = RUNTIME.lock().unwrap();
    let id = runtime.next_task_id;
    runtime.next_task_id += 1;
    
    let status = Arc::new(Mutex::new(TaskStatus::Running));
    let cond = Arc::new(Condvar::new());
    let result = Arc::new(Mutex::new(None));
    
    let task = Task {
        id,
        status: status.clone(),
        cond: cond.clone(),
        result: result.clone(),
    };
    
    runtime.tasks.insert(id, task);
    
    // Synchronization primitives for the thread
    let status_clone = status.clone();
    let cond_clone = cond.clone();
    let result_clone = result.clone();
    
    // Cast to usize to ensure Send compliance
    let func_addr = func as usize;
    let arg_addr = arg as usize;
    
    // Spawn thread to run the task
    // TODO: Use a proper thread pool instead of spawning a thread per task
    std::thread::spawn(move || {
        let func_ptr: extern "C" fn(*mut c_void) -> *mut c_void = unsafe { std::mem::transmute(func_addr) };
        let arg_ptr = arg_addr as *mut c_void;
        
        let res = func_ptr(arg_ptr);
        
        {
            let mut status_guard = status_clone.lock().unwrap();
            let mut result_guard = result_clone.lock().unwrap();
            
            *result_guard = Some(SafePtr(res));
            *status_guard = TaskStatus::Completed;
        }
        
        cond_clone.notify_all();
    });
    
    // Return task handle (ID cast to pointer)
    id as *mut c_void
}

/// Wait for a task to complete and return its result
#[no_mangle]
pub extern "C" fn aether_async_wait(task_handle: *mut c_void) -> *mut c_void {
    let id = task_handle as usize;
    
    // Get synchronization primitives
    let (status, cond, result) = {
        let runtime = RUNTIME.lock().unwrap();
        if let Some(task) = runtime.tasks.get(&id) {
            (task.status.clone(), task.cond.clone(), task.result.clone())
        } else {
            // Task not found
            return std::ptr::null_mut();
        }
    };
    
    // Wait for completion
    let mut status_guard = status.lock().unwrap();
    while *status_guard == TaskStatus::Running {
        status_guard = cond.wait(status_guard).unwrap();
    }
    
    // Get result
    let res_opt = {
        let result_guard = result.lock().unwrap();
        *result_guard
    };
    
    // Cleanup task from runtime
    {
        let mut runtime = RUNTIME.lock().unwrap();
        runtime.tasks.remove(&id);
    }
    
    res_opt.map(|p| p.0).unwrap_or(std::ptr::null_mut())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    extern "C" fn simple_task(arg: *mut c_void) -> *mut c_void {
        let val = arg as usize;
        (val * 2) as *mut c_void
    }
    
    extern "C" fn slow_task(arg: *mut c_void) -> *mut c_void {
        let val = arg as usize;
        std::thread::sleep(std::time::Duration::from_millis(50));
        (val + 1) as *mut c_void
    }

    #[test]
    fn test_async_simple() {
        aether_async_init();
        
        let arg = 21 as *mut c_void;
        let handle = aether_async_spawn(simple_task, arg);
        let result = aether_async_wait(handle);
        
        assert_eq!(result as usize, 42);
        
        aether_async_shutdown();
    }
    
    #[test]
    fn test_async_slow() {
        aether_async_init();
        
        let arg = 10 as *mut c_void;
        let handle = aether_async_spawn(slow_task, arg);
        
        // Task should be running in background
        
        let result = aether_async_wait(handle);
        
        assert_eq!(result as usize, 11);
        
        aether_async_shutdown();
    }
    
    #[test]
    fn test_multiple_tasks() {
        aether_async_init();
        
        let h1 = aether_async_spawn(simple_task, 10 as *mut c_void);
        let h2 = aether_async_spawn(simple_task, 20 as *mut c_void);
        let h3 = aether_async_spawn(slow_task, 30 as *mut c_void);
        
        let r1 = aether_async_wait(h1);
        let r2 = aether_async_wait(h2);
        let r3 = aether_async_wait(h3);
        
        assert_eq!(r1 as usize, 20);
        assert_eq!(r2 as usize, 40);
        assert_eq!(r3 as usize, 31);
        
        aether_async_shutdown();
    }
}