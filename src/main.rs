// src/main.rs
extern crate anyhow;
extern crate panel_rs;

use anyhow::Result;
use panel_rs::core::window::WindowManager;

fn main() -> Result<()> {
    // Build the window manager (initialises GTK, loads config)
    let mut wm = WindowManager::new()?;
    // Run the UI loop
    wm.run()?;
    Ok(())
}
