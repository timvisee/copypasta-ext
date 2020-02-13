//! A clipboard library providing useful extensions for the
//! [`rust-clipboard`][rust-clipboard] library.
//!
//! Here are some of these additions:
//!
//! - [`X11ForkClipboardProvider`](https://docs.rs/clipboard-ext/*/clipboard_ext/x11_fork/index.html):
//!   forks process and sets clipboard, keeps contents after exit
//! - [`X11BinClipboardProvider`](https://docs.rs/clipboard-ext/*/clipboard_ext/x11_bin/index.html):
//!   invokes `xclip`/`xsel` to set clipboard, keeps contents after exit
//! - [`CombinedClipboardProvider`](https://docs.rs/clipboard-ext/*/clipboard_ext/struct.CombinedClipboardContext.html):
//!   combine two providers, use different for getting/setting clipboard
//!
//! # Example
//!
//! Get and set clipboard contents. Keeps contents in X11 clipboard after exit by
//! forking the process. Falls back to standard clipboard provider on non X11 platforms.
//! See [`x11_fork`](https://docs.rs/clipboard-ext/*/clipboard_ext/x11_fork/index.html)
//! module for details.
//!
//! ```rust,no_run
//! use clipboard_ext::prelude::*;
//! use clipboard_ext::x11_fork::ClipboardContext;
//!
//! let mut ctx = ClipboardContext::new().unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! Get and set clipboard contents. Keeps contents in X11 clipboard after exit by
//! invoking `xclip`/`xsel`. Falls back to standard clipboard provider on non X11
//! platforms. See [`x11_bin`](https://docs.rs/clipboard-ext/*/clipboard_ext/x11_bin/index.html)
//! module for details.
//!
//! ```rust,no_run
//! use clipboard_ext::prelude::*;
//! use clipboard_ext::x11_bin::ClipboardContext;
//!
//! let mut ctx = ClipboardContext::new().unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! # Requirements
//!
//! - Rust 1.40 or above
//! - Same requirements as [`rust-clipboard`][rust-clipboard-requirements]
//! - Requirements noted in specific clipboard context modules
//!
//! [rust-clipboard]: https://github.com/aweinstock314/rust-clipboard
//! [rust-clipboard-requirements]: https://github.com/aweinstock314/rust-clipboard#prerequisites

mod combined;
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
    pub type ClipboardContext = clipboard::ClipboardContext;
}
#[cfg(not(all(
    feature = "x11-fork",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
)))]
pub mod x11_fork {
    pub type ClipboardContext = clipboard::ClipboardContext;
}

// Re-export
pub use clipboard;
pub use combined::CombinedClipboardContext;

/// Trait prelude.
///
/// ```rust
/// use clipboard_ext::prelude::*;
/// ```
pub mod prelude {
    pub use super::clipboard::ClipboardProvider;
}
