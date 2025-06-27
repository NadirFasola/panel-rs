// src/main.rs
use anyhow::Result;
use panel_rs::core::window::WindowManager;
use std::panic;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

fn main() -> Result<()> {
    // Initialize tracing subscriber for formatted, leveled logs
    //
    // - `EnvFilter::from_default_env()` rads RUST_LOG
    // - `fmt::layer()` prints to stderr with timestamps and levels
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("panel_rs=info"));
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt::layer())
        .init();

    // Example log to verify it's working
    info!("Starting panel_rs v{}", env!("CARGO_PKG_VERSION"));

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
