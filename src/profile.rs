// SPDX-License-Identifier: LGPL-2.1-or-later

use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{error::ProxyError, preauth::PreauthDocument};

pub const PROFILE_ID_ENV: &str = "BMCBL_XGAMERUNTIME_PROFILE";
pub const PREAUTH_PATH_ENV: &str = "BMCBL_XGAMERUNTIME_PREAUTH";
pub const LAUNCH_NONCE_ENV: &str = "BMCBL_XGAMERUNTIME_NONCE";
pub const MAX_PREAUTH_FILE_SIZE: u64 = 64 * 1024;

#[derive(Debug)]
pub struct LaunchProfile {
    pub profile_id: String,
    pub preauth_path: PathBuf,
    pub preauth: PreauthDocument,
}

impl LaunchProfile {
    pub fn from_environment() -> Result<Option<Self>, ProxyError> {
        let profile_id = env::var(PROFILE_ID_ENV).ok();
        let preauth_path = env::var_os(PREAUTH_PATH_ENV).map(PathBuf::from);
        let launch_nonce = env::var(LAUNCH_NONCE_ENV).ok();

        match (profile_id, preauth_path) {
            (None, None) => Ok(None),
            (Some(profile_id), Some(preauth_path)) => {
                validate_profile_id(&profile_id)?;
                if !preauth_path.is_absolute() {
                    return Err(ProxyError::PreauthPathNotAbsolute(preauth_path));
                }

                let metadata =
                    fs::metadata(&preauth_path).map_err(|source| ProxyError::PreauthRead {
                        path: preauth_path.clone(),
                        source,
                    })?;
                if !metadata.is_file() || metadata.len() > MAX_PREAUTH_FILE_SIZE {
                    return Err(ProxyError::PreauthTooLarge);
                }

                let bytes = fs::read(&preauth_path).map_err(|source| ProxyError::PreauthRead {
                    path: preauth_path.clone(),
                    source,
                })?;
                let preauth: PreauthDocument = serde_json::from_slice(&bytes)?;
                let now_epoch = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                preauth.validate(&profile_id, launch_nonce.as_deref(), now_epoch)?;

                Ok(Some(Self {
                    profile_id,
                    preauth_path,
                    preauth,
                }))
            }
            _ => Err(ProxyError::IncompleteProfileEnvironment),
        }
    }
}

fn validate_profile_id(profile_id: &str) -> Result<(), ProxyError> {
    if profile_id.is_empty()
        || profile_id.len() > 128
        || !profile_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(ProxyError::InvalidProfileId);
    }
    Ok(())
}
