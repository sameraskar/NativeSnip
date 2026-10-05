# NativeSnip privacy

Applies to NativeSnip 0.3.1, the browser extension and its Windows helper.

NativeSnip opens Windows’ built-in screenshot interface and saves the completed image to the folder you choose. Images remain on your computer. Choosing a cloud-synced folder may cause that folder’s own software to upload them; NativeSnip does not upload them.

The extension does not read website content, browsing history or clipboard text. It requests nativeMessaging to communicate with its helper, downloads to save the bundled installer, and contextMenus for toolbar commands. It does not request access to websites.

During a capture session, the helper observes clipboard updates and recognized Windows capture windows. It reads accepted clipboard image data to save a PNG. A brief keyboard hook recognizes Escape to cancel; it does not record typed text. The helper exits after a save, cancellation, timeout or settings operation. No startup task or permanent listener is installed.

Local diagnostics are enabled by default and can be turned off in Settings. They may contain timestamps, timing measurements, capture decisions, clipboard format names, owner process names, folder paths, image dimensions and error messages. Diagnostic logs contain no screenshot pixels or clipboard text. Exporting diagnostics creates local files and opens their folder; nothing is automatically sent. Review exported files before sharing them.

Settings and logs use the existing AutoSnipsBrowser data directory for upgrade compatibility. Logs roll over around 5 MiB, retaining the current and previous file. Exported reports remain until you delete them. Turning diagnostics off stops new entries but does not delete previous records. Uninstalling preserves screenshots, settings and diagnostic data.

There are no accounts, advertisements, analytics services or automatic remote telemetry in this release. The helper installer is included in the extension package. Browser/store updates and Windows security checks are handled by their respective providers.

For distribution, the publisher should provide a support contact in the repository or store listing. No external contact service is configured in this source release.
