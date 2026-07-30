# xgameruntime

**English** | [简体中文](README.zh-CN.md)

Compatibility and proxy components for Minecraft Bedrock GDK `xgameruntime.dll` environments.

This repository produces two distinct runtime artifacts:

- **Windows native** — a Rust/MSVC per-process proxy for BMCBL-managed account profiles, short-lived Xbox pre-authentication, native Microsoft runtime forwarding, and optional CNG request signing.
- **Wine** — a Wine `xgameruntime` module built from a pinned WineGDK source revision for use with matching WineGDK/GDK-Proton runtimes.

> Status: experimental. The Windows implementation includes native forwarding, profile validation, XUser identity, native XAsync integration, TokenAndSignature responses, and optional CNG-backed request signatures. The Wine artifact is reproducibly built from a pinned WineGDK commit. BMCBL integration and Minecraft end-to-end validation are not complete, so neither artifact should be treated as production-ready.

## Windows architecture

The Windows proxy owns only the account-specific XUser surface. All unrelated GDK runtime classes continue to use the original Microsoft runtime supplied by BMCBL.

```text
Minecraft Bedrock GDK
        │
        ▼
xgameruntime.dll (Rust proxy)
        ├─ validated XUser GUID → Rust IXUserImpl
        └─ every other API      → native Microsoft xgameruntime.dll
```

The Rust XUser provider uses the native Microsoft `IXThreadingImpl` for `XAsyncBegin`, scheduling, completion, and result retrieval. It does not write private `XAsyncBlock` state itself.

## Windows implementation

- Windows MSVC `cdylib` producing `xgameruntime.dll`;
- WineGDK-compatible export names and ordinals;
- absolute-path delayed loading and forwarding to the original Microsoft runtime;
- BMCBL-selected profile and strict pre-authentication schema v2;
- profile ID, nonce, time range, XUID, UHS, relying-party, and CNG key-name validation;
- secret redaction and zeroization for tokens, authorization values, digests, request bodies, and result buffers;
- validated XUser runtime-class/interface interception behind an explicit feature gate;
- Windows x64 `IXUserImpl` 50-slot vtable and `IXUserGamertag` interface;
- XUser handles, XUID, local ID, signed-in state, age group, and privilege queries;
- `XUserAddAsync`/`XUserAddResult` using the native XAsync state machine;
- ANSI and UTF-16 `XUserGetTokenAndSignature*` results;
- URL-to-relying-party routing for Xbox Live, Multiplayer, Realms, PlayFab/SISU, and Licensing;
- `XBL3.0 x=<uhs>;<token>` authorization generation from short-lived launcher pre-authentication;
- optional Xbox proof-of-possession signatures using a persistent current-user CNG P-256 key;
- a Windows CNG integration test that creates a temporary P-256 key, signs an Xbox request digest, verifies the P1363 signature, and deletes the key;
- Windows/Ubuntu CI checks and tests;
- LGPL-2.1-or-later licensing and Wine/WineGDK provenance notices.

## Wine artifact

The Wine package is not the Windows Rust proxy. It does not use the Windows CNG key store, native Gaming Services forwarding, or the BMCBL schema-v2 Windows proxy contract.

The release workflow currently builds from:

```text
Repository: https://github.com/Chlna6666/WineGDK
Commit: 75637b674e1f191e65753663c4c0c32bea05ba6e
Source path: dlls/xgameruntime
```

Wine built-in modules may depend on a matching Wine build tree, generated interfaces, and runtime layout. Use the artifact with a WineGDK/GDK-Proton runtime based on the same or a verified-compatible revision. ABI compatibility with unrelated system Wine releases is not guaranteed.

## BMCBL launch contract

BMCBL supplies the original Microsoft runtime path and optional custom-profile data through process-scoped environment variables:

```text
BMCBL_NATIVE_XGAMERUNTIME=<absolute path to original xgameruntime.dll>
BMCBL_XGAMERUNTIME_PROFILE=<profile id>
BMCBL_XGAMERUNTIME_PREAUTH=<absolute path to short-lived schema-v2 JSON>
BMCBL_XGAMERUNTIME_NONCE=<per-launch nonce>
BMCBL_XGAMERUNTIME_ENABLE_XUSER=1
```

Long-lived Microsoft refresh tokens remain in BMCBL's protected credential store and must never be written into DLL configuration or the pre-authentication document.

The schema-v2 document may contain an optional CNG key name:

```json
{
  "device_signing": {
    "cng_key_name": "BMCBL.XboxDevice.account-2535458430309376"
  }
}
```

Only the key name is passed. The private P-256 key remains non-exported in the current-user CNG key store.

Protocol documentation:

- [BMCBL protocol (English)](docs/BMCBL_PROTOCOL.md)
- [BMCBL 协议（简体中文）](docs/BMCBL_PROTOCOL.zh-CN.md)
- [Pre-authentication JSON Schema](docs/preauth-v2.schema.json)

## Xbox request signing

When `device_signing` is configured, the Windows DLL:

1. opens the named key from Microsoft Software Key Storage Provider;
2. builds the Xbox signing input from policy version, FILETIME, uppercase method, request target, authorization, selected policy headers, and body;
3. computes SHA-256;
4. signs with ECDSA P-256 through `NCryptSignHash`;
5. returns the 76-byte `version || timestamp || r || s` structure as standard padded Base64.

If signing is configured but fails, the request fails rather than silently returning an unsigned result. If `device_signing` is absent, TokenAndSignature returns a valid authorization token and an empty signature.

## Build the Windows artifact

Initial supported target:

```text
x86_64-pc-windows-msvc
```

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

Output:

```text
target/x86_64-pc-windows-msvc/release/xgameruntime.dll
```

Custom XUser interception is disabled by default. A controlled test launch must provide a complete valid profile context and explicitly set `BMCBL_XGAMERUNTIME_ENABLE_XUSER=1` in the Minecraft child process.

## Packaging and releases

`.github/workflows/package-release.yml` produces two independent archives:

```text
xgameruntime-<version>-windows-x64.zip
xgameruntime-<version>-wine-x64.zip
```

Each archive contains:

- the relevant DLL artifact;
- English and Simplified Chinese README files;
- `manifest.json` with exact source revisions;
- licensing and provenance files;
- per-file `SHA256SUMS`.

Published GitHub releases additionally contain `SHA256SUMS.txt` for the two ZIP assets.

The artifacts are not interchangeable:

- do not overwrite the system-wide Microsoft Gaming Services installation with the Windows package;
- do not copy the Wine package into Microsoft Gaming Services;
- install the Wine module only through a matching WineGDK/GDK-Proton runtime layout or packaging system.

## Validation

The GitHub Actions matrix runs on Ubuntu and Windows:

```text
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
```

The Windows job additionally builds the release DLL for `x86_64-pc-windows-msvc`. Windows tests exercise Microsoft Software Key Storage Provider by creating a temporary ECDSA P-256 key, using the production signing path, verifying the signature, and deleting the key.

The packaging workflow independently tests and builds the Windows artifact, configures a pinned WineGDK x64 build tree, builds the Wine module, creates bilingual archives, generates checksums, and publishes prereleases from release branches or version tags.

## Remaining work

- BMCBL account-store integration;
- persistent CNG P-256 key creation and public JWK export in BMCBL;
- stable Xbox device ID and device/SISU enrollment in BMCBL;
- per-launch pre-authentication schema-v2 generation;
- native runtime path resolution, environment setup, and DLL injection in BMCBL;
- `ForceRefresh` coordination when a running game's short-lived token set approaches expiry;
- completion of XUser methods currently left as explicit stubs where Minecraft requires them;
- Minecraft Bedrock GDK startup, friends, multiplayer, Realms, and Marketplace validation;
- XSAPI behavior verification with the custom XUser handle.

## Porting policy

Do not mechanically translate the complete WineGDK directory. Each source unit must first be classified as:

1. portable GDK ABI/interface logic;
2. reusable Xbox authentication or request-signing logic;
3. Wine-only loader, registry, TLS, or threading behavior;
4. Windows-native proxy behavior;
5. BMCBL-owned profile and credential management.

Unsupported runtime classes remain forwarded to the native Microsoft runtime. Unsupported XUser methods remain explicit stubs until their ABI and Minecraft call paths are validated.

## Licensing

This repository is distributed under **GNU LGPL-2.1-or-later** to preserve the Wine/WineGDK licensing boundary of the source implementation.

The implementation is derived from:

- `Chlna6666/WineGDK`;
- source path: `dlls/xgameruntime`;
- upstream lineage: WineGDK and Wine.

Existing copyright and license notices must be retained when code is ported. See [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md).
