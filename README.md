# panel-rs

A compositor-agnostic Wayland panel bar writte in Rust & GTK.

## Overview

- **Layer shell** based docking
- Workspace switcher, clock, battery, system tray
- Configurable via TOML

## Getting started

1. **Install dependencies**
    - GTK4 dev libraries
    - Rust & Cargo
1. **Build**
    ```bash
    cargo build --release
    ```
1. **Run**
    ```bash
    ./target/release/panel-rs
    ```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

MIT &copy; 2025 Nadir Fasola.
