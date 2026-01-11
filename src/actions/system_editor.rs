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
///
/// In Vim mode, this is bound to `Ctrl+e` in normal mode.
/// In Emacs mode, this is bound to `Alt+e`.
///
/// This action only sets a flag; the actual editor opening happens when
/// [`open`] is called.
#[derive(Clone, Debug)]
pub struct OpenSystemEditor;

impl Execute for OpenSystemEditor {
    fn execute(&mut self, state: &mut EditorState) {
        state.system_edit_requested = true;
    }
}

/// Opens the editor content in an external system editor if a request is pending.
///
/// This function checks if [`OpenSystemEditor`] was executed (via [`is_pending`]).
///
/// ## Terminal Mode Restoration
///
/// This function only restores raw mode and the alternate screen. Any other
/// terminal modes (mouse capture, bracketed paste, focus events, etc.) must
/// be re-enabled by the caller after this function returns.
///
/// ## Errors
///
/// Returns an error if:
/// - Terminal mode changes fail
/// - The external editor fails to open or returns an error
pub fn open<B: Backend>(state: &mut EditorState, terminal: &mut Terminal<B>) -> Result<()> {
    if !std::mem::take(&mut state.system_edit_requested) {
        return Ok(());
    }

    state.capture();

    let content = state.lines.to_string();

    disable_raw_mode()?;
    crossterm::execute!(stdout(), LeaveAlternateScreen)?;

    let result = edit::edit(&content);

    crossterm::execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let _ = terminal.clear();

    let edited = result.map_err(std::io::Error::other)?;

    state.lines = Lines::from(edited.trim_end_matches('\n'));
    state.cursor = Index2::new(0, 0);
    state.selection = None;

    Ok(())
}

/// Returns whether a system editor request is currently pending.
///
/// Use this after handling events to check if the user requested to open
/// the system editor.
///
/// ```ignore
/// event_handler.on_event(event, &mut state);
///
/// if system_editor::is_pending(&state) {
///     system_editor::open(&mut state, &mut terminal)?;
/// }
/// ```
#[must_use]
pub fn is_pending(state: &EditorState) -> bool {
    state.system_edit_requested
}
