# R1-G0 Foundation Gate

This document is the evidence bundle for the minimal `R1-G0` history/context
gate.

It is not a second status ledger. Completion state is tracked only in
`specs/PROGRESS.md`.

## 1. Tool Lineage + Format Evolution Map

`R1-G0` is not a commitment to implement commercial import now. It is the
minimum historical/context gate needed so later milestone claims are not made
without understanding how each tool family evolved and where migration risk
actually sits.

### 1.1 Why this map exists

The project already has direct experience with two open-format ecosystems:

- KiCad as the current imported-board write-back target in `M3`
- Eagle as the current library-import and replacement-plan provenance surface

That work established two relevant facts:

- format family history materially affects parser and fidelity strategy
- migration value is not "can we parse bytes", but "can we preserve usable
  design intent under automation"

The same framing must hold before any commercial-target closure claims are
allowed.

### 1.2 Lineage map

| Tool family | Major format families / evolution | Likely ingestion surfaces | Constraints that matter to this project |
|-------------|-----------------------------------|---------------------------|-----------------------------------------|
| KiCad | Versioned S-expression project/design files (`.kicad_pcb`, `.kicad_sch`, `.kicad_pro`, symbol and footprint libraries). Current repo scope treats KiCad 7/8/9 as one version-tagged family. | Native text import, direct parser, write-back for supported board slice. | Open and practical, but version churn still matters; formatting can change while semantic preservation must not. Current repo evidence already treats custom rules, teardrops, and rendering hints as bounded lossiness. |
| Eagle | Pre-Autodesk historical lineage exists, but the current repo evidence is anchored on Eagle 6.x-9.6.2 XML/DTD-backed `.brd`, `.sch`, and `.lbr` families. | Native XML import; library extraction is already proven practical. Design import is a future slice; exported artifacts and embedded libraries are still useful migration surfaces. | Better documented than commercial binaries, but still carries ecosystem-specific semantics such as deviceset/device/connect chains, ULP-generated content, and inaccessible Fusion 360 cloud references. |
| Altium | Multiple historical generations centered around native design documents (`.PcbDoc`, `.SchDoc`) with richer workflow semantics than a simple text board/schematic pair. | Prefer staged flows first: library/component extraction, vetted parsing libraries, or tool-assisted/exported intermediates before any broad native-file claims. | High semantic density, weak format accessibility, and strong user expectations once support is claimed. Risk is not just parsing; it is preserving rule/variant/library intent without contaminating canonical IR. |
| PADS | Multiple historical generations and ownership eras with materially different data/export surfaces across vintages. | First target should be the most common text/exportable path, not broad native-format write-back. Read-only migration is the likely first usable path. | Version fragmentation is itself part of the migration problem. "PADS support" is too vague unless scoped to a concrete generation and ingestion path. |
| OrCAD / Allegro | Split lineage across schematic, board, and enterprise/process-heavy domains rather than one clean format family. | Exported interchange artifacts and library extraction should come before native project ingestion. | Enterprise database/process coupling makes direct importer claims strategically expensive. The core risk is not only syntax but hidden process assumptions around libraries, variants, and manufacturing data. |

### 1.3 Directional implications

This lineage map constrains the roadmap in a few non-optional ways:

- KiCad/Eagle remain the correct early execution strategy because they are
  sufficiently open to prove parser, fidelity, and loss-reporting discipline.
- Commercial targets must be framed as migration programs, not "just add a
  parser" work items.
- Library extraction and exported/intermediate ingestion are safer first
  surfaces than native design parity claims for Altium, PADS, or OrCAD.
- Canonical IR must stay vendor-neutral. Vendor-specific concepts may map,
  become structured metadata, or be reported as unsupported; they do not get
  to redefine the core model.

## 2. Migration Pain Taxonomy

### 2.1 Evidence basis

This taxonomy is grounded in actual repo evidence, not only abstract
commercial-tool speculation:

- the current KiCad imported-board write/write-back corpus used to close `M3`
- Eagle `.lbr` import and replacement-manifest flows used in current CLI tests
- the project test strategy that explicitly treats real designs as the trust
  anchor instead of synthetic-only fixtures

Representative current examples in-repo:

- `DOA2526` as the primary KiCad board/schematic reference design
- `partial-route-demo.kicad_pcb` as the current replacement/workflow fixture
- `simple-opamp.lbr` as the current Eagle library/provenance fixture

These do not yet constitute the full commercial corpus required by broader `R1`
exit criteria. They are sufficient for the minimal `R1-G0` gate because they
already expose the categories of migration pain the project must be ready to
classify before claiming later milestone closure.

### 2.2 Recurring migration failure modes

| Pain category | Observed/derived basis in current repo | Why it matters for future commercial interop |
|---------------|----------------------------------------|----------------------------------------------|
| Format/version churn | KiCad is already treated as a version-tagged family; native-format work also assumes schema migration as formats evolve. | A migration program must separate semantic compatibility from file-version syntax compatibility. Commercial targets will be worse, not better, on this axis. |
| Library identity drift | Eagle library import and scoped replacement manifests already record library provenance explicitly. KiCad library mapping is documented as lossy because KiCad lacks Eagle's explicit device binding chain. | Component identity is not stable unless symbol/package/part provenance is carried deliberately. Commercial migration will fail early if library identity is guessed instead of tracked. |
| Representation gaps between tools | Current accepted lossiness already includes custom rules, teardrop metadata, some rendering hints, and inaccessible cloud references. | Commercial tools will introduce more of these gaps. A gate is only credible if the project distinguishes exact mapping, approximation, metadata preservation, and unsupported loss up front. |
| Board-vs-schematic asymmetry | `M3` intentionally closed only for imported-board write-back while imported-schematic editing remained deferred. | This is a direct reminder that "design import" is not one homogeneous capability. Future migration claims must state which design domains are actually preserved and editable. |
| Derived-data instability | Current round-trip policy already excludes comments/whitespace and recomputes derived geometry such as zone fills. | Commercial migration cannot promise byte identity for every derived artifact. It must define where semantic stability is enough and where exact preservation is required. |
| Rule-system mismatch | KiCad custom constraint expressions are explicitly deferred/accepted lossiness; Eagle DRC semantics are only approximate in current scope. | Rule languages and priority systems are one of the hardest cross-tool translation problems. Commercial support must not imply rule parity unless directly evidenced. |
| Hidden workflow semantics | Eagle ULP/runtime assumptions and Fusion 360 references already show that source ecosystems depend on external behaviors, not just files. | Altium/OrCAD/PADS will have even more hidden semantics: variants, databases, enterprise libraries, managed content, fabrication integrations. |
| Automation blockers | Current project value depends on deterministic CLI/MCP surfaces, stable exit codes, and explicit manifests. Existing EDA tools often do not expose those surfaces cleanly. | A migration path that requires opaque GUI replay or vendor runtimes is strategically weaker than one that produces auditable imported state plus machine-readable loss reports. |

### 2.3 Taxonomy summary

The core failure mode to avoid is silent semantic collapse. For this project,
the dangerous cases are:

- object identity guessed instead of preserved
- rules approximated without reporting
- board artifacts rewritten successfully while schematic/library intent drifts
- imported data treated as canonical without provenance
- "opens without errors" mistaken for "migrated faithfully"

That is the practical taxonomy this gate is meant to force into the project
before downstream completion claims.

## 3. Fidelity Boundary Policy

### 3.1 Policy purpose

The project already contains the beginnings of a fidelity policy in
`docs/INTEROP_SCOPE.md` and `specs/IMPORT_SPEC.md`. `R1-G0` turns that into an
explicit rule for future interop claims, especially commercial ones.

### 3.2 Policy

#### Exactness required

The following must be preserved exactly or the migration is considered failed
for the affected scope:

- component/reference identity needed to maintain deterministic object tracking
- net/connectivity intent for supported imported domains
- board geometry and authored coordinates for supported board objects
- package/part/library provenance when replacement or library-origin claims are
  made
- save/load/save determinism for the claimed native or imported persistence
  surface

#### Acceptable approximation

Approximation is acceptable only when all of the following are true:

- the approximation is explicitly documented
- the affected data is not part of the claimed exactness surface
- the result remains electrically/manufacturing usable for the claimed scope
- the approximation is detectable in tests or reports

Current examples already accepted by the repo:

- formatting, ordering, and whitespace drift in KiCad write-back
- recomputed derived data such as zone fill geometry
- deferred support for teardrop metadata, 3D assignments, and some rendering
  hints
- approximate import of source rule systems that do not cleanly map

#### Preserve as metadata

If source data cannot be mapped into canonical IR without distortion but still
matters for audit or future recovery, it must be preserved as structured import
metadata rather than discarded silently.

This is the correct destination for:

- vendor-specific rule constructs
- workflow or library-origin annotations that matter to later migration steps
- source-version and manifest provenance
- unsupported-but-detectable source attributes that may become mappable later

#### Unsupported-loss boundary

The following are not acceptable:

- silent loss of connectivity or component identity
- coordinate drift that changes board meaning while claiming write fidelity
- dropping source semantics without reportable evidence when they influenced
  migration decisions
- broad support claims based only on successful file opening
- vendor-specific concepts forced into canonical IR without a stable cross-tool
  meaning

### 3.3 Explicit future-claim language

Future interop claims in this repository must follow this form:

- what source family and version/generation are in scope
- what ingestion path is used (native parser, export, or tool-assisted)
- what is exact
- what is approximated
- what is preserved as metadata
- what is unsupported

Unqualified statements such as "supports Altium" or "imports PADS" are not
acceptable engineering claims under this policy.

## 4. Gate Result

`R1-G0` is satisfied when the project can point to this document and show:

- a historical map for the relevant tool families
- a migration-pain taxonomy grounded in actual repo evidence
- a fidelity policy that constrains future claims

That is sufficient for the minimal history/context gate. It is not a substitute
for the broader `R1` research track exit criteria, which still require corpus,
legal posture, prototype work, and target selection.
