# panel-rs

A compositor-agnostic Wayland panel bar written in Rust & GTK4.

> **Status:** This README reflects the state on branch `feature/sprint-3-essential-widgets`.

---

## Quick summary

* Language: **Rust**
* UI: **GTK4** (GTK widgets for items)
* Backends: sysfs, hwmon, thermal-zone, UPower (zbus), lm-sensors
* Design: plugin-style `Item` trait, per-module TOML configuration, unit-tested parsing/backends

---

## What this project is

`panel-rs` aims to be a small, fast, configurable status bar that is compositor-agnostic (Wayland). It uses GTK4 for widgets and layer-shell for docking. The codebase focuses on:

* Minimal, well-tested backends (prefer sysfs where possible)
* Clear plugin architecture for items
* Small set of dependencies and idiomatic Rust

---

## Current state (short)

* ✅ Clock item (`ClockItem`) — configurable format & refresh rate
* ✅ Battery item (`BatteryItem`) — `SysfsBackend` and `UpowerBackend` (zbus), dynamic icons
* ✅ CPU item (`CpuItem`) — `/proc/stat` backend, optional dynamic icon
* ✅ Memory item (`MemItem`) — `/proc/meminfo`, optional dynamic icon
* ✅ Temperature item (`TempItem`) — thermal zone / hwmon / lm-sensors, optional dynamic icon
* ✅ Config refactor: per-module subconfigs and global defaults
* ✅ Unit tests for parsing/backends
* ✅ `icon.rs` helper module added for centralized icon management

Not implemented yet: system tray (SNI), volume widget, Waybar-like format templates, click popups & animations.

---

## Roadmap & Timeline (checked = done)

* Sprint 1 — skeleton & architecture

  * [x] Project skeleton, Cargo layout, module scaffolding
  * [x] `Item` trait and `ItemManager`
  * [x] `WindowManager` & layer-shell integration
  * [x] Config loader + default TOML

* Sprint 2 — essential items & config

  * [x] `ClockItem` (lazy widget, timers)
  * [x] `BatteryItem` (sysfs, UPower backends)
  * [x] Tests and config refactor

* Sprint 3 — essential widgets (current branch)

  * [x] `CpuItem` (proc/stat)
  * [x] `MemItem` (proc/meminfo)
  * [x] `TempItem` (thermal zone, hwmon, lm-sensors)
  * [x] Unified icon support via `icon.rs`
  * [x] Unit tests for backends
  * [ ] System tray (SNI) — high priority next sprint
  * [ ] Volume widget (Pulse / PipeWire)
  * [ ] Format templates & icons (Waybar-like)
  * [ ] Click-to-open popups (volume, connectivity)
  * [ ] Animations & polished UI

* Future

  * [ ] Pure-Rust SNI (or vendor a small dependency)
  * [ ] Subscribe to DBus signals for instant updates
  * [ ] Packaging (deb / rpm / Flatpak)

---

## Build & run

### Prerequisites

* Rust (stable toolchain)
* GTK4 development libraries (for the UI)
* On Linux: `/proc`, `/sys`, and a session/system bus for D-Bus backends

### Build

```bash
cargo build --release
```

### Run (development)

```bash
RUST_LOG=info cargo run
```

### Run release binary

```bash
./target/release/panel-rs
```

### Tests

```bash
cargo test
```

Notes:

* Many unit tests are pure-Rust and should run in CI. Tests that require GTK/D-Bus might need environment setup.

---

## Configuration (TOML)

The project uses a TOML config with a global section and `modules` subsection. Module configs can override the global `refresh_secs`.

Example `config.toml`:

```toml
items = ["battery", "clock", "cpu", "mem", "temp"]
refresh_secs = 5

[modules.clock]
refresh_secs = 1
# format = "%H:%M:%S"

[modules.battery]
backend = "sysfs"  # or "upower"
refresh_secs = 5
icon = "auto"      # optional dynamic icon, or static path/name

[modules.cpu]
refresh_secs = 5
icon = "cpu-symbolic"  # optional static or dynamic icon

[modules.mem]
refresh_secs = 5
icon = "mem-symbolic"  # optional static or dynamic icon

[modules.temp]
backend = "thermal_zone"  # thermal_zone | hwmon | lmsensors
refresh_secs = 10
sensors = ["x86_pkg_temp", "acpitz"]
icon = "temperature-symbolic"  # optional static or dynamic icon
```

Behavior notes:

* If a module doesn't specify `refresh_secs`, it inherits the global `refresh_secs`.
* `modules.*.sensors` is used by temperature backends to pick only certain sensors; empty means *"auto-discover all"*.
* `modules.*.icon` is optional; *"auto"* or a theme icon name triggers dynamic icon selection where implemented.

---

## Architecture & file map (where to look)

* `src/core/item.rs` — `Item` trait
* `src/core/item_manager.rs` — constructs items from config
* `src/core/window.rs` — GTK app, layer-shell and window layout
* `src/core/config.rs` — config loader and per-module defaults
* `src/core/items/` — each item lives under this directory:

  * `clock/`, `battery/`, `cpu/`, `mem/`, `temp/`
  * backends are in each item's subdirectory (e.g. `battery/sysfs_backend.rs`)

---

## Design choices & rationale

* Prefer *local sysfs* access for metrics (lower dependency surface).
* `zbus` is used only where it makes sense (UPower D-Bus). Tests and fallbacks exist for environments without UPower.
* Timer callbacks run on the main (glib) context to avoid GTK threading issues.
* Discovery operations (like scanning `/sys/class/...`) are cached using `OnceLock`/`OnceCell` to avoid repeated expensive file system traversals.
* Icon handling is centralized to reduce duplication and improve maintainability.
* Unit tests focus on parsers and file-backed logic; GTK UI is intentionally small and isolated.

---

## Contributing

See `CONTRIBUTING.md` for contribution guidelines, coding style and PR workflow. Short notes:

* Keep modules small and testable.
* Prefer `sysfs` where possible, fall back to system services only when necessary.
* Add unit tests for parsing logic and backends.

---

## License

MIT © 2025 Nadir Fasola
