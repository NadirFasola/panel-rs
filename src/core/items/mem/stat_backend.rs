// src/core/items/mem/stat_backend.rs

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

// A snapshot of total vs available memory (in kB)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemInfo {
    pub total_kb: u64,
    pub available_kb: u64,
}

impl MemInfo {
    pub fn read_from_proc() -> Result<Self> {
        let f = File::open("/proc/meminfo").context("Opening /proc/meminfo")?;
        let mut reader = BufReader::new(f);
        let mut line = String::new();
        let mut total = None;
        let mut available = None;
        let mut free = None;
        let mut buffers = None;
        let mut cached = None;

        while reader.read_line(&mut line)? > 0 {
            if let Some(rest) = line.strip_prefix("MemTotal:") {
                if let Some(val) = rest.trim_start().split_ascii_whitespace().next() {
                    total = Some(val.parse::<u64>().context("Parsing MemTotal")?);
                }
            } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
                if let Some(val) = rest.trim_start().split_ascii_whitespace().next() {
                    available = Some(val.parse::<u64>().context("Parsing MemAvailable")?);
                }
            } else if let Some(rest) = line.strip_prefix("MemFree:") {
                if let Some(val) = rest.trim_start().split_ascii_whitespace().next() {
                    free = Some(val.parse::<u64>().context("Parsing MemFree")?);
                }
            } else if let Some(rest) = line.strip_prefix("Buffers:") {
                if let Some(val) = rest.trim_start().split_ascii_whitespace().next() {
                    buffers = Some(val.parse::<u64>().context("Parsing Buffers")?);
                }
            } else if let Some(rest) = line.strip_prefix("Cached:") {
                if let Some(val) = rest.trim_start().split_ascii_whitespace().next() {
                    cached = Some(val.parse::<u64>().context("Parsing Cached")?);
                }
            }

            if total.is_some()
                && (available.is_some()
                    || (free.is_some() && buffers.is_some() && cached.is_some()))
            {
                break;
            }
            line.clear();
        }

        let total_kb = total.context("MemTotal not found in /proc/meminfo")?;
        let available_kb = if let Some(av) = available {
            av
        } else {
            free.context("MemFree missing")?
                + buffers.context("Buffers missing")?
                + cached.context("Cached missing")?
        };

        Ok(MemInfo {
            total_kb,
            available_kb,
        })
    }

    pub fn usage_percent(&self) -> f64 {
        let used = self.total_kb.saturating_sub(self.available_kb) as f64;
        if self.total_kb > 0 {
            used / self.total_kb as f64 * 100.0
        } else {
            0.0
        }
    }
}
