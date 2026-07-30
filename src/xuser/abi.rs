// SPDX-License-Identifier: LGPL-2.1-or-later
//
// Derived from WineGDK dlls/xgameruntime/provider.idl. Existing WineGDK
// copyright and licensing terms remain applicable to translated ABI material.

use core::ffi::c_void;

use crate::abi::{Guid, HResult};

pub const E_NOINTERFACE: HResult = 0x8000_4002_u32 as i32;
pub const E_NOTIMPL: HResult = 0x8000_4001_u32 as i32;
pub const E_INVALIDARG: HResult = 0x8007_0057_u32 as i32;
pub const E_NOT_SUFFICIENT_BUFFER: HResult = 0x8007_007a_u32 as i32;

pub const IID_IUNKNOWN: Guid = Guid::new(
    0x0000_0000,
    0x0000,
    0x0000,
    [0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
);

pub const CLSID_XUSER_IMPL: Guid = Guid::new(
    0x01ac_d177,
    0x91f9,
    0x4763,
    [0xa3, 0x8e, 0xcc, 0xbb, 0x55, 0xce, 0x32, 0xe0],
);
pub const IID_IXUSER_BASE: Guid = CLSID_XUSER_IMPL;
pub const IID_IXUSER_ADD_WITH_UI: Guid = Guid::new(
    0xeb9b_f948,
    0x18dc,
    0x4d82,
    [0xbb, 0xcc, 0x40, 0xe0, 0xa8, 0x09, 0xc4, 0xc0],
);
pub const IID_IXUSER_MSA: Guid = Guid::new(
    0x1bf2_f8c5,
    0xd507,
    0x4e52,
    [0xbb, 0x05, 0xf7, 0x26, 0xd0, 0xe7, 0x11, 0x61],
);
pub const IID_IXUSER_STORE: Guid = Guid::new(
    0x0794_15e3,
    0x6727,
    0x437f,
    [0x8e, 0x9d, 0x8f, 0x8f, 0x9b, 0x24, 0x39, 0xf7],
);
pub const IID_IXUSER_PLATFORM: Guid = Guid::new(
    0x26f3_c674,
    0xa2fe,
    0x44fa,
    [0xb6, 0xc4, 0xa3, 0x23, 0xbc, 0x94, 0xff, 0x53],
);
pub const IID_IXUSER_SIGN_OUT: Guid = Guid::new(
    0x5131_d685,
    0x4394,
    0x4ee6,
    [0x8c, 0x18, 0xbf, 0xb5, 0xd4, 0xae, 0xf1, 0xff],
);
pub const IID_IXUSER_GAMERTAG: Guid = Guid::new(
    0xcef4_fac0,
    0x7676,
    0x4a94,
    [0xa1, 0x19, 0x4c, 0x43, 0xf9, 0xeb, 0x5b, 0x74],
);

pub const XUSER_STATE_SIGNED_IN: u32 = 0;
pub const XUSER_AGE_GROUP_UNKNOWN: u32 = 0;
pub const XUSER_AGE_GROUP_CHILD: u32 = 1;
pub const XUSER_AGE_GROUP_TEEN: u32 = 2;
pub const XUSER_AGE_GROUP_ADULT: u32 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XUserLocalId {
    pub value: u64,
}

pub type XUserHandle = *mut c_void;
pub type XAsyncCompletionRoutine = unsafe extern "system" fn(*mut XAsyncBlock);

#[repr(C)]
pub struct XAsyncBlock {
    pub queue: *mut c_void,
    pub context: *mut c_void,
    pub callback: Option<XAsyncCompletionRoutine>,
    pub internal: [usize; 4],
}

/// Exact slot order reconstructed from WineGDK provider.idl and XUser.c.
///
/// Slots are stored as machine words because the unsupported methods have
/// different native signatures. This project currently targets Windows x64,
/// where all methods share the Microsoft x64 calling convention. Every slot
/// is nevertheless populated with a return-type-compatible stub.
#[repr(C)]
pub struct XUserVtable {
    pub query_interface: usize,
    pub add_ref: usize,
    pub release: usize,
    pub duplicate_handle: usize,
    pub close_handle: usize,
    pub compare: usize,
    pub get_max_users: usize,
    pub add_async: usize,
    pub add_result: usize,
    pub get_local_id: usize,
    pub find_user_by_local_id: usize,
    pub get_id: usize,
    pub find_user_by_id: usize,
    pub get_is_guest: usize,
    pub get_state: usize,
    pub padding: usize,
    pub get_gamer_picture_async: usize,
    pub get_gamer_picture_result_size: usize,
    pub get_gamer_picture_result: usize,
    pub get_age_group: usize,
    pub check_privilege: usize,
    pub resolve_privilege_with_ui_async: usize,
    pub resolve_privilege_with_ui_result: usize,
    pub get_token_and_signature_async: usize,
    pub get_token_and_signature_result_size: usize,
    pub get_token_and_signature_result: usize,
    pub get_token_and_signature_utf16_async: usize,
    pub get_token_and_signature_utf16_result_size: usize,
    pub get_token_and_signature_utf16_result: usize,
    pub resolve_issue_with_ui_async: usize,
    pub resolve_issue_with_ui_result: usize,
    pub resolve_issue_with_ui_utf16_async: usize,
    pub resolve_issue_with_ui_utf16_result: usize,
    pub register_for_change_event: usize,
    pub unregister_for_change_event: usize,
    pub get_sign_out_deferral: usize,
    pub close_sign_out_deferral_handle: usize,
    pub add_by_id_with_ui_async: usize,
    pub add_by_id_with_ui_result: usize,
    pub get_msa_token_silently_async: usize,
    pub get_msa_token_silently_result: usize,
    pub get_msa_token_silently_result_size: usize,
    pub is_store_user: usize,
    pub platform_remote_connect_set_event_handlers: usize,
    pub platform_remote_connect_cancel_prompt: usize,
    pub platform_spop_prompt_set_event_handlers: usize,
    pub platform_spop_prompt_complete: usize,
    pub is_sign_out_present: usize,
    pub sign_out_async: usize,
    pub sign_out_result: usize,
}

#[repr(C)]
pub struct XUserGamertagVtable {
    pub query_interface: usize,
    pub add_ref: usize,
    pub release: usize,
    pub get_gamertag: usize,
}

pub fn is_xuser_interface(iid: &Guid) -> bool {
    matches!(
        *iid,
        IID_IUNKNOWN
            | IID_IXUSER_BASE
            | IID_IXUSER_ADD_WITH_UI
            | IID_IXUSER_MSA
            | IID_IXUSER_STORE
            | IID_IXUSER_PLATFORM
            | IID_IXUSER_SIGN_OUT
            | IID_IXUSER_GAMERTAG
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xuser_vtable_has_expected_slot_count() {
        assert_eq!(
            core::mem::size_of::<XUserVtable>(),
            50 * core::mem::size_of::<usize>()
        );
        assert_eq!(
            core::mem::size_of::<XUserGamertagVtable>(),
            4 * core::mem::size_of::<usize>()
        );
    }
}
