# Android startup troubleshooting

## Instant close after installing

Remora 0.2.0-dev initially built with Android NDK r27c (`27.2.12479018`). Slint 1.19 uses Skia 0.157, whose Android prebuilt binaries were linked against **NDK r30**'s C++ runtime. The APK can compile, align, sign, and install, yet fail during `NativeActivity.onCreate` with a missing `__ndk1` C++ symbol.

- Upstream: https://github.com/slint-ui/slint/issues/14824
- Official Slint Android documentation: https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/android/
- Fix: CI uses Android NDK **r30 (`30.0.16248370`)**; its Rust cache is keyed separately so no r27 binaries are reused.
- Native startup messages are also sent to Android logcat using the `Remora` tag.

The upstream issue is a *likely* explanation of this symptom, not confirmation for a particular phone. Confirm from device logs if needed.

## Testing a new APK

CI debug signing keys may change between runs. On the phone, uninstall the older Remora debug APK before installing a new debug build if Android reports a signature mismatch. This removes saved pairing details, so another pairing code may be required.

Download `remora-arm64-dev` from the newest successful run on `dev` and extract its `Remora.apk`. Do not install the older artifacts.

## Collect a useful crash report with Windows PowerShell

With Android Platform Tools installed and **USB debugging** enabled on the phone:

```powershell
adb devices
adb logcat -c
adb shell am force-stop dev.glawks.remora
adb shell am start -n dev.glawks.remora/android.app.NativeActivity
Start-Sleep -Seconds 3
adb logcat -d -b main -b crash -v time | Select-String -Pattern 'Remora|AndroidRuntime|UnsatisfiedLinkError|dlopen|FATAL EXCEPTION|RustStdoutStderr|libremora|Fatal signal|__ndk1' -Context 5,12 | Out-File -Encoding utf8 remora-crash.txt
```

Provide `remora-crash.txt` for diagnosis. Review the text before sharing because other apps' logs can be captured. The app does not intentionally log pairing codes, tokens, or keystrokes.

A `dlopen failed: cannot locate symbol ... __ndk1` log confirms the NDK/Skia mismatch. A `SecurityException` mentioning dynamic class loading, renderer errors, or another Rust panic points to a different issue.

## Notes

Passing `apksigner verify`, `zipalign`, and `unzip -t` verifies package integrity, **not** that its native code launches. An Android emulator smoke test or physical phone test is still needed before merging this branch to `main`.
