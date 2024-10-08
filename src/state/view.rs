use crate::{view::line_wrapper::LineWrapper, Index2, Lines};
use ratatui::layout::Rect;
use unicode_width::UnicodeWidthChar;

/// Represents the (x, y) offset of the editor's viewport.
/// It represents the top-left local editor coordinate.
#[derive(Default, Debug, Clone)]
pub(crate) struct ViewState {
    /// The offset of the viewport.
    pub(crate) viewport: Offset,
    /// The number of rows that are displayed on the viewport
    pub(crate) num_rows: usize,
    /// Sets the offset from the upper-left corner of the terminal window to the start of the textarea buffer.
    ///
    /// This offset is necessary to calculate the mouse position in relation to the text
    /// within the editor.
    pub(crate) editor_to_textarea_offset: Offset,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct Offset {
    /// The x-offset.
    pub(crate) x: usize,
    /// The y-offset.
    pub(crate) y: usize,
}

impl Offset {
    pub(crate) fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<Rect> for Offset {
    fn from(value: Rect) -> Self {
        Self {
            x: value.x as usize,
            y: value.y as usize,
        }
    }
}

impl ViewState {
    /// Sets the editors position on the screen.
    ///
    /// Equivalent to the upper left coordinate of the editor in the
    /// buffers coordinate system.
    pub(crate) fn set_editor_to_textarea_offset<T: Into<Offset>>(&mut self, offset: T) {
        self.editor_to_textarea_offset = offset.into();
    }

    /// Updates the view's offset and returns the new offset.
    /// This method is used internally to modify the view's offset coordinates.
    /// The given cursor coordinates are assumed to be in the editors absolute
    /// coordinates.
    pub(crate) fn update_viewport_offset(
        &mut self,
        size: (usize, usize),
        cursor: Index2,
        lines: &Lines,
        wrap: bool,
    ) -> Offset {
        let max_cursor_pos = (
            size.0.saturating_sub(1) + self.viewport.x,
            size.1.saturating_sub(1) + self.viewport.y,
        );

        if wrap {
            self.viewport.x = 0;
        } else {
            // scroll left
            if cursor.col < self.viewport.x {
                self.viewport.x = cursor.col;
            }
            // scroll right
            if cursor.col > max_cursor_pos.0 {
                self.viewport.x += cursor.col.saturating_sub(max_cursor_pos.0);
            }
        }

        // scroll up
        if cursor.row < self.viewport.y {
            self.viewport.y = cursor.row;
        }

        // scroll down
        if wrap {
            self.scroll_down(lines, size.0, size.1, cursor.row);
        } else if cursor.row >= max_cursor_pos.1 {
            self.viewport.y += cursor.row.saturating_sub(max_cursor_pos.1);
        }
        self.viewport
    }

    /// Updates the number of rows that are currently shown on the viewport.
    /// Refers to the number of editor lines, not visual lines.
    pub(crate) fn update_num_rows(&mut self, num_rows: usize) {
        self.num_rows = num_rows;
    }

    /// Scrolls the viewport down based on the cursor's row position.
    ///
    /// This function adjusts the viewport to ensure that the cursor remains visible
    /// when moving down in a list of lines. It calculates the required scrolling
    /// based on the line width and wraps the content to fit within the maximum width and height.
    ///
    /// # Behavior
    ///
    /// If the cursor is already visible within the current viewport, no action is taken.
    /// Otherwise, the function calculates how many rows the content would need to wrap,
    /// and adjusts the viewport accordingly.
    fn scroll_down(
        &mut self,
        lines: &Lines,
        max_width: usize,
        max_height: usize,
        cursor_row: usize,
    ) {
        // If the cursor is already within the viewport, or there are no rows to display, return early.
        if cursor_row < self.viewport.y + self.num_rows || self.num_rows == 0 {
            return;
        }

        let mut remaining_height = max_height;

        let skip = lines.len().saturating_sub(cursor_row + 1);
        for (i, line) in lines.iter_row().rev().skip(skip).enumerate() {
            let line_width = chars_width(line);
            let current_row_height = LineWrapper::determine_split(line_width, max_width).len();

            // If we run out of height or exceed it, scroll the viewport.
            if remaining_height < current_row_height {
                let first_visible_row = cursor_row.saturating_sub(i.saturating_sub(1));
                self.viewport.y = first_visible_row;
                break;
            }

            // Subtract the number of wrapped rows from the remaining height.
            remaining_height = remaining_height.saturating_sub(current_row_height);
        }
    }
}

fn chars_width(chars: &[char]) -> usize {
    chars
        .iter()
        .fold(0, |sum, ch| sum + ch.width().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! update_view_offset_test {
        ($name:ident: {
        view: $given_view:expr,
        size: $given_size:expr,
        cursor: $given_cursor:expr,
        expected: $expected_offset:expr
    }) => {
            #[test]
            fn $name() {
                // given
                let mut view = $given_view;
                let size = $given_size;
                let cursor = $given_cursor;
                let lines = Lines::default();

                // when
                let offset = view.update_viewport_offset(size, cursor, &lines, false);

                // then
                assert_eq!(offset, $expected_offset);
            }
        };
    }

    update_view_offset_test!(
        // 0 <-   | --<-
        // 1 ---- | ----
        // 2 ---- |
        scroll_up: {
            view: ViewState{
                viewport: Offset::new(0, 1),
                editor_to_textarea_offset: Offset::default(),
                num_rows: 0,
            },
            size: (1, 2),
            cursor: Index2::new(0, 0),
            expected: Offset::new(0, 0)
        }
    );

    update_view_offset_test!(
        // 0 ---- |
        // 1 ---- | ----
        // 2 <-   | --<-
        scroll_down: {
            view: ViewState{
                viewport: Offset::new(0, 0),
                editor_to_textarea_offset: Offset::default(),
                num_rows: 0,
            },
            size: (1, 2),
            cursor: Index2::new(2, 0),
            expected: Offset::new(0, 1)
        }
    );

    update_view_offset_test!(
        scroll_left: {
            view: ViewState{
                viewport: Offset::new(1, 0),
                editor_to_textarea_offset: Offset::default(),
                num_rows: 0,
            },
            size: (2, 1),
            cursor: Index2::new(0, 0),
            expected: Offset::new(0, 0)
        }
    );

    update_view_offset_test!(
        scroll_right: {
            view: ViewState{
                viewport: Offset::new(0, 0),
                editor_to_textarea_offset: Offset::default(),
                num_rows: 0,
            },
            size: (2, 1),
            cursor: Index2::new(0, 2),
            expected: Offset::new(1, 0)
        }
    );
}
