//! A clipboard library providing useful extensions for the
//! [`copypasta`][copypasta] library.
//!
//! Here are some of these additions:
//!
//! - [`X11ForkClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_fork/index.html):
//!   forks process and sets clipboard, keeps contents after exit
//! - [`X11BinClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_bin/index.html):
//!   invokes `xclip`/`xsel` to set clipboard, keeps contents after exit
//! - [`Osc52ClipboardContext`](https://docs.rs/copypasta-ext/*/copypasta_ext/osc52/index.html):
//!   use OSC 52 escape sequence to set clipboard contents
//! - [`CombinedClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/struct.CombinedClipboardContext.html):
//!   combine two providers, use different for getting/setting clipboard
//!
//! # Example
//!
//! Get and set clipboard contents. Keeps contents in X11 clipboard after exit by
//! forking the process. Falls back to standard clipboard provider on non X11 platforms.
//! See [`x11_fork`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_fork/index.html)
//! module for details.
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
#[cfg(feature = "osc52")]
pub mod osc52;
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
    feature = "x11-bin",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
)))]
pub mod x11_bin {
    pub type ClipboardContext = copypasta::ClipboardContext;
}
#[cfg(not(all(
    feature = "x11-fork",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
)))]
pub mod x11_fork {
    pub type ClipboardContext = copypasta::ClipboardContext;
}

use std::error::Error;

/// Copypasta error type, for your convenience.
pub type ClipResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;

// Re-export
pub use combined::CombinedClipboardContext;
pub use copypasta;

/// Trait prelude.
///
/// ```rust
/// use copypasta_ext::prelude::*;
/// ```
pub mod prelude {
    pub use super::copypasta::ClipboardProvider;
}
