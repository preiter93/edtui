/// The editor mode.
#[derive(Default, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum EditorMode {
    #[default]
    Normal,
    Insert,
    Visual,
    Search,
}

impl EditorMode {
    /// Returns the name of the [`EditorMode`] as a string.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Normal => "Normal".to_string(),
            Self::Insert => "Insert".to_string(),
            Self::Visual => "Visual".to_string(),
            Self::Search => "Search".to_string(),
        }
    }
}
