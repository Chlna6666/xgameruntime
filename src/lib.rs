// SPDX-License-Identifier: LGPL-2.1-or-later

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(dead_code, non_snake_case, clippy::missing_safety_doc)]

#[cfg(all(windows, not(target_arch = "x86_64")))]
compile_error!("the experimental XUser vtable currently supports Windows x64 only");

mod abi;
mod error;
mod intercept;
mod native;
mod preauth;
mod profile;
mod state;
mod xasync;
mod xuser;

use core::ffi::{c_char, c_void};

use abi::{
    DllCanUnloadNowFn, DllGetClassObjectFn, E_FAIL, E_POINTER, Guid, HResult,
    InitializeApiImplEx2Fn, InitializeApiImplExFn, InitializeApiImplFn, QueryApiImplFn, S_FALSE,
    UninitializeApiImplFn, XErrorReportFn, XGameRuntimeInitializeFn, XGameRuntimeUninitializeFn,
};

#[cfg(windows)]
use windows_sys::Win32::{Foundation::HMODULE, System::LibraryLoader::DisableThreadLibraryCalls};

#[cfg(windows)]
const DLL_PROCESS_DETACH: u32 = 0;
#[cfg(windows)]
const DLL_PROCESS_ATTACH: u32 = 1;

#[cfg(windows)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    instance: HMODULE,
    reason: u32,
    reserved: *mut c_void,
) -> i32 {
    match reason {
        DLL_PROCESS_ATTACH => {
            let _ = unsafe { DisableThreadLibraryCalls(instance) };

            // The custom XUser switch is also the proxy activation gate. When
            // it is absent, do not perform any original-runtime loading or
            // profile initialization under the loader lock. Export calls are
            // forwarded lazily to the Microsoft runtime instead.
            if xuser::enabled() {
                native::preload();
            } else {
                native::passthrough_attach();
            }
        }
        DLL_PROCESS_DETACH if reserved.is_null() => unsafe {
            native::unload();
        },
        _ => {}
    }

    1
}

fn native_symbol(name: &'static [u8]) -> Result<usize, HResult> {
    native::runtime()
        .map_err(|_| E_FAIL)?
        .proc_address(name)
        .map_err(|_| E_FAIL)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllCanUnloadNow() -> HResult {
    let Ok(address) = native_symbol(b"DllCanUnloadNow\0") else {
        return S_FALSE;
    };
    let function: DllCanUnloadNowFn = unsafe { core::mem::transmute(address) };
    unsafe { function() }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    class_id: *const Guid,
    interface_id: *const Guid,
    out: *mut *mut c_void,
) -> HResult {
    if class_id.is_null() || interface_id.is_null() || out.is_null() {
        return E_POINTER;
    }
    unsafe { out.write(core::ptr::null_mut()) };

    let Ok(address) = native_symbol(b"DllGetClassObject\0") else {
        return E_FAIL;
    };
    let function: DllGetClassObjectFn = unsafe { core::mem::transmute(address) };
    unsafe { function(class_id, interface_id, out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn InitializeApiImpl(gdk_version: u32, gs_version: u32) -> HResult {
    let Ok(address) = native_symbol(b"InitializeApiImpl\0") else {
        return E_FAIL;
    };
    let function: InitializeApiImplFn = unsafe { core::mem::transmute(address) };
    unsafe { function(gdk_version, gs_version) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn InitializeApiImplEx(
    gdk_version: u32,
    gs_version: u32,
    mode: i8,
) -> HResult {
    let Ok(address) = native_symbol(b"InitializeApiImplEx\0") else {
        return E_FAIL;
    };
    let function: InitializeApiImplExFn = unsafe { core::mem::transmute(address) };
    unsafe { function(gdk_version, gs_version, mode) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn InitializeApiImplEx2(
    gdk_version: u32,
    gs_version: u32,
    mode: i8,
    options: *mut c_void,
) -> HResult {
    let Ok(address) = native_symbol(b"InitializeApiImplEx2\0") else {
        return E_FAIL;
    };
    let function: InitializeApiImplEx2Fn = unsafe { core::mem::transmute(address) };
    unsafe { function(gdk_version, gs_version, mode, options) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn QueryApiImpl(
    runtime_class_id: *const Guid,
    interface_id: *const Guid,
    out: *mut *mut c_void,
) -> HResult {
    if runtime_class_id.is_null() || interface_id.is_null() || out.is_null() {
        return E_POINTER;
    }
    unsafe { out.write(core::ptr::null_mut()) };

    if let Some(result) = unsafe { intercept::query_api(runtime_class_id, interface_id, out) } {
        return result;
    }

    let Ok(address) = native_symbol(b"QueryApiImpl\0") else {
        return E_FAIL;
    };
    let function: QueryApiImplFn = unsafe { core::mem::transmute(address) };
    unsafe { function(runtime_class_id, interface_id, out) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn UninitializeApiImpl() -> HResult {
    let Ok(address) = native_symbol(b"UninitializeApiImpl\0") else {
        return E_FAIL;
    };
    let function: UninitializeApiImplFn = unsafe { core::mem::transmute(address) };
    unsafe { function() }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn XErrorReport(status: HResult, message: *const c_char) -> HResult {
    let Ok(address) = native_symbol(b"XErrorReport\0") else {
        return E_FAIL;
    };
    let function: XErrorReportFn = unsafe { core::mem::transmute(address) };
    unsafe { function(status, message) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn XGameRuntimeInitialize() -> HResult {
    let Ok(address) = native_symbol(b"XGameRuntimeInitialize\0") else {
        return E_FAIL;
    };
    let function: XGameRuntimeInitializeFn = unsafe { core::mem::transmute(address) };
    unsafe { function() }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn XGameRuntimeUninitialize() {
    let Ok(address) = native_symbol(b"XGameRuntimeUninitialize\0") else {
        return;
    };
    let function: XGameRuntimeUninitializeFn = unsafe { core::mem::transmute(address) };
    unsafe { function() }
}
