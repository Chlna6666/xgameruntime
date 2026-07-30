# Wine x64 package

This package contains the WineGDK-built `xgameruntime` PE module for Minecraft Bedrock GDK under Wine/GDK-Proton.

## Source compatibility

The package is built from the exact WineGDK commit recorded in `manifest.json`. It should be used with a WineGDK or GDK-Proton runtime based on the same source revision. Wine built-in modules are not guaranteed to be ABI-compatible with unrelated system Wine releases.

## Contents

- `xgameruntime.dll` — Wine PE module;
- `xgameruntime.dll.so` — included only when produced by the selected Wine build mode;
- `LICENSE.winegdk` — WineGDK/Wine LGPL license;
- `SOURCE.md` — exact source repository and commit;
- `manifest.json` — release version, architecture and source revisions;
- `SHA256SUMS` — package file checksums.

## Important

This artifact is the Wine implementation, not the native Windows Rust proxy. It does not use the Windows CNG key store or the BMCBL schema-v2 proxy contract.

Install it only through the matching WineGDK/GDK-Proton runtime layout or packaging system. Do not copy it over Microsoft Gaming Services on Windows.
