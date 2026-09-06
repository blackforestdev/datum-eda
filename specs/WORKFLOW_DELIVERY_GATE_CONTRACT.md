# Workflow delivery gate: proposed implementation contract

Status: governed WDQ-C03 proposal; no executable policy installed
Proposed authority: Product Mechanics 041; pending WDQ-C05
Refusal fixtures: `research/process-quality/WORKFLOW_DELIVERY_REFUSAL_CASES.md`

## 1. Single roadmap, four checks

Extend Frontier schema 5 to **6 only in the separately authorized implementation**.
Add optional `completion.delivery` with the closed shape below; do not add status,
assignee, authorization or order to any delivery artifact. Existing completion
steps and beads remain authoritative. Unenrolled schema-5 items migrate without
new acceptance claims. Schema-6 unknown fields still fail closed.

```json
{
  "contract_path": "specs/workflow_delivery/pilot.contract.json",
  "checkpoints": {
    "ready": "PILOT-R01",
    "activate": "PILOT-I02",
    "verify": "PILOT-V01",
    "accept": "PILOT-A01"
  }
}
```

Each non-null checkpoint is a distinct existing completion-step ID in that item,
in the listed dependency order. `ready` is planning/governance, `activate` and
`verify` execution, `accept` owner_decision. A product contract requires all four.
Infrastructure has null `activate`/`accept`, names the consuming workflow, and
cannot be presented as product acceptance. Spec-only research uses the existing
planning/owner-decision mechanism, not a fake infrastructure exemption.

Ready is checked on its completion and again before any enrolled execution claim
or relevant source change. Activation checks candidate source and production
dispatch proof before the enabling change lands. Verification requires complete
native scenario results. Acceptance requires independent review and the exact
owner receipt. Pending steps require structure/references, not nonexistent future
proof; completed transitions require the corresponding evidence. A candidate
build with the proposed activation may be tested in isolation before landing:
testing its working-tree bytes is not authorization to ship or accept it.

## 2. Closed data shapes

All JSON is UTF-8, rejects duplicate object keys, unknown keys, nonfinite numbers
and wrong types (a boolean is not an integer). IDs are nonempty ASCII
`[A-Za-z0-9][A-Za-z0-9._-]*`; IDs within each collection are unique. References are
repository-relative normalized POSIX paths: no absolute paths, `..`, symlink
escapes, URLs or shell commands. Hashes are lowercase SHA-256 hex. Arrays whose
order is not meaningful are sorted by ID/path before hashing. Empty required text
fails. A referenced marker must occur exactly once in the named governed file.

`Ref` = `{path, marker}`; `Blob` = `{path, sha256}`. Blob hashes cover raw bytes.
`Answer` = `{disposition, reason, authority_refs}` with disposition
`required|not_applicable`, nonempty reason and nonempty Ref list. Required means
a scenario must test it; not_applicable requires independent review at acceptance.
An unresolved required question cannot be made not_applicable.

### Contract (all fields required)

| Field | Exact shape and constraint |
| --- | --- |
| `schema_version` | Integer 1 |
| `id`, `frontier_key`, `issue_id` | IDs; match enrollment/Frontier/beads identity |
| `category` | `product` or `infrastructure` |
| `consuming_workflow`, `intent`, `scope`, `exclusions` | Nonempty strings; exclusions cannot contradict mandatory authority |
| `authority_refs` | Nonempty Ref array; applicable ratified decisions and requirement clauses |
| `route_ids` | Nonempty ID array; every routed authority_ref's owning route included and fresh |
| `foundation_answers` | Object with exactly `units_precision`, `numeric_entry`, `selection_identity`, `grid_snap`, `undo_cancel`, `persistence_recovery`, `library_connectivity`, `settings_scope`; every value Answer |
| `open_decisions` | Array of `{id, question, required_for_scenarios, disposition_ref}`; required_for_scenarios nonempty scenario-ID array, disposition_ref Ref or null. Null blocks those scenarios and ready; optional future work belongs in exclusions, not this array |
| `input_roots` | Nonempty normalized path array; code/build/resource dependency closure reviewed for this scenario. Include Cargo.lock/manifests/build flags where applicable; no blanket exclusion for generated source used by build |
| `consumers` | Nonempty array of Consumer objects below; infrastructure names its actual engine/API consumer rather than a pretend GUI |
| `scenarios` | Nonempty array of Scenario objects below |
| `proof_path`, `review_path` | Paths for future Proof and Review records; may be absent on disk before their checkpoints |

`Consumer` = `{id, entry_surfaces, dispatch_key, handler_ref, scope, timing,
persistence, unavailable_reason, scenario_ids}`. entry_surfaces is a nonempty
ID array identifying actual menu/toolbar/shortcut/API entries; dispatch_key is
the production registry key; handler_ref is `{path, symbol}` or null, where path
is a production source in input_roots and symbol its fully qualified handler
identity from the production registry export (not a governed Markdown marker).
Remaining prose fields
are nonempty strings, scenario_ids a nonempty scenario-ID array. A null handler
is legal **only** for deliberately unavailable controls whose required scenarios
prove no invocation/no mutation and an honest visible unavailable state. No
generic class such as `gui_local` is a handler identity. Submenus are navigation
consumers with their own behavior, not exempt enabled leaf actions.

`Scenario` = `{id, consumer_ids, requirement_refs, preconditions, inputs,
expected_visible, expected_state, dimensions, method}`. consumer_ids and
requirement_refs are nonempty ID/Ref arrays. inputs is a nonempty ordered string
array of concrete native user actions or infrastructure operations. Preconditions
and both expected fields are nonempty strings, with exact values/identities where
precision matters. method is `native_input` or `infrastructure`; all product
scenarios use native_input. dimensions is an object with exactly `normal`,
`invalid`, `cancel`, `scope`, `precision`, `undo_redo`, `save_reopen`,
`accessibility`, `failure_recovery`, each an Answer. Each required dimension
must appear in at least one actual assertion in this scenario's result; use
separate small scenarios where that makes proof clearer.

### Proof (all fields required; references never execute code)

`Proof` = `{schema_version, contract_sha256, producer_session, source_commit,
input_manifest, build, fixture, environment, results}`. Version 1; source_commit
is a resolving full Git object ID. input_manifest is a Blob containing a JSON
array of `{path, sha256}` enumerated under input_roots, including relevant
untracked source; deletion/new files change the manifest. producer_session is an
ID and audit identity, not git attribution. fixture is a Blob of a redistributable,
sanitized archive or deterministic fixture data. environment is a Blob with
OS/backend/toolchain/scale/input-method/window-size and reproduction commands.

`build` = `{command, receipt, toolchain, source_clean}`. command is a recorded
string array (not executed by the checker), receipt a Blob, toolchain a
nonempty string, source_clean a boolean. Executables need not be committed;
their hash and rebuild procedure must be committed, and the verification run
must have checked the actual binary. Blob.path names
the **committed build receipt** containing the executable hash, not an absolute
machine path; the receipt format is `{binary_sha256, build_command, toolchain,
input_manifest_sha256, exit_code}` with exit_code exactly 0. This build receipt
is separately checked against the enclosing fields; it is not binary proof by
itself. A dirty proof requires input_manifest bytes covering all dirty inputs;
source_commit alone never represents a dirty build.

Each `results` entry = `{scenario_id, outcome, actual_visible, actual_state,
assertions, artifacts, defects}`. outcome `pass|fail|blocked|unverified`; actual
fields nonempty text, artifacts nonempty Blob array (event log plus captures and
state/journal evidence as applicable), defects array of existing bead IDs.
`assertions` is a nonempty array of `{dimension, expected, observed, outcome}`,
dimension from the nine scenario dimensions, expected/observed nonempty text,
outcome `pass|fail|unverified`. Check exact scenario coverage, no unknown/duplicate
IDs, all required dimensions present and passing. These labels are claims backed
by artifacts, not automated visual understanding. Native tests must correlate
actual input, dispatch/operation and visible/persisted results; a test-only
handler or CLI edit cannot substitute for a production GUI path.

### Review and owner receipt

`Review` = `{schema_version, packet_sha256, reviewer_session, independent_of,
disposition, replay, findings, owner_receipt}`. Version 1; independent_of is the
nonempty list of implementation and original proof producer session IDs.
reviewer_session must differ from all of them. Implementation sessions are
recorded in the trusted enrollment/handoff record, not chosen by the author
after testing. disposition is `approve|revise|reject`; replay is a Blob of a
separate run's input/result record, findings an array of
`{issue_id, severity, disposition_ref}`, severity `blocking|nonblocking`, Ref or
null. Blocking findings must be resolved and replayed; only nonblocking defects
may have an explicit owner-approved deferral. owner_receipt is Ref or null.

The replay record is a second Proof-shaped record with the same contract and
input identities and a distinct producer_session. It must cover all mandatory
scenarios, not a second copy of the original log. Exact-equal original/replay
event artifacts fail independence; byte-identical expected screenshot or state
artifacts are allowed when independently produced. Identity strings alone do
not prove independence; the owner verifies the handoff and reviewer provenance.

The acceptance receipt is a governed section containing the exact owner response
`ACCEPT <frontier-key>/<accept-step> <packet-sha256> <review-sha256>`, the source
of that response and recorded date. review_sha256 excludes owner_receipt to avoid
a hash cycle. A retyped reviewer name, timestamp, refreshed route digest, or
`status: complete` is not a receipt. See the trust boundary in section 5.

## 3. Relevant-input freshness and digest rules

Canonical JSON hashing uses UTF-8 `json.dumps(value, sort_keys=True,
separators=(",", ":"), ensure_ascii=False, allow_nan=False)` plus one newline.
Contract hash covers the entire contract. Packet hash covers the canonical object
`{contract_sha256, input_manifest_sha256, proof_sha256, authority_sha256}`.
authority_sha256 covers sorted `{path, sha256}` for authority_refs' files and all
sources/consumers of route_ids. Do not include this proposal's own evolving
adoption report in a future product contract unless it actually governs behavior.
Proof and review hashes use canonical JSON, not presentation whitespace.
Proof, review and owner-receipt records are non-normative evidence and must not
be members of their own contract's authority route closure. Register receipts
as governed operational evidence under `docs/reviews/`, outside the research
source inventory, and reference them from completion_evidence. If an artifact
also changes normative requirements, split that change into a separate authority
revision and rerun readiness. This avoids receipt/proof/authority hash cycles.

Contracts and input manifests include all production dispatch/enablement paths,
consumed assets, shared numeric/model code and build inputs relevant to their
claims. The independent readiness review confirms the dependency closure.
Changes to contract, consumer registry, handler, fixture, relevant source or
governing route invalidate their dependent evidence and acceptance. Changed
source can be reverified without rewriting the product requirement. Changed
requirements require renewed readiness review as well. Environment changes
outside a declared equivalence require rerun; equivalence itself is reviewed
authority, never inferred from matching screenshots.

Unrelated files outside the reviewed dependency closure do not invalidate proof.
Original source_commit is provenance, not the sole freshness test. A later HEAD
with identical relevant bytes may reuse proof. Once accepted, changed relevant
inputs make the live claim stale; historical receipts remain immutable evidence
of the old packet. The checker reports this; it never silently rewrites Frontier
history or creates a new acceptance on behalf of the owner.

## 4. Consumer activation checks

The pilot implementation must expose a **single production action-consumer
registry**, used by enablement and dispatch, with registered key and supported
contexts. Its test export is derived from that same registry, not a separately
maintained list. Resolve all pilot entry surfaces against it. Disabled entries
have a stable explanation and cannot invoke a mutation; an enabled entry has
the actual registered handler and context. Unknown keys fail closed. Tests must
exercise pointer and keyboard routes, context changes and runtime fallback.

The checker compares the registry export, reviewed contract and proof; code
registration alone is insufficient. A no-op handler returning success fails
expected visible/state assertions in native proof. The static gate cannot decide
that arbitrary handler code is meaningful. Changing a handler requires new proof,
not merely updating the export hash. No runtime file-path scanning or requirement
JSON reading is added to Datum's GUI event loop.

## 5. Enrollment, approval and bypass boundary

The separately implemented policy file `specs/workflow_delivery_policy.json` has
closed shape `{schema_version, decision_ref, enrolled, legacy_baseline}`.
Version 1; decision_ref Ref to ratified PM041; enrolled is a nonempty array of
`{frontier_key, implementation_sessions, activation_ref}`, legacy_baseline is a
resolving Git object ID. No duplicate keys, wildcards or per-task disable flags.
activation_ref points to an owner-reviewed enrollment/handoff record. Only
enrolled keys acquire this new policy; enrollment removal, input-root shrinkage,
required-scenario removal, changed gate code or rewritten approval records is an
authority change, not an ordinary implementation update.

Enforcement invocations require `--authority-ref <owner-selected-commit>` and
`--base-ref <trusted-comparison-commit>` supplied by the owner/controlled runner,
not a fallback to candidate HEAD. Read enrollment, gate/policy hashes and accepted
owner receipts from that authority tree. A new owner acceptance is a two-stage
operation: prepare the candidate packet; the owner checks independent evidence
and supplies the exact ACCEPT response while promoting its receipt commit to the
trusted authority reference. Only then can the final completion transaction pass.
Do not accept new receipts merely because they exist in the candidate tree.

An enrollment/policy update uses the same owner-controlled promotion, preserving
prior receipt history. A candidate cannot self-promote its authority ref. Local
hooks provide early diagnostics; final acceptance/activation proof must run the
**trusted revision's** gate code against candidate content. CI must receive the
authority ref from owner-controlled configuration and the base ref from the event,
not read either from a candidate-controlled policy field. If that facility is
unavailable, remain in report-only mode and use an owner-run verification;
do not advertise tamper-proof CI. No signing service, new dependency or remote
publication is required by this local-first proposal.

Threat model: accidental drift, missing integration, unsupported claims and
in-worktree bypass attempts. A developer with unrestricted machine/CI-admin
control can replace the trust boundary; this is not an adversarial security
system. The owner controls promotion and must not delegate approval credentials
to the implementing agent. The validator verifies consistency, not human identity.

## 6. Exact integration and diagnostic contract

| Existing point | Proposed bounded change |
| --- | --- |
| `scripts/project_status.py` schema/validate | Accept schema 6 and load enrollment; call delivery validation for enrolled completion transitions. Preserve current ordering, claims and byte-for-byte next/details formatting. No automatic receipt creation or fallback selection |
| `scripts/project_task_details.py` closed completion shape | Add optional delivery, validate checkpoint references/kinds/dependencies and call focused delivery module. Do not add another lifecycle state |
| New `scripts/workflow_delivery_contract.py` | Dependency-free parsing, path/hash/schema/scenario coverage and input freshness; pure functions, no writes |
| New `scripts/check_workflow_delivery.py` | CLI over working tree, staged tree or explicit candidate ref; trusted authority/base required in enforce mode. Emit stable WDQ codes with key/step/scenario/path and corrective owning lane, never “auto-fix” |
| `scripts/git-hooks/pre-commit` | Keep lane and staged-rustfmt gates; add cheap staged-only delivery checks after ratification. Materialize the index in a private fixture or read Git blobs; never check unstaged bytes as staged proof |
| `scripts/run_drift_gates.sh`, `.github/workflows/alignment.yml` | Run hermetic negative tests and trusted delivery check, preserve all existing gates, supply trusted refs explicitly. Cargo compilation remains guarded and serial |
| `scripts/check_evidence_traceability.py` | Existing route digest remains authority freshness; call/use its checks without changing refresh semantics. New delivery contracts/decision registered in governance and relevant route |
| `scripts/test_project_status.py` and new focused tests | Prove enrollment cannot change selection, next/details output, claims or owner boundaries; split helpers instead of exceeding PM022 budgets |

CLI modes: `--report-only` emits findings but never readiness/acceptance; enforce
mode returns 0 only on success, 1 on contract/refusal, 2 on invocation/trust
configuration error. Both modes are read-only. `--staged` evaluates all references
from the index plus explicitly identified committed proof artifacts; no mixing
index content with dirty worktree references. Unavailable refs, malformed JSON,
path escapes, untracked required artifacts or missing authority fail, not skip.

The selector's ordinary read-only `next/details` validates structure, current
artifacts and existing authority references, without inventing new approval.
An enforcing transaction additionally supplies the trusted refs through the
standalone gate. The selector cannot on its own declare a newly written owner
receipt trusted; acceptance completion must already be present in the
owner-promoted authority tree. Store the externally selected authority ref in
owner-controlled clone/CI configuration, never in candidate-controlled JSON;
if absent for an enrolled acceptance query, report the missing trust boundary.

Refusal codes and exact test transitions are enumerated in the refusal matrix.
They are specifications, not tests claimed executed by this planning step.
