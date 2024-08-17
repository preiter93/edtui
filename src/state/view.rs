use crate::Index2;
use ratatui::layout::Rect;

/// Represents the (x, y) offset of the editor's viewport.
/// It represents the top-left local editor coordinate.
#[derive(Default, Debug, Clone)]
pub(crate) struct ViewState {
    /// The offset of the viewport.
    pub(crate) viewport: Offset,
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
    ) -> Offset {
        let limit = (
            size.0.saturating_sub(1) + self.viewport.x,
            size.1.saturating_sub(1) + self.viewport.y,
        );
        // scroll left
        if cursor.col < self.viewport.x {
            self.viewport.x = cursor.col;
        }
        // scroll right
        if cursor.col >= limit.0 {
            self.viewport.x += cursor.col.saturating_sub(limit.0);
        }
        // scroll up
        if cursor.row < self.viewport.y {
            self.viewport.y = cursor.row;
        }
        // scroll down
        if cursor.row >= limit.1 {
            self.viewport.y += cursor.row.saturating_sub(limit.1);
        }
        self.viewport
    }
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

                // when
                let offset = view.update_viewport_offset(size, cursor);

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
            },
            size: (2, 1),
            cursor: Index2::new(0, 2),
            expected: Offset::new(1, 0)
        }
    );
}
