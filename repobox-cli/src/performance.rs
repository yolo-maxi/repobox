//! Performance profiling and benchmarking module for repobox CLI
//!
//! This module provides runtime performance monitoring and profiling capabilities
//! for optimizing CLI command execution times.

use std::time::{Duration, Instant};

/// Performance profiler for CLI operations
pub struct PerformanceProfiler {
    start_time: Instant,
    operation_name: String,
    enabled: bool,
}

impl PerformanceProfiler {
    /// Create a new profiler for an operation
    pub fn new(operation_name: &str) -> Self {
        let enabled = std::env::var("REPOBOX_PROFILE").is_ok();
        
        if enabled {
            eprintln!("[PROFILE] Starting: {}", operation_name);
        }
        
        Self {
            start_time: Instant::now(),
            operation_name: operation_name.to_string(),
            enabled,
        }
    }

    /// Record a checkpoint in the operation
    pub fn checkpoint(&self, checkpoint_name: &str) {
        if self.enabled {
            let elapsed = self.start_time.elapsed();
            eprintln!("[PROFILE] {}: {} @ {}ms", 
                     self.operation_name, 
                     checkpoint_name, 
                     elapsed.as_millis());
        }
    }

    /// Mark the end of the operation
    pub fn finish(self) -> Duration {
        let total_time = self.start_time.elapsed();
        
        if self.enabled {
            eprintln!("[PROFILE] Finished: {} in {}ms", 
                     self.operation_name, 
                     total_time.as_millis());
        }
        
        total_time
    }
}

/// Macro to create a profiler for the current function
macro_rules! profile_operation {
    ($name:expr) => {
        let _profiler = $crate::performance::PerformanceProfiler::new($name);
    };
}

/// Macro to record a checkpoint in the current operation
macro_rules! profile_checkpoint {
    ($profiler:expr, $name:expr) => {
        $profiler.checkpoint($name);
    };
}

// Macros available for internal use

/// Performance optimization utilities
pub mod optimizations {
    use std::collections::HashMap;
    use std::sync::OnceLock;

    /// Cache for expensive operations (e.g., identity resolution)
    static IDENTITY_CACHE: OnceLock<std::sync::Mutex<HashMap<String, String>>> = OnceLock::new();

    /// Cache identity resolution to avoid repeated cryptographic operations
    pub fn cache_identity_resolution(key: &str, resolved: &str) {
        let cache = IDENTITY_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
        if let Ok(mut guard) = cache.lock() {
            guard.insert(key.to_string(), resolved.to_string());
        }
    }

    /// Get cached identity resolution
    pub fn get_cached_identity(key: &str) -> Option<String> {
        let cache = IDENTITY_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
        if let Ok(guard) = cache.lock() {
            guard.get(key).cloned()
        } else {
            None
        }
    }

    /// Clear identity cache (useful for testing)
    pub fn clear_identity_cache() {
        let cache = IDENTITY_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
        if let Ok(mut guard) = cache.lock() {
            guard.clear();
        }
    }
}