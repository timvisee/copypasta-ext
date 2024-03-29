//! Invokes [`xclip`][xclip]/[`xsel`][xsel] to access clipboard.
//!
//! This provider ensures the clipboard contents you set remain available even after your
//! application exists, unlike [`X11ClipboardContext`][X11ClipboardContext].
//!
//! When getting or setting the clipboard, the `xclip` or `xsel` binary is invoked to manage the
//! contents. When setting the clipboard contents, these binaries internally fork and stay alive
//! until the clipboard content changes.
//!
//! The `xclip` or `xsel` must be in `PATH`. Alternatively the paths of either may be set at
//! compile time using the `XCLIP_PATH` and `XSEL_PATH` environment variables. If set, the
//! clipboard context will automatically use those.
//!
//! What binary is used is deterimined at runtime on context creation based on the compile time
//! variables and the runtime environment.
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
//! - Requires [`xclip`][xclip] or [`xsel`][xsel] to be available.
//! - Less performant than alternatives due to binary invocation.
//! - Set contents may not be immediately available, because they are set in an external binary.
//! - May have undefined behaviour if `xclip` or `xsel` are modified.
//!
//! # Examples
//!
//! ```rust,no_run
//! use copypasta_ext::prelude::*;
//! use copypasta_ext::x11_bin::X11BinClipboardContext;
//!
//! let mut ctx = X11BinClipboardContext::new().unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! Use `ClipboardContext` alias for better platform compatability:
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
//! Use `new_with_x11` to combine with [`X11ClipboardContext`][X11ClipboardContext] for better performance.
//!
//! ```rust,no_run
//! use copypasta_ext::prelude::*;
//! use copypasta_ext::x11_bin::X11BinClipboardContext;
//!
//! let mut ctx = X11BinClipboardContext::new_with_x11().unwrap();
//! println!("{:?}", ctx.get_contents());
//! ctx.set_contents("some string".into()).unwrap();
//! ```
//!
//! [X11ClipboardContext]: https://docs.rs/copypasta/*/copypasta/x11_clipboard/struct.X11ClipboardContext.html
//! [x11_clipboard]: https://docs.rs/copypasta/*/copypasta/x11_clipboard/index.html
//! [xclip]: https://github.com/astrand/xclip
//! [xsel]: http://www.vergenet.net/~conrad/software/xsel/

use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write};
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;

use copypasta::x11_clipboard::X11ClipboardContext;
use which::which;

use crate::combined::CombinedClipboardContext;
use crate::display::DisplayServer;
use crate::prelude::*;

/// Platform specific context.
///
/// Alias for `X11BinClipboardContext` on supported platforms, aliases to standard
/// `ClipboardContext` provided by `rust-clipboard` on other platforms.
pub type ClipboardContext = X11BinClipboardContext;

/// Invokes [`xclip`][xclip]/[`xsel`][xsel] to access clipboard.
///
/// See module documentation for more information.
///
/// [xclip]: https://github.com/astrand/xclip
/// [xsel]: http://www.vergenet.net/~conrad/software/xsel/
pub struct X11BinClipboardContext(ClipboardType);

impl X11BinClipboardContext {
    pub fn new() -> crate::ClipResult<Self> {
        Ok(Self(ClipboardType::select()))
    }

    /// Construct combined with [`X11ClipboardContext`][X11ClipboardContext].
    ///
    /// This clipboard context invokes a binary for getting the clipboard contents. This may
    /// be considered inefficient and has other drawbacks as noted in the struct documentation.
    /// This function also constructs a `X11ClipboardContext` for getting clipboard contents and
    /// combines the two to get the best of both worlds.
    ///
    /// [X11ClipboardContext]: https://docs.rs/copypasta/*/copypasta/x11_clipboard/struct.X11ClipboardContext.html
    pub fn new_with_x11() -> crate::ClipResult<CombinedClipboardContext<X11ClipboardContext, Self>>
    {
        Self::new()?.with_x11()
    }

    /// Combine this context with [`X11ClipboardContext`][X11ClipboardContext].
    ///
    /// This clipboard context invokes a binary for getting the clipboard contents. This may
    /// be considered inefficient and has other drawbacks as noted in the struct documentation.
    /// This function constructs a `X11ClipboardContext` for getting clipboard contents and
    /// combines the two to get the best of both worlds.
    ///
    /// [X11ClipboardContext]: https://docs.rs/copypasta/*/copypasta/x11_clipboard/struct.X11ClipboardContext.html
    pub fn with_x11(
        self,
    ) -> crate::ClipResult<CombinedClipboardContext<X11ClipboardContext, Self>> {
        Ok(CombinedClipboardContext(X11ClipboardContext::new()?, self))
    }
}

impl ClipboardProvider for X11BinClipboardContext {
    fn get_contents(&mut self) -> crate::ClipResult<String> {
        Ok(self.0.get()?)
    }

    fn set_contents(&mut self, contents: String) -> crate::ClipResult<()> {
        Ok(self.0.set(&contents)?)
    }
}

impl ClipboardProviderExt for X11BinClipboardContext {
    fn display_server(&self) -> Option<DisplayServer> {
        Some(DisplayServer::X11)
    }

    fn has_bin_lifetime(&self) -> bool {
        false
    }
}

/// Available clipboard management binaries.
///
/// Invoke `ClipboardType::select()` to select the best variant to use determined at runtime.
enum ClipboardType {
    /// Use `xclip`.
    ///
    /// May contain a binary path if specified at compile time through the `XCLIP_PATH` variable.
    Xclip(Option<String>),

    /// Use `xsel`.
    ///
    /// May contain a binary path if specified at compile time through the `XSEL_PATH` variable.
    Xsel(Option<String>),
}

impl ClipboardType {
    /// Select the clipboard type to use.
    pub fn select() -> Self {
        if let Some(path) = option_env!("XCLIP_PATH") {
            ClipboardType::Xclip(Some(path.to_owned()))
        } else if let Some(path) = option_env!("XSEL_PATH") {
            ClipboardType::Xsel(Some(path.to_owned()))
        } else if which("xclip").is_ok() {
            ClipboardType::Xclip(None)
        } else if which("xsel").is_ok() {
            ClipboardType::Xsel(None)
        } else {
            // TODO: should we error here instead, as no clipboard binary was found?
            ClipboardType::Xclip(None)
        }
    }

    /// Get clipboard contents through the selected clipboard type.
    pub fn get(&self) -> Result<String, Error> {
        match self {
            ClipboardType::Xclip(path) => sys_cmd_get(
                "xclip",
                Command::new(path.as_deref().unwrap_or("xclip"))
                    .arg("-sel")
                    .arg("clip")
                    .arg("-out"),
            ),
            ClipboardType::Xsel(path) => sys_cmd_get(
                "xsel",
                Command::new(path.as_deref().unwrap_or("xsel"))
                    .arg("--clipboard")
                    .arg("--output"),
            ),
        }
    }

    /// Set clipboard contents through the selected clipboard type.
    pub fn set(&self, contents: &str) -> Result<(), Error> {
        match self {
            ClipboardType::Xclip(path) => sys_cmd_set(
                "xclip",
                Command::new(path.as_deref().unwrap_or("xclip"))
                    .arg("-sel")
                    .arg("clip"),
                contents,
            ),
            ClipboardType::Xsel(path) => sys_cmd_set(
                "xsel",
                Command::new(path.as_deref().unwrap_or("xsel")).arg("--clipboard"),
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

/// Represents X11 binary related error.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The `xclip` or `xsel` binary could not be found on the system, required for clipboard support.
    NoBinary,

    /// An error occurred while using `xclip` or `xsel` to manage the clipboard contents.
    /// This problem probably occurred when starting, or while piping the clipboard contents
    /// from/to the process.
    BinaryIo(&'static str, IoError),

    /// `xclip` or `xsel` unexpectetly exited with a non-successful status code.
    BinaryStatus(&'static str, i32),

    /// The clipboard contents could not be parsed as valid UTF-8.
    NoUtf8(FromUtf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoBinary => write!(
                f,
                "Could not find xclip or xsel binary for clipboard support"
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
