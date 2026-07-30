// SPDX-License-Identifier: LGPL-2.1-or-later

use std::sync::OnceLock;

use crate::error::ProxyError;

pub const NATIVE_RUNTIME_ENV: &str = "BMCBL_NATIVE_XGAMERUNTIME";

#[cfg(windows)]
mod platform {
    use std::{
        env,
        os::windows::ffi::OsStrExt,
        path::{Path, PathBuf},
    };

    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::LibraryLoader::{
            GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,
            LOAD_LIBRARY_SEARCH_SYSTEM32,
        },
    };

    use crate::error::ProxyError;

    use super::NATIVE_RUNTIME_ENV;

    #[derive(Debug)]
    pub struct NativeRuntime {
        module: isize,
        path: PathBuf,
    }

    impl NativeRuntime {
        pub fn load() -> Result<Self, ProxyError> {
            let path = env::var_os(NATIVE_RUNTIME_ENV)
                .map(PathBuf::from)
                .ok_or(ProxyError::MissingEnvironment(NATIVE_RUNTIME_ENV))?;

            validate_native_path(&path)?;

            let wide = to_wide(&path);
            let module = unsafe {
                LoadLibraryExW(
                    wide.as_ptr(),
                    0,
                    LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
                )
            };

            if module == 0 {
                return Err(ProxyError::NativeLoad {
                    path,
                    code: unsafe { GetLastError() },
                });
            }

            Ok(Self { module, path })
        }

        pub fn proc_address(&self, name: &'static [u8]) -> Result<usize, ProxyError> {
            debug_assert_eq!(name.last(), Some(&0));

            let proc = unsafe { GetProcAddress(self.module, name.as_ptr()) };
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

pub fn runtime() -> Result<&'static NativeRuntime, &'static ProxyError> {
    match NATIVE_RUNTIME.get_or_init(NativeRuntime::load) {
        Ok(runtime) => Ok(runtime),
        Err(error) => Err(error),
    }
}
