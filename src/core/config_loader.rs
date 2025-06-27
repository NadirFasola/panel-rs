// /src/core/config_loader.rs

use directories::BaseDirs;
use std::path::PathBuf;

fn config_paths() -> (PathBuf, PathBuf) {
    // 1. System default: directory of the binary
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.paren().map(|d| d.to_path_bud()))
        .unwrap_or_else(|| PathBuf::from("."));
    let system_default = exe_dir.join("default.toml");

    // 2. User override in XDG_CONFIG_HOME/panel-rs/config.toml
    let user_config = BaseDirs::new()
        .map(|d| d.config_dir().join("panel.rs").joing("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config/config.toml"));

    (system_default, user_config)
}
