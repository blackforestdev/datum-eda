# Datum GUI — Deep Tools & Parametric Operations

> **Status**: Governed active design spec.
> **Authority**: Under decision 019 (GUI product model) + the CLAUDE.md **Lean**
> ethos ("a capability should be a *parameter* of a small verb set, not a tool per
> object-class-times-format") and the one-mutation-path model (typed `Operation` +
> `commit()`, shared by human and AI). Informs the verb registry (decision 017):
> the deep tools below are *single parameterized verbs*, not tool families.
> **Scope**: The parameter spaces of Datum's flagship "deep" tools, and the
> tri-modal contract that lets the GUI, the AI, and the CLI all drive the same op.

## Thesis — the doorway is shallow, the operation is deep

A menu item is a doorway. The capability lives in the **parameter space of the
operation behind it**. A "tool" is therefore **one typed operation with a rich
parameter schema**, reached three ways that all emit the *same* op:

1. **Marking menu / presets** — the few most-common parameterizations, zero dialog
   (e.g. Align ▸ Left; Array ▸ 2×N grid).
2. **Tool inspector** — while the tool is active, the Inspector renders the *full*
   parameter schema as form fields (pitch, rows, seed, impedance…). Direct
   manipulation on the canvas updates the same params live.
3. **AI intent** (native terminal / assistant) — natural language populates the
   same params: *"array thermal vias 1 mm pitch under U1"*, *"place a 50 Ω diff
   pair on S101 and S102"*.

Plus the CLI (flags = the same params). **One op, many populations of its
parameters.** The parameter schema is the single source of truth — GUI form, AI
intent-mapping, and CLI flags all bind to it, and all commit the identical typed
operation (or proposal). This is the one-operation-vocabulary doctrine made
concrete, and the anti-tool-sprawl Lean ethos made structural.

**Why it wins:** professional speed lives in parametric placement/routing —
array a thermal-via field, lay an LED matrix, place a resistor array, route a diff
pair. Datum uniquely lets you reach the exact same depth by *dialing it precisely*
**or** *asking the AI*, because both drive one operation. No competitor with
separate GUI dialogs and a bolted-on chatbot can match that.

## The tri-modal contract (per deep tool)

Each deep tool declares its **parameter schema once** (typed, with defaults +
ranges). From that one schema:
- the **tool inspector** generates its form (field per parameter, canvas preview);
- the **AI intent-mapper** validates/fills params from language against the schema;
- the **CLI/verb** exposes the params as flags;
- **commit** produces the same typed `Operation` (or a proposal under assistant
  provenance) through the one path.

Adding capability to a tool = **extend its parameter schema**, never add a sibling
tool. (A new "align mode" is a new enum value, not a new verb.)

---

## Flagship deep tools (parameter spaces)

### 1. Align & Distribute — a discipline, one verb
Verb: `datum.pcb.align_components` (exists, mode-parameterized) — extend its schema;
mirror for schematic symbols.
- **mode**: `align_left | align_right | align_top | align_bottom | center_h |
  center_v | distribute_h | distribute_v | to_grid | pack`
- **reference**: `selection_bounds | primary_selection | named_object | board_origin`
- **spacing**: `equal_centers | equal_gaps | fixed_pitch(value)` (for distribute)
- **grid**: pitch + origin (for to_grid)
- **axis / edge** derived from mode.
One guarded `OperationBatch` (per-object `Set…Position`), skips locked. Marking-menu
presets = the common modes; inspector = full schema; AI = *"line these up on the
left"* / *"space evenly 2 mm"*.

### 2. Parametric Array / Pattern Placement — the speed feature
Verb (buildout): `place_array` over a source (component / via / pad / footprint).
- **pattern**: `grid | linear | circular | along_path`
- **counts**: rows, cols (grid) · count (linear/circular)
- **pitch**: pitch_x, pitch_y (grid) · pitch (linear) · angle_step + radius (circular)
- **origin / seed**: seed point + growth **direction** (±x/±y quadrant)
- **per-step transform**: rotation_step, mirror_step
- **naming / net increment**: reference auto-increment (R1…Rn), **net increment**
  (S101 → S10n) for bus-like arrays
- **grouping**: emit as a named group for later edit
Serves: thermal-via fields, **LED matrices / light engines**, resistor/passive
arrays, BGA fanout lattices. AI: *"5×5 LED grid, 5 mm pitch, refs D1.."*,
*"resistor array, 8 wide, 2 mm pitch"*.

### 3. Thermal Via Array / Stitching — array specialized to vias
Verb (buildout): `place_via_array` (a `place_array` preset over vias).
- **region**: under-selection / polygon / pad
- **pitch**, **pattern** (grid | staggered), **net**, **layer_span**
- **keepout / clearance** to pads and edges, **thermal-relief** mode
AI: *"stitch GND vias 1 mm across this zone"*, *"thermal via array under U1's pad"*.

### 4. Differential Pair Routing — impedance-aware
Verb (buildout): `route_diff_pair` (impedance solver deferred per CLAUDE.md;
`ImpedanceSpec` stub landed — this spec is its target).
- **nets**: positive, negative (e.g. S101 / S102)
- **target_impedance** (Z0, differential) → **solved** trace width + gap from the
  stackup, or explicit width/gap override
- **coupling**: gap, max **uncoupled-length budget**
- **layer**, length-tune target
AI: *"route a 50 Ω diff pair on S101 and S102"* → `route_diff_pair{nets:[S101,S102],
z0:50}`; the same op the GUI diff-pair tool builds once the params are dialed.

---

## Real vs. future (honesty)

- **Real now:** `datum.pcb.align_components` (basic align/distribute, mode-param).
- **Buildout (target defined here):** `place_array` / `place_via_array`,
  `route_diff_pair` + the impedance solver (`ImpedanceSpec` stub only today).
  These join the `not_built` worklist; this spec is the parameter-space target the
  verbs build toward, so they land *deep* rather than shallow-then-patched.

## Modularity tie-in

A deep tool stays **one verb** in the registry and **one entry** in
`menu_model.json`; its richness is its parameter schema, not extra menu rows.
The tool inspector, AI intent-mapper, and CLI are three renderers of that one
schema. This is the "add/remove, not rewrite" spine applied to *tool depth*: to
make Align or Array more capable, you widen a schema, and every surface — GUI form,
AI language, CLI flags — gains it at once.
