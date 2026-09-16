# Global Preferences production acceptance contract

Status: GP-CM05E implementation contract under the exact owner-approved
Preferences promotion e6367c62a94c8a0480c842c4f3afe6f1bdb62b64.
Implementation and proof remain in progress; production is not accepted.

Owner: GLOBAL-PREFERENCES-COMPLETION / `dat-global-preferences-completion-f84`.
Authority: PM-037, PM-038, PM-039, PM-040, the active descriptor catalog,
GP-C04 storage/recovery, GP-C05 interaction, and the product-surface contract.
The machine inventory is `global_preferences_production_acceptance_matrix.json`.

## 1. Deliverable and preserved boundaries

The deliverable is a reliable, recoverable, supportable local Linux Global
Preferences feature. An implementing agent must satisfy every requirement below
before presenting a production-acceptance packet. These requirements establish
engineering quality for this feature, not enterprise deployment features or a
certification claim for the whole application.

Preserve exactly 11 active descriptors, two sections, 45 reserved candidates,
eight individual Units seeds, Global and explicit-factory genesis, no live
Project following, one product service, and one preference writer. MCP remains
proposal-only for mutations. Appearance has live consumers; Units affects only
future Projects. Project and Publish/document authority remain separate.

Organization management, remote policy, synchronization, accounts, credential
storage, persistent unattended authority, Manage Preferences, broader Project
Preferences, aggregate seed activation, guided setup, Start-page activation,
Publish, Revision, new dependencies, and prototype changes remain excluded.
Engine backup/migration proof does not imply a shipped management UI. Existing
accepted Units behavior remains accepted within its original scope; the new
release must demonstrate that integration preserves it.

## 2. Entry and completion states

The recorded GP-CM05V revise/implement direction and exact admission in
`docs/reviews/preferences/gp-cm05-implementation/owner-direction.md` authorize
GP-CM05E. Earlier GP-CM05 evidence remains historical, with three unproved GUI
budgets. The preserved draft's proposed R/RV steps were never installed and
are not additional approval requirements. No historical completion is undone.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM05E -->
GP-CM05E implements the evidence checks and capture paths, resolves in-scope
failures, runs the complete corpus, and assembles a reviewed candidate. The
agent must not mark it complete while any required evidence is missing,
pending, skipped, unsupported on a claimed platform, stale, or failing.

<!-- REQ:GLOBAL-PREFERENCES-COMPLETION:GP-CM05A -->
<!-- OWNER:GLOBAL-PREFERENCES-COMPLETION:GP-CM05A:GP-CM05A -->
GP-CM05A is the final owner review of the exact candidate, report digest,
independent review, native observations, budgets and exclusions. Reply
`GLOBAL-PREFERENCES-COMPLETION: accept GP-CM05E <revision> <report-sha256>` or
`GLOBAL-PREFERENCES-COMPLETION: revise GP-CM05E — <specific correction>`.
The receipt records the actual response and its provenance. An agent cannot
manufacture acceptance by setting a Boolean or selecting its own candidate.

`partial` evidence is useful diagnostic output and can never pass readiness.
`candidate_ready` requires sections 3–9 to pass, with owner acceptance absent.
`production_accepted` additionally requires the matching owner receipt and
closure transaction. These states are computed, never trusted input labels.

## 3. Candidate and evidence identity

Use a clean, committed candidate. Record its full Git commit and tree IDs,
tracked input manifest, Cargo.lock digest, rustc/Cargo/Python versions, exact
build commands, executable SHA-256 values, release profile, and environment
record. The input manifest includes production source, build configuration,
fixtures, scripts, relevant contracts and matrix. Each entry is a normalized
repository-relative path plus SHA-256; paths must be unique and sorted.

Fixture identity includes every initial file and its bytes, explicit Project
and request IDs, source generations, injected fault, and expected outcome.
Expected results are specified before running the candidate. The tested program
cannot generate an expected golden from its own observed output.

Each result binds the same candidate, input-manifest digest, matrix digest,
environment ID, binary digest and fixture digest. Reject any mismatch, missing
artifact, changed artifact, unknown schema, duplicate JSON key or non-finite
number. Artifact paths cannot be absolute, escape the bundle, or resolve through
symlinks. Failure to parse evidence is a gate failure, never an empty result.

Evidence lives outside the candidate's input closure. A report does not hash
itself. Its canonical digest is SHA-256 over UTF-8 JSON with sorted keys,
compact separators, `ensure_ascii=False`, and `allow_nan=False`. The separate
owner receipt references that digest. Artifact digests hash exact file bytes.
The report does not contain the owner receipt or a mutable acceptance flag.

Proof commits may add evidence and governance after the tested commit. At final
publication, enumerate and review the complete delta from tested candidate to
publication: only report/artifact additions and necessary governance may differ.
Any product, test, fixture, dependency, build, checker, matrix or contract change
invalidates readiness and requires a new candidate and complete proof. No
agent-selected equivalence, stale HEAD comparison, or silent carry-forward.
The publication delta is a separate reviewed artifact, avoiding circular hashes.

## 4. Closed report shapes and result accounting

`ProductionReportV2` has exactly: `schema`, `origin`, `candidate`, `matrix_sha256`,
`input_manifest`, `environments`, `artifacts`, `case_results`, `measurements`,
`gate_results`, `native_observations`, `independent_review`, `release_notes`.
Schema is `datum-global-preferences-production-evidence-v2`.

`candidate` has `revision`, `tree`, `cargo_lock_sha256`, `toolchains`,
`build_commands`, `binaries`; revisions are full 40-hex Git IDs and hashes are
64 lowercase hex characters. `toolchains` maps rustc, cargo and python to their
captured version strings. Build commands are retained command observations
with exact argv and output references, as specified in section 11. `binaries`
maps the six executed surface roles to their artifact references.
An artifact reference is exactly `{path, sha256}`. `input_manifest` is such a
reference to the sorted input entries defined in section 3.

`artifacts` is the unique inventory of every referenced log, raw sample set,
capture, fixture manifest, build log and review. References must resolve into
that inventory. `environments` contains records with exactly `id`, `os`,
`architecture`, `kernel`, `cpu`, `memory_mib`, `filesystem`, `mount_options`,
`storage_device`, `gpu`, `driver`, `display_backend`, `compositor_or_server`,
`scale_factor`, `window_size`, `locale`, `network_state`, `power_mode`, `background_load`,
`clock`, `tool_versions`. Nonapplicable GUI fields use null only for CLI runs;
unknown or unavailable required environment facts block readiness.

`case_results` contains exactly one passing result for every expanded matrix
case/variant/subcase/surface/backend/scale/logical-size combination (389 rows). Each result has exactly `case_id`,
`variant_id`, `subcase_id`, `surface`, `environment_id`, `candidate_revision`, `binary_sha256`,
`input_manifest_sha256`, `matrix_sha256`, `fixture`, `command`, `exit_code`,
`executed_tests`, `assertions`, `before`, `after`, `log`.
Commands are argv arrays; test counts are positive integers; exit code is zero.
Assertions are nonempty `{id, expected, observed, evidence}` records: IDs are
the matrix's required assertions, expected values match the reviewed fixture,
observations satisfy them and `evidence` references retained artifacts. `before`
and `after` reference exact state manifests, including absence where expected.
Running zero matching tests or merely locating a function name is failure.

The same real execution may satisfy multiple rows only through explicit
references and separate assertions for each row. Never duplicate one result as
different execution evidence. Backend-independent engine cases run once;
native interaction cases run on both required display backends. Unit tests
support native observations but cannot replace them.

`gate_results` entries have exactly `id`, `candidate_revision`,
`input_manifest_sha256`, `commands`. Each command observation has `command`,
`candidate_revision`, `input_manifest_sha256`, `exit_code`, `log`; the ID set must equal section 8 except `evidence`. The evidence validator
runs over the assembled report and returns its verdict separately, avoiding
a self-referential gate result.
`native_observations` reference the section 6 scenario IDs and contain
`scenario_id`, `environment_id`, `candidate_revision`, `binary_sha256`,
`input_manifest_sha256`, `matrix_sha256`, `fixture`, `actions`, `assertions`, `capture`, `accessibility_capture`, `state_evidence`.
Actions are ordered user inputs; assertions follow the case-result shape.

`independent_review` and `release_notes` are artifact references with contents
specified in section 9. The independent review references candidate and input
identities, not the containing report digest, preventing a circular reference.
Identity inventories reject duplicates; equal sample values and repeated user actions are legitimate. Unknown fields, wrong types, Boolean-as-number,
empty required arrays, missing IDs, unrecognized IDs and skipped results fail.

## 5. Complete behavioral corpus

The matrix retains all fifteen durable case IDs and their existing test anchors.
Each row's new `variants`, `surfaces`, and `assertions` are mandatory coverage,
not suggested tests. Existing functions are evidence locators, not a definition
of sufficient proof. New tests may split ownership without reducing coverage.

Use disposable private configuration roots on durable local storage, never
the owner's real preferences or Projects. Pin known future opaque bytes and
deterministic Project identities. Check exact filesystem/journal preservation,
not only successful return values. Every mutation/refusal also proves absence
of writes outside its authorized destination. Fixtures for managed/offline
state remain injected test inputs; they cannot activate a managed provider.

Fault tests cover preference staging, file flush, generation publication,
head replacement and parent-directory flush, plus all five existing genesis
checkpoints. Include I/O error, interrupted process and response-loss variants.
Distinguish process-crash proof from physical power-loss certification; claim
only the filesystem durability contract actually exercised. On failure, the
observable authority is the old complete generation or the new complete
generation/Project, never partial truth or a false durable acknowledgement.

Test before/after absence, restart recovery, immutable receipt replay and
surviving unknown bytes. Include denied permissions, disk-full injection,
unsupported locking/atomic publication, stale previews, competing processes,
owned-stage cleanup and symlink refusal without sweeping unrelated files.
Recovery instructions must use implemented operations; no hidden repair writer.

Negative API vectors include missing fields, malformed JSON, wrong types,
unknown fields, unsupported versions, reserved keys, invalid values and every
proposal/acceptance refusal. Engine, CLI and MCP must agree on symbolic code
and disclosure-safe fields. `dat-preferences-unknown-field-code-ccc` must be
resolved before readiness: the supported-schema contract selects `invalid_request` for unknown
fields in a supported schema and `unsupported_schema_version` for an unknown
schema name/version. Trusted actor fields never become accepted payload inputs.

## 6. Native behavior and accessibility

Require real Linux Wayland and X11 sessions, each with a named compositor/server
and GPU driver. A simulated scene, headless renderer or screenshot alone cannot
prove native window ownership, focus, interaction or accessibility transport.
Missing access to either backend yields incomplete evidence, never a waiver.

Run these six scenarios on each backend at scale 1.0 and 2.0, using 1024×900
and 720×760 logical-pixel Preferences windows. Record actual dimensions; inability
to exercise a required size is a failure, not assumed equivalence:

| ID | User sequence and required observation |
|---|---|
| N01 | Ordinary Edit > Preferences > Global Preferences doorway; one owned interactive window; close/Escape returns focus to the invoking surface; reopen preserves durable values and leaves no orphan. |
| N02 | Keyboard-only section navigation, all 11 controls, search by label/description/key/each registered alias, zero results, clear, result-to-row navigation, explanation open/close; correct counts, one focus indicator, no stale explanation or focus trap. |
| N03 | Appearance Set/Reset and invalid/stale/competing-writer refusals; real declared consumer changes at the specified time; retained draft, explanation, reset target and last valid value remain truthful. |
| N04 | Create Global and factory Projects; eight-value preview and resulting receipt agree; corrupt/migration-required Global state retains name/location and focus; no automatic factory fallback; later Global changes leave both Projects unchanged. |
| N05 | Reduced motion on/off and high-contrast/non-color on/off; text, shape, controls and provenance remain complete at both widths/scales; no hidden required information or motion-dependent action. |
| N06 | Capture the actual AT-SPI tree and events while executing N01–N05: names, roles, values, enabled/read-only states, traversal, provenance, polite updates, assertive refusals and explicit focus return. Run the pinned screen reader and record spoken output for these transitions. |

Name existing tool versions in the environment. Do not fetch a new dependency
to obtain a missing tool; obtain separate dependency authority when necessary.
Any visual discrepancy requiring a prototype change returns to Claude with the
exact file/region, required outcome, preserved decisions and expected capture.
Codex cannot refresh visual authority to make its runtime output pass.

## 7. Reproducible resource measurements

Retain the existing eight numeric budget families without raising any limit.
Owner review covers the exact matrix and this protocol together. Budgets are
qualified against the recorded Linux/x86_64 durable-storage reference host;
they do not promise the same timings on arbitrary hardware. Record CPU, storage,
GPU, power mode and competing load before measurement. No concurrent builds or
other acceptance jobs; inability to control the environment is reported.

For each timed variant: run 10 warm-up operations, then 100 measured operations
in each of three complete trials. Retain every observation. Each trial must
pass individually; do not average trials or pool faster variants/backends to
hide a failure. Sort the 100 elapsed values and use nearest-rank p95, index
`ceil(0.95*N)-1`. Compare unrounded values. The maximum includes every measured
operation. A failed operation, timeout, missing sample or non-finite value
fails the trial. No outlier removal, successful-retry substitution, or relabeling
failed product runs as environmental failures. Preserve failures and their
diagnosis; after a correction, rerun the complete candidate proof.

Each measurement has exactly `budget_id`, `variant_id`, `environment_id`,
`candidate_revision`, `binary_sha256`, `input_manifest_sha256`, `matrix_sha256`,
`trials`. A trial has `index`, `warmup_count`, `warmups`, `baseline_rss_kib`, `baseline_resources`,
`cold_open`, `samples`, `log`. Closed sample schemas appear in section 11. CLI RSS uses process
`wait4` high-water accounting; GUI RSS uses the long-lived process and descendants.
Use monotonic timestamps. A non-timed storage/lifecycle sample substitutes its
named numeric observations below for elapsed time; no fabricated elapsed value.
Derived summaries are output, never trusted inputs to the checker.

| Budget | Required variants and measurement boundary |
|---|---|
| release-query | `describe`, `list`, `get`, `search`, `explain`, `preview-project-units-seed`; defaults-only and populated stores; process start through parsed successful response and exit. Each process ≤32 MiB RSS; p95 ≤50 ms and maximum ≤100 ms. |
| durable-mutation | Set and Reset separately; standalone and reachable-daemon paths; actual foreground-TTY confirmation. Measure submission of automated APPLY through durable response/exit; retain launch-to-response separately. CLI and daemon each ≤32 MiB RSS; p95 ≤100 ms and maximum ≤250 ms. |
| project-genesis | factory, missing-store Global, populated-store Global; unique destinations and real eight-item receipts; process start through durable publication/response/exit. ≤32 MiB RSS; p95 ≤100 ms and maximum ≤250 ms per variant. |
| project-validation | Each genesis-mode fixture; new release process through resolver result/exit. ≤32 MiB RSS; p95 ≤50 ms and maximum ≤100 ms. |
| storage-growth | Every created empty Project ≤16 KiB logical file bytes. After each of 100 durable mutations, each individual new immutable generation, receipt and request-index increment together ≤32 KiB; count fixed repository overhead separately. No average may hide an oversized generation. Retain per-file before/after manifests. |
| gui-feedback | Search update/clear, focus movement, explanation open/close, non-durable navigation; each backend and scale, ordinary and narrow width. Input dispatch through prepared state; p95 ≤50 ms, maximum ≤100 ms. Also record presented-frame acknowledgement, bounded by the same maximum, to expose a stalled render loop; prepared state alone never proves native usability. |
| native-window-open | Menu activation through first presented frame with accessible controls and successful focus/input round trip. Both backends/scales; p95 ≤500 ms, maximum ≤1000 ms. Ten warm opens, then 100 opens per trial. Record the first cold open separately with the same 1000 ms maximum. |
| window-lifecycle | Each backend/scale: three trials of 100 open/close cycles after 10 warm-up cycles; record each cycle's post-close RSS, owned-window count and lease ownership. Final and peak post-close growth from warmed baseline ≤4 MiB; zero orphan windows and zero orphan writer leases. A persistent daemon baseline is recorded and is not an orphan. |

On durable writes, the stopwatch must include validation, queue/IPC wait,
locking, write, fsync and acknowledgement; do not stop at scheduling the write.
User thinking time is excluded only by the declared automatic APPLY boundary.
Record IPC and GUI storage paths so work is not moved into an unmeasured process.
Configuration growth is deliberately linear in retained immutable history;
this contract does not authorize silent pruning or invent a retention policy.

## 8. Mandatory gates and fail-closed enforcement

The final acceptance command must run the complete corpus and require all
standing gates. Full-workspace tests and strict Clippy are mandatory, not
optional flags in a command that can print a production-ready verdict.
Diagnostic/subset commands must identify themselves as incomplete and cannot
produce a readiness result. Recording owner acceptance is a separate operation.

Required gate IDs, mapped to their current scripts/commands:

| ID | Required check |
|---|---|
| matrix | `check_global_preferences_production_matrix.py` |
| evidence | `check_global_preferences_production_evidence.py --ready` |
| boundary | `check_global_preferences_boundary.py` |
| menu | `check_menu_model.py` |
| resolver | `check_resolver_raw_loads.py` |
| daemon | `check_daemon_write_parity.py` |
| private-writer | `check_schematic_private_writers.py` plus the preference boundary writer inventory |
| mcp | `check_mcp_public_taxonomy.py` |
| dependencies | `check_dependency_authority.py` |
| cargo-resources | `check_cargo_resource_policy.py` |
| source-health | `check_source_health.py` and source-health policy-integrity regressions |
| governance | `check_spec_governance.py` |
| parity | `check_spec_parity.py` |
| traceability | `check_evidence_traceability.py` |
| progress | `check_progress_coverage.py` |
| alignment | `check_alignment.py` |
| project-state | `project_status.py check` and `check-render` |
| workspace-tests | `cargo test --workspace --all-targets --locked --offline` |
| clippy | `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` |
| checker-regressions | Exact negative evidence tests described below |

All Cargo proof commands use `run_cargo_guarded.py --workload proof --` serially.
Build artifacts stay on project-backed storage. Tests use no new dependency.
Never disable installed workflow checks or alter authority to make proof pass.

Checker regressions must reject: each omitted case/variant/assertion/gate/budget;
duplicate and unknown IDs; zero executed tests; skipped/failing results; stale
revision/binary/input/matrix/fixture/artifact digest; changed proof inputs after
capture; invalid JSON, NaN/Infinity, wrong types and missing files; traversal and
symlink artifact paths; fabricated pass labels without samples; too few samples;
percentile off-by-one; one failing trial masked by pooled results; each numeric
limit exceeded independently; missing Wayland/X11 or AT-SPI observations;
missing review; stale or absent owner receipt in acceptance mode (readiness requires no receipt); historical V1 report presented as
V2. A complete synthetic test bundle must pass, clearly labeled test-only and
never eligible as product evidence. Test both CLI exit behavior and diagnostics.

## 9. Independent review and release handoff

A reviewer other than the producing session checks requirement-to-result
coverage, raw measurements, fixture assertions, all eight budgets, native
behavior and the independent PM-037/PM-039 exclusions. The review artifact
records reviewer/session, candidate/input identities, exact replay commands,
result/artifact references and finding dispositions. The reviewer must rerun
the evidence checker and representative durability, security and native cases
from the retained bundle. Copying the producer's pass labels is insufficient.
No unresolved in-scope failure or waiver is compatible with readiness.

The release notes/support packet names the qualified environment and build,
11 active controls, future-Project Units timing, settings locations, read-only
diagnostic commands and stable error meanings. Include ordinary new-user launch,
open/close/reopen, upgrade/downgrade, corrupt-store preservation, explicit-factory
escape from unusable Global state, and escalation instructions that preserve
damaged bytes. Document implemented recovery capabilities and unavailable UI
honestly. Support examples must not require deleting state, editing manifests,
granting agent authority, or sending private settings off-machine.

Bind the distributed CLI/GUI/daemon artifacts to the tested hashes and record
their offline launch on the named clean-user environment. If packaging changes
executables or bundled assets, test that resulting artifact before acceptance.
No installer, signing service or dependency is introduced by this requirement.
The handoff states supported local storage, Wayland/X11 scope, data compatibility,
retained-history growth, known exclusions and the exact accepted report.

## 10. Agent implementation handoff

Under the installed GP-CM05E authorization, implement in this order:

1. Build V2 closed-shape parsing, identity/artifact validation and negative tests
   alongside the existing historical V1 reader. V1 remains inspectable but
   cannot produce a V2 readiness or acceptance verdict.
2. Expand matrix-driven execution to assert actual test discovery/results and
   every variant. Include `datum-gui-render` responsive/non-color tests; the
   current focused runner does not run that package. Retain per-case logs.
3. Upgrade release measurements to raw samples and the exact protocol; add
   native Wayland/X11 capture and lifecycle accounting through existing product
   paths. Tests must observe actual behavior rather than set synthetic pass flags.
4. Resolve the unknown-field code inconsistency and any demonstrated in-scope
   failures through the same product authority. Add cross-surface negative
   vectors. A needed new mechanism, dependency or visual change returns to its
   existing owner; it cannot be inferred from this quality contract.
5. Pin the final candidate, run the full proof serially, obtain independent
   replay/review, assemble support notes and present the exact GP-CM05A packet.

The specification is complete when these obligations are discoverable,
consistent, classified and scheduled. The implementation is complete only when
its actual evidence satisfies them. A specification commit closes neither the
product issue nor any still-unproved acceptance case.

## 11. V2 implementation schemas and replay rules

These closed schemas resolve ambiguities in the preserved proposal; they do
not activate a new product mechanism. The validator package is
`scripts/global_preferences_acceptance/`. No permissive fallback or historical
report conversion is a V2 proof path. JSON rejects duplicate keys, non-finite
numbers, lone surrogates, extra fields and Boolean numeric substitutions.
Canonical JSON uses UTF-8, sorted object keys, compact separators, no ASCII
escaping and no trailing newline. Artifact hashes cover the original bytes.

### Capture provenance, fixtures and native observations

`origin` is exactly `{kind, producer_session}`. `kind` is `product-execution`
for actual runs or `synthetic-test` for regression fixtures. Synthetic validation
requires `--test-only` and returns `synthetic_validated`; it cannot return
readiness or acceptance. A label is not proof of execution: independent review
must inspect the committed capture recipe and replay it. Hash binding detects
substitution, not a dishonest producer or invented screen-reader transcript.

`candidate.binaries` has exactly `cli`, `gui`, `daemon`, `engine`, `gui-render`,
and `mcp` artifact references. The first three are the distributed release
executables; engine and gui-render are the executed test binaries; mcp is the
executed Python interpreter with the candidate MCP source in the input closure.
Case surfaces select the identically named binary role. Headless measurements
select CLI; native measurements and scenarios select GUI. A digest from a
different role cannot substitute. `build_commands` contains the complete
release-build command observation, in the same shape as each gate command;
its exact guarded argv is maintained with the mandatory gates in `gates.py`.
A passing command with no release build cannot satisfy this requirement.

The input closure includes all tracked files beneath Cargo.toml, Cargo.lock,
.cargo, crates, mcp-server, scripts, CLAUDE.md, AGENTS.md, the traceability
manifest, and every source and consumer of the owning route. Compare each
entry with both committed bytes and current bytes/executable mode. Reject new
untracked inputs, removed inputs, symlinks and Git identity overrides. The
matrix argument must equal the current, candidate-bound matrix, not a separate
producer-selected substitute. The independently selected
`--environment-sha256` is the canonical digest of the complete environment
array recorded before trials; carry it unchanged into replay and owner review.
`window_size` is the actual two-integer logical size. Require one headless
record and the eight distinct backend/scale/size records, with no duplicates.

A fixture is exactly `{schema, source_revision, recipe, parameters,
expected_assertions, initial_state}`, schema `datum.preferences.fixture.v2`.
`recipe` is a path/digest entry in the candidate input manifest. Parameters
include `coverage`, the case identity `[case_id, variant_id, subcase_id,
surface, coordinate]` or native identity `[scenario_id, coordinate]`.
Coordinate is null for nonnative cases or `[backend, scale, [width, height]]`.
Each required assertion is an affirmative property computed by that reviewed
recipe; its frozen predicate is exactly `{operator: "equal", value: true}`.
The producer cannot weaken an expectation by choosing a false golden.

A state manifest has `{schema, root, files}`, schema
`datum.preferences.state.v2`. Files are sorted unique `{path, kind, bytes,
sha256}` entries; `kind` is `file` or `absent`. Absence has zero bytes and null
digest. Root names identify the captured fixture, not an instruction to read or
write an arbitrary path. Captures must retain exact damaged/opaque bytes as
artifacts where assertions require byte preservation.

Raw assertion artifacts contain `{schema, context, id, observed}`, schema
`datum.preferences.assertion.v2`. Context contains exactly the row's
`candidate_revision`, `input_manifest_sha256`, `matrix_sha256`, `binary_sha256`,
`environment_id`, and `fixture`. This binds repeated property names to an
exact frozen execution rather than allowing one true flag across environments.

A native capture reference resolves to `{schema, context, kind, actions, tool,
payload}`, schema `datum.preferences.native-capture.v2`. Context is as above;
kind is `native` or `at-spi`; tool names the actual recorder/version; actions
match the ordered row actions; payload references the retained capture bytes.
AT-SPI payloads include the tree/events and screen-reader spoken-output record.
Every N01–N06 scenario runs at all eight coordinates: 48 observations.
Capture manifests cannot be relabeled across those coordinates. A nonempty
payload alone does not demonstrate usability: replay and visual/accessibility
review remain mandatory and no prototype authority is changed here.

Every case command log is `{schema, command, exit_code, executed_tests, stdout,
stderr, candidate_revision, input_manifest_sha256}`, schema
`datum.preferences.command-observation.v2`. Stdout/stderr are artifact
references retained even on failure. Case stdout is a closed
`{schema, executed_tests, observations}` object with schema
`datum.preferences.case-observations.v2` and observations mapping exact assertion
IDs to computed values. Counts must be positive and agree with the row/log.
Mandatory test gates additionally parse completed Rust/unittest summaries;
listing test names or executing zero tests does not pass.

### Expanded faults and measurements

The 75 named variants expand to 389 case rows. `preference-write-faults`
expands to all fifteen checkpoint/fault pairs: generation-staging, file-flush,
generation-publication, head-replacement, parent-directory-flush, each with
io-error, process-interruption and response-loss. The process-killed genesis
variant expands to staging-prepared, project-built, project-validated,
staging-synced and project-published. Other variants use `default` subcase.

There are 90 measurement rows: 12 query variants (six verbs × defaults/populated),
four mutation variants (set/reset × standalone/daemon), three genesis and three
validation variants (factory/global-missing/global-populated), four storage
variants (those three plus generation), 48 GUI-feedback variants (six actions
× eight coordinates), eight window-open and eight lifecycle variants.
`inventory.py` carries the exact action and variant spellings checked against
this contract and the matrix; no other variant substitutes.

A trial has exactly `{index, warmup_count, warmups, baseline_rss_kib,
baseline_resources, cold_open, samples, log}`. Three trials have indices 0–2;
each retains ten warm-ups indexed 0–9 and 100 measured samples indexed 0–99.
The raw trial log contains the same fields except `log`, plus
`schema: "datum.preferences.measurement-trial.v2"` and `measurement` (the exact
measurement row without trials). Thus a complete raw trial cannot be relabeled
for another binary, candidate, variant or display configuration.

Every sample has `kind`, `index`, `started_monotonic_ns`, `outcome`, `evidence`.
Outcome must be success; start timestamps are positive and strictly ordered
through cold-open, warm-ups and measured operations and across all three trials.
Warm-ups have the same complete typed shape as measured operations; their
latency limits are excluded, but failed operations and leaked resources fail. Equal measured values are
permitted. Add exactly these fields according to kind:

- `timed`: `elapsed_ns`, `rss_kib`, `daemon_rss_kib`,
  `launch_to_response_ns`, `presented_elapsed_ns`. Nonapplicable fields are
  null. Daemon mutations require separate daemon RSS. Mutation launch interval
  includes the measured durable interval. Feedback presentation follows state
  preparation; first presentation precedes completion of the window-open
  accessibility/input round trip. Cold-open is a separate timed sample with
  index zero and the same maximum, present only for window-open trials.
- `storage`: `project_bytes`, `generation_bytes`, `receipt_bytes`,
  `request_index_bytes`, `fixed_overhead_bytes`, `before`, `after`.
  Every present file must have retained bytes in the artifact inventory with
  matching digest and exact length. State manifests use virtual prefixes project/, generations/, receipts/,
  request-index/, fixed/ to classify actual captured files. Compute every byte
  delta from these manifests, reject rewrites/removal of immutable retained
  files, require a nonempty generation addition for a mutation,
  and apply limits to each sample independently. Receipts and request indexes
  embedded in the generation manifest are counted once in generation bytes;
  their separate-file byte counts are zero. Each mutation before-state must
  equal the preceding after-state through warm-ups and measured samples;
  distinct trials may use fresh disposable repositories. Fixed overhead is separate.
- `lifecycle`: `post_close_rss_kib`, `orphan_windows`, `orphan_writer_leases`,
  `owned_windows`, `writer_leases`. Resource entries are `{identity, owner}`.
  `baseline_resources` holds both inventories captured after warm-up, and
  every post-close inventory must equal it. Baseline RSS and resources equal
  the final warmed observation. An invented `baseline:true` field
  is invalid. Only lifecycle trials have nonnull baseline resources/RSS.

### Independent review, release and owner receipt

Independent review is exactly `{schema, producer_session, reviewer_session,
review_subject_sha256, candidate_revision, input_manifest_sha256, matrix_sha256, replay,
finding_dispositions, exclusions}`, schema
`datum.preferences.independent-review.v2`. Producer matches origin; reviewer
must be a different actual session. The review-subject digest covers the canonical report except `artifacts` and
`independent_review`: all included proof roots transitively bind their artifact
bytes by digest, while review-only additions cannot form a hash cycle. The
review record and independent `--review` verdict must match this exact subject;
changed observations under the same candidate invalidate review. All findings are unique
`{id, disposition, evidence}` with disposition resolved, never waived.
Exclusions equal the product boundary. Session strings are audit identifiers;
reviewer independence is also a governance/owner verification obligation.

Each replay is `{category, command, exit_code, log, coverage}`. Require exactly
one evidence, durability, security and native replay. Evidence uses the
checker `--review` with exact candidate and environment selection. This mode
checks the complete corpus, measurements, gates and release handoff before
review, returning only `reviewable`, so independent review has no circular
requirement to approve itself. Durability and security select and rerun an
actual retained case command in their required families, with exact coverage
and positive assertion output. Native selects a scenario/environment and runs
the candidate-bound fixture recipe with `--native-scenario` and `--environment`;
its `datum.preferences.native-replay.v2` output contains exact candidate,
input and matrix identities, coverage, capture and accessibility_capture
references. All replay logs use the command-observation schema. `true` or a
nonempty arbitrary log cannot stand in for replay.

Release handoff is `{schema, candidate_revision, input_manifest_sha256,
environment_ids, binaries, boundaries, support, clean_user_launch}`, schema
`datum.preferences.release-handoff.v2`. Environments, binaries and boundaries
match the report exactly. Support maps settings_locations, diagnostics, errors,
recovery, compatibility, storage, retained_history and known_exclusions to
nonempty artifacts. Clean-user launch is retained evidence, not an unchecked
label. Independent review checks these instructions against implemented paths.

Owner receipt is `{schema, candidate_revision, report_sha256, disposition,
response, provenance, publication_delta}`, schema
`datum.preferences.owner-acceptance.v2`. Provenance has source and recorded_at.
Disposition and response must be the exact acceptance in section 2. The caller
must supply the separately selected receipt SHA-256; accepting recomputes full
readiness instead of trusting a passed-in state. Publication delta is
`{candidate, publication, changed_paths, review}`. Actual Git ancestry and
changes must agree: only modifications to .beads/issues.jsonl,
specs/active_frontier.json and specs/PROGRESS.md, or additions under
specs/evidence/preferences/ and docs/reviews/preferences/ are eligible. The
separately receipt-bound publication review has schema
`datum.preferences.publication-review.v2`, candidate, publication,
changed_paths, reviewer_session and disposition approved. These structural
checks never replace the installed owner boundary or authorize its transition.

The full runner requires explicit report, bundle, candidate, environment digest
and a new output path. It retains every mandatory gate's output, runs Cargo
serially through the resource guard, records failures/timeouts, and validates
the complete pre-review report before returning `reviewable`. Independent
review then binds that captured subject, and `check_global_preferences_production_evidence.py
--ready` validates the final reviewed report. It has no optional full-workspace or
measurement flags capable of printing a misleading production PASS. The bounded
correctness and partial release-measurement tools remain diagnostic until actual V2
case/native/measurement capture has supplied every required observation.

The partial release collector preserves per-command stdout/stderr, actual
`wait4` RSS, monotonic durations and TTY APPLY timing, all ten warm-ups and
three distinct trials. It covers standalone query/Set/Reset and real Project
genesis/validation across factory, missing Global and populated Global roots.
Temporary product state is on project-backed storage; retained artifacts may
live in a separate bundle. `partial_diagnostic` output explicitly lists missing
daemon, generation-storage, native and complete proof obligations. A smaller
`--samples` diagnostic never satisfies the fixed 100-sample V2 protocol.
