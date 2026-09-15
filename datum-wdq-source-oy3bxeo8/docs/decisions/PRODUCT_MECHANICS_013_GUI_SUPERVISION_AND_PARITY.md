# Product Mechanics 013: GUI Supervision And Parity — VACATED

Status: VACATED (tombstone). This decision was reverted and its number retired.
Date: 2026-06-22 (created), 2026-06-23 (vacated), 2026-07-05 (tombstone recorded)

> This record exists only to close the `013` slot. There is no active doctrine
> here. Decision numbering intentionally skips from `012` to `014`.

## What 013 was

`013` originally ratified a "GUI supervision-reflection" track: a read-only GUI
surface (Supervision* types, a journal/provenance/findings "instrument panel",
9 supervision goldens, net-new read-only `SessionCommand` arms) whose purpose was
to let a supervisor visually audit headless engine progress. It landed in
`720eb55` alongside a full GUI spec set (`specs/GUI_SPEC.md` + `specs/gui/*`,
7 area specs).

## Why it was vacated

The supervision-reflection framing was a **misdirection of the GUI requirement**.
The supervisor does not want a meta journal/ledger viewer bolted beside the app;
they want the **real EDA canvas** (board / inspector / data panels / schematic)
reflecting live engine state. The entire track was backed out in `7e352f4`
(2026-06-23, "Remove supervision-reflection misfire"): the supervision code, the
`013` decision record, `specs/GUI_SPEC.md`, and `specs/gui/*` were all deleted.
Core doctrine `000..012` was untouched.

## What supersedes it

GUI governance was reconstituted on the correct framing — build the real EDA
surfaces, populated from the engine, through the same operation/commit model:

- `docs/decisions/PRODUCT_MECHANICS_014_UI_LAYOUT_SYSTEM.md`
- `docs/decisions/PRODUCT_MECHANICS_015_UI_DESIGN_SYSTEM.md`
- `docs/contracts/UI_LAYOUT_SYSTEM_CONTRACT.md`
- North Star: `docs/decisions/PRODUCT_MECHANICS_016_PRODUCT_NORTH_STAR.md`

Any remaining reference to "Decision-013", "supervision-reflection", or the old
GUI spec set is obsolete; treat 014/015 + the UI layout contract as the authority.
