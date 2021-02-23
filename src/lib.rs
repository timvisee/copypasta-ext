//! A clipboard library providing useful extensions for the
//! [`copypasta`][copypasta] library.
//!
//! Here are some of these additions:
//!
//! - [`X11ForkClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_fork/index.html):
//!   forks process and sets clipboard on X11, keeps contents after exit
//! - [`X11BinClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_bin/index.html):
//!   invokes `xclip`/`xsel` to set clipboard on X11, keeps contents after exit
//! - [`WaylandBinClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/wayland_bin/index.html):
//!   invokes `wl-copy`/`wl-paste` to set clipboard on Wayland
//! - [`Osc52ClipboardContext`](https://docs.rs/copypasta-ext/*/copypasta_ext/osc52/index.html):
//!   use OSC 52 escape sequence to set clipboard contents
//! - [`CombinedClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/struct.CombinedClipboardContext.html):
//!   combine two providers, use different for getting/setting clipboard
//!
//! # Example
//!
//! Get and set clipboard contents. Tries to select the correct clipboard context at runtime using
//! `try_context`. Useful if you just want quick access to the clipboard, and if you don't want to
//! implement any clipboard context selecting logic yourself.
//!
//! ```rust,no_run
//! let mut ctx = copypasta_ext::try_context().expect("failed to get clipboard context");
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! Get and set clipboard contents. Keeps contents in X11 clipboard after exit by forking the
//! process (which normally doesn't work with copypasta's X11ClipboardContext). Falls back to
//! standard clipboard provider on non X11 platforms. See
//! [`x11_fork`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_fork/index.html) module for
//! details.
//!
//! ```rust,no_run
//! use copypasta_ext::prelude::*;
//! use copypasta_ext::x11_fork::ClipboardContext;
//!
//! let mut ctx = ClipboardContext::new().unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! Get and set clipboard contents. Keeps contents in X11 clipboard after exit by
//! invoking `xclip`/`xsel`. Falls back to standard clipboard provider on non X11
//! platforms. See [`x11_bin`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_bin/index.html)
//! module for details.
//!
//! ```rust,no_run
//! use copypasta_ext::prelude::*;
//! use copypasta_ext::x11_bin::ClipboardContext;
//!
//! let mut ctx = ClipboardContext::new().unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! # Requirements
//!
//! - Rust 1.41 or above
//! - Same requirements as [`copypasta`][copypasta]
//! - Requirements noted in specific clipboard context modules
//!
//! [copypasta]: https://github.com/alacritty/copypasta

mod combined;
pub mod display;
#[cfg(feature = "osc52")]
pub mod osc52;
#[cfg(all(
    feature = "wayland-bin",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
pub mod wayland_bin;
#[cfg(all(
    feature = "x11-bin",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
pub mod x11_bin;
#[cfg(all(
    feature = "x11-fork",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
pub mod x11_fork;

// Expose platform specific contexts
#[cfg(not(all(
    feature = "wayland-bin",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
)))]
pub mod wayland_bin {
    /// No Wayland binary (`wayland-bin`) support. Fallback to `copypasta::ClipboardContext`.
    pub type ClipboardContext = copypasta::ClipboardContext;
}
#[cfg(not(all(
    feature = "x11-bin",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
)))]
pub mod x11_bin {
    /// No X11 binary (`x11-bin`) support. Fallback to `copypasta::ClipboardContext`.
    pub type ClipboardContext = copypasta::ClipboardContext;
}
#[cfg(not(all(
    feature = "x11-fork",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
)))]
pub mod x11_fork {
    /// No X11 fork (`x11-fork`) support. Fallback to `copypasta::ClipboardContext`.
    pub type ClipboardContext = copypasta::ClipboardContext;
}

use std::error::Error;

/// Copypasta result type, for your convenience.
pub type ClipResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;

// Re-export
pub use combined::CombinedClipboardContext;
pub use copypasta;

/// Try to get clipboard context.
///
/// This attempts to obtain a clipboard context suitable for the current environment. This checks
/// at runtime which clipboard contexts are available and which is best suited. If no compatible
/// clipboard context is avaiable, or if initializing a context failed, `None` is returned.
///
/// Note: this function may be used to automatically select an X11 or Wayland clipboard on Unix
/// systems based on the runtime environment.
pub fn try_context() -> Option<Box<dyn prelude::ClipboardProvider>> {
    display::DisplayServer::select().try_context()
}

/// Trait prelude.
///
/// ```rust
/// use copypasta_ext::prelude::*;
/// ```
pub mod prelude {
    pub use super::copypasta::ClipboardProvider;
}
