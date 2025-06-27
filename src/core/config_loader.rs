// /src/core/config_loader.rs

use directories::BaseDirs;
use std::path::PathBuf;

pub fn config_paths() -> (PathBuf, PathBuf) {
    // 1. System default: directory of the binary
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    let system_default = exe_dir.join("default.toml");

    // 2. User override in XDG_CONFIG_HOME/panel-rs/config.toml
    let user_config = BaseDirs::new()
        .map(|d| d.config_dir().join("panel-rs").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config/config.toml"));

    (system_default, user_config)
}
