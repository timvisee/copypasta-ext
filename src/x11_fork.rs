//! Like [`x11_clipboard`][x11_clipboard], but forks to set contents.
//!
//! This provider ensures the clipboard contents you set remain available even after your
//! application exists, unlike [`X11ClipboardContext`][X11ClipboardContext].
//!
//! When setting the clipboard, the process is forked in which the clipboard is set. The fork is
//! kept alive until the clipboard content changes, and may outlive your application.
//!
//! Use the provided `ClipboardContext` type alias to use this clipboard context on supported
//! platforms, but fall back to the standard clipboard on others.
//!
//! ## Benefits
//!
//! - Keeps contents in clipboard even after your application exists.
//!
//! ## Drawbacks
//!
//! - Set contents may not be immediately available, because they are set in a fork.
//! - Errors when setting the clipboard contents are not catched, the fork will panic
//!   `set_contents` will return no error.
//! - The fork might cause weird behaviour for some applications.
//!
//! # Examples
//!
//! ```rust,no_run
//! use copypasta_ext::prelude::*;
//! use copypasta_ext::x11_fork::X11ForkClipboardContext;
//!
//! let mut ctx: X11ForkClipboardContext = X11ForkClipboardContext::new().unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! Use `ClipboardContext` alias for better platform compatability:
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
//! [copypasta]: https://docs.rs/copypasta/*/copypasta/x11_clipboard/index.html
//! [X11ClipboardContext]: https://docs.rs/copypasta/*/copypasta/x11_clipboard/struct.X11ClipboardContext.html

use std::error::Error as StdError;
use std::fmt;

use copypasta::x11_clipboard::{Clipboard, Selection, X11ClipboardContext};
use libc::fork;
use x11_clipboard::Clipboard as X11Clipboard;

use crate::display::DisplayServer;
use crate::prelude::*;

/// Platform specific context.
///
/// Alias for `X11ForkClipboardContext` on supported platforms, aliases to standard
/// `ClipboardContext` provided by `rust-clipboard` on other platforms.
pub type ClipboardContext = X11ForkClipboardContext;

/// Like [`X11ClipboardContext`][X11ClipboardContext], but forks to set contents.
///
/// `set_contents` forks the process, `get_contents` is an alias for
/// [`X11ClipboardContext::get_contents`][X11ClipboardContext].
///
/// See module documentation for more information.
///
/// [X11ClipboardContext]: https://docs.rs/copypasta/*/copypasta/x11_clipboard/struct.X11ClipboardContext.html
pub struct X11ForkClipboardContext<S = Clipboard>(X11ClipboardContext<S>)
where
    S: Selection;

impl X11ForkClipboardContext {
    pub fn new() -> crate::ClipResult<Self> {
        Ok(Self(X11ClipboardContext::new()?))
    }
}

impl<S> ClipboardProvider for X11ForkClipboardContext<S>
where
    S: Selection,
{
    fn get_contents(&mut self) -> crate::ClipResult<String> {
        self.0.get_contents()
    }

    fn set_contents(&mut self, contents: String) -> crate::ClipResult<()> {
        match unsafe { fork() } {
            -1 => Err(Error::Fork.into()),
            0 => {
                // Obtain new X11 clipboard context, set clipboard contents
                let clip = X11Clipboard::new().expect("failed to obtain X11 clipboard context");
                clip.store(
                    S::atom(&clip.setter.atoms),
                    clip.setter.atoms.utf8_string,
                    contents,
                )
                .expect("failed to set clipboard contents through forked process");

                // Wait for clipboard to change, then kill fork
                clip.load_wait(
                    S::atom(&clip.getter.atoms),
                    clip.getter.atoms.utf8_string,
                    clip.getter.atoms.property,
                )
                .expect("failed to wait on new clipboard value in forked process");

                std::process::exit(0)
            }
            _pid => Ok(()),
        }
    }
}

impl<S> ClipboardProviderExt for X11ForkClipboardContext<S>
where
    S: Selection,
{
    fn display_server(&self) -> Option<DisplayServer> {
        Some(DisplayServer::X11)
    }

    fn has_bin_lifetime(&self) -> bool {
        false
    }
}

/// Represents X11 fork related error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Failed to fork process, to set clipboard in.
    Fork,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Fork => write!(f, "Failed to fork process to set clipboard"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Fork => None,
        }
    }
}
