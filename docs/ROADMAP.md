# Remora roadmap

## Milestone 1 — Native Android UI and build automation

- [x] Rust/Slint Android app source.
- [x] Four navigable starter screens.
- [x] Touch event and Android keyboard input test controls.
- [x] GitHub Actions test APK workflow on `dev`.
- [x] Confirm a successful GitHub Actions ARM64 build and artifact upload.
- [ ] Confirm the app launches correctly on a physical phone.

## Milestone 2 — Connect to the existing Pi control server

- [x] Inspect Pi Remote v0.3.0 source and map the existing pairing, API and WebSocket protocol (see [PI_PROTOCOL.md](PI_PROTOCOL.md)).
- [ ] Keep the existing browser client and Wayland/uinput implementation untouched.
- [ ] Design a reusable Rust transport with error handling and authentication.
- [ ] Persist known host/pairing information securely on Android.
- [ ] Pair once, reconnect on network recovery and report truthful status.

## Milestone 3 — Actual remote control

- [ ] Cursor motion, tap, right click, drag, scrolling and sensitivity.
- [ ] Keyboard input and shortcut keys.
- [ ] Clipboard sync with explicit permission/privacy decisions.
- [ ] Moonlight controls and configurable Pi application launcher.
- [ ] DNS-SD/mDNS discovery or `steamlink.local` fallback.

## Milestone 4 — Release readiness

- [ ] Proper app icon and visual polish.
- [ ] Check on-screen/immersive layout and orientation on OnePlus 12.
- [ ] Set up a persistent signing key securely for release APKs.
- [ ] Add signed tag-triggered GitHub releases on `main`.
- [ ] Review Android local-network permissions as target SDK increases.

## Important

The starter app **does not connect to the Pi**. No touch, text or button events are transmitted. The Pi Remote v0.3.0 protocol is documented in [PI_PROTOCOL.md](PI_PROTOCOL.md). Keep compatibility with the [Remora Pi server](https://github.com/glawks/Remora-pi); never invent routes, commands or pairing tokens.
