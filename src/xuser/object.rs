// SPDX-License-Identifier: LGPL-2.1-or-later
//
// Rust translation and adaptation of the WineGDK XUser provider contract.
// The original WineGDK notices and LGPL-2.1-or-later terms are preserved by
// this repository's LICENSE and NOTICE.md.

use core::ffi::{c_char, c_void};
use std::sync::OnceLock;

use crate::{
    abi::{E_FAIL, E_POINTER, Guid, HResult, S_OK},
    profile::LaunchProfile,
    state,
    xasync::{self, XAsyncOp, XAsyncProviderData},
};

use super::{
    abi::{
        E_INVALIDARG, E_NOINTERFACE, E_NOTIMPL, E_NOT_SUFFICIENT_BUFFER, IID_IUNKNOWN,
        IID_IXUSER_ADD_WITH_UI, IID_IXUSER_BASE, IID_IXUSER_GAMERTAG, IID_IXUSER_MSA,
        IID_IXUSER_PLATFORM, IID_IXUSER_SIGN_OUT, IID_IXUSER_STORE, XAsyncBlock,
        XUserGamertagVtable, XUserHandle, XUserLocalId, XUserVtable, XUSER_AGE_GROUP_ADULT,
        XUSER_AGE_GROUP_CHILD, XUSER_AGE_GROUP_TEEN, XUSER_AGE_GROUP_UNKNOWN,
        XUSER_STATE_SIGNED_IN,
    },
    token,
};

#[repr(C)]
struct XUserGamertagInterface {
    vtable: *const XUserGamertagVtable,
}

#[repr(C)]
struct XUserObject {
    vtable: *const XUserVtable,
    gamertag: XUserGamertagInterface,
    profile: &'static LaunchProfile,
    xuid: u64,
    local_id: XUserLocalId,
    age_group: u32,
}

// The pointers only reference immutable process-lifetime vtables. The selected
// profile is initialized once and remains immutable for the process lifetime.
unsafe impl Send for XUserObject {}
unsafe impl Sync for XUserObject {}

struct XUserAddContext {
    handle: usize,
}

static USER_VTABLE: OnceLock<XUserVtable> = OnceLock::new();
static GAMERTAG_VTABLE: OnceLock<XUserGamertagVtable> = OnceLock::new();
static USER_OBJECT: OnceLock<XUserObject> = OnceLock::new();
static XUSER_ADD_IDENTITY: u8 = 0;
const XUSER_ADD_NAME: &[u8] = b"XUserAddAsync\0";

fn user_object() -> Option<&'static XUserObject> {
    let profile = state::selected_profile()?;
    Some(USER_OBJECT.get_or_init(|| XUserObject::new(profile)))
}

impl XUserObject {
    fn new(profile: &'static LaunchProfile) -> Self {
        let xuid = profile.preauth.xbox.xuid.parse::<u64>().unwrap_or(0);
        let local_id = profile
            .preauth
            .xbox
            .xbox_live
            .user_hash
            .parse::<u64>()
            .unwrap_or(xuid);

        Self {
            vtable: user_vtable(),
            gamertag: XUserGamertagInterface {
                vtable: gamertag_vtable(),
            },
            profile,
            xuid,
            local_id: XUserLocalId { value: local_id },
            age_group: parse_age_group(profile.preauth.xbox.age_group.as_deref()),
        }
    }

    fn interface(&self) -> *mut c_void {
        self as *const Self as *mut c_void
    }

    fn gamertag_interface(&self) -> *mut c_void {
        &self.gamertag as *const XUserGamertagInterface as *mut c_void
    }

    fn is_handle(&self, handle: XUserHandle) -> bool {
        handle == self.interface()
    }
}

fn parse_age_group(value: Option<&str>) -> u32 {
    match value.unwrap_or_default().to_ascii_lowercase().as_str() {
        "child" => XUSER_AGE_GROUP_CHILD,
        "teen" | "teenager" => XUSER_AGE_GROUP_TEEN,
        "adult" => XUSER_AGE_GROUP_ADULT,
        _ => XUSER_AGE_GROUP_UNKNOWN,
    }
}

fn xuser_add_identity() -> *const c_void {
    (&XUSER_ADD_IDENTITY as *const u8).cast()
}

pub fn provider_interface() -> Option<*mut c_void> {
    Some(user_object()?.interface())
}

pub fn gamertag_interface() -> Option<*mut c_void> {
    Some(user_object()?.gamertag_interface())
}

unsafe extern "system" fn query_interface(
    _iface: *mut c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> HResult {
    if iid.is_null() || out.is_null() {
        return E_POINTER;
    }
    unsafe { out.write(core::ptr::null_mut()) };

    let Some(object) = user_object() else {
        return E_FAIL;
    };
    let iid = unsafe { &*iid };

    if [
        IID_IUNKNOWN,
        IID_IXUSER_BASE,
        IID_IXUSER_ADD_WITH_UI,
        IID_IXUSER_MSA,
        IID_IXUSER_STORE,
        IID_IXUSER_PLATFORM,
        IID_IXUSER_SIGN_OUT,
    ]
    .contains(iid)
    {
        unsafe { out.write(object.interface()) };
        return S_OK;
    }

    if *iid == IID_IXUSER_GAMERTAG {
        unsafe { out.write(object.gamertag_interface()) };
        return S_OK;
    }

    E_NOINTERFACE
}

unsafe extern "system" fn add_ref(_iface: *mut c_void) -> u32 {
    2
}

unsafe extern "system" fn release(_iface: *mut c_void) -> u32 {
    1
}

unsafe extern "system" fn duplicate_handle(
    _iface: *mut c_void,
    user: XUserHandle,
    duplicated: *mut XUserHandle,
) -> HResult {
    if duplicated.is_null() {
        return E_POINTER;
    }
    let Some(object) = user_object() else {
        return E_FAIL;
    };
    let handle = if user.is_null() {
        object.interface()
    } else if object.is_handle(user) {
        user
    } else {
        return E_INVALIDARG;
    };
    unsafe { duplicated.write(handle) };
    S_OK
}

unsafe extern "system" fn close_handle(_iface: *mut c_void, _user: XUserHandle) {}

unsafe extern "system" fn compare(
    _iface: *mut c_void,
    user1: XUserHandle,
    user2: XUserHandle,
) -> i32 {
    i32::from(user1 != user2)
}

unsafe extern "system" fn get_max_users(_iface: *mut c_void, max_users: *mut u32) -> HResult {
    if max_users.is_null() {
        return E_POINTER;
    }
    unsafe { max_users.write(1) };
    S_OK
}

unsafe extern "system" fn xuser_add_provider(
    operation: XAsyncOp,
    provider_data: *const XAsyncProviderData,
) -> HResult {
    if provider_data.is_null() {
        return E_POINTER;
    }
    let provider_data = unsafe { &*provider_data };
    let context = provider_data.context.cast::<XUserAddContext>();
    if context.is_null() {
        return E_POINTER;
    }

    match operation {
        XAsyncOp::Begin => unsafe { xasync::schedule(provider_data.async_block, 0) },
        XAsyncOp::DoWork => {
            unsafe {
                xasync::complete(
                    provider_data.async_block,
                    S_OK,
                    core::mem::size_of::<XUserHandle>(),
                )
            };
            S_OK
        }
        XAsyncOp::GetResult => {
            if provider_data.buffer.is_null()
                || provider_data.buffer_size < core::mem::size_of::<XUserHandle>()
            {
                return E_NOT_SUFFICIENT_BUFFER;
            }
            let handle = unsafe { (*context).handle as XUserHandle };
            unsafe { provider_data.buffer.cast::<XUserHandle>().write(handle) };
            S_OK
        }
        XAsyncOp::Cancel => S_OK,
        XAsyncOp::Cleanup => {
            unsafe { drop(Box::from_raw(context)) };
            S_OK
        }
    }
}

unsafe extern "system" fn add_async(
    _iface: *mut c_void,
    _options: u32,
    async_block: *mut XAsyncBlock,
) -> HResult {
    if async_block.is_null() {
        return E_POINTER;
    }
    let Some(handle) = provider_interface() else {
        return E_FAIL;
    };

    let context = Box::into_raw(Box::new(XUserAddContext {
        handle: handle as usize,
    }));
    let result = unsafe {
        xasync::begin(
            async_block,
            context.cast(),
            xuser_add_identity(),
            XUSER_ADD_NAME.as_ptr().cast(),
            xuser_add_provider,
        )
    };
    if result < 0 {
        unsafe { drop(Box::from_raw(context)) };
    }
    result
}

unsafe extern "system" fn add_result(
    _iface: *mut c_void,
    async_block: *mut XAsyncBlock,
    user: *mut XUserHandle,
) -> HResult {
    if async_block.is_null() || user.is_null() {
        return E_POINTER;
    }
    unsafe {
        xasync::get_result(
            async_block,
            xuser_add_identity(),
            core::mem::size_of::<XUserHandle>(),
            user.cast(),
            core::ptr::null_mut(),
        )
    }
}

unsafe extern "system" fn get_local_id(
    _iface: *mut c_void,
    user: XUserHandle,
    local_id: *mut XUserLocalId,
) -> HResult {
    if user.is_null() || local_id.is_null() {
        return E_POINTER;
    }
    let Some(object) = user_object() else {
        return E_FAIL;
    };
    if !object.is_handle(user) {
        return E_INVALIDARG;
    }
    unsafe { local_id.write(object.local_id) };
    S_OK
}

unsafe extern "system" fn find_user_by_local_id(
    _iface: *mut c_void,
    local_id: XUserLocalId,
    user: *mut XUserHandle,
) -> HResult {
    if user.is_null() {
        return E_POINTER;
    }
    let Some(object) = user_object() else {
        return E_FAIL;
    };
    if object.local_id != local_id {
        return E_FAIL;
    }
    unsafe { user.write(object.interface()) };
    S_OK
}

unsafe extern "system" fn get_id(
    _iface: *mut c_void,
    user: XUserHandle,
    user_id: *mut u64,
) -> HResult {
    if user.is_null() || user_id.is_null() {
        return E_POINTER;
    }
    let Some(object) = user_object() else {
        return E_FAIL;
    };
    if !object.is_handle(user) {
        return E_INVALIDARG;
    }
    unsafe { user_id.write(object.xuid) };
    S_OK
}

unsafe extern "system" fn find_user_by_id(
    _iface: *mut c_void,
    user_id: u64,
    user: *mut XUserHandle,
) -> HResult {
    if user.is_null() {
        return E_POINTER;
    }
    let Some(object) = user_object() else {
        return E_FAIL;
    };
    if object.xuid != user_id {
        return E_FAIL;
    }
    unsafe { user.write(object.interface()) };
    S_OK
}

unsafe extern "system" fn get_is_guest(
    _iface: *mut c_void,
    user: XUserHandle,
    is_guest: *mut u8,
) -> HResult {
    if user.is_null() || is_guest.is_null() {
        return E_POINTER;
    }
    unsafe { is_guest.write(0) };
    S_OK
}

unsafe extern "system" fn get_state(
    _iface: *mut c_void,
    user: XUserHandle,
    state: *mut u32,
) -> HResult {
    if user.is_null() || state.is_null() {
        return E_POINTER;
    }
    unsafe { state.write(XUSER_STATE_SIGNED_IN) };
    S_OK
}

unsafe extern "system" fn get_age_group(
    _iface: *mut c_void,
    user: XUserHandle,
    age_group: *mut u32,
) -> HResult {
    if user.is_null() || age_group.is_null() {
        return E_POINTER;
    }
    let Some(object) = user_object() else {
        return E_FAIL;
    };
    unsafe { age_group.write(object.age_group) };
    S_OK
}

unsafe extern "system" fn check_privilege(
    _iface: *mut c_void,
    user: XUserHandle,
    _options: u32,
    privilege: i32,
    has_privilege: *mut u8,
    deny_reason: *mut u32,
) -> HResult {
    if user.is_null() || has_privilege.is_null() || deny_reason.is_null() {
        return E_POINTER;
    }
    let Some(object) = user_object() else {
        return E_FAIL;
    };
    let allowed = privilege >= 0
        && object
            .profile
            .preauth
            .xbox
            .privileges
            .contains(&(privilege as u32));
    unsafe {
        has_privilege.write(u8::from(allowed));
        deny_reason.write(0);
    }
    S_OK
}

unsafe extern "system" fn gamertag_query_interface(
    _iface: *mut c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> HResult {
    unsafe { query_interface(core::ptr::null_mut(), iid, out) }
}

unsafe extern "system" fn get_gamertag(
    _iface: *mut c_void,
    user: XUserHandle,
    component: u32,
    size: usize,
    gamertag: *mut c_char,
    used: *mut usize,
) -> HResult {
    if user.is_null() || gamertag.is_null() {
        return E_POINTER;
    }
    let Some(object) = user_object() else {
        return E_FAIL;
    };
    if !object.is_handle(user) {
        return E_INVALIDARG;
    }

    let value = match component {
        0 | 1 | 3 => object.profile.preauth.xbox.gamertag.as_str(),
        2 => "",
        _ => return E_INVALIDARG,
    };
    let required = value.len() + 1;
    if !used.is_null() {
        unsafe { used.write(required) };
    }
    if size < required {
        return E_NOT_SUFFICIENT_BUFFER;
    }

    unsafe {
        core::ptr::copy_nonoverlapping(value.as_ptr(), gamertag.cast::<u8>(), value.len());
        gamertag.add(value.len()).write(0);
    }
    S_OK
}

unsafe extern "system" fn stub_hresult(_iface: *mut c_void) -> HResult {
    E_NOTIMPL
}

unsafe extern "system" fn stub_boolean(_iface: *mut c_void) -> u8 {
    0
}

unsafe extern "system" fn stub_void(_iface: *mut c_void) {}

fn user_vtable() -> *const XUserVtable {
    USER_VTABLE.get_or_init(|| XUserVtable {
        query_interface: query_interface as usize,
        add_ref: add_ref as usize,
        release: release as usize,
        duplicate_handle: duplicate_handle as usize,
        close_handle: close_handle as usize,
        compare: compare as usize,
        get_max_users: get_max_users as usize,
        add_async: add_async as usize,
        add_result: add_result as usize,
        get_local_id: get_local_id as usize,
        find_user_by_local_id: find_user_by_local_id as usize,
        get_id: get_id as usize,
        find_user_by_id: find_user_by_id as usize,
        get_is_guest: get_is_guest as usize,
        get_state: get_state as usize,
        padding: stub_hresult as usize,
        get_gamer_picture_async: stub_hresult as usize,
        get_gamer_picture_result_size: stub_hresult as usize,
        get_gamer_picture_result: stub_hresult as usize,
        get_age_group: get_age_group as usize,
        check_privilege: check_privilege as usize,
        resolve_privilege_with_ui_async: stub_hresult as usize,
        resolve_privilege_with_ui_result: stub_hresult as usize,
        get_token_and_signature_async: token::get_token_and_signature_async as usize,
        get_token_and_signature_result_size: token::get_token_and_signature_result_size as usize,
        get_token_and_signature_result: token::get_token_and_signature_result as usize,
        get_token_and_signature_utf16_async: token::get_token_and_signature_utf16_async as usize,
        get_token_and_signature_utf16_result_size:
            token::get_token_and_signature_utf16_result_size as usize,
        get_token_and_signature_utf16_result:
            token::get_token_and_signature_utf16_result as usize,
        resolve_issue_with_ui_async: stub_hresult as usize,
        resolve_issue_with_ui_result: stub_hresult as usize,
        resolve_issue_with_ui_utf16_async: stub_hresult as usize,
        resolve_issue_with_ui_utf16_result: stub_hresult as usize,
        register_for_change_event: stub_hresult as usize,
        unregister_for_change_event: stub_boolean as usize,
        get_sign_out_deferral: stub_hresult as usize,
        close_sign_out_deferral_handle: stub_void as usize,
        add_by_id_with_ui_async: stub_hresult as usize,
        add_by_id_with_ui_result: stub_hresult as usize,
        get_msa_token_silently_async: stub_hresult as usize,
        get_msa_token_silently_result: stub_hresult as usize,
        get_msa_token_silently_result_size: stub_hresult as usize,
        is_store_user: stub_boolean as usize,
        platform_remote_connect_set_event_handlers: stub_hresult as usize,
        platform_remote_connect_cancel_prompt: stub_hresult as usize,
        platform_spop_prompt_set_event_handlers: stub_hresult as usize,
        platform_spop_prompt_complete: stub_hresult as usize,
        is_sign_out_present: stub_boolean as usize,
        sign_out_async: stub_hresult as usize,
        sign_out_result: stub_hresult as usize,
    }) as *const XUserVtable
}

fn gamertag_vtable() -> *const XUserGamertagVtable {
    GAMERTAG_VTABLE.get_or_init(|| XUserGamertagVtable {
        query_interface: gamertag_query_interface as usize,
        add_ref: add_ref as usize,
        release: release as usize,
        get_gamertag: get_gamertag as usize,
    }) as *const XUserGamertagVtable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_group_mapping_is_stable() {
        assert_eq!(parse_age_group(Some("Child")), XUSER_AGE_GROUP_CHILD);
        assert_eq!(parse_age_group(Some("Teen")), XUSER_AGE_GROUP_TEEN);
        assert_eq!(parse_age_group(Some("Adult")), XUSER_AGE_GROUP_ADULT);
        assert_eq!(parse_age_group(None), XUSER_AGE_GROUP_UNKNOWN);
    }
}
