// src/core/items/battery/upower_backend.rs

use super::super::super::config::BatteryConfig;
use super::item::BatteryBackend;
use anyhow::{Context, Result};
use std::convert::TryFrom;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::OwnedObjectPath;

/// UPower constants
const UPOWER_SERVICE: &str = "org.freedesktop.UPower";
const UPOWER_PATH: &str = "/org/freedesktop/UPower";
const UPOWER_IFACE: &str = "org.freedesktop.UPower";
const DEVICE_IFACE: &str = "org.freedesktop.UPower.Device";
const DEVICE_TYPE_BATTERY: u32 = 2;

// A `BatteryBackend` that talks to the system D-Bus UPower service
pub struct UpowerBackend {
    device: Proxy<'static>,
}

impl UpowerBackend {
    pub fn new(cfg: &BatteryConfig) -> Result<Self> {
        let conn = Connection::system().context("Failed to connect to the D‑Bus")?;

        // enumerate all devices once
        let proxy = Proxy::new::<_, _, &str>(
            &conn,
            UPOWER_SERVICE,
            OwnedObjectPath::try_from(UPOWER_PATH)?,
            UPOWER_IFACE,
        )?;
        let paths: Vec<OwnedObjectPath> = proxy.call("EnumerateDevices", &())?;

        // filter only batteries
        let mut batteries = Vec::new();
        for path in paths {
            let dev = Proxy::new(&conn, UPOWER_SERVICE, path.clone(), DEVICE_IFACE)?;
            if dev.get_property::<u32>("Type")? == DEVICE_TYPE_BATTERY {
                batteries.push((path.clone(), dev));
            }
        }
        if batteries.is_empty() {
            anyhow::bail!("No battery devices found via UPower");
        }

        // if user specified one, pick that; otherwise pick the first
        let device_proxy = if let Some(ref want) = cfg.device {
            // try to match either the object path or the “native path” property
            batteries
                .into_iter()
                .find_map(|(path, dev)| {
                    let native: String = dev.get_property("NativePath").ok()?;
                    if &path.to_string() == want || &native == want {
                        Some(dev)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("No UPower device matching '{}'", want))?
        } else {
            // default
            batteries.into_iter().next().unwrap().1
        };

        Ok(Self {
            device: device_proxy,
        })
    }
}

impl BatteryBackend for UpowerBackend {
    fn read(&self) -> Result<(u8, String)> {
        let pct: f64 = self
            .device
            .get_property("Percentage")
            .context("Getting UPower Percentage")?;
        let state: u32 = self
            .device
            .get_property("State")
            .context("Getting UPower state")?;

        let status = match state {
            0 => "Unknown",
            1 => "Charging",
            2 => "Discharging",
            3 => "Empty",
            4 => "Fully charged",
            _ => "Other",
        }
        .to_string();

        Ok((pct as u8, status))
    }
}
