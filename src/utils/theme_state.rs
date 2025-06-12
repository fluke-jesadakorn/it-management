#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ThemeState {
    pub is_dark: bool,
}

impl ThemeState {
    pub fn unwrap_or_default(self) -> Self {
        self
    }
}
