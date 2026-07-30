# Windows x64 发布包

本压缩包包含面向 Windows 原生 Minecraft Bedrock GDK 的 Rust/MSVC `xgameruntime.dll` 进程级代理。

[English](README.md) | **简体中文**

## 文件内容

- `xgameruntime.dll`：Rust 原生代理 DLL；
- `BMCBL_PROTOCOL.md`：英文版启动器 → DLL 进程协议；
- `BMCBL_PROTOCOL.zh-CN.md`：简体中文版启动协议；
- `preauth-v2.schema.json`：机器可读的预认证 JSON Schema；
- `LICENSE`、`NOTICE.md`：许可证与来源声明；
- `manifest.json`：构建版本、目标架构与源提交；
- `SHA256SUMS`：包内文件 SHA-256 校验值。

发布包不会重新分发 Microsoft 官方运行时。

## 代理文件布局

游戏进程使用的目录应包含：

```text
xgameruntime.dll    # 本项目生成的代理 DLL，由游戏加载
xgameruntime_o.dll  # Microsoft 官方 xgameruntime.dll 的进程级副本，重命名后使用
```

代理在 `DllMain(DLL_PROCESS_ATTACH)` 中按照 C 版本的方式同步执行：

```c
LoadLibraryA("xgameruntime_o.dll");
```

同时调用 `DisableThreadLibraryCalls` 关闭线程级 DLL 通知。代理被显式卸载时，通过 `FreeLibrary` 释放原生 DLL。若启动阶段第一次预加载失败，不会永久缓存失败结果；后续需要转发 API 时仍会再次尝试加载 `xgameruntime_o.dll`。

`BMCBL_NATIVE_XGAMERUNTIME` 继续作为可选的进程级绝对路径覆盖项。未设置该变量时，默认使用上述 `xgameruntime_o.dll` 代理布局。

## 环境要求

- Windows x64；
- 已安装 Microsoft Gaming Services；
- 准备一份 Microsoft 官方 `xgameruntime.dll` 的进程级副本，并命名为 `xgameruntime_o.dll`；
- 需要自定义 XUser 时，启动器按 `BMCBL_PROTOCOL.md` 设置对应环境变量。

## 重要说明

该 DLL 是**单个游戏进程使用的实验性代理**，不是 Gaming Services 的系统级替代品。

禁止：

- 覆盖或重命名 Microsoft Gaming Services 安装目录中的 DLL；
- 把代理 DLL 注册为系统全局运行时；
- 把 Microsoft refresh token 或账号密码写入预认证 JSON。

应当把官方 DLL 复制到游戏使用的进程级目录，再重命名为 `xgameruntime_o.dll`，不要修改系统安装目录。

自定义 XUser 默认关闭。只有提供完整且有效的 schema v2 启动上下文，并显式设置以下变量后才启用：

```text
BMCBL_XGAMERUNTIME_ENABLE_XUSER=1
```

若官方 DLL 无法加载，代理仍按 C 版本行为完成自身装载，但需要原生转发的 API 会明确失败，不会伪装为加载成功。

## 当前状态

Windows DLL 通过 CI 执行：

- `cargo fmt --all -- --check`；
- `cargo check --all-targets`；
- `cargo test --all-targets`；
- Windows CNG 临时 P-256 密钥创建、签名与验签测试；
- `x86_64-pc-windows-msvc` Release 构建。

Minecraft Bedrock GDK 的登录、好友、多人联机、Realms 和 Marketplace 仍需继续进行端到端实机验证，因此该包仍标记为实验性预发布。
