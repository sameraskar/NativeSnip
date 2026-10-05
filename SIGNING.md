# Signing and publishing NativeSnip

This guide is for an **individual resident in Egypt** releasing NativeSnip for Windows. Provider information was checked against the linked live pages on 2026-10-05; recheck eligibility and terms before applying or paying.

**Status:** No signed artifacts or store approvals are claimed here. The opt-in signing pipeline is implemented and covered by mocked unit tests, but has not yet been exercised with a real signing identity or NSIS compiler. Signing does not fix runtime behavior; runtime fixes remain deferred. The existing GitHub Actions workflow remains unsigned: signing CI is not automatically enabled.

## Three different identities

- **Windows publisher:** the legal identity in the Authenticode certificate. It is not set by the extension name or installer branding. With SignPath Foundation, the publisher is **SignPath Foundation**; with an individually validated certificate, expect your provider-approved personal identity. Confirm the exact public subject before purchase.
- **Product:** NativeSnip, with consistent product/version metadata in the helper, installer and uninstaller. Product metadata does not replace certificate identity.
- **Browser extension:** Firefox's add-on ID or a Chromium store/key-derived ID. Browser signing and store review are separate from Windows signing. Neither substitutes for the other.

A valid signature establishes signer identity and integrity, not a guarantee of safety, store acceptance, or a warning-free download. SmartScreen reputation can still produce warnings for correctly signed releases. Do not promise that IV, OV, EV or Foundation signing eliminates all warnings.

## Provider decision for an Egypt resident

| Option | Current evidence | Next action |
| --- | --- | --- |
| SignPath Foundation | Free signing for accepted open-source projects; discretionary admission based on reputation, repository control and verifiable source builds. Publisher is SignPath Foundation, not your personal name or NativeSnip. | Apply with the public repository and release history. Ask about this project's eligibility and the complete helper/NSIS signing sequence. Do not assume acceptance. |
| SSL.com IV Individual Code Signing | Currently offered to individuals, with eSigner cloud signing or a supported hardware token/HSM. This establishes that an individual product exists, **not** that an Egypt-resident applicant will be approved. | Obtain written Egypt eligibility confirmation and a current quote before paying. Ask for the exact validation and integration requirements. |
| Microsoft Artifact Signing Public Trust | Individuals must be located in the US or Canada. Its published organization-country list also excludes Egypt. | Not an available Public Trust route for the stated Egypt-resident individual situation. Selecting an Azure region does not change eligibility. Private Trust is not a substitute for public Windows distribution. |

Sources:

- [SignPath Foundation](https://signpath.org)
- [SignPath Foundation terms and code of conduct](https://signpath.org/terms) (the page labels the code of conduct a draft)
- [SSL.com code signing products](https://www.ssl.com/products/software-integrity/code-signing/)
- [Microsoft Artifact Signing quickstart and eligibility](https://learn.microsoft.com/en-us/azure/artifact-signing/quickstart)

No prices are quoted here. Request a complete current quote covering certificate duration, renewal, eSigner subscription/signing limits or token costs, delivery to Egypt, taxes, and any validation charges. NativeSnip is a user-mode helper, not a kernel driver; do not buy an EV driver-signing product merely to assume instant SmartScreen reputation.

### SignPath Foundation application checklist

1. Publish and maintain the source repository, build scripts, license, documentation and actual releases. The project currently uses MIT; confirm all distributed components satisfy the Foundation's open-source conditions.
2. Provide evidence of project reputation, maintenance and repository ownership. A newly published repository is not automatically eligible; Foundation acceptance is discretionary.
3. Explain the Windows screenshot helper, native messaging, installer system changes and uninstall behavior. Link the privacy policy and release/download pages.
4. Name authors/committers, reviewers and signing approvers. Enable MFA for both repository and SignPath access. Ask how the roles apply to a solo-maintainer project rather than inventing approval arrangements.
5. Arrange a verifiable source-to-binary CI build, including build scripts and dependencies. Every release requires manual signing approval; do not bypass the service's provenance restrictions by uploading unrelated local executables.
6. Set and enforce NativeSnip product-name metadata and one consistent product version for each build's signed binaries. Ask explicitly about the Rust helper, NSIS setup and temporary uninstaller coverage.
7. Once accepted, publish a **Code signing policy** on the project home and download/release pages, including actual role holders, a privacy-policy link, and the required attribution: “Free code signing provided by SignPath.io, certificate by SignPath Foundation”. Do not present that attribution as an existing sponsorship before approval.

**Integration remains provider-specific and pending approval.** SignPath may require asynchronous CI submission, manual approval, retrieval of signed artifacts, and repackaging. It is not automatically compatible with a generic synchronous `sign_command`. Agree on how the helper is signed before setup embeds it, how the temporary NSIS uninstaller is signed before embedding, and how the final setup is signed before extension packaging. An outer setup signature alone does not sign its embedded helper or uninstaller.

### SSL.com inquiry to send before ordering

Use the provider's official sales/support channel:

> I am an individual resident in Egypt releasing an open-source Windows user-mode helper and NSIS installer called NativeSnip. Can you issue IV Individual Code Signing to me at my actual Egyptian residential address? Please confirm accepted identity/address documents, any translation or verification requirements, the exact public certificate subject/publisher name, and a complete current quote. Please compare eSigner and a supported token delivered to Egypt, including renewal, signing limits and timestamping. I need to sign a helper EXE, an NSIS temporary uninstaller during compilation, and the final setup EXE. Please provide the supported unattended/local signing instructions, MFA requirements and certificate-thumbprint retrieval steps.

Submit identity evidence only through the provider's verified secure process. Approval is not established until SSL.com confirms it and completes validation. Do not assume OV organization requirements apply to IV, or that a normal exportable PFX is available: current code-signing key storage uses supported hardware/HSM or cloud custody.

For a token, confirm supported Windows middleware, Windows certificate-store visibility and SignTool access. For eSigner, confirm whether the chosen supported integration exposes the certificate to SignTool or needs a cloud CLI. Obtain the actual command and authentication procedure rather than guessing flags.

## Keep secrets and identity documents out of assistance chats

**Never send the assistant passwords, API keys, PFX files, private keys, token PINs, MFA/OTP codes, recovery codes, or identity documents.** Do not put them in Git, configuration examples, build logs, ZIPs or support screenshots. Complete provider identity validation yourself in its official secure portal.

Safe information to provide for integration:

- A nonsecret approval summary and selected provider/product.
- The public certificate subject and certificate thumbprint (not a private key).
- The approved HTTPS timestamp endpoint.
- Provider tool/version and signing instructions with credentials and private personal data redacted.
- Nonsecret errors or verification results, reviewed before sharing.

Keep credentials in the provider's supported environment/secure store or token. Restrict local config access and keep it outside the repository. Public certificate subjects can expose your legal name; decide deliberately whether that public identity is acceptable.

## Opt-in Windows signing pipeline

The following describes the implemented opt-in pipeline. Real provider signing and NSIS integration still require end-to-end validation. `--sign` is **Windows-only** and requires Windows SDK SignTool plus PowerShell and an approved signing identity. Unsigned builds remain supported.

### External configuration

`NATIVESNIP_SIGN_CONFIG` points to a JSON file outside the repository. Required configuration:

| Key | Meaning |
| --- | --- |
| `signtool` | Path to the installed Windows SDK `signtool.exe`; required even with a custom cloud command because verification uses SignTool. |
| `certificate_thumbprint` | Expected signer certificate thumbprint: exactly 40 hexadecimal characters, without spaces. This SHA-1 certificate identifier is not a choice of file-signature digest. |
| `timestamp_url` | Provider-approved HTTPS RFC 3161 endpoint; required for direct SignTool signing. For a custom command it is needed only if that command's integration requires it. |
| `machine_store` | Optional JSON boolean; selects the machine rather than current-user certificate store for direct signing. |
| `sign_command` | Optional argv list for a supported cloud tool, containing exactly one `{file}` placeholder for the file being signed. |

Illustrative **direct SignTool** configuration; replace every example value locally with verified values before use:

```json
{
  "signtool": "C:\\installed-sdk-path\\signtool.exe",
  "certificate_thumbprint": "0123456789ABCDEF0123456789ABCDEF01234567",
  "timestamp_url": "https://timestamp-provider.example/rfc3161",
  "machine_store": false
}
```

The example thumbprint and endpoint are not operational provider details. Use a certificate available to SignTool through the selected store and approved key provider. Direct signing uses SHA-256 file hashing and RFC 3161 timestamping with SHA-256 timestamp hashing.

For a custom cloud tool, use its approved argv list, not a shell command string. There must be **exactly one `{file}` placeholder**. The tool must complete signing and timestamping before returning success. There is no shell expansion: `$VAR`, `%VAR%`, pipes and shell redirections are not interpreted as command syntax. Credentials must remain in the environment/secure store, never literal argv/config values. No universal SSL.com or SignPath CLI recipe is supplied because the integration depends on approval and the chosen tool.

Custom signing still requires `signtool` and a matching `certificate_thumbprint`. The custom tool is responsible for the timestamp; `timestamp_url` is only required by the direct SignTool path. A rotating cloud certificate requires the expected thumbprint to be updated from trusted public certificate information; do not disable the identity check to accommodate rotation.

### Build and verification commands

Run from the source root in PowerShell. The external path below is an example; create your real config locally, outside the checkout:

```powershell
$env:NATIVESNIP_SIGN_CONFIG = 'C:\NativeSnipSigning\signing.json'
python scripts/build.py --sign
```

The signed build sequence must be:

1. Build and sign the helper, then verify it.
2. Build NSIS setup. Its signing hook invokes the same `scripts/sign.py` for the temporary uninstaller before it is embedded.
3. Sign and verify the completed setup.
4. Package the verified setup into both browser packages.

For individual operations, `FILE` below means the actual path to the intended EXE, quoted if it contains spaces:

```text
python scripts/sign.py sign FILE
python scripts/sign.py verify FILE
```

For existing signed helper/setup artifacts, use:

```powershell
python scripts/build.py --package-only --sign
```

This mode **verifies the existing helper and setup without re-signing them**, then packages them. It also requires `dist/signing-record.json` from a completed signing build, checks the helper/setup hashes against that record, and requires the matching-signer uninstaller verification evidence. The NSIS hook writes `dist/uninstaller-signature.json`; the completed build binds that evidence to the final setup hash. These are local build audit records, not independent cryptographic attestations: they rely on a trusted build workspace, and package-only does not extract or independently verify the uninstaller inside setup. Do not construct or copy receipts from unrelated builds to accept externally signed installers. Keep them with the build outputs. This mode is not a repair command for unsigned artifacts and does not retroactively sign an embedded uninstaller. Check the installed uninstaller on a Windows test machine too.

**A full rebuild overwrites signed build outputs.** After external/provider signing, do not run a normal full rebuild and assume the signatures survive. Retrieve approved signed artifacts into the expected pipeline locations and use verified package-only processing. For an asynchronous provider workflow, settle those locations, trusted verification records and dependency order in its integration before releasing; externally signed files alone do not satisfy the current package-only audit-record requirements.

### Required verification gate

Every sign/verify operation must run both:

- `SignTool verify /pa /all /v /tw` against the file.
- A strict PowerShell Authenticode check requiring `Status` to be `Valid`, the signer thumbprint to match `certificate_thumbprint`, and `TimeStamperCertificate` to exist.

SignTool's `/tw` warning alone is not the timestamp gate; the PowerShell check must reject an absent timestamp. A successful cloud CLI exit is not verification. Missing timestamps, mismatched signers, invalid trust, or verification errors must fail the signing/package operation rather than silently ship unsigned files.

Verify on the Windows release machine and on a clean supported Windows installation. Check the helper, final setup and installed uninstaller, then confirm each browser package contains the verified setup. Record signer subject/thumbprint, timestamp and artifact SHA-256 hashes with the release evidence. Signing does not replace the Windows capture, cancellation, native connection and uninstall acceptance tests.

## Browser signing and stable identities

Current `project.json` values:

| Identity | Value |
| --- | --- |
| Native messaging host | `com.autosnips.capture` |
| Firefox add-on ID | `autosnips@autosnips.local` |
| Chromium development/key-derived ID | `fibdpigmdcoeannimooiajdkkhioijno` |

Preserve the host name and existing upgrade-compatible paths. Never broaden native messaging access with wildcard origins merely to get a new store package working.

### Firefox: AMO listed or unlisted

1. Choose a **listed** AMO submission for public AMO distribution, or **unlisted** signing for self-distribution. Both routes require Mozilla signing for persistent installation in normal release Firefox; renaming an unsigned ZIP to XPI is not signing.
2. Preserve `autosnips@autosnips.local` in the generated Firefox manifest and the helper's `allowed_extensions`. The `.local` suffix is part of the existing add-on ID, not a support URL or email contact to publish.
3. Submit the Firefox package through AMO's supported process. Provide source/build instructions and explain the bundled native installer, nativeMessaging permission and local privacy behavior as requested.
4. Distribute the signed XPI returned by Mozilla without changing its contents. Use AMO's supported version/update process for later releases; unlisted distribution also needs an explicit update/distribution plan.
5. Test the actual signed XPI with the helper. Windows Authenticode signing remains a separate requirement/choice and does not grant AMO approval.

### Chrome: reserve the final identity before release signing

1. Create a Chrome Web Store **draft** item and obtain its assigned extension ID and public key through the developer dashboard's supported process before final release packaging.
2. Compare these with `project.json`. Update `chromium_key` and `chromium_id` together to the intended store identity when needed. The key is public; never send or commit a private extension signing key.
3. Rebuild the extension and helper/native registration so the manifest identity and exact `chrome-extension://ID/` allowed origin agree. Test the draft/store-installed extension against the rebuilt helper.
4. Only then produce the final signed helper/setup and submission packages. A development ID/key is not evidence that a future store listing will have that ID.
5. Complete permission justifications, disclosure and store review. Bundling a signed helper installer does not guarantee Chrome Web Store acceptance.

### Edge: defer a separate store launch

A separate Microsoft Edge Add-ons listing has a different store-assigned identity from Chrome. The current project has **one configured Chromium identity/allowance**, not independent Chrome and Edge store identities. Browser registration for Edge does not by itself authorize a second extension ID.

Do not launch a separate Edge listing or silently add another origin as part of this guide. Defer it unless requested. A later Edge launch needs an explicit per-store identity/packaging and native-allowlist change, followed by rebuilds and tests of both store installations. Edge developer-mode testing of the current Chromium package is distinct from an Edge store release.

## Public support and privacy URLs

Before provider application or store submission, publish stable, publicly accessible HTTPS URLs for:

- **Support:** a maintained project support page or issue tracker with clear contact/reporting instructions. Do not use `autosnips@autosnips.local` as a contact address.
- **Privacy:** a published version of [PRIVACY.md](PRIVACY.md), accurately covering local screenshot saving, diagnostic metadata, user-controlled cloud-synced folders and no automatic uploads.
- **Source/build and releases:** the real public repository, build instructions and download/release pages. Add the accepted provider's code signing policy/attribution where required.

No actual repository/domain/support URL is configured in the source privacy document, so none is invented here. Supply the real public URLs in provider applications and store listings, and verify that they work without login. Identify any provider/store document sharing separately from NativeSnip's runtime behavior; cloud signing uploads build artifacts to the chosen service and is subject to its terms, limits and costs.

## Release stop/go checklist

- [ ] Provider has actually approved this applicant/project; certificate publisher identity is understood.
- [ ] Provider-specific signing and timestamping integration is tested, including the temporary NSIS uninstaller.
- [ ] External configuration contains the real matching public thumbprint and no secrets.
- [ ] Final browser IDs and exact native allowlists agree; Firefox's existing ID is preserved.
- [ ] Helper, setup and uninstaller have valid matching-signer timestamped signatures.
- [ ] Browser packages embed the verified setup; no later full rebuild overwrote signed outputs.
- [ ] Support/privacy/source URLs and required signing-policy information are publicly available.
- [ ] Windows acceptance tests and the relevant browser-store/signing reviews are complete.
- [ ] Release notes describe the actual signing/store status without claiming guaranteed SmartScreen reputation.

Until these gates are met, label artifacts as development/unsigned or otherwise accurately describe their verified status. This guide alone does not produce a signed release.
