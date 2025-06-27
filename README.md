# panel-rs

A compositor-agnostic Wayland panel bar writte in Rust & GTK.

## Overview

- __Layer shell__ based docking
- Workspace switcher, clock, battery, system tray
- Configurable via TOML

## Getting started

1. __Install dependencies__
    - GTK4 dev libraries
    - Rust & Cargo
1. __Build__
    ```bash
    cargo build --release
    ```
1. __Run__
    ```bash
    ./target/release/panel-rs
    ```

## Usage

### Configuration

By default, the system config lives in the repository under `config/deafult.toml`.

To override settings (for example, to change which items appear or tweak refresh intervals), copy this file into your user config directory `$XDG_CONFIG_HOME/panel-rs`. You can then edit `$XDG_CONFIG_HOME/panel-rs/config.toml` to your liking. When you next run `panel-rs`, it will load your user config instead of the bundled default.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

MIT &copy; 2025 Nadir Fasola.
