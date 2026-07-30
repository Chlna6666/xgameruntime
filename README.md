# xgameruntime

**English** | [简体中文](README.zh-CN.md)

Compatibility, proxy, and account-bridge components for Minecraft Bedrock GDK `xgameruntime.dll` environments.

This repository produces two different artifacts:

- **Windows native** — a Rust/MSVC per-process man-in-the-middle proxy with Microsoft runtime forwarding, BMCBL profile support, short-lived Xbox pre-authentication, and optional CNG request signing;
- **Wine** — a Wine `xgameruntime` module built from a pinned WineGDK source revision for matching WineGDK/GDK-Proton environments.

> Status: experimental. The Windows proxy loading chain, native forwarding, XUser, XAsync, TokenAndSignature, and CNG signing paths are implemented. Minecraft sign-in, friends, multiplayer, Realms, Marketplace, and related end-to-end scenarios still require validation.

## Windows proxy architecture

The Windows artifact is the `xgameruntime.dll` loaded by the game. A process-local copy of the original Microsoft runtime is named `xgameruntime_o.dll`:

```text
Minecraft Bedrock GDK
        │
        ▼
xgameruntime.dll (Rust MITM proxy)
        ├─ validated XUser GUID → Rust IXUserImpl
        └─ other runtime APIs   → xgameruntime_o.dll
```

Default layout:

```text
<game runtime directory>/
├─ Minecraft.Windows.exe
├─ xgameruntime.dll      # proxy built by this project
└─ xgameruntime_o.dll    # process-local copy of Microsoft's xgameruntime.dll
```

Do not overwrite or rename files inside the Microsoft Gaming Services installation. Copy the original DLL into the process-local game layout and rename that copy to `xgameruntime_o.dll`.

## Windows preload chain

The proxy follows the reference C implementation's `DllMain` behavior:

1. the game loads `xgameruntime.dll`;
2. `DllMain(DLL_PROCESS_ATTACH)` calls `DisableThreadLibraryCalls`;
3. it synchronously calls `LoadLibraryA("xgameruntime_o.dll")`;
4. `QueryApiImpl` first attempts the Rust XUser interception path;
5. non-intercepted interfaces and native exports are resolved through `GetProcAddress` on the original module;
6. an explicit proxy unload releases the original module with `FreeLibrary`.

An attach-time preload failure does not prevent the proxy itself from attaching, matching the C implementation. The failure is not cached permanently: later forwarded calls retry loading `xgameruntime_o.dll`. If the original runtime still cannot be loaded, APIs requiring native forwarding fail explicitly.

## Optional native-runtime override

The default proxy layout does not require a runtime-path environment variable. It loads:

```text
xgameruntime_o.dll
```

A launcher may optionally provide a different process-local copy through an absolute-path override:

```text
BMCBL_NATIVE_XGAMERUNTIME=<absolute path to a copy of Microsoft's runtime>
```

The override should be scoped to the Minecraft child process and must not be configured globally.

## Windows implementation

- Windows x64 MSVC `cdylib` producing `xgameruntime.dll`;
- WineGDK-compatible export names and ordinals;
- C-style `DllMain` preload and `xgameruntime_o.dll` proxy layout;
- dynamic lookup and forwarding of native exports;
- `QueryApiImpl` man-in-the-middle interception;
- BMCBL profile handling and strict pre-authentication schema v2;
- Windows x64 `IXUserImpl` 50-slot vtable and `IXUserGamertag`;
- XUser handles, XUID, local ID, signed-in state, age group, and privilege queries;
- reuse of Microsoft native `IXThreadingImpl` and XAsync state management;
- `XUserAddAsync` and `XUserAddResult`;
- ANSI and UTF-16 `XUserGetTokenAndSignature*`;
- relying-party routing for Xbox Live, Multiplayer, Realms, PlayFab/SISU, and Licensing;
- optional Windows CNG P-256 Xbox proof-of-possession signatures;
- zeroization of token, authorization, digest, body, and result buffers;
- Windows/Ubuntu CI, Windows CNG integration tests, and MSVC release builds.

## BMCBL custom-XUser launch contract

Pure native proxy forwarding does not require BMCBL profile variables.

To enable the custom BMCBL XUser provider, set these variables only for the Minecraft child process:

```text
BMCBL_XGAMERUNTIME_PROFILE=<profile id>
BMCBL_XGAMERUNTIME_PREAUTH=<absolute path to schema-v2 JSON>
BMCBL_XGAMERUNTIME_NONCE=<per-launch random nonce>
BMCBL_XGAMERUNTIME_ENABLE_XUSER=1
```

Long-lived Microsoft refresh tokens, account passwords, and OAuth authorization codes must never be passed to the DLL. The DLL receives only short-lived Xbox pre-authentication material for one game process and an optional CNG key name.

Protocol documentation:

- [BMCBL protocol](docs/BMCBL_PROTOCOL.md)
- [BMCBL 协议（简体中文）](docs/BMCBL_PROTOCOL.zh-CN.md)
- [Pre-authentication JSON Schema](docs/preauth-v2.schema.json)

## Build the Windows artifact

Primary target:

```text
x86_64-pc-windows-msvc
```

Build:

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

Output:

```text
target/x86_64-pc-windows-msvc/release/xgameruntime.dll
```

Deployment also requires a process-local copy of the original Microsoft DLL named:

```text
xgameruntime_o.dll
```

The Microsoft DLL is not redistributed by this project.

## Packaging and version

Current version:

```text
v0.1.0-beta.2
```

The release workflow produces:

```text
xgameruntime-<version>-windows-x64.zip
xgameruntime-<version>-wine-x64.zip
```

The Windows archive contains the proxy, protocol files, schema, licenses, build manifest, and SHA-256 checksums. It does not contain Microsoft's original `xgameruntime.dll`.

## Wine artifact

The Wine package is not the Windows Rust proxy. It does not use the Windows `DllMain` preload path, Microsoft Gaming Services, Windows CNG, or the `xgameruntime_o.dll` layout.

Pinned source:

```text
Repository: https://github.com/Chlna6666/WineGDK
Commit: 75637b674e1f191e65753663c4c0c32bea05ba6e
Source path: dlls/xgameruntime
```

Use the Wine module with a WineGDK/GDK-Proton runtime based on the same or a verified-compatible source revision.

## Security and limitations

- do not modify the Microsoft Gaming Services installation;
- do not register the proxy as a system-wide runtime;
- do not write refresh tokens, passwords, or reusable private keys into DLL configuration;
- custom XUser interception is disabled by default;
- unsupported methods remain explicit failures or stubs;
- Minecraft sign-in, friends, multiplayer, Realms, Marketplace, and XSAPI still require end-to-end validation.

## Licensing

This repository is distributed under **GNU LGPL-2.1-or-later** and preserves the Wine/WineGDK provenance and licensing boundary. See [LICENSE](LICENSE) and [NOTICE.md](NOTICE.md).
