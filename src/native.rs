// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::OnceLock;

use crate::error::ProxyError;

pub const NATIVE_RUNTIME_ENV: &str = "BMCBL_NATIVE_XGAMERUNTIME";
pub const NATIVE_RUNTIME_FILENAME: &str = "xgameruntime_o.dll";

#[cfg(windows)]
mod platform {
    use std::{
        env,
        ffi::OsString,
        os::windows::ffi::{OsStrExt, OsStringExt},
        path::{Path, PathBuf},
    };

    use windows_sys::Win32::{
        Foundation::{GetLastError, HMODULE},
        System::LibraryLoader::{
            GetModuleFileNameW, GetProcAddress, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
            LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
        },
    };

    use crate::error::ProxyError;

    use super::{NATIVE_RUNTIME_ENV, NATIVE_RUNTIME_FILENAME};

    const MAX_WINDOWS_PATH_CHARS: usize = 32_768;

    #[derive(Debug)]
    pub struct NativeRuntime {
        module: usize,
        path: PathBuf,
    }

    impl NativeRuntime {
        pub fn load() -> Result<Self, ProxyError> {
            let path = env::var_os(NATIVE_RUNTIME_ENV)
                .map(PathBuf::from)
                .ok_or(ProxyError::MissingEnvironment(NATIVE_RUNTIME_ENV))?;

            Self::load_path(path)
        }

        pub fn load_for_proxy(proxy_module: usize) -> Result<Self, ProxyError> {
            let path = match env::var_os(NATIVE_RUNTIME_ENV) {
                Some(path) => PathBuf::from(path),
                None => sibling_runtime_path(proxy_module as HMODULE)?,
            };

            Self::load_path(path)
        }

        fn load_path(path: PathBuf) -> Result<Self, ProxyError> {
            validate_native_path(&path)?;

            let wide = to_wide(&path);
            let module = unsafe {
                LoadLibraryExW(
                    wide.as_ptr(),
                    core::ptr::null_mut(),
                    LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
                )
            };
            if module.is_null() {
                return Err(ProxyError::NativeLoad {
                    path,
                    code: unsafe { GetLastError() },
                });
            }

            Ok(Self {
                module: module as usize,
                path,
            })
        }

        pub fn proc_address(&self, name: &'static [u8]) -> Result<usize, ProxyError> {
            debug_assert_eq!(name.last(), Some(&0));

            let module = self.module as HMODULE;
            let proc = unsafe { GetProcAddress(module, name.as_ptr()) };
            let Some(proc) = proc else {
                let printable = std::str::from_utf8(&name[..name.len().saturating_sub(1)])
                    .unwrap_or("<invalid export name>");
                return Err(ProxyError::MissingExport(printable.to_owned()));
            };

            Ok(proc as usize)
        }

        #[allow(dead_code)]
        pub fn path(&self) -> &Path {
            &self.path
        }
    }

    fn sibling_runtime_path(proxy_module: HMODULE) -> Result<PathBuf, ProxyError> {
        let mut buffer = vec![0_u16; MAX_WINDOWS_PATH_CHARS];
        let length = unsafe {
            GetModuleFileNameW(
                proxy_module,
                buffer.as_mut_ptr(),
                u32::try_from(buffer.len()).expect("Windows path buffer fits in u32"),
            )
        } as usize;

        if length == 0 || length >= buffer.len() {
            return Err(ProxyError::ProxyModulePath {
                code: unsafe { GetLastError() },
            });
        }

        buffer.truncate(length);
        let mut path = PathBuf::from(OsString::from_wide(&buffer));
        path.set_file_name(NATIVE_RUNTIME_FILENAME);
        Ok(path)
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

static NATIVE_RUNTIME: OnceLock<Result<NativeRuntime, ProxyError>> = OnceLock::new();

#[cfg(windows)]
pub fn preload(proxy_module: usize) -> Result<(), &'static ProxyError> {
    match NATIVE_RUNTIME.get_or_init(|| NativeRuntime::load_for_proxy(proxy_module)) {
        Ok(_) => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn runtime() -> Result<&'static NativeRuntime, &'static ProxyError> {
    match NATIVE_RUNTIME.get_or_init(NativeRuntime::load) {
        Ok(runtime) => Ok(runtime),
        Err(error) => Err(error),
    }
}
