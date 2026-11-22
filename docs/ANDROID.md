# Android development

## Continuous integration (recommended)

Push to `dev`, then open **Actions → Android ARM64 - dev APK → latest run → Artifacts → remora-arm64-dev**. The artifact is a ZIP containing an unsigned-for-production, debug-signed ARM64 APK.

The workflow checks out source, installs Rust, Android SDK/NDK and `cargo-apk2`, builds, and uploads the APK. **Do not tag a stable release yet.**

Because the debug certificate is recreated on each ephemeral CI runner, a newer test APK may require uninstalling the older Remora test APK first. This loses any app data; a persistent signing key will be required for update-in-place releases.

## Local Android build

Install Rust stable, Android SDK and NDK, Java/JDK, and `cargo-apk2`:

```bash
rustup target add aarch64-linux-android
cargo install cargo-apk2 --locked
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/27.2.12479018"
cargo apk2 build --target aarch64-linux-android --lib
```

The APK appears under `target/debug/apk/`. To install and start it via USB debugging:

```bash
cargo apk2 run --target aarch64-linux-android --lib
```

For a desktop preview, run `cargo run --bin remora-desktop`. This does not test the Android native activity.

## Technical notes

- Android package: `dev.glawks.remora`
- Android target: `aarch64-linux-android`
- Min SDK: 26; target SDK: 35
- UI: Slint, Android NativeActivity
- Network permissions are declared in anticipation of the Pi transport; network functionality is not yet implemented.
