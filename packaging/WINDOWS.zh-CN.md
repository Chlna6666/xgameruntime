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

## 环境要求

- Windows x64；
- 已安装 Microsoft Gaming Services；
- 可以定位 Microsoft 原生 `xgameruntime.dll`；
- 启动器能够按协议设置 BMCBL 进程级环境变量并完成 DLL 重定向/注入。

## 重要说明

该 DLL 是**单个游戏进程使用的实验性代理**，不是 Gaming Services 的系统级替代品。

禁止：

- 覆盖 Microsoft Gaming Services 安装目录中的 DLL；
- 把该 DLL 注册为系统全局运行时；
- 在缺少原生 Runtime 路径时直接启动游戏；
- 把 Microsoft refresh token 或账号密码写入预认证 JSON。

BMCBL 必须通过以下变量传入 Microsoft 原生运行时绝对路径：

```text
BMCBL_NATIVE_XGAMERUNTIME=<absolute path>
```

自定义 XUser 默认关闭。只有提供完整且有效的 schema v2 启动上下文，并显式设置以下变量后才启用：

```text
BMCBL_XGAMERUNTIME_ENABLE_XUSER=1
```

## 当前状态

Windows DLL 已通过：

- `cargo check --all-targets`；
- `cargo test --all-targets`；
- Windows CNG 临时 P-256 密钥创建、签名与验签测试；
- `x86_64-pc-windows-msvc` Release 构建。

但 Minecraft Bedrock GDK 的登录、好友、多人联机、Realms 和 Marketplace 尚未完成端到端实机验证，因此该包仍标记为实验性预发布。
