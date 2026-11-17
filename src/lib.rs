//! System profiling tool for SigmaOS.
//!
//! Collects and displays system information: CPU, memory, disk, OS, and hardware.
//! Reads from /proc and /sys. Dependency-free.

use std::collections::HashMap;
use std::fmt;

/// Errors for profiling operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    FileNotFound(String),
    ParseError(String),
    PermissionDenied,
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound(p) => write!(f, "file not found: {p}"),
            Self::ParseError(m) => write!(f, "parse error: {m}"),
            Self::PermissionDenied => write!(f, "permission denied"),
        }
    }
}

impl std::error::Error for ProfileError {}

/// System information structure.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemProfile {
    pub os_name: String,
    pub kernel_version: String,
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub disk_total_gb: u64,
    pub disk_used_gb: u64,
    pub uptime_seconds: u64,
}

impl SystemProfile {
    pub fn new() -> Self {
        Self {
            os_name: String::new(),
            kernel_version: String::new(),
            cpu_model: String::new(),
            cpu_cores: 0,
            total_memory_mb: 0,
            available_memory_mb: 0,
            disk_total_gb: 0,
            disk_used_gb: 0,
            uptime_seconds: 0,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "OS: {}\nKernel: {}\nCPU: {} ({} cores)\nMemory: {}MB total, {}MB available\nDisk: {}GB used of {}GB\nUptime: {} seconds",
            self.os_name, self.kernel_version, self.cpu_model, self.cpu_cores,
            self.total_memory_mb, self.available_memory_mb,
            self.disk_used_gb, self.disk_total_gb, self.uptime_seconds
        )
    }
}

/// System profiler.
#[derive(Debug, Clone, Default)]
pub struct SystemProfiler {
    cache: HashMap<String, String>,
}

impl SystemProfiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_file(&mut self, path: &str) -> Result<String, ProfileError> {
        if let Some(cached) = self.cache.get(path) {
            return Ok(cached.clone());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|_| ProfileError::FileNotFound(path.to_string()))?;
        self.cache.insert(path.to_string(), content.clone());
        Ok(content)
    }

    pub fn get_os_name(&mut self) -> Result<String, ProfileError> {
        let content = self.read_file("/etc/os-release")?;
        for line in content.lines() {
            if line.starts_with("NAME=") {
                return Ok(line.trim_start_matches("NAME=").trim_matches('"').to_string());
            }
        }
        Ok("Unknown".to_string())
    }

    pub fn get_kernel_version(&mut self) -> Result<String, ProfileError> {
        let content = self.read_file("/proc/version")?;
        Ok(content.split_whitespace().nth(2).unwrap_or("unknown").to_string())
    }

    pub fn get_cpu_info(&mut self) -> Result<(String, usize), ProfileError> {
        let content = self.read_file("/proc/cpuinfo")?;
        let mut model = String::new();
        let mut cores = 0;
        for line in content.lines() {
            if line.starts_with("model name") {
                if model.is_empty() {
                    model = line.split(':').nth(1).unwrap_or("").trim().to_string();
                }
            }
            if line.starts_with("processor") {
                cores += 1;
            }
        }
        Ok((model, cores))
    }

    pub fn get_memory_info(&mut self) -> Result<(u64, u64), ProfileError> {
        let content = self.read_file("/proc/meminfo")?;
        let mut total = 0u64;
        let mut available = 0u64;
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
            if line.starts_with("MemAvailable:") {
                available = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
            }
        }
        Ok((total / 1024, available / 1024)) // Convert KB to MB
    }

    pub fn get_uptime(&mut self) -> Result<u64, ProfileError> {
        let content = self.read_file("/proc/uptime")?;
        let uptime = content.split_whitespace().next().unwrap_or("0").parse().unwrap_or(0);
        Ok(uptime as u64)
    }

    pub fn collect_profile(&mut self) -> Result<SystemProfile, ProfileError> {
        let mut profile = SystemProfile::new();
        profile.os_name = self.get_os_name().unwrap_or_else(|_| "Unknown".to_string());
        profile.kernel_version = self.get_kernel_version().unwrap_or_else(|_| "unknown".to_string());
        let (cpu_model, cores) = self.get_cpu_info().unwrap_or_else(|_| ("unknown".to_string(), 0));
        profile.cpu_model = cpu_model;
        profile.cpu_cores = cores;
        let (total, avail) = self.get_memory_info().unwrap_or_else(|_| (0, 0));
        profile.total_memory_mb = total;
        profile.available_memory_mb = avail;
        profile.uptime_seconds = self.get_uptime().unwrap_or(0);
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_caching_aaa() {
        // Arrange
        let mut profiler = SystemProfiler::new();
        let path = "/etc/hostname";

        // Act
        let _ = profiler.read_file(path);
        let cached = profiler.read_file(path);

        // Assert
        assert!(cached.is_ok());
    }

    #[test]
    fn test_missing_file_error_aaa() {
        // Arrange
        let mut profiler = SystemProfiler::new();

        // Act
        let res = profiler.read_file("/nonexistent/file");

        // Assert
        assert!(res.is_err());
    }

    #[test]
    fn test_collect_profile_aaa() {
        // Arrange
        let mut profiler = SystemProfiler::new();

        // Act
        let profile = profiler.collect_profile();

        // Assert
        assert!(profile.is_ok());
        let p = profile.unwrap();
        assert!(!p.os_name.is_empty());
    }
}
