# xgameruntime

[English](README.md) | **简体中文**

面向 Minecraft Bedrock GDK 的 `xgameruntime.dll` 兼容与代理项目。

本仓库同时维护两种不同的运行时产物：

- **Windows 原生版**：使用 Rust/MSVC 构建的进程级代理 DLL，用于接入 BMCBL 管理的账号 Profile、短期 Xbox 预认证数据和 CNG 请求签名。
- **Wine 版**：从固定 WineGDK 源码提交构建的 Wine `xgameruntime` 模块，用于 WineGDK/GDK-Proton 环境。

> 当前状态：实验阶段。Windows 版已经实现原生运行时转发、Profile 校验、XUser 身份、原生 XAsync、TokenAndSignature 和可选 CNG 请求签名；Wine 版从 WineGDK 固定源码构建。BMCBL 集成和 Minecraft 端到端实机验证尚未完成，不建议作为稳定生产组件使用。

## 架构

Windows 原生版只接管账号相关的 XUser 接口，其余 GDK Runtime Class 继续转发给 BMCBL 指定的 Microsoft 原生运行时。

```text
Minecraft Bedrock GDK
        │
        ▼
xgameruntime.dll（Rust 代理）
        ├─ 已验证的 XUser GUID → Rust IXUserImpl
        └─ 其他全部 API         → Microsoft 原生 xgameruntime.dll
```

Rust XUser Provider 复用 Microsoft 原生 `IXThreadingImpl` 提供的：

- `XAsyncBegin`
- `XAsyncSchedule`
- `XAsyncComplete`
- `XAsyncGetResult`

项目不会自行写入私有 `XAsyncBlock` 状态。

## Windows 原生版已实现

- Windows MSVC `cdylib`，输出 `xgameruntime.dll`；
- 与 WineGDK 一致的导出名称和 ordinal；
- 通过绝对路径延迟加载并转发 Microsoft 原生运行时；
- BMCBL Profile 和严格的预认证 schema v2；
- Profile ID、启动 nonce、有效期、XUID、UHS、Relying Party 与 CNG key name 校验；
- Token、Authorization、摘要、请求 Body 和结果缓冲区脱敏/清零；
- 显式开关控制的 XUser Runtime Class/Interface 拦截；
- Windows x64 `IXUserImpl` 50 槽 vtable 与 `IXUserGamertag`；
- XUser Handle、XUID、Local ID、登录状态、年龄组和权限查询；
- 基于原生 XAsync 状态机的 `XUserAddAsync` / `XUserAddResult`；
- ANSI 和 UTF-16 `XUserGetTokenAndSignature*`；
- Xbox Live、Multiplayer、Realms、PlayFab/SISU 和 Licensing 的 URL → Relying Party 路由；
- 从启动器短期预认证生成 `XBL3.0 x=<uhs>;<token>`；
- 使用当前用户 CNG P-256 持久密钥生成 Xbox Proof-of-Possession Signature；
- Windows CNG 集成测试：创建临时 P-256 密钥、签名、验签并删除密钥；
- Windows/Ubuntu CI；
- LGPL-2.1-or-later 许可证和 Wine/WineGDK 来源声明。

## Wine 版

Wine 版不是 Windows Rust 代理，也不使用 Windows CNG 或 BMCBL schema v2。

发布工作流从以下固定源码构建：

```text
仓库：https://github.com/Chlna6666/WineGDK
提交：75637b674e1f191e65753663c4c0c32bea05ba6e
路径：dlls/xgameruntime
```

Wine 内置模块通常和特定 Wine 构建树存在 ABI/布局耦合，因此应与相同或兼容源码版本的 WineGDK/GDK-Proton 一起使用，不保证能直接覆盖任意系统 Wine。

## BMCBL 启动协议

BMCBL 通过进程级环境变量提供原生运行时路径和可选账号 Profile：

```text
BMCBL_NATIVE_XGAMERUNTIME=<Microsoft 原生 xgameruntime.dll 的绝对路径>
BMCBL_XGAMERUNTIME_PROFILE=<Profile ID>
BMCBL_XGAMERUNTIME_PREAUTH=<schema-v2 JSON 的绝对路径>
BMCBL_XGAMERUNTIME_NONCE=<每次启动随机 nonce>
BMCBL_XGAMERUNTIME_ENABLE_XUSER=1
```

长期 Microsoft refresh token 必须保留在 BMCBL 的安全凭据存储中，禁止写入 DLL 配置或预认证 JSON。

schema v2 可提供可选 CNG key name：

```json
{
  "device_signing": {
    "cng_key_name": "BMCBL.XboxDevice.account-2535458430309376"
  }
}
```

这里只传递密钥名称。P-256 私钥保留在 Windows 当前用户的 CNG Key Storage Provider 中，不导出到文件，也不通过每次请求 IPC 传输。

完整协议见：

- [BMCBL 协议（中文）](docs/BMCBL_PROTOCOL.zh-CN.md)
- [BMCBL Protocol（English）](docs/BMCBL_PROTOCOL.md)
- [预认证 JSON Schema](docs/preauth-v2.schema.json)

## Xbox 请求签名

启用 `device_signing` 时，DLL 会：

1. 从 Microsoft Software Key Storage Provider 打开指定密钥；
2. 按 WineGDK 规则构造 `policy version + FILETIME + uppercase method + request target + authorization + policy headers + body`；
3. 计算 SHA-256；
4. 使用 `NCryptSignHash` 和 ECDSA P-256 签名；
5. 返回 `version || timestamp || r || s` 共 76 字节结构的标准 Base64。

若配置了签名密钥但签名失败，请求会明确失败，不会静默降级成无签名结果。未配置 `device_signing` 时，TokenAndSignature 返回 Authorization Token，Signature 为空。

## 构建 Windows 原生版

首要支持目标：

```text
x86_64-pc-windows-msvc
```

构建命令：

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

输出：

```text
target/x86_64-pc-windows-msvc/release/xgameruntime.dll
```

XUser 拦截默认关闭。受控测试时必须提供完整且有效的 Profile 环境变量，并显式设置：

```text
BMCBL_XGAMERUNTIME_ENABLE_XUSER=1
```

## 打包与发布

工作流 `.github/workflows/package-release.yml` 会生成两个独立 ZIP：

```text
xgameruntime-<version>-windows-x64.zip
xgameruntime-<version>-wine-x64.zip
```

每个压缩包包含：

- DLL 产物；
- 中英文 README；
- `manifest.json`；
- 来源和许可证文件；
- 包内文件的 `SHA256SUMS`。

正式 GitHub Release 还会包含统一的 `SHA256SUMS.txt`，用于校验两个 ZIP。

Windows 与 Wine 产物不是同一个实现，不能互换：

- Windows 包不得覆盖系统全局 Gaming Services；
- Wine 包不得复制到 Windows Microsoft Gaming Services；
- Wine 包应通过匹配的 WineGDK/GDK-Proton 布局安装。

## CI 验证

Ubuntu 与 Windows 矩阵执行：

```text
cargo fmt --all -- --check
cargo check --all-targets
cargo test --all-targets
```

Windows 额外执行：

```text
cargo build --release --target x86_64-pc-windows-msvc
```

## 尚未完成

- BMCBL 账号存储接入；
- BMCBL CNG P-256 建钥和公开 JWK 导出；
- 稳定 Xbox Device ID 与 Device/SISU 注册；
- 每次启动生成 schema-v2 预认证文件；
- BMCBL 原生运行时路径解析、环境变量和 DLL 注入；
- `ForceRefresh` 与运行中 Token 刷新协调；
- Minecraft 实机启动、好友、多人联机、Realms 和 Marketplace 验证；
- 自定义 XUser Handle 与 XSAPI 的完整行为验证。

## 移植原则

不要机械翻译整个 WineGDK 目录。每个源文件必须先分类为：

1. 可移植 GDK ABI/接口逻辑；
2. 可复用 Xbox 认证或请求签名逻辑；
3. Wine 专用 Loader、Registry、TLS 或线程逻辑；
4. Windows 原生代理逻辑；
5. BMCBL 所有的 Profile 和凭据管理逻辑。

未支持的 Runtime Class 继续转发给 Microsoft 原生运行时。尚未验证的 XUser 方法保持显式 Stub，直到 ABI 和 Minecraft 调用路径确认。

## 开源许可证

本仓库整体采用 **GNU LGPL-2.1-or-later**，用于保持 Wine/WineGDK 衍生实现的授权边界。

来源：

- `Chlna6666/WineGDK`；
- 源路径：`dlls/xgameruntime`；
- 上游来源：WineGDK 与 Wine。

移植代码时必须保留已有版权与许可证声明。详细信息见 [LICENSE](LICENSE) 和 [NOTICE.md](NOTICE.md)。
