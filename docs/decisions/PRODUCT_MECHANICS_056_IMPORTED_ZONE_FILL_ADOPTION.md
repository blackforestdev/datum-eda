# Product Mechanics 056: Saved copper-fill adoption

Status: owner-approved bounded repair under S5A-C03.
Issue: `dat-imported-zone-fill-authority-udb8`.

The owner reported missing filled copper on the imported DOA2526 board and
approved the concrete reconciliation/repair proposal with “proceed”. This
amendment adds native KiCad board adoption as an explicit generated-fill
producer; it does not grant a new native fill solver or manufacturing allowance.

## Source and generated copper

Preserve the authored Zone boundary and stable engine/import identity. Complete
saved filled polygons belong to revision-bound `ZoneFill` evidence, separately
from source. Multiple islands do not create competing authored Zone identities.
The inherited single-layer Zone representation retains one source per imported
layer; existing deterministic import/layer allocation remains engine-owned.
Compatibility-only scenes retain their recorded presentation behavior.

A saved fill is accepted only with validated layer/Net/source correspondence,
complete geometry, source hash and producer provenance. Exact source-vertex
decomposition of a bridged hole contour is permitted when certified integer
area and oriented boundary are preserved; no vertex rounding, filled hole,
outline-as-copper fallback, omitted island or new dependency is permitted.
Malformed or uncertifiable copper refuses atomically and identifies the source.
The inherited non-destructive reimport still records `MissingInSource` for
unchanged imported copper absent from the new source; it does not delete native
authored objects. Its incoming saved fill is explicitly Stale with a source
correspondence diagnostic and cannot become current/rendered occupied copper.
Changed or unrelated native source conflicts refuse atomic adoption.
Missing saved fill remains Unfilled; absence cannot assert successful empty fill.

## Canonical adoption and lifetime

Source and saved-fill changes use one engine-built, guarded, journaled import
batch. Bind generated evidence to the final native model and source-Zone
revisions, not the pre-import model. Reuse `SetZoneFill`, its inverse and existing
source operations. No CLI/GUI private writer and no selection-triggered refill.

A known old import that stored island geometry as source may migrate only with
matching import provenance and unchanged legacy payload. Retire only its proven
legacy per-island source objects; edited/ambiguous source refuses. Preserve the
source Zone's allocated identity.
The same source-correspondence check may adopt the existing importer's explicit
pad `layer_connection` fact omitted by an old converter, using `SetBoardPad` only
when source hash/path and untouched import ownership match and every other pad
field is byte-identical. Preserve pad identity and geometry; edited pads refuse.
This reuses inherited pad semantics and adds no pad model or geometric mechanism. Native edits invalidate the fill through the
existing revision rules; repeat launch/import cannot silently revive Stale fill.
Undo/replay/reopen preserve the atomic source/evidence result. Compensation may
rebind restored imported geometry to its actual restored source basis; it cannot
invent geometry, treat an unrelated cache as current, or resurrect dissolved
selection. Rendering, inspection, connectivity and nominal DRC consume the same
current evidence and retain PM026/054 lifetime and non-mutation rules.

## Scope and proof

Implement only this saved-fill adoption gap under the retained S5A-C03 grant.
Existing native solver limitations remain explicit. The imported copper result
is not independent clearance verification, a complete import qualification,
CAM/T04 closure, foundation reacceptance or S5A native acceptance.

Focused proof: complete multi-island source membership; exact hole/area oracles;
source identity and old-import migration; absent/malformed/stale refusal;
canonical atomicity, unchanged reimport, undo/redo and reopen; actual DOA2526
saved copper through the shared native renderer. Preserve PM049/051/052,
C05 independent review, source-authority blockers and deferred editors.
