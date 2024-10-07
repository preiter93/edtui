use jagged::Index2;
use ratatui::text::Span;
use unicode_width::UnicodeWidthChar;

#[derive(Default)]
pub(crate) struct LineWrapper;

impl LineWrapper {
    /// Splits a given line width into multiple smaller widths, ensuring each width
    /// is no larger than the specified maximum width.
    pub(crate) fn determine_split(line_width: usize, max_width: usize) -> Vec<usize> {
        if line_width == 0 {
            return vec![0];
        }

        let mut remaining_width = line_width;
        let mut split_widths = Vec::new();

        while remaining_width > 0 {
            let current_chunk = std::cmp::min(remaining_width, max_width);
            split_widths.push(current_chunk);
            remaining_width = remaining_width.saturating_sub(max_width);
        }

        split_widths
    }

    pub(crate) fn wrap_spans(spans: Vec<Span>, max_width: usize) -> (Vec<usize>, Vec<Vec<Span>>) {
        let mut wrapped_lines = Vec::new();
        let mut current_line = Vec::new();
        let mut current_line_width = 0;
        let mut line_widths = Vec::new();

        for span in spans {
            let span_width = span.width();

            // If adding this span exceeds the max width, handle wrapping
            if current_line_width + span_width > max_width {
                let mut remaining_span = span.clone();
                let mut split_at = max_width - current_line_width;

                while remaining_span.width() > split_at {
                    let (fitting_part, rest) = Self::split_span_at(remaining_span, split_at);
                    current_line_width += fitting_part.width();
                    current_line.push(fitting_part.clone());
                    wrapped_lines.push(current_line.clone());
                    line_widths.push(current_line_width);

                    // Prepare for the next line
                    current_line.clear();
                    current_line_width = 0;
                    remaining_span = rest;
                    split_at = max_width; // FIXME: Take unicode size into account
                }

                // Add remaining part to the current line
                current_line_width = remaining_span.width();
                current_line.push(remaining_span);
            } else {
                // No wrapping needed, just add the span
                current_line.push(span);
                current_line_width += span_width;
            }
        }

        // Add any remaining content as the last line
        if !current_line.is_empty() {
            wrapped_lines.push(current_line);
            line_widths.push(current_line_width);
        }

        (line_widths, wrapped_lines)
    }

    fn split_span_at(span: Span, split_at: usize) -> (Span, Span) {
        let mut current_width = 0;
        let span_content = span.content;
        let style = span.style;
        for (i, ch) in span_content.chars().enumerate() {
            current_width += ch.width().unwrap_or(0);
            if current_width > split_at {
                let (a, b) = span_content.split_at(i);
                return (
                    Span::styled(a.to_string(), style),
                    Span::styled(b.to_string(), style),
                );
            }
        }

        (Span::styled(span_content, style), Span::styled("", style))
    }

    pub(crate) fn find_position(line_widths: &[usize], col: usize) -> Index2 {
        if line_widths.is_empty() {
            return Index2::new(0, col);
        }

        let mut length_offset = 0;

        for (i, &length) in line_widths.iter().enumerate() {
            if col < length_offset + length {
                return Index2::new(i, col.saturating_sub(length_offset));
            }
            if i + 1 < line_widths.len() {
                length_offset += length;
            }
        }

        Index2::new(
            line_widths.len().saturating_sub(1),
            col.saturating_sub(length_offset),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_wrapper() {
        let spans = vec![Span::raw("Hello"), Span::raw("World")];
        let (line_widths, wrapped_spans) = LineWrapper::wrap_spans(spans, 3);

        assert_eq!(wrapped_spans[0], vec![Span::raw("Hel")]);
        assert_eq!(wrapped_spans[1], vec![Span::raw("lo"), Span::raw("W")]);
        assert_eq!(wrapped_spans[2], vec![Span::raw("orl")]);
        assert_eq!(wrapped_spans[3], vec![Span::raw("d")]);

        assert_eq!(line_widths[0], 3);
        assert_eq!(line_widths[1], 3);
        assert_eq!(line_widths[2], 3);
        assert_eq!(line_widths[3], 1);
    }

    fn test_line_wrapper_determine_split() {
        let line_widths = LineWrapper::determine_split(5, 3);

        assert_eq!(line_widths[0], 3);
        assert_eq!(line_widths[1], 2);

        let line_widths = LineWrapper::determine_split(6, 3);

        assert_eq!(line_widths[0], 3);
        assert_eq!(line_widths[1], 3);
    }

    #[test]
    fn test_line_wrapper_find_position() {
        let line_widths = vec![2, 2, 1];

        assert_eq!(
            LineWrapper::find_position(&line_widths, 2),
            Index2::new(1, 0)
        );
        assert_eq!(
            LineWrapper::find_position(&line_widths, 5),
            Index2::new(2, 1)
        );
    }
}
