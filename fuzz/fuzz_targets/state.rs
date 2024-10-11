#![no_main]

use edtui::{events::KeyEvent, EditorEventHandler, EditorState};
use libfuzzer_sys::fuzz_target;

// run: cargo fuzz run state -- -rss_limit_mb=4096
fuzz_target!(|data: Vec<KeyEvent>| {
    let mut state = EditorState::default();
    let mut input = EditorEventHandler::default();
    for key in data {
        input.on_key_event(key, &mut state)
    }
});
