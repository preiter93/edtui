#![no_main]

use edtui::{input::key::Key, EditorState, Input};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<Key>| {
    let mut state = EditorState::default();
    let mut input = Input::default();
    for key in data {
        input.on_key(key, &mut state)
    }
});
