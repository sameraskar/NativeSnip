# Contributing to NativeSnip

Read README.md and RELEASE-CHECKLIST.md first. Keep the core experience small: native Windows capture, local saving and clear cancellation. Preserve browser IDs and native host registration compatibility unless a migration is intentional.

Use python scripts/build.py with the documented Windows build dependencies. This runs browser/UI and Rust tests and creates both extension packages plus the helper installer. Do not commit dist/, native/target/, signing keys, screenshots or diagnostic files.

For bug reports, provide Windows version, browser/version, NativeSnip/helper version, capture mode, launch method and reproducible steps. Local diagnostics are optional; review them for personal paths before sharing. Keep missed or unexpected screenshots separate from detector decision counts.

For pull requests, describe the observed problem, resulting behavior and relevant validation. Changes to capture or dismissal need a real Windows trial in addition to automated checks. Source contributions use the repository’s MIT license; include notices for any new dependency.
