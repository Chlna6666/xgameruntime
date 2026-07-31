// SPDX-License-Identifier: LGPL-2.1-or-later

use core::ffi::c_void;

use crate::{
    abi::{Guid, HResult},
    state, xuser,
};

/// Attempts to satisfy a GDK `QueryApiImpl` request from the selected custom
/// profile. Returning `None` means the request must be forwarded to the native
/// Microsoft runtime.
///
/// XUser interception is strictly opt-in. The enable switch is checked before
/// reading any profile or pre-authentication environment, so a process without
/// `BMCBL_XGAMERUNTIME_ENABLE_XUSER=1` remains a transparent proxy and follows
/// the native loader chain (`xgameruntime_o.dll`, then the System32 runtime).
pub unsafe fn query_api(
    runtime_class_id: *const Guid,
    interface_id: *const Guid,
    out: *mut *mut c_void,
) -> Option<HResult> {
    // This is the hard pass-through gate. Profile variables by themselves must
    // never replace the official Microsoft sign-in implementation.
    if !xuser::enabled() {
        return None;
    }

    // Missing, incomplete, invalid, or expired profile data also disables the
    // custom provider and falls back to the native Microsoft runtime.
    let _profile = state::selected_profile()?;

    let runtime_class_id = unsafe { &*runtime_class_id };
    if *runtime_class_id != xuser::CLSID_XUSER_IMPL {
        return None;
    }

    Some(unsafe { xuser::query_interface(interface_id, out) })
}
