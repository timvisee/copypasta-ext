use copypasta::ClipboardProvider;

/// Combined, use different clipboard context for getting & setting.
///
/// Useful to combine different clipboard contexts to get the best of both worlds.
///
/// This may be constructed using helpers such as
/// [`X11BinClipboardContext::new_with_x11`][new_with_x11] or
/// [`X11BinClipboardContext::with_x11`][with_x11].
///
/// [new_with_x11]: ../copypasta_ext/x11_bin/struct.X11BinClipboardContext.html#method.new_with_x11
/// [with_x11]: ../copypasta_ext/x11_bin/struct.X11BinClipboardContext.html#method.with_x11
pub struct CombinedClipboardContext<G, S>(pub G, pub S)
where
    G: ClipboardProvider,
    S: ClipboardProvider;

// impl<G, S> CombinedClipboardContext<G, S>
// where
//     G: ClipboardProvider,
//     S: ClipboardProvider,
// {
//     pub fn new() -> Result<Self, Box<dyn Error>> {
//         Ok(Self(G::new()?, S::new()?))
//     }
// }

impl<G, S> ClipboardProvider for CombinedClipboardContext<G, S>
where
    G: ClipboardProvider,
    S: ClipboardProvider,
{
    fn get_contents(&mut self) -> crate::ClipResult<String> {
        self.0.get_contents()
    }

    fn set_contents(&mut self, contents: String) -> crate::ClipResult<()> {
        self.1.set_contents(contents)
    }
}
