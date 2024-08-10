#![no_main]

use edtui::{events::KeyEvent, EditorInput, EditorState};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Vec<KeyEvent>| {
    let mut state = EditorState::default();
    let mut input = EditorInput::default();
    for key in data {
        input.on_event(key, &mut state)
    }
});
