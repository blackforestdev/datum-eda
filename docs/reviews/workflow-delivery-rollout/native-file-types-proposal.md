# Native file types and extensions proposal

Status: **proposed; extension family, opening behavior and migration pending owner ratification**.
Planning lane: GUI-SURFACE-SPECS / SURFACE-S01, coordinated claim
`codex-library-foundation-20261009`, renewed 2026-10-09.
Owner request, relayed by the appointed manager:
> can we establish all the file types and apends required then document this?

The manager interprets this request as planning authorization for this proposal;
implementation, adoption and the new naming/opening rules remain unratified.
Recommend the dedicated **`.datum-<kind>`** family for user-facing native documents,
with JSON contents. The owner's `.pcb.json`, `.sch.json` and `.fprnt.json` examples
are illustrative, not ratified names. Internal support files retain their formats.

## User-facing native documents

Every row is a proposed suffix, alongside the actual baseline. These are storage
partitions of one resolved DesignModel, not independent competing design authorities.
Existing nested objects stay in their owning document; this proposal creates no
per-Pad, per-Track, per-Net or per-SheetInstance file.

| Document kind | Actual baseline | Proposed suffix and example |
| --- | --- | --- |
| Project manifest / opening entry | `project.json` | `.datum-project` — `amplifier.datum-project` |
| Board root | `board/board.json` | `.datum-board` — `main.datum-board` |
| Schematic root | `schematic/schematic.json` | `.datum-schematic` — `main.datum-schematic` |
| Schematic sheet | Manifest-referenced path under `schematic/`, ending `.json`; schema example `schematic/sheets/<uuid>.json` | `.datum-sheet` — `<uuid>.datum-sheet` |
| Reusable schematic definition | Manifest-referenced path under `schematic/`, ending `.json`; NativeFormat example `schematic/definitions/<uuid>.json` | `.datum-schematic-definition` — `<uuid>.datum-schematic-definition` |
| Rules root | `rules/rules.json` | `.datum-rules` — `design.datum-rules` |
| Project settings documents | NativeFormat describes optional `settings/checking.json`, `settings/output.json`; not a claim of complete implemented settings workflow | `.datum-settings` — `checking.datum-settings`, `output.datum-settings`; embedded role distinguishes checking/output |
| Library Unit | `pool/units/<uuid>.json` | `.datum-unit` — `<uuid>.datum-unit` |
| Library Symbol | `pool/symbols/<uuid>.json` | `.datum-symbol` — `<uuid>.datum-symbol` |
| Library Entity | `pool/entities/<uuid>.json` | `.datum-entity` — `<uuid>.datum-entity` |
| Library Part | `pool/parts/<uuid>.json` | `.datum-part` — `<uuid>.datum-part` |
| Library Package | `pool/packages/<uuid>.json` | `.datum-package` — `<uuid>.datum-package` |
| Library Footprint | `pool/footprints/<uuid>.json` | `.datum-footprint` — `<uuid>.datum-footprint` |
| Library Padstack | `pool/padstacks/<uuid>.json` | `.datum-padstack` — `<uuid>.datum-padstack` |
| Library PinPadMap | `pool/pin_pad_maps/<uuid>.json` | `.datum-pin-pad-map` — `<uuid>.datum-pin-pad-map` |
| Pool opening entry, if established | Current project pool references name directories; no implemented pool-manifest UUID/schema is established here | Reserve `.datum-pool` — `standard.datum-pool`; conditional naming slot, not authorization to introduce a manifest |

Package and Footprint remain distinct types under PM008. A Unit's pins, a Symbol's
graphics, a Footprint's geometry and a Padstack's apertures remain nested records.
Library filenames retain existing UUID basenames; friendly names belong in metadata.
No shipped library, imported content or third-party dependency is approved here.

## Internal authored records, metadata and generated evidence

**No new dedicated suffix** for the following classes. Keeping `.json` here does
not make authored source disposable. The current authority table in
[substrate/mod.rs](../../../crates/engine/src/substrate/mod.rs) distinguishes these
families; they retain canonical resolver, transaction and lifecycle ownership.
Proposed shared self-identification/typed validation applies to internal authored
JSON too: embedded kind/schema and applicable persisted identity must agree with
the resolved ownership context. Folder location alone never establishes type.

| Class / files under project root | Current and proposed retained format | Authority / retention |
| --- | --- | --- |
| `.datum/component_instances/`, `.datum/electrical_identities/`, `.datum/relationships/` | JSON shards, `.json` | AuthoredDesign: component bindings, persistent electrical identities and explicit relationships; preserve stable identities and recorded transitions |
| `.datum/variants/` | JSON shards, `.json` | AuthoredDesign: variant overlays, not an ephemeral display filter |
| `.datum/manufacturing_plans/`, `.datum/panel_projections/`, `.datum/output_jobs/` | JSON shards, `.json` | AuthoredDesign: production intent and configuration; no standalone document split introduced |
| `.datum/import_map/` | JSON shards, `.json` | SidecarMetadata: persistent source-to-native identity bridge; retain for refresh/replay, not a rebuildable visual index |
| `.datum/proposals/`, `.datum/forward_annotation_review/review.json` | JSON, `.json` | SidecarMetadata: proposal/review state under existing guarded ownership |
| Legacy `.ids/<source>.ids.json` / beside-source `.ids.json` descriptions | JSON, `.ids.json` | Historical import identity metadata; acknowledge documented location disagreement, preserve explicit compatibility instead of silently replacing with ImportMap |
| `.datum/output_job_runs/`, `.datum/artifact_runs/`, `.datum/check_runs/` | JSON shards, `.json` | GeneratedEvidence: revision/source-qualified run records; not authored design intent |
| `.datum/zone_fills/` | JSON shards, `.json` | GeneratedEvidence: current-fill basis, provenance and version matter; stale/missing/unsupported never means current empty copper |
| `.datum/artifacts/` | JSON metadata, `.json`; output payload keeps its real format | GeneratedEvidence: artifact provenance/lifecycle metadata distinct from manufacturing plans and jobs |

These rows cover every current SourceShardKind, together with the document table.
Net/Bus identity and correspondence records remain engine-owned source; Run membership,
Zone-region qualifiers, scene indices and exact rotated-Pad query geometry are derived
results, not new persisted design document classes. PM058's scene version 2 is transient
protocol transport, not a native filename suffix.

## Storage infrastructure and local application files

Retain existing formats and internal opening rules. An extensionless file is still
a file type; a directory/container is not a new document suffix.

| Class | Existing / proposed retained representation | Meaning |
| --- | --- | --- |
| Canonical transaction journal and cursor | Transactions `.jsonl`; cursor `.json` under `.datum/journal/` | Commit/replay/undo history and cursor; not disposable logging |
| Revision repository head and generations | `.datum/revision/v1/head.json`, generation JSON | Revision/materialization authority with existing atomic publication and recovery |
| Content-addressed blobs, stage/recovery files and locks | Existing extensionless hash blobs and internal staging/recovery/lock names | Raw preimages/content and write coordination; no document association, schema or cleanup change |
| Pool lookup index | `pool.sqlite`, SQLite | Derived lookup index; never replaces source JSON/document identity |
| Local preferences repository | XDG config `datum/preferences/`: JSON head/generation manifests, `opaque.bin`, backups | Local preferences authority and recovery; retain storage format and existing migration policy |
| Legacy GUI preferences | `datum/gui-preferences.json`, JSON | Explicit legacy preference input, distinct from project settings |
| Terminal contexts / tool sessions / events | Existing JSON context/session records and JSONL event streams | Runtime/session evidence under its existing owner; not a design/library document |
| Credential/runtime state | Existing opaque or runtime-owned representations | Retain credential-provider/runtime handling; no exported native credential document |
| Render, query, search, check and geometry caches | Existing in-memory or owner-defined cache representations | Derived; this proposal adds no cache format, source authority or per-object suffix |
| Vendor model assets | Original bytes with content hash and original extension | Preserve vendor semantics/provenance; validated references do not turn these into Datum-native documents |

## External standards and generated outputs

Keep external names and formats. This inventory describes file classes, **not a
claim that every importer/exporter or model consumer is implemented or authorized**.
Native-first and import-freeze boundaries remain in force except exact existing
amendments; manufacturing capability/refusal and retained CAM/T04 obligations remain.

| Class | Extensions / container convention retained |
| --- | --- |
| KiCad interchange/source | `.kicad_pro`, `.kicad_sch`, `.kicad_pcb`, `.kicad_sym`, `.kicad_mod`; legacy formats stay with their importer |
| Eagle interchange/source | `.sch`, `.brd`, `.lbr` with Eagle content validation; suffix alone cannot distinguish other tools' files |
| Gerber copper / mask / paste / silk / mechanical | `.gbr` or existing layer-specific Gerber suffixes such as `.gtl`/`.gbl`; retain output-job naming |
| Excellon drill | `.drl` / existing drill naming; typed format controls interpretation |
| BOM, placement and reports | `.csv`, `.json`, `.txt`, `.html` as selected by the existing output/report contract |
| Drawings / documents / previews | `.pdf`, `.svg`, `.png`; `.dxf` where supported |
| Mechanical / 3D assets | `.step`/`.stp`, `.wrl` and original referenced asset formats |
| Structured manufacturing exchange | IPC-2581 XML; ODB++ directory/archive conventions where supported; no invented `.datum-*` replacement |
| Electrical/simulation models | IBIS `.ibs`; SPICE `.cir`/`.lib`/`.sub`/`.inc`; Touchstone `.sNp`; AMI parameter/binary bundle formats where supported |
| Output bundles / archives | Existing `.zip`/archive and directory conventions; payload formats and provenance remain separate |

## Opening, validation and compatibility proposed for ratification

1. Use filename association to select an opening route, then validate embedded kind,
   supported schema/profile and typed contents. Add a kind discriminator only through
   a ratified schema change where absent today, including internal authored JSON.
   Malformed content, type/suffix/context disagreement or unsupported future schema
   produces an explicit diagnostic and blocks mutation, never a guessed type.
2. Opening a Project resolves its complete model. Opening a Board, Schematic, Sheet,
   definition or library item resolves the owning Project/pool context. If ownership
   cannot be resolved, provide clearly read-only inspection or explicit attachment;
   do not infer complete electrical authority from an isolated file or invent an owner.
3. Persisted object UUIDs remain identity. Filenames, display names and paths do not
   replace them. Existing Project/Board/Schematic/pool-item UUIDs are preserved;
   do not demand a new UUID for every settings, cursor or support file. A pool UUID
   requires separately specified pool-manifest authority. Read/query/open allocates
   no identities and performs no adoption or migration. Refuse conflicting duplicate
   authored UUID declarations within the resolved ownership domain; lawful repeated
   occurrences/references to one source identity are not duplicate declarations.
4. Retain a documented legacy `.json` profile and existing legacy-version handling.
   Explicit migration updates manifest/root references, pool scanning, source ownership,
   genesis, writers, replay/recovery, provenance and any path-derived shard identity
   coherently. Rename alone is insufficient: current shard IDs derive from relative
   paths, and source/history hashes qualify exact bytes. Object UUIDs remain unchanged.
   Moving a file requires explicit owning-reference repair under the same guarded
   migration/transaction contract; changing its extension alone never changes kind.
5. Define collision handling, partial-failure recovery and old-reader refusal before
   migration implementation. Qualify ordinary create/open/save/reopen and transaction
   undo/replay under both supported profiles, plus relocation, mismatch and failed
   migration. These are future delivery obligations, not tests run or PASS here.
6. Register per-kind desktop associations and filters after ratification. A compound
   alternative such as `.board.datum` is technically possible using full filename
   globs; the single `.datum-board` family is recommended for readable names and
   direct last-extension discrimination. JSON serialization needs no `.json` suffix.
   Full filename globs are supported by the
   [freedesktop shared MIME specification](https://specifications.freedesktop.org/shared-mime-info/latest-single/).

## Evidence review and source conflicts

Reviewed the complete `foundation-consumer-completion` route before drafting: its
[source dossier](../../../research/process-quality/FOUNDATION_CONSUMER_COMPLETION_REPAIR.md)
and all 16 listed consumers in the [traceability manifest](../../../specs/evidence_traceability_manifest.json),
including the foundation plan, gap amendment, derived-authority plan/research,
PM054/056/057/058, manufacturing contract, source-health policy, NativeFormat,
Import, Engine, C03 disposition, exact-30 proposal and delivery receipt. Also reviewed
PM000D and PM008. Current resolver, source-shard, pool-write and genesis audits
provide implementation evidence; no fresh runtime/build proof was requested or run.
The actual suffix/path constraints are in [source_shard.rs](../../../crates/engine/src/substrate/source_shard.rs),
[project_resolver.rs](../../../crates/engine/src/substrate/project_resolver.rs) and
[pool_journal_ops.rs](../../../crates/engine/src/substrate/pool_journal_ops.rs).

Numbered amendments and later receipts control historical pending proposals:
PM054 current-fill/arc authority, PM057 explicit BoardRoot profile 4 adoption and
PM058 exact-query transport do not authorize extension migration. Older NativeFormat
tree/version/locking and import-centric summaries are not complete current behavior.
NativeFormat's `.ids/` layout and Import's beside-source identity layout disagree;
the implemented `.datum/import_map/` family is recorded separately above. The
manufacturing contract's early “OutputJob only authored” summary must be read with
its later Plan/Panel CRUD and the current AuthoredDesign authority table.

Required reconciliation after review: owner disposition of this exact naming/opening/
compatibility proposal; manager-owned classification, traceability/parity/digests and
roadmap synchronization; independent review receipt. No scope reduction, new dependency,
library adoption, production edit, acceptance or successor authorization is asserted.
