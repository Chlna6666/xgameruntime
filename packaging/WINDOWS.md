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

Place the files used by the game process as follows:

```text
xgameruntime.dll    # this project, loaded by the game
xgameruntime_o.dll  # original Microsoft xgameruntime.dll, renamed
```

During `DllMain(DLL_PROCESS_ATTACH)`, the proxy follows the C implementation and synchronously calls:

```c
LoadLibraryA("xgameruntime_o.dll");
```

It also disables per-thread DLL notifications. On an explicit proxy unload, it releases the original module with `FreeLibrary`. A failed attach-time preload is not cached permanently: a later forwarded API call retries loading `xgameruntime_o.dll`.

`BMCBL_NATIVE_XGAMERUNTIME` remains available as an optional process-scoped absolute-path override. When it is absent, the default proxy layout above is used.

## Requirements

- Windows x64;
- Microsoft Gaming Services and an original Microsoft `xgameruntime.dll` copied for this process as `xgameruntime_o.dll`;
- optionally, a launcher that supplies the BMCBL custom-XUser environment variables documented in `BMCBL_PROTOCOL.md`.

## Important

This is an experimental per-process proxy. It is not a system-wide replacement for Gaming Services. Do not overwrite or rename files inside the Microsoft Gaming Services installation. Prepare a process-local copy instead.

Custom XUser interception remains disabled unless `BMCBL_XGAMERUNTIME_ENABLE_XUSER=1` and the complete schema-v2 launch context are supplied. If the original runtime cannot be loaded, the proxy DLL still attaches like the C implementation, but APIs that require forwarding fail instead of silently reporting success.
