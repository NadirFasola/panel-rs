// src/main.rs
extern crate anyhow;
extern crate gtk4;
extern crate gtk4_layer_shell;

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
