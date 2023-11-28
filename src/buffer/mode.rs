/// The editor mode.
#[derive(Default, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual,
}

impl Mode {
    /// Returns the name of the [`Mode`] as a string.
    pub fn name(&self) -> String {
        match self {
            Self::Normal => "Normal".to_string(),
            Self::Insert => "Insert".to_string(),
            Self::Visual => "Visual".to_string(),
        }
    }
}
