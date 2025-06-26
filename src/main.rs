// src/main.rs

mod core;

use anyhow::Result;
use core::window::WindowManager;

fn main() -> Result<()> {
    // Build the window manager (initialises GTK, loads config)
    let mut wm = WindowManager::new()?;
    // Run the UI loop
    wm.run()?;
    Ok(())
}
