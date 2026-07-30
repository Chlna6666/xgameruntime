# BMCBL → xgameruntime launch protocol

This document defines the initial process-level contract between Better Minecraft Bedrock Launcher and the Rust `xgameruntime.dll` proxy.

## Security boundary

BMCBL owns long-lived account credentials. The DLL must never receive or persist:

- Microsoft refresh tokens;
- OAuth authorization codes or device codes;
- account passwords;
- reusable private device keys owned by the launcher.

The DLL receives only a selected profile identifier and short-lived Xbox pre-authentication material for one game process.

## Environment variables

BMCBL sets these variables before creating the Minecraft process:

| Variable | Required | Description |
| --- | --- | --- |
| `BMCBL_NATIVE_XGAMERUNTIME` | Yes | Absolute path to the original Microsoft `xgameruntime.dll` that the proxy forwards to. |
| `BMCBL_XGAMERUNTIME_PROFILE` | For custom profile | Stable BMCBL profile ID using only ASCII letters, digits, `.`, `_`, and `-`. |
| `BMCBL_XGAMERUNTIME_PREAUTH` | For custom profile | Absolute path to the versioned pre-authentication JSON file. |
| `BMCBL_XGAMERUNTIME_NONCE` | Recommended | Per-launch random nonce that must match the JSON document. |

If the custom profile variables are absent, the DLL operates as a native proxy. If they are incomplete or invalid, custom XUser interception remains disabled and calls continue to the native runtime.

## Pre-authentication schema v1

Example with credentials redacted:

```json
{
  "schema_version": 1,
  "profile_id": "account-2535458430309376",
  "launch_nonce": "a-random-value-generated-for-this-launch",
  "issued_at_epoch": 1785390000,
  "expires_at_epoch": 1785393300,
  "xbox": {
    "xuid": "2535458430309376",
    "gamertag": "ExamplePlayer",
    "age_group": "Adult",
    "privileges": [185, 186, 187],
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
    "sisu": null,
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

Unknown JSON fields are rejected. The document must match the selected profile and optional launch nonce, have a valid time range, and contain tokens that remain valid for at least 30 seconds.

## File handling requirements

BMCBL should:

1. create the JSON in a profile-private temporary directory;
2. apply an ACL that grants access only to the current user and launched process context;
3. write to a temporary file and atomically rename it into place;
4. set the environment variables only for the child process;
5. delete the file after the Minecraft process exits;
6. never log token values or include the file in crash reports.

A later protocol revision should pass a duplicated read-only file handle instead of a path to reduce path substitution and race risks.

## Native runtime forwarding

The proxy does not search the machine for Gaming Services. BMCBL resolves the original runtime and passes its absolute path. This avoids:

- loading the proxy recursively;
- modifying the system Gaming Services installation;
- depending on a hard-coded package version or installation directory.

## Planned XUser interception

The bootstrap forwards every `QueryApiImpl` request. A future implementation may intercept only validated XUser runtime class/interface GUID pairs. All other classes remain owned by the native Microsoft runtime.

Before enabling an XUser interface, the project requires tests for:

- exact GUID pairs and interface versions;
- vtable slot ordering and calling conventions;
- `IUnknown` lifetime and thread safety;
- `XAsync` provider/result behavior;
- XUID, gamertag, privilege and token audience consistency;
- fallback behavior when the pre-authentication set expires.
