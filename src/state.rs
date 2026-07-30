// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::OnceLock;

use crate::{error::ProxyError, profile::LaunchProfile};

static PROFILE: OnceLock<Result<Option<LaunchProfile>, ProxyError>> = OnceLock::new();

pub fn selected_profile() -> Option<&'static LaunchProfile> {
    match PROFILE.get_or_init(LaunchProfile::from_environment) {
        Ok(Some(profile)) => Some(profile),
        Ok(None) | Err(_) => None,
    }
}

pub fn profile_error() -> Option<&'static ProxyError> {
    match PROFILE.get_or_init(LaunchProfile::from_environment) {
        Err(error) => Some(error),
        Ok(_) => None,
    }
}
