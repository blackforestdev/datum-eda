# M7-IMP-008 Implementation Brief (Revised)

> **Ticket**: `M7-IMP-008`
> **Stage**: Stage 2
> **Track**: Imported board fidelity inside opening `M7`
> **Status**: Rejected prior implementation; ready for corrected implementation
> **Supersedes**: prior M7-IMP-008 implementation (rejected)
>
> Prior M7-IMP-008 was rejected because it:
> - stored pad-local rotation alongside board-space pad position (mixed
>   coordinate spaces — an invalid placed-pad model)
> - removed the calibrated width/height swap that was carrying visible
>   correctness while no renderer consumer existed
> - treated "data-only" as renderer-neutral when it was not
>
> This revision corrects the model: world-space placed-pad transform
> computed at import time, no shims, no mixed-space semantics, no detours.

## Purpose

Resolve the mixed-coordinate-space model bug in imported pad rotation. A
placed pad must carry its position and its rotation in the same coordinate
space: board/world. This brief defines the correct transform composition
(footprint placement + pad-local rotation + back-side mirror) and how it
lands in the IR, the scene contract, and the validation surface.

## Problem

KiCad encodes pad transforms hierarchically:
- footprint `(at fp_x fp_y fp_rot)` places the footprint on the board
- footprint `(layer F.Cu | B.Cu | ...)` selects the side; back-side
  footprints are mirrored in footprint-local space
- pad `(at pad_x pad_y pad_rot)` places the pad inside the footprint's
  local frame

The final placed-pad world transform is the composition of these.

Datum currently:
- stores `PlacedPad.position` in board/world space (correct)
- does NOT store a final pad rotation
- applies a calibrated width/height swap at `sin(pad_local_rot) > 0.4` to
  carry approximate visual correctness through to the renderer

The swap was a correctness shim, not a model. This ticket replaces it with
the correct model. Nothing in the IR should live in two coordinate spaces.

## Scope

This ticket covers:
- adding `PlacedPad.rotation: i32` (integer degrees, board/world space)
- computing final world-space pad rotation from composed transforms in the
  KiCad importer
- computing final world-space pad center from the same composition
  (front-side is already correct via `transform_board_local_point`; back-side
  must apply the mirror before rotate/translate)
- removing the width/height swap as part of the same correction (not
  separately preserved as a shim)
- propagating world rotation through the scene contract
  (`PadPrimitive.rotation_degrees`)
- real-fixture validation against DOA2526 and datum-test

This ticket does **NOT** cover:
- renderer consumption of `rotation_degrees` — that is `M7-REN-007`
  (paired follow-on, separate ticket, must land immediately after)
- roundrect corner semantics (`M7-IMP-009`)
- mask/paste import (`M7-IMP-012`)
- any other pad-rotation path outside the KiCad importer

## Non-Goals (Explicit)

- No width/height swap will be reintroduced as a compatibility shim
- No local-rotation field will be added alongside world-space position
- No "temporary comfort moves" that restore previous visible behavior while
  bypassing the transform fix
- No synthetic KiCad strings in the validation tests

The accepted state between this ticket landing and the renderer follow-on
landing is: the IR carries correct world-space pad rotation, and the
renderer has not yet been updated to draw it. Visible pad orientation may be
incorrect in that window. That is the accepted price of the corrected model
and it forces the renderer follow-on to land promptly.

## Required Change

### IR Shape

Add to `PlacedPad` in `crates/engine/src/board/pad.rs`:

```rust
/// Final board/world-space pad orientation in normalized integer degrees
/// (CCW) in the range `[0, 360)`. Composed from footprint placement
/// rotation, pad-local rotation, and back-side mirror behavior. A placed
/// pad's `position` is in board space; `rotation` is in the same
/// coordinate space. Never store local rotation alongside world position.
#[serde(default)]
pub rotation: i32,
```

### Stored Angle Range

`PlacedPad.rotation` is normalized world-space degrees in `[0, 360)`.
`atan2` output is converted via `((deg.round() as i32).rem_euclid(360))`.
`PadPrimitive.rotation_degrees` (f32) mirrors the same range. Consumers
must not assume a `[-180, 180)` range.

### Transform Composition

For each pad inside a footprint:

- **Inputs:**
  - `fp_pos` — footprint world position (from footprint `(at x y rot)`)
  - `fp_rot` — footprint placement rotation in degrees (integer)
  - `fp_flipped` — `true` if footprint is on a back-side copper layer
    (e.g., `B.Cu`); `false` otherwise
  - `p_local` — pad local position (from pad `(at x y rot)`)
  - `theta_pad` — pad-local rotation in degrees

- **Back-side mirror `M`:** reflection in footprint-local space that KiCad
  applies when a footprint is flipped. Applied to local coordinates and to
  the pad orientation basis before the footprint rotation.

  **M must not remain symbolic in the landed implementation.** The chosen
  mirror axis/sign convention must be proven against real KiCad fixture
  output before code lands, and the final chosen convention must be
  recorded below in the "Back-Side Mirror Convention" section. Candidate
  conventions (reflect x: `(x,y) → (-x,y)` or reflect y: `(x,y) → (x,-y)`)
  must be discriminated by observing the world-space coordinates of a
  known back-side pad in a real KiCad board and comparing to this
  implementation's output. Do not pick based on "mathematically neat."

### Back-Side Mirror Convention

To be filled in by the implementing engineer before landing code. Must
cite the real KiCad fixture used to prove the choice, the named pad/pin
whose KiCad-truth coordinates were observed, and the chosen mirror matrix.

Until this section is filled in, no code implementing back-side composition
may land.

- **Front-side composition:**
  - `pad_center_world = T(fp_pos) * R(fp_rot) * p_local`
  - `pad_axes_world = R(fp_rot) * R(theta_pad)`

- **Back-side composition:**
  - `pad_center_world = T(fp_pos) * R(fp_rot) * M * p_local`
  - `pad_axes_world = R(fp_rot) * M * R(theta_pad)`

- **World rotation derived from the transformed basis:**
  - `x_axis_world = pad_axes_world * [1, 0]`
  - `pad_rotation_world = atan2(x_axis_world.y, x_axis_world.x)` converted
    to integer degrees and normalized to `[0, 360)` via `rem_euclid(360)`

**One helper produces both world center and world rotation from the same
composed transform.** No separate ad-hoc center vs. angle logic. Back-side
world center and back-side world rotation land together in the same change,
from the same helper. This is mandatory, not tentative.

This is the single source of truth for placed-pad orientation. No sign-rule
maintenance in scattered call sites; only the composition helper.

### No Downstream Recomposition

Once import produces world-space pad rotation, downstream layers must treat
it as authoritative:

- `gui-protocol` forwards imported world rotation unchanged. It does not
  add footprint rotation again.
- `gui-render` consumes `PadPrimitive.rotation_degrees` as the final
  display angle. It does not compose with component rotation.

If a scene primitive carries a world-space rotation, no layer downstream
re-composes it from component-level transforms. Double-application is
forbidden.

### Removal Of The Swap

The width/height swap at `sin(theta_pad) > 0.4` is removed. Dimensions
(`width`, `height`) are stored as KiCad declares them in the source. Final
placed-pad orientation is expressed via `PlacedPad.rotation`. No shim is
left behind.

### Scene Contract Propagation

Add to `PadPrimitive` in `crates/gui-protocol/src/lib.rs`:

```rust
#[serde(default)]
pub rotation_degrees: f32,
```

Populate from `PlacedPad.rotation`. Thread through the `BoardPadPayload`
scene-loader glue (`rotation: i32`) and the `EnginePadPayload` native-project
load glue (`rotation: Option<i32>`).

### Mechanical Test Sweep

The new required field on `PlacedPad` will break every engine test that
constructs `PlacedPad` via struct literal. Mechanical `rotation: 0` sweep
across those sites is required as validation maintenance (same category as
the earlier drill sweep). Sweep count is approximately 122 across 43 files.

## Minimum Code Surface

- `crates/engine/src/board/pad.rs` — add `rotation` field
- `crates/engine/src/import/kicad/skeleton.rs` — remove swap; implement
  composition (front + back-side), derive world rotation via atan2, store on
  `PlacedPad.rotation`; world-space pad center for back-side footprints also
  lands here if currently missing
- `crates/engine/src/api/ops_helpers.rs` — propagate `rotation: 0` in the
  two `PlacedPad` construction sites
- `crates/cli/src/command_project_board_pad.rs` — propagate `rotation: 0`
  in the two construction sites
- engine test files — mechanical `rotation: 0` sweep
- `crates/gui-protocol/src/lib.rs` — add `rotation_degrees` to
  `PadPrimitive`, thread through `BoardPadPayload` and `EnginePadPayload`,
  populate in the scene builder

## Validation On Real KiCad Fixtures

The validation surface is real KiCad boards from the frozen Stage 0 fixture
authority. No synthetic KiCad strings.

### Minimum four-case coverage

1. **Front-side footprint, rotated pad.** Pad-local rotation in
   {0°, 90°, 45°}. World rotation must equal pad-local rotation when
   footprint rotation is zero.
2. **Rotated footprint, unrotated pad.** Footprint rotation in
   {0°, 90°, 45°, 180°}. World rotation must equal footprint rotation.
3. **Rotated footprint + rotated pad.** Both non-zero. World rotation is the
   composition.
4. **Back-side footprint with rotated pad.** Mirror applied in
   footprint-local space before rotation. World rotation derived from the
   transformed basis via `atan2`.

If the frozen fixture set does not contain a real KiCad board that exercises
a specific case (e.g. a 45° placed pad), the implementing engineer must
ask the user to provide one. Do not fabricate.

### Fixture-Backed Assertions — KiCad Is The Oracle

Validation must use KiCad as the oracle. A test that computes the expected
rotation using the same composition helper under test is insufficient —
that only proves internal consistency with our own formula, not correctness
against KiCad.

For each covered case the test must:

1. Name a specific pad on a specific component on a real KiCad fixture
   (e.g. `Q5 pad "1"` on DOA2526).
2. Record the expected world-space angle as **observed from KiCad truth**
   — either by reading the KiCad PCB editor's displayed pad orientation or
   by hand-computing the expected world angle from the KiCad file contents
   independently of this implementation's helper.
3. Assert that `PlacedPad.rotation` produced by the importer equals that
   KiCad-observed expected angle (to within ±1° after normalization).

Self-referential tests (helper computes both the expected and actual
value) are not accepted as proof of correctness. A reviewer must be able
to open the fixture in KiCad and visually confirm that the test's expected
angle matches what KiCad displays.

Tests assert on the IR after import, not on GUI visual output (renderer
consumption is `M7-REN-007` scope). That means a KiCad-truth angle is
asserted against `PlacedPad.rotation` directly.

## Acceptance Criteria

- `PlacedPad` carries `position` and `rotation` in the same coordinate
  space (board/world)
- Importer computes world rotation via the composition helper; no ad-hoc
  sign rules scattered elsewhere
- Back-side footprint mirror is applied in footprint-local space before
  footprint rotation
- Width/height swap is gone; no shim reintroduced
- `PadPrimitive.rotation_degrees` in the scene matches `PlacedPad.rotation`
- Real-fixture tests pass for all four coverage cases
- No synthetic KiCad strings in tests

## Paired Follow-On: M7-REN-007

This ticket does not close the user-visible pad rotation story. The paired
renderer ticket `M7-REN-007` consumes `PadPrimitive.rotation_degrees` to
draw oriented pad shapes (rect, oval, roundrect) and produce oriented hit
regions. `M7-REN-007` must land immediately after `M7-IMP-008`. Until then,
pad visuals may render unrotated — that is the explicit, accepted
intermediate state.

## What This Brief Forbids

- A new local-rotation field alongside world-space position
- A width/height-swap shim marked "temporary"
- A revert-as-progress step
- Synthetic KiCad strings as validation
- Self-referential tests where the expected angle is computed by the same
  helper under test
- Landing back-side composition code before the Back-Side Mirror Convention
  section is filled in with real-fixture evidence
- Closing M7-IMP-008 before real-fixture assertions pass on all four cases
- Downstream layers (`gui-protocol`, `gui-render`) recomposing pad rotation
  from component rotation after import has produced the world-space value
