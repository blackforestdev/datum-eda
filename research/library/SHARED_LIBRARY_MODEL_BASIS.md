# Shared engineering library amendment basis

Date: 2026-11-16. Task: `dat-shared-library-spec-vvtz`.
Scope: owner-directed specification reconciliation; no runtime delivery claim.

## Owner requirements

The owner requests a unified editable library that reuses electrical symbols,
physical footprints and device-specific mappings; common asset identity,
authorship/origin/modification records and version history; vendor and community
contributions reviewed against factual evidence; GitHub as collaboration host;
checks connected to ERC/DRC and CAM publication; reliable fast searching without
individual browsing surveillance. Specification completion precedes integration.

The owner authorized specification changes through the active PM after review of
the isolated proposal. PM reserved decision 066; QA/spec transferred append-only
consumer sections and retained all preceding QA013 evidence. No statement here
ratifies a numerical standards requirement, source-content license or dependency.

## Existing product basis and delta

PM008/008A already define the reusable graph and pinned references. PM034 keeps
Git optional transport and Datum authority local; PM038 keeps formal governance
opt-in. PM009 owns check runs/findings. PM061 owns effective mask/paste geometry
and unknown-basis distinctions; PM062 preserves pending placement/pool choices.
The amendment links these authorities rather than replacing their settled rules.

Inspected implementation: `pool/mod.rs` defines distinct objects and a SQLite
PoolIndex; `api/mod.rs` initializes that index in memory; rebuild currently clears
and repopulates tables in `PoolIndex::rebuild_from_pool`; native pool changes use
`substrate/pool_journal_ops.rs` staging and journal machinery. These facts justify
explicit consistency/recovery requirements, not an assertion of corruption or
adequate large-library performance. `pool/symbol.rs` has optional source/review
fields while `pool/footprint.rs` lacks the same common provenance envelope.
The new common record and portable revision exchange are targets, not existing
features proven merely by local technical counters or source hashes.

## Reviewed baseline and boundaries

Reused prior library complete-route evidence in
`docs/reviews/workflow-delivery-rollout/library-surface-route-preflight.json` and
`surface-specification-combined-review.json`; current QA013 baseline is recorded
in its task and spec-reconciliation recovery. Amendment review must assess only
the new target clauses and affected claims while preserving original proof pins.
Existing accepted prototype states, historical authors and runtime readiness
remain unchanged. No prototype is edited or newly accepted by this amendment.

Remaining decision boundaries and exact implementation dependencies are stated in
the governed specification, not resolved by this evidence note. The numbered
decision and contract are the authority; this file records the basis only.

## Persistent database architecture correction — 2026-11-18

Task `dat-library-database-architecture-study-t9fj`. The owner explicitly rejects
SQLite as only a search index and directs comprehensive study followed by a
peer-reviewed implementation-ready specification. The native library must retain
the actual engineering JSON and composition relationships in SQLite. Symbols and
footprints need native editors; meaningful pad/shape reuse, standards-informed
construction, exact revisions/authorship and fast/light runtime behavior must be
designed together. Ordinary redraw must not query/recompose the library graph.
Earlier Cartesian/table/grouped-snapshot examples are exploratory, not ratification
of an arbitrary three-column table or pixel-raster authority.

### Actual Datum evidence

Production `Engine::new` constructs `PoolIndex::open_in_memory` (engine api/mod.rs);
pool/mod.rs indexes Parts/tags/parametrics. Existence of a file-backed constructor
does not establish a persistent engineering store. import/library_bundle.rs reads
embedded native symbol JSON by indexed byte range; library_source_catalog.rs
discovers it. Catalog counts of converted/refused symbols are not a full unified
Part/Footprint inventory or evidence of full migration.

pool/padstack.rs supports Circle/Rect/RoundRect apertures and process policies;
pool/process.rs explicitly refuses nonzero rounded process margins/scaling.
Package and Footprint both retain legacy pads/drawing fields. These structs are
implementation inputs, not the target geometry ceiling or competing copper
authority. library_assets records/slots/closure/lock already verify exact revisions,
but V1 closure rejects concurrent revisions of the same asset. Same-ID coexistence
requires contextual resolver/native identity changes, not merely a composite SQL
key.

substrate/component_pad_source stores frozen geometry/process/source bindings;
schematic_scene_import/native_library resolves at a cached document boundary.
gui-render session_sources, retained_scene_owner and world_bundles retain/share
scene data and GPU commands. This study does not find SQLite queries on every
redraw and supplies no measured speed/memory improvement. Existing project commit
stages journal/authority/shard/cursor publication with uncertain-outcome recovery;
a SQL transaction beside it cannot make those files atomically commit together.

### Primary-source comparison

- [Altium template definition](https://www.altium.com/documentation/altium-designer/components-libraries/pad-via-templates-libraries)
  separates base pad/via configuration from instance placement and supports use
  in footprints. [Template management](https://www.altium.com/documentation/altium-designer/pcb/pad-via-templates)
  distinguishes local retention, external links and explicit updates.
- [Altium custom pad stacks](https://www.altium.com/documentation/altium-designer/pcb/custom-pad-stack)
  supports contour/region construction and independent mask/paste shapes. Origin
  is an editing/routing anchor, not necessarily an asymmetric shape's centroid.
- [Altium Workspace model](https://docs.developer.altium.com/cdm/subsets/library/)
  distinguishes component, symbol and footprint revisions. [DbLib editing](https://www.altium.com/documentation/altium-designer/components-libraries/database-libraries/modifying-symbols-footprints)
  instead references library model files; its revision-comparison UI does not
  disclose internal SQL geometry tables.
- [Cadence Padstacks and Allegro SKILL, 17.2](https://community.cadence.com/cfs-file/__key/communityserver-discussions-components-files/137/MakePadstackInAllegroSkill.pdf)
  documents per-layer padstack data, regular/thermal/antipad roles, shape-symbol
  references and design-database-first loading before disk-library lookup.
  [Cadence OrCAD X overview](https://resources.academic.cadence.com/pcb-design-and-analysis-resources/2024-pcb-padstack-features-within-orcad-x)
  supplies a newer, separately scoped authoring/refresh example. Neither proves
  a proprietary SQL layout or Datum performance.

### Storage evidence and selected tradeoffs

[SQLite application uses](https://www.sqlite.org/whentouse.html) support embedded
application persistence. [Isolation](https://www.sqlite.org/isolation.html) and
[WAL](https://www.sqlite.org/wal.html) establish committed snapshots, single-writer
and local-host constraints; attached WAL databases are not atomically committed
as one set. [Foreign keys](https://www.sqlite.org/foreignkeys.html) require verified
connection enforcement. [Backup](https://www.sqlite.org/backup.html) provides a
coherent snapshot mechanism, not permission to copy an active main file alone.
[JSON](https://www.sqlite.org/json1.html) supports internal text/binary choices;
JSONB is not automatically constant-time or a portable interchange encoding.
[FTS](https://www.sqlite.org/fts5.html) and [expression indexes](https://www.sqlite.org/expridx.html)
are candidate derived search facilities requiring consistent maintenance;
[R*Tree](https://www.sqlite.org/rtree.html) is broad-phase indexing, not unrestricted
exact engineering coordinate storage. Current documentation can exceed the
repository's bundled SQLite version; supported features must be verified before
enabling a writer.

Read-only inspection of the cached libsqlite3-sys 0.30.1 sqlite3.h identifies
SQLite 3.46.0; this is a source fact, not an executed runtime probe. The official
[WAL-reset disclosure](https://www.sqlite.org/wal.html#the_wal_reset_bug) describes
an affected multi-connection write/checkpoint race. The initial target therefore
selects verified DELETE/FULL durability with foreign-key enforcement and bounded
transactions off the viewport thread. This trades WAL reader/writer overlap for
an explicit supported initial policy; no latency outcome is inferred. Future WAL
support requires a verified fixed build or a validated serialized write/checkpoint
configuration. No dependency update or new acquisition is authorized by this
specification work.

The selected target is relational identity/revision/relationship/query structures
plus bounded typed engineering JSON **inside SQLite**. Index-only external files
fail owner intent; one opaque component blob hides useful reuse; independently
versioning every coordinate adds unjustified coupling. Meaningful reusable
geometry/process/padstack assets and identified footprint-owned uses separate
engineering reuse from physical byte deduplication and runtime instancing.

Project-retained SQLite stores adopted exact content and immutable receipts.
Canonical project journal/revision authority commits active bindings. Durable SQL
preparation precedes receipted project publication, permitting explicit recovery
without a false cross-store atomicity guarantee. Exact occurrence contexts and
persisted local-ID projections support coexisting asset revisions. Resolved
engineering snapshots retain identities/mapping/process basis for offline checks,
CAM and render preparation; pixels/meshes are discardable derived data. This
solves library persistence without requiring a wholesale project-database rewrite.

Format-only adoption preserves existing V2-managed providers; per-occurrence
migration explicitly changes active source ownership and bindings with an exact
inverse. When remaining users need an old definition, keep its local ID/provider
and allocate a new local definition ID for the new revision. Neither an empty DB
nor pool priority decides ownership. A sealed V1 Part requires an explicit
validated successor/owned physical-choice conversion before ContextV2 use.

Earlier PARAMETRIC_FOOTPRINT_EDITOR_RESEARCH proposals for a ParametricPackage
pool kind and baked Package copper are superseded target sketches. The present
Package/body versus Footprint/land-pattern ownership and geometry/process kinds
are explicit in PM066/section 9. Historical DXF/export proposals supply no new
canonical payload, interoperability or dependency authority.

Typed shape/process/wire/operation/recovery contracts are normative in
SHARED_LIBRARY_ASSET_SPEC section 9, linked by PM066, PM008/008A, Native Format
and Library Authoring. Numerical performance/standards claims remain evidence-bound;
unsupported figures do not create a blanket implementation gate. Full migration
must account for unavailable/refused, legacy and unmanaged data with explicit
coverage/refusal, preserving semantics rather than claiming success from row counts.
Implementation must qualify actual shapes/consumers and recovery; this research
does not certify code or manufacture new source-rights obligations.
