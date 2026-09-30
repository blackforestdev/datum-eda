# Pre-measurement chrome assessment

The terminal difference is material to submitted work, even outside board damage.
`gpu_frame_painter.rs` submits Workspace text for each selected tile;
`text_gpu/draw.rs::render_layer` filters by layer then draws complete instance
ranges, without CPU culling glyphs against the tile scissor. Changed labels,
close glyphs and tab decorations change submitted geometry/text. Scissor rejection
is not proof of zero vertex/backend cost. No timing magnitude is inferred.

An empty tab list follows the fallback path: `terminal`, active decoration and
short width. The archived native run has one `shell 1` tab with close glyph and
closed dock. Native `terminal_session.rs` independently confirms the space in
that label. The corrected diagnostic initializes that one running active tab,
without unread indicators; active_dock_tab remainsNone, so its closed-dock styling
matches the archive. No terminal process, design model or renderer changes.

**Status required no correction.** Exact pixel comparison shows the original
setup's full status strip(y774..800) already matches the archived R4 final image
and pinned reference: zero differing pixels. The earlier visual report claiming
DRC68 was absent from the archive was wrong. A provisional setup removed it and
used `shell1` without a space; the pre-timing pixel gate rejected299pixels per
image (251status,48label). That failed setup and its executable are preserved.
The final initializer leaves finding_count68 untouched and spells `shell 1`.
No timing samples were collected during either setup proof.

The original R4 final PNG was independently extracted from its existing archive;
its y742..800 terminal/status pixels exactly equal the pinned reference. Before
timing, require every pixel of that full strip to match in all four corrected
H/V images. This covers tab geometry, close/plus glyphs, colors, status text and
segment geometry. No mask or tolerance. Retain the independently archived
camera/layout/selection/focus/pointer/hover comparisons and crosshair checks.

The fixed diagnostic session ID and empty event-log/activity metadata are inert
bookkeeping here. Source trace shows they do not enter the tab label, glyph
instances or status projection; the ID names hit regions and close-hover/drag
comparisons, with neither interaction active. The dock is closed and no terminal
screen is submitted. Reproducing an opaque historical session ID or launching a
PTY is unnecessary for this fixed resident-render diagnostic.

All corrections are selected from source and archived pixels before timing.
No renderer/control change, favorable-result selection or extra measured campaign.
Earlier invalid samples and the mistaken status assessment remain historical;
this exact-pixel finding supersedes that assessment without erasing it.
