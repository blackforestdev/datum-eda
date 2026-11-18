# Product Mechanics 066: Shared engineering library

Status: owner-directed specification amendment, 2026-11-16; persistent database target corrected 2026-11-18; implementation pending.
Task: `dat-shared-library-spec-vvtz`.

## Disposition

The owner directs a unified, editable engineering library with standardized asset
provenance, reusable component definitions, exact revisions, open contributions,
evidence-based review, and validation connected to ERC/DRC and manufacturing.
GitHub is the initial proposed collaboration host. Specification completion and
independent review precede application integration and asset migration.

Owner-directed architecture correction (`dat-library-database-architecture-study-t9fj`):
the native library itself is persistent SQLite containing typed engineering JSON,
meaningful reusable geometry/process assets, exact relationships and revision
history. A SQL index over authoritative external JSON does not satisfy this target.
The owner authorized comprehensive investigation and peer-reviewed specification
delivery before product implementation. The normative implementation contract is
SHARED_LIBRARY_ASSET_SPEC section 9; its project-manifest-V3 and V2 interchange
targets explicitly supersede the earlier V1-only target restrictions, while
retaining legacy readers/profiles under their original contracts.

Native symbol/footprint editors use the shared engine/editor backbone. Footprints
compose reusable padstacks/geometry with owned pad identities and transforms;
Package owns physical body/terminals rather than competing land-pattern copper.
Library edits, publication and project adoption remain distinct. Project-retained
SQLite closure data and exact placed snapshots support offline use and coexisting
revision contexts. The canonical project journal alone commits active design
bindings; durable prepared SQL receipts do not independently adopt content.
Routine redraw uses resident prepared geometry without library SQL queries or
dependency re-resolution. No benchmark, numerical standards rule or delivered
runtime capability follows from these requirements.

The normative target contract is
[Shared Library Asset Specification](../../specs/SHARED_LIBRARY_ASSET_SPEC.md).
This is a specification amendment, not proof that the application implements it.
It grants no new third-party source acquisition/distribution, dependency, public
publication or bulk migration authority. Ordinary authoring, editing, inspection,
local copying/export and QA of Datum work product proceed under their existing
execution grant; the project manager selects the needed test inputs and copies.
Optional provenance declarations do not impose a per-asset clearance workflow.
Specific identified outside-material conditions retain their actual affected
scope. Preserve recorded source evidence and notices; this amendment supplies no
blanket conclusion about attribution from file formats or refactor counts.
Repository stewardship and public publishing policy remain decisions to settle
before enabling public publishing, without blocking local foundations.

## Supersession and retained foundation

This amendment supplements PM008/008A and supersedes conflicting *target*
interpretations of their library provenance, revision and binding clauses:

- A library is a unified engineering graph, not independently assigned symbol
  and footprint catalogs as the ordinary design workflow.
- A reusable component binds electrical definition, symbol, physical choice and
  gate-aware pin/pad mapping by exact asset revision. Definitions and geometry
  remain separately reusable; a monolithic duplicated component file is not the
  target.
- Reusable geometry and process-definition kinds in the new profile have explicit
  contracts; owned vertices/segments are not automatically independent library
  assets. Equal bytes can share storage without merging authorship/identity.
- An asset UUID, mutable pool path, local `object_revision`, Git commit or MPN
  alone is insufficient portable revision identity. The portable reference and
  dependency-closure contract in the specification governs exchange.
- Provenance is a common record for all asset kinds, not optional symbol-only
  fields. Unknown imported history remains honestly unknown.
- Direct raw-file/Git merging into a live project, automatic latest-wins updates,
  automatic approval from popularity or a vendor badge, and silent fallback from
  an invalid declared mapping are forbidden target behavior.
- Existing compatibility layouts and fallback mappings remain explicitly labeled
  migration inputs, not authority for new shared-library definitions. Their
  removal requires a separately scoped compatibility decision and proof.
- The earlier source/index-only storage destination, V1 eight-kind ceiling and
  one-active-revision-per-asset project limitation are superseded only for the
  new explicitly adopted profile. Legacy operation/history meanings remain intact.

Preserve PM001's single mutation path, one resolver-owned `DesignModel`, PM008's
object separation and Import Map identity, PM009's checks, PM010's evidence-based
standards posture, and PM061's effective process geometry. PM034 keeps Git an
optional transport; PM038 keeps formal project document control opt-in. Technical
asset history must not impose a governed-project workflow on ordinary designers.
PM062's deferred pool priority, placement gating and ranking choices are not
silently settled here. Existing review/AI approval guardrails remain controlling.

## Required product behavior

The Library Editor lets a user create, inspect, edit, derive, compare and validate
symbols, footprints, components and mappings together. Normal component search
selects engineering properties and an explicit supported physical implementation;
the user does not assemble Entity/Part/Map records manually. The same selected
component identity continues into PCB implementation. Manual and AI actions use
the same engine operations, review boundaries and journal.

History records engineering accountability, not personal search or usage habits.
Working asset content remains lean; supporting history/evidence is linked and
loaded when needed. Library checks feed revision-qualified design checks; blocked
manufacturing findings prevent CAM publication under the applicable existing
release policy. Standards basis, unknowns and legitimate deviations stay visible.

## Delivery and prevention of regression

Implementation tasks cite this decision and the specification, identify applicable
requirement IDs, and state which target behavior they deliver. Historical slices
keep their original scope and proof. Review rejects reintroduction of superseded
target behavior even if older code or examples still use it. Acquisition,
conversion and revision-aware bindings are proven before bulk migration.

Complete migration of the full current catalog is an explicit implementation
deliverable after proving slices; an empty lock, QA subset, index or metadata
editor cannot stand in for it. Retain/adapt accepted workflows or record an
explicit compatibility disposition; preserved code is not a mandatory porting
backlog. Disputed product expansions remain separately disposed, not architecture
prerequisites. Existing file layouts, metadata-only workflows and successful
tests do not justify deferring the approved database destination.

The first delivery scope and done statement require owner/authorized-PM scheduling
under PM059. This specification is not a successor selection or product PASS.
No new cryptographic provider, third-party code, standards content or signature
service is adopted by this decision.
