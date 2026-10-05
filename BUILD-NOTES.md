# NativeSnip 0.3.1 verification

Built on 5 October 2026 on Linux with Rust, Windows x64 GNU target and NSIS.

Automated validation: native policy/protocol/PNG tests, shortcut key balancing, cancellation grace/busy/reappearance tests; browser API mocks; both package integrity checks; Rust formatting and JavaScript syntax; Windows native release build and NSIS setup/uninstall compilation.

User trial of 0.2.1 confirmed installation, correctly named helper download, screenshot saving and close-button cancellation. User measured launch around 2 seconds, up to 5 seconds on first use, and cancellation around 3–5 seconds. Version 0.3.1 changes the launch to Win+Shift+S, removes readiness settings/COM work and pre-launch folder write probes, and shortens the cancellation grace to 750 ms. These timings are NOT measured improvements yet.

Windows runtime testing for 0.3.1 is pending. This Linux environment cannot validate Windows capture, input injection, timing, reinstall/uninstall behavior or actual browser/native messaging. The previous Chromium launch attempt was blocked by environment socket restrictions. No Wine result is substituted for Windows testing.

Installer remains unsigned. Firefox package remains an unsigned temporary developer add-on. No store publication, store approval or trademark clearance is claimed. Internal identifiers and paths remain compatible with AutoSnips 0.2.x.

Final review on 5 October: the owner accepted 0.3.0 as polished. Version 0.3.1 retains its capture engine and adds one settings-button UI fix plus release documentation. Final automated suite contains 18 Rust tests, 13 browser/UI tests and 2 package checks, 33 total. No local Windows runtime testing is claimed.
