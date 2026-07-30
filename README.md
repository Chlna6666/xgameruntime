# xgameruntime

Rust-based, per-process `xgameruntime.dll` compatibility and proxy layer for Minecraft Bedrock GDK on Windows.

The project is intended to integrate with [Better Minecraft Bedrock Launcher](https://github.com/Chlna6666/Better-Minecraft-Bedrock-Launcher) and provide launcher-managed account profiles plus short-lived Xbox pre-authentication data without modifying the system-wide Microsoft Gaming Services installation.

> Status: architecture bootstrap. The current branch forwards the native runtime and validates BMCBL profile/pre-authentication input, but deliberately does not replace XUser interfaces yet. It is not ready for production game launches.

## Why Rust

Most of the Windows port can be implemented in Rust:

- profile and token lifetime management;
- strict pre-authentication schema validation;
- native DLL discovery contract;
- proxy state and error handling;
- COM object ownership helpers;
- future XUser and request-signing implementations.

A narrow `unsafe extern "system"` layer is still required for exported Windows ABI functions, COM vtables and calls through native function pointers. The project keeps these operations isolated from the safe profile/authentication code.

## Current bootstrap

Implemented:

- `cdylib` build producing `xgameruntime.dll` on Windows MSVC;
- WineGDK-compatible export names and ordinals;
- absolute-path loading and forwarding to the original Microsoft runtime;
- BMCBL-selected Profile and pre-authentication schema v1;
- strict profile ID, nonce, expiry, XUID and relying-party validation;
- secret redaction and zeroization for token strings;
- native-only fallback when custom profile initialization is unavailable;
- Windows/Ubuntu CI checks;
- LGPL-2.1-or-later license and WineGDK provenance notices.

Not implemented yet:

- XUser runtime class GUID interception;
- Rust COM/vtable implementation of `IXUserImpl`;
- `XUserAddAsync` and `XAsync` result provider integration;
- token/signature response generation;
- XSAPI context switching;
- BMCBL injector integration and native runtime path resolution;
- Minecraft launch validation.

## BMCBL launch contract

BMCBL supplies the original runtime path and optional custom-profile data through process-scoped environment variables:

```text
BMCBL_NATIVE_XGAMERUNTIME=<absolute path to original xgameruntime.dll>
BMCBL_XGAMERUNTIME_PROFILE=<profile id>
BMCBL_XGAMERUNTIME_PREAUTH=<absolute path to short-lived JSON>
BMCBL_XGAMERUNTIME_NONCE=<per-launch nonce>
```

Long-lived Microsoft refresh tokens stay in BMCBL's protected credential store and must never be written into this DLL's configuration. See [`docs/BMCBL_PROTOCOL.md`](docs/BMCBL_PROTOCOL.md).

## Build

Windows x64 MSVC is the initial supported target:

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

Output:

```text
target/x86_64-pc-windows-msvc/release/xgameruntime.dll
```

## Porting policy

Do not mechanically translate the complete WineGDK directory. Each source unit must first be classified as:

1. portable GDK ABI/interface logic;
2. reusable Xbox authentication or request-signing logic;
3. Wine-only loader, registry, TLS or threading behavior;
4. Windows-native proxy behavior;
5. BMCBL-owned profile and credential management.

The initial Windows implementation forwards every unsupported interface to the native runtime. XUser interception will be enabled only after GUID, vtable, calling convention and `XAsync` tests exist.

## Licensing

This repository is distributed under **GNU LGPL-2.1-or-later** to preserve the Wine/WineGDK licensing boundary of the source implementation.

The original implementation is derived from:

- `Chlna6666/WineGDK`;
- path: `dlls/xgameruntime`;
- upstream lineage: WineGDK and Wine.

Existing copyright and license notices must be retained when code is ported. See [`LICENSE`](LICENSE) and [`NOTICE.md`](NOTICE.md).
