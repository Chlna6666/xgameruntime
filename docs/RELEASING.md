# Release procedure

**English** | [简体中文](RELEASING.zh-CN.md)

The project publishes two non-interchangeable artifacts:

- `xgameruntime-<version>-windows-x64.zip`
- `xgameruntime-<version>-wine-x64.zip`

## Pre-release checklist

1. the planned version is reflected in project metadata and `CHANGELOG.md`;
2. `WINEGDK_COMMIT` points to a reviewed WineGDK revision;
3. status statements in both README languages are accurate;
4. Windows and Ubuntu CI are green;
5. a `Package and Release` test run produces both artifacts;
6. downloaded test artifacts contain the expected DLL type, bilingual documentation, exact manifests, provenance, and valid internal checksums.

## Test packaging

Normal development-branch and pull-request runs use:

```text
v0.1.0-dev.<run-number>
```

They upload GitHub Actions artifacts without creating a GitHub Release.

A manual `Package and Release` run may also use:

```text
version = v0.1.0-beta.1
publish = false
```

## Create a prerelease

Create a release branch from a merged, validated commit:

```text
release/v0.1.0-beta.1
```

The workflow will:

1. test and build the Windows Rust DLL;
2. check out the pinned WineGDK revision;
3. configure and build the Wine x64 module;
4. create two bilingual ZIP archives;
5. generate combined `SHA256SUMS.txt`;
6. create or update a GitHub prerelease.

Release branches must match:

```text
release/v<major>.<minor>.<patch>[-prerelease]
```

A `v*` tag also publishes, but release branches are preferred during the experimental phase because an asset fix can be rebuilt without moving a version tag.

## Required release assets

A release should contain:

```text
xgameruntime-v0.1.0-beta.1-windows-x64.zip
xgameruntime-v0.1.0-beta.1-wine-x64.zip
SHA256SUMS.txt
```

The Windows ZIP must contain:

- `xgameruntime.dll`;
- `README.md` and `README.zh-CN.md`;
- `BMCBL_PROTOCOL.md` and `BMCBL_PROTOCOL.zh-CN.md`;
- `preauth-v2.schema.json`;
- `LICENSE` and `NOTICE.md`;
- `manifest.json` and `SHA256SUMS`.

The Wine ZIP must contain:

- `xgameruntime.dll`;
- `README.md` and `README.zh-CN.md`;
- `LICENSE.winegdk` and `SOURCE.md`;
- `manifest.json` and `SHA256SUMS`;
- optional `xgameruntime.dll.so` when produced by the selected Wine build mode.

## Failure handling

- Windows test/build failure: fix Rust ABI, CNG tests, or the archive script before retrying.
- Wine configure failure: review Ubuntu dependencies and the pinned WineGDK revision.
- Wine module build failure: run the generated module Makefile in `wine-build/dlls/xgameruntime` with verbose output.
- ZIP validation failure: do not publish; fix the manifest, required-file list, or checksum generation.
- Release upload failure: rerun the publishing workflow; `--clobber` replaces same-name assets.

## Post-release verification

1. verify that the release is marked as a prerelease;
2. verify both ZIP files and `SHA256SUMS.txt` are present;
3. confirm release notes contain English and Simplified Chinese sections;
4. download assets and run `sha256sum -c SHA256SUMS.txt`;
5. extract each archive and validate its internal `SHA256SUMS`;
6. confirm the Windows and Wine artifacts are clearly documented as non-interchangeable;
7. record the release URL in the future BMCBL dependency manifest.
