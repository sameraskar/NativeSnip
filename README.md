# NativeSnip 0.3: extension-first Windows screenshots

Click a browser toolbar button, use Windows' existing screenshot bar, and save the capture locally. A small Rust helper does the native work and exits after the operation. This version replaces the always-running MVP workflow with **on-demand capture**.

No .NET, account, cloud upload, startup service, browser-page access, or permanent clipboard monitoring. The helper also runs briefly when checking the connection, changing settings, selecting a folder, opening a folder, or exporting diagnostics.

## Downloaded package: start here

### Chromium, Chrome, Edge, Thorium

1. Extract `NativeSnip-chromium-0.3.1.zip` to a permanent folder.
2. Open your browser's extensions page (`chrome://extensions`, `edge://extensions`, or the equivalent), enable **Developer mode**, and choose **Load unpacked**.
3. Select the extracted folder containing manifest.json. Pin NativeSnip to the toolbar.
4. Click its toolbar icon. The setup page appears because the Windows helper is not installed.
5. Click **Save helper installer**. Choose where to save the bundled installer in the Save As dialog. It then reveals the installer in Explorer. Double-click it and finish setup.
6. Return to the extension's setup tab and click **Check connection**, or click the toolbar icon again.
7. Click the toolbar icon to open Windows' capture bar. Select your region/window/full screen, or press Escape to cancel.

The public key in project.json keeps the unpacked Chromium ID stable: `fibdpigmdcoeannimooiajdkkhioijno`. Do not remove or replace it without rebuilding the helper too.

Chrome/Edge registrations and Chromium/Thorium registrations are included. Thorium is intended to work through its Chromium-compatible native messaging lookup; the exact behavior of a specific Thorium build still needs a Windows trial.

### Firefox / Floorp

1. Extract `NativeSnip-firefox-0.3.1.zip`.
2. For this unsigned development build, open `about:debugging#/runtime/this-firefox`, click **Load Temporary Add-on**, and select manifest.json in the extracted folder.
3. Pin NativeSnip and follow the same setup process above.

Temporary add-ons disappear when Firefox restarts. A persistent normal Firefox installation requires Mozilla signing; renaming this unsigned ZIP to XPI does not remove that requirement. The Firefox build targets Firefox 140+ (and compatible derivatives). Its fixed add-on ID is `autosnips@autosnips.local`.

Both packages include the identical Windows x64 installer. The first installation registers the helper for both browser families. You do not need to run it once per browser, but each browser still needs its extension installed.

## What happens when you click

- The extension verifies the installed helper's protocol and exact version.
- It opens one native messaging connection for a capture session.
- The helper registers a temporary Windows clipboard listener, then invokes `ms-screenclip:`.
- The user selects a capture in the existing Windows bar.
- The helper recognizes a likely screenshot, writes one PNG, returns its filename/status, and exits.
- The toolbar badge displays a check mark and its tooltip shows the saved filename.
- Escape cancels saving. The helper's brief keyboard hook only checks Escape during the session; no typed keys or clipboard text are logged. If the hook cannot be installed, use the extension's Cancel saving action or wait for timeout.
- A 90-second timeout ends a session with no accepted image. Timeout is reported separately from cancellation.

Right-click the toolbar icon for Settings, Open screenshots folder, or Cancel current saving session. Browser extension options also opens Settings. A setup tab can show progress; it is not required to remain open for a capture to finish.

**Cancel saving stops the helper's saving session.** It does not forcibly close Windows' capture overlay; press Escape there to dismiss the overlay. Escape anywhere during the active session cancels that session, even if focus has moved to another app. Windows overlays and cancellation behavior still need live testing on the user's Windows build.

## Connection and save location

The default destination is Windows' known Screenshots folder, including redirection such as `D:\Pictures\Screenshots`. Change it in Settings using Browse or an absolute path. Empty folder text restores the Windows default.

The installer sets up a current-user connection under `%LOCALAPPDATA%\Programs\AutoSnipsBrowser`. It adds an entry called **NativeSnip Windows Screenshot Connection** to Windows' installed-app list and a Start Menu uninstall shortcut. No admin elevation should be needed.

If a legacy AutoSnips MVP is running, setup asks that it be stopped as part of installation and removes its known Startup shortcut. Its old executable/data remain. This avoids having the old always-running listener save a second copy of the same image.

The new helper imports legacy `%LOCALAPPDATA%\AutoSnips\settings.json` on first use if available. Its own settings and logs are stored under `%LOCALAPPDATA%\AutoSnipsBrowser`.

To uninstall, remove the connection from Windows Settings → Apps → Installed apps, or use its Start Menu shortcut. Remove the browser extension separately. Screenshots, settings, and diagnostic reports are retained. If uninstallation says the helper is in use, close the native folder picker or cancel the active capture, then retry.

## Privacy and detection limits

The extension has nativeMessaging, downloads, and contextMenus permissions. It has no host permissions, content scripts, tabs permission, or clipboardRead permission. The native helper handles clipboard image access; no screenshot image is sent through browser native messages.

All saving and diagnostics are local. Logs include capture-session metadata, local timestamps, owner basenames, clipboard format names, rules used, errors, and saved paths. They do not intentionally include clipboard text, screenshot pixels, page URLs, or browsing history. Folder paths and custom format names can identify apps or users; review exports before sharing. Logging is on by default for this trial build and can be disabled in Settings. It rotates at roughly 5 MiB with one previous file retained. Exports create separate timestamped directories and do not overwrite earlier reports.

The working MVP trial found Windows reporting captures as owned by svchost.exe. This build preserves the configurable **broker candidate** rule: svchost.exe plus bitmap, PNG, DataObject, and Preferred DropEffect formats. It also recognizes ScreenClippingHost.exe, SnippingTool.exe and ScreenSketch.exe. The rule is heuristic, not a guaranteed Windows screenshot marker. During an active session an unrelated copied image with this signature could be saved. Unidentified-source images are ignored and may include missed captures. The session exits after its first accepted save, eliminating repeated clipboard saves for that session without an ongoing duplicate cache.

Existing clipboard data at session launch is ignored by sequence number. Windows itself supplies the capture interface; this product does not install Microsoft's Snipping Tool app. The default launch simulates Windows’ Win+Shift+S shortcut after the listener is ready. If modifier keys are held or input injection fails, it uses the `ms-screenclip:` protocol instead. Settings also offers protocol-only compatibility mode. No helper remains running between captures.

Image conversion uses Windows GDI and produces 8-bit RGB PNG. HDR, ICC profiles and alpha fidelity are not supported. Decoded BGRA size is capped at 100 MiB. Large images can temporarily use more memory than the decoded BGRA size. Resource use has not been measured on Windows.

The helper and installer are unsigned. SmartScreen or executable-download confirmation may appear. Bundling installers in public extension-store submissions still needs each store's review. These packages are development builds, not approved store releases.

## Build everything from source

Requirements on a Windows build machine:

- Stable Rust with the `x86_64-pc-windows-msvc` target and the Visual Studio C++ build tools required by Rust.
- Python 3.10+.
- NSIS 3, with makensis on PATH or its path in the MAKENSIS environment variable.
- Node.js for JavaScript tests (the extension itself requires no Node installation).

From this source directory:

```powershell
node --test tests/background.test.cjs
$env:MAKENSIS = 'C:\Program Files (x86)\NSIS\makensis.exe'
python scripts/build.py
```

**One build command produces both Chromium and Firefox packages**, the shared helper EXE and its installer, under dist/. It builds/testing Rust first, then NSIS, then creates both manifests and ZIPs from the common extension sources. Cargo.lock is committed; dependency resolution is reproducible. The workflow in `.github/workflows/build.yml` runs this build and uploads both packages plus the installer.

Linux cross-builds are also supported:

```bash
rustup target add x86_64-pc-windows-gnu
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
python3 scripts/build.py --target x86_64-pc-windows-gnu
```

MinGW and Linux NSIS are needed. NSISDIR may need to point to your NSIS data directory if its compiler is unpacked rather than installed.

## Repository layout

- extension/: shared browser scripts, setup UI, local icons; no third-party JS dependencies.
- native/: Rust on-demand helper and Cargo.lock.
- installer/setup.nsi: current-user installer/uninstaller.
- project.json: release version, host name, fixed browser IDs and Chromium public key.
- scripts/build.py: builds helper, installer, and both extension ZIPs.
- tests/background.test.cjs: browser API mock tests of capture/setup/session lifecycle.
- tests/package_test.py: checks generated packages, identities, required permissions, and bundled installer.
- .github/workflows/build.yml: Windows CI build.
- LICENSE: MIT.

A public key is included to preserve development extension identity; no private key, signing certificate, credentials, or user trial data are in the source package. Mozilla signing and store publishing are separate steps, not performed by the build.

## Windows release signing

The default build and supplied release artifacts remain unsigned. The opt-in
`python scripts/build.py --sign` sequence signs and verifies the helper, signs
the embedded uninstaller through NSIS, then signs and verifies setup before
packaging. It requires Windows, the Windows SDK SignTool, an approved signing
identity, and external configuration selected by `NATIVESNIP_SIGN_CONFIG`.
See [SIGNING.md](SIGNING.md) for provider choices, secure setup and browser IDs.

To repackage a completed signed build without rebuilding or re-signing, use
`python scripts/build.py --package-only --sign`. This verifies the existing
helper/setup and their build signing record before generating packages and
checksums. A full build overwrites signed binaries. Keep credentials outside
this repository; never enable release signing on untrusted pull-request jobs.

## Publishing later

Create the GitHub repository using this directory as its root. Then push the code; GitHub Actions should build both packages. This workflow has been included but has not been executed in a newly created GitHub repository during this conversation.

Before a store release, verify bundling rules, submit complete native/helper source as requested, arrange code signing if desired, and complete Windows acceptance tests. Public Chrome/Edge store extension IDs can differ from the development key ID: update project.json's public key/IDs and native origin allowlist together, then rebuild. The helper intentionally allows only the configured extension identity. Do not use wildcard origins or accept an arbitrary browser-supplied installer origin parameter.

Firefox signing should retain the fixed gecko ID, or that ID and the helper's allowed_extensions must be updated together. Self-distributed Firefox packages also require signing for normal release Firefox.

## Initial Windows acceptance test

1. Stop the legacy AutoSnips before testing the extension build.
2. Install the extension but not the helper. Confirm that its button opens setup.
3. Save bundled installer; confirm Explorer selects the file. Run it and check the completion instructions.
4. Check helper connection; choose your save folder.
5. Test rectangle, window, freeform, fullscreen and cancelled captures.
6. Confirm one readable PNG per capture and that the clipboard still pastes normally.
7. Copy ordinary images during normal browser use: no helper should be running, so nothing should save.
8. Test copying an unrelated image while a capture session is active, as a detection edge case.
9. Cancel through Escape and through extension controls. Test a 90-second timeout and old clipboard image at launch.
10. Close/reopen the setup tab during capture. Try multiple toolbar clicks and another browser capturing concurrently.
11. Try a missing/unwritable save folder. Confirm that failure is reported and existing files remain untouched.
12. Reboot: no helper process should remain running until the extension invokes an operation.
13. Uninstall the helper from Windows; confirm screenshots remain and the extension returns to setup.
14. Reinstall, then test the other browser package and the intended Thorium/Floorp builds.

See BUILD-NOTES.md for the actual completed checks and material limitations.

Version 0.3.1 reuses one helper process per toolbar click. Closing an observed capture window ends saving after a 750 ms clipboard grace period. This uses a window lifecycle heuristic, not a Windows cancellation callback, and needs testing across Windows builds. If no capture window is recognized, Escape, Cancel saving and the 90-second timeout remain available.

The release keeps its existing browser IDs, native messaging name, installation directory and data paths for upgrade compatibility. The visible product name is NativeSnip. Launch timing records include browser-to-helper readiness, native preparation/launch request, capture-window appearance, closure and session finish. Windows timing is inferred from the recognized capture window, not a direct operating-system callback.

Export diagnostics includes timings.csv, joining records per session. estimated_click_to_bar_ms adds browser readiness to observed native UI delay; it is a heuristic, not a measured screen-render timestamp. close_to_finish_ms covers closing through saving/cancellation completion. Older events without timing values remain blank.
