# Changelog

## NativeSnip 0.3.1 - 5 October 2026
Final packaging release. Fix settings capture-button state during active capture; add two UI regression tests. Add quick start, privacy, release notes, contributor guidance and automated release bundle/checksums. Preserve the user-tested 0.3.0 capture engine and upgrade-compatible IDs/data paths.

## NativeSnip 0.3.0 - 5 October 2026
Rename visible extension and installer to NativeSnip. Launch via native Win+Shift+S with protocol fallback and selectable compatibility mode. Use a lightweight hello handshake; remove pre-launch folder write probes and diagnostic writes. Shorten cancellation grace to 750 ms, preserving busy-clipboard handling. Add timing CSV exports, immediate opening feedback, saved/cancelled badges and rapid double-click protection. The owner reported the release polished and ready to ship; no precise updated timings were provided.

## AutoSnips 0.2.1 - 4 October 2026
Reuse one helper process for readiness and capture. Observe the Windows capture window closing and apply a two-second clipboard grace period. Preserve Escape, explicit cancel and 90-second timeout fallbacks. User confirmed saving and close-button cancellation worked, with remaining launch/dismissal delays.

## AutoSnips 0.2.0 - 4 October 2026
Introduce extension-first on-demand capture for Windows x64: shared Chromium/Firefox source, toolbar capture, bundled current-user helper setup/uninstaller, local PNG saving, folder picker, settings and diagnostic export. Preserve the observed clipboard broker candidate rule. No startup listener. Follow-up repairs saved installer bytes through a data URL and enforced an executable filename using Chromium’s final naming stage and Save As.

## Separate native MVP
0.1.0: event-driven listener and observation data; process-name allowlist rejected captures on the owner’s PC.
0.1.1: add observed svchost-plus-format heuristic, two-second duplicate suppression and folder helper. User confirmed saves. Overnight report showed three saves, seven suppressed updates, fourteen ignored non-image events and six rejected image events. No ground-truth labels were supplied; detector decisions did not establish accuracy.

All builds remain unsigned and unapproved by browser stores. Automated checks are distinguished from user Windows trials in BUILD-NOTES.md.
