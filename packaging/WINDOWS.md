# Windows x64 package

This package contains the Rust/MSVC `xgameruntime.dll` proxy for native Windows Minecraft Bedrock GDK.

## Contents

- `xgameruntime.dll` — Rust proxy DLL;
- `BMCBL_PROTOCOL.md` — launcher-to-DLL process contract;
- `preauth-v2.schema.json` — machine-readable pre-authentication schema;
- `LICENSE` and `NOTICE.md` — licensing and provenance;
- `manifest.json` — build version, target and source commit;
- `SHA256SUMS` — package file checksums.

## Requirements

- Windows x64;
- Microsoft Gaming Services and the original Microsoft `xgameruntime.dll`;
- a launcher that supplies the process-scoped BMCBL environment variables documented in `BMCBL_PROTOCOL.md`.

## Important

This is an experimental per-process proxy. It is not a system-wide replacement for Gaming Services. Do not overwrite the Microsoft Gaming Services installation.

BMCBL must provide an absolute path to the original runtime through `BMCBL_NATIVE_XGAMERUNTIME`. Custom XUser interception remains disabled unless `BMCBL_XGAMERUNTIME_ENABLE_XUSER=1` and the complete schema-v2 launch context are supplied.
