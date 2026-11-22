//! Remora application entry points.
//!
//! The Android UI is deliberately independent of the Pi transport during the
//! first milestone. No simulated pairing or remote-control commands are sent.

slint::include_modules!();

pub fn run_ui() -> Result<(), slint::PlatformError> {
    let app = MainWindow::new()?;
    app.run()
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: slint::android::AndroidApp) {
    slint::android::init(android_app).expect("Unable to initialise Slint for Android");
    run_ui().expect("Remora UI failed");
}
