# Changelog / 变更记录

本项目在稳定版本前使用实验性预发布版本号。Windows 与 Wine 产物是不同实现，不能互换。

This project uses experimental prerelease versions before the first stable release. Windows and Wine artifacts are different implementations and are not interchangeable.

## v0.1.0-beta.1

### 中文

首个公开双运行时预发布：

- 新增 Rust/MSVC Windows x64 `xgameruntime.dll` 进程级代理；
- 实现 Microsoft 原生 Runtime 绝对路径转发；
- 实现实验性 `IXUserImpl`、`IXUserGamertag` 和 50 槽 vtable；
- 复用原生 `IXThreadingImpl` 和 XAsync 状态机；
- 实现 `XUserAddAsync`、XUID、Gamertag、Privilege 和 TokenAndSignature；
- 新增 Xbox Live、Multiplayer、Realms、PlayFab/SISU 和 Licensing Token 路由；
- 新增 Windows CNG P-256 Xbox 请求签名；
- 新增 schema v2、机器可读 JSON Schema 和严格输入校验；
- 新增 Token、摘要、Body 和结果缓冲区清零；
- 新增固定 WineGDK 提交构建的 Wine x64 模块；
- 新增 Windows/Wine 双 ZIP 打包、manifest、来源声明和 SHA-256；
- 新增完整英文和简体中文文档；
- 新增 Windows/Ubuntu CI 与 Windows CNG 建钥、签名、验签集成测试。

已知限制：

- 尚未接入 BMCBL 账号库和启动流程；
- 尚未实现 BMCBL CNG 建钥、JWK 导出和 Xbox Device/SISU 注册；
- 尚未完成 Minecraft 登录、好友、多人联机、Realms、Marketplace 和 XSAPI 端到端验证；
- Windows 自定义 XUser 默认关闭；
- Wine 包应与匹配的 WineGDK/GDK-Proton 源码版本一起使用。

### English

First public dual-runtime prerelease:

- adds a Rust/MSVC Windows x64 per-process `xgameruntime.dll` proxy;
- forwards unrelated APIs to an absolute-path Microsoft native runtime;
- implements experimental `IXUserImpl`, `IXUserGamertag`, and a 50-slot vtable;
- reuses native `IXThreadingImpl` and XAsync state management;
- implements `XUserAddAsync`, XUID, gamertag, privilege, and TokenAndSignature surfaces;
- routes Xbox Live, Multiplayer, Realms, PlayFab/SISU, and Licensing tokens;
- adds Windows CNG P-256 Xbox request signing;
- adds schema v2, a machine-readable JSON Schema, and strict input validation;
- zeroizes token, digest, request-body, and result buffers;
- adds a Wine x64 module built from a pinned WineGDK commit;
- adds Windows/Wine ZIP packaging, manifests, provenance, and SHA-256 checksums;
- adds complete English and Simplified Chinese documentation;
- adds Windows/Ubuntu CI and a Windows CNG create/sign/verify integration test.

Known limitations:

- BMCBL account-store and launch integration are not implemented;
- BMCBL CNG key creation, JWK export, and Xbox device/SISU enrollment are not implemented;
- Minecraft sign-in, friends, multiplayer, Realms, Marketplace, and XSAPI have not been validated end to end;
- Windows custom XUser interception is disabled by default;
- the Wine package should be used with a matching WineGDK/GDK-Proton source revision.
