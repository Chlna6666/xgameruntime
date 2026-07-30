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
/// XUser interception is intentionally opt-in until the ABI and Minecraft
/// integration tests cover the complete async and token/signature path.
pub unsafe fn query_api(
    runtime_class_id: *const Guid,
    interface_id: *const Guid,
    out: *mut *mut c_void,
) -> Option<HResult> {
    let _profile = state::selected_profile()?;
    if !xuser::enabled() {
        return None;
    }

    let runtime_class_id = unsafe { &*runtime_class_id };
    if *runtime_class_id != xuser::CLSID_XUSER_IMPL {
        return None;
    }

    Some(unsafe { xuser::query_interface(interface_id, out) })
}
