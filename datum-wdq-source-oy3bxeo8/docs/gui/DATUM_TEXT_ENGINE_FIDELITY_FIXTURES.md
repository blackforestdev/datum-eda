# Datum Text Engine Fidelity Fixtures

> **Status**: Active native fixture manifest for Datum-owned board text
> fidelity.

These fixtures are the visual regression surface for the Phase 2 text engine.
They exist to keep typography quality from regressing into local, symptom-level
fixes.

## Doctrine

- Text quality is not negotiable.
- Manufacturing intent must not downgrade visual fidelity.
- Backend selection is a semantic policy, not a renderer escape hatch.
- Font-family selection is source-aware: implicit defaults may be promoted by
  intent, while explicit family choices must be preserved.
- Renderer output must be judged from native Datum text fixtures, not only from
  imported-board examples.
- Bugs must be investigated at the responsible contract boundary before code is
  changed.

## Fixture Location

Stable fixture projects live under:

```text
crates/engine/testdata/golden/text/native/
```

The `tmp/text-*-repro` projects may be used as scratch copies while iterating,
but the stable truth set is the checked fixture directory above.

## Fixture Set

| Fixture | Path | Purpose |
| --- | --- | --- |
| `text-intent-repro` | `crates/engine/testdata/golden/text/native/text-intent-repro` | Intent and family comparison: manufacturing, annotation, branding, documentation, and explicit mono override. |
| `text-fidelity-repro` | `crates/engine/testdata/golden/text/native/text-fidelity-repro` | Hero-size and tiny curved-glyph fidelity using `O/D/S/Q/8`-class glyphs. |
| `text-transform-repro` | `crates/engine/testdata/golden/text/native/text-transform-repro` | Rotation, mirror, keep-upright, and alignment behavior through the native text path. |
| `text-density-repro` | `crates/engine/testdata/golden/text/native/text-density-repro` | Dense annotation fields, manufacturing defaults, tiny text, and mixed family/intent behavior. |

## Launch Commands

```bash
cargo run -p datum-gui-app -- --project-root /home/bfadmin/Documents/datum-eda/crates/engine/testdata/golden/text/native/text-intent-repro
```

```bash
cargo run -p datum-gui-app -- --project-root /home/bfadmin/Documents/datum-eda/crates/engine/testdata/golden/text/native/text-fidelity-repro
```

```bash
cargo run -p datum-gui-app -- --project-root /home/bfadmin/Documents/datum-eda/crates/engine/testdata/golden/text/native/text-transform-repro
```

```bash
cargo run -p datum-gui-app -- --project-root /home/bfadmin/Documents/datum-eda/crates/engine/testdata/golden/text/native/text-density-repro
```

## Inspector Control Surface

Board text selection exposes the current semantic text controls in the
Inspector. Center explicit-value controls and edge/cycle controls now focus the
project terminal with journaled `datum-eda project edit-board-text` commands
for review before execution. The renderer only emits hit targets and redraws
the resulting scene.

Current controls:

- `INTENT`: cycles Datum render intent through manufacturing, annotation,
  branding, documentation, and UI preview.
- `FONT`: cycles the registered product families: Newstroke, Inter, Inter
  Display, IBM Plex Sans Condensed, and JetBrains Mono.
- `HEIGHT`: scales text height by a proportional step and scales stroke width
  by the same ratio so weight does not silently drift.
- `ROT`: rotates selected board text in normalized 90-degree steps.
- `ALIGN`: cycles horizontal and vertical alignment independently.
- `LINE`: steps line spacing within the bounded semantic range.
- `BOLD`: toggles the engine-backed bold semantic, which maps to the outline
  variable-font weight axis where the active font supports it.
- `MIRROR` and `UPRIGHT`: toggle the boolean engine text semantics.

Style controls are intentionally not exposed yet. The engine registry currently
defines only `regular`, so a style button would be a fake UI affordance until
additional registered styles exist.
Italic is also intentionally not exposed yet: the native field exists, but the
engine does not currently apply italic outline geometry.

## Explicit Value Commands

The Inspector edit affordances now prefill the project terminal with
canonical `datum-eda project edit-board-text "$DATUM_PROJECT_ROOT" --text
<uuid> ...` commands. They are not assistant prompts and do not use a
GUI-private JSON writer. The user can review or edit the shell command before
pressing Enter, and execution routes through the journaled `SetBoardText`
operation path.

Supported terminal-prefilled options:

- `--height-nm <nm>` sets text height in nanometers.
- `--rotation-deg <degrees>` normalizes into `0..359`, for example `-90`
  stores `270`.
- `--line-spacing-ratio-ppm <ppm>` sets multiline spacing directly.
- `--value <text>` replaces the selected board text string.
- `--h-align <left|center|right>` and `--v-align <top|center|bottom>` set
  anchor alignment.
- `--render-intent <manufacturing|annotation|branding|documentation|ui_preview>`
  sets the render intent exactly.
- `--family <newstroke|inter|inter_display|ibm_plex_sans_condensed|jetbrains_mono>`
  sets the registered font family exactly and records the family as an
  explicit text choice.

Inspector entry points:

- The selected board-text Inspector shows `EDGE +/-   CENTER EDIT` as the
  in-product hint for numeric rows.
- Click the selected board-text `TEXT` row or `EDIT CONTENT` row to focus the
  terminal prefilled with an `edit-board-text --value ...` command.
- Click the center of `HEIGHT`, `ROT`, or `LINE` to focus the terminal
  prefilled with that row's explicit `edit-board-text` command.
- Click the left/right edges of `HEIGHT`, `ROT`, or `LINE` to focus the
  terminal prefilled with the next stepper command.
- Click the center of `INTENT`, `FONT`, or `ALIGN` to focus the terminal
  prefilled with that row's exact `edit-board-text` command.
- Click the edges of `INTENT` or `FONT` to focus the terminal prefilled with
  the next cycle command.
- Click the left edge of `ALIGN` to prefill the next horizontal alignment, or
  the right edge to prefill the next vertical alignment.
- Press Enter in the terminal to submit the same CLI command a user or
  terminal-launched agent would run. The Inspector does not write values
  through a separate mutation path.

The explicit center `--height-nm` command edits height only. The Inspector
`HEIGHT` edge steppers generate `--height-nm` plus `--stroke-width-nm` so
proportional height/stroke scaling still routes through the same journaled CLI
path.

## Required Review Checks

### Intent Fixture

- Manufacturing text uses the same high-fidelity outline path as other default
  text.
- Branding has a stronger visual voice than annotation.
- Documentation and mono overrides remain visually distinct.
- No line looks like fallback stroke text unless the fixture explicitly asks for
  a stroke family.

### Fidelity Fixture

- Curves in `O`, `D`, `S`, `Q`, and `8` remain smooth at hero scale.
- Tiny text remains legible without ballooning or collapsing counters.
- Manufacturing text does not regress to a lower-quality backend.

### Transform Fixture

- `0/90/180/270` text preserves orientation and glyph quality.
- Mirrored text uses the engine's mirror semantics, not renderer-side tricks.
- Keep-upright behavior remains readable and alignment-correct.
- Center/right/top/bottom anchoring remains stable under rotation.

### Density Fixture

- Dense small labels remain readable.
- `R101 10K 1%`, `R102 10K 1%`, and `tiny curve odsq8` stay on the outline path.
- Mixed intent/family text reads as one coherent typography system.
- Right-aligned notes anchor consistently.

The `text-density-repro` fixture intentionally makes right-aligned vertical
text alignment explicit. The `hero label density` object and the far-right
`dense notes align-right` object both declare `v_align: bottom` so the visual
golden is tied to Datum text semantics, not to an implicit fixture default.

The April 2026 `text-density-repro` rebless was accepted after the diff was
traced to the vertical `hero label density` object only. Alignment and family
probe fixtures showed the checked-in golden encoded stale text geometry, while
the current renderer output matched the Datum-owned text engine semantics.
Only this fixture was reblessed.

## Screenshot Golden Policy

Layer A visual goldens are now the primary protection for this fixture set.

The adopted harness design is:

- [DATUM_GUI_VISUAL_REGRESSION_HARNESS.md](/home/bfadmin/Documents/datum-eda/docs/gui/DATUM_GUI_VISUAL_REGRESSION_HARNESS.md)

The active harness command is:

```bash
cargo test -p datum-gui-render --features visual --test visual_goldens -- --ignored --nocapture
```

The harness must:

- load each native fixture through the same Datum project-root semantics used
  for manual review
- set deterministic window size, zoom, pan, and layer visibility
- render through `gui-render` offscreen
- compare against accepted `.golden.png` files
- retain failure artifacts and clean passing artifacts

Manual screenshots remain useful review evidence, but they are no longer the
only regression surface for text-engine changes that affect layout, backend
selection, outline flattening, fill resolution, font assets, or renderer fill
behavior.

## Non-Negotiable Regression Rule

If a text change makes one of these fixtures look better by locally special
casing a string, glyph, family, or visual symptom, the change is invalid.

Acceptable fixes must identify and repair the responsible semantic contract:

- default family / style policy
- family source policy
- backend selection
- text attribute normalization
- outline flattening policy
- font variation policy
- fill-rule resolution
- renderer consumption of already-resolved geometry
