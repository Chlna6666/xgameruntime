// SPDX-License-Identifier: LGPL-2.1-or-later

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("environment variable {0} is not set")]
    MissingEnvironment(&'static str),

    #[error("native runtime path must be absolute: {0}")]
    NativePathNotAbsolute(PathBuf),

    #[error("native runtime path does not exist: {0}")]
    NativePathMissing(PathBuf),

    #[error("failed to load native xgameruntime from {path}: Win32 error {code}")]
    NativeLoad { path: PathBuf, code: u32 },

    #[error("native xgameruntime does not export {0}")]
    MissingExport(String),

    #[error("custom profile configuration is incomplete")]
    IncompleteProfileEnvironment,

    #[error("profile identifier is invalid")]
    InvalidProfileId,

    #[error("pre-authentication file path must be absolute: {0}")]
    PreauthPathNotAbsolute(PathBuf),

    #[error("pre-authentication file is too large")]
    PreauthTooLarge,

    #[error("failed to read pre-authentication file {path}: {source}")]
    PreauthRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to decode pre-authentication document: {0}")]
    PreauthJson(#[from] serde_json::Error),

    #[error("unsupported pre-authentication schema version {0}")]
    UnsupportedSchema(u32),

    #[error("pre-authentication profile does not match the selected profile")]
    ProfileMismatch,

    #[error("pre-authentication document is not currently valid")]
    PreauthExpired,

    #[error("pre-authentication document has an invalid time range")]
    InvalidTimeRange,

    #[error("pre-authentication token {0} is invalid or expired")]
    InvalidToken(&'static str),

    #[error("this operation is only available on Windows")]
    UnsupportedPlatform,
}
