use crate::state::selection::Selection;
#[cfg(feature = "syntax-highlighting")]
use crate::SyntaxHighlighter;
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
            let (_, right) = first_span.content.split_at(split_span_at);
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
            let span_width = span.content.len();
            let span_start = offset;
            let span_end = offset + span_width.saturating_sub(1);

            // Case a: Span ends before split_start, append it unchanged
            // Case b: Span starts after split_end, append it unchanged
            if span_end < split_start || span_start > split_end {
                new_spans.push(span.clone());
            }
            // Case c: Split front
            else if split_start <= span_start && split_end < span_end {
                let split_point = split_end - span_start + 1;
                let (left, right) = span.content.split_at(split_point);
                new_spans.push(InternalSpan::new(left, style));
                new_spans.push(InternalSpan::new(right, &span.style));
            }
            // Case d: Split back
            else if split_start > span_start && split_end >= span_end {
                let split_point = split_start - span_start;
                let (left, right) = span.content.split_at(split_point);
                new_spans.push(InternalSpan::new(left, &span.style));
                new_spans.push(InternalSpan::new(right, style));
            }
            // Case e: Split middle
            else if split_start > span_start && split_end < span_end {
                let split_front = split_start - span_start;
                let split_back = split_end - span_start + 1;
                let (left, rest) = span.content.split_at(split_front);
                let (middle, right) = rest.split_at(split_back - split_front);

                new_spans.push(InternalSpan::new(left, &span.style));
                new_spans.push(InternalSpan::new(middle, style));
                new_spans.push(InternalSpan::new(right, &span.style));
            }
            // Case f: Split none (entire span is between split_start and split_end)
            else if split_start <= span_start && split_end >= span_end {
                new_spans.push(InternalSpan::new(span.content.clone(), style));
            }

            offset += span_width;
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
}
