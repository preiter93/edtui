/// Represents the (x, y) offset of the editor's viewport.
/// It represents the top-left local editor coordinate.
#[derive(Default, Debug, Clone)]
pub struct ViewState {
    /// The x-coordinate offset of the viewport.
    x: usize,
    /// The y-coordinate offset of the viewport.
    y: usize,
}

impl ViewState {
    /// Instantiates a new [`ViewOffset`] with specified x and y coordinates.
    pub(crate) fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    /// Updates the view's offset and returns the new offset.
    /// This method is used internally to modify the view's offset coordinates.
    pub(crate) fn update_offset(
        &mut self,
        size: (usize, usize),
        cursor: (usize, usize),
    ) -> (usize, usize) {
        let bottom_right = (
            self.x + size.0.saturating_sub(1),
            self.y + size.1.saturating_sub(1),
        );
        // scroll left
        if cursor.0 < self.x {
            self.x = cursor.0;
        }
        // scroll right
        if cursor.0 > bottom_right.0 {
            self.x += cursor.0.saturating_sub(size.0);
        }
        // scroll up
        if cursor.1 < self.y {
            self.y = cursor.1;
        }
        // scroll down
        if cursor.1 > bottom_right.1 {
            self.y += cursor.1.saturating_sub(bottom_right.1);
        }
        (self.x, self.y)
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
                let offset = view.update_offset(size, cursor);

                // then
                assert_eq!(offset, $expected_offset);
            }
        };
    }

    update_view_offset_test!(
        scroll_left: {
            view: ViewState::new(2, 0),
            size: (2, 1),
            cursor: (1, 0),
            expected: (1, 0)
        }
    );

    update_view_offset_test!(
        scroll_right: {
            view: ViewState::new(1, 0),
            size: (2, 1),
            cursor: (3, 0),
            expected: (2, 0)
        }
    );

    update_view_offset_test!(
        scroll_up: {
            view: ViewState::new(0, 2),
            size: (1, 2),
            cursor: (0, 1),
            expected: (0, 1)
        }
    );

    update_view_offset_test!(
        scroll_down: {
            view: ViewState::new(0, 1),
            size: (1, 2),
            cursor: (0, 3),
            expected: (0, 2)
        }
    );
}
