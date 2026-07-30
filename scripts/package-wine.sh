#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 6 ]]; then
  echo "usage: $0 <version> <project-commit> <winegdk-commit> <winegdk-dir> <wine-build-dir> <output-dir>" >&2
  exit 2
fi

version="$1"
project_commit="$2"
winegdk_commit="$3"
winegdk_dir="$(realpath "$4")"
wine_build_dir="$(realpath "$5")"
output_dir="$(mkdir -p "$6" && realpath "$6")"
repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

mapfile -t dll_candidates < <(find "$wine_build_dir/dlls/xgameruntime" -maxdepth 2 -type f -name 'xgameruntime.dll' -print)
if [[ ${#dll_candidates[@]} -ne 1 ]]; then
  echo "expected exactly one Wine xgameruntime.dll, found ${#dll_candidates[@]}" >&2
  find "$wine_build_dir/dlls/xgameruntime" -maxdepth 2 -type f -print >&2 || true
  exit 1
fi

package_name="xgameruntime-${version}-wine-x64"
temporary_root="$(mktemp -d)"
stage_dir="${temporary_root}/${package_name}"
validation_root="${temporary_root}/validation"
archive_path="${output_dir}/${package_name}.zip"
mkdir -p "$stage_dir"

cp "${dll_candidates[0]}" "$stage_dir/xgameruntime.dll"

mapfile -t so_candidates < <(find "$wine_build_dir/dlls/xgameruntime" -maxdepth 2 -type f -name 'xgameruntime.dll.so' -print)
if [[ ${#so_candidates[@]} -gt 0 ]]; then
  cp "${so_candidates[0]}" "$stage_dir/xgameruntime.dll.so"
fi

cp "$repository_root/packaging/WINE.md" "$stage_dir/README.md"
cp "$repository_root/packaging/WINE.zh-CN.md" "$stage_dir/README.zh-CN.md"
cp "$winegdk_dir/LICENSE" "$stage_dir/LICENSE.winegdk"
cat > "$stage_dir/SOURCE.md" <<EOF
# Source provenance / 源码来源

- Package project / 打包项目: https://github.com/Chlna6666/xgameruntime
- Package project commit / 打包项目提交: \`${project_commit}\`
- Wine implementation / Wine 实现: https://github.com/Chlna6666/WineGDK
- WineGDK commit / WineGDK 提交: \`${winegdk_commit}\`
- Wine source path / Wine 源路径: \`dlls/xgameruntime\`

The Wine artifact is distributed under the Wine/WineGDK LGPL terms included in \`LICENSE.winegdk\`.
Wine 产物依据 \`LICENSE.winegdk\` 中包含的 Wine/WineGDK LGPL 条款分发。
EOF

python3 - "$stage_dir/manifest.json" <<PY
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
manifest = {
    "schema_version": 1,
    "package": "xgameruntime",
    "version": "${version}",
    "variant": "winegdk",
    "architecture": "x86_64",
    "source_repository": "https://github.com/Chlna6666/xgameruntime",
    "source_commit": "${project_commit}",
    "winegdk_repository": "https://github.com/Chlna6666/WineGDK",
    "winegdk_commit": "${winegdk_commit}",
    "documentation_languages": ["en", "zh-CN"],
    "experimental": True,
}
path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
PY

(
  cd "$stage_dir"
  find . -maxdepth 1 -type f ! -name SHA256SUMS -printf '%f\n' \
    | LC_ALL=C sort \
    | xargs -r sha256sum > SHA256SUMS
  sha256sum -c SHA256SUMS
)

rm -f "$archive_path"
(
  cd "$temporary_root"
  zip -9 -r "$archive_path" "$package_name" >/dev/null
)

mkdir -p "$validation_root"
unzip -q "$archive_path" -d "$validation_root"
validated_package="$validation_root/$package_name"
required_files=(
  xgameruntime.dll
  README.md
  README.zh-CN.md
  LICENSE.winegdk
  SOURCE.md
  manifest.json
  SHA256SUMS
)
for required_file in "${required_files[@]}"; do
  if [[ ! -f "$validated_package/$required_file" ]]; then
    echo "Wine package validation failed, missing: $required_file" >&2
    exit 1
  fi
done
(
  cd "$validated_package"
  sha256sum -c SHA256SUMS
)

archive_hash="$(sha256sum "$archive_path" | cut -d' ' -f1)"
echo "Created $archive_path"
echo "SHA256 $archive_hash"
