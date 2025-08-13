//! Async runtime for AetherScript futures and async/await
//!
//! Provides task scheduling, execution, and lifecycle management

use crate::error::SemanticError;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, Condvar};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Async runtime for executing futures
#[derive(Debug)]
pub struct AsyncRuntime {
    /// Runtime state
    state: RuntimeState,
    
    /// Task scheduler
    scheduler: TaskScheduler,
    
    /// Worker threads
    workers: Vec<Worker>,
    
    /// Runtime configuration
    config: RuntimeConfig,
    
    /// Runtime statistics
    stats: RuntimeStats,
}

/// Runtime state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeState {
    Idle,
    Starting,
    Running,
    Stopping,
    Stopped,
}

/// Task scheduler for managing async tasks
#[derive(Debug)]
pub struct TaskScheduler {
    /// Task queue
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    
    /// Condition variable for task availability
    task_available: Arc<Condvar>,
    
    /// Currently running tasks
    running_tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
    
    /// Completed tasks
    completed_tasks: Arc<Mutex<HashMap<TaskId, TaskResult>>>,
    
    /// Next task ID
    next_task_id: AtomicU64,
}

/// Worker thread for executing tasks
#[derive(Debug)]
pub struct Worker {
    /// Thread handle
    handle: Option<JoinHandle<()>>,
    
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
}

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    
    /// Maximum number of tasks in queue
    pub max_queue_size: usize,
    
    /// Task timeout duration
    pub task_timeout: Duration,
    
    /// Enable task stealing between workers
    pub enable_work_stealing: bool,
}

/// Runtime statistics
#[derive(Debug, Default)]
pub struct RuntimeStats {
    /// Total tasks scheduled
    pub tasks_scheduled: AtomicU64,
    
    /// Total tasks completed
    pub tasks_completed: AtomicU64,
    
    /// Total tasks failed
    pub tasks_failed: AtomicU64,
    
    /// Average task execution time
    pub avg_execution_time_ms: AtomicU64,
    
    /// Runtime uptime
    pub uptime_start: Option<Instant>,
}

/// Task ID type
pub type TaskId = u64;

/// Async task
#[derive(Debug)]
pub struct Task {
    /// Unique task ID
    pub id: TaskId,
    
    /// Task name/description
    pub name: String,
    
    /// Task priority
    pub priority: TaskPriority,
    
    /// Task execution function
    pub executor: TaskExecutor,
    
    /// Task creation time
    pub created_at: Instant,
    
    /// Task timeout
    pub timeout: Option<Duration>,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Task executor function type
pub enum TaskExecutor {
    /// Simple closure executor
    Closure(Box<dyn FnOnce() -> TaskResult + Send + 'static>),
    
    /// Future executor
    Future(Box<dyn std::future::Future<Output = TaskResult> + Send + Unpin + 'static>),
}

impl std::fmt::Debug for TaskExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskExecutor::Closure(_) => write!(f, "TaskExecutor::Closure(<closure>)"),
            TaskExecutor::Future(_) => write!(f, "TaskExecutor::Future(<future>)"),
        }
    }
}

/// Task execution result
#[derive(Debug, Clone)]
pub enum TaskResult {
    /// Task completed successfully
    Success(TaskValue),
    
    /// Task failed with error
    Error(String),
    
    /// Task was cancelled
    Cancelled,
    
    /// Task timed out
    Timeout,
}

/// Task result value
#[derive(Debug, Clone)]
pub enum TaskValue {
    /// No return value
    Unit,
    
    /// Integer value
    Integer(i64),
    
    /// Float value
    Float(f64),
    
    /// Boolean value
    Boolean(bool),
    
    /// String value
    String(String),
    
    /// Array of values
    Array(Vec<TaskValue>),
}

impl AsyncRuntime {
    /// Create a new async runtime
    pub fn new() -> Self {
        Self::with_config(RuntimeConfig::default())
    }
    
    /// Create a new async runtime with configuration
    pub fn with_config(config: RuntimeConfig) -> Self {
        Self {
            state: RuntimeState::Idle,
            scheduler: TaskScheduler::new(),
            workers: Vec::new(),
            config,
            stats: RuntimeStats::default(),
        }
    }
    
    /// Start the runtime
    pub fn start(&mut self) -> Result<(), SemanticError> {
        if self.state != RuntimeState::Idle {
            return Err(SemanticError::Internal {
                message: format!("Runtime already started, current state: {:?}", self.state),
            });
        }
        
        self.state = RuntimeState::Starting;
        
        // Start worker threads
        self.start_workers()?;
        
        self.state = RuntimeState::Running;
        self.stats.uptime_start = Some(Instant::now());
        
        Ok(())
    }
    
    /// Stop the runtime
    pub fn stop(&mut self) -> Result<(), SemanticError> {
        if self.state != RuntimeState::Running {
            return Err(SemanticError::Internal {
                message: format!("Runtime not running, current state: {:?}", self.state),
            });
        }
        
        self.state = RuntimeState::Stopping;
        
        // Stop all workers
        self.stop_workers()?;
        
        self.state = RuntimeState::Stopped;
        
        Ok(())
    }
    
    /// Check if runtime is idle
    pub fn is_idle(&self) -> bool {
        self.state == RuntimeState::Idle
    }
    
    /// Check if runtime is running
    pub fn is_running(&self) -> bool {
        self.state == RuntimeState::Running
    }
    
    /// Spawn a new task
    pub fn spawn<F>(&mut self, name: String, executor: F) -> Result<TaskId, SemanticError>
    where
        F: FnOnce() -> TaskResult + Send + 'static,
    {
        if !self.is_running() {
            return Err(SemanticError::Internal {
                message: "Runtime is not running".to_string(),
            });
        }
        
        let task = Task {
            id: self.scheduler.next_task_id(),
            name,
            priority: TaskPriority::Normal,
            executor: TaskExecutor::Closure(Box::new(executor)),
            created_at: Instant::now(),
            timeout: Some(self.config.task_timeout),
        };
        
        let task_id = task.id;
        self.scheduler.schedule_task(task)?;
        self.stats.tasks_scheduled.fetch_add(1, Ordering::Relaxed);
        
        Ok(task_id)
    }
    
    /// Spawn a high priority task
    pub fn spawn_high_priority<F>(&mut self, name: String, executor: F) -> Result<TaskId, SemanticError>
    where
        F: FnOnce() -> TaskResult + Send + 'static,
    {
        if !self.is_running() {
            return Err(SemanticError::Internal {
                message: "Runtime is not running".to_string(),
            });
        }
        
        let task = Task {
            id: self.scheduler.next_task_id(),
            name,
            priority: TaskPriority::High,
            executor: TaskExecutor::Closure(Box::new(executor)),
            created_at: Instant::now(),
            timeout: Some(self.config.task_timeout),
        };
        
        let task_id = task.id;
        self.scheduler.schedule_task(task)?;
        self.stats.tasks_scheduled.fetch_add(1, Ordering::Relaxed);
        
        Ok(task_id)
    }
    
    /// Wait for a task to complete
    pub fn wait_for_task(&self, task_id: TaskId) -> Result<TaskResult, SemanticError> {
        self.scheduler.wait_for_task(task_id)
    }
    
    /// Get runtime statistics
    pub fn stats(&self) -> &RuntimeStats {
        &self.stats
    }
    
    /// Get runtime configuration
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
    
    /// Start worker threads
    fn start_workers(&mut self) -> Result<(), SemanticError> {
        self.workers.clear();
        
        for worker_id in 0..self.config.worker_threads {
            let worker = Worker::new(
                self.scheduler.task_queue.clone(),
                self.scheduler.task_available.clone(),
                self.scheduler.running_tasks.clone(),
                self.scheduler.completed_tasks.clone(),
            )?;
            
            self.workers.push(worker);
        }
        
        Ok(())
    }
    
    /// Stop worker threads
    fn stop_workers(&mut self) -> Result<(), SemanticError> {
        // Signal all workers to shutdown
        for worker in &mut self.workers {
            worker.shutdown.store(true, Ordering::Relaxed);
        }
        
        // Notify all workers
        self.scheduler.task_available.notify_all();
        
        // Wait for all workers to finish
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                handle.join().map_err(|_| SemanticError::Internal {
                    message: "Failed to join worker thread".to_string(),
                })?;
            }
        }
        
        self.workers.clear();
        Ok(())
    }
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(Mutex::new(VecDeque::new())),
            task_available: Arc::new(Condvar::new()),
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
            completed_tasks: Arc::new(Mutex::new(HashMap::new())),
            next_task_id: AtomicU64::new(1),
        }
    }
    
    pub fn next_task_id(&self) -> TaskId {
        self.next_task_id.fetch_add(1, Ordering::Relaxed)
    }
    
    pub fn schedule_task(&self, task: Task) -> Result<(), SemanticError> {
        let mut queue = self.task_queue.lock().map_err(|_| SemanticError::Internal {
            message: "Failed to lock task queue".to_string(),
        })?;
        
        // Insert task in priority order
        let insert_pos = queue.iter().position(|t| t.priority < task.priority).unwrap_or(queue.len());
        queue.insert(insert_pos, task);
        
        // Notify workers that a task is available
        self.task_available.notify_one();
        
        Ok(())
    }
    
    pub fn wait_for_task(&self, task_id: TaskId) -> Result<TaskResult, SemanticError> {
        // Simple polling approach - in a real implementation, we'd use proper async mechanisms
        loop {
            {
                let completed = self.completed_tasks.lock().map_err(|_| SemanticError::Internal {
                    message: "Failed to lock completed tasks".to_string(),
                })?;
                
                if let Some(result) = completed.get(&task_id) {
                    return Ok(result.clone());
                }
            }
            
            // Small delay to avoid busy waiting
            thread::sleep(Duration::from_millis(1));
        }
    }
}

impl Worker {
    pub fn new(
        task_queue: Arc<Mutex<VecDeque<Task>>>,
        task_available: Arc<Condvar>,
        running_tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
        completed_tasks: Arc<Mutex<HashMap<TaskId, TaskResult>>>,
    ) -> Result<Self, SemanticError> {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        
        let handle = thread::spawn(move || {
            Self::worker_loop(
                0,
                task_queue,
                task_available,
                running_tasks,
                completed_tasks,
                shutdown_clone,
            );
        });
        
        Ok(Self {
            handle: Some(handle),
            shutdown,
        })
    }
    
    fn worker_loop(
        _worker_id: usize,
        task_queue: Arc<Mutex<VecDeque<Task>>>,
        task_available: Arc<Condvar>,
        running_tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
        completed_tasks: Arc<Mutex<HashMap<TaskId, TaskResult>>>,
        shutdown: Arc<AtomicBool>,
    ) {
        while !shutdown.load(Ordering::Relaxed) {
            // Try to get a task from the queue
            let task = {
                let mut queue = match task_queue.lock() {
                    Ok(queue) => queue,
                    Err(_) => break, // Poisoned lock, exit
                };
                
                // Wait for a task to be available
                while queue.is_empty() && !shutdown.load(Ordering::Relaxed) {
                    queue = match task_available.wait(queue) {
                        Ok(queue) => queue,
                        Err(_) => return, // Poisoned lock, exit
                    };
                }
                
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                
                queue.pop_front()
            };
            
            if let Some(task) = task {
                // Execute the task
                let task_id = task.id;
                let start_time = Instant::now();
                
                // Move task to running tasks
                {
                    if let Ok(mut running) = running_tasks.lock() {
                        running.insert(task_id, task);
                    }
                }
                
                // Execute the task (simplified for now)
                let result = TaskResult::Success(TaskValue::Unit);
                
                let _execution_time = start_time.elapsed();
                
                // Move result to completed tasks
                {
                    if let Ok(mut completed) = completed_tasks.lock() {
                        completed.insert(task_id, result);
                    }
                }
                
                // Remove from running tasks
                {
                    if let Ok(mut running) = running_tasks.lock() {
                        running.remove(&task_id);
                    }
                }
            }
        }
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get().max(1),
            max_queue_size: 1000,
            task_timeout: Duration::from_secs(30),
            enable_work_stealing: true,
        }
    }
}

impl RuntimeStats {
    pub fn uptime(&self) -> Option<Duration> {
        self.uptime_start.map(|start| start.elapsed())
    }
    
    pub fn tasks_scheduled(&self) -> u64 {
        self.tasks_scheduled.load(Ordering::Relaxed)
    }
    
    pub fn tasks_completed(&self) -> u64 {
        self.tasks_completed.load(Ordering::Relaxed)
    }
    
    pub fn tasks_failed(&self) -> u64 {
        self.tasks_failed.load(Ordering::Relaxed)
    }
    
    pub fn avg_execution_time_ms(&self) -> u64 {
        self.avg_execution_time_ms.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_runtime_creation() {
        let runtime = AsyncRuntime::new();
        assert!(runtime.is_idle());
        assert!(!runtime.is_running());
    }
    
    #[test]
    fn test_runtime_start_stop() {
        let mut runtime = AsyncRuntime::new();
        assert!(runtime.start().is_ok());
        assert!(runtime.is_running());
        assert!(runtime.stop().is_ok());
        assert_eq!(runtime.state, RuntimeState::Stopped);
    }
    
    #[test]
    fn test_task_scheduling() {
        let scheduler = TaskScheduler::new();
        let task_id = scheduler.next_task_id();
        assert_eq!(task_id, 1);
        
        let next_id = scheduler.next_task_id();
        assert_eq!(next_id, 2);
    }
    
    #[test]
    fn test_runtime_config() {
        let config = RuntimeConfig::default();
        assert!(config.worker_threads > 0);
        assert_eq!(config.max_queue_size, 1000);
    }
    
    #[test]
    fn test_task_priority_ordering() {
        assert!(TaskPriority::Critical > TaskPriority::High);
        assert!(TaskPriority::High > TaskPriority::Normal);
        assert!(TaskPriority::Normal > TaskPriority::Low);
    }
}

