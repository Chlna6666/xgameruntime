# xgameruntime

[English](README.md) | **简体中文**

面向 Minecraft Bedrock GDK 的 `xgameruntime.dll` 兼容、代理和账号桥接项目。

本仓库维护两种不同产物：

- **Windows 原生版**：Rust/MSVC 构建的进程级中间人代理 DLL，支持 Microsoft 官方 Runtime 转发、BMCBL Profile、短期 Xbox 预认证和可选 CNG 请求签名；
- **Wine 版**：从固定 WineGDK 源码提交构建的 Wine `xgameruntime` 模块，用于匹配的 WineGDK/GDK-Proton 环境。

> 当前状态：实验阶段。Windows 代理加载链、原生 API 转发、XUser、XAsync、TokenAndSignature 和 CNG 签名已经实现，但 Minecraft 登录、好友、多人联机、Realms、Marketplace 等仍需端到端实机验证。

## Windows 代理架构

Windows 版以同名 `xgameruntime.dll` 作为游戏加载入口。Microsoft 官方 DLL 使用进程级副本并命名为 `xgameruntime_o.dll`：

```text
Minecraft Bedrock GDK
        │
        ▼
xgameruntime.dll（Rust 中间人代理）
        ├─ 已验证的 XUser GUID → Rust IXUserImpl
        └─ 其他 Runtime/API     → xgameruntime_o.dll
```

默认文件布局：

```text
<游戏运行目录>/
├─ Minecraft.Windows.exe
├─ xgameruntime.dll      # 本项目生成的代理 DLL
└─ xgameruntime_o.dll    # Microsoft 官方 xgameruntime.dll 的副本
```

不要覆盖或重命名 Microsoft Gaming Services 安装目录中的原文件。应当复制官方 DLL 到游戏使用的进程级目录，再命名为 `xgameruntime_o.dll`。

## Windows preload 加载链

代理按照参考 C 版本的行为实现 Windows `DllMain`：

1. 游戏加载 `xgameruntime.dll`；
2. `DllMain(DLL_PROCESS_ATTACH)` 调用 `DisableThreadLibraryCalls`；
3. 同步调用 `LoadLibraryA("xgameruntime_o.dll")`；
4. `QueryApiImpl` 先尝试 Rust XUser 中间人拦截；
5. 未拦截接口通过 `GetProcAddress` 转发到官方 DLL；
6. 显式卸载代理时，通过 `FreeLibrary` 释放官方 DLL。

若 attach 阶段第一次预加载失败，代理仍按 C 版本返回成功；失败结果不会永久缓存，后续需要原生转发时会再次尝试加载 `xgameruntime_o.dll`。如果仍无法加载，需要原生 Runtime 的 API 会返回失败，不会伪装为成功。

## 可选原生 DLL 路径覆盖

默认情况下不需要设置原生 Runtime 路径。代理直接加载同目录：

```text
xgameruntime_o.dll
```

如启动器需要使用其他进程级副本，可设置可选绝对路径覆盖：

```text
BMCBL_NATIVE_XGAMERUNTIME=<Microsoft 官方 DLL 副本的绝对路径>
```

该变量仅对当前游戏子进程生效，不应写入系统全局环境。

## Windows 原生版已实现

- Windows x64 MSVC `cdylib`，输出 `xgameruntime.dll`；
- 与 WineGDK 一致的导出名称和 ordinal；
- C 风格 `DllMain` preload 和 `xgameruntime_o.dll` 代理布局；
- 原生导出的动态查找和转发；
- `QueryApiImpl` 中间人拦截；
- BMCBL Profile 与严格的预认证 schema v2；
- Windows x64 `IXUserImpl` 50 槽 vtable 和 `IXUserGamertag`；
- XUser Handle、XUID、Local ID、登录状态、年龄组和权限查询；
- 复用 Microsoft 原生 `IXThreadingImpl` 和 XAsync 状态机；
- `XUserAddAsync` / `XUserAddResult`；
- ANSI 与 UTF-16 `XUserGetTokenAndSignature*`；
- Xbox Live、Multiplayer、Realms、PlayFab/SISU 和 Licensing 的 Relying Party 路由；
- 可选 Windows CNG P-256 Xbox Proof-of-Possession 签名；
- Token、Authorization、摘要、Body 和结果缓冲区清零；
- Windows/Ubuntu CI、Windows CNG 建钥/签名/验签测试和 MSVC Release 构建。

## BMCBL 自定义 XUser 启动协议

仅使用纯代理转发时，不需要提供 Profile 环境变量。

启用 BMCBL 自定义 XUser 时，游戏子进程需要设置：

```text
BMCBL_XGAMERUNTIME_PROFILE=<Profile ID>
BMCBL_XGAMERUNTIME_PREAUTH=<schema-v2 JSON 的绝对路径>
BMCBL_XGAMERUNTIME_NONCE=<每次启动随机 nonce>
BMCBL_XGAMERUNTIME_ENABLE_XUSER=1
```

长期 Microsoft refresh token、账号密码和 OAuth 授权码禁止传给 DLL。DLL 只接收一个游戏进程使用的短期 Xbox 预认证材料和可选 CNG key name。

完整协议：

- [BMCBL 协议（中文）](docs/BMCBL_PROTOCOL.zh-CN.md)
- [BMCBL Protocol（English）](docs/BMCBL_PROTOCOL.md)
- [预认证 JSON Schema](docs/preauth-v2.schema.json)

## 构建 Windows 原生版

首要目标：

```text
x86_64-pc-windows-msvc
```

构建：

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

输出：

```text
target/x86_64-pc-windows-msvc/release/xgameruntime.dll
```

部署时还需要自行准备 Microsoft 官方 DLL 的进程级副本：

```text
xgameruntime_o.dll
```

官方 DLL 不包含在本项目发布包中。

## 打包与版本

当前版本：

```text
v0.1.0-beta.2
```

发布工作流生成：

```text
xgameruntime-<version>-windows-x64.zip
xgameruntime-<version>-wine-x64.zip
```

Windows 包包含代理 DLL、协议、Schema、许可证、构建 manifest 和 SHA-256，不重新分发 Microsoft 官方 `xgameruntime.dll`。

## Wine 版

Wine 版不是 Windows Rust 代理，不使用 Windows `DllMain` preload、Microsoft Gaming Services、Windows CNG 或 `xgameruntime_o.dll` 布局。

当前固定源码：

```text
仓库：https://github.com/Chlna6666/WineGDK
提交：75637b674e1f191e65753663c4c0c32bea05ba6e
路径：dlls/xgameruntime
```

Wine 模块应与相同或已验证兼容的 WineGDK/GDK-Proton 运行时一起使用。

## 安全与限制

- 禁止覆盖 Microsoft Gaming Services 系统安装；
- 禁止把代理注册为系统全局 Runtime；
- 禁止把 refresh token、账号密码或可复用私钥写入 DLL 配置；
- 自定义 XUser 默认关闭；
- 未实现方法保持显式失败或 Stub；
- Minecraft 登录、好友、多人联机、Realms、Marketplace 和 XSAPI 仍需端到端验证。

## 许可证

本项目使用 **GNU LGPL-2.1-or-later**，保留 Wine/WineGDK 来源和许可证边界。详见 [LICENSE](LICENSE) 与 [NOTICE.md](NOTICE.md)。
