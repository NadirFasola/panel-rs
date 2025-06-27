// /src/core/config_loader.rs

use directories::BaseDirs;
use std::path::{Path, PathBuf};

pub fn config_paths() -> (PathBuf, PathBuf) {
    // 1. System default: directory of the binary
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let mut system_default = exe_dir.join("default.toml");

    // **Fallback for development**
    // If the system default isn't there, use the project's
    // `config/default.toml` via CARGO_MANIFEST_DIR
    if !system_default.exists() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let fallback = manifest.join("config").join("default.toml");
        if fallback.exists() {
            system_default = fallback;
        }
    }

    // 2. User override in XDG_CONFIG_HOME/panel-rs/config.toml
    let user_config = BaseDirs::new()
        .map(|d| d.config_dir().join("panel-rs").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config/config.toml"));

    (system_default, user_config)
}
