use crate::{
    helper::{char_width, chars_width},
    view::line_wrapper::LineWrapper,
    Lines,
};
use ratatui::layout::Rect;

/// Represents the (x, y) offset of the editor's viewport.
/// It represents the top-left local editor coordinate.
#[derive(Default, Debug, Clone)]
pub(crate) struct ViewState {
    /// The offset of the viewport.
    pub(crate) viewport: Offset,
    /// The number of rows that are displayed on the viewport
    pub(crate) num_rows: usize,
    /// Sets the coordinates (upper-left corner of the terminal window) where
    /// the editor text is rendered to.
    ///
    /// Required to calculate the mouse position in relation to the text within the editor.
    pub(crate) screen_coordinates: Offset,
    /// The number of spaces used to display a tab.
    pub(crate) tab_width: usize,
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
    pub(crate) fn set_screen_coordinates<T: Into<Offset>>(&mut self, offset: T) {
        self.screen_coordinates = offset.into();
    }

    /// Updates the viewports horizontal offset.
    pub(crate) fn update_viewport_horizontal(
        &mut self,
        width: usize,
        cursor_col: usize,
        line: Option<&Vec<char>>,
    ) -> usize {
        let Some(line) = line else {
            self.viewport.x = 0;
            return self.viewport.x;
        };

        // scroll left
        if cursor_col < self.viewport.x {
            self.viewport.x = cursor_col;
            return self.viewport.x;
        }

        // Iterate forward from the viewport.x position and calculate width
        let mut max_cursor_pos = self.viewport.x;
        let mut current_width = 0;
        for &ch in line.iter().skip(self.viewport.x) {
            current_width += char_width(ch, self.tab_width);
            if current_width >= width {
                break;
            }
            max_cursor_pos += 1;
        }

        // scroll right
        if cursor_col > max_cursor_pos {
            let mut backward_width = 0;
            let mut new_viewport_x = cursor_col;

            // Iterate backward from max_cursor_pos to find the first fitting character
            for i in (0..=cursor_col).rev() {
                let char_width = match line.get(i) {
                    Some(&ch) => char_width(ch, self.tab_width),
                    None => 1,
                };
                backward_width += char_width;
                if backward_width >= width {
                    break;
                }
                new_viewport_x = new_viewport_x.saturating_sub(1);
            }

            self.viewport.x = new_viewport_x;
        }

        self.viewport.x
    }

    /// Updates the view ports vertical offset.
    pub(crate) fn update_viewport_vertical(&mut self, height: usize, cursor_row: usize) -> usize {
        let max_cursor_pos = height.saturating_sub(1) + self.viewport.y;

        // scroll up
        if cursor_row < self.viewport.y {
            self.viewport.y = cursor_row;
        }

        // scroll down
        if cursor_row >= max_cursor_pos {
            self.viewport.y += cursor_row.saturating_sub(max_cursor_pos);
        }

        self.viewport.y
    }

    /// Updates the view ports vertical offset.
    pub(crate) fn update_viewport_vertical_wrap(
        &mut self,
        width: usize,
        height: usize,
        cursor_row: usize,
        lines: &Lines,
    ) -> usize {
        // scroll up
        if cursor_row < self.viewport.y {
            self.viewport.y = cursor_row;
        }

        // scroll down
        self.scroll_down(lines, width, height, cursor_row);

        self.viewport.y
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
            let line_width = chars_width(line, self.tab_width);
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! update_view_vertical_test {
        ($name:ident: {
        view: $given_view:expr,
        height: $given_height:expr,
        cursor: $given_cursor:expr,
        expected: $expected_offset:expr
    }) => {
            #[test]
            fn $name() {
                // given
                let mut view = $given_view;
                let height = $given_height;
                let cursor = $given_cursor;

                // when
                let offset = view.update_viewport_vertical(height, cursor);

                // then
                assert_eq!(offset, $expected_offset);
            }
        };
    }

    macro_rules! update_view_horizontal_test {
        ($name:ident: {
        view: $given_view:expr,
        width: $given_width:expr,
        cursor: $given_cursor:expr,
        expected: $expected_offset:expr
    }) => {
            #[test]
            fn $name() {
                // given
                let mut view = $given_view;
                let width = $given_width;
                let cursor = $given_cursor;
                let line = vec![];

                // when
                let offset = view.update_viewport_horizontal(width, cursor, Some(&line));

                // then
                assert_eq!(offset, $expected_offset);
            }
        };
    }

    update_view_vertical_test!(
        // 0      | --<-
        // 1 --<- | ----
        // 2 ---- |
        scroll_up: {
            view: ViewState{
                viewport: Offset::new(0, 1),
                screen_coordinates: Offset::default(),
                num_rows: 0,
                tab_width: 2,
            },
            height:  2,
            cursor: 0,
            expected: 0
        }
    );

    update_view_vertical_test!(
        // 0 ---- |
        // 1 ---- | ----
        // 2 <-   | --<-
        scroll_down: {
            view: ViewState{
                viewport: Offset::new(0, 0),
                screen_coordinates: Offset::default(),
                num_rows: 0,
                tab_width: 2,
            },
            height:  2,
            cursor: 2,
            expected: 1
        }
    );

    update_view_horizontal_test!(
        scroll_left: {
            view: ViewState{
                viewport: Offset::new(1, 0),
                screen_coordinates: Offset::default(),
                num_rows: 0,
                tab_width: 2,
            },
            width: 2,
            cursor: 0,
            expected: 0
        }
    );

    update_view_horizontal_test!(
        scroll_right: {
            view: ViewState{
                viewport: Offset::new(0, 0),
                screen_coordinates: Offset::default(),
                num_rows: 0,
                tab_width: 2,
            },
            width: 2,
            cursor: 2,
            expected: 1
        }
    );
}
