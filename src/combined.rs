use std::error::Error;

use clipboard::ClipboardProvider;

/// Combined, use different clipboard context for getting and setting.
pub struct CombinedClipboardContext<G, S>(pub G, pub S)
where
    G: ClipboardProvider,
    S: ClipboardProvider;

impl<G, S> ClipboardProvider for CombinedClipboardContext<G, S>
where
    G: ClipboardProvider,
    S: ClipboardProvider,
{
    fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self(G::new()?, S::new()?))
    }

    fn get_contents(&mut self) -> Result<String, Box<dyn Error>> {
        self.0.get_contents()
    }

    fn set_contents(&mut self, contents: String) -> Result<(), Box<dyn Error>> {
        self.1.set_contents(contents)
    }
}
