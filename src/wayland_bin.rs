//! Invokes [`wl-copy`/`wl-paste`][wl-clipboard] to access clipboard.
//!
//! This provider allows setting clipboard contentx when using the Wayland display manager.
//!
//! When getting or setting the clipboard, the `wl-copy` and `wl-paste` binary is invoked to manage the
//! contents. When setting the clipboard contents, these binaries are provided by the
//! [`wl-clipboard][wl-clipboard] clipboard manager.
//!
//! The `wl-copy` or `wl-paste` must be in `PATH`. Alternatively the paths of either may be set at
//! compile time using the `WL_COPY_PATH` and `WL_PASTE_PATH` environment variables.
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
//! - Requires `wl-copy` and `wl-paste` binaries from [`wl-clipboard`][wl-clipboard] clipboard manager.
//! - Less performant than alternatives due to binary invocation.
//! - Set contents may not be immediately available, because they are set in an external binary.
//! - May have undefined behaviour if `wl-copy` or `wl-paste` are modified.
//!
//! # Examples
//!
//! ```rust,no_run
//! use copypasta_ext::prelude::*;
//! use copypasta_ext::wayland_bin::WaylandBinClipboardContext;
//!
//! let mut ctx = WaylandBinClipboardContext::new().unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! Use `ClipboardContext` alias for better platform compatability:
//!
//! ```rust,no_run
//! use copypasta_ext::prelude::*;
//! use copypasta_ext::wayland_bin::ClipboardContext;
//!
//! let mut ctx = ClipboardContext::new().unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! [wl-clipboard]: https://github.com/bugaevc/wl-clipboard

use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write};
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;

use copypasta::ClipboardProvider;
use which::which;

/// Platform specific context.
///
/// Alias for `WaylandBinClipboardContext` on supported platforms, aliases to standard
/// `ClipboardContext` provided by `rust-clipboard` on other platforms.
pub type ClipboardContext = WaylandBinClipboardContext;

/// Invokes [`wl-clipboard`][wl-clipboard] binaries to access clipboard.
///
/// See module documentation for more information.
///
/// [wl-clipboard]: https://github.com/bugaevc/wl-clipboard
pub struct WaylandBinClipboardContext(ClipboardType);

impl WaylandBinClipboardContext {
    pub fn new() -> crate::ClipResult<Self> {
        Ok(Self(ClipboardType::select()))
    }
}

impl ClipboardProvider for WaylandBinClipboardContext {
    fn get_contents(&mut self) -> crate::ClipResult<String> {
        Ok(self.0.get()?)
    }

    fn set_contents(&mut self, contents: String) -> crate::ClipResult<()> {
        Ok(self.0.set(&contents)?)
    }
}

/// Available clipboard management binaries.
///
/// Invoke `ClipboardType::select()` to select the best variant to use determined at runtime.
enum ClipboardType {
    /// Use `wl-copy` and `wl-paste` from `wl-clipboard`.
    ///
    /// May contain a binary path if specified at compile time through the `XCLIP_PATH` variable.
    WlClipboard(Option<String>, Option<String>),
}

impl ClipboardType {
    /// Select the clipboard type to use.
    pub fn select() -> Self {
        if option_env!("WL_COPY_PATH").is_some() || option_env!("WL_PASTE_PATH").is_some() {
            ClipboardType::WlClipboard(
                option_env!("WL_COPY_PATH")
                    .filter(|p| !p.trim().is_empty())
                    .map(|p| p.into()),
                option_env!("WL_PASTE_PATH")
                    .filter(|p| !p.trim().is_empty())
                    .map(|p| p.into()),
            )
        } else if which("wl-copy").is_ok() || which("wl-paste").is_ok() {
            ClipboardType::WlClipboard(None, None)
        } else {
            // TODO: should we error here instead, as no clipboard binary was found?
            ClipboardType::WlClipboard(None, None)
        }
    }

    /// Get clipboard contents through the selected clipboard type.
    pub fn get(&self) -> Result<String, Error> {
        match self {
            ClipboardType::WlClipboard(_, path) => sys_cmd_get(
                "wl-paste",
                &mut Command::new(path.as_deref().unwrap_or_else(|| "wl-paste")),
            ),
        }
    }

    /// Set clipboard contents through the selected clipboard type.
    pub fn set(&self, contents: &str) -> Result<(), Error> {
        match self {
            ClipboardType::WlClipboard(path, _) => sys_cmd_set(
                "wl-copy",
                &mut Command::new(path.as_deref().unwrap_or_else(|| "wl-copy")),
                contents,
            ),
        }
    }
}

/// Get clipboard contents using a system command.
fn sys_cmd_get(bin: &'static str, command: &mut Command) -> Result<String, Error> {
    // Spawn the command process for getting the clipboard
    let output = match command.output() {
        Ok(output) => output,
        Err(err) => {
            return Err(match err.kind() {
                IoErrorKind::NotFound => Error::NoBinary,
                _ => Error::BinaryIo(bin, err),
            });
        }
    };

    // Check process status code
    if !output.status.success() {
        return Err(Error::BinaryStatus(bin, output.status.code().unwrap_or(0)));
    }

    // Get and parse output
    String::from_utf8(output.stdout).map_err(Error::NoUtf8)
}

/// Set clipboard contents using a system command.
fn sys_cmd_set(bin: &'static str, command: &mut Command, contents: &str) -> Result<(), Error> {
    // Spawn the command process for setting the clipboard
    let mut process = match command.stdin(Stdio::piped()).stdout(Stdio::null()).spawn() {
        Ok(process) => process,
        Err(err) => {
            return Err(match err.kind() {
                IoErrorKind::NotFound => Error::NoBinary,
                _ => Error::BinaryIo(bin, err),
            });
        }
    };

    // Write the contents to the xclip process
    process
        .stdin
        .as_mut()
        .unwrap()
        .write_all(contents.as_bytes())
        .map_err(|err| Error::BinaryIo(bin, err))?;

    // Wait for process to exit
    let status = process.wait().map_err(|err| Error::BinaryIo(bin, err))?;
    if !status.success() {
        return Err(Error::BinaryStatus(bin, status.code().unwrap_or(0)));
    }

    Ok(())
}

/// Represents Wayland binary related error.
#[derive(Debug)]
pub enum Error {
    /// The `wl-copy` or `wl-paste` binary could not be found on the system, required for clipboard support.
    NoBinary,

    /// An error occurred while using `wl-copy` or `wl-paste` to manage the clipboard contents.
    /// This problem probably occurred when starting, or while piping the clipboard contents
    /// from/to the process.
    BinaryIo(&'static str, IoError),

    /// `wl-copy` or `wl-paste` unexpectetly exited with a non-successful status code.
    BinaryStatus(&'static str, i32),

    /// The clipboard contents could not be parsed as valid UTF-8.
    NoUtf8(FromUtf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoBinary => write!(
                f,
                "Could not find wl-copy or wl-paste binary for clipboard support"
            ),
            Error::BinaryIo(cmd, err) => {
                write!(f, "Failed to access clipboard using {}: {}", cmd, err)
            }
            Error::BinaryStatus(cmd, code) => write!(
                f,
                "Failed to use clipboard, {} exited with status code {}",
                cmd, code
            ),
            Error::NoUtf8(err) => write!(
                f,
                "Failed to parse clipboard contents as valid UTF-8: {}",
                err
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::BinaryIo(_, err) => Some(err),
            Error::NoUtf8(err) => Some(err),
            _ => None,
        }
    }
}
