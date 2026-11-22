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
    // Capture early startup failures so they can be diagnosed via ADB logcat.
    // Do not log pairing tokens, typed text, or the contents of network messages.
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("Remora"),
    );
    std::panic::set_hook(Box::new(|panic| {
        log::error!("Native panic: {panic}");
    }));
    log::info!("Entering Remora native activity");

    // Android's internal data path is app-private.
    let storage = android_app.internal_data_path().map(|p| p.join("remora-session.json"));
    log::info!("App data path initialised");

    if let Err(error) = slint::android::init(android_app) {
        log::error!("Slint backend failed to initialise: {error}");
        return;
    }
    log::info!("Slint backend initialised");

    if let Err(error) = run_ui(storage) {
        log::error!("Remora UI failed: {error}");
    } else {
        log::info!("Remora UI event loop ended");
    }
}
