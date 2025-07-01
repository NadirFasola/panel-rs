// src/core/items/mem/stat_backend.rs

use anyhow::{Context, Result};
use std::fs;

// A snapshot of total vs available memory (in kB)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemInfo {
    pub total_kb: u64,
    pub available_kb: u64,
}

impl MemInfo {
    // Parse the full contents of `/proc/sys/meminfo`
    pub fn from_meminfo(contents: &str) -> Result<Self> {
        let mut total = None;
        let mut available = None;
        let mut free = None;
        let mut buffers = None;
        let mut cached = None;

        for line in contents.lines() {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                // rest is like " 16389856 kB"
                if let Some(val) = rest.split_whitespace().next() {
                    total = Some(val.parse::<u64>().context("Parsing MemTotal value")?);
                }
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                if let Some(val) = rest.split_whitespace().next() {
                    available = Some(val.parse::<u64>().context("Parsing MemAvailable")?);
                }
            } else if let Some(rest) = line.strip_prefix("MemFree:") {
                if let Some(val) = rest.split_whitespace().next() {
                    free = Some(val.parse::<u64>().context("Parsing MemFree")?);
                }
            } else if let Some(rest) = line.strip_prefix("Buffers:") {
                if let Some(val) = rest.split_whitespace().next() {
                    buffers = Some(val.parse::<u64>().context("Parsing Buffers")?);
                }
            } else if let Some(rest) = line.strip_prefix("Cached:") {
                if let Some(val) = rest.split_whitespace().next() {
                    cached = Some(val.parse::<u64>().context("Parsing Cached")?);
                }
            }

            // once we have total AND (available OR (free + buffers + cached)) we can stop
            if total.is_some()
                && (available.is_some()
                    || (free.is_some() && buffers.is_some() && cached.is_some()))
            {
                break;
            }
        }

        let total_kb = total.context("MemTotal not found in /proc/meminfo")?;

        // Prefer MemAvailabe if present; otherwise compute it
        let available_kb = if let Some(av) = available {
            av
        } else {
            let free = free.context("MemFree not found in /proc/meminfo")?;
            let buffers = buffers.context("Buffers not found in /proc/meminfo")?;
            let cached = cached.context("Cached not found in /proc/meminfo")?;
            free + buffers + cached
        };

        Ok(MemInfo {
            total_kb,
            available_kb,
        })
    }

    // Read `/proc/meminfo` on disk and parse it
    pub fn read_from_proc() -> Result<Self> {
        let data = fs::read_to_string("/proc/meminfo").context("Reading /proc/meminfo")?;
        MemInfo::from_meminfo(&data)
    }

    // Compute usage percentage (0.0 - 100.0)
    pub fn usage_percent(&self) -> f64 {
        let used = self.total_kb.saturating_sub(self.available_kb) as f64;
        let total = self.total_kb as f64;
        if total > 0.0 {
            (used / total) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
MemTotal:       16389856 kB
MemFree:         1234567 kB
MemAvailable:   12345678 kB
Buffers:          234567 kB
Cached:          3456789 kB
";

    #[test]
    fn parse_meminfo() {
        let info = MemInfo::from_meminfo(SAMPLE).unwrap();
        assert_eq!(info.total_kb, 16_389_856);
        assert_eq!(info.available_kb, 12_345_678);
    }

    #[test]
    fn usage_precent() {
        let info = MemInfo {
            total_kb: 100,
            available_kb: 25,
        };
        assert!((info.usage_percent() - 75.0).abs() < 1e-6);
    }
}
