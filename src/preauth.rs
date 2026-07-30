// SPDX-License-Identifier: LGPL-2.1-or-later

use std::fmt;

use serde::Deserialize;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::ProxyError;

pub const PREAUTH_SCHEMA_VERSION: u32 = 1;
pub const MAX_PREAUTH_LIFETIME_SECONDS: u64 = 24 * 60 * 60;
pub const CLOCK_SKEW_SECONDS: u64 = 5 * 60;
pub const MIN_TOKEN_REMAINING_SECONDS: u64 = 30;

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
pub struct PreauthDocument {
    pub schema_version: u32,
    pub profile_id: String,
    pub launch_nonce: String,
    pub issued_at_epoch: u64,
    pub expires_at_epoch: u64,
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
            || self.expires_at_epoch
                <= now_epoch.saturating_add(MIN_TOKEN_REMAINING_SECONDS)
        {
            return Err(ProxyError::PreauthExpired);
        }

        if self.xbox.xuid.is_empty() || !self.xbox.xuid.bytes().all(|byte| byte.is_ascii_digit()) {
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
        || token.user_hash.trim().is_empty()
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
