# Standards and contract implementation coverage

> Detailed evidence relocated from `specs/PROGRESS.md`. The tracker remains the implementation-status entry point; the generated Active Frontier alone selects work. Dated history below does not grant execution or establish a fresh PASS.

## Standards Audit Batch 1 — Spec Stubs Awaiting Implementation

Spec edits landed in commit `db98eff` (2026-04-17) per the apply order in
`docs/STANDARDS_AUDIT_BATCH_1_GUIDANCE.md`. This section tracks each stub
against its implementation status. Status semantics:
- `[x]` spec/doc text landed in the named anchor
- `[ ]` implementation work not started (no Rust type, no engine op, no
  pool storage, no MCP runtime entry, no importer/exporter, etc.)
- A row may carry both: `[x]` spec landed + `[ ]` implementation pending —
  shown as two columns

### Pass 0 — Standards Compliance Disposition Refresh

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| Domain 1 disposition refresh (STEP/IDF/IDX/EDMD/DXF prerequisites; Gerber X3 / IPC-2581C / IPC-D-356 / ODB++ contracts) | `specs/STANDARDS_COMPLIANCE_SPEC.md` §4.1 | [x] | [—] N/A (disposition text only) |
| Domain 2 disposition refresh (IBIS/Touchstone/SPICE attachment; encrypted-content policy) | `specs/STANDARDS_COMPLIANCE_SPEC.md` §4.2 | [x] | [—] N/A (disposition text only) |

### Pass 1 — Engine Schema Bedrock

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| `ModelRole`, `SpiceDialect`, `EncryptionScheme`, `ModelAttachment`, `ModelProvenance`, `ModelFormatMetadata` (D2-1) | `specs/ENGINE_SPEC.md` §1.1a | [x] | [~] (engine schema is model-backed and validated through typed part behavioural-model authoring; attach command extracts file hash/provenance and promotes source files into `pool/models`; detach removes attachment edges; richer file parsing remains pending) |
| `ModelFormat` enum, typed `Transform3D`, expanded `ModelRef`, `Package.body_height_nm` / `body_height_mounted_nm` (D1-2) | `specs/ENGINE_SPEC.md` §1.1a + §1.2 | [x] | [~] (`ModelFormat`, deterministic typed `Transform3D`, expanded `ModelRef`, and package body-height fields are model-backed; `add-pool-package-model-3d` and `set-pool-package-body-heights` journal typed authoring; model provenance remains an empty slot and richer 3D import/export use remains pending) |
| `StackupLayer` material fields (`dielectric_constant`, `loss_tangent`, `copper_weight_oz`, `roughness_um`, `material_name`) (D1-3) | `specs/ENGINE_SPEC.md` §1.3 | [x] | [~] (engine schema is model-backed with deterministic JSON-number material fields, legacy board JSON deserializes with unset material metadata, and `set-board-stackup` plus `datum.pcb.set_stackup` accept backward-compatible material-aware layer tuples; richer KiCad stackup-material extraction and impedance-solving use remain pending) |
| `Net.controlled_impedance: Option<ImpedanceSpec>` and `ImpedanceSpec` (D1-4) | `specs/ENGINE_SPEC.md` §1.3 | [x] | [~] (engine schema is model-backed with deterministic JSON-number impedance target/tolerance fields, legacy board-net JSON deserializes with no impedance target, and `place-board-net` / `edit-board-net` plus canonical MCP `datum.pcb.place_net` / `edit_net` can author, update, and clear per-net controlled-impedance metadata; impedance solving/export use remains pending) |
| `Part` extensions (`manufacturer_jep106`, `packaging_options`, `behavioural_models`, `thermal`, `supply_chain_offers`, `last_supply_chain_check`) plus `ThermalSpec`, `PackagingKind`, `PackagingOption`, `SupplyOffer` (D2-2) | `specs/ENGINE_SPEC.md` §1.2 | [x] | [~] (`manufacturer_jep106`, `packaging_options`, `behavioural_models`, `thermal`, `supply_chain_offers`, and `last_supply_chain_check` are model-backed and journal-authorable through typed part commands; timestamp is currently serialized as a string rather than a chrono `DateTime<Utc>`) |
| `AttachModel` / `DetachModel` operations with `inverse()` reversibility (D2-4) | `specs/ENGINE_SPEC.md` §3 | [x] | [~] (`attach-pool-part-model` promotes a content-addressed model file and journals typed `AttachPoolPartModel` operations over the part attachment list; `detach-pool-part-model` journals typed `DetachPoolPartModel` operations by attachment/model UUID; undo/redo restores exact attachment arrays while retaining shared content-addressed blobs; richer blob lifecycle policy remains pending) |

### Pass 2 — Pool and Native Persistence

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| `pool/models/{ibis,spice,touchstone,ami,thermal}/` directory; `models` and `part_model_attachments` SQL index tables (D2-3) | `docs/POOL_ARCHITECTURE.md` §2 | [x] | [~] (`attach-pool-part-model` writes content-addressed model files under `pool/models/<role>/`; `project query pool-models`, flat MCP `get_pool_model_blobs`, and canonical `datum.library.pool_models` enumerate blobs, recompute SHA-256, derive deterministic model UUIDs, report part attachment references, and expose `referenced` / `orphaned` lifecycle flags; `project gc-pool-models` provides dry-run/apply cleanup for orphaned regular hash-matching blobs; `project validate` now fails missing provenance-backed blobs, filename/hash mismatches, and bad deterministic model UUIDs; SQL/index tables, richer GC policy, and bundle handling remain pending) |
| `pool/models/` in native project layout; new "Pool Model Files" schema (D2-9) | `specs/NATIVE_FORMAT_SPEC.md` §4 + §6.x | [x] | [~] (native projects can materialize, query, validate, and conservatively garbage-collect orphaned regular `pool/models/<role>/<sha256>.<ext>` blobs via attach/query/validate/gc commands with hash and attachment-integrity verification; migration, AMI bundle hashing, richer orphan lifecycle policy, and full bundle schema remain pending) |

### Pass 3 — MCP API Stubs

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| M7+ Export Tools: `export_step`, `export_idf`, `export_odbpp`, `export_ipc2581`, `import_dxf_outline` (D1-5) | `specs/MCP_API_SPEC.md` "M7+ Export Tools" | [x] | [ ] (not in `tools_catalog_data.py`; would need exporter implementations) |
| Component Modelling Tools: `attach_ibis`, `attach_touchstone`, `attach_spice`, `validate_*`, `extract_*`, `export_spice_netlist`, `lookup_part_*`, `refresh_supply_chain`, `find_alternate_parts`, `query_packaging_options`, `normalize_manufacturer`, `infer_diffpair_from_pinnames` (D2-5) | `specs/MCP_API_SPEC.md` "Component Modelling Tools (M7+)" | [x] | [ ] (not in `tools_catalog_data.py`) |
| Encrypted Content Handling Policy (D2-6) | `specs/MCP_API_SPEC.md` top-level | [x] | [—] N/A (policy framing) |

### Pass 4 — Import Spec

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| IPC-2581 Import (Future — Post-M7) rationale + feature matrix (D1-7) | `specs/IMPORT_SPEC.md` §5 | [x] | [ ] (importer deferred) |
| KiCad and Eagle import matrices: SPICE/IBIS/Touchstone rows promoted Deferred → Best-effort (M7+) with `Part.behavioural_models` mapping note (D2-8) | `specs/IMPORT_SPEC.md` §3 + §4 | [x] | [ ] (importer deferred) |

### Pass 5 — Architecture and Scope Docs

| Stub | Spec anchor | Spec | Impl |
|------|-------------|:----:|:----:|
| Behavioural model attachment subsection (D2-7) | `docs/LIBRARY_ARCHITECTURE.md` | [x] | [—] N/A (architecture text) |
| Interop scope re-organisation — Hard/Should/On-demand/Out-of-scope per Domain 1 (D1-1) | `docs/INTEROP_SCOPE.md` §Future (M5+) | [x] | [—] N/A (scope text) |
| Behavioural model attachment & export scope buckets (D2-10) | `docs/INTEROP_SCOPE.md` new section | [x] | [—] N/A (scope text) |

### Deferred — Held For A Later Batch

| # | Spec target | Why deferred |
|---|-------------|--------------|
| D1-6 | `specs/NATIVE_FORMAT_SPEC.md` §12 or `docs/POOL_ARCHITECTURE.md` | `.gitignore` / `.gitattributes` conventions — descriptive, no contract impact |
| D1-8 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` §10 | "Datum's Open-Stack Position" appendix — marketing position |
| D2-11 | `docs/COMMERCIAL_INTEROP_STRATEGY.md` | "Behavioural Model Stack — Open-Stack Position" appendix — marketing position |

**Standards Audit Batch 1 overall**: [x] spec/doc/policy stubs landed; [~]
implementation is mixed. Model schema, part metadata, stackup materials,
controlled-impedance metadata, package/body/model geometry, part model
attach/detach, and first pool-model blob query/verification are partial.
SQL indexes, AMI bundle handling, blob lifecycle GC, migration, richer parsing,
MCP query aliases, and importer/exporter handlers remain pending.

---

## ENGINE_SPEC.md — Core Types

PM053 / DA-A2-E1: typed source identity, canonical persistence, Bus semantic
distribution and binding diagnostics are partial implementation. Complete graph
producer/automatic topology transitions, adoption coverage and independent E1
review remain pending; no full foundation or S5A acceptance.

### §1.1 Geometry Primitives

| Type | Status |
|------|--------|
| Point | [x] |
| Rect | [x] |
| Polygon | [x] |
| Arc | [x] |
| LayerId | [x] |

### §1.1a Shared Enums

| Type | Status | Notes |
|------|--------|-------|
| PinDirection (10 variants) | [x] | In pool/mod.rs as PinElectricalType on SymbolPin |
| Lifecycle | [x] | In pool/mod.rs |
| StackupLayerType | [x] | In board/mod.rs |
| PortDirection | [x] | In schematic/mod.rs |
| Primitive | [x] | In pool/mod.rs (Symbol graphics) |

### §1.2 Pool Types

| Type | Status |
|------|--------|
| Pin | [x] |
| Unit | [x] |
| Gate | [x] |
| Entity | [x] |
| Pad | [x] |
| Package | [x] |
| PadMapEntry | [x] |
| Part | [x] |

### §1.3 Board Types

| Type | Status |
|------|--------|
| Board | [x] |
| PlacedPackage | [x] |
| Track | [x] |
| Via | [x] |
| Zone | [x] |
| Net | [x] |
| NetClass | [x] |
| Stackup / StackupLayer | [x] |
| Keepout | [x] |
| Dimension | [x] |
| BoardText | [x] |

### §1.4 Schematic Types

| Type | Status |
|------|--------|
| Schematic | [x] |
| Sheet | [x] |
| PlacedSymbol | [x] |
| SheetDefinition | [x] |
| SheetInstance | [x] |
| SchematicWire | [x] |
| Junction | [x] |
| NetLabel / LabelKind | [x] |
| Bus | [x] |
| BusEntry | [x] |
| HierarchicalPort | [x] |
| NoConnectMarker | [x] |
| Variant | [x] |

### §1.5 Rule Types

| Type | Status |
|------|--------|
| Rule | [x] |
| RuleType (7 variants) | [x] |
| RuleParams (7 variants) | [x] |
| RuleScope (leaf nodes) | [x] |
| RuleScope (combinators) | [x] parse, [—] eval deferred to M6 |

### §2 Object Invariants

| Invariant | Status | Notes |
|-----------|--------|-------|
| All authored objects have non-nil UUID | [x] | Enforced by constructors |
| No duplicate UUIDs within type | [x] | `project validate` checks persisted UUID-key consistency and duplicate authored UUIDs across native project object types |
| Non-dangling references | [x] | `project validate` checks required native files plus persisted schematic/board cross-file references and board-side object references |
| Integer coordinates (no float) | [x] | i64 throughout |
| Connectivity recomputed from authored data | [x] | |

### §3 Operations

| Item | Status |
|------|--------|
| Operation trait | [ ] |
| OpDiff | [ ] |
| Transaction | [ ] |
| Undo/redo semantics | [ ] |
| Derived data invalidation | [ ] |

### §5 Engine API

| Method | Status | Notes |
|--------|--------|-------|
| new() | [x] | |
| has_open_project() | [x] | |
| import() | [x] | KiCad skeleton + Eagle .lbr |
| save() | [~] | Writes unmodified imported KiCad boards byte-identically, persists current `delete_track`/`delete_via`/`delete_component` removals, rewrites current `move_component`/`rotate_component` footprint placement and component `Value`/`Reference` properties, rewrites package-backed footprint bodies for current `set_package`, `set_package_with_part`, `replace_component`, `replace_components`, `apply_component_replacement_plan`, `apply_component_replacement_policy`, `apply_scoped_component_replacement_policy`, and `apply_scoped_component_replacement_plan` slices, and writes rule/part-assignment/package-assignment/net-class sidecars for current `set_design_rule`/`assign_part`/`set_package`/`set_package_with_part`/`replace_component`/`replace_components`/`apply_component_replacement_plan`/`apply_component_replacement_policy`/`apply_scoped_component_replacement_policy`/`apply_scoped_component_replacement_plan`/`set_net_class` slice |
| save_to_original() | [~] | Current M3 helper for imported-design write-back to original file |
| search_pool() | [x] | |
| import_eagle_library() | [x] | |
| get_board_summary() | [x] | |
| get_components() | [x] | |
| get_package_change_candidates() | [x] | Current engine API supports component-scoped package compatibility introspection |
| get_part_change_candidates() | [x] | Current engine API supports component-scoped part compatibility introspection |
| get_component_replacement_plan() | [x] | Current engine API supports unified component replacement planning introspection |
| get_scoped_component_replacement_plan() | [x] | Current engine API supports scoped policy-driven replacement preview introspection |
| edit_scoped_component_replacement_plan() | [x] | Current engine API supports scoped replacement preview post-processing via exclusions and explicit compatible target overrides |
| replace_components() | [x] | Current M3 board write slice supports batched explicit component replacement as one transaction / one undo step |
| apply_component_replacement_plan() | [x] | Current M3 board write slice resolves package/part selections from the unified replacement plan and applies them as one transaction |
| apply_component_replacement_policy() | [x] | Current M3 board write slice resolves deterministic best-candidate replacement policies from the unified replacement plan and applies them as one transaction |
| apply_scoped_component_replacement_policy() | [x] | Current M3 board write slice resolves deterministic best-candidate replacement policies over a scoped component filter and applies them as one transaction |
| apply_scoped_component_replacement_plan() | [x] | Current M3 board write slice validates and applies a previously previewed scoped replacement plan without re-resolving policy |
| get_net_info() | [x] | Returns all nets, not single-net |
| get_stackup() | [x] | |
| get_unrouted() | [x] | |
| get_schematic_summary() | [x] | |
| get_sheets() | [x] | |
| get_symbols() | [x] | |
| get_ports() | [x] | |
| get_labels() | [x] | |
| get_buses() | [x] | |
| get_noconnects() | [x] | |
| get_hierarchy() | [x] | |
| get_schematic_net_info() | [x] | Returns all nets, not single-net |
| get_connectivity_diagnostics() | [x] | |
| get_check_report() | [x] | |
| delete_component() | [x] | Current M3 board write slice |
| delete_track() | [x] | Current M3 board write slice |
| delete_via() | [x] | Current M3 board write slice |
| move_component() | [x] | Current M3 board write slice |
| rotate_component() | [x] | Current M3 board write slice |
| set_value() | [x] | Current M3 board write slice |
| set_reference() | [x] | Current M3 board write slice |
| assign_part() | [x] | Current M3 board write slice |
| set_package() | [x] | Current M3 board write slice |
| set_package_with_part() | [x] | Current M3 board write slice |
| set_net_class() | [x] | Current M3 board write slice |
| set_design_rule() | [x] | Current M3 board write slice |
| run_erc_prechecks() | [x] | |
| run_drc() | [x] | Engine API method implemented (connectivity + clearance checks currently) |
| execute() / execute_batch() | [ ] | |
| undo() / redo() | [~] | Current engine API supports transaction reversal for `delete_component`, `delete_track`, `delete_via`, `move_component`, `rotate_component`, `set_value`, `set_reference`, `assign_part`, `set_package`, `set_package_with_part`, `set_net_class`, and `set_design_rule` |

---

## CHECKING_ARCHITECTURE_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| CheckDomain enum (ERC, DRC) | [x] | Implemented as check report domain field |
| CheckSeverity (Error, Warning, Info) | [x] | ErcSeverity in erc/mod.rs |
| CheckSummary | [x] | In api/mod.rs |
| CheckWaiver (domain, target, rationale) | [x] | In schematic/mod.rs |
| WaiverTarget variants | [x] | Object, RuleObject, and RuleObjects matching exercised across ERC/DRC tests |
| Waiver matching in ERC | [x] | Includes authored native-project waivers in existing project ERC/check flows |
| Waiver matching in DRC | [x] | `run_drc` now applies authored DRC waivers and keeps waived findings visible |
| Cross-domain checks excluded from M2 | [x] | Not implemented (correct) |

---

## ERC_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| PinElectricalType enum (10 variants) | [x] | In schematic/mod.rs SymbolPin |
| NetSemanticClass enum | [~] | Partial; power/signal inference exists |
| M2 rule: output_to_output_conflict | [x] | |
| M2 rule: undriven_input | [x] | |
| M2 rule: power_without_source | [x] | |
| M2 rule: noconnect_connected | [x] | |
| M2 rule: unconnected_required_pin | [x] | |
| M2 rule: passive_only_net | [~] | No distinct rule/code ships; passive pins only modulate `input_without_explicit_driver` severity (verified 0 grep hits in `erc/` 2026-06-22). Standalone check is target work. |
| M2 rule: hierarchical_connectivity_mismatch | [x] | Sheet-level hierarchical label/port mismatch check implemented |
| Compatibility matrix (pin-pair) | [~] | Driving analysis exists; full matrix not formalized |
| ErcReport struct | [x] | Via CheckReport |
| ErcViolation struct | [x] | ErcFinding |
| Waiver integration | [x] | |
| Configurable severity | [x] | ErcConfig with BTreeMap overrides |

---

## IMPORT_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| NAMESPACE_KICAD UUID | [x] | ir/ids.rs |
| NAMESPACE_EAGLE UUID | [x] | ir/ids.rs |
| import_uuid() function | [x] | |
| KiCad object path construction | [~] | Skeleton parser; not all object types mapped |
| Eagle object path construction | [x] | Full library mapping |
| .ids.json schema | [x] | import/ids_sidecar.rs |
| .ids.json precedence rules (3 cases) | [x] | restore_or_merge_mappings() |
| KiCad board feature matrix (Required items) | [~] | Skeleton only; geometry not fully parsed |
| KiCad schematic feature matrix (Required items) | [~] | Skeleton only |
| Eagle board feature matrix | [ ] | Design import not implemented |
| Eagle schematic feature matrix | [ ] | Design import not implemented |
| Eagle library feature matrix | [x] | Symbols, packages, devicesets |

---

## SCHEMATIC_CONNECTIVITY_SPEC.md

| Item | Status | Notes |
|------|--------|-------|
| Sheet-local wire connectivity | [x] | Union-find in connectivity/mod.rs |
| Junction semantics | [x] | |
| Local labels | [x] | |
| Global labels | [x] | |
| Hierarchical labels/ports | [~] | Basic support; full hierarchy links partial |
| Power symbols | [x] | |
| Bus/member expansion | [x] | Basic imported KiCad subset now supports deterministic `NAME[n]` scalar member normalization, `NAME[a..b]` bus-range expansion, and geometric bus-entry association; advanced syntax edge cases remain deferred |
| Net naming and identity | [x] | |
| Connectivity diagnostics | [x] | |
| Deterministic graph output | [x] | Sorted by UUID |

---

## SCHEMATIC_EDITOR_SPEC.md — M4 Operations

Status: [x] Closed for scoped M4 slice
- Verified by code audit in `crates/cli/src/cli_args.rs`,
  `crates/cli/src/command_exec.rs`, and dedicated CLI test shards:
  `main_tests_project_symbol*.rs`, `main_tests_project_label.rs`,
  `main_tests_project_wire.rs`, `main_tests_project_junction.rs`,
  `main_tests_project_port.rs`, `main_tests_project_bus.rs`,
  and `main_tests_project_noconnect.rs`.
- Implemented schematic operation families include symbol
  place/move/rotate/mirror/delete, symbol field add/edit/delete,
  label place/rename/delete, wire draw/delete, junction place/delete,
  hierarchical port place/edit/delete, bus create/edit/place-entry/delete-entry,
  and no-connect place/delete.
- Remaining editor expansion items such as power-symbol placement,
  sheet-instance lifecycle operations, and deterministic annotate flow
  completion are deferred beyond scoped M4 closure.

---

## NATIVE_FORMAT_SPEC.md

Status: [x] Closed for scoped M4 slice
- PM061 bounded optional process implementation landed in `ef32fbe1`, with
  [source-pinned delivery/availability](../../docs/reviews/workflow-delivery-rollout/effective-process-owner-delegation.json). Standards, unsupported CAM, seed rights, HUMAN and full qualification remain open.
- PM053 / DA-A2-E1 adds optional schema-1 electrical identity partitions and
  optional placed-pin library references. Canonical source/history proof is
  bounded; adoption and complete derived authority remain unfinished.
- Native project scaffold, deterministic file layout, and first native
  read/query/check surfaces are implemented in the current M4 slice.
- Remaining native-format contract areas (full schema coverage, migration
  completeness, and richer manufacturing/output semantics remain open beyond
  scoped M4 closure.

---
