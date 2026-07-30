// SPDX-License-Identifier: LGPL-2.1-or-later

use std::fmt;

use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::ProxyError;

pub const PREAUTH_SCHEMA_VERSION: u32 = 2;
pub const MAX_PREAUTH_LIFETIME_SECONDS: u64 = 24 * 60 * 60;
pub const CLOCK_SKEW_SECONDS: u64 = 5 * 60;
pub const MIN_TOKEN_REMAINING_SECONDS: u64 = 30;
pub const CNG_KEY_NAME_PREFIX: &str = "BMCBL.XboxDevice.";
pub const MAX_CNG_KEY_NAME_LENGTH: usize = 240;

#[derive(Clone, Deserialize, Zeroize, ZeroizeOnDrop)]
#[serde(transparent)]
pub struct SecretString(String);

impl SecretString {
    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretString([REDACTED])")
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenEnvelope {
    pub token: SecretString,
    pub user_hash: String,
    pub relying_party: String,
    pub expires_at_epoch: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XboxPreauth {
    pub xuid: String,
    pub gamertag: String,
    pub age_group: Option<String>,
    #[serde(default)]
    pub privileges: Vec<u32>,
    pub user: TokenEnvelope,
    pub xbox_live: TokenEnvelope,
    pub sisu: Option<TokenEnvelope>,
    pub multiplayer: Option<TokenEnvelope>,
    pub realms: Option<TokenEnvelope>,
    pub licensing: Option<TokenEnvelope>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceSigningPreauth {
    pub cng_key_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreauthDocument {
    pub schema_version: u32,
    pub profile_id: String,
    pub launch_nonce: String,
    pub issued_at_epoch: u64,
    pub expires_at_epoch: u64,
    pub device_signing: Option<DeviceSigningPreauth>,
    pub xbox: XboxPreauth,
}

impl PreauthDocument {
    pub fn validate(
        &self,
        expected_profile_id: &str,
        expected_nonce: Option<&str>,
        now_epoch: u64,
    ) -> Result<(), ProxyError> {
        if self.schema_version != PREAUTH_SCHEMA_VERSION {
            return Err(ProxyError::UnsupportedSchema(self.schema_version));
        }

        if self.profile_id != expected_profile_id {
            return Err(ProxyError::ProfileMismatch);
        }

        if let Some(expected_nonce) = expected_nonce {
            if self.launch_nonce != expected_nonce {
                return Err(ProxyError::ProfileMismatch);
            }
        }

        if self.issued_at_epoch > self.expires_at_epoch
            || self.expires_at_epoch.saturating_sub(self.issued_at_epoch)
                > MAX_PREAUTH_LIFETIME_SECONDS
        {
            return Err(ProxyError::InvalidTimeRange);
        }

        if self.issued_at_epoch > now_epoch.saturating_add(CLOCK_SKEW_SECONDS)
            || self.expires_at_epoch <= now_epoch.saturating_add(MIN_TOKEN_REMAINING_SECONDS)
        {
            return Err(ProxyError::PreauthExpired);
        }

        if let Some(device_signing) = &self.device_signing {
            validate_cng_key_name(&device_signing.cng_key_name)?;
        }

        if parse_nonzero_decimal_u64(&self.xbox.xuid).is_none() {
            return Err(ProxyError::InvalidToken("xuid"));
        }
        if self.xbox.gamertag.trim().is_empty() {
            return Err(ProxyError::InvalidToken("gamertag"));
        }

        validate_token("user", &self.xbox.user, None, now_epoch)?;
        validate_token(
            "xbox_live",
            &self.xbox.xbox_live,
            Some("http://xboxlive.com"),
            now_epoch,
        )?;
        validate_optional_token("sisu", self.xbox.sisu.as_ref(), None, now_epoch)?;
        validate_optional_token(
            "multiplayer",
            self.xbox.multiplayer.as_ref(),
            Some("https://multiplayer.minecraft.net/"),
            now_epoch,
        )?;
        validate_optional_token(
            "realms",
            self.xbox.realms.as_ref(),
            Some("https://pocket.realms.minecraft.net/"),
            now_epoch,
        )?;
        validate_optional_token(
            "licensing",
            self.xbox.licensing.as_ref(),
            Some("http://licensing.xboxlive.com"),
            now_epoch,
        )?;

        Ok(())
    }
}

fn validate_cng_key_name(key_name: &str) -> Result<(), ProxyError> {
    let Some(suffix) = key_name.strip_prefix(CNG_KEY_NAME_PREFIX) else {
        return Err(ProxyError::InvalidToken("device_signing"));
    };
    if suffix.is_empty()
        || key_name.len() > MAX_CNG_KEY_NAME_LENGTH
        || !key_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(ProxyError::InvalidToken("device_signing"));
    }
    Ok(())
}

fn validate_optional_token(
    name: &'static str,
    token: Option<&TokenEnvelope>,
    expected_relying_party: Option<&str>,
    now_epoch: u64,
) -> Result<(), ProxyError> {
    if let Some(token) = token {
        validate_token(name, token, expected_relying_party, now_epoch)?;
    }
    Ok(())
}

fn validate_token(
    name: &'static str,
    token: &TokenEnvelope,
    expected_relying_party: Option<&str>,
    now_epoch: u64,
) -> Result<(), ProxyError> {
    if token.token.is_empty()
        || parse_nonzero_decimal_u64(&token.user_hash).is_none()
        || token.relying_party.trim().is_empty()
        || token.expires_at_epoch <= now_epoch.saturating_add(MIN_TOKEN_REMAINING_SECONDS)
    {
        return Err(ProxyError::InvalidToken(name));
    }

    if let Some(expected) = expected_relying_party {
        if token.relying_party != expected {
            return Err(ProxyError::InvalidToken(name));
        }
    }

    Ok(())
}

fn parse_nonzero_decimal_u64(value: &str) -> Option<u64> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse::<u64>().ok().filter(|value| *value != 0)
}

#[cfg(test)]
mod tests {
    use super::{parse_nonzero_decimal_u64, validate_cng_key_name};

    #[test]
    fn validates_decimal_identifiers() {
        assert_eq!(
            parse_nonzero_decimal_u64("2535458430309376"),
            Some(2535458430309376)
        );
        assert_eq!(parse_nonzero_decimal_u64("0"), None);
        assert_eq!(parse_nonzero_decimal_u64("12x"), None);
        assert_eq!(parse_nonzero_decimal_u64("18446744073709551616"), None);
    }

    #[test]
    fn restricts_device_signing_key_names() {
        assert!(validate_cng_key_name("BMCBL.XboxDevice.account-1").is_ok());
        assert!(validate_cng_key_name("OtherProvider.account-1").is_err());
        assert!(validate_cng_key_name("BMCBL.XboxDevice.").is_err());
        assert!(validate_cng_key_name("BMCBL.XboxDevice.account/1").is_err());
    }
}
