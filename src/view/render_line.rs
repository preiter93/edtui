use ratatui_core::{
    buffer::Buffer,
    layout::{Position, Rect},
    text::{Line, Span},
    widgets::Widget,
};

use crate::helper::{rect_indent_y, replace_tabs_in_span};

use super::internal::{find_position_in_spans, find_position_in_wrapped_spans};

/// An internal data type that represents a line for rendering.
/// A vector of spans represents a line. Wrapped lines consist
/// of an array of lines.
pub(super) enum RenderLine<'a> {
    Wrapped(Vec<Vec<Span<'a>>>),
    Single(Vec<Span<'a>>),
}

impl RenderLine<'_> {
    pub(super) fn num_lines(&self) -> usize {
        match self {
            RenderLine::Wrapped(lines) => lines.len().max(1),
            RenderLine::Single(_) => 1,
        }
    }

    /// Transforms from data coordinate to screen coordinate.
    pub(super) fn data_coordinate_to_screen_coordinate(
        &self,
        data_col: usize,
        area: Rect,
        tab_width: usize,
    ) -> Position {
        let index2 = match self {
            RenderLine::Wrapped(lines) => {
                find_position_in_wrapped_spans(lines, data_col, area.width as usize, tab_width)
            }

            RenderLine::Single(line) => find_position_in_spans(line, data_col, tab_width),
        };

        Position::new(
            area.left() + (index2.col as u16).min(area.width),
            area.top() + index2.row as u16,
        )
    }

    pub(super) fn render(self, mut area: Rect, buf: &mut Buffer, tab_width: usize) {
        match self {
            RenderLine::Wrapped(lines) => {
                for line in lines {
                    if area.height == 0 {
                        break;
                    }

                    render_line(area, buf, line, tab_width);
                    area = rect_indent_y(area, 1);
                }
            }
            RenderLine::Single(line) => render_line(area, buf, line, tab_width),
        }
    }
}

fn render_line(area: Rect, buf: &mut Buffer, mut line: Vec<Span>, tab_width: usize) {
    for span in &mut line {
        replace_tabs_in_span(span, tab_width);
    }

    Line::from(line).render(area, buf);
}
