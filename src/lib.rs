//! Remora — native Rust/Slint Android client for the existing Remora-pi server.
mod remote;

slint::include_modules!();

fn run_ui(storage: Option<std::path::PathBuf>) -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    remote::attach(&ui, storage);
    ui.run()
}

pub fn main() {
    // Desktop preview: avoid storing any credentials if HOME is unset.
    let storage = std::env::var_os("HOME").map(|home| {
        std::path::PathBuf::from(home).join(".config/remora/session.json")
    });
    run_ui(storage).expect("Remora UI failed");
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: slint::android::AndroidApp) {
    // Android's internal data path is app-private. Never write pairing tokens
    // to external/shared storage.
    let storage = android_app.internal_data_path().map(|p| p.join("remora-session.json"));
    slint::android::init(android_app).expect("Unable to initialise Slint on Android");
    run_ui(storage).expect("Remora UI failed");
}
