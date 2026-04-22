/// The current screen/state of the application.
pub enum AppScreen {
    Detecting,
    NoPermission(String),
    NoKeyboards,
    SelectKeyboard,
    Loading,
    Connected,
}

/// Which main tab is active in the connected view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectedTab {
    Keymap,
    Lighting,
    TapDance,
    Combos,
    KeyOverrides,
}

/// Status message shown temporarily after an action.
pub struct StatusMessage {
    pub text:      String,
    pub is_error:  bool,
    pub expire_at: std::time::Instant,
}

impl StatusMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text:      text.into(),
            is_error:  false,
            expire_at: std::time::Instant::now() + std::time::Duration::from_secs(3),
        }
    }
    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text:      text.into(),
            is_error:  true,
            expire_at: std::time::Instant::now() + std::time::Duration::from_secs(5),
        }
    }
    pub fn is_expired(&self) -> bool {
        std::time::Instant::now() >= self.expire_at
    }
}

/// A modal confirmation dialog.
pub struct ConfirmDialog {
    pub title:   String,
    pub message: String,
    pub action:  ConfirmAction,
}

pub enum ConfirmAction {
    Import,
}
