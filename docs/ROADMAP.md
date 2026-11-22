# Remora roadmap

## Milestone 1 — Native Android UI and build automation

- [x] Rust/Slint Android app source.
- [x] Four navigable starter screens.
- [x] Touch event and Android keyboard input test controls.
- [x] GitHub Actions test APK workflow on `dev`.
- [ ] Confirm a successful GitHub Actions build and launch on a physical phone.

## Milestone 2 — Connect to the existing Pi control server

- [ ] Obtain/inspect the current Pi server implementation and documented port/protocol.
- [ ] Keep the existing browser client and Wayland/uinput implementation untouched.
- [ ] Design a reusable Rust transport with error handling and authentication.
- [ ] Persist known host/pairing information securely on Android.
- [ ] Pair once, reconnect on network recovery and report truthful status.

## Milestone 3 — Actual remote control

- [ ] Cursor motion, tap, right click, drag, scrolling and sensitivity.
- [ ] Keyboard input and shortcut keys.
- [ ] Clipboard sync with explicit permission/privacy decisions.
- [ ] Moonlight controls and configurable Pi application launcher.
- [ ] DNS-SD/mDNS discovery or `streamlink.local` fallback.

## Milestone 4 — Release readiness

- [ ] Proper app icon and visual polish.
- [ ] Check on-screen/immersive layout and orientation on OnePlus 12.
- [ ] Set up a persistent signing key securely for release APKs.
- [ ] Add signed tag-triggered GitHub releases on `main`.
- [ ] Review Android local-network permissions as target SDK increases.

## Important

The starter app **does not connect to the Pi**. No touch, text or button events are transmitted. The Pi service must be examined before implementing its protocol; do not guess its routes, commands or pairing tokens.
