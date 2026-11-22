# Pi Remote v0.3.0 compatibility contract

Based on the user-provided `pi-remote-v0.3.0` source archive (`server.py`, `static/app.js`, README, CHANGELOG). This is an **integration reference**, not a reimplementation of the server.

## Existing Pi server

- User systemd unit: `pi-remote.service`, running the Python `aiohttp` server.
- Normal LAN endpoint: `http://steamlink.local:3100` (substitute the Pi's LAN IP if mDNS fails).
- WebSocket: `ws://steamlink.local:3100/ws`; HTTPS deployments use `wss://.../ws`.
- Wayland/labwc + `/dev/uinput` remain server-side.
- **Do not replace, reinstall, or migrate the Pi server** just to add the native Android client.

## Pairing and token lifecycle

1. Pi generates an eight-digit, one-use pairing code, valid for ten minutes. Can run `python3 ~/.local/share/pi-remote/server.py pair`, or the Pi's pairing QR helper.
2. Android: `POST /api/pair` with JSON `{"code":"12345678","name":"Remora Android"}`. Receive a random `token`.
3. Before connecting: `POST /api/verify` with JSON `{"token":"..."}`; HTTP 401 means revoked/not paired.
4. Open `/ws` and immediately send `{"type":"auth","token":"..."}`; expect `{"type":"ready"}` or `{"type":"denied"}`.
5. Never put the token in logs, URLs, unencrypted backups, or app icon URLs. Store it in Android-protected app storage, preferably with a Keystore-backed mechanism.
6. Tokens do not expire automatically; users can revoke devices. The native app cannot read the existing Chrome localStorage token, so it will need **one new pairing** without affecting existing browser pairing.

## Important server security contract

`check_same_origin` checks Host/Origin and the WebSocket explicitly rejects clients with a missing Origin header. A native Rust WebSocket client must set an `Origin` header **matching its chosen Pi base URL exactly**, e.g. `Origin: http://steamlink.local:3100`; do not weaken or remove the server's origin check. For Tailscale/HTTPS configurations use that deployment's actual origin. Keep LAN HTTP restricted to trusted networks, never internet-expose port 3100.

## WebSocket request examples

All of these are sent **after** authenticated `ready`:

| Function | JSON |
|---|---|
| Keep-alive | `{"type":"ping"}` → `{"type":"pong"}` |
| Move | `{"type":"move","dx":12,"dy":-4}` |
| Scroll | `{"type":"scroll","amount":-2}` |
| Click | `{"type":"click","button":"left"}` |
| Drag start/stop | `{"type":"drag","down":true}` / `false` |
| Key | `{"type":"key","name":"enter"}` |
| Key combo | `{"type":"combo","name":"v","modifiers":["ctrl"]}` |
| Type ASCII text | `{"type":"text","value":"hello"}` |
| Clipboard set | `{"type":"clipboard-set","value":"hello","action":"paste"}` |
| Clipboard get | `{"type":"clipboard-get"}` |
| Pi status | `{"type":"system-status"}` → `system-info` |
| Available apps | `{"type":"apps-list"}` → `apps` |
| Launch app | `{"type":"app-launch","id":"moonlight"}` → `app-status` |
| Prepare power action | `{"type":"power-prepare","action":"reboot"}` → `power-confirm` |
| Execute power action | `{"type":"power-execute","action":"reboot","nonce":"..."}` → `power-status` |

Application icons: `POST /api/app-icon` with JSON `{"token":"...","id":"server-issued-app-id"}`, response PNG bytes.

Validate commands against the server implementation, rather than inventing app IDs or key names. Movement is limited by the server to ±250 per axis, scroll to integer ±8 per message, and type-text to 200 characters per message. Power operations are disabled server-side unless explicitly enabled and require a fresh one-use confirmation nonce.

## Reconnection and gesture behaviour

- Do not buffer input events for replay across reconnect.
- WebSocket heartbeat and ping/pong; exponential retry. Show disconnected / connecting / paired / connected status truthfully.
- Server releases a held left mouse button on socket disconnect.
- Mirror the existing browser gestures and sensitivity settings; maintain phone-specific preferences locally rather than changing Pi credentials.
- Moonlight navigation uses Linux key events, not the Moonlight streaming API.
- `app-launch` accepts only IDs from the server's installed-app catalogue, not raw commands or executable paths.

## Implementation milestones

1. Native pairing and secure persisted credentials.
2. Authenticated socket, heartbeat, reconnect and verified status.
3. Real touchpad, keyboard and clipboard.
4. Pi dashboard, approved app list/launch and Moonlight mode.
5. Power confirmation UI, only after explicit opt-in.
