// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::{Mutex, OnceLock};

use crate::error::ProxyError;

pub const NATIVE_RUNTIME_ENV: &str = "BMCBL_NATIVE_XGAMERUNTIME";
pub const NATIVE_RUNTIME_FILENAME: &str = "xgameruntime_o.dll";
pub const SYSTEM_RUNTIME_PATH: &str = r"C:\Windows\System32\xgameruntime.dll";

#[cfg(windows)]
mod platform {
    use std::{
        env,
        io::Write,
        os::windows::ffi::OsStrExt,
        path::{Path, PathBuf},
    };

    use windows_sys::Win32::{
        Foundation::{GetLastError, HMODULE},
        System::{
            Diagnostics::Debug::OutputDebugStringW,
            LibraryLoader::{
                FreeLibrary, GetProcAddress, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
                LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryA, LoadLibraryExW,
            },
        },
    };

    use crate::error::ProxyError;

    use super::{NATIVE_RUNTIME_ENV, NATIVE_RUNTIME_FILENAME, SYSTEM_RUNTIME_PATH};

    const LOG_PREFIX: &str = "[xgameruntime-proxy] ";

    #[derive(Debug)]
    pub struct NativeRuntime {
        module: usize,
        path: PathBuf,
    }

    impl NativeRuntime {
        pub fn load() -> Result<Self, ProxyError> {
            debug_log("开始加载 Microsoft 原生 xgameruntime");

            if let Some(path) = env::var_os(NATIVE_RUNTIME_ENV) {
                let path = PathBuf::from(path);
                debug_log(&format!("尝试环境变量覆盖路径: {}", path.display()));
                match Self::load_absolute(path, true) {
                    Ok(runtime) => return Ok(runtime),
                    Err(error) => debug_log(&format!(
                        "环境变量覆盖路径加载失败: {error}; 继续尝试默认代理布局"
                    )),
                }
            } else {
                debug_log("未设置 BMCBL_NATIVE_XGAMERUNTIME，使用默认代理布局");
            }

            match Self::load_proxy_sibling() {
                Ok(runtime) => return Ok(runtime),
                Err(error) => debug_log(&format!(
                    "加载同目录 {NATIVE_RUNTIME_FILENAME} 失败: {error}; 尝试系统 Runtime"
                )),
            }

            let system_path = PathBuf::from(SYSTEM_RUNTIME_PATH);
            debug_log(&format!("尝试系统 Runtime: {}", system_path.display()));
            match Self::load_absolute(system_path, false) {
                Ok(runtime) => Ok(runtime),
                Err(error) => {
                    debug_log(&format!("系统 Runtime 加载失败: {error}"));
                    Err(error)
                }
            }
        }

        fn load_proxy_sibling() -> Result<Self, ProxyError> {
            let path = PathBuf::from(NATIVE_RUNTIME_FILENAME);
            debug_log(&format!("尝试加载同目录 {NATIVE_RUNTIME_FILENAME}"));

            let module = unsafe { LoadLibraryA(b"xgameruntime_o.dll\0".as_ptr()) };
            if module.is_null() {
                let code = unsafe { GetLastError() };
                return Err(ProxyError::NativeLoad { path, code });
            }

            let runtime = Self {
                module: module as usize,
                path,
            };
            runtime.log_loaded();
            Ok(runtime)
        }

        fn load_absolute(path: PathBuf, validate_path: bool) -> Result<Self, ProxyError> {
            if validate_path {
                validate_native_path(&path)?;
            }

            let wide = to_wide(&path);
            let module = unsafe {
                LoadLibraryExW(
                    wide.as_ptr(),
                    core::ptr::null_mut(),
                    LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
                )
            };
            if module.is_null() {
                let code = unsafe { GetLastError() };
                return Err(ProxyError::NativeLoad { path, code });
            }

            let runtime = Self {
                module: module as usize,
                path,
            };
            runtime.log_loaded();
            Ok(runtime)
        }

        fn log_loaded(&self) {
            debug_log(&format!(
                "原生 Runtime 加载成功: {} (HMODULE=0x{:X})",
                self.path.display(),
                self.module
            ));
        }

        pub fn proc_address(&self, name: &'static [u8]) -> Result<usize, ProxyError> {
            debug_assert_eq!(name.last(), Some(&0));

            let module = self.module as HMODULE;
            let proc = unsafe { GetProcAddress(module, name.as_ptr()) };
            let Some(proc) = proc else {
                let printable = std::str::from_utf8(&name[..name.len().saturating_sub(1)])
                    .unwrap_or("<invalid export name>");
                debug_log(&format!(
                    "原生 Runtime 缺少导出 {printable}: {}",
                    self.path.display()
                ));
                return Err(ProxyError::MissingExport(printable.to_owned()));
            };

            Ok(proc as usize)
        }

        pub unsafe fn unload(&self) {
            let module = self.module as HMODULE;
            if !module.is_null() {
                debug_log(&format!(
                    "释放原生 Runtime: {} (HMODULE=0x{:X})",
                    self.path.display(),
                    self.module
                ));
                unsafe {
                    FreeLibrary(module);
                }
            }
        }

        pub fn path(&self) -> &Path {
            &self.path
        }
    }

    pub fn debug_log(message: &str) {
        let line = format!("{LOG_PREFIX}{message}");

        let mut stderr = std::io::stderr().lock();
        let _ = writeln!(stderr, "{line}");

        let wide = line
            .encode_utf16()
            .chain(std::iter::once(b'\n' as u16))
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        unsafe {
            OutputDebugStringW(wide.as_ptr());
        }
    }

    fn validate_native_path(path: &Path) -> Result<(), ProxyError> {
        if !path.is_absolute() {
            return Err(ProxyError::NativePathNotAbsolute(path.to_path_buf()));
        }
        if !path.is_file() {
            return Err(ProxyError::NativePathMissing(path.to_path_buf()));
        }
        Ok(())
    }

    fn to_wide(path: &Path) -> Vec<u16> {
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }
}

#[cfg(not(windows))]
mod platform {
    use crate::error::ProxyError;

    #[derive(Debug)]
    pub struct NativeRuntime;

    impl NativeRuntime {
        pub fn load() -> Result<Self, ProxyError> {
            Err(ProxyError::UnsupportedPlatform)
        }

        pub fn proc_address(&self, _name: &'static [u8]) -> Result<usize, ProxyError> {
            Err(ProxyError::UnsupportedPlatform)
        }
    }
}

pub use platform::NativeRuntime;

static NATIVE_RUNTIME: OnceLock<NativeRuntime> = OnceLock::new();
static NATIVE_RUNTIME_INIT: Mutex<()> = Mutex::new(());

#[cfg(windows)]
pub fn preload() {
    platform::debug_log("DLL_PROCESS_ATTACH: 开始同步预加载原生 Runtime");
    match runtime() {
        Ok(runtime) => platform::debug_log(&format!(
            "DLL_PROCESS_ATTACH: 预加载完成，转发目标为 {}",
            runtime.path().display()
        )),
        Err(error) => platform::debug_log(&format!(
            "DLL_PROCESS_ATTACH: 预加载失败: {error}; 代理继续装载，后续调用将重试"
        )),
    }
}

#[cfg(windows)]
pub unsafe fn unload() {
    if let Some(runtime) = NATIVE_RUNTIME.get() {
        unsafe {
            runtime.unload();
        }
    } else {
        platform::debug_log("DLL_PROCESS_DETACH: 原生 Runtime 未加载，无需释放");
    }
}

pub fn runtime() -> Result<&'static NativeRuntime, ProxyError> {
    if let Some(runtime) = NATIVE_RUNTIME.get() {
        return Ok(runtime);
    }

    let _guard = NATIVE_RUNTIME_INIT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if let Some(runtime) = NATIVE_RUNTIME.get() {
        return Ok(runtime);
    }

    let runtime = NativeRuntime::load()?;
    let _ = NATIVE_RUNTIME.set(runtime);

    Ok(NATIVE_RUNTIME
        .get()
        .expect("native runtime is initialized before returning"))
}
