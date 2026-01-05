//! <div align="center">
//!
//! ## `EdTUI`
//!
//! [![Crate Badge]](https://crates.io/crates/edtui) [![Continuous Integration](https://github.com/preiter93/edtui/actions/workflows/ci.yml/badge.svg)](https://github.com/preiter93/edtui/actions/workflows/ci.yml) [![Deps Status](https://deps.rs/repo/github/preiter93/edtui/status.svg)](https://deps.rs/repo/github/preiter93/edtui) [![License Badge]](./LICENSE)
//!
//! </div>
//!
//! ## Overview
//! `EdTUI` is a text editor widget for the [Ratatui](https://github.com/ratatui-org/ratatui) ecosystem.
//! It is designed to provide a user experience inspired by Vim. Edtui is developed to be used as an
//! editor in ratatui apps. It is not supposed to be a stand-alone code editor.
//!
//! Create a new `EditorState` and render it using `EditorView`.
//! You can customize the theme, enable line wrapping, syntax highlight the text or set the tab
//! width:
//! ```ignore
//! use edtui::{EditorState, EditorTheme, EditorView};
//! use ratatui::widgets::Widget;
//!
//! let mut state = EditorState::default();
//! EditorView::new(&mut state)
//!         .theme(EditorTheme::default())
//!         .wrap(true)
//!         .syntax_highlighter(None)
//!         .tab_width(2)
//!         .render(area, buf);
//! ```
//!
//! Handle events (Vim mode by default):
//! ```ignore
//! use edtui::EditorEventHandler;
//!
//! let mut event_handler = EditorEventHandler::default();
//! event_handler.on_key_event(key_event, &mut state);
//! ```
//!
//! Or use Emacs mode (modeless editing):
//! ```ignore
//! use edtui::{EditorState, EditorEventHandler, Lines};
//!
//! let mut state = EditorState::new(Lines::from("Hello World"));
//! let mut event_handler = EditorEventHandler::emacs_mode();
//! event_handler.on_key_event(key_event, &mut state);
//! ```
//!
//! Or customize keybindings:
//! ```ignore
//! use edtui::{KeyEventHandler, KeyEventRegister, EditorEventHandler};
//!
//! let mut key_handler = KeyEventHandler::vim_mode();
//! key_handler.insert(
//!     KeyEventRegister::n(vec![KeyEvent::Ctrl('x')]),
//!     SwitchMode(EditorMode::Insert),
//! );
//! let event_handler = EditorEventHandler::new(key_handler);
//! ```
//!
//! ## Demo
//!
//! ![](resources/app.gif)
//!
//! ## Features
//! - Custom theming.
//! - Mouse events.
//! - Vim and Emacs keybindings.
//! - Copy paste using the systems clipboard.
//! - Line wrapping.
//! - Syntax highlighting.
//! - Line numbers (absolute and relative).
//!
//! ## Theming
//!
//! Customize the editor `EditorTheme`:
//!
//! ```ignore
//! use edtui::{EditorTheme, EditorStatusLine};
//! use ratatui::style::{Style, Color};
//! use ratatui::widgets::Block;
//!
//! let theme = EditorTheme::default()
//!     .block(Block::default())
//!     .base(Style::default().bg(Color::Black).fg(Color::White))
//!     .cursor_style(Style::default().bg(Color::White).fg(Color::Black))
//!     .selection_style(Style::default().bg(Color::Yellow).fg(Color::Black))
//!     .hide_status_line(); // or use `.status_line(..)` for styling the status line
//! ```
//!
//! ## Line Numbers
//!
//! Display absolute or relative line numbers:
//!
//! ```ignore
//! use edtui::{EditorView, EditorState, EditorTheme, LineNumbers};
//! use ratatui::style::{Style, Color};
//!
//! EditorView::new(&mut EditorState::default())
//!         .theme(EditorTheme::default().line_numbers_style(Style::default().fg(Color::DarkGray)))
//!         .line_numbers(LineNumbers::Absolute)  // or LineNumbers::Relative
//!         .render(area, buf);
//! ```
//!
//! ![](resources/line_numbers.png)
//!
//! ## Mouse Events
//!
//! `Edtui` supports mouse input for moving the cursor and selecting text.
//! Mouse handling is **enabled by default** via a feature toggle.
//! Typically, mouse events are processed automatically when you call `on_event`:
//! ```ignore
//! let event_handler = EditorEventHandler::default();
//! event_handler.on_event(event, &mut state); // handles mouse events too
//! ```
//! If you want finer control you can handle mouse events explicitly using `on_mouse_event`:
//! ```ignore
//! event_handler.on_mouse_event(mouse_event, &mut state);
//! ```
//!
//! ## Syntax highlighting
//!
//! Syntax highlighting was added in version `0.8.4`.
//!
//! `Edtui` offers a number of custom themes, see [`SyntaxHighlighter::theme`] for a complete list.
//! If you want to use a custom theme, see [`SyntaxHighlighter::custom_theme`]. Check [syntect](https://github.com/trishume/syntect)
//! for more details about themes and extensions.
//!
//! ```ignore
//! use edtui::{EditorView, EditorState, SyntaxHighlighter};
//!
//! let syntax_highlighter = SyntaxHighlighter::new("dracula", "rs");
//! EditorView::new(&mut EditorState::default())
//!         .syntax_highlighter(Some(syntax_highlighter))
//!         .render(area, buf);
//! ```
//!
//! ![](resources/syntax_highlighting.gif)
//!
//! ## Paste Support
//!
//! If you want to enable paste (via ctrl+y or cmd+y) you must explicitly enable it at the start of your app:
//!
//! ```ignore
//! use ratatui::crossterm::event::EnableBracketedPaste;
//! let mut stdout = std::io::stdout();
//! ratatui::crossterm::xecute!(stdout, EnableBracketedPaste);
//! ```
//!
//! and disable it during cleanup:
//!
//! ```ignore
//! use ratatui::crossterm::event::DisableBracketedPaste;
//! ratatui::crossterm::execute!(std::io::stdout(), DisableBracketedPaste);
//! ```
//!
//! See `examples/app/term.rs` for a an example.
//!
//! ## Keybindings
//! `EdTUI` offers Vim keybindings by default and Emacs keybindings as an alternative.
//!
//! ### Vim Mode (default)
//!
//! #### Normal Mode:
//!
//! | Keybinding                | Description                                  |
//! |---------------------------|----------------------------------------------|
//! | `i`                       | Enter Insert mode                            |
//! | `v`                       | Enter Visual mode                            |
//! | `h`, `j`, `k`, `l`        | Navigate left, down, up, and right           |
//! | `w`                       | Move forward to the start of a word          |
//! | `e`                       | Move forward to the end of a word            |
//! | `b`                       | Move backward to the start of a word         |
//! | `ctrl+d`                  | Jump a half page down                        |
//! | `ctrl+u`                  | Jump a half page up                          |
//! | `x`                       | Delete the character under the cursor        |
//! | `u`, `ctrl+r`             | Undo/Redo last action                        |
//! | `Esc`                     | Escape Visual mode                           |
//! | `0`                       | Move cursor to start of line                 |
//! | `_`                       | Move cursor to first non-blank character     |
//! | `$`                       | Move cursor to end of line                   |
//! | `gg`                      | Move cursor to the first row                 |
//! | `G `                      | Move cursor to the last row                  |
//! | `%`                       | Move cursor to closing/opening bracket       |
//! | `a`                       | Append after the cursor                      |
//! | `A`                       | Append at the end of the line                |
//! | `o`                       | Add a new line below and enter Insert mode   |
//! | `O`                       | Add a new line above and enter Insert mode   |
//! | `J`                       | Join current line with the line below        |
//! | `d`                       | Delete the selection (Visual mode)           |
//! | `dd`                      | Delete the current line                      |
//! | `D`                       | Delete to the end of the line                |
//! | `viw`                     | Select between word.                         |
//! | `ciw`                     | Change between word.                         |
//! | `vi` + `", ', (, [ or {`  | Select between delimiter `", ', (, [ or {`   |
//! | `ci` + `", ', (, [ or {`  | Change between delimiter `", ', (, [ or {`   |
//! | `u`                       | Undo the last change                         |
//! | `r`                       | Redo the last undone action                  |
//! | `y`                       | Copy the selected text in visual mode        |
//! | `yy`                      | Copy the current line in normal mode         |
//! | `p`                       | Paste the copied text                        |
//! | `Home`                    | Move cursor to start of line                 |
//! | `End`                     | Move cursor to end of line                   |
//!
//! #### Insert Mode:
//!
//! | Keybinding  | Description                             |
//! |-------------|-----------------------------------------|
//! | `Esc`       | Return to Normal mode                   |
//! | `Backspace` | Delete the previous character           |
//! | `Enter`     | Insert line break                       |
//! | `Arrows`    | Navigation                              |
//! | `Home`      | Move cursor to start of line            |
//! | `End`       | Move cursor to end of line              |
//! | `ctrl+u`    | Delete until first character            |
//!
//! ### Emacs Mode
//!
//! Emacs Mode was added in version 0.10.1.
//!
//! Note that Emacs Mode is less feature complete and less tested than vim mode.
//!
//! | Keybinding      | Description                             |
//! |-----------------|-----------------------------------------|
//! | `Ctrl+f`        | Move forward                            |
//! | `Ctrl+b`        | Move backward                           |
//! | `Ctrl+n`        | Move to next line                       |
//! | `Ctrl+p`        | Move to previous line                   |
//! | `Ctrl+a`        | Move to start of line                   |
//! | `Ctrl+e`        | Move to end of line                     |
//! | `Ctrl+v`        | Page down                               |
//! | `Alt+v`         | Page up                                 |
//! | `Alt+f`         | Forward word                            |
//! | `Alt+b`         | Backward word                           |
//! | `Alt+<`         | Beginning of buffer                     |
//! | `Alt+>`         | End of buffer                           |
//! | `Ctrl+d`        | Delete character forward                |
//! | `Ctrl+h`        | Delete character backward               |
//! | `Alt+d`         | Delete word forward                     |
//! | `Alt+Backspace` | Delete word backward                    |
//! | `Ctrl+k`        | Delete to end of line                   |
//! | `Alt+u`         | Delete to start of line                 |
//! | `Ctrl+o`        | Open line (insert newline, stay)        |
//! | `Ctrl+j`        | Newline                                 |
//! | `Ctrl+y`        | Paste                                   |
//! | `Ctrl+u`        | Undo                                    |
//! | `Ctrl+r`        | Redo                                    |
//! | `Ctrl+g`        | Cancel search                           |
//! | `Enter`         | Insert line break                       |
//! | `Backspace`     | Delete previous character               |
//! | `Arrows`        | Navigation                              |
//! | `Home`          | Move to start of line                   |
//! | `End`           | Move to end of line                     |
//! | `Ctrl+s`        | Start search                            |
//! | `Ctrl+s`        | Search mode: Go to next match           |
//! | `Ctrl+r`        | Search mode: Go to previous match       |
//! | `Enter`         | Search mode: Select current match       |
//!
//! ### Roadmap
//! - [ ] Support termwiz and termion
//!
//! [Crate Badge]: https://img.shields.io/crates/v/edtui?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44
//! [License Badge]: https://img.shields.io/crates/l/edtui?style=flat-square&color=1370D3
#![allow(
    dead_code,
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation
)]
pub mod actions;
pub mod clipboard;
mod debug;
pub mod events;
mod helper;
mod state;
mod view;

#[allow(deprecated)]
pub use events::deprecated_input::EditorInput;
pub use events::EditorEventHandler;
pub use state::{mode::EditorMode, EditorState};
pub use view::{theme::EditorTheme, EditorStatusLine, EditorView, LineNumbers};

#[cfg(feature = "syntax-highlighting")]
pub use view::syntax_higlighting::SyntaxHighlighter;

#[cfg(feature = "syntax-highlighting")]
pub use syntect;

/// A data structure that contains chars organized in rows and columns
pub type Lines = jagged::Jagged<char>;
pub use jagged::index::RowIndex;
pub use jagged::Index2;
