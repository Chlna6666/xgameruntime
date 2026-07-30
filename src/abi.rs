// SPDX-License-Identifier: LGPL-2.1-or-later

use core::ffi::{c_char, c_void};

pub type HResult = i32;

pub const S_OK: HResult = 0;
pub const S_FALSE: HResult = 1;
pub const E_FAIL: HResult = 0x8000_4005_u32 as i32;
pub const E_POINTER: HResult = 0x8000_4003_u32 as i32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

pub type DllCanUnloadNowFn = unsafe extern "system" fn() -> HResult;
pub type DllGetClassObjectFn =
    unsafe extern "system" fn(*const Guid, *const Guid, *mut *mut c_void) -> HResult;
pub type InitializeApiImplFn = unsafe extern "system" fn(u32, u32) -> HResult;
pub type InitializeApiImplExFn = unsafe extern "system" fn(u32, u32, i8) -> HResult;
pub type InitializeApiImplEx2Fn =
    unsafe extern "system" fn(u32, u32, i8, *mut c_void) -> HResult;
pub type QueryApiImplFn =
    unsafe extern "system" fn(*const Guid, *const Guid, *mut *mut c_void) -> HResult;
pub type UninitializeApiImplFn = unsafe extern "system" fn() -> HResult;
pub type XErrorReportFn = unsafe extern "system" fn(HResult, *const c_char) -> HResult;
pub type XGameRuntimeInitializeFn = unsafe extern "system" fn() -> HResult;
pub type XGameRuntimeUninitializeFn = unsafe extern "system" fn();
