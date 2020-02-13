mod combined;
#[cfg(all(
    feature = "x11-bin",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
pub mod x11_bin;
#[cfg(all(
    feature = "x11-fork",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
pub mod x11_fork;

// Expose platform specific contexts
#[cfg(not(all(
    feature = "x11-bin",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
)))]
pub mod x11_bin {
    pub type ClipboardContext = clipboard::ClipboardContext;
}
#[cfg(not(all(
    feature = "x11-fork",
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
)))]
pub mod x11_fork {
    pub type ClipboardContext = clipboard::ClipboardContext;
}

// Re-export
pub use clipboard;
pub use combined::CombinedClipboardContext;
