Unreleased
--------

0.3.5 - 4 May 2024
===================
- Map Redo from `r` to `<ctrl>+r`

Released
--------
0.3.4 - 4 May 2024
===================
- Fix panic when appending new line to empty buffer
 
0.3.3 - 2 April 2024
===================
- Fix panic in delete selection
- Add SelectLine
- Fix selection bug
- Render visual selection correctly
- Bump edtui-jagged to v0.1.3

0.3.2 - 18 February 2024
===================
- Use "ciw" for selection between delimiters instead of "cw".
- Minor bugfixes

0.3.1 - 11 February 2024
===================

- Paste over selection
- Support more motions in visual mode
- Add demo

0.3.0 - 3 February 2024
===================

Bump ratatui to version 0.26.0

0.2.2 - 3 February 2024
===================

- Fix bug in append string when no data was present
- Update default color scheme
- Add search functionality. Trigger search via '/'.
- Small breaking change in StatusLine widget. Replace 'content()' with 'mode()'.
- Bugfix: Fix panic in some cases when deleting selection
- Bugfix: Move left when cursor pos was larger than current col len

0.2.1 - 29 December 2023
===================

- Fix bug in WordBackward action
- Fix bug in RemoveChar action
- Fix bug in DeleteSelection action
 
0.2.0 - 25 December 2023
===================

- Add clipboard support
- Bump ratatui to version 0.25
- Refactor action enum
- Move jagged datatype into separate crate "edtui-jagged"
