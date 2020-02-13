use std::error::Error as StdError;
use std::fmt;
use std::marker::PhantomData;

use clipboard::x11_clipboard::{Clipboard, Selection, X11ClipboardContext};
use clipboard::{ClipboardContext, ClipboardProvider};
use libc::fork;

pub struct X11ForkClipboarContext<S = Clipboard>(X11ClipboardContext<S>, PhantomData<S>)
where
    S: Selection;

impl<S> ClipboardProvider for X11ForkClipboarContext<S>
where
    S: Selection,
{
    fn new() -> Result<Self, Box<dyn StdError>> {
        Ok(Self(X11ClipboardContext::<S>::new()?, PhantomData))
    }

    fn get_contents(&mut self) -> Result<String, Box<dyn StdError>> {
        self.0.get_contents()
    }

    fn set_contents(&mut self, contents: String) -> Result<(), Box<dyn StdError>> {
        match unsafe { fork() } {
            -1 => Err(Error::Fork.into()),
            0 => {
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                ctx.set_contents(contents).unwrap();

                // TODO: wait for clipboard change instead
                std::thread::sleep(std::time::Duration::from_secs(60));
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
