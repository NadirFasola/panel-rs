// src/core/items/cpu/stat_backend.rs

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

// small helper:
fn parse_next<'a, I>(it: &mut I, field: &'static str) -> Result<u64>
where
    I: Iterator<Item = &'a str>,
{
    it.next()
        .context(format!("Missing {field} field in /proc/stat"))?
        .parse::<u64>()
        .context(format!("Parsing {field} jiffy"))
}

// Raw CPU snapshot: idle and total jiffies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuSnapshot {
    pub idle: u64,
    pub total: u64,
}

impl CpuSnapshot {
    // Parse a line startin with "cpu" from /proc/stat
    pub fn from_line(line: &str) -> Result<Self> {
        let mut fields = line.split_ascii_whitespace();
        // 1st field must be “cpu”
        match fields.next() {
            Some("cpu") => {}
            _ => anyhow::bail!("Line did not start with “cpu”: {line}"),
        }
        // Now parse the first 4 or 5 jiffies directly:
        let user = parse_next(&mut fields, "user")?;
        let nice = parse_next(&mut fields, "nice")?;
        let system = parse_next(&mut fields, "system")?;
        let idle = parse_next(&mut fields, "idle")?;
        // optional extra fields: iowait, irq, softirq, steal, guest…
        let mut total = user + nice + system + idle;
        for name in ["iowait", "irq", "softirq", "steal"] {
            if let Some(v) = fields.next() {
                let n = v
                    .parse::<u64>()
                    .with_context(|| format!("Parsing {name} jiffy"))?;
                total += n;
            } else {
                break;
            }
        }
        Ok(CpuSnapshot { idle, total })
    }

    // Read the real /proc/stat and get the first line
    pub fn read_from_proc() -> Result<Self> {
        let f = File::open("/proc/stat").context("Opening /proc/stat")?;
        let mut reader = BufReader::new(f);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .context("Reading cpu line from /proc/stat")?;
        Self::from_line(line.trim_end())
    }

    /// returns one `CpuSnapshot` per line, skipping “cpu” summary
    pub fn all_from_proc() -> Result<Vec<CpuSnapshot>> {
        let f = File::open("/proc/stat")?;
        let reader = BufReader::new(f);
        reader
            .lines()
            .skip(1) // skip header “cpu …”
            .take_while(|l| l.as_ref().map(|l| l.starts_with("cpu")).unwrap_or(false))
            .map(|l| CpuSnapshot::from_line(&l?))
            .collect()
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
