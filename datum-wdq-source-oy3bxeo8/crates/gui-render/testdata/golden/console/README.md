# Output-only Datum Console build goldens

These Layer-A fixtures inject deterministic, typed consumer feedback after
loading the in-repository native board fixture. They exercise the visual
acceptance matrix ratified by Product Mechanics 033: routine action echo,
maximized focused pane, editor-unfocused/Terminal-focused ownership, 1.0/1.25/
1.5/2.0 UI scales, tool prompt with the terminal dock open, narrow-pane refusal,
and expanded session history with resolver-journal projections.

The fixtures never inject command text, terminal lifecycle output, or authored
operations through the Console. Regenerate deliberately with
`datum-visual-fixture bless <manifest>` and record owner review in the GUI
conformance specification before accepting changed pixels.
