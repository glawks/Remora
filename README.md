# Remora

**Remora** is a native Android remote-control app for Raspberry Pi and Linux, built with **Rust** and **Slint**. Its server is [Remora-pi](https://github.com/glawks/Remora-pi).

## Development status: 0.2.0-dev

The `dev` branch contains the first remote-control implementation:

- Pair to Remora-pi using a one-time eight-digit code.
- Save the paired Pi in Android private storage and reconnect automatically.
- Authenticate through the server's existing WebSocket with the required Origin header.
- Swipe to move, tap to left-click, use left/right click and scroll buttons, and hold/release drag.
- Use Android keyboard input to send ASCII or paste Unicode through the Pi clipboard.
- Send navigation keys and launch the server-approved Moonlight application.

**Phone testing and a green Android build for this change are still required.** The implementation does not yet include multi-touch gestures, QR scanning, full app catalogue, or Keystore-encrypted token storage.

The existing Pi Python server and web controller remain unchanged. Connect only on a trusted network; the existing LAN HTTP protocol is not encrypted.

## Branches

- `main` — reserved for stable releases.
- [`dev`](../../tree/dev) — active Android development and test APKs.

## Get an Android test APK

Go to [GitHub Actions](../../actions/workflows/android-dev.yml). After a successful `dev` run, download and extract the `remora-arm64-dev` artifact and install the APK.

If no run starts automatically after the update, click **Run workflow** and select the `dev` branch. **Do not install an older 0.1.0 APK expecting these new controls.**

See [docs/CONTROL.md](docs/CONTROL.md) for pairing and testing, [docs/ANDROID.md](docs/ANDROID.md) for build instructions, and [docs/PI_PROTOCOL.md](docs/PI_PROTOCOL.md) for the exact Pi API.

## License

To be selected before a stable release.
