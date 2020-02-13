use std::error::Error as StdError;
use std::fmt;

use clipboard::x11_clipboard::{Clipboard, Selection, X11ClipboardContext};
use clipboard::ClipboardProvider;
use libc::fork;
use x11_clipboard::Clipboard as X11Clipboard;

/// Like [`X11ClipboardContext`][X11ClipboardContext], but forks to set contents.
///
/// This provider ensures the clipboard contents you set remain available even after your
/// application exists, unlike [`X11ClipboardContext`][X11ClipboardContext].
///
/// When setting the clipboard with `set_contents`, the process is forked in which the clipboard is
/// set. The fork is kept alive until the clipboard content changes, and may outlive your
/// application.
///
/// ## Drawbacks
///
/// - Set contents may not be immediately available, because they are set in a fork.
/// - Errors when setting the clipboard contents are not catched, the fork will panic
///   `set_contents` will return no error.
/// - The fork might cause weird behaviour for some applications.
///
/// [X11ClipboardContext]: ../../clipboard/x11_clipboard/struct.X11ClipboardContext.html
pub struct X11ForkClipboardContext<S = Clipboard>(X11ClipboardContext<S>)
where
    S: Selection;

impl<S> ClipboardProvider for X11ForkClipboardContext<S>
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

#[derive(Debug)]
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
