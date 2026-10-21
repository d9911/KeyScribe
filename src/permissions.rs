use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Access {
    pub(crate) accessibility: bool,
    pub(crate) backend_ready: bool,
}

impl Access {
    pub(crate) fn allowed(self) -> bool {
        self.accessibility && self.backend_ready
    }
}

pub(crate) struct Permissions {
    pub(crate) access: Access,
    pub(crate) dialog_open: bool,
    pub(crate) notice: Option<String>,
    last_checked: Instant,
}

impl Default for Permissions {
    fn default() -> Self {
        Self::from_access(current_access())
    }
}

impl Permissions {
    fn from_access(access: Access) -> Self {
        Self {
            access,
            dialog_open: !access.allowed(),
            notice: None,
            last_checked: Instant::now(),
        }
    }

    pub(crate) fn can_start(&self) -> bool {
        self.access.allowed() && !self.dialog_open
    }

    fn observe(&mut self, access: Access) -> bool {
        let changed = self.access != access;
        self.access = access;
        if !access.allowed() {
            self.dialog_open = true;
        }
        changed
    }

    pub(crate) fn refresh(&mut self) -> bool {
        self.last_checked = Instant::now();
        self.observe(current_access())
    }

    pub(crate) fn poll(&mut self, force: bool) -> bool {
        if force || self.last_checked.elapsed() >= Duration::from_secs(1) {
            self.refresh()
        } else {
            false
        }
    }

    pub(crate) fn request(&mut self) {
        platform::request();
        self.refresh();
        // A request is asynchronous; only a fresh OS check can unlock Continue.
        self.notice = Some("Запрос отправлен macOS. Если системное окно не появилось, открой настройки и включи доступ вручную.".into());
    }

    pub(crate) fn confirm(&mut self) -> bool {
        self.confirm_access(current_access())
    }

    fn confirm_access(&mut self, access: Access) -> bool {
        self.last_checked = Instant::now();
        self.observe(access);
        if self.access.allowed() {
            self.dialog_open = false;
            self.notice = None;
            true
        } else {
            false
        }
    }
}

pub(crate) fn current_access() -> Access {
    platform::check()
}

pub(crate) fn application_path() -> Option<std::path::PathBuf> {
    let executable = std::env::current_exe().ok()?;
    Some(
        executable
            .ancestors()
            .find(|path| path.extension().is_some_and(|extension| extension == "app"))
            .map_or_else(|| executable.clone(), std::path::Path::to_path_buf),
    )
}

pub(crate) fn open_settings() -> Result<(), String> {
    platform::open_settings()
}

pub(crate) fn reveal_application() -> Result<(), String> {
    platform::reveal_application()
}

#[cfg(target_os = "macos")]
mod platform {
    use super::Access;
    use core_foundation::{
        base::TCFType,
        boolean::CFBoolean,
        dictionary::{CFDictionary, CFDictionaryRef},
        string::{CFString, CFStringRef},
    };

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
        static kAXTrustedCheckOptionPrompt: CFStringRef;
    }

    pub(super) fn check() -> Access {
        // Match Enigo's own Accessibility check. Creating an empty input connection
        // does not post events; dropping it has no held keys or buttons to release.
        let accessibility = unsafe { AXIsProcessTrusted() };
        let backend_ready = accessibility
            && enigo::Enigo::new(&enigo::Settings {
                open_prompt_to_get_permissions: false,
                ..enigo::Settings::default()
            })
            .is_ok();
        Access {
            accessibility,
            backend_ready,
        }
    }

    pub(super) fn request() {
        if !check().accessibility {
            // The system owns this global CFString; retain it instead of taking ownership.
            let key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
            let options = CFDictionary::from_CFType_pairs(&[(key, CFBoolean::true_value())]);
            unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) };
        }
    }

    fn open(arguments: &[&std::ffi::OsStr]) -> Result<(), String> {
        let status = std::process::Command::new("/usr/bin/open")
            .args(arguments)
            .status()
            .map_err(|error| format!("Не удалось открыть: {error}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("macOS не смогла открыть окно. Открой Системные настройки → Конфиденциальность и безопасность → Универсальный доступ вручную.".into())
        }
    }

    pub(super) fn open_settings() -> Result<(), String> {
        open(&[std::ffi::OsStr::new(
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        )])
    }

    pub(super) fn reveal_application() -> Result<(), String> {
        let path = super::application_path().ok_or("Не удалось определить путь приложения.")?;
        open(&[std::ffi::OsStr::new("-R"), path.as_os_str()])
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::Access;

    pub(super) fn check() -> Access {
        Access {
            accessibility: true,
            backend_ready: true,
        }
    }

    pub(super) fn request() {}

    pub(super) fn open_settings() -> Result<(), String> {
        Err("Это окно разрешений доступно только на macOS.".into())
    }

    pub(super) fn reveal_application() -> Result<(), String> {
        Err("Показ приложения в Finder доступен только на macOS.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRANTED: Access = Access {
        accessibility: true,
        backend_ready: true,
    };
    const DENIED: Access = Access {
        accessibility: false,
        backend_ready: false,
    };

    #[test]
    fn accessibility_and_backend_connection_are_needed_and_a_dialog_blocks_start() {
        for access in [
            DENIED,
            Access {
                accessibility: true,
                backend_ready: false,
            },
            Access {
                accessibility: false,
                backend_ready: true,
            },
        ] {
            let gate = Permissions::from_access(access);
            assert!(!gate.can_start());
            assert!(gate.dialog_open);
        }
        let mut gate = Permissions::from_access(GRANTED);
        assert!(gate.can_start());
        gate.dialog_open = true;
        assert!(!gate.can_start());
    }

    #[test]
    fn grant_does_not_dismiss_the_dialog_and_revocation_blocks_another_run() {
        let mut gate = Permissions::from_access(DENIED);
        assert!(!gate.confirm_access(DENIED));
        assert!(gate.observe(GRANTED));
        assert!(gate.dialog_open);
        assert!(!gate.can_start());
        assert!(gate.confirm_access(GRANTED));
        assert!(gate.can_start());
        assert!(gate.observe(DENIED));
        assert!(!gate.can_start());
        assert!(gate.dialog_open);
    }

    #[test]
    fn continue_rechecks_access_instead_of_trusting_a_previous_grant() {
        let mut gate = Permissions::from_access(GRANTED);
        gate.dialog_open = true;
        assert!(!gate.confirm_access(DENIED));
        assert!(!gate.can_start());
        assert!(gate.dialog_open);
    }
}
