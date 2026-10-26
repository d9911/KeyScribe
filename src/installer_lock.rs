//! Hold an installation mutex while a Windows GUI/worker is alive.

pub(crate) struct InstallationLock {
    #[cfg(target_os = "windows")]
    handle: *mut std::ffi::c_void,
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateMutexW(
        attributes: *const std::ffi::c_void,
        owner: i32,
        name: *const u16,
    ) -> *mut std::ffi::c_void;
    fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
}

pub(crate) fn acquire(_identifier: &str) -> std::io::Result<InstallationLock> {
    #[cfg(target_os = "windows")]
    {
        let name: Vec<u16> = format!("Local\\{_identifier}")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        // SAFETY: name is NUL-terminated and lives through the call; default security
        // attributes are requested and the returned handle is owned by this guard.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        Ok(InstallationLock { handle })
    }
    #[cfg(not(target_os = "windows"))]
    Ok(InstallationLock {})
}

impl Drop for InstallationLock {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        // SAFETY: this guard owns the successful CreateMutexW handle and closes it once.
        unsafe {
            CloseHandle(self.handle);
        }
    }
}
