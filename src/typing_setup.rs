use keyscribe::{keyboard::Layout, shortcut::Shortcut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Environment {
    MacOs,
    Windows,
}

impl Environment {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::MacOs => "macOS / Citrix с Mac",
            Self::Windows => "Windows / Citrix с Windows",
        }
    }
}

#[derive(Default)]
pub(crate) struct TypingSetup {
    pub(crate) environment: Option<Environment>,
    pub(crate) initial_layout: Option<Layout>,
    pub(crate) shortcut: Shortcut,
}

impl TypingSetup {
    pub(crate) fn select_environment(&mut self, environment: Environment) {
        if self.environment != Some(environment) {
            self.environment = Some(environment);
            self.initial_layout = None;
            self.shortcut = match environment {
                Environment::MacOs => Shortcut::CTRL_OPTION_SPACE,
                Environment::Windows => Shortcut::WIN_SPACE,
            };
        }
    }

    pub(crate) fn validate_context(&self) -> Result<Layout, &'static str> {
        self.environment
            .ok_or("Выбери окружение, в котором работает Keyscribe.")?;
        self.initial_layout
            .ok_or("Укажи текущий язык в целевом окне перед запуском.")
    }

    pub(crate) fn validate_text(&self, auto_switch: bool) -> Result<Layout, &'static str> {
        let layout = self.validate_context()?;
        if auto_switch && !self.shortcut.is_valid() {
            return Err("Выбери сочетание из минимум двух клавиш с модификатором. Ctrl+Alt+F11/F12 зарезервированы приложением.");
        }
        Ok(layout)
    }

    pub(crate) fn require_current_language(&mut self) {
        // The remote layout may have changed even if a run failed or was cancelled.
        self.initial_layout = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_environment_and_current_language_are_required_even_without_switching() {
        let mut setup = TypingSetup::default();
        assert!(setup.validate_text(false).is_err());
        setup.initial_layout = Some(Layout::Us);
        assert!(setup.validate_context().is_err());
        setup.select_environment(Environment::MacOs);
        assert!(setup.validate_text(false).is_err());
        setup.initial_layout = Some(Layout::Russian);
        assert_eq!(setup.validate_text(false), Ok(Layout::Russian));
    }

    #[test]
    fn changing_environment_selects_its_preset_and_requires_a_new_language() {
        let mut setup = TypingSetup::default();
        setup.select_environment(Environment::MacOs);
        assert_eq!(setup.shortcut, Shortcut::CTRL_OPTION_SPACE);
        setup.initial_layout = Some(Layout::Us);
        setup.shortcut = Shortcut::CTRL_WIN_SPACE;
        setup.select_environment(Environment::MacOs);
        assert_eq!(setup.validate_text(true), Ok(Layout::Us));
        assert_eq!(setup.shortcut, Shortcut::CTRL_WIN_SPACE);
        setup.select_environment(Environment::Windows);
        assert_eq!(setup.shortcut, Shortcut::WIN_SPACE);
        assert!(setup.validate_context().is_err());
    }

    #[test]
    fn the_next_run_requires_a_new_language_but_keeps_the_environment_and_shortcut() {
        let mut setup = TypingSetup::default();
        setup.select_environment(Environment::Windows);
        setup.initial_layout = Some(Layout::Russian);
        setup.shortcut = Shortcut::CTRL_WIN_SPACE;
        assert_eq!(setup.validate_text(true), Ok(Layout::Russian));
        setup.require_current_language();
        assert!(setup.validate_text(true).is_err());
        assert_eq!(setup.environment, Some(Environment::Windows));
        assert_eq!(setup.shortcut, Shortcut::CTRL_WIN_SPACE);
        setup.initial_layout = Some(Layout::Us);
        assert_eq!(setup.validate_text(true), Ok(Layout::Us));
    }

    #[test]
    fn an_invalid_switch_blocks_text_but_does_not_affect_cycles_or_single_layout() {
        let mut setup = TypingSetup::default();
        setup.select_environment(Environment::Windows);
        setup.initial_layout = Some(Layout::Us);
        setup.shortcut = Shortcut {
            trigger: Some(keyscribe::shortcut::Trigger::Function(12)),
            ..Shortcut::CTRL_OPTION_SPACE
        };
        assert!(setup.validate_text(true).is_err());
        assert_eq!(setup.validate_text(false), Ok(Layout::Us));
        assert_eq!(setup.validate_context(), Ok(Layout::Us));
    }
}
