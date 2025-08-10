fn main() {
    // Avoid pulling GUI deps (glib/gdk) during headless CI/tests unless explicitly enabled
    #[cfg(feature = "tauri_build")]
    {
        tauri_build::build();
    }
}
