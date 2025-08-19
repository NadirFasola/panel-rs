// src/core/utils/icon.rs
//! Icon loading helper: load icon from file path or theme name, with a small in-process cache.
//!
//! It also installs a one-time watcher for icon-theme changes and clears the cache automatically
//! when the theme changes.
//!
//! Public API:
//!   - load_icon(spec: Option<&str>, pixel_size: i32) -> anyhow::Result<Option<gtk4::Image>>
//!   - load_paintable(spec: Option<&str>, pixel_size: i32) -> anyhow::Result<Option<gdk::Paintable>>
//!   - clear_cache()
//!   - apply_paintable(img: &Image, paintable: Option<&Paintable>)
//!   - image_from_spec(spec, size, css_class) -> Option<Image>
//!
//! Usage: call `load_icon(cfg.icon.as_deref(), 16)?` or `image_from_spec(...)`
//! from your widget creation code (must be called on GTK main thread).

use anyhow::Result;
use gtk4::prelude::*;
use gtk4::{
    IconLookupFlags, IconPaintable, IconTheme, Image, TextDirection,
    gdk::{Display, Paintable, Texture},
    gdk_pixbuf::Pixbuf,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Once;
use tracing::warn;

// Thread-local cache: key is "<spec>|<size>" -> Paintable.
// Using `thread_local!` avoids `Sync`/`Send` trouble with GTK objects because
// the cache is meant to be used only from the GTK main thread.
thread_local! {
    static ICON_CACHE: RefCell<HashMap<String, Paintable>> = RefCell::new(HashMap::new());
}

/// Run one-time initialization (connect to theme-changed signal).
/// We use `Once` so we only ever connect a single handler.
static INIT: Once = Once::new();

/// Ensure the "theme changed" watcher is installed once. This function is cheap to call
/// repeatedly; the `Once` guarantees the handler is only installed once.
fn init_theme_watcher() {
    INIT.call_once(|| {
        if let Some(display) = Display::default() {
            let theme = IconTheme::for_display(&display);

            theme.connect_changed(|_theme| {
                clear_cache();
                tracing::info!("Icon theme changed: cleared icon cache");
            });
        } else {
            tracing::warn!("No GDK Display available: icon theme watcher not installed");
        }
    });
}

/// Public: load a Paintable for `spec` at `pixel_size`.
///
/// Returns `Ok(Some(Paintable))` on success or `Ok(None)` if not found.
/// This is the cheap, cache-backed helper the panel should call when it wants
/// to update an `Image` via `image.set_paintable(Some(&paint))`.
pub fn load_paintable(spec: Option<&str>, pixel_size: i32) -> Result<Option<Paintable>> {
    init_theme_watcher();

    let spec = match spec {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return Ok(None),
    };

    let key = format!("{spec}|{pixel_size}");

    if let Some(p) = get_cached_paintable(&key) {
        return Ok(Some(p));
    }

    let path = Path::new(spec);
    if path.exists() {
        match Pixbuf::from_file(spec) {
            Ok(pix) => {
                // Convert Pixbuf -> Texture (a Paintable) and cache it.
                let texture = Texture::for_pixbuf(&pix);
                let paint: Paintable = texture.upcast();
                cache_paintable(key.clone(), &paint);
                return Ok(Some(paint));
            }
            Err(e) => {
                warn!(%spec, error = %e, "Failed to load icon from file; falling back to theme");
            }
        }
    }

    if let Some(display) = Display::default() {
        let theme = IconTheme::for_display(&display);
        let flags = IconLookupFlags::FORCE_REGULAR;

        let icon_paintable: IconPaintable = theme.lookup_icon(
            spec,
            &[], // no fallbacks
            pixel_size,
            1, // scale
            TextDirection::None,
            flags,
        );

        let paint: Paintable = icon_paintable.upcast();
        cache_paintable(key, &paint);
        Ok(Some(paint))
    } else {
        tracing::warn!(%spec, "No GDK Display available; cannot lookup theme icon");
        Ok(None)
    }
}

/// Public convenience: load an Image directly from spec (uses `load_paintable` under the hood)
pub fn load_icon(spec: Option<&str>, pixel_size: i32) -> Result<Option<Image>> {
    if let Some(paint) = load_paintable(spec, pixel_size)? {
        let img = Image::from_paintable(Some(&paint));
        img.set_pixel_size(pixel_size);
        Ok(Some(img))
    } else {
        Ok(None)
    }
}

/// Get a cached Paintable if available
fn get_cached_paintable(key: &str) -> Option<Paintable> {
    ICON_CACHE.with(|c| c.borrow().get(key).cloned())
}

/// Cache a Paintable
fn cache_paintable(key: String, paint: &Paintable) {
    ICON_CACHE.with(|c| {
        c.borrow_mut().insert(key, paint.clone());
    });
}

/// Clear the local cache (public so callers can force it).
pub fn clear_cache() {
    ICON_CACHE.with(|c| {
        c.borrow_mut().clear();
    });
}

/// Apply a paintable to an Image safely, handling the None case.
/// This avoids repeating the type annotations everywhere.
pub fn apply_paintable(img: &Image, paintable: Option<&Paintable>) {
    img.set_paintable(paintable);
}

/// Build a gtk4::Image directly from a spec (file path or theme icon).
/// Optionally attach a CSS class. Returns None if nothing could be loaded.
pub fn image_from_spec(
    spec: Option<&str>,
    pixel_size: i32,
    css_class: Option<&str>,
) -> Option<Image> {
    match load_paintable(spec, pixel_size) {
        Ok(Some(paint)) => {
            let img = Image::from_paintable(Some(&paint));
            img.set_pixel_size(pixel_size);
            if let Some(class) = css_class {
                img.style_context().add_class(class);
            }
            Some(img)
        }
        Ok(None) => None,
        Err(e) => {
            tracing::warn!(?spec, %e, "Failed to build Image via loader");
            None
        }
    }
}

/// Ensure an Image exists in `slot` (RefCell<Option<Image>>), create if missing.
/// Optionally tries to populate it from `spec` and applies `class` and `pixel_size`.
pub fn ensure_icon(
    slot: &RefCell<Option<Image>>,
    spec: Option<&str>,
    pixel_size: i32,
    class: Option<&str>,
) -> Image {
    let mut slot_mut = slot.borrow_mut();
    if slot_mut.is_none() {
        let img = Image::new();
        if let Some(cls) = class {
            img.style_context().add_class(cls);
        }
        img.set_pixel_size(pixel_size);

        if let Some(s) = spec {
            if let Ok(Some(p)) = load_paintable(Some(s), pixel_size) {
                apply_paintable(&img, Some(&p));
            }
        }

        *slot_mut = Some(img);
    }
    slot_mut.as_ref().unwrap().clone()
}
