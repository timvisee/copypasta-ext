[![Build status on GitLab CI][gitlab-ci-master-badge]][gitlab-ci-link]
[![Newest release on crates.io][crate-version-badge]][crate-link]
[![Documentation][docs-badge]][docs]
[![Number of downloads on crates.io][crate-download-badge]][crate-link]
[![Project license][crate-license-badge]](#License)

[crate-download-badge]: https://img.shields.io/crates/d/copypasta-ext.svg
[crate-license-badge]: https://img.shields.io/crates/l/copypasta-ext.svg
[crate-link]: https://crates.io/crates/copypasta-ext
[crate-version-badge]: https://img.shields.io/crates/v/copypasta-ext.svg
[docs-badge]: https://docs.rs/copypasta-ext/badge.svg
[docs]: https://docs.rs/copypasta-ext
[gitlab-ci-link]: https://gitlab.com/timvisee/copypasta-ext/pipelines
[gitlab-ci-master-badge]: https://gitlab.com/timvisee/copypasta-ext/badges/master/pipeline.svg

# copypasta-ext
A clipboard library providing useful extensions for the
[`copypasta`][copypasta] library.

I had a growing annoyance with `copypasta`, because the clipboard is
cleared on the Linux/X11 platform when your application exits as per X11
design. The crate maintainer didn't want to implement workarounds (for valid
reasons). This `copypasta-ext` crate provides additional
clipboard contexts that solve this, along with a few other additions.

Here are some of these additions:

- [`X11ForkClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_fork/index.html):
  forks process and sets clipboard, keeps contents after exit
- [`X11BinClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_bin/index.html):
  invokes `xclip`/`xsel` to set clipboard, keeps contents after exit
- [`Osc52ClipboardContext`](https://docs.rs/copypasta-ext/*/copypasta_ext/osc52/index.html):
  use OSC 52 escape sequence to set clipboard contents
- [`CombinedClipboardProvider`](https://docs.rs/copypasta-ext/*/copypasta_ext/struct.CombinedClipboardContext.html):
  combine two providers, use different for getting/setting clipboard

To guess at runtime what clipboard provider is best used see the [`DisplayServer`](https://docs.rs/copypasta-ext/*/copypasta_ext/display/enum.DisplayServer.html) class.
Enable all desired compiler feature flags for clipboard systems to support, and
use `DisplayServer::select().try_context()` to obtain a clipboard context.

This crate should work with the latest [`copypasta`][copypasta]. Feel free to
open an issue or pull request otherwise. The `copypasta` crate is exposed as
`copypasta_ext::copypasta`.

## Example
Get and set clipboard contents. Keeps contents in X11 clipboard after exit by
forking the process. Falls back to standard clipboard provider on non X11 platforms.
See [`x11_fork`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_fork/index.html)
module for details.

```rust
use copypasta_ext::prelude::*;
use copypasta_ext::x11_fork::ClipboardContext;

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();
    println!("{:?}", ctx.get_contents());
    ctx.set_contents("some string".into()).unwrap();
}
```

Get and set clipboard contents. Keeps contents in X11 clipboard after exit by
invoking `xclip`/`xsel`. Falls back to standard clipboard provider on non X11
platforms. See [`x11_bin`](https://docs.rs/copypasta-ext/*/copypasta_ext/x11_bin/index.html)
module for details.

```rust
use copypasta_ext::prelude::*;
use copypasta_ext::x11_bin::ClipboardContext;

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();
    println!("{:?}", ctx.get_contents());
    ctx.set_contents("some string".into()).unwrap();
}
```

## Requirements
- Rust 1.57 or above (MSRV)
- Same requirements as [`copypasta`][copypasta]
- Requirements noted in specific clipboard context modules

## Special thanks
- to the maintainers/contributors of [`rust-clipboard`][rust-clipboard] and [`copypasta`][copypasta]
- to everyone involved in all crate dependencies used

## License
This project is dual-licensed under the [MIT](./LICENSE.mit) and
[Apache2](./LICENSE.apache2) license.

[copypasta]: https://github.com/alacritty/copypasta
[rust-clipboard]: https://github.com/aweinstock314/rust-clipboard
