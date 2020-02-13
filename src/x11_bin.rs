//! Like [`x11_clipboard`][x11_clipboard], but invokes [`xclip`][xclip]/[`xsel`][xsel] to access
//! clipboard.
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
//! [x11_clipboard]: https://docs.rs/clipboard/*/clipboard/x11_clipboard/index.html
//! [X11ClipboardContext]: https://docs.rs/clipboard/0.5.0/clipboard/x11_clipboard/struct.X11ClipboardContext.html
//! [xclip]: https://github.com/astrand/xclip
//! [xsel]: http://www.vergenet.net/~conrad/software/xsel/

use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write};
use std::process::{Command, Stdio};
use std::string::FromUtf8Error;

use clipboard::{x11_clipboard::X11ClipboardContext, ClipboardProvider};
use which::which;

use crate::combined::CombinedClipboardContext;

/// Platform specific context.
///
/// Alias for `X11BinClipboardContext` on supported platforms, aliases to standard
/// `ClipboardContext` provided by `rust-clipboard` on other platforms.
pub type ClipboardContext = X11BinClipboardContext;

/// Like [`X11ClipboardContext`][X11ClipboardContext], but invokes [`xclip`][xclip]/[`xsel`][xsel]
/// to access clipboard.
///
/// See module documentation for more information.
///
/// [X11ClipboardContext]: https://docs.rs/clipboard/0.5.0/clipboard/x11_clipboard/struct.X11ClipboardContext.html
/// [xclip]: https://github.com/astrand/xclip
/// [xsel]: http://www.vergenet.net/~conrad/software/xsel/
pub struct X11BinClipboardContext(ClipboardType);

impl X11BinClipboardContext {
    /// Construct combined with [`X11ClipboardContext`][X11ClipboardContext].
    ///
    /// This clipboard context also invokes a binary for getting the clipboard contents. This may
    /// be considered inefficient and has other drawbacks as noted in the struct documentation.
    /// This function also constructs a `X11ClipboardContext` for getting clipboard contents and
    /// combines the two to get the best of both worlds.
    ///
    /// [X11ClipboardContext]: https://docs.rs/clipboard/0.5.0/clipboard/x11_clipboard/struct.X11ClipboardContext.html
    pub fn new_with_x11(
    ) -> Result<CombinedClipboardContext<X11ClipboardContext, Self>, Box<dyn StdError>> {
        Self::new()?.with_x11()
    }

    /// Combine this context with [`X11ClipboardContext`][X11ClipboardContext].
    ///
    /// This clipboard context also invokes a binary for getting the clipboard contents. This may
    /// be considered inefficient and has other drawbacks as noted in the struct documentation.
    /// This function constructs a `X11ClipboardContext` for getting clipboard contents and
    /// combines the two to get the best of both worlds.
    ///
    /// [X11ClipboardContext]: https://docs.rs/clipboard/0.5.0/clipboard/x11_clipboard/struct.X11ClipboardContext.html
    pub fn with_x11(
        self,
    ) -> Result<CombinedClipboardContext<X11ClipboardContext, Self>, Box<dyn StdError>> {
        Ok(CombinedClipboardContext(X11ClipboardContext::new()?, self))
    }
}

impl ClipboardProvider for X11BinClipboardContext {
    fn new() -> Result<Self, Box<dyn StdError>> {
        Ok(Self(ClipboardType::select()))
    }

    fn get_contents(&mut self) -> Result<String, Box<dyn StdError>> {
        Ok(self.0.get()?)
    }

    fn set_contents(&mut self, contents: String) -> Result<(), Box<dyn StdError>> {
        Ok(self.0.set(&contents)?)
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
                Command::new(path.as_deref().unwrap_or_else(|| "xclip"))
                    .arg("-sel")
                    .arg("clip")
                    .arg("-out"),
            ),
            ClipboardType::Xsel(path) => sys_cmd_get(
                "xsel",
                Command::new(path.as_deref().unwrap_or_else(|| "xsel"))
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
                Command::new(path.as_deref().unwrap_or_else(|| "xclip"))
                    .arg("-sel")
                    .arg("clip"),
                contents,
            ),
            ClipboardType::Xsel(path) => sys_cmd_set(
                "xsel",
                Command::new(path.as_deref().unwrap_or_else(|| "xsel")).arg("--clipboard"),
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
