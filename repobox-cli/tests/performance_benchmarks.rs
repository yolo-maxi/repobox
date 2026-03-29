//! CLI Performance Benchmarking Framework for repobox-cli
//!
//! This module implements performance profiling and optimization for CLI commands
//! to achieve <100ms response times for agent automation scenarios.
//!
//! ## Benchmarked Commands
//!
//! - `status`: Show current status (identity, branch, permissions summary)
//! - `check`: Check if an identity can perform an action
//! - `whoami`: Show current identity
//! - `init`: Initialize repo.box in current git repo
//!
//! ## Performance Targets
//!
//! - **status/check/whoami**: <100ms for agent automation workflows
//! - **init**: <500ms (less frequently used)
//! - **Cold start penalty**: <50ms additional overhead
//!
//! ## Regression Detection
//!
//! Benchmarks establish baseline metrics and detect performance regressions
//! above 20% threshold from recorded baselines.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::TempDir;

/// Performance benchmark results for a CLI command
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub command: String,
    pub iterations: u32,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub mean_duration: Duration,
    pub median_duration: Duration,
    pub std_deviation: Duration,
    pub cold_start_overhead: Option<Duration>,
}

impl BenchmarkResult {
    /// Check if this result meets the performance target
    pub fn meets_target(&self, target_ms: u64) -> bool {
        self.median_duration.as_millis() as u64 <= target_ms
    }

    /// Check if this result represents a performance regression
    pub fn is_regression(&self, baseline: &BenchmarkResult, threshold_percent: f64) -> bool {
        let current = self.median_duration.as_millis() as f64;
        let baseline_time = baseline.median_duration.as_millis() as f64;
        let regression_threshold = baseline_time * (1.0 + threshold_percent / 100.0);
        current > regression_threshold
    }
}

/// Performance test environment setup
pub struct PerformanceTester {
    temp_dir: TempDir,
    repo_root: PathBuf,
    repobox_binary: PathBuf,
}

impl PerformanceTester {
    /// Create a new performance test environment
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let repo_root = temp_dir.path().to_path_buf();

        // Find repobox binary (built target or current exe)
        let repobox_binary = find_repobox_binary()?;

        // Initialize a git repo for testing
        let git_init = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&repo_root)
            .output()?;

        if !git_init.status.success() {
            return Err("Failed to initialize git repo".into());
        }

        Ok(Self {
            temp_dir,
            repo_root,
            repobox_binary,
        })
    }

    /// Setup repobox configuration for performance testing
    pub fn setup_repobox_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Run repobox init to create configuration
        let init_result = Command::new(&self.repobox_binary)
            .args(["init", "--force"])
            .current_dir(&self.repo_root)
            .output()?;

        if !init_result.status.success() {
            let stderr = String::from_utf8_lossy(&init_result.stderr);
            return Err(format!("Failed to init repobox: {}", stderr).into());
        }

        // Generate a test identity for performance tests
        let keygen_result = Command::new(&self.repobox_binary)
            .args(["keys", "generate", "--alias", "perftest"])
            .current_dir(&self.repo_root)
            .env("HOME", self.temp_dir.path())
            .output()?;

        if !keygen_result.status.success() {
            let stderr = String::from_utf8_lossy(&keygen_result.stderr);
            return Err(format!("Failed to generate keys: {}", stderr).into());
        }

        Ok(())
    }

    /// Benchmark a CLI command with specified arguments
    pub fn benchmark_command(
        &self,
        command_args: &[&str],
        iterations: u32,
        measure_cold_start: bool,
    ) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut durations = Vec::with_capacity(iterations as usize);
        let command_str = command_args.join(" ");

        // Measure cold start overhead if requested
        let cold_start_overhead = if measure_cold_start {
            Some(self.measure_cold_start_overhead(command_args)?)
        } else {
            None
        };

        // Run warmup iterations to stabilize timing
        for _ in 0..3 {
            self.execute_command(command_args)?;
        }

        // Benchmark iterations
        for _ in 0..iterations {
            let start = Instant::now();
            let result = self.execute_command(command_args)?;
            let duration = start.elapsed();

            // Only record successful executions
            if result.status.success() {
                durations.push(duration);
            } else {
                eprintln!("Warning: Command failed during benchmarking: {}", command_str);
            }
        }

        if durations.is_empty() {
            return Err("All benchmark iterations failed".into());
        }

        // Calculate statistics
        durations.sort();
        let min_duration = durations[0];
        let max_duration = durations[durations.len() - 1];
        let median_duration = durations[durations.len() / 2];

        let sum: Duration = durations.iter().sum();
        let mean_duration = sum / durations.len() as u32;

        // Calculate standard deviation
        let variance: f64 = durations
            .iter()
            .map(|&d| {
                let diff = d.as_nanos() as f64 - mean_duration.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>()
            / durations.len() as f64;
        let std_deviation = Duration::from_nanos(variance.sqrt() as u64);

        Ok(BenchmarkResult {
            command: command_str,
            iterations: durations.len() as u32,
            min_duration,
            max_duration,
            mean_duration,
            median_duration,
            std_deviation,
            cold_start_overhead,
        })
    }

    /// Measure cold start overhead by comparing first run vs warmed up runs
    fn measure_cold_start_overhead(
        &self,
        command_args: &[&str],
    ) -> Result<Duration, Box<dyn std::error::Error>> {
        // Cold start measurement
        let cold_start = Instant::now();
        self.execute_command(command_args)?;
        let cold_duration = cold_start.elapsed();

        // Warmed up measurements (average of 5 runs)
        let mut warm_durations = Vec::new();
        for _ in 0..5 {
            let warm_start = Instant::now();
            self.execute_command(command_args)?;
            warm_durations.push(warm_start.elapsed());
        }

        let warm_median = {
            warm_durations.sort();
            warm_durations[warm_durations.len() / 2]
        };

        Ok(cold_duration.saturating_sub(warm_median))
    }

    /// Execute a repobox command and return the result
    fn execute_command(
        &self,
        command_args: &[&str],
    ) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        let result = Command::new(&self.repobox_binary)
            .args(command_args)
            .current_dir(&self.repo_root)
            .env("HOME", self.temp_dir.path())
            .output()?;

        Ok(result)
    }

    /// Get the repo root path for test setup
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Get the repobox binary path
    pub fn binary_path(&self) -> &Path {
        &self.repobox_binary
    }
}

/// Find the repobox binary to benchmark
fn find_repobox_binary() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Try built target first
    let target_debug = Path::new("target/debug/repobox");
    let target_release = Path::new("target/release/repobox");

    if target_release.exists() {
        return Ok(target_release.to_path_buf());
    }

    if target_debug.exists() {
        return Ok(target_debug.to_path_buf());
    }

    // Try current executable
    if let Ok(current_exe) = std::env::current_exe() {
        if current_exe.file_name().and_then(|n| n.to_str()) == Some("repobox") {
            return Ok(current_exe);
        }
    }

    // Try in PATH
    if let Ok(output) = Command::new("which").arg("repobox").output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout);
            let path = Path::new(path_str.trim());
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }
    }

    Err("Could not find repobox binary for benchmarking".into())
}

/// Performance baseline storage and regression detection
pub struct PerformanceBaselines {
    baselines: HashMap<String, BenchmarkResult>,
}

impl PerformanceBaselines {
    /// Create new baseline storage
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
        }
    }

    /// Load baselines from disk
    pub fn load_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let baselines = serde_json::from_str(&content)?;
        Ok(Self { baselines })
    }

    /// Save baselines to disk
    pub fn save_to_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.baselines)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add or update a baseline
    pub fn set_baseline(&mut self, command: String, result: BenchmarkResult) {
        self.baselines.insert(command, result);
    }

    /// Get baseline for a command
    pub fn get_baseline(&self, command: &str) -> Option<&BenchmarkResult> {
        self.baselines.get(command)
    }

    /// Check for performance regressions
    pub fn check_regressions(
        &self,
        results: &[BenchmarkResult],
        threshold_percent: f64,
    ) -> Vec<(String, BenchmarkResult, BenchmarkResult)> {
        let mut regressions = Vec::new();

        for result in results {
            if let Some(baseline) = self.get_baseline(&result.command) {
                if result.is_regression(baseline, threshold_percent) {
                    regressions.push((result.command.clone(), baseline.clone(), result.clone()));
                }
            }
        }

        regressions
    }
}

/// Run the complete CLI performance benchmark suite
pub fn run_performance_suite() -> Result<Vec<BenchmarkResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    let tester = PerformanceTester::new()?;

    // Setup test environment
    tester.setup_repobox_config()?;

    println!("Running CLI performance benchmarks...");

    // Benchmark critical commands with performance targets
    let benchmarks = [
        // Fast commands: <100ms target
        ("status", &["status"][..], 100),
        ("whoami", &["whoami"][..], 100),
        ("check me push >main", &["check", "perftest", "push", ">main"][..], 100),
        ("lint", &["lint"][..], 100),
        
        // Slower commands: <500ms target
        ("init --force", &["init", "--force"][..], 500),
    ];

    for (name, command_args, target_ms) in &benchmarks {
        println!("  Benchmarking: {}", name);
        
        let result = tester.benchmark_command(command_args, 20, true)?;
        
        println!("    Median: {:3}ms | Target: {}ms | {}",
            result.median_duration.as_millis(),
            target_ms,
            if result.meets_target(*target_ms) { "✅ PASS" } else { "❌ FAIL" }
        );
        
        if let Some(overhead) = result.cold_start_overhead {
            println!("    Cold start overhead: {}ms", overhead.as_millis());
        }

        results.push(result);
    }

    Ok(results)
}

/// Profile individual CLI commands to identify performance bottlenecks
pub fn profile_command_bottlenecks(
    command_args: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let tester = PerformanceTester::new()?;
    tester.setup_repobox_config()?;

    println!("Profiling command: {}", command_args.join(" "));

    // Enable detailed timing if available (requires instrumentation in main binary)
    let profile_result = Command::new(tester.binary_path())
        .args(command_args)
        .current_dir(tester.repo_root())
        .env("HOME", tester.temp_dir.path())
        .env("REPOBOX_PROFILE", "1") // Signal to enable detailed profiling
        .output()?;

    if profile_result.status.success() {
        let stdout = String::from_utf8_lossy(&profile_result.stdout);
        let stderr = String::from_utf8_lossy(&profile_result.stderr);
        
        println!("Command output:\n{}", stdout);
        if !stderr.is_empty() {
            println!("Profiling info:\n{}", stderr);
        }
    } else {
        let stderr = String::from_utf8_lossy(&profile_result.stderr);
        println!("Command failed: {}", stderr);
    }

    Ok(())
}

// Implement serialization for BenchmarkResult (required for baseline storage)
impl serde::Serialize for BenchmarkResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let mut state = serializer.serialize_struct("BenchmarkResult", 7)?;
        state.serialize_field("command", &self.command)?;
        state.serialize_field("iterations", &self.iterations)?;
        state.serialize_field("min_duration_ms", &self.min_duration.as_millis())?;
        state.serialize_field("max_duration_ms", &self.max_duration.as_millis())?;
        state.serialize_field("mean_duration_ms", &self.mean_duration.as_millis())?;
        state.serialize_field("median_duration_ms", &self.median_duration.as_millis())?;
        state.serialize_field("std_deviation_ms", &self.std_deviation.as_millis())?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for BenchmarkResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Command,
            Iterations,
            MinDurationMs,
            MaxDurationMs,
            MeanDurationMs,
            MedianDurationMs,
            StdDeviationMs,
        }

        struct BenchmarkResultVisitor;

        impl<'de> Visitor<'de> for BenchmarkResultVisitor {
            type Value = BenchmarkResult;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BenchmarkResult")
            }

            fn visit_map<V>(self, mut map: V) -> Result<BenchmarkResult, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut command = None;
                let mut iterations = None;
                let mut min_duration_ms = None;
                let mut max_duration_ms = None;
                let mut mean_duration_ms = None;
                let mut median_duration_ms = None;
                let mut std_deviation_ms = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Command => {
                            if command.is_some() {
                                return Err(de::Error::duplicate_field("command"));
                            }
                            command = Some(map.next_value()?);
                        }
                        Field::Iterations => {
                            if iterations.is_some() {
                                return Err(de::Error::duplicate_field("iterations"));
                            }
                            iterations = Some(map.next_value()?);
                        }
                        Field::MinDurationMs => {
                            if min_duration_ms.is_some() {
                                return Err(de::Error::duplicate_field("min_duration_ms"));
                            }
                            min_duration_ms = Some(map.next_value::<u128>()?);
                        }
                        Field::MaxDurationMs => {
                            if max_duration_ms.is_some() {
                                return Err(de::Error::duplicate_field("max_duration_ms"));
                            }
                            max_duration_ms = Some(map.next_value::<u128>()?);
                        }
                        Field::MeanDurationMs => {
                            if mean_duration_ms.is_some() {
                                return Err(de::Error::duplicate_field("mean_duration_ms"));
                            }
                            mean_duration_ms = Some(map.next_value::<u128>()?);
                        }
                        Field::MedianDurationMs => {
                            if median_duration_ms.is_some() {
                                return Err(de::Error::duplicate_field("median_duration_ms"));
                            }
                            median_duration_ms = Some(map.next_value::<u128>()?);
                        }
                        Field::StdDeviationMs => {
                            if std_deviation_ms.is_some() {
                                return Err(de::Error::duplicate_field("std_deviation_ms"));
                            }
                            std_deviation_ms = Some(map.next_value::<u128>()?);
                        }
                    }
                }

                let command = command.ok_or_else(|| de::Error::missing_field("command"))?;
                let iterations = iterations.ok_or_else(|| de::Error::missing_field("iterations"))?;
                let min_ms = min_duration_ms.ok_or_else(|| de::Error::missing_field("min_duration_ms"))?;
                let max_ms = max_duration_ms.ok_or_else(|| de::Error::missing_field("max_duration_ms"))?;
                let mean_ms = mean_duration_ms.ok_or_else(|| de::Error::missing_field("mean_duration_ms"))?;
                let median_ms = median_duration_ms.ok_or_else(|| de::Error::missing_field("median_duration_ms"))?;
                let std_ms = std_deviation_ms.ok_or_else(|| de::Error::missing_field("std_deviation_ms"))?;

                Ok(BenchmarkResult {
                    command,
                    iterations,
                    min_duration: Duration::from_millis(min_ms as u64),
                    max_duration: Duration::from_millis(max_ms as u64),
                    mean_duration: Duration::from_millis(mean_ms as u64),
                    median_duration: Duration::from_millis(median_ms as u64),
                    std_deviation: Duration::from_millis(std_ms as u64),
                    cold_start_overhead: None, // Not persisted currently
                })
            }
        }

        const FIELDS: &'static [&'static str] = &[
            "command", "iterations", "min_duration_ms", "max_duration_ms",
            "mean_duration_ms", "median_duration_ms", "std_deviation_ms"
        ];
        deserializer.deserialize_struct("BenchmarkResult", FIELDS, BenchmarkResultVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_benchmark_result_meets_target() {
        let result = BenchmarkResult {
            command: "status".to_string(),
            iterations: 10,
            min_duration: Duration::from_millis(50),
            max_duration: Duration::from_millis(150),
            mean_duration: Duration::from_millis(90),
            median_duration: Duration::from_millis(85),
            std_deviation: Duration::from_millis(20),
            cold_start_overhead: None,
        };

        assert!(result.meets_target(100)); // 85ms <= 100ms
        assert!(!result.meets_target(80));  // 85ms > 80ms
    }

    #[test]
    fn test_regression_detection() {
        let baseline = BenchmarkResult {
            command: "check".to_string(),
            iterations: 10,
            min_duration: Duration::from_millis(40),
            max_duration: Duration::from_millis(60),
            mean_duration: Duration::from_millis(50),
            median_duration: Duration::from_millis(50),
            std_deviation: Duration::from_millis(5),
            cold_start_overhead: None,
        };

        let current = BenchmarkResult {
            command: "check".to_string(),
            iterations: 10,
            min_duration: Duration::from_millis(55),
            max_duration: Duration::from_millis(75),
            mean_duration: Duration::from_millis(65),
            median_duration: Duration::from_millis(65), // 30% increase from baseline
            std_deviation: Duration::from_millis(5),
            cold_start_overhead: None,
        };

        assert!(current.is_regression(&baseline, 20.0)); // 30% > 20% threshold
        assert!(!current.is_regression(&baseline, 40.0)); // 30% < 40% threshold
    }

    #[test]
    fn test_performance_tester_creation() {
        let tester = PerformanceTester::new();
        assert!(tester.is_ok());

        let tester = tester.unwrap();
        assert!(tester.repo_root().exists());
        assert!(tester.binary_path().exists());
    }

    #[test]
    fn test_baseline_storage() {
        let mut baselines = PerformanceBaselines::new();
        
        let result = BenchmarkResult {
            command: "status".to_string(),
            iterations: 10,
            min_duration: Duration::from_millis(45),
            max_duration: Duration::from_millis(65),
            mean_duration: Duration::from_millis(55),
            median_duration: Duration::from_millis(54),
            std_deviation: Duration::from_millis(4),
            cold_start_overhead: Some(Duration::from_millis(15)),
        };

        baselines.set_baseline("status".to_string(), result.clone());
        
        let retrieved = baselines.get_baseline("status");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().command, "status");
        assert_eq!(retrieved.unwrap().median_duration.as_millis(), 54);
    }
}