// SPDX-License-Identifier: LGPL-2.1-or-later
//
// The Xbox proof-of-possession payload layout follows WineGDK DeviceAuth.c.
// Hashing is implemented in Rust; the persistent P-256 private key remains in
// the Windows CNG current-user key store and is referenced only by key name.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::abi::HResult;

use super::abi::{E_INVALIDARG, E_NOTIMPL};

const SIGNATURE_POLICY_VERSION: u32 = 1;
const WINDOWS_TO_UNIX_EPOCH_SECONDS: u64 = 11_644_473_600;
const FILETIME_TICKS_PER_SECOND: u64 = 10_000_000;
const P256_SIGNATURE_SIZE: usize = 64;
const SIGNATURE_HEADER_SIZE: usize = 4 + 8 + P256_SIGNATURE_SIZE;

pub struct RequestSignatureInput<'a> {
    pub cng_key_name: &'a str,
    pub method: &'a str,
    pub request_target: &'a str,
    pub authorization: &'a str,
    pub policy_header_values: &'a [&'a str],
    pub body: &'a [u8],
}

pub fn sign_request(input: RequestSignatureInput<'_>) -> Result<String, HResult> {
    validate_input(&input)?;

    let timestamp = current_filetime();
    let mut digest = hash_request(&input, timestamp);
    let signature_result = sign_hash_with_cng(input.cng_key_name, &digest);
    digest.zeroize();
    let mut signature = signature_result?;

    let mut header = [0u8; SIGNATURE_HEADER_SIZE];
    header[..4].copy_from_slice(&SIGNATURE_POLICY_VERSION.to_be_bytes());
    header[4..12].copy_from_slice(&timestamp.to_be_bytes());
    header[12..].copy_from_slice(&signature);
    signature.zeroize();

    let encoded = STANDARD.encode(header);
    header.zeroize();
    Ok(encoded)
}

fn validate_input(input: &RequestSignatureInput<'_>) -> Result<(), HResult> {
    if input.cng_key_name.is_empty()
        || input.method.is_empty()
        || input.request_target.is_empty()
        || !input.method.is_ascii()
        || !input.request_target.is_ascii()
    {
        return Err(E_INVALIDARG);
    }
    Ok(())
}

fn hash_request(input: &RequestSignatureInput<'_>, timestamp: u64) -> [u8; 32] {
    let mut uppercase_method = input
        .method
        .bytes()
        .map(|byte| byte.to_ascii_uppercase())
        .collect::<Vec<_>>();

    let mut hasher = Sha256::new();
    hasher.update(SIGNATURE_POLICY_VERSION.to_be_bytes());
    hasher.update([0]);
    hasher.update(timestamp.to_be_bytes());
    hasher.update([0]);
    hasher.update(&uppercase_method);
    hasher.update([0]);
    hasher.update(input.request_target.as_bytes());
    hasher.update([0]);
    hasher.update(input.authorization.as_bytes());
    hasher.update([0]);
    for value in input.policy_header_values {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    hasher.update(input.body);
    hasher.update([0]);
    uppercase_method.zeroize();

    hasher.finalize().into()
}

fn current_filetime() -> u64 {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    WINDOWS_TO_UNIX_EPOCH_SECONDS
        .saturating_add(duration.as_secs())
        .saturating_mul(FILETIME_TICKS_PER_SECOND)
        .saturating_add(u64::from(duration.subsec_nanos()) / 100)
}

#[cfg(windows)]
fn sign_hash_with_cng(key_name: &str, digest: &[u8; 32]) -> Result<[u8; 64], HResult> {
    use windows_sys::Win32::Security::Cryptography::{
        NCRYPT_KEY_HANDLE, NCRYPT_PROV_HANDLE, NCryptFreeObject, NCryptOpenKey,
        NCryptOpenStorageProvider, NCryptSignHash,
    };

    const PROVIDER_NAME: &str = "Microsoft Software Key Storage Provider";

    struct NcryptHandle(usize);

    impl Drop for NcryptHandle {
        fn drop(&mut self) {
            if self.0 != 0 {
                unsafe { NCryptFreeObject(self.0) };
            }
        }
    }

    let provider_name = PROVIDER_NAME
        .encode_utf16()
        .chain(core::iter::once(0))
        .collect::<Vec<_>>();
    let key_name = key_name
        .encode_utf16()
        .chain(core::iter::once(0))
        .collect::<Vec<_>>();

    let mut provider: NCRYPT_PROV_HANDLE = 0;
    let status = unsafe { NCryptOpenStorageProvider(&mut provider, provider_name.as_ptr(), 0) };
    if status != 0 {
        return Err(status);
    }
    let provider = NcryptHandle(provider);

    let mut key: NCRYPT_KEY_HANDLE = 0;
    let status = unsafe { NCryptOpenKey(provider.0, &mut key, key_name.as_ptr(), 0, 0) };
    if status != 0 {
        return Err(status);
    }
    let key = NcryptHandle(key);

    let mut signature = [0u8; P256_SIGNATURE_SIZE];
    let mut written = 0u32;
    let status = unsafe {
        NCryptSignHash(
            key.0,
            core::ptr::null_mut(),
            digest.as_ptr() as *mut u8,
            digest.len() as u32,
            signature.as_mut_ptr(),
            signature.len() as u32,
            &mut written,
            0,
        )
    };
    if status != 0 {
        signature.zeroize();
        return Err(status);
    }
    if written as usize != signature.len() {
        signature.zeroize();
        return Err(E_INVALIDARG);
    }

    Ok(signature)
}

#[cfg(not(windows))]
fn sign_hash_with_cng(_key_name: &str, _digest: &[u8; 32]) -> Result<[u8; 64], HResult> {
    Err(E_NOTIMPL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filetime_uses_windows_epoch() {
        assert!(current_filetime() > 116_444_736_000_000_000);
    }

    #[test]
    fn rejects_non_ascii_request_components() {
        let result = sign_request(RequestSignatureInput {
            cng_key_name: "BMCBL.XboxDevice.test",
            method: "GÉT",
            request_target: "/",
            authorization: "XBL3.0 x=1;token",
            policy_header_values: &[],
            body: &[],
        });
        assert_eq!(result, Err(E_INVALIDARG));
    }

    #[test]
    fn request_hash_matches_winegdk_layout() {
        let input = RequestSignatureInput {
            cng_key_name: "BMCBL.XboxDevice.test",
            method: "post",
            request_target: "/path?q=1",
            authorization: "XBL3.0 x=1;token",
            policy_header_values: &[],
            body: b"abc",
        };
        assert_eq!(
            hash_request(&input, 132_537_600_000_000_000),
            [
                112, 236, 215, 162, 14, 158, 58, 119, 204, 59, 148, 220, 224, 77, 173, 127,
                31, 122, 72, 213, 197, 107, 46, 157, 202, 114, 8, 198, 118, 103, 129, 78,
            ]
        );
    }

    #[test]
    fn signature_header_shape_matches_xbox_contract() {
        assert_eq!(SIGNATURE_HEADER_SIZE, 76);
        assert_eq!(STANDARD.encode([0u8; SIGNATURE_HEADER_SIZE]).len(), 104);
    }

    #[cfg(windows)]
    #[test]
    fn signs_and_verifies_with_temporary_cng_key() {
        use windows_sys::Win32::Security::Cryptography::{
            NCRYPT_KEY_HANDLE, NCRYPT_OVERWRITE_KEY_FLAG, NCRYPT_PROV_HANDLE,
            NCryptCreatePersistedKey, NCryptDeleteKey, NCryptFinalizeKey, NCryptFreeObject,
            NCryptOpenStorageProvider, NCryptVerifySignature,
        };

        const PROVIDER_NAME: &str = "Microsoft Software Key Storage Provider";
        const ALGORITHM_NAME: &str = "ECDSA_P256";

        struct TemporaryKey {
            provider: NCRYPT_PROV_HANDLE,
            key: NCRYPT_KEY_HANDLE,
        }

        impl Drop for TemporaryKey {
            fn drop(&mut self) {
                if self.key != 0 {
                    unsafe { NCryptDeleteKey(self.key, 0) };
                    self.key = 0;
                }
                if self.provider != 0 {
                    unsafe { NCryptFreeObject(self.provider) };
                    self.provider = 0;
                }
            }
        }

        let provider_name = PROVIDER_NAME
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect::<Vec<_>>();
        let algorithm_name = ALGORITHM_NAME
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect::<Vec<_>>();
        let key_name = format!(
            "BMCBL.XboxDevice.ci-{}-{}",
            std::process::id(),
            current_filetime()
        );
        let key_name_utf16 = key_name
            .encode_utf16()
            .chain(core::iter::once(0))
            .collect::<Vec<_>>();

        let mut temporary_key = TemporaryKey {
            provider: 0,
            key: 0,
        };
        assert_eq!(
            unsafe {
                NCryptOpenStorageProvider(
                    &mut temporary_key.provider,
                    provider_name.as_ptr(),
                    0,
                )
            },
            0
        );
        assert_eq!(
            unsafe {
                NCryptCreatePersistedKey(
                    temporary_key.provider,
                    &mut temporary_key.key,
                    algorithm_name.as_ptr(),
                    key_name_utf16.as_ptr(),
                    0,
                    NCRYPT_OVERWRITE_KEY_FLAG,
                )
            },
            0
        );
        assert_eq!(unsafe { NCryptFinalizeKey(temporary_key.key, 0) }, 0);

        let input = RequestSignatureInput {
            cng_key_name: &key_name,
            method: "POST",
            request_target: "/path?q=1",
            authorization: "XBL3.0 x=1;token",
            policy_header_values: &[],
            body: b"abc",
        };
        let encoded = sign_request(input).expect("CNG signing should succeed");
        let header = STANDARD
            .decode(encoded)
            .expect("signature header must be valid base64");
        assert_eq!(header.len(), SIGNATURE_HEADER_SIZE);
        assert_eq!(
            u32::from_be_bytes(header[..4].try_into().unwrap()),
            SIGNATURE_POLICY_VERSION
        );

        let timestamp = u64::from_be_bytes(header[4..12].try_into().unwrap());
        let verify_input = RequestSignatureInput {
            cng_key_name: &key_name,
            method: "POST",
            request_target: "/path?q=1",
            authorization: "XBL3.0 x=1;token",
            policy_header_values: &[],
            body: b"abc",
        };
        let digest = hash_request(&verify_input, timestamp);
        assert_eq!(
            unsafe {
                NCryptVerifySignature(
                    temporary_key.key,
                    core::ptr::null_mut(),
                    digest.as_ptr() as *mut u8,
                    digest.len() as u32,
                    header[12..].as_ptr() as *mut u8,
                    P256_SIGNATURE_SIZE as u32,
                    0,
                )
            },
            0
        );
    }
}
