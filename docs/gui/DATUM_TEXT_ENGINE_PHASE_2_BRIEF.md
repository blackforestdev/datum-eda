# Datum Text Engine Phase 2 Brief

> **Status**: Ready for implementation
> **Primary research input**:
> `research/pcb-text-rendering/DATUM_TEXT_ENGINE_PHASE_2_RESEARCH.md`
> **Phase 1 predecessor**:
> `docs/gui/M7_IMP_014_IMPORTED_TEXT_NORMALIZATION_BRIEF.md`
> **Execution plan**:
> `docs/gui/DATUM_TEXT_ENGINE_PHASE_2_IMPLEMENTATION_PLAN.md`
> **Outline ownership note**:
> `docs/gui/DATUM_TEXT_ENGINE_OUTLINE_FILL_OWNERSHIP_NOTE.md`

## Purpose

Define the bounded implementation contract for evolving Datum's owned PCB text
engine from a deterministic stroke-only engine into a dual-mode text platform
capable of both:
- engineering-grade manufacturable text
- design-grade outline text for annotation, branding, and documentation

This brief exists to keep the next text-engine phase product-first and
architecture-driven rather than import-driven.

## Product Rule

Datum text is a first-class CAD subsystem.

Therefore:
- Datum owns the semantic text model
- Datum owns layout and transform semantics
- Datum owns final geometry generation
- importers adapt source text into Datum semantics
- renderers consume Datum-owned geometry

Datum text quality is not negotiable.

Therefore:
- Datum must aim for beautiful, high-fidelity text across all intents
- `Manufacturing` intent does **not** grant permission to render uglier text
- manufacturing policy belongs in validation, DRC, export, and explicit user
  workflow decisions, not in silent quality degradation
- authored beautiful fonts are valid manufacturing text unless a downstream fab
  rule explicitly says otherwise
- Datum should preserve authored intent and report fabrication risk, not
  preemptively downgrade appearance

Source CAD tools, cached source polygons, and font-specific importer quirks do
not define Datum's internal text architecture.

## Required Architecture

Phase 2 must formalize the three-layer split already established by the
research:

1. **Semantic text model**
   - content
   - font family / style
   - size
   - weight
   - line spacing
   - alignment
   - mirror
   - keep-upright
   - rotation
   - layer
   - render intent
   - optional class/style binding

2. **Layout engine**
   - line breaking
   - anchor handling
   - spacing
   - transforms
   - upright / mirror rules
   - block bounds

3. **Glyph backend**
   - stroke backend
   - outline backend
   - later custom / variable backend if needed

Phase 2 must not collapse these layers back together.

## Required Product Mode

Datum must become a high-fidelity text system across all intents.

Intent exists to control:
- semantics
- defaults
- validation policy
- downstream fabrication/export policy

Intent does **not** exist to decide whether text may look crude.

Phase 2 must support:
- beautiful high-fidelity text for manufacturing, annotation, branding,
  documentation, and UI preview
- deterministic geometry generation across intents
- explicit fabrication validation without visual downgrade

Phase 2 must not encode the doctrine:
- manufacturing text may be ugly
- design text is the only text allowed to be beautiful

## Backend Strategy

The required backend strategy is **hybrid**, but backend choice must not create
a quality caste system.

Default mapping:
- `RenderIntent::Manufacturing` -> stroke backend
- `RenderIntent::Annotation` -> outline backend
- `RenderIntent::Branding` -> outline backend
- `RenderIntent::Documentation` -> outline backend
- `RenderIntent::UiPreview` -> outline backend

Allowed adjustment:
- specific intents may later become user-configurable, but the system default
  must remain stable and documented

Required doctrine:
- all defaults must target high fidelity
- backend choice must optimize fidelity, determinism, and manufacturability
  together
- manufacturing intent may add validation and export constraints, but it must
  not exist as a justification for crude visuals

Not acceptable:
- stroke-only forever
- outline-only replacement of engineering text
- per-importer backend selection
- silent quality downgrade because text is tagged `Manufacturing`

## Required Engine Interfaces

Phase 2 must introduce explicit engine-owned contracts for:

- `RenderIntent`
- `GlyphBackend`
- text-family / style registry
- outline tessellation contract
- backend selection policy

The layout engine must be backend-agnostic.

Recommended shape:

```rust
pub enum RenderIntent {
    Manufacturing,
    Annotation,
    Branding,
    Documentation,
    UiPreview,
}

pub trait GlyphBackend {
    fn kind(&self) -> GlyphBackendKind;
    fn metrics(&self, run: &TextRun) -> Result<TextRunMetrics>;
    fn shape(&self, run: &PositionedTextRun) -> Result<OwnedTextGeometry>;
}
```

Exact type names may differ, but the split must remain explicit.

## Library Policy

Phase 2 outline work must follow the researched permissive stack:

- `ttf-parser` for outline extraction
- `kurbo` for curve math / flattening support
- `lyon` for tessellation
- `cavalier_contours` for engineering-mode offset work
- `i_overlay` for boolean union / fill preparation

No GPL-class direct integration is allowed.
No system-font runtime dependency is allowed.

## Font Bundle Policy

Phase 2 default bundle is the researched five-role set:

- stroke / engineering: `Newstroke` (CC0)
- technical sans: `Inter` (OFL-1.1)
- condensed annotation: `IBM Plex Sans Condensed` (OFL-1.1)
- display / branding: `Inter Display` (OFL-1.1)
- mono: `JetBrains Mono` (OFL-1.1)

Bundle policy:
- compile-time vendored assets
- explicit provenance tracking
- no ambient OS font dependency for required product behavior

## Manufacturing Constraint Policy

Phase 2 must turn fab text constraints into explicit product rules.

Default thresholds from research:
- default minimum height: `0.8 mm`
- default minimum stroke width: `0.15 mm`
- recommended conservative height: `1.0 mm`
- recommended conservative stroke width: `0.20 mm`

Behavior rule:
- warn and preserve authored geometry at DRC time
- do not silently resize or clamp geometry during normal editing/render
- do not silently substitute a lower-fidelity backend just because text is
  manufacturing-facing
- allow beautiful authored fonts for manufacturing text; use validation to flag
  risk, not degradation to preempt it

Not acceptable:
- silent auto-resize
- importer-time coercion
- renderer-time hidden correction
- policy that equates manufacturable with visually inferior

## Typography Scope

Phase 2 must implement only the researched Tier 1 / Tier 2 controls.

Tier 1:
- font selection
- text height
- stroke width
- H/V justification
- layer
- mirror
- rotation
- multiline

Tier 2:
- keep-upright
- italic
- bold
- line spacing
- class-based style presets

Skip for this phase:
- full justification
- curved/path text
- drop caps
- variable-axis UI controls
- general-purpose desktop-publishing features

## Determinism Contract

The biggest implementation risk identified by research is cross-architecture
variation in outline flattening / tessellation.

Required contract:
- font parsing must be deterministic
- flatten output must be stable across runs on the supported CI architecture
- Datum may explicitly scope golden-fixture determinism to
  `x86_64-linux-gnu`

Required mitigation:
- add a determinism gate test before outline backend rollout spreads
- test reads a vendored font, flattens a fixed glyph set at fixed tolerance,
  and byte-compares the output against the golden fixture

Phase 2 must not ship outline text without this gate.

## Fidelity Fixture Contract

The native text-engine visual fixture manifest lives in:

- [DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md](/home/bfadmin/Documents/datum-eda/docs/gui/DATUM_TEXT_ENGINE_FIDELITY_FIXTURES.md)

Stable project fixtures live under:

- `crates/engine/testdata/golden/text/native/`

These fixtures are the accepted visual regression surface for:

- intent/family policy
- manufacturing-default typography
- outline curve fidelity
- dense small-label behavior
- rotated/mirrored/keep-upright text
- mixed family and explicit override behavior

Any text-engine change that affects these areas must be reviewed against this
fixture set. The long-term gate is automated screenshot-golden comparison; until
that harness exists, manual screenshots from these fixtures are required review
evidence.

## Scope

This phase covers:
- PCB board text engine evolution
- imported-board text backend expansion
- native board text backend expansion
- style / intent architecture needed to support both

This phase does not require:
- schematic text unification in the same patch set
- arbitrary complex-script shaping
- path text
- a generic desktop publishing toolchain inside Datum

## Acceptance Criteria

Phase 2 is complete only when all of the following are true:

- Datum text uses an explicit three-layer architecture
- Datum text supports both engineering and design render intents
- backend selection is driven by `RenderIntent`, not importer/source quirks
- manufacturing intent does not impose lower visual fidelity
- outline backend exists and is usable for annotation/branding/documentation
- bundled fonts are vendored with explicit provenance and permissive licenses
- DRC policy for minimum text geometry is explicit and non-silent
- determinism gate tests exist for the outline path
- Phase 1 owned stroke engine remains the shared foundation rather than being
  bypassed by new ad hoc code paths
