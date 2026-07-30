# xgameruntime

Rust-based, per-process `xgameruntime.dll` compatibility and proxy layer for Minecraft Bedrock GDK on Windows.

The project is intended to integrate with [Better Minecraft Bedrock Launcher](https://github.com/Chlna6666/Better-Minecraft-Bedrock-Launcher) and provide launcher-managed account profiles plus short-lived Xbox pre-authentication data without modifying the system-wide Microsoft Gaming Services installation.

> Status: experimental XUser implementation. Native runtime forwarding, profile validation, XUser identity, native XAsync integration, TokenAndSignature responses and optional CNG-backed Xbox request signatures are implemented. BMCBL integration and Minecraft end-to-end validation are not complete, so this branch is not production-ready.

## Architecture

The proxy owns only the account-specific XUser surface. All unrelated GDK runtime classes continue to use the original Microsoft runtime supplied by BMCBL.

```text
Minecraft Bedrock GDK
        │
        ▼
xgameruntime.dll (Rust proxy)
        ├─ validated XUser GUID → Rust IXUserImpl
        └─ every other API      → native Microsoft xgameruntime.dll
```

The Rust XUser provider uses the native Microsoft `IXThreadingImpl` for `XAsyncBegin`, scheduling, completion and result retrieval. It does not write private `XAsyncBlock` state itself.

## Implemented

- Windows MSVC `cdylib` producing `xgameruntime.dll`;
- WineGDK-compatible export names and ordinals;
- absolute-path delayed loading and forwarding to the original Microsoft runtime;
- BMCBL-selected profile and strict pre-authentication schema v2;
- profile ID, nonce, time range, XUID, UHS, relying-party and CNG key-name validation;
- secret redaction and zeroization for token, authorization, digest and result buffers;
- validated XUser runtime-class/interface interception behind an explicit feature gate;
- Windows x64 `IXUserImpl` 50-slot vtable and `IXUserGamertag` interface;
- XUser handles, XUID, local ID, signed-in state, age group and privilege queries;
- `XUserAddAsync`/`XUserAddResult` using the native XAsync state machine;
- ANSI and UTF-16 `XUserGetTokenAndSignature*` results;
- URL-to-relying-party routing for Xbox Live, Multiplayer, Realms, PlayFab/SISU and Licensing;
- `XBL3.0 x=<uhs>;<token>` authorization generation from short-lived launcher pre-authentication;
- optional Xbox proof-of-possession signatures using a persistent current-user CNG P-256 key;
- Windows/Ubuntu CI checks;
- LGPL-2.1-or-later license and Wine/WineGDK provenance notices.

## Remaining work

- BMCBL account-store integration;
- persistent CNG P-256 key creation and public JWK export in BMCBL;
- stable Xbox device ID and device/SISU enrollment in BMCBL;
- per-launch pre-authentication schema-v2 generation;
- native runtime path resolution, environment setup and DLL injection in BMCBL;
- `ForceRefresh` coordination when a running game's short-lived token set approaches expiry;
- completion of XUser methods currently left as explicit stubs where Minecraft requires them;
- Minecraft Bedrock GDK startup, friends, multiplayer, Realms and Marketplace validation;
- XSAPI behavior verification with the custom XUser handle.

## BMCBL launch contract

BMCBL supplies the original runtime path and optional custom-profile data through process-scoped environment variables:

```text
BMCBL_NATIVE_XGAMERUNTIME=<absolute path to original xgameruntime.dll>
BMCBL_XGAMERUNTIME_PROFILE=<profile id>
BMCBL_XGAMERUNTIME_PREAUTH=<absolute path to short-lived schema-v2 JSON>
BMCBL_XGAMERUNTIME_NONCE=<per-launch nonce>
BMCBL_XGAMERUNTIME_ENABLE_XUSER=1
```

Long-lived Microsoft refresh tokens stay in BMCBL's protected credential store and must never be written into the DLL configuration.

The schema-v2 document may contain an optional CNG key name such as:

```json
{
  "device_signing": {
    "cng_key_name": "BMCBL.XboxDevice.account-2535458430309376"
  }
}
```

Only the key name is passed. The private P-256 key remains non-exported in the Windows current-user CNG key store. See [`docs/BMCBL_PROTOCOL.md`](docs/BMCBL_PROTOCOL.md) for the complete contract.

## Xbox request signing

When `device_signing` is configured, the DLL:

1. opens the named key from the Microsoft Software Key Storage Provider;
2. builds the Xbox signature input from policy version, FILETIME, uppercase method, request target, authorization, selected policy headers and body;
3. computes SHA-256;
4. signs with ECDSA P-256 through `NCryptSignHash`;
5. returns the 76-byte `version || timestamp || r || s` structure as standard padded Base64.

If signing is configured but fails, the request fails rather than silently returning an unsigned result. If `device_signing` is absent, the TokenAndSignature result contains a valid authorization token and an empty signature.

## Build

Windows x64 MSVC is the initial supported target:

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

Output:

```text
target/x86_64-pc-windows-msvc/release/xgameruntime.dll
```

Custom XUser interception is disabled by default. For a controlled test launch, all profile variables must be valid and `BMCBL_XGAMERUNTIME_ENABLE_XUSER=1` must be set in the Minecraft child process.

## Porting policy

Do not mechanically translate the complete WineGDK directory. Each source unit must first be classified as:

1. portable GDK ABI/interface logic;
2. reusable Xbox authentication or request-signing logic;
3. Wine-only loader, registry, TLS or threading behavior;
4. Windows-native proxy behavior;
5. BMCBL-owned profile and credential management.

Unsupported runtime classes remain forwarded to the native Microsoft runtime. Unsupported XUser methods are explicit stubs until their ABI and Minecraft call paths are validated.

## Licensing

This repository is distributed under **GNU LGPL-2.1-or-later** to preserve the Wine/WineGDK licensing boundary of the source implementation.

The implementation is derived from:

- `Chlna6666/WineGDK`;
- path: `dlls/xgameruntime`;
- upstream lineage: WineGDK and Wine.

Existing copyright and license notices must be retained when code is ported. See [`LICENSE`](LICENSE) and [`NOTICE.md`](NOTICE.md).
