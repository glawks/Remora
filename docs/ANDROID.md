# Android development

## Continuous integration (recommended)

Push to `dev`, then open **Actions → Android ARM64 - dev APK → latest run → Artifacts → remora-arm64-dev**. The artifact is a ZIP containing an unsigned-for-production, debug-signed ARM64 APK.

The workflow checks out source, installs Rust, Android SDK/NDK and `cargo-apk2`, builds, and uploads the APK. **Do not tag a stable release yet.**

Because the debug certificate is recreated on each ephemeral CI runner, a newer test APK may require uninstalling the older Remora test APK first. This loses any app data; a persistent signing key will be required for update-in-place releases.

## Local Android build

Install Rust stable, Android SDK and **NDK r30** (required by Slint 1.19 / Skia 0.157), Java/JDK, and `cargo-apk2`:

```bash
rustup target add aarch64-linux-android
cargo install cargo-apk2 --locked
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk/30.0.16248370"
cargo apk2 build --target aarch64-linux-android --lib
```

The APK appears under `target/debug/apk/`. Use the **signed `Remora.apk`**, not `Remora-unaligned.apk`. See [troubleshooting](TROUBLESHOOTING.md) if Android installs the app but it immediately closes. To install and start it via USB debugging:

```bash
cargo apk2 run --target aarch64-linux-android --lib
```

For a desktop preview, run `cargo run --bin remora-desktop`. This does not test the Android native activity.

## Technical notes

- Android package: `dev.glawks.remora`
- Android target: `aarch64-linux-android`
- Min SDK: 26; target SDK: 35
- UI: Slint, Android NativeActivity
- Initial pairing, reconnect, touchpad and keyboard WebSocket controls are implemented on `dev`; real-device testing is still required.
- App-private paired credentials are not yet Android Keystore-encrypted. Never expose the Pi server's HTTP port to the internet.
