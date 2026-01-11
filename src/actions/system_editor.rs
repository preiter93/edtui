//! Support for opening the editor content in an external system editor.

use crate::actions::Execute;
use crate::{EditorState, Index2, Lines};
#[cfg(feature = "mouse-support")]
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
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

/// Checks if a system editor request is pending and runs the system editor if so.
///
/// Temporarily exits the TUI, opens the system's default text editor with the current
/// content, waits for the editor to close, and updates the editor state.
///
/// ## Terminal state handling
///
/// If the `mouse-support` feature is enabled mouse capture is explicitly
/// re-enabled when returning to the TUI.
///
/// Other terminal modes (such as bracketed paste, focus reporting, or any
/// application-specific terminal flags) are not restored.
/// This is due to limitations in crossterm, which does not expose a way
/// (to my knowledge) to get the current terminal mode state.
///
/// Callers that rely on additional terminal modes must re-enable them after
/// this function returns.
pub fn open<B: Backend>(state: &mut EditorState, terminal: &mut Terminal<B>) -> Result<()> {
    if !std::mem::take(&mut state.system_edit_requested) {
        return Ok(());
    }

    state.capture();

    let content = state.lines.to_string();

    #[cfg(feature = "mouse-support")]
    {
        let _ = crossterm::execute!(stdout(), DisableMouseCapture);
    }
    disable_raw_mode()?;
    crossterm::execute!(stdout(), LeaveAlternateScreen)?;

    let result = edit::edit(&content);

    crossterm::execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    #[cfg(feature = "mouse-support")]
    {
        let _ = crossterm::execute!(stdout(), EnableMouseCapture);
    }
    let _ = terminal.clear();

    let edited = result.map_err(std::io::Error::other)?;

    state.lines = Lines::from(edited.trim_end_matches('\n'));
    state.cursor = Index2::new(0, 0);
    state.selection = None;

    Ok(())
}

/// Returns whether a system editor request is currently pending.
#[must_use]
pub fn is_pending(state: &EditorState) -> bool {
    state.system_edit_requested
}
