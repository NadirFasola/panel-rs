// src/core/items/cpu/stat_backend.rs

use anyhow::{Context, Result};
use std::fs;
use std::str::FromStr;

// Raw CPU snapshot: idle and total jiffies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuSnapshot {
    pub idle: u64,
    pub total: u64,
}

impl CpuSnapshot {
    // Parse a line startin with "cpu" from /proc/stat
    pub fn from_line(line: &str) -> Result<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.first() != Some(&"cpu") || parts.len() < 5 {
            anyhow::bail!("Invalid /proc/stat cpu line: {line}");
        }
        let nums: Vec<u64> = parts[1..]
            .iter()
            .map(|s| u64::from_str(s).with_context(|| format!("Parsing jiffy {s}")))
            .collect::<Result<_>>()?;
        let idle = nums[3];
        let total: u64 = nums.iter().sum();
        Ok(CpuSnapshot { idle, total })
    }

    // Read the real /proc/stat and ge tteh first line
    pub fn read_from_proc() -> Result<Self> {
        let content = fs::read_to_string("/proc/stat").context("Reading /proc/stat")?;
        content
            .lines()
            .next()
            .context("Missing cpu line in /proc/stat")
            .and_then(CpuSnapshot::from_line)
    }
}

pub fn compute_usage(old: CpuSnapshot, new: CpuSnapshot) -> f64 {
    let idle_delta = new.idle.saturating_sub(old.idle);
    let total_delta = new.total.saturating_sub(old.total);

    if total_delta == 0 {
        0.0
    } else {
        100.0 * (total_delta - idle_delta) as f64 / total_delta as f64
    }
}

// A backend holding the previous snapshot internally
pub struct CpuStatBackend {
    prev: CpuSnapshot,
}

impl CpuStatBackend {
    pub fn new() -> Result<Self> {
        let snap = CpuSnapshot::read_from_proc()?;
        Ok(CpuStatBackend { prev: snap })
    }

    pub fn read(&mut self) -> Result<f64> {
        let current = CpuSnapshot::read_from_proc().context("Failed to read CPU stats")?;
        let usage = compute_usage(self.prev, current);
        self.prev = current;
        Ok(usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cpu_line() {
        let line = "cpu 100 200 300 400 50 60";
        let snap = CpuSnapshot::from_line(line).unwrap();
        assert_eq!(snap.idle, 400);
        assert_eq!(snap.total, 1110);
    }

    #[test]
    fn compute_usage_zero_delta() {
        let s = CpuSnapshot {
            idle: 10,
            total: 20,
        };
        assert_eq!(compute_usage(s, s), 0.0);
    }

    #[test]
    fn compute_usage_nonzero() {
        let old = CpuSnapshot {
            idle: 10,
            total: 20,
        };
        let new = CpuSnapshot {
            idle: 20,
            total: 40,
        };
        assert!((compute_usage(old, new) - 50.0).abs() < 1e-6);
    }
}
