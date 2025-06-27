// src/core/window.rs
use anyhow::{Context, Result};
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use tracing::{error, info};

use super::config::Config;

// Manages the panel window lifecycle
pub struct WindowManager {
    _config: Config,
    // In future: GTK application and window handles
}

impl WindowManager {
    // Initialises GTK and configuration
    pub fn new() -> Result<Self> {
        info!("Initialising WindowManager");
        // 1. Load and validate config
        let config = Config::load().context("Loading application configuration")?;

        // 2. (TODO) Load configuration from file
        info!(?config, "WindowManager initialised with config");
        Ok(WindowManager { _config: config })
    }

    // Builds and runs the panel UI loop
    pub fn run(&mut self) -> Result<()> {
        // 0. Initialize GTK
        info!("Starting GTK event loop");
        gtk4::init()?;

        // 1. Create a GTK4 Application with a reverse-domain ID
        let app = Application::new(Some("com.nadirfasola.panel"), Default::default());

        // 2. When the app activates, build our panel window
        app.connect_activate(move |app| {
            let build_ui = || {
                // 2.a. Create a window tied to the application
                let window = ApplicationWindow::new(app);
                window.set_default_size(400, 30); // 400 x 30 px window
                window.set_decorated(false); // remove titlebar

                // 2.b. Dock it with layer-shell at the bottom
                window.init_layer_shell();
                window.set_layer(Layer::Top);
                window.set_anchor(Edge::Bottom, true);
                window.set_exclusive_zone(30);

                // 2.c. Show the window (and all its children)
                window.show();
            };

            if let Err(panic_err) = std::panic::catch_unwind(build_ui) {
                error!("Panig in GTK callback: {panic_err:?}");
                std::process::exit(1);
            }
        });

        // 3. Run the GTK4 main loop
        app.run();

        Ok(())
    }
}
