# BMCBL → xgameruntime launch protocol

This document defines the process-level contract between Better Minecraft Bedrock Launcher and the Rust `xgameruntime.dll` proxy.

## Windows proxy layout

The default Windows deployment uses two process-local DLLs:

```text
xgameruntime.dll    # Rust proxy loaded by Minecraft
xgameruntime_o.dll  # copy of Microsoft's original xgameruntime.dll
```

The proxy synchronously attempts to load `xgameruntime_o.dll` from `DllMain(DLL_PROCESS_ATTACH)`. A failed preload is not cached permanently; later forwarded API calls retry the load.

Do not modify, overwrite, or rename files in the Microsoft Gaming Services installation. BMCBL should copy the original runtime into the game process layout and rename the copy to `xgameruntime_o.dll`.

## Security boundary

BMCBL owns long-lived account credentials. The DLL must never receive or persist:

- Microsoft refresh tokens;
- OAuth authorization codes or device codes;
- account passwords;
- exported reusable private device-key bytes.

The DLL receives only a selected profile identifier, short-lived Xbox pre-authentication material for one game process, and an optional Windows CNG key name. The P-256 private key remains non-exported in the current user's CNG key store.

## Environment variables

BMCBL may set these variables before creating the Minecraft process:

| Variable | Required | Description |
| --- | --- | --- |
| `BMCBL_NATIVE_XGAMERUNTIME` | No | Optional absolute-path override for a process-local copy of Microsoft's runtime. If absent, the proxy loads sibling `xgameruntime_o.dll`. |
| `BMCBL_XGAMERUNTIME_PROFILE` | For custom profile | Stable BMCBL profile ID using only ASCII letters, digits, `.`, `_`, and `-`. |
| `BMCBL_XGAMERUNTIME_PREAUTH` | For custom profile | Absolute path to the versioned pre-authentication JSON file. |
| `BMCBL_XGAMERUNTIME_NONCE` | Recommended | Per-launch random nonce that must match the JSON document. |
| `BMCBL_XGAMERUNTIME_ENABLE_XUSER` | For custom XUser | Set to `1` to enable the experimental Rust XUser provider. If absent, all APIs are forwarded to the native runtime. |

For pure native forwarding, no BMCBL environment variables are required when `xgameruntime_o.dll` is present beside the proxy.

If custom-profile variables are absent, the DLL operates as a native proxy. If they are incomplete or invalid, custom XUser interception remains disabled and calls continue to the Microsoft runtime.

## Pre-authentication schema v2

Example with credentials redacted:

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

Unknown JSON fields are rejected. The document must match the selected profile and optional launch nonce, have a valid time range, and contain tokens that remain valid for at least 30 seconds.

`device_signing` is optional. When omitted, `XUserGetTokenAndSignature*` returns the Xbox authorization token with an empty signature. When present:

- `cng_key_name` must begin with `BMCBL.XboxDevice.`;
- only ASCII letters, digits, `.`, `_`, and `-` are allowed;
- the named key must be an ECDSA P-256 key in Microsoft Software Key Storage Provider;
- the key must be accessible to the Minecraft process under the same Windows user;
- signing failures are returned to the caller and are not silently downgraded to unsigned requests.

## Device signing lifecycle

BMCBL owns device enrollment and CNG key creation. The intended lifecycle is:

1. create or open a persistent current-user P-256 key named `BMCBL.XboxDevice.<profile-id>`;
2. obtain its public JWK (`x` and `y`) without exporting the private scalar;
3. use that proof key and a stable device ID during Xbox device/SISU authentication;
4. cache device-related credentials in BMCBL's protected account store;
5. pre-mint the short-lived relying-party tokens required for launch;
6. place only the CNG key name and short-lived tokens in the per-launch schema-v2 document;
7. keep the same key and device ID for later launches unless the user explicitly resets device identity.

The DLL opens the named key with `NCryptOpenKey` and signs request hashes with `NCryptSignHash`. It does not create, export, delete, rotate, or back up the private key.

## Xbox request signature format

For a configured CNG key, the DLL follows the Xbox proof-of-possession format used by WineGDK:

1. concatenate, with NUL separators:
   - big-endian policy version `1`;
   - Windows FILETIME timestamp;
   - uppercase HTTP method;
   - URL path and query, excluding the fragment;
   - the `XBL3.0 x=<uhs>;<token>` authorization value;
   - endpoint-policy header values, when defined;
   - opaque request body;
2. compute SHA-256;
3. sign the digest with ECDSA P-256, producing 64-byte P1363 `r || s`;
4. encode `version || timestamp || signature` as a 76-byte structure;
5. return standard padded Base64 in the GDK result structure.

The current XUser implementation signs no additional policy headers, matching the WineGDK default path. Caller headers are validated but are not automatically included unless an endpoint policy explicitly selects them.

## File handling requirements

BMCBL should:

1. create the JSON in a profile-private temporary directory;
2. apply an ACL that grants access only to the current user and launched process context;
3. write to a temporary file and atomically rename it into place;
4. set environment variables only for the Minecraft child process;
5. delete the file after the Minecraft process exits;
6. never log token values or the complete signature input, and never include the file in crash reports.

A later protocol revision should pass a duplicated read-only file handle instead of a path to reduce path-substitution and race risks.

## Native runtime forwarding

The default forwarding target is sibling `xgameruntime_o.dll`. `BMCBL_NATIVE_XGAMERUNTIME` may override it with a validated absolute path.

The proxy never modifies the Gaming Services installation and never intentionally loads itself as the forwarding target. Only validated XUser runtime-class/interface GUIDs are intercepted when the explicit XUser feature gate is enabled. All other runtime classes and native exports remain owned by the Microsoft runtime.

## Implemented XUser surface

The Rust provider currently implements:

- `IUnknown` routing for the XUser interface family;
- user-handle duplication, close, and compare semantics;
- `XUserAddAsync` and `XUserAddResult` through the native Microsoft XAsync state machine;
- XUID and local-ID lookup;
- guest, state, and age-group queries;
- privilege lookup from the pre-authenticated claim;
- ANSI gamertag lookup;
- ANSI and UTF-16 TokenAndSignature requests;
- URL-to-relying-party selection for Xbox Live, Multiplayer, Realms, PlayFab/SISU, and Licensing;
- optional CNG Xbox proof-of-possession signatures.

Unimplemented XUser methods remain explicit stubs. The provider remains experimental until Minecraft Bedrock GDK startup, friends, multiplayer, Realms, and Marketplace are validated end to end.
