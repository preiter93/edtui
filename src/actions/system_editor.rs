//! Support for opening the editor content in an external system editor.

use crate::actions::Execute;
use crate::{EditorState, Index2, Lines};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui_core::backend::Backend;
use ratatui_core::terminal::Terminal;
use std::io::{stdout, Result};

/// Action that requests opening the editor content in an external system editor.
/// Bound to `Ctrl+e` in normal mode.
#[derive(Clone, Debug)]
pub struct OpenSystemEditor;

impl Execute for OpenSystemEditor {
    fn execute(&mut self, state: &mut EditorState) {
        state.system_edit_requested = true;
    }
}

/// Opens the system editor if a request is pending.
///
/// Temporarily exits the TUI, opens the system's default text editor with the current
/// content, waits for the editor to close, and updates the editor state.
pub fn open<B: Backend>(state: &mut EditorState, terminal: &mut Terminal<B>) -> Result<()> {
    if !std::mem::take(&mut state.system_edit_requested) {
        return Ok(());
    }

    let content = state.lines.to_string();

    disable_raw_mode()?;
    crossterm::execute!(stdout(), LeaveAlternateScreen)?;

    let result = edit::edit(&content);

    crossterm::execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let _ = terminal.clear();

    let edited = result.map_err(std::io::Error::other)?;

    state.lines = Lines::from(edited.as_str());
    state.cursor = Index2::new(0, 0);
    state.selection = None;

    Ok(())
}

/// Returns whether a system editor request is currently pending.
#[must_use]
pub fn is_pending(state: &EditorState) -> bool {
    state.system_edit_requested
}
