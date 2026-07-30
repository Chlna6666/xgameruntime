# 发布流程（简体中文）

[English](RELEASING.md) | **简体中文**

本项目发布两个不可互换的产物：

- `xgameruntime-<version>-windows-x64.zip`
- `xgameruntime-<version>-wine-x64.zip`

## 发布前检查

1. `Cargo.toml` 版本与计划版本一致；
2. `CHANGELOG.md` 已记录本次版本；
3. `WINEGDK_COMMIT` 固定到经过确认的 WineGDK 提交；
4. `README.md` 和 `README.zh-CN.md` 状态说明准确；
5. Windows 与 Ubuntu CI 全绿；
6. `Package and Release` 测试运行同时生成 Windows 和 Wine Artifact；
7. 下载两个 Artifact，确认：
   - DLL 文件类型正确；
   - 中英文 README 存在；
   - manifest 中提交和版本正确；
   - 包内 `SHA256SUMS` 校验通过。

## 测试打包

向普通开发分支或 PR 推送时，工作流生成开发版本：

```text
v0.1.0-dev.<run-number>
```

该模式只上传 Actions Artifact，不创建 GitHub Release。

也可以在 Actions 中手动运行 `Package and Release`，设置：

```text
version = v0.1.0-beta.1
publish = false
```

## 创建预发布

推荐在已合并并通过 CI 的提交上创建发布分支：

```text
release/v0.1.0-beta.1
```

该分支会自动：

1. 测试并构建 Windows Rust DLL；
2. 检出固定 WineGDK 提交；
3. 配置并构建 Wine x64 模块；
4. 生成两个双语 ZIP；
5. 生成统一 `SHA256SUMS.txt`；
6. 创建或更新 GitHub prerelease。

版本分支必须符合：

```text
release/v<major>.<minor>.<patch>[-prerelease]
```

也可以推送 `v*` 标签触发发布，但在当前阶段优先使用 release 分支，以便修复预发布资产后重新运行。

## 发布资产

GitHub Release 应包含：

```text
xgameruntime-v0.1.0-beta.1-windows-x64.zip
xgameruntime-v0.1.0-beta.1-wine-x64.zip
SHA256SUMS.txt
```

Windows ZIP 内必须包含：

- `xgameruntime.dll`
- `README.md`
- `README.zh-CN.md`
- `BMCBL_PROTOCOL.md`
- `BMCBL_PROTOCOL.zh-CN.md`
- `preauth-v2.schema.json`
- `LICENSE`
- `NOTICE.md`
- `manifest.json`
- `SHA256SUMS`

Wine ZIP 内必须包含：

- `xgameruntime.dll`
- `README.md`
- `README.zh-CN.md`
- `LICENSE.winegdk`
- `SOURCE.md`
- `manifest.json`
- `SHA256SUMS`
- 可选的 `xgameruntime.dll.so`

## 失败处理

- Windows 编译或测试失败：修复 Rust ABI、CNG 测试或归档脚本后重新运行；
- Wine `configure` 失败：检查 Ubuntu 依赖和固定 WineGDK 提交；
- Wine 模块编译失败：在 `wine-build/dlls/xgameruntime` 目录执行详细构建并检查生成头文件；
- ZIP 校验失败：不得发布，先修复 manifest、文件清单或 SHA-256；
- GitHub Release 上传失败：重新运行发布工作流，`--clobber` 会替换同名资产。

## 发布后检查

1. Release 标记为 Prerelease；
2. 两个 ZIP 和 `SHA256SUMS.txt` 都存在；
3. Release 说明同时包含中文和英文；
4. 下载后运行：

```bash
sha256sum -c SHA256SUMS.txt
```

5. 解压两个包并分别运行包内校验；
6. 确认 Windows 和 Wine 包没有被错误标记为可互换；
7. 将 Release 地址记录到后续 BMCBL 依赖清单。
