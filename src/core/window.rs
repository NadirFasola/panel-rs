// src/core/window.rs

use anyhow::{Context, Result};
use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, CssProvider, Orientation,
    STYLE_PROVIDER_PRIORITY_APPLICATION, style_context_add_provider_for_display,
};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use tracing::{error, info};

use super::config::Config;
use super::item_manager::ItemManager;

// Manages the panel window lifecycle
pub struct WindowManager {
    _config: Config,
    // In future: GTK application and window handles
}

impl WindowManager {
    fn load_css() {
        // 1. Create a new CSS provider
        let provider = CssProvider::new();

        // 2. Load your stylesheet from disk (no Result—this returns `()`)
        //    If you need error signals, you can connect to parsing_error on `provider`.
        let css_path = "assets/style.css";
        provider.load_from_path(css_path);

        // 3. Grab the default GDK Display
        let display = Display::default().expect("Could not get default GDK Display");

        // 4. Add the provider for *all* contexts on this display
        //    This is the correct replacement for gtk_style_context_add_provider_for_display
        style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
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

        WindowManager::load_css();

        // Clone config so we can move it into the ItemManager
        let config = self._config.clone();
        // Build the ItemManager from the config
        let manager = ItemManager::load(&config);
        info!(
            num_items = manager.items().len(),
            "Loaded items from config"
        );

        // 1. Create a GTK4 Application with a reverse-domain ID
        let app = Application::new(Some("com.nadirfasola.panel"), Default::default());

        // 2. When the app activates, build our panel window
        app.connect_activate(move |app| {
            // Create a window tied to the application
            let window = ApplicationWindow::new(app);
            window.set_default_size(400, 30); // 400 x 30 px window
            window.set_decorated(false); // remove titlebar

            // Dock it with layer-shell at the bottom
            window.init_layer_shell();
            window.set_layer(Layer::Top);
            window.set_anchor(Edge::Bottom, true);
            window.set_exclusive_zone(30);
            window.set_widget_name("panel-window");

            // Create the bar's main container
            let container = Box::new(Orientation::Horizontal, 0);

            // For each item, build its widget and add it
            for item in manager.items() {
                let widget = item.widget();
                container.append(&widget);
            }

            // Set the container as the window's sole child
            window.set_child(Some(&container));

            // Show the window (and all its children)
            window.show();

            // After showing, start each item's background logic
            for item in manager.items() {
                if let Err(e) = item.start() {
                    // Log but don't panic
                    // One item failing shouldn't kill the bar
                    error!(item = item.name(), error = %e, "Failed to start item");
                }
            }
        });

        // 3. Run the GTK4 main loop
        app.run();

        Ok(())
    }
}
