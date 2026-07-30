# xgameruntime

Rust-based, per-process `xgameruntime.dll` compatibility and proxy layer for Minecraft Bedrock GDK on Windows.

The project is intended to integrate with [Better Minecraft Bedrock Launcher](https://github.com/Chlna6666/Better-Minecraft-Bedrock-Launcher) and provide launcher-managed account profiles plus short-lived Xbox pre-authentication data without modifying the system-wide Microsoft Gaming Services installation.

> Status: architecture bootstrap. The current implementation is not yet suitable for launching Minecraft or replacing the native runtime.

## Design goals

- Build the DLL implementation primarily in Rust.
- Preserve the GDK ABI through narrowly scoped `unsafe extern "system"` boundaries.
- Forward unsupported runtime interfaces to the native Microsoft runtime.
- Intercept only selected XUser-related interfaces after ABI validation.
- Keep Microsoft refresh tokens in BMCBL's protected credential store.
- Accept only versioned, short-lived pre-authentication data for the selected launch profile.
- Fail closed and restore native runtime behavior when custom profile initialization fails.

## Licensing

This repository is distributed under **GNU LGPL-2.1-or-later** to preserve the Wine/WineGDK licensing boundary of the source implementation.

The original implementation is derived from:

- `Chlna6666/WineGDK`
- path: `dlls/xgameruntime`
- upstream project: WineGDK / Wine

Existing copyright and license notices must be retained when code is ported. See `LICENSE` and `NOTICE.md`.
