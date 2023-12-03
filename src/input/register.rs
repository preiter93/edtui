use std::collections::HashMap;

use crate::{actions::Action, EditorMode, EditorState};

use super::key::Key;

#[derive(Clone, Debug, Default)]
pub struct Register {
    lookup: Vec<Key>,
    // register: HashMap<RegisterKey, RegisterVal>,
    register: HashMap<RegisterKey, Action>,
}

impl Register {
    /// Constructs a new Register
    #[must_use]
    pub fn new() -> Self {
        Self {
            lookup: Vec::new(),
            register: HashMap::new(),
            // actions: HashMap::new(),
        }
    }

    /// Insert a new callback to the registry
    pub fn insert<T: Into<Action>>(&mut self, k: RegisterKey, v: T) {
        self.register.insert(k, v.into());
    }

    // /// Insert a new action to the registry
    // pub fn insert_action(&mut self, k: RegisterKey, a: Action) {
    //     self.actions.insert(k, a);
    // }

    /// Returns a callback for a specific register key, if present.
    /// Returns a callback only if there is an exact match. If there
    /// are multiple matches or an inexact match, the specified key
    /// is appended to the lookup vector.
    /// If there is an exact match or if none of the keys in the registry
    /// starts with the current sequence, the lookup sequence is reset.
    #[must_use]
    pub fn get(&mut self, c: Key, mode: EditorMode) -> Option<Action> {
        let key = self.create_register_key(c, mode);

        match self
            .register
            .keys()
            .filter(|k| k.mode == key.mode && k.keys.starts_with(&key.keys))
            .count()
        {
            0 => {
                self.lookup.clear();
                None
            }
            1 => self.register.get(&key).map(|action| {
                self.lookup.clear();
                action.clone()
            }),
            _ => None,
        }
    }

    fn create_register_key(&mut self, c: Key, mode: EditorMode) -> RegisterKey {
        self.lookup.push(c);
        RegisterKey::new(self.lookup.clone(), mode)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct RegisterKey {
    pub keys: Vec<Key>,
    pub mode: EditorMode,
}

pub type RegisterCB = fn(&mut EditorState);

#[derive(Clone, Debug)]
pub struct RegisterVal(pub fn(&mut EditorState));

impl RegisterKey {
    pub fn new<T>(key: T, mode: EditorMode) -> Self
    where
        T: Into<Vec<Key>>,
    {
        Self {
            keys: key.into(),
            mode,
        }
    }

    pub fn n<T>(key: T) -> Self
    where
        T: Into<Vec<Key>>,
    {
        Self::new(key.into(), EditorMode::Normal)
    }

    pub fn v<T>(key: T) -> Self
    where
        T: Into<Vec<Key>>,
    {
        Self::new(key.into(), EditorMode::Visual)
    }

    pub fn i<T>(key: T) -> Self
    where
        T: Into<Vec<Key>>,
    {
        Self::new(key.into(), EditorMode::Insert)
    }
}

pub enum Find {
    NoMatch,
    MultipleMatches,
    PartialMatch,
    ExactMatch(RegisterCB),
}
