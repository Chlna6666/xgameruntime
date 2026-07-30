# Windows x64 package

This package contains the Rust/MSVC `xgameruntime.dll` proxy for native Windows Minecraft Bedrock GDK.

## Contents

- `xgameruntime.dll` — Rust proxy DLL;
- `BMCBL_PROTOCOL.md` — launcher-to-DLL process contract;
- `preauth-v2.schema.json` — machine-readable pre-authentication schema;
- `LICENSE` and `NOTICE.md` — licensing and provenance;
- `manifest.json` — build version, target and source commit;
- `SHA256SUMS` — package file checksums.

The Microsoft runtime is not redistributed in this package.

## Proxy layout

Recommended process-local layout:

```text
xgameruntime.dll    # this project, loaded by the game
xgameruntime_o.dll  # process-local copy of Microsoft's xgameruntime.dll
```

During `DllMain(DLL_PROCESS_ATTACH)`, the proxy follows the reference C implementation: it disables per-thread DLL notifications and synchronously starts the native-runtime loading chain.

The native runtime is attempted in this order:

```text
1. BMCBL_NATIVE_XGAMERUNTIME absolute-path override, when configured
2. sibling xgameruntime_o.dll
3. C:\Windows\System32\xgameruntime.dll
```

A failed attach-time preload is not cached permanently. Later forwarded API calls retry the complete chain. On an explicit proxy unload, the loaded native module is released with `FreeLibrary`.

## Diagnostics

No third-party logging library is required. The proxy reports load attempts, failures, Win32 error codes, the selected target, missing exports, and unload state through both standard error and Win32 `OutputDebugStringW`.

The output can be viewed with Visual Studio, WinDbg, DebugView, or a console-attached test host.

## Requirements

- Windows x64;
- Microsoft Gaming Services or an accessible System32 `xgameruntime.dll`;
- preferably, an original Microsoft runtime copied into the process-local directory as `xgameruntime_o.dll`;
- optionally, a launcher that supplies the BMCBL custom-XUser environment variables documented in `BMCBL_PROTOCOL.md`.

## Important

This is an experimental per-process proxy. It is not a system-wide replacement for Gaming Services. Do not overwrite or rename files inside the Microsoft Gaming Services installation. Prepare a process-local copy instead.

Custom XUser interception remains disabled unless `BMCBL_XGAMERUNTIME_ENABLE_XUSER=1` and the complete schema-v2 launch context are supplied. If every native-runtime candidate fails, the proxy DLL still attaches like the C implementation, but APIs that require forwarding fail explicitly.
