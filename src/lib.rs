//! ## `EdTUI`
//!
//! ### Overview
//! `EdTUI` is a text editor widget for the [Ratatui](https://github.com/ratatui-org/ratatui) ecosystem.
//! It is designed to provide a light-weight user experience inspired by Vim.
//!
//! ## Features
//! - Vim-like keybindings and editing modes for efficient text manipulation.
//! - Normal, Insert and Visual mode.
//! - Clipboard: Uses the `arboard` clibpboard by default which allows copy pasting between the
//!   system clipboard and the editor.
//!
//! ## Keybindings
//! `EdTUI` offers a set of keybindings similar to Vim. Here are some of the most common keybindings:
//!
//! #### Normal Mode:
//!
//! | Keybinding         | Description                             |
//! |--------------------|-----------------------------------------|
//! | `i`                | Enter Insert mode                       |
//! | `v`                | Enter Visual mode                       |
//! | `h`, `j`, `k`, `l` | Navigate left, down, up, and right      |
//! | `w`, `b`           | Move forward or backward by word        |
//! | `x`                | Delete the character under the cursor   |
//! | `Del`              | Delete the character left of the cursor |
//! | `u`, `r`           | Undo/Redo last action                   |
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
//!![](resources/screenshot.png)
//!
//! ### Roadmap
//!
//! - [x] Clipboard
//! - [ ] Search
//! - [ ] Vims `f`/`t` go to first
//! - [ ] Support termwiz and termion
//! - [ ] Display line numbers
//! - [ ] Remap keybindings
//! - [ ] Soft-wrap lines
#![allow(
    dead_code,
    clippy::module_name_repetitions,
    clippy::cast_possible_truncation
)]
pub mod actions;
pub mod clipboard;
mod debug;
mod helper;
pub mod input;
pub mod state;
pub mod view;

pub use input::Input;
pub use state::{mode::EditorMode, EditorState};
pub use view::{theme::EditorTheme, EditorView, StatusLine};

/// A data structure that contains chars organized in rows and columns
pub type Lines = jagged::Jagged<char>;
pub use jagged::Index2;
