use crate::{
    actions::{cpaste::PasteOverSelection, Execute, Paste},
    clipboard::ClipboardTrait,
    EditorState,
};

/// Handles a paste event.
#[derive(Clone, Debug, Default)]
pub(crate) struct PasteEventHandler {}
impl PasteEventHandler {
    pub(crate) fn on_event(text: String, state: &mut EditorState) {
        state.clip.set_text(text);
        match state.mode {
            crate::EditorMode::Normal | crate::EditorMode::Insert => Paste.execute(state),
            crate::EditorMode::Visual => PasteOverSelection.execute(state),
            crate::EditorMode::Search => {} // TODO: Insert into search
        }
    }
}
