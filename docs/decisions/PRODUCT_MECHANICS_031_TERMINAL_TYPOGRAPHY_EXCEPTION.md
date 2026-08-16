# Product Mechanics 031: Terminal Typography Exception

Status: ratified doctrine

## Decision

Datum's native terminal cell plane uses the already-vendored JetBrains Mono
Regular face. This is a narrow exception to the program-wide IBM Plex
typography law: application chrome, EDA-authored text, aligned data outside the
terminal, documentation, and manufactured text remain IBM Plex.

The exception was owner-approved on 2026-08-15 after native agent sessions
demonstrated that IBM Plex Mono lacks required box-drawing and core Powerline
glyphs. A terminal run uses JetBrains Mono as a whole; this is not per-glyph
fallback or ambient system-font discovery.

## Boundary

- The approved asset is `JetBrainsMono-Regular.ttf`, whose internal name/head
  metadata identifies version 2.305. It was vendored from an upstream repo
  snapshot on 2026-04-18 and is pinned exactly by SHA-256
  `e6fd0d7e91550b3ed2b735d4312474362c4716edc4fc0577a0f61ed782d5aed1`.
- Its applicable copyright and SIL OFL 1.1 text ship beside the asset at
  `jetbrains_mono/OFL.txt`.
- The asset is build-time embedded and never downloaded or discovered at
  runtime.
- This is a font-asset authorization, not authorization for a third-party code
  package, terminal implementation, parser, runtime fallback, or build tool.
- Decision 029 remains controlling: the terminal implementation and PTY/core
  source remain Datum-owned, and no Cargo dependency is added.
- The terminal renderer may claim only glyphs demonstrated by deterministic
  cmap/shaping evidence. Full Unicode, emoji, grapheme, width, and fallback
  behavior remain governed by the later terminal-core text/render packages.

## Evidence

Datum's font-system regression shapes representative ASCII, box-drawing, and
Powerline text entirely through the JetBrains Mono family with no `.notdef`
glyphs. Render-contract coverage proves terminal cells use the terminal face
while terminal chrome remains on the program-wide IBM Plex mono face.
