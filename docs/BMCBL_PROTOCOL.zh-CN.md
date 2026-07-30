# BMCBL → xgameruntime 启动协议

[English](BMCBL_PROTOCOL.md) | **简体中文**

本文定义 Better Minecraft Bedrock Launcher 与 Rust `xgameruntime.dll` 代理之间的进程级契约。

## 安全边界

长期账号凭据由 BMCBL 管理。DLL 禁止接收或持久化：

- Microsoft refresh token；
- OAuth authorization code 或 device code；
- 账号密码；
- 可重复使用的已导出设备私钥字节。

DLL 只接收：

- 当前选择的 Profile ID；
- 仅供一个游戏进程使用的短期 Xbox 预认证数据；
- 可选的 Windows CNG 密钥名称。

P-256 私钥必须保持为当前用户 CNG Key Store 中的不可导出密钥。

## 环境变量

BMCBL 在创建 Minecraft 进程前设置：

| 变量 | 是否必需 | 说明 |
| --- | --- | --- |
| `BMCBL_NATIVE_XGAMERUNTIME` | 是 | Microsoft 原生 `xgameruntime.dll` 的绝对路径，代理把未拦截接口转发到该 DLL。 |
| `BMCBL_XGAMERUNTIME_PROFILE` | 自定义 Profile 时 | 稳定的 BMCBL Profile ID，只允许 ASCII 字母、数字、`.`、`_` 和 `-`。 |
| `BMCBL_XGAMERUNTIME_PREAUTH` | 自定义 Profile 时 | 带版本号的预认证 JSON 绝对路径。 |
| `BMCBL_XGAMERUNTIME_NONCE` | 建议 | 每次启动生成的随机 nonce，必须与 JSON 内容一致。 |
| `BMCBL_XGAMERUNTIME_ENABLE_XUSER` | 自定义 XUser 时 | 设置为 `1` 启用实验性 Rust XUser Provider。未设置时全部 API 转发到原生运行时。 |

未提供自定义 Profile 变量时，DLL 作为纯原生代理运行。变量不完整或校验失败时，自定义 XUser 拦截保持关闭，并继续使用 Microsoft 原生运行时。

## 预认证 schema v2

脱敏示例：

```json
{
  "schema_version": 2,
  "profile_id": "account-2535458430309376",
  "launch_nonce": "a-random-value-generated-for-this-launch",
  "issued_at_epoch": 1785390000,
  "expires_at_epoch": 1785393300,
  "device_signing": {
    "cng_key_name": "BMCBL.XboxDevice.account-2535458430309376"
  },
  "xbox": {
    "xuid": "2535458430309376",
    "gamertag": "ExamplePlayer",
    "age_group": "Adult",
    "privileges": [185, 188, 189, 203, 252, 254],
    "user": {
      "token": "[REDACTED]",
      "user_hash": "1234567890",
      "relying_party": "http://auth.xboxlive.com",
      "expires_at_epoch": 1785393300
    },
    "xbox_live": {
      "token": "[REDACTED]",
      "user_hash": "1234567890",
      "relying_party": "http://xboxlive.com",
      "expires_at_epoch": 1785393300
    },
    "sisu": {
      "token": "[REDACTED]",
      "user_hash": "1234567890",
      "relying_party": "https://b980a380.minecraft.playfabapi.com/",
      "expires_at_epoch": 1785393300
    },
    "multiplayer": {
      "token": "[REDACTED]",
      "user_hash": "1234567890",
      "relying_party": "https://multiplayer.minecraft.net/",
      "expires_at_epoch": 1785393300
    },
    "realms": {
      "token": "[REDACTED]",
      "user_hash": "1234567890",
      "relying_party": "https://pocket.realms.minecraft.net/",
      "expires_at_epoch": 1785393300
    },
    "licensing": {
      "token": "[REDACTED]",
      "user_hash": "1234567890",
      "relying_party": "http://licensing.xboxlive.com",
      "expires_at_epoch": 1785393300
    }
  }
}
```

未知 JSON 字段会被拒绝。文档必须：

- 与当前选择的 Profile ID 一致；
- 与可选启动 nonce 一致；
- 具有合法的签发时间和失效时间范围；
- 所有 Token 至少还剩 30 秒有效期。

`device_signing` 为可选字段。省略时，`XUserGetTokenAndSignature*` 返回 Xbox Authorization Token，Signature 为空。提供该字段时：

- `cng_key_name` 必须以 `BMCBL.XboxDevice.` 开头；
- 只允许 ASCII 字母、数字、`.`、`_` 和 `-`；
- 指定密钥必须是 Microsoft Software Key Storage Provider 中的 ECDSA P-256 密钥；
- Minecraft 进程必须以同一 Windows 用户身份访问该密钥；
- 签名失败必须返回给调用方，禁止静默降级成无签名请求。

## 设备签名生命周期

设备注册和 CNG 密钥创建由 BMCBL 管理：

1. 创建或打开当前用户持久 P-256 密钥，命名为 `BMCBL.XboxDevice.<profile-id>`；
2. 获取公开 JWK 的 `x` 和 `y`，不得导出私钥标量；
3. Xbox Device/SISU 认证时使用该 Proof Key 和稳定 Device ID；
4. 设备相关凭据存入 BMCBL 受保护的账号存储；
5. 游戏启动前生成所需 Relying Party 的短期 Token；
6. 每次启动 schema-v2 文件只包含 CNG key name 与短期 Token；
7. 后续启动继续使用同一密钥和 Device ID，除非用户明确重置设备身份。

DLL 使用 `NCryptOpenKey` 打开密钥，使用 `NCryptSignHash` 对请求摘要签名。DLL 不负责：

- 创建密钥；
- 导出私钥；
- 删除密钥；
- 轮换密钥；
- 备份私钥。

## Xbox 请求签名格式

配置 CNG 密钥后，DLL 按 WineGDK 使用的 Xbox Proof-of-Possession 格式处理：

1. 以下字段使用 NUL 分隔后拼接：
   - 大端序策略版本 `1`；
   - Windows FILETIME 时间戳；
   - 转成大写的 HTTP Method；
   - URL Path 和 Query，不包含 Fragment；
   - `XBL3.0 x=<uhs>;<token>` Authorization 值；
   - Endpoint Policy 指定的 Header Value；
   - 原始 Request Body；
2. 计算 SHA-256；
3. 使用 ECDSA P-256 签名，输出 64 字节 P1363 `r || s`；
4. 编码为 `version || timestamp || signature`，总长度 76 字节；
5. 在 GDK 结果结构中返回标准带 Padding 的 Base64。

当前 XUser 实现不额外签入 Policy Header，与 WineGDK 默认路径一致。调用方 Header 会执行合法性校验，但只有 Endpoint Policy 明确选择时才会加入签名输入。

## 文件处理要求

BMCBL 应当：

1. 在 Profile 私有临时目录创建 JSON；
2. 设置 ACL，只允许当前用户和目标进程上下文访问；
3. 先写临时文件，再通过原子重命名替换目标文件；
4. 环境变量只设置给 Minecraft 子进程；
5. Minecraft 退出后删除文件；
6. 禁止记录 Token、完整签名输入，也不得把该文件加入崩溃报告。

后续协议版本应考虑传递复制后的只读文件句柄，而不是路径，以减少路径替换和竞态风险。

## 原生运行时转发

代理不会自行扫描 Gaming Services。Microsoft 原生运行时路径由 BMCBL 解析后以绝对路径传入，从而避免：

- 代理 DLL 递归加载自身；
- 修改系统 Gaming Services；
- 依赖硬编码的包版本或安装目录。

只有经过验证的 XUser Runtime Class/Interface GUID 在显式开启 XUser Feature Gate 时被拦截。其他 Runtime Class 仍由 Microsoft 原生运行时处理。

## 当前实现的 XUser 接口

Rust Provider 当前实现：

- XUser 接口族的 `IUnknown` 路由；
- User Handle 的复制、关闭和比较语义；
- 通过 Microsoft 原生 XAsync 状态机实现 `XUserAddAsync` 和 `XUserAddResult`；
- XUID 和 Local ID 查询；
- Guest、State 和 Age Group 查询；
- 从预认证 Claim 查询权限；
- ANSI Gamertag 查询；
- ANSI 和 UTF-16 TokenAndSignature；
- Xbox Live、Multiplayer、Realms、PlayFab/SISU 和 Licensing 的 URL → Relying Party 选择；
- 可选的 CNG Xbox Proof-of-Possession Signature。

未实现的 XUser 方法保持显式 Stub。只有在 Minecraft Bedrock GDK 启动、好友、多人联机、Realms 和 Marketplace 端到端验证完成后，Provider 才能视为稳定。
