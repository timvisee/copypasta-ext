fn main() {
    #[cfg(all(
        feature = "x11-bin",
        unix,
        not(any(target_os = "macos", target_os = "android", target_os = "emscripten"))
    ))]
    {
        // xclip and xsel paths are inserted at compile time
        println!("cargo:rerun-if-env-changed=XCLIP_PATH");
        println!("cargo:rerun-if-env-changed=XSEL_PATH");
    }
}
