# Wine x64 发布包

本压缩包包含从 WineGDK 构建的 `xgameruntime` Wine PE 模块，用于 WineGDK/GDK-Proton 环境中的 Minecraft Bedrock GDK。

[English](README.md) | **简体中文**

## 源码兼容性

该包从 `manifest.json` 和 `SOURCE.md` 记录的精确 WineGDK 提交构建。

Wine 内置模块可能与特定 Wine 构建树、生成头文件和运行时布局耦合，因此应配合相同或兼容源码版本的 WineGDK/GDK-Proton 使用，不保证与任意系统 Wine 版本 ABI 兼容。

## 文件内容

- `xgameruntime.dll`：Wine PE 模块；
- `xgameruntime.dll.so`：仅在当前 Wine 构建模式生成时包含；
- `LICENSE.winegdk`：WineGDK/Wine LGPL 许可证；
- `SOURCE.md`：精确来源仓库和提交；
- `manifest.json`：发布版本、架构和源码版本；
- `SHA256SUMS`：包内文件 SHA-256 校验值。

## 重要说明

该产物是 **Wine 实现**，不是 Windows 原生 Rust 代理。

它不会使用：

- Windows CNG 密钥存储；
- BMCBL schema v2 原生代理协议；
- Microsoft Windows Gaming Services 的原生 DLL 转发路径。

禁止：

- 把 Wine DLL 覆盖到 Windows Microsoft Gaming Services；
- 把它与 Windows 原生发布包互换；
- 在不匹配的系统 Wine 中直接覆盖模块而不验证 ABI；
- 删除 `LICENSE.winegdk` 或 `SOURCE.md`。

## 安装原则

应通过匹配的 WineGDK/GDK-Proton Runtime 布局或打包系统安装，例如由 Runtime 构建流程把 PE 模块和可选 Unix 模块放入对应的 `dlls/xgameruntime` / Wine DLL 搜索路径。

具体目标目录取决于所使用的 WineGDK/GDK-Proton 发行方式，本项目不会自动修改系统 Wine 或用户 Wine Prefix。

## 当前固定来源

发布工作流当前固定到：

```text
仓库：https://github.com/Chlna6666/WineGDK
提交：75637b674e1f191e65753663c4c0c32bea05ba6e
源路径：dlls/xgameruntime
```

每个发布包内的 `manifest.json` 和 `SOURCE.md` 会再次记录实际使用的提交。
