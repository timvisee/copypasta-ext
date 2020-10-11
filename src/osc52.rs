//! OSC 52 escape sequence to set clipboard contents.
//!
//! This provider can set clipboard contents by outputting a sequence to stdout in supported
//! terminals. It uses Xterm escape sequences, OSC 52 to be exact.
//!
//! Getting clipboard contents is not supported through this context and will error.
//!
//! ## Benefits
//!
//! - Keeps contents in clipboard for the terminal lifetime even after your application exists.
//!
//! ## Drawbacks
//!
//! - Requires terminal that supports these escape codes.
//! - Doesn't catch errors while setting clipboard contents.
//! - Cannot get clipboard contents.
//!
//! # Examples
//!
//! ```rust,no_run
//! use copypasta_ext::prelude::*;
//! use copypasta_ext::x11_bin::X11BinClipboardContext;
//!
//! let mut ctx = X11BinClipboardContext::new().unwrap();
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! Use `new_with` to combine with another context such as [`X11ClipboardContext`][X11ClipboardContext] to support getting clipboard contents as well.
//!
//! ```rust,no_run
//! use copypasta_ext::prelude::*;
//! use copypasta_ext::osc52::Osc52ClipboardContext;
//! use copypasta_ext::x11_bin::X11BinClipboardContext;
//!
//! let mut ctx = Osc52ClipboardContext::new_with(X11BinClipboardContext::new().unwrap()).unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! [X11ClipboardContext]: https://docs.rs/copypasta/*/copypasta/x11_clipboard/struct.X11ClipboardContext.html

use std::error::Error as StdError;
use std::fmt;

use base64;
use copypasta::ClipboardProvider;

use crate::combined::CombinedClipboardContext;

/// OSC 52 escape sequence to set clipboard contents.
///
/// See module documentation for more information.
pub struct Osc52ClipboardContext;

impl Osc52ClipboardContext {
    pub fn new() -> Result<Self, Box<dyn StdError>> {
        Ok(Self)
    }

    /// Construct combined with another context for getting the clipboard.
    ///
    /// This clipboard context only supports setting the clipboard contents.
    /// You can combine this with the given context to support getting clipboard contents as well
    /// to get the best of both worlds.
    pub fn new_with<G>(get: G) -> Result<CombinedClipboardContext<G, Self>, Box<dyn StdError>>
    where
        G: ClipboardProvider,
    {
        Self::new()?.with(get)
    }

    /// Combine this context with [`X11ClipboardContext`][X11ClipboardContext].
    ///
    /// This clipboard context only supports setting the clipboard contents.
    /// You can combine this with the given context to support getting clipboard contents as well
    /// to get the best of both worlds.
    pub fn with<G>(self, get: G) -> Result<CombinedClipboardContext<G, Self>, Box<dyn StdError>>
    where
        G: ClipboardProvider,
    {
        Ok(CombinedClipboardContext(get, self))
    }
}

impl ClipboardProvider for Osc52ClipboardContext {
    fn get_contents(&mut self) -> crate::ClipResult<String> {
        Err(Error::Unsupported.into())
    }

    fn set_contents(&mut self, contents: String) -> crate::ClipResult<()> {
        // Use OSC 52 escape sequence to set clipboard through stdout
        print!("\x1B]52;c;{}\x07", base64::encode(&contents));
        Ok(())
    }
}

/// Represents OSC 52 clipboard related error.
#[derive(Debug)]
pub enum Error {
    /// Getting clipboard contents is not supported.
    Unsupported,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Unsupported => write!(
                f,
                "Getting clipboard contents is not supported through this context"
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}
