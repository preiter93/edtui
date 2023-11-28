/// The editor mode.
#[derive(Default, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum EditorMode {
    #[default]
    Normal,
    Insert,
    Visual,
}

impl EditorMode {
    /// Returns the name of the [`Mode`] as a string.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Normal => "Normal".to_string(),
            Self::Insert => "Insert".to_string(),
            Self::Visual => "Visual".to_string(),
        }
    }
}
