# NativeSnip 0.3.1

Final packaging release, 5 October 2026.

NativeSnip links a browser toolbar button to Windows’ native screenshot bar, saves one PNG locally and returns to idle after completion or cancellation. It supports Chromium and Firefox extension packages with a small user-level Windows x64 helper and uninstaller.

The capture engine is unchanged from the user-tested 0.3.0 build. Default launch uses Win+Shift+S, with protocol compatibility mode. The internal cancellation grace period is 750 ms. The helper runs on demand, with no startup listener. Local timing reports help diagnose remaining delays.

Final review corrected the settings-page screenshot button so it remains disabled while a capture is active. Added privacy and quick-start documentation, release checksums and a complete distribution bundle. The build command now runs the browser/UI tests as well as native tests and package checks.

User acceptance: the project owner reported 0.3.0 polished and ready to ship. Earlier trials confirmed helper download/install, saving and cancellation. No precise post-0.3.0 timing results were supplied, so no measured speed claim is made.

Validation: automated native, browser/UI and package tests; JavaScript syntax; Rust formatting; Windows x64 cross-build; NSIS setup/uninstall compilation; ZIP integrity and identical bundled helper checks. Windows runtime verification remains based on the owner’s trial, not local testing in the Linux build environment.

Distribution limits: Windows x64 only; unsigned installer; Firefox temporary developer loading until Mozilla signing; Chromium developer loading until store distribution. No store approval, code signing, trademark clearance or broad Windows/browser compatibility certification is claimed.

Known limits: capture-owner and window-dismissal detection are heuristics. A matching unrelated clipboard image during an active capture could be saved; unrecognized capture sources may be missed. If no window is recognized, Escape, explicit Cancel saving and the 90-second timeout remain fallbacks. Delayed clipboard delivery beyond the shortened grace period needs further trial coverage. HDR/ICC preservation is not implemented.

Read RELEASE-CHECKLIST.md for wider Windows trials and store publication work. Source is MIT-licensed with dependency license notices. Browser IDs and internal AutoSnips paths remain stable for upgrades.
