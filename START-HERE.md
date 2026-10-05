# NativeSnip 0.3.1: start here

Windows 64-bit. This package contains browser extensions, a Windows helper installer and MIT-licensed source.

## Chrome, Chromium, Edge or Thorium
1. Extract NativeSnip-chromium-0.3.1.zip to a permanent folder.
2. Open the browser’s extensions page, enable Developer mode, choose Load unpacked and select the folder containing manifest.json.
3. Pin NativeSnip. Click its toolbar button.
4. Save the bundled helper installer and run it manually. Alternatively run the separate NativeSnip-Helper-Setup-0.3.1.exe supplied here.
5. Finish installation and click Check connection or the toolbar button again.
6. Use Windows’ screenshot bar. Select a capture or dismiss the bar to cancel. Captures save to the chosen Screenshots folder.

## Firefox
Extract NativeSnip-firefox-0.3.1.zip. Open about:debugging#/runtime/this-firefox, choose Load Temporary Add-on and select manifest.json. Follow the helper setup above. This unsigned package is temporary and must be loaded again after Firefox restarts. Normal persistent installation requires Mozilla signing.

## Updating AutoSnips or NativeSnip
Replace the old extracted extension files, reload the extension, and run the matching helper installer. IDs and settings paths are preserved. Stop any old standalone AutoSnips listener to avoid duplicate saves. The extension and helper must have matching versions.

## Settings and removal
Right-click the toolbar button for Settings or Open screenshots folder. Settings offers folder selection, local diagnostics, export and protocol compatibility mode if the default Windows shortcut fails.
Remove the helper through Windows Settings > Apps > Installed apps > NativeSnip Windows Screenshot Connection. Remove the browser extension separately. Screenshots and local settings/reports are retained.

## Source and validation
Extract NativeSnip-Source-0.3.1.zip. Read README.md for build dependencies, then run python scripts/build.py from the NativeSnip source directory. It builds both browser packages and the Windows installer. No private keys or user trial data are included.

The installer is unsigned. This release is ready for manual distribution and further Windows trials; it is not a signed store release. See RELEASE-NOTES.md and PRIVACY.md.
