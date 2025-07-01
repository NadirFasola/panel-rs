// src/core/items/battery/upower_backend.rs

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

pub struct UpowerBackend {
    // conn: Connection,
    device: Proxy<'static>,
    // batteries: Vec<OwnedObjectPath>,
}

impl UpowerBackend {
    pub fn new() -> Result<Self> {
        let conn = Connection::system().context("Failed to connect to the D-Bus")?;

        let upower_path = OwnedObjectPath::try_from(UPOWER_PATH)
            .expect("Static string should be a valid ObjectPath");

        let proxy = Proxy::new::<&str, OwnedObjectPath, &str>(
            &conn,
            UPOWER_SERVICE,
            upower_path,
            UPOWER_IFACE,
        )
        .context("Failed to create UPower proxy")?;

        let list: Vec<OwnedObjectPath> = proxy
            .call("EnumerateDevices", &())
            .context("Enumerating UPower devices")?;

        // let mut batteries: Vec<OwnedObjectPath> = Vec::new();
        let mut batteries = Vec::new();
        for path in list {
            let dev = Proxy::new::<&str, OwnedObjectPath, &str>(
                &conn,
                UPOWER_SERVICE,
                path.clone(),
                DEVICE_IFACE,
            )
            .context("Creating Device proxy")?;
            let typ: u32 = dev.get_property("Type")?;
            if typ == DEVICE_TYPE_BATTERY {
                batteries.push(path);
            }
        }

        if batteries.is_empty() {
            anyhow::bail!("No battery devices found via UPower");
        }

        let battery_path = batteries.remove(0);
        let device = Proxy::new(&conn, UPOWER_SERVICE, battery_path, DEVICE_IFACE)
            .context("Creating Device proxy")?;

        Ok(Self {
            // conn,
            // batteries,
            device,
        })
    }
}

impl BatteryBackend for UpowerBackend {
    fn read(&self) -> Result<(u8, String)> {
        let pct: f64 = self.device.get_property("Percentage")?;
        let state: u32 = self.device.get_property("State")?;

        let status = match state {
            1 => "Unknown",
            2 => "Charging",
            3 => "Discharging",
            4 => "Empty",
            5 => "Fully charged",
            _ => "Other",
        }
        .to_string();

        Ok((pct as u8, status))
    }
}
