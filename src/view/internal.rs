#[cfg(feature = "syntax-highlighting")]
use crate::SyntaxHighlighter;
use crate::{
    helper::{char_width, span_width, split_str_at},
    state::selection::Selection,
};
use jagged::Index2;
use ratatui::{style::Style, text::Span};

pub(crate) enum DisplayLine<'a> {
    Wrapped(Vec<Vec<Span<'a>>>),
    Single(Vec<Span<'a>>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct InternalSpan {
    pub(crate) content: String,
    pub(crate) style: Style,
}

impl InternalSpan {
    pub(crate) fn new<T: Into<String>>(content: T, style: &Style) -> Self {
        Self {
            content: content.into(),
            style: *style,
        }
    }

    pub(crate) fn spans_len(spans: &[Self]) -> usize {
        spans.iter().fold(0, |sum, span| sum + span.content.len())
    }

    /// Applies a [`Selection`] to an array of spans returning a modified
    /// array of spans.
    fn apply_selection(
        spans: &[Self],
        row_index: usize,
        selection: &Selection,
        style: &Style,
    ) -> Option<Vec<InternalSpan>> {
        let spans_len = InternalSpan::spans_len(spans);
        let (start_col, end_col) = selection.selected_columns_in_row(spans_len, row_index)?;
        debug_assert!(end_col >= start_col, "{start_col} {end_col}");

        Some(Self::split_spans(spans, start_col, end_col, style))
    }

    /// Splits spans by `crop_at` from the left.
    ///
    /// When the editor is scrolled horizontally, we have to crop
    /// text from the left.
    fn crop_spans(spans: &mut Vec<Self>, crop_at: usize) {
        if crop_at == 0 {
            return;
        }

        let mut span_offset = 0;
        let mut first_visible_span = 0;
        let mut split_span_at = 0;

        for (i, span) in spans.iter().enumerate() {
            let span_width = span.content.len();
            let span_start = span_offset;
            let span_end = span_offset + span_width;

            // span is fully on screen
            // ---|span|
            //  |
            if span_start >= crop_at {
                break;
            }

            // span must be cut to fit on screen
            // -|span|
            //   |
            if span_end > crop_at {
                split_span_at = crop_at - span_start;
                break;
            }

            // span is not shown on screen
            first_visible_span = i + 1; // remove the full span
            span_offset += span_width;
        }

        if first_visible_span > 0 {
            spans.drain(0..first_visible_span);
        }

        if split_span_at > 0 {
            let first_span = spans.remove(0);
            let (_, right) = split_str_at(&first_span.content, split_span_at);
            spans.insert(0, InternalSpan::new(right, &first_span.style));
        }
    }

    fn split_spans(
        spans: &[Self],
        split_start: usize,
        split_end: usize,
        style: &Style,
    ) -> Vec<Self> {
        let mut new_spans: Vec<InternalSpan> = Vec::new();
        let mut offset = 0;

        for span in spans {
            let span_len = span.content.chars().count();
            let span_start = offset;
            let span_end = offset + span_len.saturating_sub(1);

            // Case a: Span ends before split_start, append it unchanged
            // Case b: Span starts after split_end, append it unchanged
            if span_end < split_start || span_start > split_end {
                new_spans.push(span.clone());
            }
            // Case c: Split front
            else if split_start <= span_start && split_end < span_end {
                let split_point = split_end - span_start + 1;
                let (left, right) = split_str_at(&span.content, split_point);
                new_spans.push(InternalSpan::new(left, style));
                new_spans.push(InternalSpan::new(right, &span.style));
            }
            // Case d: Split back
            else if split_start > span_start && split_end >= span_end {
                let split_point = split_start - span_start;
                let (left, right) = split_str_at(&span.content, split_point);
                new_spans.push(InternalSpan::new(left, &span.style));
                new_spans.push(InternalSpan::new(right, style));
            }
            // Case e: Split middle
            else if split_start > span_start && split_end < span_end {
                let split_front = split_start - span_start;
                let split_back = split_end - span_start + 1;
                let (left, rest) = split_str_at(&span.content, split_front);
                let (middle, right) = split_str_at(&rest, split_back - split_front);

                new_spans.push(InternalSpan::new(left, &span.style));
                new_spans.push(InternalSpan::new(middle, style));
                new_spans.push(InternalSpan::new(right, &span.style));
            }
            // Case f: Split none (entire span is between split_start and split_end)
            else if split_start <= span_start && split_end >= span_end {
                new_spans.push(InternalSpan::new(span.content.clone(), style));
            }

            offset += span_len;
        }

        new_spans
    }
}

impl<'a> From<Span<'a>> for InternalSpan {
    fn from(value: Span) -> Self {
        Self::new(value.content, &value.style)
    }
}

impl<'a> From<InternalSpan> for Span<'a> {
    fn from(value: InternalSpan) -> Self {
        Self::styled(value.content, value.style)
    }
}

pub(crate) struct InternalLine<'a> {
    pub(crate) line: &'a [char],
    base: Style,
    pub(crate) highlighted: Style,
    pub(crate) row_index: usize,
    scroll_offset: usize,
}

impl<'a> InternalLine<'a> {
    pub(crate) fn new(
        line: &'a [char],
        base: Style,
        highlighted: Style,
        row_index: usize,
        col_offset: usize,
    ) -> Self {
        Self {
            line,
            base,
            highlighted,
            row_index,
            scroll_offset: col_offset,
        }
    }
}

impl<'a> InternalLine<'a> {
    fn get_style(&self, is_selected: bool) -> Style {
        if is_selected {
            self.highlighted
        } else {
            self.base
        }
    }

    /// Converts an `InternalLine` into a vector of `Span`s, applying styles based on the
    /// given selections.
    pub(crate) fn into_spans(self, selections: &[&Option<Selection>]) -> Vec<Span<'a>> {
        let mut spans = Vec::new();
        let mut current_span = String::new();
        let mut previous_is_selected = false;

        // Iterate over the line's characters, starting from the offset
        for (i, &ch) in self.line.iter().skip(self.scroll_offset).enumerate() {
            let position = Index2::new(self.row_index, self.scroll_offset + i);

            // Check if the current position is selected by any selection
            let current_is_selected = selections
                .iter()
                .filter_map(|selection| selection.as_ref())
                .any(|selection| selection.contains(&position));

            // If the selection state has changed, push the current span and start a new one
            if i != 0 && previous_is_selected != current_is_selected {
                spans.push(Span::styled(
                    current_span.clone(),
                    self.get_style(previous_is_selected),
                ));
                current_span.clear();
            }

            previous_is_selected = current_is_selected;
            current_span.push(ch);
        }

        // Push the final span
        spans.push(Span::styled(
            current_span,
            self.get_style(previous_is_selected),
        ));

        spans
    }

    #[cfg(feature = "syntax-highlighting")]
    pub(crate) fn into_highlighted_spans(
        self,
        selections: &[&Option<Selection>],
        syntax_highligher: &SyntaxHighlighter,
    ) -> Vec<Span<'a>> {
        let line = self.line.iter().collect::<String>();
        let mut internal_spans = syntax_highligher.highlight_line(&line);

        let selections = selections.iter().filter_map(|selection| {
            selection
                .as_ref()
                .filter(|s| s.contains_row(self.row_index))
        });

        for selection in selections {
            if let Some(new_span) = InternalSpan::apply_selection(
                &internal_spans,
                self.row_index,
                selection,
                &self.highlighted,
            ) {
                internal_spans = new_span;
            }
        }

        if self.scroll_offset > 0 {
            InternalSpan::crop_spans(&mut internal_spans, self.scroll_offset);
        }

        internal_spans.into_iter().map(Span::from).collect()
    }
}

/// Finds the position of a character within wrapped spans based on a given
/// index position.
///
/// # Example
/// ```ignore
/// let wrapped_spans = vec![vec![Span::from("hello")], vec![Span::from("world")]];
/// let index = find_position_in_wrapped_spans(&wrapped_spans, 6, 5);
/// assert_eq!(index, Index2::new(1, 1));
/// ```
pub(super) fn find_position_in_wrapped_spans(
    wrapped_spans: &[Vec<Span>],
    position: usize,
    max_width: usize,
    tab_width: usize,
) -> Index2 {
    if wrapped_spans.is_empty() {
        return Index2::new(0, position);
    }

    let mut char_pos = position;

    for (row, spans) in wrapped_spans.iter().enumerate() {
        let row_char_count = count_characters_in_spans(spans);
        let max_char_pos = row_char_count.saturating_sub(1);

        if char_pos <= max_char_pos {
            let col = unicode_width_position_in_spans(spans, char_pos, tab_width);
            return Index2::new(row, col);
        }

        if row + 1 < wrapped_spans.len() {
            char_pos -= row_char_count;
        }
    }

    let last_span_width = match wrapped_spans.last() {
        Some(span) => spans_width(span, tab_width),
        None => 0,
    };

    if last_span_width >= max_width {
        Index2::new(wrapped_spans.len(), 0)
    } else {
        Index2::new(wrapped_spans.len().saturating_sub(1), last_span_width)
    }
}

/// Returns the position of a char in a string taking into
/// account unicode width.
pub(super) fn unicode_width_position_in_spans(spans: &[Span], n: usize, tab_width: usize) -> usize {
    let mut total_width = 0;
    let mut chars_counted = 0;

    for span in spans {
        for ch in span.content.chars() {
            if chars_counted >= n {
                return total_width;
            }
            total_width += char_width(ch, tab_width);
            chars_counted += 1;
        }
    }

    total_width
}

pub(crate) fn find_position_in_spans(spans: &[Span], char_pos: usize, tab_width: usize) -> usize {
    if spans.is_empty() {
        return char_pos;
    }

    unicode_width_position_in_spans(spans, char_pos, tab_width)
}

fn count_characters_in_spans(spans: &[Span]) -> usize {
    spans.iter().map(|span| span.content.chars().count()).sum()
}

fn spans_width(spans: &[Span], tab_width: usize) -> usize {
    spans
        .iter()
        .fold(0, |sum, span| sum + span_width(span, tab_width))
}

#[cfg(test)]
mod tests {
    use ratatui::style::Stylize;

    use super::*;

    #[test]
    fn test_internal_line_into_spans() {
        // given
        let base = Style::default();
        let hightlighted = Style::default().red();
        let line = "Hello".chars().into_iter().collect::<Vec<char>>();

        let selection = Some(Selection::new(Index2::new(0, 0), Index2::new(0, 2)));
        let selections = vec![&selection];

        // when
        let spans = InternalLine::new(&line, base, hightlighted, 0, 0).into_spans(&selections);

        // then
        assert_eq!(spans[0], Span::styled("Hel", hightlighted));
        assert_eq!(spans[1], Span::styled("lo", base));
    }

    #[test]
    fn test_internal_span_split_spans() {
        // given
        let base = &Style::default();
        let hightlighted = &Style::default().red();
        let spans = vec![
            InternalSpan::new("Hel", base),
            InternalSpan::new("lo!", base),
        ];

        // when
        let new_spans = InternalSpan::split_spans(&spans, 1, 1, &hightlighted);

        // then
        assert_eq!(new_spans[0], InternalSpan::new("H", base));
        assert_eq!(new_spans[1], InternalSpan::new("e", hightlighted));
        assert_eq!(new_spans[2], InternalSpan::new("l", base));
        assert_eq!(new_spans[3], InternalSpan::new("lo!", base));

        // when
        let new_spans = InternalSpan::split_spans(&spans, 1, 2, &hightlighted);

        // then
        assert_eq!(new_spans[0], InternalSpan::new("H", base));
        assert_eq!(new_spans[1], InternalSpan::new("el", hightlighted));
        assert_eq!(new_spans[2], InternalSpan::new("lo!", base));

        // when
        let new_spans = InternalSpan::split_spans(&spans, 1, 3, &hightlighted);

        // then
        assert_eq!(new_spans[0], InternalSpan::new("H", base));
        assert_eq!(new_spans[1], InternalSpan::new("el", hightlighted));
        assert_eq!(new_spans[2], InternalSpan::new("l", hightlighted));
        assert_eq!(new_spans[3], InternalSpan::new("o!", base));

        // when
        let new_spans = InternalSpan::split_spans(&spans, 1, 10, &hightlighted);

        // then
        assert_eq!(new_spans[0], InternalSpan::new("H", base));
        assert_eq!(new_spans[1], InternalSpan::new("el", hightlighted));
        assert_eq!(new_spans[2], InternalSpan::new("lo!", hightlighted));
    }

    #[test]
    fn test_split_spans_with_emoji() {
        // given
        let base = &Style::default();
        let hightlighted = &Style::default().red();
        let spans = vec![InternalSpan::new("Hell🙂!", base)];

        // when
        let new_spans = InternalSpan::split_spans(&spans, 2, 4, &hightlighted);

        // then
        assert_eq!(new_spans[0], InternalSpan::new("He", base));
        assert_eq!(new_spans[1], InternalSpan::new("ll🙂", hightlighted));
        assert_eq!(new_spans[2], InternalSpan::new("!", base));
    }

    #[test]
    fn test_internal_span_crop_spans() {
        // given
        let base = &Style::default();
        let original_spans = vec![
            InternalSpan::new("Hel", base),
            InternalSpan::new("lo", base),
            InternalSpan::new("World!", base),
        ];
        let mut spans = original_spans.clone();

        // when
        InternalSpan::crop_spans(&mut spans, 1);

        // then
        assert_eq!(spans[0], InternalSpan::new("el", base));
        assert_eq!(spans[1], InternalSpan::new("lo", base));
        assert_eq!(spans[2], InternalSpan::new("World!", base));

        // when
        let mut spans = original_spans.clone();
        InternalSpan::crop_spans(&mut spans, 2);

        // then
        assert_eq!(spans[0], InternalSpan::new("l", base));
        assert_eq!(spans[1], InternalSpan::new("lo", base));
        assert_eq!(spans[2], InternalSpan::new("World!", base));

        // when
        let mut spans = original_spans.clone();
        InternalSpan::crop_spans(&mut spans, 3);

        // then
        assert_eq!(spans[0], InternalSpan::new("lo", base));
        assert_eq!(spans[1], InternalSpan::new("World!", base));
    }

    #[test]
    fn test_internal_span_apply_selection() {
        // given
        let base = &Style::default();
        let hightlighted = &Style::default().red();
        let spans = vec![
            InternalSpan::new("Hel", base),
            InternalSpan::new("lo!", base),
        ];

        // when
        let selection = Selection::new(Index2::new(0, 1), Index2::new(0, 3));
        let new_spans =
            InternalSpan::apply_selection(&spans, 0, &selection, &hightlighted).unwrap();

        // then
        assert_eq!(new_spans[0], InternalSpan::new("H", base));
        assert_eq!(new_spans[1], InternalSpan::new("el", hightlighted));
        assert_eq!(new_spans[2], InternalSpan::new("l", hightlighted));
        assert_eq!(new_spans[3], InternalSpan::new("o!", base));
    }

    #[test]
    fn test_internal_span_apply_selection_with_emoji() {
        // given
        let base = &Style::default();
        let hightlighted = &Style::default().red();
        let spans = vec![
            InternalSpan::new("Hell🙂", base),
            InternalSpan::new("!", base),
        ];

        // when
        let selection = Selection::new(Index2::new(0, 3), Index2::new(0, 5));
        let new_spans =
            InternalSpan::apply_selection(&spans, 0, &selection, &hightlighted).unwrap();

        // then
        assert_eq!(new_spans[0], InternalSpan::new("Hel", base));
        assert_eq!(new_spans[1], InternalSpan::new("l🙂", hightlighted));
        assert_eq!(new_spans[2], InternalSpan::new("!", hightlighted));
    }

    #[test]
    fn test_unicode_width_position_in_spans() {
        let spans = vec![Span::from("a😀b"), Span::from("c😀d")];

        let index = unicode_width_position_in_spans(&spans, 2, 0);
        assert_eq!(index, 3);

        let index = unicode_width_position_in_spans(&spans, 5, 0);
        assert_eq!(index, 7);

        let index = unicode_width_position_in_spans(&spans, 99, 0);
        assert_eq!(index, 8);
    }

    #[test]
    fn test_find_position_in_wrapped_spans() {
        let line_1 = vec![Span::from("abc")];
        let line_2 = vec![Span::from("def")];
        let spans = vec![line_1, line_2];

        let position = find_position_in_wrapped_spans(&spans, 2, 3, 0);
        assert_eq!(position, Index2::new(0, 2));

        let position = find_position_in_wrapped_spans(&spans, 3, 3, 0);
        assert_eq!(position, Index2::new(1, 0));

        let position = find_position_in_wrapped_spans(&spans, 5, 3, 0);
        assert_eq!(position, Index2::new(1, 2));

        let position = find_position_in_wrapped_spans(&spans, 6, 3, 0);
        assert_eq!(position, Index2::new(2, 0));
    }

    #[test]
    fn test_find_position_in_wrapped_spans_with_emoji() {
        let line_1 = vec![Span::from("a😀b")];
        let line_2 = vec![Span::from("c😀")];
        let spans = vec![line_1, line_2];

        let position = find_position_in_wrapped_spans(&spans, 2, 4, 0);
        assert_eq!(position, Index2::new(0, 3));

        let position = find_position_in_wrapped_spans(&spans, 3, 4, 0);
        assert_eq!(position, Index2::new(1, 0));

        let position = find_position_in_wrapped_spans(&spans, 4, 4, 0);
        assert_eq!(position, Index2::new(1, 1));

        let position = find_position_in_wrapped_spans(&spans, 5, 4, 0);
        assert_eq!(position, Index2::new(1, 3));
    }
}
