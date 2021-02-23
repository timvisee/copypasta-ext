//! Display server management.
//!
//! Provides functionality to select used display server based on the runtime environment.

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
}

impl DisplayServer {
    /// Select current used display server.
    ///
    /// This selection is made at runtime. This uses a best effort approach and does not reliably
    /// select the current display server.
    #[allow(unreachable_code)]
    pub fn select() -> DisplayServer {
        #[cfg(target_os = "macos")]
        return DisplayServer::MacOs;
        #[cfg(windows)]
        return DisplayServer::Windows;

        // Runtime check on Unix
        // TODO: improve this, see: https://unix.stackexchange.com/a/355476/61092
        let dm = option_env!("XDG_SESSION_TYPE");
        if dm == Some("wayland") || (dm != Some("x11") && option_env!("WAYLAND_DISPLAY").is_some())
        {
            return DisplayServer::Wayland;
        } else {
            return DisplayServer::X11;
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
        }
    }
}
