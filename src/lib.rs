//! ## `EdTUI`
//!
//! ### Overview
//! `EdTUI` is a text editor widget for the [Ratatui](https://github.com/ratatui-org/ratatui) ecosystem.
//! It is designed to provide a light-weight user experience inspired by Vim.
//!
//! ```ignore
//! use edtui::{EditorState, EditorTheme, EditorView};
//! use ratatui::widgets::Widget;
//!
//! let mut state = EditorState::default();
//! EditorView::new(&mut state)
//!         .theme(EditorTheme::default())
//!         .wrap(true) // line wrapping
//!         .render(area, buf)
//! ```
//!
//! ## Features
//! - Vim-like keybindings and editing modes for efficient text manipulation.
//! - Normal, Insert and Visual mode.
//! - Clipboard: Uses the `arboard` clibpboard by default which allows copy pasting between the
//!   system clipboard and the editor.
//! - Line wrapping
//!
//! ## Keybindings
//! `EdTUI` offers a set of keybindings similar to Vim. Here are some of the most common keybindings:
//!
//! #### Normal/Visual Mode:
//!
//! | Keybinding              | Description                                  |
//! |-------------------------|----------------------------------------------|
//! | `i`                     | Enter Insert mode                            |
//! | `v`                     | Enter Visual mode                            |
//! | `h`, `j`, `k`, `l`      | Navigate left, down, up, and right           |
//! | `w`, `b`                | Move forward or backward by word             |
//! | `x`                     | Delete the character under the cursor        |
//! | `Del`                   | Delete the character left of the cursor      |
//! | `u`, `<ctrl>+r`         | Undo/Redo last action                        |
//! | `Esc`                   | Escape Insert mode or Visual mode            |
//! | `0`                     | Move cursor to start of line                 |
//! | `^`                     | Move cursor to first non-blank character     |
//! | `$`                     | Move cursor to end of line                   |
//! | `gg`                    | Move cursor to the first row                 |
//! | `G `                    | Move cursor to the last row                  |
//! | `%`                     | Move cursor to closing/opening bracket       |
//! | `a`                     | Append after the cursor                      |
//! | `A`                     | Append at the end of the line                |
//! | `o`                     | Add a new line below and enter Insert mode   |
//! | `O`                     | Add a new line above and enter Insert mode   |
//! | `Backspace`             | Delete the previous character                |
//! | `d`                     | Delete the selection                         |
//! | `dd`                    | Delete the current line                      |
//! | `ciw`                   | Select between delimiters. Supported: [`"`]  |
//! | `u`                     | Undo the last change                         |
//! | `r`                     | Redo the last undone action                  |
//! | `y`                     | Copy the selected text                       |
//! | `p`                     | Paste the copied text                        |
//!
//! #### Insert Mode:
//!
//! | Keybinding | Description                             |
//! |------------|-----------------------------------------|
//! | `Esc`      | Return to Normal mode                   |
//!
//! For more keybindings and customization options, refer to the code.
//!
//! ## Demo
//!
//!![](resources/app.gif)
//!
//! ## Experimental Mouse Support
//!
//! `Edtui` now includes experimental mouse support. To enable it activate the feature
//! ```toml
//! [dependencies.edtui]
//! version = "0.7"
//! features = ["mouse-support"]
//! ```
//! and use the mouse event handler
//! ```ignore
//! let event_handler = EditorEvent::default();
//! event_handler.on_mouse_event(mouse_event, &mut state);
//! ```
//!
//! **Note**: This feature is experimental, so expect potential bugs and breaking changes.
//!
//! ### Roadmap
//!
//! - [x] Clipboard
//! - [x] Search
//! - [x] Soft-wrap lines
//!
//! - [ ] Vims `f`/`t` go to first
//! - [ ] Support termwiz and termion
//! - [ ] Display line numbers
//! - [ ] Remap keybindings
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
pub use view::{theme::EditorTheme, EditorStatusLine, EditorView};

/// A data structure that contains chars organized in rows and columns
pub type Lines = jagged::Jagged<char>;
pub use jagged::index::RowIndex;
pub use jagged::Index2;
