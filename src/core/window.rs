// src/core/window.rs

use anyhow::Result;

// Manages the panel window lifecycle
pub struct WindowManager {
    // In future: GTK application and window handles
}

impl WindowManager {
    // Initialises GTK and configuration
    pub fn new() -> Result<self> {
        // 1. Initialises GTK4
        gtk4::init()?
        // 2. (TODO) Load configuration from file
        Ok(WindowManager {})
    };

    // Builds and runs the panel UI loop
    pub fn run(&mut self) -> Result<()> {
        // TODO
        Ok(())
    };
}
