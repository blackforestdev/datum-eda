# PLM & Lifecycle Integration — Industry Survey & Datum EDA Implementation Strategy

> Phase 2 deep-dive on Domain 7 of the 8-domain standards audit.
> Continues from `research/standards-audit/STANDARDS_AUDIT.md § 7`.
> **Consolidates cross-domain handoffs** from Domains 2 (Octopart MCP
> design deferred here), 4 (AS9102 FAI consumes variant substrate;
> `data_egress_policy` gates supply-chain calls; ITAR PLM hosting),
> 5 (consolidated `refresh_supply_chain` field map ratified here),
> 6 (PHY-IC vendor lookup is a PLM concern; rule packs are pool content
> with vault implications).
> Reads against post-Standards-Audit-Batch-1 spec baseline (PR #1
> merged 2026-04-18); Batches 3/4/5 (Domains 4/5/6) edits are in
> flight per project owner's integration cadence.

> **Pending Exclusions Policy (verbatim from STANDARDS_AUDIT.md § Phase 2 Triage Recommendations).**
>
> > The audit's "Recommended low-priority / skip" list is an **advisory
> > exclusion** for Phase 2 work. Phase 2 agents MUST NOT re-investigate
> > these standards. Final ratification of skips into binding scope
> > documents happens in a single consolidated pass after Domain 8 lands,
> > when full cross-domain context is available.
>
> For Domain 7 specifically the advisory exclusion list contains:
> Windchill / Teamcenter / Aras / Arena / OpenBOM PLM connectors
> (connector work, not Datum-engine work; "if a customer demands it,
> build per-customer; do not build pre-emptively") and IHS Markit
> Engineering Workbench / SiliconExpert (paid commercial catalogs,
> already deferred to Domain 2 advisory exclusion).
>
> Domain 7 surveys these PLM platforms at the **abstraction level**
> only — what generic vault / lifecycle-event / document-reference
> hooks Datum needs so that any future connector binds to ready
> data shapes — and does NOT specify per-vendor connector code.
> Per project owner standing instruction, Claude Code is in
> research-only mode (memory rule `feedback_research_only_mode`):
> recommendations only, no spec edits, no commits, no PRs.

## Executive Summary

- Datum cannot be a PLM, and should not try. Datum is the **library
  substrate** that PLM systems wrap around. Domain 7's job is to
  specify the **abstract interface** (vault-API surface, lifecycle-event
  feed, document-reference resolution chain, ECO grouping shape) so
  any future connector — Windchill, Teamcenter, Aras, Arena, OpenBOM,
  custom-internal — has data hooks to bind to. This is the same
  substrate-not-certifier framing Domains 4 (compliance) and 6 (EMC/SI)
  arrived at independently.
- Datum's existing **pool layering is already a PLM-substrate
  primitive**: project-local + shared (org) + shipped (base) is the
  same three-tier authoritative-source pattern that OrCAD CIS, Altium
  DBLib, and KiCad ODBC libraries surface. Domain 7 formalises
  pool layering as the CIS / library-distribution contract and
  extends it with **library object lifecycle states** (Draft → InReview
  → Approved → Released → Deprecated → Obsolete) parallel to the
  existing per-Part `Lifecycle` enum.
- The most-asked-for cross-domain coordination is the **consolidated
  `refresh_supply_chain(part_uuid)` field map**. Domains 2, 5, and 6
  each touched part of it; Domain 7 ratifies the contract. A single
  call atomically populates **Domain 2** behavioural-model URLs and
  thermal hints, **Domain 5** RoHS / REACH SVHC / China RoHS / J-MOSS /
  halogen-free / SCIP fields, **Domain 6** packaging fields and
  PHY-IC parametric anchors, and **Domain 7** `supply_chain_offers`,
  `lifecycle`, `last_supply_chain_check`, plus alternates and
  supersede-chain candidates. Partial failures are explicit and
  transaction-logged as one event.
- The **`data_egress_policy` gate** (introduced in Domain 4 for
  ITAR / EAR / EU dual-use posture) terminates at Domain 7's
  supply-chain integration layer. Every external-network MCP
  tool — `lookup_part_octopart`, `lookup_part_digikey`,
  `lookup_part_mouser`, `refresh_supply_chain`,
  `find_alternate_parts`, plus future PLM connectors and any
  AI-routed tool — must consult `data_egress_policy` before
  execution and refuse-or-warn on `NoExternalNetwork` /
  `NoExternalAi` projects.
- The **`DocumentRef.uri` resolution scheme** (introduced in
  Domain 5 for vendor-cert / IPC-1752A declaration evidence) is a
  Domain 7 deliverable. Resolution chain: **local file → project
  pool → shared pool → external PLM (CMIS-ish) → cached
  fetch**. Resolution is deterministic, cached, and the cache
  hit/miss is auditable. CMIS (OASIS Content Management
  Interoperability Services) is the natural contract surface for
  the external-PLM step.
- Datum's **transaction model is the substrate for ECO**
  (Engineering Change Order) workflow. Every authored op is already
  logged. The missing piece is the **ECO grouping layer**: a
  `EngineeringChangeOrder` aggregate that bundles N transactions
  with rationale, approver, effective date, and disposition.
  Recommend a `eco_workflow_required: bool` on `Project.compliance`
  to gate the workflow.
- The **CIS pattern is Altium DBLib / OrCAD CIS / KiCad-7+ ODBC
  libraries**. All three are ODBC-driven external part-source
  patterns. Datum's current pool serves the same role
  natively (JSON + SQLite index); a future "external CIS bridge"
  is `Out of scope` — building per-customer if demanded — but the
  pool's `Part.parametric` shape already satisfies the ODBC-attribute
  contract.
- **AS9102 First Article Inspection** is the single biggest
  industry-vertical PLM consumer of Datum's variant substrate. AS9102
  Forms 1/2/3 require BOM-with-rev, design-rev evidence,
  variant-selection record, ECO history, and signed-off approval —
  every one of these maps onto Datum primitives already present
  (`Part.uuid`, `SheetFrame.revision`, `Variant.fitted_components`,
  `Transaction.id`). Domain 7 specifies the **AS9102 evidence package
  contract**; Domain 8 will specify the signing layer (electronic
  signature substrate) on top.
- **Supersede chains** (Part B replaces EOL Part A) need first-class
  representation. Today `Lifecycle::Eol` is a flag with no pointer
  to the recommended replacement. Recommend `Part.supersede:
  Option<SupersedeRef>` with `SupersedeRef { successor_part_uuid,
  reason, recommendation_source, recommended_at }` plus an MCP
  `follow_supersede_chain(part_uuid)` tool that walks the chain
  to the latest still-Active part. **AI-narrated supersede
  navigation is a Datum differentiator** — the AI surface can
  explain "Part C42 is EOL; recommended replacement is Part X
  (3 generations down the chain), pin-compatible, +5 % cost,
  same package".
- **Where-used queries** ("given this Part UUID, list all Boards /
  Schematics that reference it") are an industry PLM table-stake
  Datum can deliver as **a one-call MCP query** because the
  canonical IR already indexes every Part reference. Recommend
  `where_used(part_uuid)` returning `{ schematics: [...], boards:
  [...], variants: [...] }` plus a project-scope filter.
- **Library object lifecycle** (Draft / InReview / Approved /
  Released / Deprecated / Obsolete) parallels the per-Part
  `Lifecycle` but applies to symbols, packages, padstacks, and
  parts as **library artefacts**. Recommend `LibraryObjectStatus`
  enum on each pool object plus a separate
  `library_audit_log` Vec keyed off the pool object UUID. This
  is the prerequisite for any approval-workflow overlay; it is
  also the data hook a future Windchill / Teamcenter connector
  would bind to.
- **Identity / authentication** for PLM context: **OAuth 2.1 +
  OIDC 1.0** is the modern pattern for Octopart, Digi-Key,
  Mouser, and most cloud PLM (Arena, OpenBOM); **SAML 2.0** is
  the enterprise-SSO pattern for Windchill / Teamcenter; **PKCS#11
  v3.0** is the hardware-token interface for electronic-signature
  workflows (cross-ref Domain 8). Datum needs an OAuth/OIDC client
  in the supply-chain MCP layer; SAML and PKCS#11 are Domain 8
  prerequisites and are deferred-with-prerequisite here.
- The Phase 1 audit found **2 standards Implemented** (multi-pool
  layering, per-Part lifecycle metadata) and **12 blind spots**.
  This deep-dive resolves them as: **5 Planned** (consolidated
  refresh contract, library object lifecycle, supersede chains,
  where-used queries, OAuth-2.1 client substrate), **3 Deferred
  with prerequisite** (AS9102 evidence package — depends on
  Domain 8 signature; library approval workflow — depends on
  audit-log surface; CMIS document-resolution external step —
  depends on PLM-connector partner), **3 Out of scope** (Windchill,
  Teamcenter, Aras, Arena, OpenBOM connectors — per-customer
  build only; SiliconExpert and IHS Markit Engineering Workbench
  — paid catalogs already deferred to Domain 2; CMII methodology
  enforcement — methodology, not toolable), **1 Reference-only**
  (OrCAD CIS pattern — Datum's pool is the moral equivalent;
  no need to bridge to OrCAD's database). The 11 audit-list rows
  collapse into 12 disposition entries because supersede chains
  and library object lifecycle were not on the original Phase 1
  list — they emerged from the cross-domain consolidation work.
- **Datum differentiators**: AI-explained EOL impact analysis
  ("which BOM lines are EOL with no supersede?"), MCP-queryable
  AVL ("which approved vendors are still active?"),
  deterministic ECO replay ("re-apply ECO #42 onto rev B"),
  where-used-as-one-call (vs Altium's multi-step Vault query),
  AI-narrated supersede-chain navigation, and AI-narrated
  variant diff between BOM revisions. Each is one-shot via MCP;
  no incumbent CIS makes any of these conversational.

## The Substrate-Not-Connector Framing

Domain 7's central position in the eight-domain audit is
**substrate-not-certifier**, exactly parallel to Domains 4 (compliance
posture, not certifying authority) and 6 (rule-pack substrate, not
silicon-vendor surrogate). The framing has three load-bearing
consequences:

1. **Datum is not a PLM and never becomes one.** PLM systems
   (Windchill, Teamcenter, Aras, Arena, OpenBOM) own multi-domain
   workflows: bill-of-material mastery, manufacturing routing,
   change-control approval, supplier scorecards, document
   distribution, regulatory filings. None of those belong in an
   EDA tool's spec. Datum's job is to be the **canonical, vendor-
   neutral source of design truth** that any PLM can wrap around.
   The wrapper is the connector; the connector is per-customer
   work; Datum's job is to expose stable data hooks the wrapper
   binds to.

2. **The abstract interface is the deliverable, not the wrapper.**
   Three hook surfaces matter:
   - **Vault-API surface** — read/write of pool objects with
     check-out semantics (optimistic by default; pessimistic
     check-out as opt-in for shared pools).
   - **Lifecycle-event feed** — every library object lifecycle
     transition (Draft → InReview → Approved → Released →
     Deprecated → Obsolete) and every Part lifecycle transition
     (Active → NRND → EOL → Obsolete) is emitted to a queryable
     event log. PLM connectors poll or subscribe; Datum does not
     depend on the PLM being live.
   - **DocumentRef resolution chain** — `DocumentRef.uri` is
     resolved through a deterministic ordered fallback that
     terminates at the local filesystem, the project pool, the
     shared pool, the external PLM (CMIS-shaped), or a cached
     fetch. The resolution is the contract; the external-PLM
     step is pluggable.

3. **Pool layering is the PLM substrate already.** Datum's
   existing pool layering (project-local + shared + shipped) is
   not a workaround for missing PLM features; it is the
   architecture-correct way to express the CIS pattern in a
   git-native, file-based EDA tool. OrCAD CIS, Altium DBLib, and
   KiCad-7+ ODBC libraries all express the same three-tier
   authoritative-source pattern via ODBC bridges to MS-SQL or
   ERP/PLM databases. Datum expresses it through pool priority
   ordering with deterministic UUID identity. The data shape is
   the same; Datum is not missing a feature, it is implementing
   the feature differently.

The substrate-vs-connector dividing line is sharp: anything that
specifies **what data Datum holds, how it's identified, how it
transitions state, and how external systems read/write it** is in
Datum's scope. Anything that specifies **the per-vendor wire
protocol, authentication idiosyncrasies, business-process
workflows, or paid-catalog ingestion** is connector work and is
out of Datum's scope unless a paying customer commissions it.

## Standards Catalog

### Library Lifecycle & Vault Semantics

#### Component lifecycle states

**Current Datum.** `Lifecycle::{Active, Nrnd, Eol, Obsolete,
Unknown}` (`specs/ENGINE_SPEC.md:50-56`); `Part.lifecycle` field
(`specs/ENGINE_SPEC.md:257`). The five-state enum mirrors industry
practice (Active = in production, NRND = Not Recommended for
New Designs, EOL = End of Life, Obsolete = no longer purchasable).

**Industry practice.** The five-state model is widely deployed:
- **TI Product-Status Codes** — `ACT` (Active), `NRND` (Not
  Recommended for New Designs), `LO` (Last-Time Buy / Last-Order),
  `OBSOLETE`, `PREVIEW`. The `LO` state (Last-Time Buy / final
  order window) is the one industry-distinct state Datum
  currently flattens into `Eol`.
- **Microchip Status Codes** — `ACTIVE`, `NRND`, `OBSOLETE`,
  `EOL`, `PROMOTIONAL` (preview).
- **Maxim / Analog Devices** — `Production`, `Last-Time Buy`,
  `Obsolete`, `Pre-Release`.
- **Octopart / Nexar `lifecycle` field** — normalised to
  `Active`, `NRND`, `Obsolete`, `Discontinued`, `Pre-Release`.

The two industry distinctions Datum's current enum collapses are
**Last-Time Buy** (a finite window during which final orders are
accepted; clinically distinct from EOL because the part is still
purchasable but on a deadline) and **Pre-Release / Preview**
(silicon sampling, not yet in production). Recommend extending
the enum:

```rust
pub enum Lifecycle {
    Preview,     // pre-production sampling
    Active,
    Nrnd,
    LastTimeBuy, // finite final-order window — distinct from Eol
    Eol,
    Obsolete,
    Unknown,
}
```

Plus a `last_time_buy_deadline: Option<DateTime<Utc>>` on `Part`
for the LTB cutoff date when `Lifecycle::LastTimeBuy` is set.
Octopart / Digi-Key / Mouser all surface this date; Datum should
ingest it during `refresh_supply_chain`.

**Disposition.** `Implemented` (current five-state). Recommend
`Planned` extension to seven-state.

#### Library object lifecycle

**Industry practice.** Distinct from per-Part lifecycle, library
**objects** (symbols, packages, padstacks, parts as library
artefacts) carry their own approval/release lifecycle. The
canonical six-state model — drawn from Windchill, Teamcenter, and
Aras CMII-aligned conventions — is:

- **Draft** — under active development; not visible to projects
- **InReview** — submitted for review; visible but not pickable
- **Approved** — passed review; pickable but not yet released
- **Released** — production-ready; default pick for new designs
- **Deprecated** — discouraged for new designs; existing usage
  unaffected
- **Obsolete** — withdrawn; existing usage flagged

Library object lifecycle is **orthogonal to per-Part lifecycle**.
A `Part` representing a TI op-amp can be `Released` as a library
object (the symbol/package/binding is approved) while
simultaneously `Lifecycle::Eol` as a manufacturer status (TI is
sunsetting the silicon).

**Datum spec gap.** No spec mentions library object lifecycle
state. The Part record (`specs/ENGINE_SPEC.md:242-263`) carries
manufacturer lifecycle but no library-artefact lifecycle.

**Recommendation.** Add `LibraryObjectStatus` enum and apply it
to Symbol, Package, Padstack, Part, and Unit:

```rust
pub enum LibraryObjectStatus {
    Draft,
    InReview,
    Approved,
    Released,
    Deprecated,
    Obsolete,
}

// Applied to Unit, Symbol, Entity, Package, Padstack, Part
// as a separate field; default Released for current pool content.
```

**Disposition.** `Planned` — substrate. Approval-workflow overlay
that consumes the state is `Deferred with prerequisite` until
Domain 8 audit-log surface lands.

#### Check-in / check-out semantics

**Industry practice.** Two patterns:
- **Pessimistic locking** — Windchill, Teamcenter, Altium Vault.
  User explicitly checks out an object; nobody else can edit
  until checked back in. Strong consistency; weak concurrency.
- **Optimistic locking** — git, Aras Innovator (modern mode),
  KiCad ODBC libraries. No explicit lock; conflicts surface at
  commit/merge time. Strong concurrency; eventual consistency.

Datum's pool today is **implicitly optimistic** because pool
files are git-versioned JSON. Two designers editing the same
Part file conflict at git-merge time, not at edit time.

**Recommendation.** Keep optimistic by default for project-local
pools (preserves git-native workflow). Add **opt-in pessimistic
check-out for shared pools** via an advisory lock file
(`pool/locks/<object-uuid>.lock` containing actor identity and
acquired-at timestamp). The lock is purely advisory — Datum
warns on edit, does not refuse — because the underlying file
is still git-tracked and the user may have a legitimate reason
to override. Document the advisory-lock semantics in
`POOL_ARCHITECTURE.md`.

```rust
pub struct PoolLock {
    pub object_uuid: Uuid,
    pub object_kind: PoolObjectKind,        // Unit, Symbol, Package, Padstack, Part
    pub acquired_by: String,                // user identity (Domain 8 prerequisite)
    pub acquired_at: DateTime<Utc>,
    pub purpose: Option<String>,            // free-text rationale
}
```

**Disposition.** `Planned` (advisory lock); `Deferred with
prerequisite` (pessimistic vault-style check-out — needs Domain 8
user identity).

#### Supersede chains

**Industry practice.** When Part B replaces EOL Part A,
manufacturers publish a recommended-replacement
(supersede / cross-reference) record. Octopart / Digi-Key /
Mouser carry these as `replacement_mpn` or `compatible_with`
fields. PLM systems (Windchill PartsLink, Aras Component
Engineering) maintain explicit supersede graphs the user can
navigate. Some chains are deep (`A → B → C → D`); naive
single-step lookup misses the latest replacement.

**Datum spec gap.** No supersede record. `Lifecycle::Eol` is a
flag with no pointer to the replacement.

**Recommendation.** Add `Part.supersede: Option<SupersedeRef>`:

```rust
pub struct SupersedeRef {
    pub successor_part_uuid: Uuid,                // → Part (the recommended replacement)
    pub reason: SupersedeReason,
    pub recommendation_source: SupersedeSource,
    pub recommended_at: DateTime<Utc>,
}

pub enum SupersedeReason {
    Eol,
    NrndAlternate,
    PinCompatibleUpgrade,
    PackageChangeOnly,
    ManufacturerDiscontinuation,
    Other(String),
}

pub enum SupersedeSource {
    Manufacturer,                                 // vendor cross-reference
    Distributor(String),                          // "Octopart" / "DigiKey" / "Mouser"
    AuthoredByUser,                               // librarian-asserted
    AiSuggested,                                  // AI-recommended; needs review
}
```

Plus an MCP tool `follow_supersede_chain(part_uuid)` that walks
the chain to the latest still-`Active` part:

```
Method: follow_supersede_chain
Input:  { "part_uuid": uuid, "max_hops": int }     // default 10
Output: { "chain": [{ "part_uuid": uuid, "lifecycle": string,
                      "reason": string, "source": string }],
          "terminal_status": string,                // "ActiveFound" | "ChainEndedAtEol" | "MaxHopsReached"
          "recommended_replacement": uuid | null }
```

**Disposition.** `Planned` — substrate.

#### Branch / fork / merge semantics

**Industry practice.** Branching libraries is an emerging PLM
pattern: variant libraries (Aras Variant Management), project-
local pool overrides (Altium Designer's "project pool"),
shipped-base + customer-delta libraries (Cadence Allegro
"site library" + project library). The merge semantics back
upstream are universally hand-wavy in commercial tools — typical
practice is "submit a request, librarian merges manually".

**Datum positioning.** Datum's pool layering already supports
branching (project-local pool overrides shared pool overrides
shipped pool by priority ordering). Merge back upstream is a
**git workflow** — the user copies the project-local pool object
into the shared pool repository and submits a pull request. This
is fine.

**Recommendation.** Document the workflow in
`POOL_ARCHITECTURE.md` as the canonical merge-back-upstream
pattern; do not invent a parallel mechanism. **Datum's git-native
posture is a differentiator**: incumbent CIS / vault systems
require special tooling to merge library deltas; Datum just uses
git.

**Disposition.** `Implemented` (via existing pool layering + git);
add documentation only.

#### Approval workflows

**Industry practice.** Universally pluggable: designer → reviewer
→ librarian → released, with configurable per-pool / per-object-
class workflow definitions. Windchill ships a workflow engine;
Teamcenter ships a workflow engine; Aras ships a workflow engine.
None of these are toolable inside an EDA core engine; they live
in the PLM layer.

**Datum positioning.** **Approval workflow is connector work,
not engine work.** Datum's responsibility ends at exposing the
state-transition primitives (`LibraryObjectStatus` transitions
above) and emitting the lifecycle-event feed. Workflow engines
plug in via the event feed and the per-state-transition MCP
tools.

**Recommendation.** Spec the **state-transition MCP tools**:
- `submit_library_object_for_review(object_uuid)` — Draft → InReview
- `approve_library_object(object_uuid, approver_id, rationale)`
  — InReview → Approved
- `release_library_object(object_uuid)` — Approved → Released
- `deprecate_library_object(object_uuid, rationale)` — Released
  → Deprecated
- `obsolete_library_object(object_uuid)` — Deprecated → Obsolete

Each is one transaction; each writes a `library_audit_log`
entry; each emits a lifecycle event. **The workflow that decides
who can call which transition lives in the PLM connector, not in
Datum.**

**Disposition.** `Planned` (state-transition primitives).
Workflow-engine integration is `Out of scope`.

#### Library audit trail (cross-ref Domain 8)

**Industry practice.** ISO 9001 § 7.5 (documented information
control), ISO 13485 § 4.2.5 (design-control records), 21 CFR
Part 820.30 (design history file), AS9100D § 8.5.6 (control of
changes), and IATF 16949 § 8.3.6 (engineering changes) all
require a tamper-evident library-change audit trail with actor
identity, timestamp, before/after state, and rationale. The
audit-trail format is not standardised; the **content
requirements** are.

**Datum positioning.** The transaction model
(`specs/ENGINE_SPEC.md:863-870`) already captures the
substrate. The missing pieces are:
- **Acting user identity** on `Transaction` (currently absent;
  Domain 8 prerequisite)
- **Timestamp** on `Transaction` (currently absent in the struct;
  is captured at Operation log time but not first-class on
  Transaction)
- **Rationale** beyond the free-text `description` (currently
  unstructured)
- **Per-pool-object audit log** keyed off pool UUID rather than
  flat operation log

**Recommendation.** Defer the **audit-log export surface** to
Domain 8 (the Phase 1 audit already routed it there). Domain 7
specifies what data feeds the surface for **library objects**
specifically:

```rust
pub struct LibraryAuditEntry {
    pub object_uuid: Uuid,                        // Pool object affected
    pub object_kind: PoolObjectKind,
    pub transaction_id: Uuid,                     // → Transaction
    pub event: LibraryAuditEvent,
    pub at: DateTime<Utc>,
    pub actor: Option<String>,                    // user identity (Domain 8)
}

pub enum LibraryAuditEvent {
    Created,
    Modified,
    StatusChanged { from: LibraryObjectStatus, to: LibraryObjectStatus },
    Deprecated { reason: String },
    Superseded { successor: Uuid },
    Approved { approver: String, rationale: Option<String> },
    Released,
    Withdrawn { reason: String },
}
```

**Disposition.** `Planned` (data shape); `Deferred with
prerequisite` (export surface — Domain 8).

### CIS (Component Information System) Patterns

The CIS pattern is consistent across vendors: **external
authoritative source-of-truth for component data; EDA tool stays
in sync via refresh / pull**. ODBC is the dominant wire protocol
because it predates REST and works against any SQL database
(Oracle, MS-SQL, PostgreSQL, MySQL). Modern variants (PartQuest,
Altium Manufacturer Parts Search) layer REST/GraphQL on top of
the same conceptual model.

#### OrCAD CIS

**Full title.** Component Information System (CIS), part of
Cadence OrCAD Capture. Documented in *Cadence OrCAD Capture and
CIS User Guide*, current revision 22.1 (2024).

**Pattern.** ODBC-driven external part database. Per-component
properties (MPN, manufacturer, package, value, datasheet URL,
custom fields) live in MS-SQL / Oracle / Access. Schematic
symbol carries a `Part Number` field; Capture queries the
database at design time and pulls properties.

**Adoption.** Mainstream in Cadence shops; not portable to
non-Cadence tools. The pattern itself (external DB → EDA tool
property-fetch) is the universal CIS shape.

**License / IP.** OrCAD CIS is a paid Capture license tier;
the database schema is user-defined.

**Datum positioning.** Datum's pool **is the database** in the
OrCAD-CIS sense. Pool JSON files + SQLite index serve the same
authoritative-source role; deterministic UUIDs + parametric
search serve the same query role. The **architecture difference**:
OrCAD CIS expects the database to be external (DBA-managed);
Datum's pool is git-native and self-contained.

**Disposition.** **`Reference-only`** — Datum's pool is the
moral equivalent. Bridging to an OrCAD-CIS-style external
database is `Out of scope` unless commissioned by a paying
customer; if commissioned, a per-customer connector reads from
ODBC and emits Datum pool JSON. The connector pattern is
identical to a Windchill connector; both are
read-from-external-system / write-to-Datum-pool flows.

#### Altium DBLib / SVNDBLib

**Full title.** Altium Database Library (`.DbLib`) and
SVN-Versioned Database Library (`.SVNDbLib`). Documented in
Altium Designer documentation, current revision Altium Designer
24 (2024).

**Pattern.** ODBC-driven database library — same CIS pattern
as OrCAD. SVNDBLib adds Subversion versioning of the schema and
parametric content. Altium Vault (now Altium 365 Workspace)
extends this with a hosted SaaS variant.

**Adoption.** Altium-shop standard. SVNDBLib is the older
on-prem pattern; Altium 365 Workspace is the cloud-hosted
modern pattern.

**License / IP.** Altium Designer paid license; ODBC drivers
are vendor-supplied.

**Datum positioning.** Same as OrCAD CIS — pool layering is
the moral equivalent.

**Disposition.** **`Reference-only`** for the pattern; bridge
connector is `Out of scope`. Note: Altium 365 Workspace
hosting is **incompatible with `data_egress_policy:
NoExternalNetwork` / `NoExternalAi`** — projects under those
policies cannot use cloud library hosting. This is a Domain 4
constraint; Domain 7 surfaces it at the integration layer.

#### KiCad database libraries

**Full title.** KiCad Database Libraries, introduced in
**KiCad 7.0 (February 2023)**, expanded in KiCad 8.0
(February 2024) and KiCad 9.0 (February 2025).

**Pattern.** ODBC connection to external database (MS-SQL,
PostgreSQL, MySQL, SQLite). Database row → KiCad symbol/footprint
binding via configurable column mapping. Open-source ODBC
drivers (unixODBC).

**Adoption.** Growing in KiCad community; user-base reports
mixed feelings — powerful but operationally heavy for
small teams.

**License / IP.** KiCad is GPL-3.0; ODBC drivers vary
(unixODBC is LGPL).

**Datum positioning.** A KiCad-database-library importer would
read the ODBC table at import time and emit Datum pool JSON.
This is feasible but only valuable for KiCad shops with
established CIS infrastructure. **Defer to per-customer
demand.**

**Disposition.** `Out of scope` for pre-emptive build;
**`Planned`** as a per-customer-on-demand connector recipe
(read ODBC table → emit Datum pool JSON; identical pattern to
the existing KiCad library importer).

#### Mentor PADS Library Manager

**Full title.** PADS Library Manager (Siemens PADS Professional /
Standard Plus). Documented in Siemens PADS Professional
documentation, current revision PADS VX.2.13 (2024).

**Pattern.** Centrally-managed proprietary parts library with
SVN-style versioning. Bridges to Teamcenter / Windchill via
Siemens-specific connectors.

**Adoption.** PADS-shop standard. Less common than Altium /
OrCAD in 2026 (PADS market share has shrunk).

**Datum positioning.** Pattern equivalent to OrCAD/Altium;
no separate Datum integration needed.

**Disposition.** `Reference-only` for the pattern.

#### PartQuest / PartQuest Xpress

**Full title.** PartQuest (Siemens, free tier and paid tier)
and PartQuest Xpress (cloud-hosted CIS, paid). Originally
Mentor Graphics; now Siemens EDA. Current platform 2024.

**Pattern.** Cloud-hosted authoritative parts database with
vendor-curated parametric data, IBIS/SPICE/STEP models, and
distributor offers. EDA tool plug-ins (PADS, Xpedition) query
PartQuest at design time.

**Adoption.** Siemens-ecosystem standard; modest direct adoption
outside Siemens tools.

**Datum positioning.** PartQuest exposes a REST API; an MCP
tool `lookup_part_partquest` is feasible. **Same shape as
`lookup_part_octopart`**; not a strategic add until a paying
customer demands it.

**Disposition.** `Out of scope` for pre-emptive build.

#### Altium Manufacturer Parts Search

**Full title.** Altium Manufacturer Parts Search (formerly
Altium Live, integrating Octopart). Bundled with Altium
Designer 21+.

**Pattern.** Distributor-data lookup integrated into Altium
Designer; reads Octopart/Nexar (Octopart is owned by Altium).

**Adoption.** Altium-only.

**Datum positioning.** Equivalent to Datum's planned
`lookup_part_octopart` MCP tool. **Datum has the underlying
capability via Octopart/Nexar API directly**; no Altium
intermediation needed.

**Disposition.** `Reference-only` for the pattern;
underlying Octopart/Nexar is the actual integration target,
already specified in Domain 2 / Batch 1.

### Supply-Chain Attachment

#### Octopart / Nexar (cross-ref Domain 2; consolidated map owned here)

**Cross-reference.** Octopart / Nexar full standard write-up
lives in `research/component-modeling/COMPONENT_MODELING_RESEARCH.md`
§ "Octopart / Nexar API". This deep-dive does not re-investigate
the API itself.

**Domain 7 ownership.** The **consolidated `refresh_supply_chain`
field map** (specified below in § "Cross-Domain Consolidation")
is the Domain 7 deliverable. A single
`refresh_supply_chain(part_uuid)` call atomically populates:
- Domain 7 fields: `supply_chain_offers`, `lifecycle`,
  `last_supply_chain_check`, `last_time_buy_deadline`,
  `supersede` (when supersede metadata is available),
  `orderable_mpns`
- Domain 5 fields: `compliance.rohs_status`,
  `compliance.reach_svhc`, `compliance.china_rohs`,
  `compliance.j_moss`, `compliance.halogen_free`,
  `compliance.scip_id`, `compliance.last_compliance_check`
- Domain 2 fields: `behavioural_models` provenance refresh
  (when distributor exposes IBIS / SPICE / Touchstone URLs);
  `thermal` refresh (Octopart's `thermal_resistance_*`
  parametrics; rare but non-zero)
- Domain 6 fields: `packaging_options` (EIA-481 reel/tape/tray)
  cache update
- Domain 7 alternates: `find_alternate_parts` candidate list

**Disposition.** `Planned` — already in Batch 1 spec; the
**field-map ratification** in this report is the consolidation
work three Phase-2 reports each touched part of.

#### Digi-Key / Mouser / Arrow / Avnet (cross-ref Domain 2)

**Cross-reference.** Per-distributor MCP tools
(`lookup_part_digikey`, `lookup_part_mouser`) are already
specified in Batch 1. Arrow and Avnet have public REST APIs
(Arrow Electronics Developer Portal, Avnet Cloud Connect) but
modest community adoption; defer to per-customer demand.

**Disposition.** `Planned` (Digi-Key, Mouser per Batch 1);
`Out of scope` for Arrow/Avnet pre-emptive build.

#### Newark/Element14 / LCSC

- **Newark / Element14 API** — Newark Electronics part of
  Avnet; REST API at api.element14.com. Modest adoption;
  defer.
- **LCSC API** — JLCPCB ecosystem; LCSC is JLCPCB's stocking
  distributor and exposes a REST API critical for users
  ordering through JLCPCB's combined PCB-fab + assembly
  service. **High value for the Datum-target hobbyist /
  prosumer / startup user.**

**Recommendation for LCSC.** Add `lookup_part_lcsc` MCP tool
in a follow-on batch (post-M7). The LCSC catalog has growing
strategic importance as JLCPCB has become the price-leader
PCB fabricator for low-volume work; surfacing LCSC stock /
price / extended-parts-library-availability into Datum's
supply-chain refresh closes the loop for the JLCPCB workflow.

**Disposition.** `Planned` (LCSC, in a follow-on batch);
`Out of scope` for Newark/Element14.

#### CMRT / EMRT attestation evidence (cross-ref Domain 5)

- **CMRT** — Conflict Minerals Reporting Template (RMI / RBA),
  current rev **CMRT 6.32 (December 2024)**. Excel-based
  template that suppliers fill in to attest 3TG sourcing.
- **EMRT** — Extended Minerals Reporting Template (RMI), current
  rev **EMRT 1.4 (October 2024)**. Adds cobalt and mica
  tracking to CMRT.

**Cross-reference.** Domain 5 specifies the per-Part
`compliance.conflict_minerals_status` field. Domain 7's
addition: **the CMRT / EMRT Excel attachment lives in the
pool as a `DocumentRef`**, identified by the supplier UUID
and a `received_at` timestamp. The DocumentRef resolves
through the `DocumentRef.uri` resolution chain (specified
below).

**Disposition.** `Planned` (DocumentRef attachment); per-row
CMRT/EMRT XML extraction is `Out of scope` for v1.

#### AVL (Approved Vendor List) management

**Industry pattern.** Not a standard — a near-universal PLM
practice. Each `Part` carries one or more **approved
manufacturer-MPN combinations** ("MPN A from Manufacturer X is
approved; MPN B from Manufacturer Y is approved-equivalent;
MPN C is unapproved"). The AVL is enforced at BOM-export time
and at procurement time.

**Datum spec gap.** No AVL field. `Part.orderable_mpns: Vec<String>`
captures alternate purchasable MPNs but does not encode approval
status.

**Recommendation.** Add `Part.avl_status: Option<AvlStatus>`:

```rust
pub struct AvlStatus {
    pub approved_combinations: Vec<AvlApprovedCombination>,
    pub policy: AvlPolicy,
    pub last_reviewed: DateTime<Utc>,
    pub reviewer: Option<String>,                       // user identity (Domain 8)
}

pub struct AvlApprovedCombination {
    pub manufacturer: String,
    pub manufacturer_jep106: Option<u16>,
    pub mpn: String,
    pub status: AvlApprovalStatus,                      // Approved | Conditional | Probation | Rejected
    pub conditions: Option<String>,                     // free-text
    pub effective_from: Option<DateTime<Utc>>,
    pub effective_until: Option<DateTime<Utc>>,
}

pub enum AvlPolicy {
    StrictApprovedOnly,         // BOM export refuses unapproved MPNs
    PreferApproved,             // BOM export warns on unapproved
    Unrestricted,               // any MPN accepted
}

pub enum AvlApprovalStatus {
    Approved,
    Conditional,
    Probation,
    Rejected,
}
```

Plus a `Project.compliance.avl_policy: AvlPolicy` for the
project-default. Plus an MCP query
`query_avl_status(part_uuid)` and a BOM-export-time check.

**Disposition.** `Planned`.

### First Article Inspection

#### AS9102 (aerospace) — Form 1 / Form 2 / Form 3

**Full title.** *AS9102 — Aerospace First Article Inspection
Requirement*. SAE / IAQG. Current revision **AS9102 Rev C
(October 2023)**, supersedes Rev B (2014).

**Issuing body.** SAE International (society) / International
Aerospace Quality Group (IAQG).

**License / IP.** Free PDF via SAE; AS9102 Rev C downloadable
from sae.org without paywall (current as of 2026-04).

**Adoption.** Mandatory for aerospace primes (Boeing, Airbus,
Lockheed, Northrop, RTX), DOD subcontractors, and most space-
launch suppliers (SpaceX subcontractors, ULA, Blue Origin
suppliers). The defining QMS doc for aerospace electronics
manufacturing.

**Scope.** AS9102 Rev C defines the data content of the
First Article Inspection report:

- **Form 1 — Part Number Accountability** — identifies the
  part being inspected: part number, revision, drawing
  reference, supplier identity, FAI report number, FAI rationale
  (initial production / change-driven / lapsed-production).
- **Form 2 — Product Accountability** — Material /
  Specifications / Special-Process verification: each
  manufacturing process and material spec referenced in the
  drawing is listed with the supplier's evidence of compliance.
- **Form 3 — Characteristic Accountability** — every
  characteristic on the drawing is enumerated with its
  measured value and pass/fail result.

**Datum's data hooks.** AS9102 consumers need:
- **BOM with part identity** — `Part.uuid` + `Part.mpn` +
  `Part.manufacturer` + `Part.manufacturer_jep106`. ✅ already
  present.
- **Version-of-record** — `SheetFrame.revision` (per
  `specs/ENGINE_SPEC.md` § 1.4 SheetFrame), the project
  schema-version, and the Part identity-stable UUIDs. ✅
  substrate present.
- **Design-rev evidence** — Transaction history with
  description and timestamp. ✅ substrate present
  (`Transaction { id, operations, description }`); needs
  user identity (Domain 8) for full evidence.
- **Variant selection** — `Variant.fitted_components`
  (per `specs/ENGINE_SPEC.md` § 1.4 Variant). ✅ already
  present.
- **ECO history** — Engineering Change Order grouping
  (recommended below); each ECO maps to one or more
  Transactions.
- **Signature evidence** — electronic signatures on
  Released library objects + on Approved ECOs. ❌
  prerequisite Domain 8.

**Recommendation.** Specify the **AS9102 evidence package
contract**:

```rust
pub struct As9102EvidencePackage {
    pub project_uuid: Uuid,
    pub project_revision: String,                   // SheetFrame.revision
    pub captured_at: DateTime<Utc>,
    pub captured_by: Option<String>,                // user identity (Domain 8)
    pub bom: Vec<As9102BomLine>,
    pub variant_uuid: Option<Uuid>,                 // which Variant was selected
    pub eco_history: Vec<As9102EcoSummary>,         // every ECO since previous FAI
    pub approval_signatures: Vec<As9102Signature>,  // Domain 8 signature substrate
    pub design_rev_evidence: As9102DesignRevEvidence,
}

pub struct As9102BomLine {
    pub designator: String,                         // R42, U7
    pub part_uuid: Uuid,
    pub mpn: String,
    pub manufacturer: String,
    pub manufacturer_jep106: Option<u16>,
    pub avl_status: Option<AvlStatus>,
    pub lifecycle: Lifecycle,
    pub supersede_chain: Vec<Uuid>,                 // if supersede has been followed
}

pub struct As9102EcoSummary {
    pub eco_uuid: Uuid,
    pub effective_at: DateTime<Utc>,
    pub rationale: String,
    pub approver: Option<String>,
    pub affected_part_uuids: Vec<Uuid>,
}

pub struct As9102Signature {
    pub role: String,                               // "Designer" | "Reviewer" | "Approver"
    pub actor: String,                              // user identity (Domain 8)
    pub signed_at: DateTime<Utc>,
    pub signature_blob: Option<Vec<u8>>,            // optional cryptographic signature (Domain 8)
}

pub struct As9102DesignRevEvidence {
    pub schematic_sha256: String,                   // identity-stable hash
    pub board_sha256: String,
    pub pool_object_uuids_used: Vec<Uuid>,          // pool objects this rev depends on
    pub native_format_schema_version: u32,
}
```

Plus an MCP tool `export_as9102_evidence(project_uuid,
output_path)` that emits the evidence package as a structured
JSON artifact + accompanying CSV bills (Form 1 / Form 2 / Form
3 helpers).

**Disposition.** `Deferred with prerequisite` — prerequisite is
Domain 8 audit-log + signature substrate. The data shape is
specified now; the export tool waits.

#### PPAP (automotive)

**Full title.** *PPAP — Production Part Approval Process*. AIAG.
Current revision **PPAP 4th Edition (2006; reaffirmed in 2018)**.

**Issuing body.** Automotive Industry Action Group (AIAG).

**License / IP.** Paid PDF via AIAG ($75 USD; not paywall-blocking
for research).

**Adoption.** Mandatory for IATF 16949-certified automotive
suppliers. 18 distinct evidence elements (Design Records, ECNs,
DFMEA, PFMEA, Control Plan, MSA, etc.).

**Datum's relevance.** Of PPAP's 18 elements, only **Element 1
(Design Records)** and **Element 4 (Control Plan — partial,
the BOM portion)** consume EDA-tool data. The other 16 are
manufacturing-process data Datum has no part of.

**Recommendation.** PPAP Element 1 (Design Records) is satisfied
by the same evidence package as AS9102 Form 1+3 — specifically,
BOM-with-revision, variant selection, design-rev evidence, ECO
history. **No separate Datum-side spec needed**; the AS9102
evidence package above covers the EDA-tool slice of PPAP. The
PLM tool (or per-customer connector) wraps the Datum evidence
into PPAP format.

**Disposition.** `Reference-only` — covered by AS9102 contract.

#### FAR (defence)

**Full title.** *First Article Records*, varies by program.
Common references: MIL-STD-1535C (cancelled 1994; superseded by
contractor-specific FAR practice), defence-prime-specific FAR
templates (Lockheed FAR-LM, Northrop FAR-NG, Raytheon FAR-RTX).

**Adoption.** Per-program; not a single industry standard.

**Datum positioning.** Same data shape as AS9102 (defence FAR
predates AS9102 and AS9102 was modelled on it); the AS9102
evidence package covers the EDA-tool slice.

**Disposition.** `Reference-only` — covered by AS9102 contract.

#### Cross-ref Domain 4 vertical mandates

Domain 4's industry-vertical research already established that
AS9100D (aerospace QMS), IATF 16949 (automotive QMS), ISO 13485
(medical QMS), and 21 CFR Part 820 (FDA QSR) all consume the
same FAI / variant / ECO substrate. **One Datum-side
evidence-package contract serves all four QMS regimes**; the
QMS-specific report formatting lives in the PLM tool.

### ECO (Engineering Change Order) Management

#### CMII methodology

**Full title.** *Configuration Management II (CMII)*. Institute
of Configuration Management (ICM); current methodology revision
CMII-100J (2018).

**Issuing body.** Institute of Configuration Management (ICM,
Phoenix AZ).

**Adoption.** Methodology, not standard. Influential in
aerospace / defence / automotive PLM design (Windchill ECO
workflow is CMII-aligned; Aras ECO workflow is CMII-aligned).
Not directly toolable.

**Datum positioning.** **CMII methodology enforcement is out
of scope.** Datum cannot enforce process discipline; it can
only expose primitives (ECO grouping, approver, effective date,
disposition) that CMII-aligned PLM workflows consume.

**Disposition.** `Out of scope` for methodology enforcement;
**`Planned`** for the data primitives (ECO grouping below).

#### MIL-HDBK-61A (note only)

**Full title.** *MIL-HDBK-61A — Configuration Management
Guidance*. US DoD. Current rev **MIL-HDBK-61A (Change Notice 1,
2001)**, never re-issued; effectively succeeded by EIA-649.

**Adoption.** Reference handbook for US-defence configuration
management; not a binding standard.

**Disposition.** `Reference-only`.

#### EIA-649

**Full title.** *EIA-649C — Configuration Management Standard*.
Electronic Industries Alliance (now SAE / EIA standards).
Current revision **EIA-649C (2019)**.

**Adoption.** Industry-wide CM reference; widely cited in
contracts and supplier requirements. Five CM functions
(Planning, Identification, Change Management, Status
Accounting, Verification).

**Datum positioning.** The five functions map to Datum
primitives:
- **Planning** — `Project.compliance.eco_workflow_required`
  flag (recommended below).
- **Identification** — UUID identity + library object lifecycle
  status. ✅ substrate present.
- **Change Management** — ECO grouping (recommended below).
- **Status Accounting** — lifecycle-event feed +
  `library_audit_log` (recommended above).
- **Verification** — AS9102 evidence package
  (recommended above).

**Disposition.** `Reference-only` for the standard;
`Planned` for the substrate primitives that satisfy it.

#### ISO 10007

**Full title.** *ISO 10007:2017 — Quality management —
Guidelines for configuration management*. ISO. Current revision
2017 (replaces 2003).

**Issuing body.** ISO TC 176 (Quality management).

**License / IP.** Paid PDF via ISO (~CHF 138). Cited but not
quoted in Datum docs — substantive content can be obtained from
free derivative documents (EIA-649C cross-references it).

**Adoption.** International QMS configuration-management
guidance; aligned with ISO 9001.

**Datum positioning.** Same five-function mapping as EIA-649C.

**Disposition.** `Reference-only`.

#### ECO data structure

Industry-converged ECO data shape (synthesised from Windchill
ChangeMaster, Aras Change Management, Arena ECO, OpenBOM ECO):

```rust
pub struct EngineeringChangeOrder {
    pub uuid: Uuid,
    pub eco_number: String,                    // human-readable; "ECO-2026-042"
    pub title: String,
    pub rationale: String,                     // why
    pub originator: Option<String>,            // user identity (Domain 8)
    pub originated_at: DateTime<Utc>,
    pub status: EcoStatus,
    pub priority: EcoPriority,
    pub disposition: EcoDisposition,
    pub effective_at: Option<DateTime<Utc>>,   // when the change becomes binding
    pub transactions: Vec<Uuid>,               // → Transaction (the bundled ops)
    pub affected_part_uuids: Vec<Uuid>,
    pub affected_board_uuids: Vec<Uuid>,
    pub affected_schematic_uuids: Vec<Uuid>,
    pub reviewers: Vec<EcoReviewerRecord>,
    pub approvers: Vec<EcoApproverRecord>,
    pub attachments: Vec<DocumentRef>,         // FMEA, test reports, vendor cert
}

pub enum EcoStatus {
    Open,
    UnderReview,
    Approved,
    Implemented,
    Rejected,
    Closed,
    Cancelled,
}

pub enum EcoPriority {
    Routine,
    Standard,
    Urgent,
    Emergency,
}

pub enum EcoDisposition {
    UseAsIs,
    Rework,
    Scrap,
    ReturnToVendor,
    NotApplicable,
}

pub struct EcoReviewerRecord {
    pub actor: String,
    pub reviewed_at: DateTime<Utc>,
    pub disposition: EcoReviewDisposition,
    pub comments: Option<String>,
}

pub enum EcoReviewDisposition {
    ApproveAsIs,
    ApproveWithComments,
    RequestChanges,
    Reject,
}

pub struct EcoApproverRecord {
    pub actor: String,
    pub role: String,                          // "Engineering" | "Quality" | "Manufacturing"
    pub approved_at: DateTime<Utc>,
    pub signature_blob: Option<Vec<u8>>,       // optional cryptographic signature (Domain 8)
}
```

#### Datum's transaction model as ECO substrate

Datum's `Transaction` (`specs/ENGINE_SPEC.md:863-870`) already
captures every authored op atomically with a UUID and
description. The ECO grouping layer wraps **N transactions** as
one ECO, adds rationale / approver / effective-date metadata,
and writes the bundle to a project-level ECO log.

**Recommendation.** The ECO entity lives at the project level,
not in a single Schematic / Board file (because an ECO can
affect both):

```
project.json
  ├── ecos/
  │   ├── eco-2026-042.json       # one ECO per file
  │   ├── eco-2026-043.json
  │   └── ...
```

Plus MCP tools:
- `open_eco { title, rationale, priority }` — creates ECO in
  `Open` status; returns ECO UUID; subsequent transactions
  on the project record their UUID into `Eco.transactions`
  if the ECO is the active one.
- `submit_eco_for_review { eco_uuid }` — transitions to
  `UnderReview`; emits lifecycle event.
- `review_eco { eco_uuid, actor, disposition, comments }` —
  records reviewer; multiple reviewers can record.
- `approve_eco { eco_uuid, actor, role, signature_blob? }` —
  records approver; multiple approvers per role can record.
- `mark_eco_implemented { eco_uuid, effective_at }` —
  transitions to `Implemented`.
- `close_eco { eco_uuid }` — final close.
- `query_ecos { project_uuid, status_filter? }` — list ECOs.
- `replay_eco { eco_uuid, target_revision }` — **deterministic
  replay of an ECO's transaction bundle onto a different
  starting revision**. This is a Datum differentiator: the
  transaction log is deterministic, so an ECO can be
  re-applied to a different starting state with full
  predictability.

**Disposition.** `Planned` (data shape + open/close MCP tools);
`Deferred with prerequisite` (review/approve transitions —
prerequisite is Domain 8 user identity + signature substrate).

### Document Management

#### `DocumentRef.uri` resolution chain (Datum-specific)

**Cross-reference.** `DocumentRef` was introduced in Domain 5
research for vendor-cert / IPC-1752A declaration / CMRT-EMRT
attestation evidence attachment. Domain 7 specifies the
`DocumentRef.uri` resolution scheme.

**Recommendation.**

```rust
pub struct DocumentRef {
    pub uri: String,
    pub kind: DocumentKind,
    pub sha256: Option<String>,                  // content-hash (when known)
    pub fetched_at: Option<DateTime<Utc>>,
    pub provenance: Option<DocumentProvenance>,
}

pub enum DocumentKind {
    Datasheet,
    AppNote,
    IpcDeclaration,             // IPC-1752A XML
    CmrtAttestation,            // CMRT 6.32 XLSX
    EmrtAttestation,            // EMRT 1.4 XLSX
    VendorCert,                 // RoHS / REACH cert
    EcoAttachment,              // FMEA / test report / vendor reply
    UserAttachment,             // arbitrary
}

pub struct DocumentProvenance {
    pub source: String,
    pub fetched_at: DateTime<Utc>,
    pub fetcher_version: Option<String>,         // Datum version that fetched
}
```

**Resolution chain (deterministic, ordered fallback).** When a
caller resolves a `DocumentRef.uri`:

1. **Local file** — if `uri` is a relative path, resolve
   relative to the project root. If the file exists and (when
   `sha256` is known) the hash matches, return the local file
   bytes.
2. **Project pool** — if the project pool contains a file at
   `pool/documents/<sha256>.<ext>`, return it. The project pool
   is git-tracked, so the file travels with the project.
3. **Shared pool** — if any shared pool (in priority order)
   contains the same `<sha256>.<ext>`, return it.
4. **External PLM** (CMIS-shaped) — if a CMIS-compatible PLM
   endpoint is configured (`Project.plm_endpoint:
   Option<CmisEndpoint>`), perform a CMIS read by URI. **Gated
   by `data_egress_policy`** — refuses on
   `NoExternalNetwork`.
5. **Cached fetch** — if `uri` is an HTTP/HTTPS URL and is not
   resolved by the previous steps, perform an HTTP GET and
   cache to `pool/documents/<sha256>.<ext>`. **Gated by
   `data_egress_policy`** — refuses on `NoExternalNetwork`;
   warns on `NoExternalAi` (non-AI external network is allowed).

The resolution is **cached** — the first successful resolution
populates the project pool and subsequent resolutions terminate
at step 2.

The cache hit/miss for each resolution is **auditable** — a
log entry records `{ uri, resolved_at, resolved_via, cache_hit:
bool, sha256_verified: bool, data_egress_decision: string }`.

**MCP tool.** `resolve_document_ref { document_ref }` returns
`{ resolved: bool, resolved_via: string, local_path: string?,
bytes: bytes?, error: string? }`.

**Disposition.** `Planned` (resolution chain + MCP tool); CMIS
external step is `Deferred with prerequisite` (prerequisite:
PLM-connector partner).

#### CMIS (OASIS standard)

**Full title.** *CMIS — Content Management Interoperability
Services*. OASIS. Current revision **CMIS 1.1 (2013)**.

**Issuing body.** OASIS Content Management Interoperability
Services Technical Committee.

**License / IP.** Free standard via oasis-open.org. Open
specification.

**Adoption.** Cross-vendor document repository API. Supported
by Alfresco, IBM FileNet, OpenText Documentum, Microsoft
SharePoint (partial), Nuxeo. **Variable** PLM vendor support —
Windchill exposes a CMIS-shaped facade; Teamcenter via the AWC
gateway; Aras via custom REST. **Not universal** — CMIS adoption
in the PLM space has been slower than the OASIS authors hoped.

**Datum positioning.** CMIS is the right **abstract contract
shape** for the external-PLM step in `DocumentRef.uri`
resolution. Datum should specify the abstract `CmisEndpoint`
contract (root URL, authentication credential reference,
namespace mapping); **per-vendor connector implementations are
out of scope.**

```rust
pub struct CmisEndpoint {
    pub base_url: String,
    pub auth_credential_ref: String,             // reference to credential store; not stored in spec
    pub repository_id: String,
    pub object_id_namespace: String,             // mapping from Datum DocumentRef.uri to CMIS object IDs
}
```

**Disposition.** `Planned` (abstract endpoint shape);
`Deferred with prerequisite` (per-vendor connector — connector work).

#### DITA (note only)

**Full title.** *DITA — Darwin Information Typing Architecture*.
OASIS. Current revision **DITA 2.0 (October 2023)**.

**Adoption.** Technical-documentation XML standard. **Not
relevant to PCB design tooling**; Datum does not produce DITA
content.

**Disposition.** `Out of scope`.

### Identity / Authentication

#### OAuth 2.0 / OIDC

**Full titles.**
- *OAuth 2.1 — The OAuth 2.1 Authorization Framework*. IETF
  draft, expected RFC ratification 2026; supersedes OAuth 2.0
  (RFC 6749, 2012) and consolidates OAuth 2.0 best-practice
  errata.
- *OpenID Connect Core 1.0*. OpenID Foundation. Current
  revision **OpenID Connect Core 1.0 (incorporating errata
  set 2)**.

**Adoption.** **Universal** for cloud-API authentication.
Octopart / Nexar uses OAuth 2.0 client-credentials flow.
Digi-Key uses OAuth 2.0 authorization-code-with-PKCE. Mouser
uses API-key (legacy) but OAuth 2.0 migration in progress.
Most cloud PLM (Arena, OpenBOM) use OAuth 2.0 + OIDC.

**License / IP.** Free RFC; open specification.

**Datum positioning.** Datum needs an OAuth 2.0 / 2.1 client
in the supply-chain MCP layer. The OAuth client is a permissive-
licensed Rust dependency; recommend **`oauth2` crate** (MIT)
for the client implementation. **Forbid GPL OAuth crates** per
the no-copyleft-integration rule.

**Disposition.** `Planned` — substrate. The Octopart / Digi-Key
/ Mouser MCP tools (Batch 1) need this client.

#### SAML

**Full title.** *SAML 2.0 — Security Assertion Markup Language
2.0*. OASIS. Current revision **SAML 2.0 (2005, OASIS Standard;
errata 2017)**.

**Adoption.** Universal for **enterprise SSO**. Windchill,
Teamcenter, Aras, Arena, OpenBOM all support SAML 2.0 SSO via
identity providers (Okta, Azure AD, Google Workspace, ADFS).
Heavy enterprise dependency.

**License / IP.** Free OASIS standard.

**Datum positioning.** **SAML is enterprise-PLM-connector
territory.** Datum does not need to embed a SAML SP (Service
Provider); SAML lives in the connector. Datum needs to accept
**federated user-identity tokens** (from any IdP) so the
audit-log surface (Domain 8) can record actor identity.

**Disposition.** `Out of scope` for engine-side SAML support;
**`Planned`** for federated-identity acceptance via a
generic `ActorIdentity` struct (Domain 8 prerequisite).

#### PKCS#11 cross-ref Domain 4

**Full title.** *PKCS #11 v3.0 — Cryptographic Token Interface
Standard*. OASIS PKCS 11 Technical Committee. Current revision
**PKCS#11 v3.0 (June 2020)**, errata 02 (2023).

**Adoption.** Universal hardware-token interface (YubiKey,
Nitrokey, smartcards, TPM 2.0 via abstraction layers).

**Datum positioning.** **Hardware-backed electronic-signature
substrate** for AS9102 / 21 CFR Part 11 / ISO 13485 sign-off
workflows. Domain 8 owns the signing surface; Domain 7
references the hardware-token interface as the prerequisite.

**Disposition.** `Deferred with prerequisite` (Domain 8
signing surface).

### Variant / Option Management

#### 180% BOM

**Industry pattern.** A "180 % BOM" enumerates every component
that **could** be fitted under any variant, even though only
~80 % are fitted in any single build. Each line carries a
"variants where fitted" set. Standard PLM practice in Aras
Variant Management, Windchill MPMLink, Arena Variants.

**Datum positioning.** Datum's `Variant` substrate
(`specs/ENGINE_SPEC.md` § 1.4 Variant) already supports this
shape. Each `Variant` has a `fitted_components: HashMap<Uuid,
bool>` (per Phase 1 audit notes); the union of all
`fitted_components` keys across all Variants is the 180 % BOM.

**Recommendation.** Add an MCP query tool
`export_180_percent_bom { project_uuid }` that returns the
union BOM with per-line variant-fitment columns. **Datum
already has the data**; this is a query convenience.

**Disposition.** `Planned` (MCP query tool only;
substrate already present).

#### Effectivity dates

**Industry pattern.** Each BOM line carries `effective_from`
and `effective_until` dates; the BOM at any rev/date is the
union of all lines effective at that rev/date. Common in
production-volume manufacturing where ECOs roll out gradually.

**Datum positioning.** Datum's `EngineeringChangeOrder`
(recommended above) carries `effective_at`. The per-Part
effectivity is implicit in the ECO transaction history; the
BOM-at-date query walks the transaction log to date X.

**Recommendation.** Add MCP query
`export_bom_effective_at { project_uuid, effective_date }`
that walks ECOs to compute the BOM as of date X.

**Disposition.** `Planned` (MCP query; substrate via ECO
grouping).

#### Where-used queries

**Industry pattern.** Universal PLM capability. "Given Part
UUID, what assemblies use it?" Aras has `Where-Used` action;
Windchill has `Where Used Report`; Teamcenter has the equivalent.
Typically multi-step in incumbent PLM tools.

**Datum positioning.** **Datum's canonical IR already indexes
every Part reference** because the schematic and board both
reference Part UUIDs explicitly. The query is **one canonical-IR
lookup** away.

**Recommendation.** Add MCP tool:

```
Method: where_used
Input:  { "part_uuid": uuid, "scope": "current_project" | "all_open_projects" | "shared_pool" }
Output: { "schematics": [{ "schematic_uuid": uuid, "sheet_uuid": uuid,
                            "designator": string, "instance_uuid": uuid }],
          "boards": [{ "board_uuid": uuid, "designator": string,
                       "placed_package_uuid": uuid }],
          "variants": [{ "variant_uuid": uuid, "fitted": bool }],
          "ecos": [{ "eco_uuid": uuid, "role": string }] }
```

**This is a Datum differentiator** — incumbent PLM tools require
multiple clicks and panel-navigation; Datum makes it one MCP call
that an AI agent can chain naturally.

**Disposition.** `Planned`.

## Cross-Domain Consolidation (Domain 7 Owns)

### Unified `refresh_supply_chain` field map

A single `refresh_supply_chain(part_uuid)` MCP call atomically
populates fields owned by Domains 2, 5, 6, and 7. The field
map is the consolidation that prior Phase-2 reports each
touched part of and explicitly deferred here.

**Field map (ratified):**

| Source field (distributor) | Datum target field | Owning Domain | Notes |
|----------------------------|--------------------|---------------|-------|
| `lifecycle` | `Part.lifecycle` | 7 | Normalised to seven-state enum |
| `last_time_buy_date` | `Part.last_time_buy_deadline` | 7 | When `Lifecycle::LastTimeBuy` |
| `replacement_mpn` (when present) | `Part.supersede.successor_part_uuid` | 7 | Resolved to Datum Part UUID |
| `offers[]` | `Part.supply_chain_offers` | 7 | Replaced atomically |
| `orderable_mpns` | `Part.orderable_mpns` | 7 | Merged (additive) |
| `packaging_options[]` | `Part.packaging_options` | 6 | EIA-481; merged |
| `compliance.rohs_status` | `Part.compliance.rohs_status` | 5 | RoHS-2/RoHS-3 flag |
| `compliance.reach_svhc[]` | `Part.compliance.reach_svhc` | 5 | List cleared and re-populated |
| `compliance.china_rohs` | `Part.compliance.china_rohs` | 5 | |
| `compliance.j_moss` | `Part.compliance.j_moss` | 5 | |
| `compliance.halogen_free` | `Part.compliance.halogen_free` | 5 | |
| `compliance.scip_id` | `Part.compliance.scip_id` | 5 | |
| `models.ibis_url` | `Part.behavioural_models[role=Ibis].provenance.source` | 2 | Provenance only; file fetch is separate |
| `models.spice_url` | `Part.behavioural_models[role=Spice].provenance.source` | 2 | Provenance only |
| `models.touchstone_url` | `Part.behavioural_models[role=Touchstone].provenance.source` | 2 | Provenance only |
| `models.step_url` | `Part.package.models_3d[].provenance.source` | 1/2 | Provenance only |
| `thermal.theta_ja` (rare) | `Part.thermal.theta_ja_c_per_w` | 2 | Only if distributor exposes |
| `parametric.*` | `Part.parametric` | 2 | Merged; never overwritten if authored value present |
| (computed) | `Part.last_supply_chain_check` | 7 | Set to "now" |
| (computed) | `Part.compliance.last_compliance_check` | 5 | Set to "now" if compliance fields touched |

**Output contract.**

```
Method: refresh_supply_chain
Input:  { "part_uuid": uuid,
          "distributors": ["octopart" | "digikey" | "mouser" | "lcsc"]?,    // default: all configured
          "fetch_models": bool }                                              // default false; true triggers behavioural-model file fetch
Output: { "part_uuid": uuid,
          "supply_chain_updated": bool,
          "compliance_updated": bool,
          "behavioural_models_updated": bool,
          "packaging_updated": bool,
          "supersede_updated": bool,
          "lifecycle_changed": { "from": string, "to": string } | null,
          "errors": [{ "distributor": string, "error_code": string,
                       "message": string, "field": string | null }],
          "transaction_id": uuid,                                              // single Transaction wrapping all field updates
          "data_egress_policy_decision": string }                              // "Allowed" | "AllowedNoAi" | "Refused"
```

**Atomicity.** All field updates land in **one Transaction**.
Partial failures (e.g., Octopart returned compliance data but
not pricing) are surfaced in `errors[]`; the Transaction
commits whatever fields succeeded. Failure to consult any
distributor is **not** an overall refresh failure —
distributor-level errors are per-row, not all-or-nothing.

**Audit logging.** The single Transaction carries description
`"refresh_supply_chain via {distributors}"` plus the
`data_egress_policy_decision` in the transaction metadata.
Domain 8's audit-log export surface picks this up natively.

**Caching / freshness.**
- `refresh_supply_chain` writes `last_supply_chain_check`
  unconditionally on success (even partial).
- A separate **freshness check** MCP tool
  `query_supply_chain_freshness { part_uuid }` returns
  `{ last_check: timestamp, age_days: int, lifecycle: string,
    reach_svhc_list_date: date }` for AI-side staleness
  warnings ("Part C42 last refreshed 90 days ago; REACH SVHC
  list updated since then").

**Disposition.** `Planned` — already in Batch 1 contract; this
report ratifies the consolidated field map.

### `data_egress_policy` gate at PLM/lookup integration layer

**Cross-reference.** Domain 4 introduced `data_egress_policy`
on `Project.compliance` for ITAR / EAR / EU dual-use posture.
The enum:

```rust
pub enum DataEgressPolicy {
    Unrestricted,         // open commercial; full external network + AI
    NoExternalAi,         // distributor lookups OK; AI tools refused
    NoExternalNetwork,    // air-gapped; nothing external
}
```

**Domain 7's specification.** Every Domain 7 MCP tool with
external-network or external-AI side effects MUST consult
`data_egress_policy` before execution and refuse-or-warn
appropriately:

| MCP tool | `Unrestricted` | `NoExternalAi` | `NoExternalNetwork` |
|----------|----------------|----------------|---------------------|
| `lookup_part_octopart` | Allowed | Allowed | **Refused** |
| `lookup_part_digikey` | Allowed | Allowed | **Refused** |
| `lookup_part_mouser` | Allowed | Allowed | **Refused** |
| `lookup_part_lcsc` | Allowed | Allowed | **Refused** |
| `refresh_supply_chain` | Allowed | Allowed | **Refused** |
| `find_alternate_parts` | Allowed | Allowed | **Refused** |
| `query_packaging_options` | Allowed | Allowed | **Refused** |
| `normalize_manufacturer` | Allowed | Allowed | **Refused** |
| `follow_supersede_chain` | Allowed (uses local cache) | Allowed | Allowed (local cache only) |
| `where_used` | Allowed | Allowed | Allowed (local-only) |
| `resolve_document_ref` (steps 1-3) | Allowed | Allowed | Allowed |
| `resolve_document_ref` (step 4 CMIS) | Allowed | Allowed | **Refused** |
| `resolve_document_ref` (step 5 HTTP) | Allowed | Warned | **Refused** |
| `export_as9102_evidence` | Allowed | Allowed | Allowed (local-only) |
| `submit_eco_for_review` | Allowed | Allowed | Allowed (local-only; PLM connector skipped) |

**Implementation contract.** The `data_egress_policy_check`
hook is a pre-execution wrapper applied uniformly to all
network-side-effect MCP tools. The hook reads the active
project's `Project.compliance.data_egress_policy`; if the
policy refuses, the tool returns
`error: data_egress_policy_violation` with a structured
violation record `{ tool, policy, project_uuid }`. Audit-log
records every policy decision.

**Disposition.** `Planned` — implementation contract ratified
here; tool-side enforcement lands when the
`data_egress_policy` field is added to `Project.compliance`
per Domain 4 batch.

### `DocumentRef.uri` resolution scheme

Specified above in § "Document Management → DocumentRef.uri
resolution chain". The scheme is the Datum-specific resolution
contract; the CMIS step is the abstract external-PLM
extension point.

**Disposition.** `Planned` (resolution chain + MCP tool); CMIS
external step `Deferred with prerequisite`.

## Cross-Cutting Patterns

### Substrate-vs-connector

The dividing line is sharp:
- **In scope (Datum engine):** data shapes, identity,
  state-transition primitives, lifecycle-event feed,
  resolution chain, atomic refresh contract, MCP query
  surface, ECO grouping, AS9102 evidence-package contract.
- **Out of scope (connector):** per-vendor wire protocols,
  authentication idiosyncrasies, business-process workflows,
  paid-catalog ingestion, PLM-tool UI, organisation-specific
  approval matrices.

This is the same framing Domains 4 (substrate-not-certifier)
and 6 (rule-pack-substrate-not-silicon-vendor) arrived at
independently. **The convergence is load-bearing — Datum's
positioning across the three "wraps a regulated workflow"
domains is internally consistent.**

### Pool layering as PLM substrate

Datum's existing pool layering (project-local + shared +
shipped, prioritised) is already the CIS / PLM substrate
pattern. Per-pool `LibraryObjectStatus` (recommended above)
extends layering with library-artefact lifecycle. Per-pool
advisory locks (recommended above) extend layering with
opt-in pessimistic check-out. **No architectural break with
existing pool semantics is required.**

### Event-driven library updates

When a library object is approved / released / deprecated,
downstream projects need to know. Two patterns:

1. **Polling (recommended for v1)** — projects re-scan their
   pool dependencies on open. Diff the cached
   `LibraryObjectStatus` against the current pool state;
   surface deltas to the user.
2. **Subscription (deferred)** — pool emits a
   `LibraryEventStream` that consumers subscribe to. Useful
   for long-running engine-daemon scenarios; not needed for
   v1 CLI / MCP usage.

**Recommendation.** Implement polling on project open; defer
subscription to a follow-on milestone.

**MCP tool:** `query_pool_library_changes_since { project_uuid,
since_timestamp }` returns `{ updated: [{ object_uuid, kind,
status_from, status_to, changed_at }] }`.

**Disposition.** `Planned` (polling); `Deferred` (subscription).

### Where-used queries

Specified above in § "Variant / Option Management →
Where-used queries". Datum's canonical IR makes this a
one-call query — **a clear differentiator over incumbents**
that require multiple panel navigations.

### Supersede chain navigation

Specified above in § "Library Lifecycle & Vault Semantics →
Supersede chains". The MCP tool `follow_supersede_chain`
walks the chain to the latest still-Active part. **AI surface
narrates the chain** ("Part C42 → C43 (EOL replacement) →
C45 (latest revision)") with rationale and pin-compatibility
notes.

### AI-native PLM (the differentiator)

Datum's AI-assistant surface can answer PLM queries that no
incumbent CIS makes conversational:

- **"Which parts in this BOM are end-of-life and have no
  supersede?"** — chain `query_bom` + `follow_supersede_chain`
  per line; AI summarises the EOL-without-replacement
  parts.
- **"Which parts have a stale supply-chain refresh?"** —
  `query_supply_chain_freshness` per BOM line; AI
  prioritises by lifecycle risk.
- **"What's the variant difference between rev A and rev B
  BOMs?"** — `export_bom_effective_at` for two dates; AI
  diffs and narrates.
- **"This part is going EOL — what's the impact?"** — chain
  `where_used` + `find_alternate_parts` + AI narrates the
  blast radius and recommended action.
- **"Generate the AS9102 evidence package for the upcoming
  build."** — `export_as9102_evidence`; AI summarises any
  missing approvals or stale data.
- **"Who approved ECO #42?"** — `query_ecos` + AI narrates
  the approval chain.
- **"What changed in the library since I last opened this
  project?"** — `query_pool_library_changes_since`; AI
  narrates relevant deltas.

**None of these are queries that incumbent CIS (OrCAD CIS,
Altium DBLib, KiCad ODBC) supports natively.** The AI surface
is the multiplier; Datum's existing canonical IR + transaction
log + pool indexes provide the data substrate; the MCP
catalog provides the query primitives.

### AS9102 evidence package contract (cross-ref Domain 8 signature substrate)

Specified above in § "First Article Inspection → AS9102". The
data shape lands in Domain 7; the signature substrate
(electronic signatures with cryptographic backing) lands in
Domain 8. **Domain 7 cannot land the export tool until Domain 8
specifies the signature surface** — they are sequenced.

### ECO model on top of the transaction substrate

Specified above in § "ECO Management → Datum's transaction
model as ECO substrate". The ECO grouping wraps N
transactions; the deterministic transaction log enables
**deterministic ECO replay**, which is a Datum differentiator.

## EDA Tool Support Matrix

Coverage levels: ✅ first-class, ◐ partial / via add-on or
plugin, ⚪ not supported, N/A not applicable to that tool's
positioning.

| Capability | Altium Designer | OrCAD CIS | PADS Library Mgr | Cadence Allegro | KiCad 7+ ODBC | Eagle / Fusion | Horizon EDA | LibrePCB | DipTrace | EasyEDA | Datum-current | Datum-recommended |
|------------|-----------------|-----------|------------------|-----------------|----------------|----------------|-------------|----------|----------|---------|---------------|-------------------|
| External CIS / DB library | ✅ DBLib / SVNDBLib | ✅ OrCAD CIS | ✅ PADS LM | ✅ Allegro CIS | ◐ ODBC (KiCad 7+) | ⚪ | ◐ Pool model | ⚪ | ◐ DipTrace Patterns Editor | ◐ Cloud catalog | ✅ Pool layering | ✅ Pool layering (formalised) |
| Library object lifecycle (Draft/Approved/Released/...) | ✅ Vault Workflow | ◐ via PLM bridge | ✅ via Teamcenter | ✅ Allegro Vault | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ `LibraryObjectStatus` (Planned) |
| Check-in / check-out (vault-style) | ✅ Vault | ✅ via PLM bridge | ✅ PADS LM | ✅ Allegro Vault | ⚪ (git only) | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ (git only) | ✅ Advisory lock (Planned) + git |
| Supersede chains | ◐ via Vault | ◐ via PLM | ⚪ | ◐ via Allegro Vault | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ `Part.supersede` + `follow_supersede_chain` (Planned) |
| Approval workflow | ✅ Vault Workflow (paid) | ⚪ | ✅ PADS LM | ✅ Allegro Vault | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ◐ State-transition primitives (Planned); workflow connector Out-of-scope |
| Library audit trail | ✅ Vault | ◐ DB row history | ✅ PADS LM | ✅ Allegro Vault | ⚪ (git log) | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ◐ Transaction log (substrate) | ✅ `library_audit_log` (Planned) |
| Octopart / Nexar integration | ✅ Manufacturer Parts Search | ⚪ | ⚪ | ⚪ | ◐ community plugin | ⚪ (deprecated) | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ MCP tools (Batch 1) |
| Direct Digi-Key / Mouser | ✅ via Manufacturer Parts Search | ⚪ | ⚪ | ⚪ | ◐ community plugins | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ MCP tools (Batch 1) |
| Atomic supply-chain refresh | ⚪ (separate refreshes) | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ `refresh_supply_chain` consolidated map (Planned) |
| Where-used (one-call) | ◐ multi-step | ◐ via PLM | ◐ via PLM | ◐ via Allegro | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ `where_used` MCP (Planned) |
| ECO grouping (engine-side) | ✅ Vault ECO | ◐ via PLM | ✅ via Teamcenter | ✅ Allegro ECO | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ◐ Transaction log (substrate) | ✅ `EngineeringChangeOrder` (Planned) |
| Deterministic ECO replay | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ `replay_eco` (Planned) |
| AS9102 evidence package | ✅ Vault FAI (paid) | ◐ via PLM | ✅ via Teamcenter FAI | ✅ via Allegro Vault | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ◐ Variant substrate only | ◐ `export_as9102_evidence` (Deferred with prerequisite — Domain 8) |
| 180 % BOM | ✅ Variants | ⚪ | ◐ via PLM | ✅ Allegro Variants | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ◐ Variant substrate only | ✅ `export_180_percent_bom` (Planned) |
| AVL management | ✅ Vault AVL | ◐ via PLM | ✅ via Teamcenter | ✅ Allegro AVL | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ `Part.avl_status` (Planned) |
| `data_egress_policy` gate | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ Project-level gate (Planned, cross-Domain-4) |
| AI-narrated PLM queries | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ⚪ | ✅ Datum differentiator (via MCP) |

**Reading the matrix.** Datum-recommended is the only column
with ✅ in the AI-native rows (atomic refresh, where-used as
one-call, deterministic ECO replay, AI-narrated PLM queries).
Incumbent tools have stronger vault/workflow stories
(Altium Vault Workflow, PADS LM, Allegro Vault) but those are
paid extensions whose value Datum can match through
substrate primitives + AI orchestration.

## Pending Exclusions (re-affirmed)

**Re-affirmed Out of scope (advisory; pending consolidated
post-Domain-8 ratification):**

- **Windchill connector** — connector work; "if a customer
  demands it, build per-customer". Datum exposes vault-API +
  lifecycle-event feed + DocumentRef CMIS hook so a
  Windchill connector can bind. **No pre-emptive build.**
- **Teamcenter connector** — same framing as Windchill.
- **Aras Innovator connector** — same framing.
- **Arena PLM connector** — same framing; cloud SaaS, so
  also incompatible with `data_egress_policy: NoExternalNetwork`.
- **OpenBOM connector** — same framing; cloud SaaS;
  hobbyist / startup-positioned tool with growing API.
- **SiliconExpert** — paid commercial catalog; deferred to
  Domain 2 advisory exclusion. **Re-affirmed.**
- **IHS Markit Engineering Workbench** — paid commercial
  catalog (now S&P Global); deferred to Domain 2 advisory
  exclusion. **Re-affirmed.**

**New advisory exclusions surfaced by this deep-dive:**

- **CMII methodology enforcement** — methodology, not toolable.
  Substrate primitives Datum provides (ECO grouping, approver,
  effective date) satisfy CMII data needs; methodology
  enforcement lives in PLM workflows, not in Datum.
- **DITA documentation pipelines** — out of EDA-tool scope.
- **PartQuest connector** — Siemens-ecosystem; defer per same
  framing as Windchill.
- **Per-vendor SAML SP integration** — federated-identity
  acceptance is in scope; per-IdP integration is connector
  work.

**Recommend NOT adding to the formal exclusion list yet:**

- **LCSC API** — strategically valuable for the JLCPCB-
  workflow user. Recommend `Planned` for a follow-on batch.
- **CMIS abstract endpoint** — `Planned` as a contract; the
  per-vendor implementation is `Deferred with prerequisite`,
  but the abstract shape should be specified now so future
  connectors have a target.

## User Pain Points & Wishlist Items

Distilled from EDA forum discussion (KiCad forum, Altium
forums, EEVblog, Reddit r/PrintedCircuitBoard, Hacker News
threads on EDA tools, 2023-2026):

1. **"My CIS is out of date — half my parts are EOL and I
   didn't know until BOM-ordering time."** — Universal
   complaint. Datum's `query_supply_chain_freshness` +
   AI-narrated freshness summary closes this gap.

2. **"I changed manufacturers on Part X, but my old assemblies
   still reference the old MPN — there's no easy way to find
   them."** — Where-used query. **Datum makes this one MCP
   call**; incumbent tools require multi-step navigation.

3. **"I need to give my contract manufacturer a 180 % BOM with
   variant fitment columns and they want it in CSV."** —
   `export_180_percent_bom` MCP tool surfaces this directly.

4. **"My customer asked for an AS9102 FAI report and I have no
   idea where to start."** — `export_as9102_evidence` produces
   the structured evidence; PLM tool / contractor formats the
   final report.

5. **"Octopart says the part is EOL but Mouser still has 5000 in
   stock."** — Lifecycle field shows EOL; supply_chain_offers
   shows Mouser stock. **Both true; the AI surface explains the
   reconciliation.**

6. **"My DBLib schema changed and now half my parts are
   broken."** — Datum's git-tracked pool avoids this — schema
   is in JSON, schema versioning is per-file. Migration is a
   pool-import operation, not a database-schema migration.

7. **"My ITAR-controlled project — can I use Octopart?"** —
   `data_egress_policy: NoExternalNetwork` on the project;
   the gate refuses the tool. Audit-logged.

8. **"Our PLM team wants to integrate with our EDA tool but
   our EDA tool doesn't expose anything useful."** — The
   vault-API surface + lifecycle-event feed + DocumentRef
   CMIS hook are exactly what a PLM team needs; Datum
   exposes them natively.

9. **"I keep losing track of which ECO introduced which
   change."** — `query_ecos` + transaction-log link from each
   ECO. AI narrates the change history.

10. **"Our librarian deprecated a part six months ago and
    nobody noticed."** — `query_pool_library_changes_since`
    surfaces the deprecation on next project open; AI
    narrates the impact.

## Datum EDA Implementation Strategy

### Hard Requirements (must support)

**HR-1. Consolidated `refresh_supply_chain` field map**

Already specified in Batch 1; this report ratifies the
cross-domain field map. Implementation is the field-map
contract above.

- Part struct changes: none beyond Batch 1; field map is the
  contract surface.
- Pool changes: none.
- Transaction model: refresh wraps in a single Transaction
  with the multi-domain field updates.
- MCP API: `refresh_supply_chain` already specified; the
  output shape needs the consolidated map fields.
- `Project.compliance` changes: `data_egress_policy` (cross-
  Domain-4) gates the call.
- Minimum viable: per-distributor lookup wired to the field
  map; partial-failure handling.
- Full: + caching + freshness query + supersede follow.
- Partner / library deps: `oauth2` crate (MIT) for OAuth
  client; `reqwest` (MIT/Apache-2.0) for HTTP client; per-
  distributor schema mapping code (Datum-internal).
- Effort: **~1 week** for the consolidated mapping on top of
  Batch 1's per-distributor stubs.

**HR-2. `data_egress_policy` gate at PLM/lookup layer**

The gate enforcement at the supply-chain integration layer is
specified in this report; the field is added in Domain 4
batch. Each external-side-effect MCP tool gets a
pre-execution wrapper.

- Part struct changes: none.
- Pool changes: none.
- `Project.compliance` changes: `data_egress_policy` field
  added per Domain 4.
- MCP API: pre-execution wrapper on every external-side-effect
  tool; structured violation record on refusal.
- Minimum viable: enum check + refusal + audit-log entry.
- Full: + structured warning surfaces + per-tool override flag
  for one-shot bypass with explicit user acknowledgement.
- Partner / library deps: none.
- Effort: **~3 days** for the wrapper + audit entries + tests.

**HR-3. Where-used MCP query**

`where_used { part_uuid, scope }` returning structured
schematic / board / variant / ECO references.

- Part struct changes: none (canonical IR already indexes).
- Pool changes: none.
- Transaction model: none.
- MCP API: one new tool.
- Minimum viable: scope = `current_project` only; schematic +
  board references.
- Full: + `all_open_projects` + `shared_pool` scopes; +
  variant + ECO references.
- Partner / library deps: none.
- Effort: **~3 days** (current_project scope; ~5 days full).

**HR-4. Supersede chain primitives**

`Part.supersede: Option<SupersedeRef>` field +
`follow_supersede_chain` MCP tool.

- Part struct: + `supersede: Option<SupersedeRef>`.
- Pool: + supersede provenance fields.
- Transaction model: + `SetPartSupersede` / `ClearPartSupersede`
  operations.
- MCP API: `follow_supersede_chain`, `set_part_supersede`,
  `clear_part_supersede`.
- Minimum viable: data shape + manual set + chain-walk MCP.
- Full: + auto-population from `refresh_supply_chain` when
  distributor exposes replacement_mpn.
- Partner / library deps: none.
- Effort: **~4 days** (including chain-walk + cycle detection).

**HR-5. Library object lifecycle states**

`LibraryObjectStatus` enum on Unit, Symbol, Entity, Package,
Padstack, Part. Default `Released` for current pool content
(no breaking change).

- Pool object structs: + `library_status: LibraryObjectStatus`
  on each.
- Pool index: + `library_status` column on each table.
- Transaction model: + state-transition operations.
- MCP API: `submit_library_object_for_review`,
  `approve_library_object`, `release_library_object`,
  `deprecate_library_object`, `obsolete_library_object`,
  `query_library_status`.
- `Project.compliance` changes: + `library_lifecycle_required:
  bool` (when true, only `Released` library objects are
  pickable).
- Minimum viable: enum + default Released + query MCP.
- Full: + state-transition MCP tools + library_audit_log
  entries.
- Partner / library deps: none.
- Effort: **~5 days** (substrate + transitions; audit-log
  consumption is Domain 8).

### Should Support (post-M7)

**SS-1. ECO grouping (`EngineeringChangeOrder`)**

Project-level ECO entity bundling N transactions with
rationale, approver, effective date.

- Part struct: none.
- Pool changes: none.
- Project changes: + `ecos/` directory in project layout.
- Transaction model: ECOs reference transactions; transactions
  optionally reference active ECO.
- MCP API: `open_eco`, `submit_eco_for_review`, `review_eco`,
  `approve_eco`, `mark_eco_implemented`, `close_eco`,
  `query_ecos`, `replay_eco`.
- `Project.compliance` changes: + `eco_workflow_required: bool`.
- Minimum viable: data shape + open/close + transaction-bundling.
- Full: + review + approve transitions (need Domain 8 user
  identity) + replay.
- Partner / library deps: none.
- Effort: **~1 week** (substrate + open/close); **~1 additional
  week** for review/approve/replay (Domain 8 prerequisite).

**SS-2. AVL management**

`Part.avl_status: Option<AvlStatus>` + project-level AVL
policy.

- Part struct: + `avl_status: Option<AvlStatus>`.
- Pool: AVL status is per-Part; persists with Part.
- Transaction model: + `SetPartAvlStatus` operation.
- MCP API: `set_part_avl_status`, `query_avl_status`,
  `validate_bom_against_avl`.
- `Project.compliance` changes: + `avl_policy: AvlPolicy`.
- Minimum viable: data shape + manual set + BOM-export warning.
- Full: + per-distributor AVL ingestion (per-customer).
- Partner / library deps: none.
- Effort: **~4 days**.

**SS-3. `DocumentRef` resolution chain (steps 1-3 + 5)**

Steps 1-3 (local, project pool, shared pool) + step 5 (HTTP
cached fetch) implementable now. Step 4 (CMIS) is `Deferred
with prerequisite` (PLM-connector partner).

- Pool changes: + `pool/documents/<sha256>.<ext>` directory
  contract.
- Transaction model: + `AttachDocumentRef` operation.
- MCP API: `resolve_document_ref`, `attach_document_ref`,
  `cache_document_ref` (HTTP fetch).
- Minimum viable: steps 1-3 + manual HTTP fetch.
- Full: + automated cache + step 4 (CMIS, prerequisite).
- Partner / library deps: `reqwest` (MIT/Apache-2.0) for
  HTTP; `sha2` (MIT/Apache-2.0) for hash verification.
- Effort: **~4 days** (steps 1-3 + 5).

**SS-4. Library audit log**

`library_audit_log: Vec<LibraryAuditEntry>` keyed off pool
object UUID.

- Pool changes: + `pool/audit_log/` directory or per-object
  `__history` JSON.
- Transaction model: every transaction touching a pool object
  appends to that object's audit log.
- MCP API: `query_library_audit_log { object_uuid }`.
- Minimum viable: append-only log + query MCP.
- Full: + tamper-evident hash chain (per Domain 8).
- Partner / library deps: none.
- Effort: **~3 days** (substrate); + Domain 8 tamper-evidence.

**SS-5. `LCSC` distributor MCP**

`lookup_part_lcsc { mpn, manufacturer? }` + LCSC contributor
to `refresh_supply_chain`.

- Part struct: none beyond Batch 1.
- Pool: none.
- MCP API: `lookup_part_lcsc`.
- Minimum viable: per-tool MCP wrapper.
- Full: + LCSC integration into `refresh_supply_chain`
  field map.
- Partner / library deps: LCSC API key (user-supplied);
  `reqwest` for HTTP.
- Effort: **~3 days**.

**SS-6. Federated `ActorIdentity` substrate**

Cross-Domain-8 prerequisite. `Transaction` carries optional
actor identity; SAML / OIDC / OAuth tokens map to actor IDs.

- Transaction model: + `actor_identity: Option<ActorIdentity>`.
- MCP API: `set_session_actor_identity { token }`.
- Minimum viable: opaque actor-identity string.
- Full: + per-IdP token validation (per-customer connector
  for SAML; OIDC/OAuth in core).
- Partner / library deps: `oidc-client` or equivalent
  permissive-licensed Rust crate.
- Effort: **~1 week** (substrate); per-IdP connector is
  per-customer.

### On-Demand Only

**OD-1. KiCad ODBC database library importer**

Read ODBC table → emit Datum pool JSON. Same pattern as the
existing KiCad library importer; valuable for KiCad shops
with established CIS infrastructure but not pre-emptively
buildable.

**OD-2. Per-vendor PLM connector (Windchill, Teamcenter,
Aras, Arena, OpenBOM)**

Each is a per-customer connector that reads from the PLM
vault, writes to Datum pool, and binds to the lifecycle-
event feed. **Datum exposes the abstract surface; the
per-vendor wire protocol is connector work.**

**OD-3. PartQuest / SiliconExpert / IHS Markit ingestion**

Paid commercial catalogs; per-customer ingestion only.

**OD-4. CMRT / EMRT XLSX row-level extraction**

Per-supplier CMRT files; XLSX parser + 3TG-row extraction.
Per-customer; the DocumentRef attachment substrate is in
scope, the row-level parser is not.

**OD-5. PPAP-specific report formatting**

PPAP report wrapping the AS9102 evidence package; per-
customer or per-PLM connector.

**OD-6. CMIS per-vendor PLM endpoint**

Per-vendor CMIS bridge (Windchill, Alfresco, etc.);
per-customer connector.

### Out of Scope (recommend formal exclusion)

Per-customer build only. **Recommend formal exclusion in the
post-Domain-8 consolidated ratification pass:**

- **Windchill, Teamcenter, Aras, Arena, OpenBOM connectors** —
  per-customer build only; abstract hooks ship.
- **SiliconExpert, IHS Markit Engineering Workbench** — paid
  catalogs; deferred to Domain 2.
- **CMII methodology enforcement** — methodology, not toolable.
- **DITA documentation pipelines** — out of scope.
- **PartQuest / Altium Manufacturer Parts Search** — vendor-
  specific catalog bridges; defer to per-customer demand.
- **Per-IdP SAML SP integration** — connector work; federated-
  identity acceptance is in scope.

For each "must support" and "should support" the deep-dive
breakouts above already enumerate the changes; the table
below is the consolidated effort summary.

| ID | Capability | Effort (min viable) | Effort (full) | Domain 8 prereq? |
|----|-----------|---------------------|---------------|--------------------|
| HR-1 | Consolidated refresh | ~5 days | ~1 week | No |
| HR-2 | data_egress_policy gate | ~2 days | ~3 days | No (cross-Domain-4) |
| HR-3 | where_used MCP | ~3 days | ~5 days | No |
| HR-4 | Supersede chain | ~3 days | ~4 days | No |
| HR-5 | Library object lifecycle | ~3 days | ~5 days | Partial (audit-log full requires Domain 8) |
| SS-1 | ECO grouping | ~1 week | ~2 weeks | Yes (review/approve transitions) |
| SS-2 | AVL management | ~3 days | ~4 days | No |
| SS-3 | DocumentRef resolution | ~3 days | ~4 days | No (CMIS step deferred) |
| SS-4 | Library audit log | ~2 days | ~3 days | Yes (tamper-evident hash chain) |
| SS-5 | LCSC distributor MCP | ~2 days | ~3 days | No |
| SS-6 | ActorIdentity substrate | ~5 days | ~1 week | Cross-Domain-8 |

**Total Hard Requirements effort:** ~3 weeks (full
implementation, sequencable in parallel).
**Total Should Support effort:** ~5-6 weeks (post-M7,
sequencable in parallel; SS-1 ECO and SS-4 audit log are
Domain-8-paced).

### Datum Differentiators

Where Datum's pool + transaction + AI surfaces can do better
than incumbent CIS / PLM tools:

1. **AI-explained EOL impact analysis** — chain `query_bom`
   + `follow_supersede_chain` + `where_used`; AI summarises
   which projects are affected and what supersede candidates
   exist. Incumbent CIS surfaces raw data; Datum surfaces the
   conclusion.

2. **MCP-queryable AVL** — `query_avl_status` + per-BOM-line
   filter. Incumbent CIS requires manual cross-reference;
   Datum makes it a single MCP call.

3. **Deterministic ECO replay** — `replay_eco { eco_uuid,
   target_revision }` re-applies an ECO's transaction bundle
   onto a different starting state. **No incumbent CIS
   does this** because their transaction models are not
   deterministic.

4. **Where-used as one MCP call** — vs Altium's multi-step
   Vault query, vs OrCAD's report-builder navigation, vs
   KiCad's complete absence of the feature.

5. **AI-narrated supersede-chain navigation** — "Part C42 is
   EOL; the recommended replacement is Part X (3 generations
   down the chain), pin-compatible, +5 % cost, same package."
   Incumbent CIS surfaces the chain as a table; Datum surfaces
   the recommendation.

6. **AI-narrated variant diff** — "Rev A had R42 = 10kΩ; Rev
   B has R42 = 22kΩ; the change was introduced by ECO #38
   on 2026-03-12; rationale: 'EMC tuning per CISPR 22 Class
   B compliance'." All four data points come from one chain
   of MCP calls.

7. **`data_egress_policy` is auditable** — every refused-or-
   warned external call is logged with the policy decision.
   Auditors can show ITAR / EAR compliance from the audit
   log; no incumbent CIS provides this.

8. **Pool layering is git-native** — no DBA, no schema
   migration, no SVN-on-top-of-MS-SQL-on-top-of-ODBC
   plumbing. Pool changes are PRs; library reviews are PR
   reviews.

9. **AS9102 evidence package as one MCP call** —
   `export_as9102_evidence` produces the structured evidence;
   no incumbent CIS makes this conversational.

10. **Federated identity acceptance** — Datum doesn't bind to
    a specific IdP; the `ActorIdentity` substrate accepts any
    federated identity token. Enterprise customers don't have
    to migrate IdPs to use Datum.

### Recommended Spec Edits

**NOTE.** Claude Code is in research-only mode per
`feedback_research_only_mode` memory rule. The following are
**recommendations only**; the user reviews and applies the
spec edits.

1. **`specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.7** — extend the
   Domain 7 disposition list with the 12 new dispositions
   surfaced in this report. Suggested replacement text:

   ```
   ### 4.7 PLM And Lifecycle Integration

   - Multi-pool layering and per-Part lifecycle status:
     `Implemented` (extend `Lifecycle` enum to seven-state
     with `Preview` and `LastTimeBuy`).
   - Library object lifecycle states (`LibraryObjectStatus`):
     `Planned`.
   - Library object check-in / check-out (advisory locks):
     `Planned`. Pessimistic vault-style: `Deferred with
     prerequisite` (Domain 8 user identity).
   - Supersede chains (`Part.supersede`): `Planned`.
   - Approval-workflow primitives (state-transition MCP):
     `Planned`. Workflow-engine integration: `Out of scope`.
   - Library audit trail substrate: `Planned`.
     Audit-log export surface: `Deferred with prerequisite`
     (Domain 8).
   - Consolidated `refresh_supply_chain` field map (Domains
     2/5/6/7 atomic refresh): `Planned`.
   - `data_egress_policy` gate at PLM/lookup integration
     layer: `Planned` (cross-Domain-4).
   - `DocumentRef.uri` resolution chain (steps 1-3 + 5):
     `Planned`. CMIS external step (step 4): `Deferred with
     prerequisite` (PLM-connector partner).
   - AVL (Approved Vendor List) management: `Planned`.
   - 180 % BOM export, effectivity-date BOM export: `Planned`
     (substrate via Variant; query MCP tools).
   - Where-used MCP query: `Planned`.
   - Engineering Change Order grouping
     (`EngineeringChangeOrder`): `Planned` (substrate +
     open/close); review/approve transitions: `Deferred with
     prerequisite` (Domain 8).
   - AS9102 First Article Inspection evidence package:
     `Deferred with prerequisite` (Domain 8 signature
     substrate).
   - PPAP / FAR evidence: `Reference-only` (covered by
     AS9102 contract).
   - CMII / EIA-649 / ISO 10007 methodology enforcement:
     `Reference-only`. Substrate primitives (ECO grouping,
     library lifecycle): `Planned`.
   - OAuth 2.1 + OIDC client substrate: `Planned`.
   - SAML 2.0 SP integration: `Out of scope` (federated-
     identity acceptance via `ActorIdentity`: `Planned`,
     cross-Domain-8).
   - PKCS#11 hardware-token signing: `Deferred with
     prerequisite` (Domain 8 signature surface).
   - LCSC distributor MCP tool: `Planned` (post-M7 batch).
   - Octopart / Nexar / Digi-Key / Mouser MCP tools:
     `Planned` (already in Batch 1).
   - Vendor PLM connectors (Windchill, Teamcenter, Aras,
     Arena, OpenBOM): `Out of scope` (per-customer build
     only; abstract vault-API + lifecycle-event feed +
     DocumentRef CMIS hook ship in scope).
   - Paid commercial catalogs (SiliconExpert, IHS Markit
     Engineering Workbench, PartQuest): `Out of scope`
     (per-customer ingestion only).
   - DITA documentation pipelines: `Out of scope`.
   ```

2. **`specs/ENGINE_SPEC.md` § 1.1 / § 1.2** — extend `Lifecycle`
   enum and `Part` struct:

   - `Lifecycle::Preview`, `Lifecycle::LastTimeBuy` added.
   - `Part.last_time_buy_deadline: Option<DateTime<Utc>>`.
   - `Part.supersede: Option<SupersedeRef>` + `SupersedeRef`,
     `SupersedeReason`, `SupersedeSource` types.
   - `Part.avl_status: Option<AvlStatus>` + `AvlStatus`,
     `AvlApprovedCombination`, `AvlPolicy`,
     `AvlApprovalStatus` types.
   - Pool object structs (`Unit`, `Symbol`, `Entity`,
     `Package`, `Padstack`, `Part`) gain
     `library_status: LibraryObjectStatus` (default
     `Released`).
   - `LibraryObjectStatus` enum.
   - `LibraryAuditEntry`, `LibraryAuditEvent` types.
   - `EngineeringChangeOrder`, `EcoStatus`, `EcoPriority`,
     `EcoDisposition`, `EcoReviewerRecord`,
     `EcoApproverRecord` types.
   - `DocumentRef`, `DocumentKind`, `DocumentProvenance`
     types.
   - `As9102EvidencePackage`, `As9102BomLine`,
     `As9102EcoSummary`, `As9102Signature`,
     `As9102DesignRevEvidence` types.
   - `CmisEndpoint` type (abstract endpoint shape only).
   - `PoolLock` type (advisory lock).
   - `ActorIdentity` type (cross-Domain-8 substrate).

3. **`specs/ENGINE_SPEC.md` § 3 Operations** — add operations:

   - `SetPartSupersede`, `ClearPartSupersede`.
   - `SetPartAvlStatus`.
   - `SetLibraryObjectStatus` (operates on any pool object).
   - `OpenEco`, `SubmitEcoForReview`, `ApproveEco`,
     `MarkEcoImplemented`, `CloseEco`.
   - `AttachDocumentRef`, `DetachDocumentRef`.
   - `AcquirePoolLock`, `ReleasePoolLock` (advisory).

4. **`specs/MCP_API_SPEC.md` § Supply Chain (extend)** —
   ratify the consolidated `refresh_supply_chain` output
   shape. Add new tools:

   - `lookup_part_lcsc` (post-M7 batch).
   - `query_supply_chain_freshness`.
   - `follow_supersede_chain`.
   - `find_alternate_parts` (extend output shape with
     supersede chain integration).

5. **`specs/MCP_API_SPEC.md` § PLM & Library** (new section) —
   add MCP tools:

   - `where_used`, `export_180_percent_bom`,
     `export_bom_effective_at`.
   - `query_avl_status`, `set_part_avl_status`,
     `validate_bom_against_avl`.
   - `submit_library_object_for_review`,
     `approve_library_object`, `release_library_object`,
     `deprecate_library_object`, `obsolete_library_object`,
     `query_library_status`,
     `query_library_audit_log`,
     `query_pool_library_changes_since`.
   - `open_eco`, `submit_eco_for_review`, `review_eco`,
     `approve_eco`, `mark_eco_implemented`, `close_eco`,
     `query_ecos`, `replay_eco`.
   - `resolve_document_ref`, `attach_document_ref`,
     `cache_document_ref`.
   - `acquire_pool_lock`, `release_pool_lock`,
     `query_pool_locks`.
   - `export_as9102_evidence` (Deferred with prerequisite).
   - `set_session_actor_identity` (cross-Domain-8).

6. **`docs/POOL_ARCHITECTURE.md` § 2 / § 5** — extend with:

   - `pool/documents/<sha256>.<ext>` directory contract for
     `DocumentRef` storage.
   - `pool/locks/<object-uuid>.lock` directory contract for
     advisory locks.
   - `pool/audit_log/` or per-object `__history` directory
     contract for library audit log.
   - Add a "Pool Layering as PLM Substrate" subsection
     formalising the three-tier authoritative-source pattern
     and explicitly framing it as the OrCAD-CIS / Altium-
     DBLib equivalent.

7. **`docs/LIBRARY_ARCHITECTURE.md`** — extend with:

   - "Library Object Lifecycle" section covering Draft /
     InReview / Approved / Released / Deprecated / Obsolete.
   - "Vault Semantics" section covering optimistic-by-default,
     advisory-lock-opt-in, git-native merge.
   - "PLM Connector Surface" section covering vault-API +
     lifecycle-event feed + DocumentRef CMIS hook as the
     abstract interface for any future PLM connector.

8. **`docs/INTEROP_SCOPE.md`** — add explicit rows:

   - **PLM connectors (Windchill, Teamcenter, Aras, Arena,
     OpenBOM)**: per-customer connector work only; abstract
     hooks ship in scope.
   - **CMIS (OASIS Content Management Interoperability
     Services)**: abstract endpoint shape `Planned`;
     per-vendor implementation deferred.
   - **OAuth 2.1 / OIDC client**: `Planned` substrate.
   - **SAML 2.0 SP**: `Out of scope`; federated-identity
     acceptance via `ActorIdentity` is `Planned`.
   - **LCSC API**: `Planned` post-M7.
   - **Newark / Element14, Arrow, Avnet, PartQuest,
     SiliconExpert, IHS Markit**: `Out of scope` for pre-emptive
     build; per-customer connector recipe.

9. **`specs/NATIVE_FORMAT_SPEC.md` § 6.1** — extend
   `project.json` schema with optional fields:

   - `compliance.eco_workflow_required: bool`.
   - `compliance.library_lifecycle_required: bool`.
   - `compliance.avl_policy: "StrictApprovedOnly" |
     "PreferApproved" | "Unrestricted"`.
   - `plm_endpoint: CmisEndpoint | null` (when PLM connector
     is configured).

10. **`specs/NATIVE_FORMAT_SPEC.md` § 6 (new subsection)** —
    `ecos/<eco-uuid>.json` per-ECO file contract.

11. **`specs/NATIVE_FORMAT_SPEC.md` § 6 (new subsection)** —
    `pool/documents/<sha256>.<ext>` content-addressed
    document storage contract.

12. **`specs/NATIVE_FORMAT_SPEC.md` § 6 (new subsection)** —
    `pool/locks/<object-uuid>.lock` advisory-lock contract.

**Total recommended spec edits: 12** (across 5 spec/doc
files).

## Cross-Domain Insights to Thread Forward

Domain 8 (process & quality) is the natural sibling and
consumer of Domain 7's substrate. Domain 8 deep-dive should
know:

1. **ECO grouping is a Domain 7 deliverable; Domain 8 owns the
   approval-workflow gating, signature substrate, and audit-log
   export surface.** The ECO data shape (above) carries
   `EcoApproverRecord.signature_blob` as an opaque hook for
   Domain 8's signature substrate. Domain 8 specifies the
   signature schema; Domain 7 specifies the data shape that
   carries it.

2. **AS9102 evidence package is specified in Domain 7;
   electronic-signature surface is Domain 8.** The evidence
   package (above) carries `As9102Signature.signature_blob`
   as an opaque hook. Domain 8 specifies the cryptographic
   primitives, hardware-token (PKCS#11) integration, and
   signature-verification semantics. Domain 7 specifies what
   data is being signed.

3. **Library audit trail substrate (recommended above) is the
   data feed for Domain 8's audit-log export surface.** The
   `LibraryAuditEntry` shape carries every library-object
   state transition; Domain 8 wraps this with a queryable
   export surface (CSV / JSON / structured report) that
   satisfies ISO 9001 § 7.5, ISO 13485 § 4.2.5, 21 CFR Part
   820.30 audit requirements.

4. **`refresh_supply_chain` events are audit-log entries.**
   Each refresh writes one Transaction with the
   `data_egress_policy_decision` in the Transaction metadata.
   Domain 8 audit-log queries can filter on
   `data_egress_policy_decision` to surface ITAR / EAR /
   compliance posture violations.

5. **`ActorIdentity` substrate (recommended above) is the
   cross-domain user-identity model.** Domain 7 introduces
   the type as a hook for transaction attribution; Domain 8
   specifies the IdP integration patterns (OIDC, SAML
   federation), token validation, and per-action authority
   model.

6. **Library lifecycle event feed is a queryable audit
   primitive.** Domain 8's audit-log surface queries the
   event feed for "what library objects changed status
   between date X and date Y" — directly satisfies ISO 9001
   § 8.5.6 (control of changes).

7. **Pessimistic check-out (vault-style) is `Deferred with
   prerequisite` on Domain 8 user identity.** The advisory
   lock substrate (recommended above) ships before; the
   vault-style check-out enforcement layers on once
   `ActorIdentity` is real.

8. **Tamper-evident hash chain on `library_audit_log` is a
   Domain 8 deliverable.** Domain 7 specifies the audit-log
   data shape; Domain 8 specifies whether to add a hash
   chain (Git-style commit hashes link entries; or a
   purpose-built Merkle log; or rely on the underlying
   git's content-addressed history).

9. **The `data_egress_policy` audit log is a Domain 8 surface
   consumer.** Every gated tool call writes
   `{ tool, policy, decision, project_uuid, actor_id?,
   at }` — Domain 8 surfaces this to auditors as the ITAR /
   EAR / EU-dual-use compliance evidence.

## Sources

- **STANDARDS_AUDIT.md** (`research/standards-audit/`)
  — Phase 1 audit; Domain 7 inventory and triage. Frames the
  12 blind spots resolved here and the advisory-exclusion
  list ratified here.
- **COMPONENT_MODELING_RESEARCH.md**
  (`research/component-modeling/`) — Domain 2 deep-dive;
  consolidated Octopart / Nexar / Digi-Key / Mouser MCP-tool
  design that Domain 7 references for the field map.
- **MATERIALS_ENVIRONMENTAL_RESEARCH.md**
  (`research/materials-environmental/`) — Domain 5 deep-dive;
  Domain 7 ratifies the cross-domain refresh field map first
  recommended in Domain 5.
- **EMC_SIGNAL_INTEGRITY_RESEARCH.md**
  (`research/emc-signal-integrity/`) — Domain 6 deep-dive;
  PHY-IC vendor lookup and rule-pack pool-content framing.
- **INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md**
  (`research/industry-vertical-compliance/`) — Domain 4
  deep-dive; `data_egress_policy` posture and AS9102
  consumes-variant-substrate framing.
- **IPC_COMPLIANCE_RESEARCH.md** (`research/ipc-compliance/`)
  — IPC-1782 traceability cross-reference; CMRT/EMRT
  attestation framing.
- **AS9102 Rev C** — *Aerospace First Article Inspection
  Requirement*. SAE / IAQG; **free PDF via sae.org**
  (verified accessible 2026-04). Source for AS9102 evidence
  package contract.
- **EIA-649C** — *Configuration Management Standard*.
  Electronic Industries Alliance (now SAE); paid PDF via
  sae.org. Substantive content available via free
  cross-reference summaries (NIST, MIL-HDBK-61A).
- **ISO 10007:2017** — *Quality management — Guidelines for
  configuration management*. ISO; paid PDF (CHF 138).
  Cross-referenced through EIA-649C; not separately quoted.
- **CMII Methodology** — Institute of Configuration
  Management; CMII-100J (2018). Methodology, not standard.
  Free overview at icminstitute.org.
- **PPAP 4th Edition** — *Production Part Approval Process*.
  AIAG; paid PDF ($75 USD). Cross-referenced via AIAG
  summary materials.
- **OAuth 2.1 (IETF draft)** —
  https://datatracker.ietf.org/doc/draft-ietf-oauth-v2-1/
  Free.
- **OpenID Connect Core 1.0** —
  https://openid.net/specs/openid-connect-core-1_0.html
  Free.
- **SAML 2.0** — OASIS;
  http://docs.oasis-open.org/security/saml/v2.0/
  Free.
- **PKCS#11 v3.0** — OASIS PKCS 11 TC;
  https://docs.oasis-open.org/pkcs11/pkcs11-base/v3.0/
  Free.
- **CMIS 1.1** — OASIS;
  http://docs.oasis-open.org/cmis/CMIS/v1.1/
  Free.
- **OrCAD CIS User Guide** — Cadence Design Systems
  documentation; reference-only; vendor-internal in
  the modern Cadence Help system.
- **Altium DBLib documentation** — Altium Designer 24
  Help; vendor-internal at
  https://www.altium.com/documentation/altium-designer/
  ; reference-only.
- **KiCad Database Library documentation** —
  https://docs.kicad.org/8.0/en/eeschema/eeschema.html
  § Database Libraries. Free.
- **Windchill PLM** — PTC (now Hexagon); vendor-internal
  documentation; surveyed at the abstraction-only level
  per pending-exclusions policy.
- **Teamcenter** — Siemens Digital Industries Software;
  vendor-internal documentation; surveyed at the
  abstraction-only level.
- **Aras Innovator** — Aras Corporation;
  https://www.aras.com/en/community/documentation
  ; surveyed at the abstraction-only level.
- **Arena PLM** — PTC;
  https://www.arenasolutions.com/resources/
  ; surveyed at the abstraction-only level.
- **OpenBOM** — OpenBOM LLC;
  https://www.openbom.com/api
  ; surveyed at the abstraction-only level.
- **Octopart / Nexar GraphQL API** —
  https://docs.nexar.com/
  Free tier with attribution requirement.
- **Digi-Key Developer Portal** —
  https://developer.digikey.com/
  Free OAuth 2.0 access.
- **Mouser API Hub** — https://www.mouser.com/api-hub/
  Free API-key tier.
- **LCSC API** — https://www.lcsc.com/help-center/
  API documentation. Free.
- **CMRT 6.32** — Responsible Minerals Initiative;
  https://www.responsiblemineralsinitiative.org/reporting-templates/cmrt/
  Free template.
- **EMRT 1.4** — RMI;
  https://www.responsiblemineralsinitiative.org/reporting-templates/emrt/
  Free template.
- **TI Product-Status Codes** — public lifecycle
  documentation;
  https://www.ti.com/support-quality/quality-policies-procedures/product-discontinuance-pcn.html
  Free reference.
