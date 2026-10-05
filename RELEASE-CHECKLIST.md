# NativeSnip release checks

## Required Windows trials
- Upgrade from AutoSnips 0.2.1: settings and chosen folder remain; extension ID is unchanged; Installed apps shows NativeSnip once.
- Test Windows restart and browser restart, then first capture and repeated captures. Export diagnostics after several trials.
- Test region, window, full screen, high-resolution and multiple monitors; ensure no lost or duplicate saves.
- Test Escape, close button and explicit Cancel saving. Check cancellation does not discard a real capture with delayed clipboard delivery.
- Test with modifier keys held; compare shortcut and protocol compatibility modes.
- Test helper missing/outdated, unwritable folder, occupied clipboard and a 90-second timeout.
- Uninstall while idle and during a capture, reinstall, and confirm user screenshots/settings survive.
- Test Chromium, Thorium, Edge and Firefox on Windows 10/11 before advertising each as supported.

## Distribution
- Review licensing and name availability before public branding. Search results alone are not trademark clearance.
- Choose final store browser IDs, rebuild helper allowlists together, test the resulting packages.
- Submit Firefox for signing and both stores for review; validate each store’s policy on bundled Windows installers.
- Prepare accurate privacy policy, screenshots and support instructions. Diagnostics are local, optional and user-exported.
- Consider Windows code signing when budget permits; do not claim unsigned builds are trusted by Windows.

## Timing interpretation
launch_requested.browser_ready_ms covers click-to-native-readiness observed by the browser. Native elapsed_ms starts at the capture request. capture_window_observed is a heuristic for visible UI appearance. Compare capture_window_closed and finished elapsed_ms for dismissal delay. Diagnostic records do not prove screenshot detection accuracy. Keep a separate list of missed or unexpected captures.
