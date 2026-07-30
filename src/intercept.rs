// SPDX-License-Identifier: LGPL-2.1-or-later

use core::ffi::c_void;

use crate::{
    abi::{Guid, HResult},
    state,
};

/// Attempts to satisfy a GDK `QueryApiImpl` request from the selected custom
/// profile. Returning `None` means the request must be forwarded to the native
/// Microsoft runtime.
///
/// The bootstrap deliberately intercepts nothing until the XUser GUIDs,
/// interface versions and vtable layouts have dedicated ABI tests.
pub unsafe fn query_api(
    _runtime_class_id: *const Guid,
    _interface_id: *const Guid,
    _out: *mut *mut c_void,
) -> Option<HResult> {
    let _profile = state::selected_profile()?;
    None
}
