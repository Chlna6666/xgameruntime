// SPDX-License-Identifier: LGPL-2.1-or-later
//
// ABI layout and request-routing behavior are adapted from Wine/WineGDK's
// xuser.idl and XUser token provider. Existing LGPL notices are preserved by
// this repository's LICENSE and NOTICE.md.

use core::ffi::{c_char, c_void};
use std::{
    ffi::CStr,
    time::{SystemTime, UNIX_EPOCH},
};

use zeroize::Zeroize;

use crate::{
    abi::{E_FAIL, E_POINTER, HResult, S_OK},
    preauth::{TokenEnvelope, XboxPreauth, MIN_TOKEN_REMAINING_SECONDS},
    state,
    xasync::{self, XAsyncOp, XAsyncProviderData},
};

use super::{
    abi::{E_INVALIDARG, E_NOT_SUFFICIENT_BUFFER, XAsyncBlock, XUserHandle},
    object,
};

const TOKEN_OPTIONS_MASK: u32 = 0x03;
const MAX_UTF16_INPUT_UNITS: usize = 32 * 1024;
const TOKEN_NAME_ANSI: &[u8] = b"XUserGetTokenAndSignatureAsync\0";
const TOKEN_NAME_UTF16: &[u8] = b"XUserGetTokenAndSignatureUtf16Async\0";
static TOKEN_IDENTITY_ANSI: u8 = 0x41;
static TOKEN_IDENTITY_UTF16: u8 = 0x57;

#[repr(C)]
pub struct XUserGetTokenAndSignatureData {
    pub token_size: usize,
    pub signature_size: usize,
    pub token: *const c_char,
    pub signature: *const c_char,
}

#[repr(C)]
pub struct XUserGetTokenAndSignatureHttpHeader {
    pub name: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
pub struct XUserGetTokenAndSignatureUtf16Data {
    pub token_count: usize,
    pub signature_count: usize,
    pub token: *const u16,
    pub signature: *const u16,
}

#[repr(C)]
pub struct XUserGetTokenAndSignatureUtf16HttpHeader {
    pub name: *const u16,
    pub value: *const u16,
}

struct TokenContext {
    utf16: bool,
    authorization: Vec<u8>,
    authorization_utf16: Vec<u16>,
}

impl TokenContext {
    fn new(user: XUserHandle, url: &str, utf16: bool) -> Result<Self, HResult> {
        let profile = profile_for_user(user)?;
        let token = select_token(&profile.preauth.xbox, url).ok_or(E_FAIL)?;
        if token.expires_at_epoch
            <= now_epoch().saturating_add(MIN_TOKEN_REMAINING_SECONDS)
        {
            return Err(E_FAIL);
        }

        let authorization_text = format!(
            "XBL3.0 x={};{}",
            token.user_hash,
            token.token.expose()
        );
        let mut authorization = authorization_text.as_bytes().to_vec();
        authorization.push(0);
        let mut authorization_utf16 = authorization_text.encode_utf16().collect::<Vec<_>>();
        authorization_utf16.push(0);

        Ok(Self {
            utf16,
            authorization,
            authorization_utf16,
        })
    }

    fn required_size(&self) -> Option<usize> {
        if self.utf16 {
            self.authorization_utf16
                .len()
                .checked_mul(core::mem::size_of::<u16>())?
                .checked_add(core::mem::size_of::<XUserGetTokenAndSignatureUtf16Data>())
        } else {
            self.authorization
                .len()
                .checked_add(core::mem::size_of::<XUserGetTokenAndSignatureData>())
        }
    }
}

impl Drop for TokenContext {
    fn drop(&mut self) {
        self.authorization.zeroize();
        self.authorization_utf16.zeroize();
    }
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn profile_for_user(
    user: XUserHandle,
) -> Result<&'static crate::profile::LaunchProfile, HResult> {
    if user.is_null() {
        return Err(E_POINTER);
    }
    let Some(provider) = object::provider_interface() else {
        return Err(E_FAIL);
    };
    if provider != user {
        return Err(E_INVALIDARG);
    }
    state::selected_profile().ok_or(E_FAIL)
}

fn token_identity(utf16: bool) -> *const c_void {
    if utf16 {
        (&TOKEN_IDENTITY_UTF16 as *const u8).cast()
    } else {
        (&TOKEN_IDENTITY_ANSI as *const u8).cast()
    }
}

fn token_name(utf16: bool) -> *const c_char {
    if utf16 {
        TOKEN_NAME_UTF16.as_ptr().cast()
    } else {
        TOKEN_NAME_ANSI.as_ptr().cast()
    }
}

fn select_token<'a>(xbox: &'a XboxPreauth, url: &str) -> Option<&'a TokenEnvelope> {
    let relying_party = resolve_relying_party_for_url(url);
    [
        xbox.licensing.as_ref(),
        xbox.multiplayer.as_ref(),
        xbox.realms.as_ref(),
        xbox.sisu.as_ref(),
        Some(&xbox.xbox_live),
    ]
    .into_iter()
    .flatten()
    .find(|token| token.relying_party == relying_party)
}

fn resolve_relying_party_for_url(url: &str) -> &'static str {
    let Some(host) = url_host(url) else {
        return "http://xboxlive.com";
    };

    if host_matches(&host, "collections.mp.microsoft.com", false)
        || host_matches(&host, "purchase.mp.microsoft.com", false)
        || host_matches(&host, "displaycatalog.mp.microsoft.com", false)
        || host_matches(&host, "inventory.xboxlive.com", false)
        || host_matches(&host, "licensing.xboxlive.com", false)
    {
        return "http://licensing.xboxlive.com";
    }

    if host_matches(&host, "playfabapi.com", true) {
        return "https://b980a380.minecraft.playfabapi.com/";
    }

    if host_matches(&host, "multiplayer.minecraft.net", true) {
        return "https://multiplayer.minecraft.net/";
    }

    if host_matches(&host, "pocket.realms.minecraft.net", false)
        || host_matches(
            &host,
            "bedrock.frontend.realms.minecraft-services.net",
            false,
        )
        || host_matches(
            &host,
            "bedrock.frontendlegacy.realms.minecraft-services.net",
            false,
        )
    {
        return "https://pocket.realms.minecraft.net/";
    }

    "http://xboxlive.com"
}

fn url_host(url: &str) -> Option<String> {
    let authority = url.split_once("://")?.1;
    let authority_end = authority
        .char_indices()
        .find_map(|(index, character)| {
            matches!(character, '/' | '?' | '#').then_some(index)
        })
        .unwrap_or(authority.len());
    let authority = &authority[..authority_end];
    let host_port = authority.rsplit_once('@').map_or(authority, |(_, host)| host);

    let host = if let Some(bracketed) = host_port.strip_prefix('[') {
        bracketed.split_once(']')?.0
    } else {
        host_port.split_once(':').map_or(host_port, |(host, _)| host)
    };

    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

fn host_matches(host: &str, expected: &str, allow_subdomains: bool) -> bool {
    host.eq_ignore_ascii_case(expected)
        || (allow_subdomains
            && host.len() > expected.len()
            && host.ends_with(expected)
            && host.as_bytes()[host.len() - expected.len() - 1] == b'.')
}

unsafe fn utf16_to_string(value: *const u16) -> Result<String, HResult> {
    if value.is_null() {
        return Err(E_POINTER);
    }

    let mut length = 0usize;
    while length < MAX_UTF16_INPUT_UNITS {
        if unsafe { value.add(length).read() } == 0 {
            let units = unsafe { core::slice::from_raw_parts(value, length) };
            return String::from_utf16(units).map_err(|_| E_INVALIDARG);
        }
        length += 1;
    }

    Err(E_INVALIDARG)
}

unsafe extern "system" fn token_provider(
    operation: XAsyncOp,
    provider_data: *const XAsyncProviderData,
) -> HResult {
    if provider_data.is_null() {
        return E_POINTER;
    }
    let provider_data = unsafe { &*provider_data };
    let context = provider_data.context.cast::<TokenContext>();
    if context.is_null() {
        return E_POINTER;
    }

    match operation {
        XAsyncOp::Begin => unsafe { xasync::schedule(provider_data.async_block, 0) },
        XAsyncOp::DoWork => {
            let Some(required_size) = (unsafe { &*context }).required_size() else {
                unsafe { xasync::complete(provider_data.async_block, E_FAIL, 0) };
                return S_OK;
            };
            unsafe { xasync::complete(provider_data.async_block, S_OK, required_size) };
            S_OK
        }
        XAsyncOp::GetResult => {
            let context = unsafe { &*context };
            let Some(required_size) = context.required_size() else {
                return E_FAIL;
            };
            if provider_data.buffer.is_null() || provider_data.buffer_size < required_size {
                return E_NOT_SUFFICIENT_BUFFER;
            }

            if context.utf16 {
                let data = provider_data
                    .buffer
                    .cast::<XUserGetTokenAndSignatureUtf16Data>();
                let token = unsafe { data.add(1).cast::<u16>() };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        context.authorization_utf16.as_ptr(),
                        token,
                        context.authorization_utf16.len(),
                    );
                    data.write(XUserGetTokenAndSignatureUtf16Data {
                        token_count: context.authorization_utf16.len()
                            * core::mem::size_of::<u16>(),
                        signature_count: 0,
                        token,
                        signature: core::ptr::null(),
                    });
                }
            } else {
                let data = provider_data.buffer.cast::<XUserGetTokenAndSignatureData>();
                let token = unsafe { data.add(1).cast::<u8>() };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        context.authorization.as_ptr(),
                        token,
                        context.authorization.len(),
                    );
                    data.write(XUserGetTokenAndSignatureData {
                        token_size: context.authorization.len(),
                        signature_size: 0,
                        token: token.cast(),
                        signature: core::ptr::null(),
                    });
                }
            }
            S_OK
        }
        XAsyncOp::Cancel => S_OK,
        XAsyncOp::Cleanup => {
            unsafe { drop(Box::from_raw(context)) };
            S_OK
        }
    }
}

unsafe fn begin_token_request(
    user: XUserHandle,
    options: u32,
    method: &str,
    url: &str,
    header_count: usize,
    headers: *const c_void,
    body_size: usize,
    body: *const c_void,
    async_block: *mut XAsyncBlock,
    utf16: bool,
) -> HResult {
    if user.is_null() || async_block.is_null() {
        return E_POINTER;
    }
    if (header_count != 0 && headers.is_null()) || (body_size != 0 && body.is_null()) {
        return E_POINTER;
    }
    if method.is_empty() || url.is_empty() || options & !TOKEN_OPTIONS_MASK != 0 {
        return E_INVALIDARG;
    }

    // ForceRefresh cannot mint a new token inside the injected process. The
    // launcher must provide a fresh short-lived preauth document before launch;
    // this path therefore reuses it only while its expiry margin remains valid.
    let context = match TokenContext::new(user, url, utf16) {
        Ok(context) => Box::into_raw(Box::new(context)),
        Err(error) => return error,
    };
    let result = unsafe {
        xasync::begin(
            async_block,
            context.cast(),
            token_identity(utf16),
            token_name(utf16),
            token_provider,
        )
    };
    if result < 0 {
        unsafe { drop(Box::from_raw(context)) };
    }
    result
}

pub unsafe extern "system" fn get_token_and_signature_async(
    _iface: *mut c_void,
    user: XUserHandle,
    options: u32,
    method: *const c_char,
    url: *const c_char,
    header_count: usize,
    headers: *const XUserGetTokenAndSignatureHttpHeader,
    body_size: usize,
    body: *const c_void,
    async_block: *mut XAsyncBlock,
) -> HResult {
    if method.is_null() || url.is_null() {
        return E_POINTER;
    }
    let method = match unsafe { CStr::from_ptr(method) }.to_str() {
        Ok(method) => method,
        Err(_) => return E_INVALIDARG,
    };
    let url = match unsafe { CStr::from_ptr(url) }.to_str() {
        Ok(url) => url,
        Err(_) => return E_INVALIDARG,
    };

    unsafe {
        begin_token_request(
            user,
            options,
            method,
            url,
            header_count,
            headers.cast(),
            body_size,
            body,
            async_block,
            false,
        )
    }
}

pub unsafe extern "system" fn get_token_and_signature_result_size(
    _iface: *mut c_void,
    async_block: *mut XAsyncBlock,
    size: *mut usize,
) -> HResult {
    unsafe { xasync::get_result_size(async_block, size) }
}

pub unsafe extern "system" fn get_token_and_signature_result(
    _iface: *mut c_void,
    async_block: *mut XAsyncBlock,
    size: usize,
    buffer: *mut c_void,
    data: *mut *mut XUserGetTokenAndSignatureData,
    used: *mut usize,
) -> HResult {
    if async_block.is_null() || buffer.is_null() || data.is_null() {
        return E_POINTER;
    }
    let result = unsafe {
        xasync::get_result(
            async_block,
            token_identity(false),
            size,
            buffer,
            used,
        )
    };
    if result >= 0 {
        unsafe { data.write(buffer.cast()) };
    }
    result
}

pub unsafe extern "system" fn get_token_and_signature_utf16_async(
    _iface: *mut c_void,
    user: XUserHandle,
    options: u32,
    method: *const u16,
    url: *const u16,
    header_count: usize,
    headers: *const XUserGetTokenAndSignatureUtf16HttpHeader,
    body_size: usize,
    body: *const c_void,
    async_block: *mut XAsyncBlock,
) -> HResult {
    let method = match unsafe { utf16_to_string(method) } {
        Ok(method) => method,
        Err(error) => return error,
    };
    let url = match unsafe { utf16_to_string(url) } {
        Ok(url) => url,
        Err(error) => return error,
    };

    unsafe {
        begin_token_request(
            user,
            options,
            &method,
            &url,
            header_count,
            headers.cast(),
            body_size,
            body,
            async_block,
            true,
        )
    }
}

pub unsafe extern "system" fn get_token_and_signature_utf16_result_size(
    _iface: *mut c_void,
    async_block: *mut XAsyncBlock,
    size: *mut usize,
) -> HResult {
    unsafe { xasync::get_result_size(async_block, size) }
}

pub unsafe extern "system" fn get_token_and_signature_utf16_result(
    _iface: *mut c_void,
    async_block: *mut XAsyncBlock,
    size: usize,
    buffer: *mut c_void,
    data: *mut *mut XUserGetTokenAndSignatureUtf16Data,
    used: *mut usize,
) -> HResult {
    if async_block.is_null() || buffer.is_null() || data.is_null() {
        return E_POINTER;
    }
    let result = unsafe {
        xasync::get_result(
            async_block,
            token_identity(true),
            size,
            buffer,
            used,
        )
    };
    if result >= 0 {
        unsafe { data.write(buffer.cast()) };
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_minecraft_service_hosts_to_expected_relying_parties() {
        assert_eq!(
            resolve_relying_party_for_url("https://multiplayer.minecraft.net/session"),
            "https://multiplayer.minecraft.net/"
        );
        assert_eq!(
            resolve_relying_party_for_url(
                "https://bedrock.frontend.realms.minecraft-services.net/worlds"
            ),
            "https://pocket.realms.minecraft.net/"
        );
        assert_eq!(
            resolve_relying_party_for_url("https://inventory.xboxlive.com/items"),
            "http://licensing.xboxlive.com"
        );
        assert_eq!(
            resolve_relying_party_for_url("https://foo.playfabapi.com/Client/Login"),
            "https://b980a380.minecraft.playfabapi.com/"
        );
        assert_eq!(
            resolve_relying_party_for_url("https://social.xboxlive.com/users"),
            "http://xboxlive.com"
        );
    }

    #[test]
    fn parses_hosts_without_trusting_credentials_or_ports() {
        assert_eq!(
            url_host("https://user:pass@Multiplayer.Minecraft.Net:443/path").as_deref(),
            Some("multiplayer.minecraft.net")
        );
        assert_eq!(url_host("invalid"), None);
    }

    #[test]
    fn result_structs_match_four_word_gdk_layout() {
        assert_eq!(
            core::mem::size_of::<XUserGetTokenAndSignatureData>(),
            4 * core::mem::size_of::<usize>()
        );
        assert_eq!(
            core::mem::size_of::<XUserGetTokenAndSignatureUtf16Data>(),
            4 * core::mem::size_of::<usize>()
        );
    }
}
