// src/core/window.rs
use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

// Manages the panel window lifecycle
pub struct WindowManager {
    // In future: GTK application and window handles
}

impl WindowManager {
    // Initialises GTK and configuration
    pub fn new() -> Result<Self> {
        // 1. Initialises GTK4
        gtk4::init()?;
        // 2. (TODO) Load configuration from file
        Ok(WindowManager {})
    }

    // Builds and runs the panel UI loop
    pub fn run(&mut self) -> Result<()> {
        // 1. Create a GTK4 Application with a reverse-domain ID
        let app = Application::new(
            Some("com.nadirfasola.panel"),
            Default::default(),
        );

        // 2. When the app activats, build our panel window
        app.connect_activate(move |app| {
            // 2.a. Create a window tied to the application
            let window = ApplicationWindow::new(app);
            window.set_default_size(400,30); // 400 x 30 px window
            window.set_decorated(false); // remove titlebar

            // 2.b. Dock it with layer-shell at the bottom
            window.init_layer_shell();
            window.set_layer(Layer::Top);
            window.set_anchor(Edge::Bottom, true);
            window.set_exclusive_zone(30);
            // gtk4_layer_shell::gtk_layer_init_for_window(&window);
            // gtk4_layer_shell::gtk_layer_set_layer(&window, Layer::Top);
            // gtk4_layer_shell::gtk_layer_set_anchor(&window, Edge::Bottom, true);
            // gtk4_layer_shell::gtk_layer_set_exclusive_zone(&window, 30);

            // 2.c. Show the window (and all its children)
            window.show();
        });

        // 3. Run the GTK4 main loop
        app.run();

        Ok(())
    }
}
