// SPDX-License-Identifier: LGPL-2.1-or-later

mod abi;
mod object;

use core::ffi::c_void;
use std::{env, sync::OnceLock};

use crate::abi::{E_POINTER, Guid, HResult, S_OK};

pub use abi::CLSID_XUSER_IMPL;

pub const XUSER_ENABLE_ENV: &str = "BMCBL_XGAMERUNTIME_ENABLE_XUSER";

static ENABLED: OnceLock<bool> = OnceLock::new();

pub fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        env::var(XUSER_ENABLE_ENV)
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false)
    })
}

pub unsafe fn query_interface(iid: *const Guid, out: *mut *mut c_void) -> HResult {
    if iid.is_null() || out.is_null() {
        return E_POINTER;
    }
    unsafe { out.write(core::ptr::null_mut()) };

    let iid_ref = unsafe { &*iid };
    if !abi::is_xuser_interface(iid_ref) {
        return abi::E_NOINTERFACE;
    }

    let interface = if *iid_ref == abi::IID_IXUSER_GAMERTAG {
        object::gamertag_interface()
    } else {
        object::provider_interface()
    };

    match interface {
        Some(interface) => {
            unsafe { out.write(interface) };
            S_OK
        }
        None => crate::abi::E_FAIL,
    }
}
