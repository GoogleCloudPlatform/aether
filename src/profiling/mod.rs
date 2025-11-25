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

//! Compilation profiling and performance measurement
//!
//! Provides timing and performance metrics for compiler phases

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance profiler for tracking compilation phases
#[derive(Debug, Default)]
pub struct CompilationProfiler {
    /// Phase timing data
    phases: HashMap<String, PhaseMetrics>,

    /// Total compilation start time
    start_time: Option<Instant>,

    /// Memory usage snapshots
    memory_snapshots: Vec<MemorySnapshot>,
}

/// Metrics for a single compilation phase
#[derive(Debug, Clone)]
pub struct PhaseMetrics {
    /// Phase name
    pub name: String,

    /// Total time spent in this phase
    pub total_duration: Duration,

    /// Number of times this phase was executed
    pub execution_count: u32,

    /// Average duration per execution
    pub average_duration: Duration,

    /// Maximum duration seen
    pub max_duration: Duration,

    /// Minimum duration seen
    pub min_duration: Duration,
}

/// Memory usage snapshot
#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    /// Phase name when snapshot was taken
    pub phase: String,

    /// Timestamp of snapshot
    pub timestamp: Duration,

    /// Memory usage in bytes
    pub memory_usage: usize,
}

/// Handle for timing a specific phase
pub struct PhaseTimer<'a> {
    profiler: &'a mut CompilationProfiler,
    phase_name: String,
    start_time: Instant,
}

impl CompilationProfiler {
    /// Create a new profiler
    pub fn new() -> Self {
        Self::default()
    }

    /// Start profiling compilation
    pub fn start_compilation(&mut self) {
        self.start_time = Some(Instant::now());
        self.phases.clear();
        self.memory_snapshots.clear();
    }

    /// Start timing a phase
    pub fn start_phase(&mut self, phase_name: &str) -> PhaseTimer {
        PhaseTimer {
            profiler: self,
            phase_name: phase_name.to_string(),
            start_time: Instant::now(),
        }
    }

    /// Record phase completion
    fn record_phase(&mut self, phase_name: String, duration: Duration) {
        let metrics = self
            .phases
            .entry(phase_name.clone())
            .or_insert_with(|| PhaseMetrics {
                name: phase_name,
                total_duration: Duration::ZERO,
                execution_count: 0,
                average_duration: Duration::ZERO,
                max_duration: Duration::ZERO,
                min_duration: Duration::MAX,
            });

        metrics.total_duration += duration;
        metrics.execution_count += 1;
        metrics.average_duration = metrics.total_duration / metrics.execution_count;
        metrics.max_duration = metrics.max_duration.max(duration);
        metrics.min_duration = metrics.min_duration.min(duration);
    }

    /// Take a memory snapshot
    pub fn snapshot_memory(&mut self, phase: &str) {
        if let Some(start) = self.start_time {
            let timestamp = start.elapsed();
            let memory_usage = self.get_current_memory_usage();

            self.memory_snapshots.push(MemorySnapshot {
                phase: phase.to_string(),
                timestamp,
                memory_usage,
            });
        }
    }

    /// Get current memory usage (platform-specific)
    fn get_current_memory_usage(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            // Read from /proc/self/statm
            if let Ok(contents) = std::fs::read_to_string("/proc/self/statm") {
                if let Some(rss_pages) = contents.split_whitespace().nth(1) {
                    if let Ok(pages) = rss_pages.parse::<usize>() {
                        return pages * 4096; // Convert pages to bytes
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Use rusage on macOS
            use libc::{getrusage, rusage, RUSAGE_SELF};
            use std::mem;

            unsafe {
                let mut usage: rusage = mem::zeroed();
                if getrusage(RUSAGE_SELF, &mut usage) == 0 {
                    return usage.ru_maxrss as usize;
                }
            }
        }

        // Fallback: return 0 if we can't get memory usage
        0
    }

    /// Get total compilation time
    pub fn total_time(&self) -> Duration {
        self.start_time
            .map(|start| start.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Generate a profiling report
    pub fn generate_report(&self) -> ProfilingReport {
        let mut phases: Vec<_> = self.phases.values().cloned().collect();
        phases.sort_by_key(|p| std::cmp::Reverse(p.total_duration));

        ProfilingReport {
            total_time: self.total_time(),
            phases,
            memory_snapshots: self.memory_snapshots.clone(),
        }
    }

    /// Print a summary report to stderr
    pub fn print_summary(&self) {
        let report = self.generate_report();

        eprintln!("\n=== Compilation Performance Report ===");
        eprintln!(
            "Total compilation time: {:.3}s",
            report.total_time.as_secs_f64()
        );
        eprintln!();

        eprintln!("Phase Breakdown:");
        eprintln!(
            "{:<30} {:>10} {:>10} {:>10} {:>10}",
            "Phase", "Total", "Count", "Average", "Max"
        );
        eprintln!("{:-<70}", "");

        for phase in &report.phases {
            eprintln!(
                "{:<30} {:>10.3}s {:>10} {:>10.3}s {:>10.3}s",
                phase.name,
                phase.total_duration.as_secs_f64(),
                phase.execution_count,
                phase.average_duration.as_secs_f64(),
                phase.max_duration.as_secs_f64()
            );
        }

        if !report.memory_snapshots.is_empty() {
            eprintln!();
            eprintln!("Memory Usage:");
            eprintln!("{:<30} {:>15}", "Phase", "Memory (MB)");
            eprintln!("{:-<45}", "");

            for snapshot in &report.memory_snapshots {
                eprintln!(
                    "{:<30} {:>15.2}",
                    snapshot.phase,
                    snapshot.memory_usage as f64 / 1_048_576.0
                );
            }
        }
    }
}

/// Profiling report
#[derive(Debug)]
pub struct ProfilingReport {
    /// Total compilation time
    pub total_time: Duration,

    /// Phase metrics sorted by total time
    pub phases: Vec<PhaseMetrics>,

    /// Memory usage snapshots
    pub memory_snapshots: Vec<MemorySnapshot>,
}

impl<'a> Drop for PhaseTimer<'a> {
    fn drop(&mut self) {
        let duration = self.start_time.elapsed();
        self.profiler
            .record_phase(self.phase_name.clone(), duration);
    }
}

/// Macro for timing a code block
#[macro_export]
macro_rules! profile_phase {
    ($profiler:expr, $phase:expr, $block:block) => {{
        let _timer = $profiler.start_phase($phase);
        $block
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_profiling() {
        let mut profiler = CompilationProfiler::new();
        profiler.start_compilation();

        // Simulate some phases
        {
            let _timer = profiler.start_phase("lexing");
            thread::sleep(Duration::from_millis(10));
        }

        {
            let _timer = profiler.start_phase("parsing");
            thread::sleep(Duration::from_millis(20));
        }

        {
            let _timer = profiler.start_phase("optimization");
            thread::sleep(Duration::from_millis(5));
        }

        let report = profiler.generate_report();
        assert_eq!(report.phases.len(), 3);

        // Verify phases are sorted by duration
        assert!(report.phases[0].total_duration >= report.phases[1].total_duration);
        assert!(report.phases[1].total_duration >= report.phases[2].total_duration);
    }

    #[test]
    fn test_repeated_phases() {
        let mut profiler = CompilationProfiler::new();
        profiler.start_compilation();

        // Run the same phase multiple times
        for _ in 0..3 {
            let _timer = profiler.start_phase("repeated");
            thread::sleep(Duration::from_millis(5));
        }

        let report = profiler.generate_report();
        let phase = &report.phases[0];

        assert_eq!(phase.execution_count, 3);
        assert!(phase.total_duration >= Duration::from_millis(15));
    }
}
