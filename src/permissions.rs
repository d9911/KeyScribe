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
    pub(crate) reset_confirmation: bool,
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
            reset_confirmation: false,
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
        let result = platform::request();
        self.refresh();
        // A request is asynchronous; only a fresh OS check can unlock Continue.
        self.notice = Some(match result {
            Ok(()) => "Запрос отправлен macOS. Если системное окно не появилось, открой настройки и включи доступ вручную.".into(),
            Err(error) => error,
        });
    }

    pub(crate) fn reset_access(&mut self) {
        // Only the explicit confirmation in the recovery UI can reset this grant.
        if !self.reset_confirmation {
            return;
        }
        self.reset_confirmation = false;
        let result = platform::reset_access();
        self.refresh();
        self.notice = Some(match result {
            Ok(()) => "Старое разрешение Keyscribe сброшено. В настройках включи доступ для текущей копии Keyscribe.app. Если приложение не появилось, добавь его кнопкой «+» через «Показать в Finder». Затем перезапусти Keyscribe.".into(),
            Err(error) => error,
        });
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

// Keep reset scope explicit: never accept another app ID or the All service.
#[cfg(any(target_os = "macos", test))]
fn reset_arguments(bundle_id: &str) -> Result<[&str; 3], String> {
    if bundle_id != "local.keyscribe.desktop" {
        return Err(
            "Сброс отменён: идентификатор запущенного приложения не соответствует Keyscribe."
                .into(),
        );
    }
    Ok(["reset", "Accessibility", bundle_id])
}

#[cfg(target_os = "macos")]
mod platform {
    use super::Access;
    use core_foundation::{
        base::TCFType,
        boolean::CFBoolean,
        bundle::CFBundle,
        dictionary::{CFDictionary, CFDictionaryRef},
        string::{CFString, CFStringRef},
        url::CFURLRef,
    };

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
        static kAXTrustedCheckOptionPrompt: CFStringRef;
    }

    #[link(name = "CoreServices", kind = "framework")]
    unsafe extern "C" {
        fn LSRegisterURL(url: CFURLRef, update: u8) -> i32;
    }

    fn register_current_application() -> Result<String, String> {
        let path = super::application_path().ok_or("Не удалось определить путь приложения.")?;
        if path.extension().is_none_or(|extension| extension != "app") {
            return Err("Для восстановления доступа запусти dist/Keyscribe.app через make run, а не отдельный бинарник.".into());
        }
        let bundle = CFBundle::main_bundle();
        let info = bundle.info_dictionary();
        let bundle_id = info
            .find(CFString::new("CFBundleIdentifier"))
            .and_then(|value| value.downcast::<CFString>())
            .ok_or("Не удалось прочитать идентификатор текущего приложения.")?
            .to_string();
        super::reset_arguments(&bundle_id)?;
        let url = bundle
            .bundle_url()
            .ok_or("Не удалось определить URL текущего приложения.")?;
        if url.to_path().as_deref() != Some(path.as_path()) {
            return Err("Путь запущенного приложения не совпадает с его bundle. Перезапусти dist/Keyscribe.app.".into());
        }
        // A moved app can leave LaunchServices pointing TCC at an obsolete path.
        let status = unsafe { LSRegisterURL(url.as_concrete_TypeRef(), 1) };
        if status != 0 {
            return Err(format!(
                "Не удалось зарегистрировать текущую копию Keyscribe в macOS (код {status})."
            ));
        }
        Ok(bundle_id)
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

    pub(super) fn request() -> Result<(), String> {
        register_current_application()?;
        prompt();
        Ok(())
    }

    fn prompt() {
        if !check().accessibility {
            // The system owns this global CFString; retain it instead of taking ownership.
            let key = unsafe { CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt) };
            let options = CFDictionary::from_CFType_pairs(&[(key, CFBoolean::true_value())]);
            unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) };
        }
    }

    pub(super) fn reset_access() -> Result<(), String> {
        // Validate the running bundle and repair its path before revoking its grant.
        let bundle_id = register_current_application()?;
        let output = std::process::Command::new("/usr/bin/tccutil")
            .args(super::reset_arguments(&bundle_id)?)
            .output()
            .map_err(|error| format!("Не удалось сбросить разрешение Keyscribe: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "macOS не сбросила разрешение Keyscribe ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        prompt();
        open_settings().map_err(|error| format!("Разрешение Keyscribe сброшено. {error}"))
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
            backend_ready: {
                #[cfg(target_os = "linux")]
                {
                    crate::linux_input::ensure_session().is_ok()
                }
                #[cfg(not(target_os = "linux"))]
                {
                    true
                }
            },
        }
    }

    pub(super) fn request() -> Result<(), String> {
        Ok(())
    }

    pub(super) fn reset_access() -> Result<(), String> {
        Err("Восстановление разрешения доступно только на macOS.".into())
    }

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
    fn reset_is_limited_to_keyscribe_accessibility_and_needs_confirmation() {
        assert_eq!(
            reset_arguments("local.keyscribe.desktop").unwrap(),
            ["reset", "Accessibility", "local.keyscribe.desktop"]
        );
        for id in [
            "",
            "All",
            "com.apple.Terminal",
            "local.keyscribe.access-check",
        ] {
            assert!(reset_arguments(id).is_err());
        }
        let mut gate = Permissions::from_access(DENIED);
        gate.reset_access();
        assert!(gate.notice.is_none());
        assert_eq!(gate.access, DENIED);
    }

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
