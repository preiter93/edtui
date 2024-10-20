use ratatui::text::Span;

use crate::helper::{char_width, span_width};

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

    pub(crate) fn wrap_spans(spans: Vec<Span<'_>>, max_width: usize) -> Vec<Vec<Span<'_>>> {
        let mut wrapped_lines = Vec::new();
        let mut current_line = Vec::new();
        let mut current_line_width = 0;

        for span in spans {
            // If adding this span exceeds the max width, handle wrapping
            if current_line_width + span_width(&span) > max_width {
                let mut remaining_span = span.clone();
                let mut split_at = max_width - current_line_width;

                while span_width(&remaining_span) > split_at {
                    let (fitting_part, rest) = Self::split_span_at(remaining_span, split_at);
                    current_line.push(fitting_part.clone());
                    wrapped_lines.push(current_line.clone());

                    // Prepare for the next line
                    current_line.clear();
                    remaining_span = rest;
                    split_at = max_width; // FIXME: Take unicode size into account
                }

                // Add remaining part to the current line
                current_line_width = span_width(&remaining_span);
                current_line.push(remaining_span);
            } else {
                // No wrapping needed, just add the span
                current_line_width += span_width(&span);
                current_line.push(span);
            }
        }

        // Add any remaining content as the last line
        if !current_line.is_empty() {
            wrapped_lines.push(current_line);
        }

        wrapped_lines
    }

    fn split_span_at(span: Span, split_at: usize) -> (Span, Span) {
        let mut current_width = 0;
        let span_content = span.content;
        let style = span.style;
        for (i, ch) in span_content.chars().enumerate() {
            current_width += char_width(ch);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_wrapper() {
        let spans = vec![Span::raw("Hello"), Span::raw("World")];
        let wrapped_spans = LineWrapper::wrap_spans(spans, 3);

        assert_eq!(wrapped_spans[0], vec![Span::raw("Hel")]);
        assert_eq!(wrapped_spans[1], vec![Span::raw("lo"), Span::raw("W")]);
        assert_eq!(wrapped_spans[2], vec![Span::raw("orl")]);
        assert_eq!(wrapped_spans[3], vec![Span::raw("d")]);
    }

    fn test_line_wrapper_determine_split() {
        let line_widths = LineWrapper::determine_split(5, 3);

        assert_eq!(line_widths[0], 3);
        assert_eq!(line_widths[1], 2);

        let line_widths = LineWrapper::determine_split(6, 3);

        assert_eq!(line_widths[0], 3);
        assert_eq!(line_widths[1], 3);
    }
}
