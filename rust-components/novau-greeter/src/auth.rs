//! PAM authentication via direct FFI to libpam.
//!
//! We don't use the `pam` crate because it pulls in `bindgen` (and thus
//! libclang) at build time. PAM's C API is tiny and stable, so we
//! declare just the symbols we need here.

use anyhow::{anyhow, Result};
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;

// ── PAM constants (from /usr/include/security/_pam_types.h) ─────────────
const PAM_SUCCESS: c_int = 0;
const PAM_PROMPT_ECHO_OFF: c_int = 1;
const PAM_PROMPT_ECHO_ON: c_int = 2;
const PAM_ERROR_MSG: c_int = 3;
const PAM_TEXT_INFO: c_int = 4;

// ── Opaque types ────────────────────────────────────────────────────────
#[repr(C)]
struct PamHandle { _private: [u8; 0] }

#[repr(C)]
struct PamMessage {
    msg_style: c_int,
    msg: *const c_char,
}

#[repr(C)]
struct PamResponse {
    resp: *mut c_char,
    resp_retcode: c_int,
}

#[repr(C)]
struct PamConv {
    conv: Option<unsafe extern "C" fn(
        num_msg: c_int,
        msgm: *const *const PamMessage,
        response: *mut *mut PamResponse,
        appdata_ptr: *mut c_void,
    ) -> c_int>,
    appdata_ptr: *mut c_void,
}

extern "C" {
    fn pam_start(
        service_name: *const c_char,
        user: *const c_char,
        pam_conversation: *const PamConv,
        pamh: *mut *mut PamHandle,
    ) -> c_int;

    fn pam_end(pamh: *mut PamHandle, pam_status: c_int) -> c_int;

    fn pam_authenticate(pamh: *mut PamHandle, flags: c_int) -> c_int;

    fn pam_open_session(pamh: *mut PamHandle, flags: c_int) -> c_int;

    fn pam_close_session(pamh: *mut PamHandle, flags: c_int) -> c_int;

    fn pam_strerror(pamh: *mut PamHandle, errnum: c_int) -> *const c_char;
}

/// Authenticate `username` against the `novau-greeter` PAM service.
pub fn authenticate(username: &str, password: &str) -> Result<()> {
    let svc = CString::new("novau-greeter").unwrap();
    let user = CString::new(username).map_err(|e| anyhow!("username: {e}"))?;

    // Box the password so the conv callback can read it.
    let pw_box = Box::new(password.to_string());
    let appdata = &*pw_box as *const String as *mut c_void;

    let conv = PamConv {
        conv: Some(conv_cb),
        appdata_ptr: appdata,
    };

    let mut pamh: *mut PamHandle = ptr::null_mut();
    let r = unsafe { pam_start(svc.as_ptr(), user.as_ptr(), &conv, &mut pamh) };
    if r != PAM_SUCCESS {
        return Err(anyhow!("pam_start failed: code {r}"));
    }

    let r = unsafe { pam_authenticate(pamh, 0) };
    let auth_ok = r == PAM_SUCCESS;

    // Always call pam_end
    unsafe { pam_end(pamh, r) };

    if !auth_ok {
        let msg = unsafe {
            let p = pam_strerror(pamh, r);
            if p.is_null() { format!("PAM error {r}") } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        };
        return Err(anyhow!("authentication failed: {msg}"));
    }

    Ok(())
}

/// Conversation callback — PAM uses this to ask for the password.
///
/// We answer any `PAM_PROMPT_ECHO_OFF` (password) prompt with the
/// pre-stashed password, ignore info/error messages.
unsafe extern "C" fn conv_cb(
    num_msg: c_int,
    msgm: *const *const PamMessage,
    response: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> c_int {
    if num_msg <= 0 || msgm.is_null() || response.is_null() {
        return 1;
    }

    let count = num_msg as usize;
    let layout = std::alloc::Layout::array::<PamResponse>(count).unwrap();
    let resp_ptr = std::alloc::alloc_zeroed(layout) as *mut PamResponse;
    if resp_ptr.is_null() {
        return 1;
    }

    let pw: &String = &*(appdata_ptr as *const String);

    for i in 0..count {
        let m_ptr = *msgm.add(i);
        if m_ptr.is_null() { continue; }
        let m = &*m_ptr;
        match m.msg_style {
            PAM_PROMPT_ECHO_OFF | PAM_PROMPT_ECHO_ON => {
                let cstr = match CString::new(pw.as_str()) {
                    Ok(c) => c,
                    Err(_) => return 1,
                };
                // Duplicate so the buffer outlives this scope
                let dup = libc::strdup(cstr.as_ptr());
                (*resp_ptr.add(i)).resp = dup;
                (*resp_ptr.add(i)).resp_retcode = 0;
            }
            PAM_ERROR_MSG | PAM_TEXT_INFO => {
                // No response needed
            }
            _ => {}
        }
    }

    *response = resp_ptr;
    PAM_SUCCESS
}

/// Open a PAM session for `username`. Returns a `PamSession` that
/// closes the session on drop.
pub struct PamSession {
    pamh: *mut PamHandle,
}

impl PamSession {
    pub fn open(username: &str, password: &str) -> Result<Self> {
        let svc = CString::new("novau-greeter").unwrap();
        let user = CString::new(username).map_err(|e| anyhow!("username: {e}"))?;
        let pw_box = Box::new(password.to_string());
        let appdata = &*pw_box as *const String as *mut c_void;
        let conv = PamConv { conv: Some(conv_cb), appdata_ptr: appdata };

        let mut pamh: *mut PamHandle = ptr::null_mut();
        let r = unsafe { pam_start(svc.as_ptr(), user.as_ptr(), &conv, &mut pamh) };
        if r != PAM_SUCCESS {
            return Err(anyhow!("pam_start: {r}"));
        }
        let r = unsafe { pam_authenticate(pamh, 0) };
        if r != PAM_SUCCESS {
            unsafe { pam_end(pamh, r) };
            return Err(anyhow!("authenticate: {r}"));
        }
        let r = unsafe { pam_open_session(pamh, 0) };
        if r != PAM_SUCCESS {
            unsafe { pam_end(pamh, r) };
            return Err(anyhow!("open_session: {r}"));
        }
        // Leak pw_box intentionally so the pointer stays valid for the
        // session lifetime (we don't call any more conv callbacks after
        // open_session, so this is fine).
        std::mem::forget(pw_box);
        Ok(Self { pamh })
    }
}

impl Drop for PamSession {
    fn drop(&mut self) {
        unsafe {
            pam_close_session(self.pamh, 0);
            pam_end(self.pamh, PAM_SUCCESS);
        }
    }
}
