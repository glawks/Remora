# Remora

A native Android remote-control client for Raspberry Pi/Linux, built with **Rust** and **Slint**.

The planned app provides touchpad, keyboard, clipboard, application launching, and Moonlight-friendly controls. It will reuse the existing Pi/Wayland service rather than replacing it. The Pi backend is maintained separately in [glawks/Remora-pi](https://github.com/glawks/Remora-pi).

## Current status — 0.1.0-dev

This branch contains the **UI-only Android prototype**. It shows four navigable screens and checks that Android touch and keyboard input reach the UI. **It does not connect to the Pi or send control commands yet.**

## Development and builds

- `main`: stable releases (none yet).
- `dev`: source and test APKs; pushes trigger the [Android ARM64 workflow](./.github/workflows/android-dev.yml).
- Test APK: GitHub **Actions → Android ARM64 - dev APK → latest successful run → Artifacts → remora-arm64-dev**.
- Build instructions: [docs/ANDROID.md](./docs/ANDROID.md).
- Milestones and unfinished work: [docs/ROADMAP.md](./docs/ROADMAP.md).

The Pi-side v0.3.0 protocol has been inspected and documented in [docs/PI_PROTOCOL.md](docs/PI_PROTOCOL.md). The native transport is not yet implemented. The current web app should remain available as a fallback.

## License

Not selected yet. Decide on an open-source license before public source distribution or contributions.
