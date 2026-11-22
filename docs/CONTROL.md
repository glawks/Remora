# Remora Android 0.2.0-dev — Control integration

The Android client now uses the **unchanged** [Remora-pi server](https://github.com/glawks/Remora-pi).

## Pair and test

1. Confirm the existing Pi Remote/Remora-pi service is running on Raspberry Pi Wayland/labwc, reachable at `http://steamlink.local:3100` (or your private IP).
2. On the Pi, generate a one-time pairing code: `python3 ~/.local/share/pi-remote/server.py pair`. This command only works if your server is installed at that path; alternatively use your existing QR pairing helper.
3. Open Remora Android → Setup, enter the Pi host and the eight-digit code, then tap **Pair this Android device**.
4. Confirm the status says **Connected**. Use Touchpad: swipe to move, tap to click, or use explicit left/right click, scroll and hold/release drag buttons.
5. In Keys, send ASCII or paste text as Unicode using the Pi clipboard. Special navigation keys are also available.
6. In Apps, launch Moonlight if the server has approved and detected it.

The app stores an app-private token in internal storage (0600 on Unix) for subsequent reconnection; tokens are not logged or sent in URLs. **This is not yet Android Keystore-backed encryption**. Keep Android backups disabled for this app until encrypted storage is implemented.

## Architecture and limits

- `POST /api/pair` → saved device token; `POST /api/verify` → check current pairing
- `ws://HOST:3100/ws` (or `wss` over HTTPS) with a matching `Origin` header and first message `{"type":"auth","token":"..."}`
- WebSocket commands are sent only after `{"type":"ready"}`. On disconnect, held drag is released by server and stale Android commands are dropped.
- App ping/pong heartbeat + automatic reconnect (bounded backoff), no replay of offline events.
- UI initial version supports **single finger** motion/tap only. True Android multitouch scroll/gesture support needs separate testing. Scroll buttons are provided.
- Text entry is explicit rather than keystroke mirroring; ASCII uses the Pi's US layout, Unicode paste uses `clipboard-set`.
- Moonlight uses the approved server `moonlight` launcher ID, not arbitrary shell execution.
- Pi discovery, QR scanning, complete application catalogue and Android Keystore encryption are not yet implemented.
- HTTP is unencrypted by design on the existing trusted LAN. Never expose port 3100 to the public internet. For remote use, configure a secure trusted HTTPS reverse proxy or private network.

## Build status

Push to `dev` to trigger GitHub Actions ARM64 APK upload. Verify the latest workflow succeeded before installing. Test on an actual phone before merging to `main`.
