// src/main.rs

use anyhow::Result;
use panel_rs::core::window::WindowManager;
use std::panic;

fn main() -> Result<()> {
    // Set a custom panic hook.
    // This replaces the default panic handler.
    // Print panic info and immediately terminate the process
    // with exit code 1.
    panic::set_hook(Box::new(|info| {
        // info carries panic information + location
        eprintln!("Application panicked: {info}");
        std::process::exit(1);
    }));

    // Build the window manager (initialises GTK, loads config)
    let mut wm = WindowManager::new()?;
    // Run the UI loop
    wm.run()?;
    Ok(())
}
