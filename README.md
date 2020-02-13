[![Build status on GitLab CI][gitlab-ci-master-badge]][gitlab-ci-link]
[![Newest release on crates.io][crate-version-badge]][crate-link]
[![Documentation][docs-badge]][docs]
[![Number of downloads on crates.io][crate-download-badge]][crate-link]
[![Project license][crate-license-badge]](LICENSE)

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

This library uses [`clipboard`][rust-clipboard] as base, and provides
additional clipboard providers:

- `X11ForkClipboardProvider`: forks process and sets clipboard, keeps contents
  after quit
- `X11BinClipboardProvider`: invokes `xclip`/`xsel` to set clipboard, keeps
  contents after quit

## Special thanks
- to `aweinstock314` for building [`rust-clipboard`][rust-clipboard]
- to everyone involved in all crate dependencies used

## License
This project is dual-licensed under the [MIT](./LICENSE.mit) and
[Apache2](./LICENSE.apache2) license.

[rust-clipboard]: https://github.com/aweinstock314/rust-clipboard
