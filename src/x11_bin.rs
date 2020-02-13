use std::error::Error as StdError;
use std::fmt;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write};
use std::process::{Command, Stdio};

use clipboard::x11_clipboard::{Clipboard, Selection, X11ClipboardContext};
use clipboard::ClipboardProvider;

// TODO: support XCLIP_BIN paths like ffsend
// TODO: also use `xclip` or `xsel` for getting contents

/// Like [`X11ClipboardContext`][X11ClipboardContext], but invokes [`xclip`][xclip]/[`xsel`][xsel]
/// to set contents.
///
/// This provider ensures the clipboard contents you set remain available even after your
/// application exists, unlike [`X11ClipboardContext`][X11ClipboardContext].
///
/// When setting the clipboard with `set_contents`, the [`xclip`][xclip] or [`xsel`][xsel] binary
/// is invoked to set the contents. These binaries internally fork and stay alive until the
/// clipboard content changes.
///
/// ## Drawbacks
///
/// - Set contents may not be immediately available, because they are set in an external binary.
/// - Requires [`xclip`][xclip] or [`xsel`][xsel] to be available.
/// - May have undefined behaviour if `xclip` or `xsel` are modified.
///
/// [X11ClipboardContext]: ../../clipboard/x11_clipboard/struct.X11ClipboardContext.html
/// [xclip]: https://github.com/astrand/xclip
/// [xsel]: http://www.vergenet.net/~conrad/software/xsel/
pub struct X11BinClipboardContext<S = Clipboard>(X11ClipboardContext<S>)
where
    S: Selection;

impl<S> ClipboardProvider for X11BinClipboardContext<S>
where
    S: Selection,
{
    fn new() -> Result<Self, Box<dyn StdError>> {
        Ok(Self(X11ClipboardContext::new()?))
    }

    fn get_contents(&mut self) -> Result<String, Box<dyn StdError>> {
        self.0.get_contents()
    }

    fn set_contents(&mut self, contents: String) -> Result<(), Box<dyn StdError>> {
        Ok(xclip_set(None, &contents).or_else(|_| xsel_set(None, &contents))?)
    }
}

/// Set clipboard contents using xclip binary.
fn xclip_set(path: Option<String>, contents: &str) -> Result<(), Error> {
    sys_cmd_set(
        "xclip",
        Command::new(path.unwrap_or_else(|| "xclip".into()))
            .arg("-sel")
            .arg("clip"),
        contents,
    )
}

/// Set clipboard contents using xsel binary.
fn xsel_set(path: Option<String>, contents: &str) -> Result<(), Error> {
    sys_cmd_set(
        "xsel",
        Command::new(path.unwrap_or_else(|| "xsel".into())).arg("--clipboard"),
        contents,
    )
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

#[derive(Debug)]
pub enum Error {
    /// The `xclip` or `xsel` binary could not be found on the system, required for clipboard support.
    NoBinary,

    /// An error occurred while using `xclip` or `xsel` to set the clipboard contents.
    /// This problem probably occurred when starting, or while piping the clipboard contents to
    /// the process.
    BinaryIo(&'static str, IoError),

    /// `xclip` or `xsel` unexpectetly exited with a non-successful status code.
    BinaryStatus(&'static str, i32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoBinary => write!(f, "Could not find xclip/xsel binary"),
            Error::BinaryIo(cmd, err) => {
                write!(f, "Failed to access clipboard using {}: {}", cmd, err)
            }
            Error::BinaryStatus(cmd, code) => write!(
                f,
                "Failed to use clipboard, {} exited with status code {}",
                cmd, code
            ),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::BinaryIo(_, err) => Some(err),
            _ => None,
        }
    }
}
