// SPDX-License-Identifier: LGPL-2.1-or-later
//
// Derived from Wine/WineGDK include/xasyncprovider.idl. Existing copyright
// notices and LGPL-2.1-or-later terms are preserved in LICENSE and NOTICE.md.

use core::ffi::{c_char, c_void};

use crate::{
    abi::{E_FAIL, E_POINTER, Guid, HResult},
    native,
    xuser::abi::XAsyncBlock,
};

pub const CLSID_XTHREADING_IMPL: Guid = Guid::new(
    0x073b_7dcb,
    0x1fcf,
    0x4030,
    [0x94, 0xbe, 0xe3, 0xc9, 0xeb, 0x62, 0x34, 0x28],
);
pub const IID_IXTHREADING_IMPL: Guid = CLSID_XTHREADING_IMPL;

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum XAsyncOp {
    Begin = 0,
    DoWork = 1,
    GetResult = 2,
    Cancel = 3,
    Cleanup = 4,
}

#[repr(C)]
pub struct XAsyncProviderData {
    pub async_block: *mut XAsyncBlock,
    pub buffer_size: usize,
    pub buffer: *mut c_void,
    pub context: *mut c_void,
}

pub type XAsyncProvider =
    unsafe extern "system" fn(XAsyncOp, *const XAsyncProviderData) -> HResult;

type QueryApiImplFn =
    unsafe extern "system" fn(*const Guid, *const Guid, *mut *mut c_void) -> HResult;

#[repr(C)]
struct XThreadingInterface {
    vtable: *const XThreadingVtable,
}

#[repr(C)]
struct XThreadingVtable {
    query_interface:
        unsafe extern "system" fn(*mut XThreadingInterface, *const Guid, *mut *mut c_void) -> HResult,
    add_ref: unsafe extern "system" fn(*mut XThreadingInterface) -> u32,
    release: unsafe extern "system" fn(*mut XThreadingInterface) -> u32,
    async_get_status:
        unsafe extern "system" fn(*mut XThreadingInterface, *mut XAsyncBlock, u8) -> HResult,
    async_get_result_size: unsafe extern "system" fn(
        *mut XThreadingInterface,
        *mut XAsyncBlock,
        *mut usize,
    ) -> HResult,
    async_cancel: unsafe extern "system" fn(*mut XThreadingInterface, *mut XAsyncBlock),
    async_run: unsafe extern "system" fn(
        *mut XThreadingInterface,
        *mut XAsyncBlock,
        *mut c_void,
    ) -> HResult,
    async_begin: unsafe extern "system" fn(
        *mut XThreadingInterface,
        *mut XAsyncBlock,
        *mut c_void,
        *const c_void,
        *const c_char,
        XAsyncProvider,
    ) -> HResult,
    padding: unsafe extern "system" fn(*mut XThreadingInterface) -> HResult,
    async_schedule:
        unsafe extern "system" fn(*mut XThreadingInterface, *mut XAsyncBlock, u32) -> HResult,
    async_complete:
        unsafe extern "system" fn(*mut XThreadingInterface, *mut XAsyncBlock, HResult, usize),
    async_get_result: unsafe extern "system" fn(
        *mut XThreadingInterface,
        *mut XAsyncBlock,
        *const c_void,
        usize,
        *mut c_void,
        *mut usize,
    ) -> HResult,
}

struct ThreadingHandle(*mut XThreadingInterface);

impl ThreadingHandle {
    fn acquire() -> Result<Self, HResult> {
        let runtime = native::runtime().map_err(|_| E_FAIL)?;
        let address = runtime
            .proc_address(b"QueryApiImpl\0")
            .map_err(|_| E_FAIL)?;
        let query: QueryApiImplFn = unsafe { core::mem::transmute(address) };
        let mut interface = core::ptr::null_mut();
        let status = unsafe {
            query(
                &CLSID_XTHREADING_IMPL,
                &IID_IXTHREADING_IMPL,
                &mut interface,
            )
        };
        if status < 0 {
            return Err(status);
        }
        if interface.is_null() {
            return Err(E_POINTER);
        }
        Ok(Self(interface.cast()))
    }

    fn vtable(&self) -> &XThreadingVtable {
        unsafe { &*(*self.0).vtable }
    }
}

impl Drop for ThreadingHandle {
    fn drop(&mut self) {
        unsafe { (self.vtable().release)(self.0) };
    }
}

pub unsafe fn begin(
    async_block: *mut XAsyncBlock,
    context: *mut c_void,
    identity: *const c_void,
    identity_name: *const c_char,
    provider: XAsyncProvider,
) -> HResult {
    if async_block.is_null() || identity.is_null() || identity_name.is_null() {
        return E_POINTER;
    }
    let Ok(threading) = ThreadingHandle::acquire() else {
        return E_FAIL;
    };
    unsafe {
        (threading.vtable().async_begin)(
            threading.0,
            async_block,
            context,
            identity,
            identity_name,
            provider,
        )
    }
}

pub unsafe fn schedule(async_block: *mut XAsyncBlock, delay_ms: u32) -> HResult {
    if async_block.is_null() {
        return E_POINTER;
    }
    let Ok(threading) = ThreadingHandle::acquire() else {
        return E_FAIL;
    };
    unsafe { (threading.vtable().async_schedule)(threading.0, async_block, delay_ms) }
}

pub unsafe fn complete(async_block: *mut XAsyncBlock, result: HResult, required_size: usize) {
    if async_block.is_null() {
        return;
    }
    let Ok(threading) = ThreadingHandle::acquire() else {
        return;
    };
    unsafe {
        (threading.vtable().async_complete)(threading.0, async_block, result, required_size)
    };
}

pub unsafe fn get_result(
    async_block: *mut XAsyncBlock,
    identity: *const c_void,
    buffer_size: usize,
    buffer: *mut c_void,
    used: *mut usize,
) -> HResult {
    if async_block.is_null() || identity.is_null() || (buffer_size != 0 && buffer.is_null()) {
        return E_POINTER;
    }
    let Ok(threading) = ThreadingHandle::acquire() else {
        return E_FAIL;
    };
    unsafe {
        (threading.vtable().async_get_result)(
            threading.0,
            async_block,
            identity,
            buffer_size,
            buffer,
            used,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_data_layout_is_pointer_aligned() {
        assert_eq!(
            core::mem::size_of::<XAsyncProviderData>(),
            4 * core::mem::size_of::<usize>()
        );
    }

    #[test]
    fn threading_guid_matches_winegdk_contract() {
        assert_eq!(CLSID_XTHREADING_IMPL, IID_IXTHREADING_IMPL);
        assert_eq!(CLSID_XTHREADING_IMPL.data1, 0x073b_7dcb);
    }
}
