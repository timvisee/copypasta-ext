[![Build status on GitLab CI][gitlab-ci-master-badge]][gitlab-ci-link]
[![Newest release on crates.io][crate-version-badge]][crate-link]
[![Documentation][docs-badge]][docs]
[![Number of downloads on crates.io][crate-download-badge]][crate-link]
[![Project license][crate-license-badge]](#License)

[crate-download-badge]: https://img.shields.io/crates/d/clipboard-ext.svg
[crate-license-badge]: https://img.shields.io/crates/l/clipboard-ext.svg
[crate-link]: https://crates.io/crates/clipboard-ext
[crate-version-badge]: https://img.shields.io/crates/v/clipboard-ext.svg
[docs-badge]: https://docs.rs/clipboard-ext/badge.svg
[docs]: https://docs.rs/clipboard-ext
[gitlab-ci-link]: https://gitlab.com/timvisee/rust-clipboard-ext/pipelines
[gitlab-ci-master-badge]: https://gitlab.com/timvisee/rust-clipboard-ext/badges/master/pipeline.svg

# rust-clipboard-ext
A clipboard library providing useful extensions for the
[`rust-clipboard`][rust-clipboard] library.

I had a growing annoyance with `rust-clipboard`, because the clipboard is
cleared on the Linux/X11 platform when your application exists as per X11
design. The crate maintainer didn't want to implement workarounds (for valid
reasons). This `clipboard-ext` crate provides additional
clipboard contexts that solve this, along with a few other additions.

Here are some of these additions:

- [`X11ForkClipboardProvider`](https://docs.rs/clipboard-ext/*/clipboard_ext/x11_fork/index.html):
  forks process and sets clipboard, keeps contents after exit
- [`X11BinClipboardProvider`](https://docs.rs/clipboard-ext/*/clipboard_ext/x11_bin/index.html):
  invokes `xclip`/`xsel` to set clipboard, keeps contents after exit
- [`Osc52ClipboardContext`](https://docs.rs/clipboard-ext/*/clipboard_ext/osc52/index.html):
  use OSC 52 escape sequence to set clipboard contents
- [`CombinedClipboardProvider`](https://docs.rs/clipboard-ext/*/clipboard_ext/struct.CombinedClipboardContext.html):
  combine two providers, use different for getting/setting clipboard

## Example
Get and set clipboard contents. Keeps contents in X11 clipboard after exit by
forking the process. Falls back to standard clipboard provider on non X11 platforms.
See [`x11_fork`](https://docs.rs/clipboard-ext/*/clipboard_ext/x11_fork/index.html)
module for details.

```rust
use clipboard_ext::prelude::*;
use clipboard_ext::x11_fork::ClipboardContext;

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();
    println!("{:?}", ctx.get_contents());
    ctx.set_contents("some string".into()).unwrap();
}
```

Get and set clipboard contents. Keeps contents in X11 clipboard after exit by
invoking `xclip`/`xsel`. Falls back to standard clipboard provider on non X11
platforms. See [`x11_bin`](https://docs.rs/clipboard-ext/*/clipboard_ext/x11_bin/index.html)
module for details.

```rust
use clipboard_ext::prelude::*;
use clipboard_ext::x11_bin::ClipboardContext;

fn main() {
    let mut ctx = ClipboardContext::new().unwrap();
    println!("{:?}", ctx.get_contents());
    ctx.set_contents("some string".into()).unwrap();
}
```

## Requirements
- Rust 1.40 or above
- Same requirements as [`rust-clipboard`][rust-clipboard-requirements]
- Requirements noted in specific clipboard context modules

## Special thanks
- to `aweinstock314` for building [`rust-clipboard`][rust-clipboard]
- to everyone involved in all crate dependencies used

## License
This project is dual-licensed under the [MIT](./LICENSE.mit) and
[Apache2](./LICENSE.apache2) license.

[rust-clipboard]: https://github.com/aweinstock314/rust-clipboard
[rust-clipboard-requirements]: https://github.com/aweinstock314/rust-clipboard#prerequisites
