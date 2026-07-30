# Notices and provenance

## Project license

This repository is distributed under **GNU Lesser General Public License version 2.1 or later** (`LGPL-2.1-or-later`). New source files use the SPDX identifier:

```text
SPDX-License-Identifier: LGPL-2.1-or-later
```

## WineGDK source provenance

The project is a Rust port and Windows-oriented adaptation of concepts and ABI work from:

- repository: `https://github.com/Chlna6666/WineGDK`
- source path: `dlls/xgameruntime`
- extraction baseline: commit `75637b674e1f191e65753663c4c0c32bea05ba6e`
- upstream lineage: WineGDK and Wine

When a Rust file is translated from, structurally derived from, or incorporates a non-trivial portion of a WineGDK/Wine source file, the original copyright and license header must be retained in that Rust file, followed by a clear modification notice.

## CC0 statement in WineGDK

The WineGDK README states that contributions written by its author and not derived from other parts of Wine, including identified `xgameruntime` work, are offered under CC0/public-domain terms. That statement does **not** relicense code derived from Wine or other third-party work.

For a clear distribution boundary, this standalone project uses `LGPL-2.1-or-later` as its unified project license while preserving any narrower or more permissive original notices. Applying the unified project license does not remove original authorship or third-party notices.

## Microsoft materials

This repository must not include:

- proprietary Microsoft GDK headers or SDK libraries;
- Microsoft Gaming Services binaries;
- extracted private symbols or confidential documentation;
- Microsoft client secrets;
- user refresh tokens, Xbox tokens, device keys, pre-authentication files, or other credentials.

Microsoft, Xbox, Minecraft, Windows, and related names are trademarks of their respective owners. This project is not affiliated with or endorsed by Microsoft.

## BMCBL separation

Better Minecraft Bedrock Launcher remains a separate work. Long-lived account credentials belong in BMCBL's protected credential store. This DLL accepts only the selected profile identifier and short-lived, versioned pre-authentication data for one process launch.
