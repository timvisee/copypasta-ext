//! Display server management.
//!
//! Provides functionality to select used display server based on the runtime environment.

use std::env;

use crate::prelude::ClipboardProvider;

/// A display server type.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum DisplayServer {
    /// The X11 display server.
    X11,

    /// The Wayland display server.
    Wayland,

    /// The default macOS display server.
    MacOs,

    /// The default Windows display server.
    Windows,

    /// For TTYs.
    /// Not an actual display server, but something with a clipboard context to fall back to.
    Tty,
}

impl DisplayServer {
    /// Select current used display server.
    ///
    /// This selection is made at runtime. This uses a best effort approach and does not reliably
    /// select the current display server. Selects any recognized display server regardless of
    /// compiler feature flag configuration. Defaults to `X11` on Unix if display server could not
    /// be determined.
    #[allow(unreachable_code)]
    pub fn select() -> DisplayServer {
        #[cfg(target_os = "macos")]
        return DisplayServer::MacOs;
        #[cfg(windows)]
        return DisplayServer::Windows;

        // Runtime check on Unix
        if is_wayland() {
            DisplayServer::Wayland
        } else if is_x11() {
            DisplayServer::X11
        } else if is_tty() {
            DisplayServer::Tty
        } else {
            // TODO: return Option::None if this isn't X11 either.
            DisplayServer::X11
        }
    }

    /// Build clipboard context for display server.
    ///
    /// This attempts to build a clipboard context for the selected display server based on what
    /// contexts are available.
    ///
    /// If no compatible context is available or if no compatible context could be initialized,
    /// `None` is returned.
    pub fn try_context(self) -> Option<Box<dyn ClipboardProvider>> {
        match self {
            DisplayServer::X11 => {
                #[cfg(feature = "x11-fork")]
                {
                    let context = crate::x11_fork::ClipboardContext::new();
                    if let Ok(context) = context {
                        return Some(Box::new(context));
                    }
                }
                #[cfg(feature = "x11-bin")]
                {
                    let context = crate::x11_bin::ClipboardContext::new();
                    if let Ok(context) = context {
                        return Some(Box::new(context));
                    }
                }
                None
            }
            DisplayServer::Wayland => {
                #[cfg(feature = "wayland-bin")]
                {
                    let context = crate::wayland_bin::ClipboardContext::new();
                    if let Ok(context) = context {
                        return Some(Box::new(context));
                    }
                }
                None
            }
            DisplayServer::MacOs | DisplayServer::Windows => copypasta::ClipboardContext::new()
                .ok()
                .map(|c| -> Box<dyn ClipboardProvider> { Box::new(c) }),
            DisplayServer::Tty => {
                #[cfg(feature = "osc52")]
                {
                    let context = crate::osc52::ClipboardContext::new();
                    if let Ok(context) = context {
                        return Some(Box::new(context));
                    }
                }
                None
            }
        }
    }
}

/// Check whether we're in an X11 environment.
///
/// This is a best effort, may be unreliable.
/// Checks the `XDG_SESSION_TYPE` and `DISPLAY` environment variables.
/// Always returns false on unsupported platforms such as Windows/macOS.
///
/// Available regardless of the `x11-*` compiler feature flags.
pub fn is_x11() -> bool {
    if !cfg!(all(unix, not(all(target_os = "macos", target_os = "ios")))) {
        return false;
    }

    match option_env!("XDG_SESSION_TYPE") {
        Some("x11") => true,
        Some("wayland") => false,
        _ => has_non_empty_env("DISPLAY"),
    }
}

/// Check whether we're in a Wayland environment.
///
/// This is a best effort, may be unreliable.
/// Checks the `XDG_SESSION_TYPE` and `WAYLAND_DISPLAY` environment variables.
/// Always returns false on Windows/macOS.
///
/// Available regardless of the `wayland-*` compiler feature flags.
pub fn is_wayland() -> bool {
    if !cfg!(all(unix, not(all(target_os = "macos", target_os = "ios")))) {
        return false;
    }

    match option_env!("XDG_SESSION_TYPE") {
        Some("wayland") => true,
        Some("x11") => false,
        _ => has_non_empty_env("WAYLAND_DISPLAY"),
    }
}

/// Check whether we're in a TTY environment.
///
/// This is a basic check and only returns true if `XDG_SESSION_TYPE` is set to `tty` explicitly.
pub fn is_tty() -> bool {
    option_env!("XDG_SESSION_TYPE") == Some("tty")
}

/// Check if an environment variable is set and is not empty.
#[inline]
fn has_non_empty_env(env: &str) -> bool {
    env::var_os(env).map(|v| !v.is_empty()).unwrap_or(false)
}
