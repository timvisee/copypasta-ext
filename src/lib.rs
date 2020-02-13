#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
pub mod x11_fork;

// Re-export clipboard
pub use clipboard;

// Re-export
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
))]
pub use x11_fork::X11ForkClipboarContext;
