# REV-C05 Impact, Staleness, and Reproducibility Contract

> **Status:** REV-C05 candidate specification; not ratified and not an
> implementation authorization. This packet makes dependency impact, library
> uptake, derived-evidence freshness, regeneration, baseline comparison, and
> release reproduction precise enough for REV-C06 visual study and REV-C07
> owner disposition.

## 1. Scope and controlling law

REV-C05 consumes the owner-approved REV-C03 authority model and the REV-C04
standalone/offline boundary. It does not change revision identity, release
authority, Git's subordinate role, or the immutable status of an established
baseline or issued record.

The central law is:

> **Changed, affected, stale, orphaned, and non-reproducible are different
> facts. Datum records each independently and never substitutes one for
> another.**

Consequences:

1. A changed source can be proven to leave a consumer unaffected.
2. An unchanged source can still have stale evidence when its generator,
   policy, environment, or another declared input changed.
3. Historical evidence remains valid evidence of its frozen configuration
   after successor work changes. It does not become retrospectively stale.
4. Missing retained inputs can make a historical release non-reproducible
   without changing what was issued or its standing.
5. Superseded, Withdrawn, and Obsolete are standing facts. None means stale.

REV-C05 owns:

- the dependency snapshot and semantic impact contract;
- affected, unaffected, unknown, stale, and orphan classifications;
- explicit uptake of new library revisions;
- baseline-to-baseline comparison and change accounting;
- regeneration planning and immutable successor evidence;
- reproduction manifests, attempts, and byte-comparison results; and
- typed findings, refusals, queries, and proof obligations for these facts.

REV-C05 does not own:

- user-visible impact/release choreography or visual presentation (REV-C06);
- owner ratification (REV-C07);
- implementation sequencing or dependencies (REV-C08);
- cryptographic algorithm or third-party dependency selection;
- multi-writer merge, live collaboration, or permission topology
  (`dat-distributed-collaboration-architecture-lt1`); or
- the future Global Preferences storage/resolution mechanism. The engine
  consumes an explicit resolved Project `RevisionPolicy`.

## 2. Evidence

### 2.1 Internal Datum evidence

| Evidence | Present fact | REV-C05 consequence |
|---|---|---|
| `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:144-190` | `ConfigurationRef` distinguishes exact Working state from an immutable Baseline manifest; generated evidence is separate. | Every impact, freshness, comparison, and reproduction result names its exact configuration. |
| `PRODUCT_REVISION_ENGINE_AUTHORITY_MODEL.md:580-640` | Exact baseline resolution, no floating release dependencies, immutable evidence, complete change accounting, and typed stale/non-reproducible refusals are already candidate invariants. | REV-C05 refines those obligations; it does not weaken them. |
| `PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md:89-108` | The researched ECSS profile requires impact assessment, common-element propagation, complete change accounting, status retrieval, and configuration comparison. | Impact paths and dispositions must be durable, queryable records rather than transient UI guesses. |
| Product Mechanics 000D, lines 151-190 and 208-219 | Derived connectivity and zone-fill state retain source revisions and are recomputed when their declared inputs differ. | Derived-input comparison is Datum's existing narrow staleness precedent. |
| `crates/engine/src/substrate/zone_fill.rs:22-47,292-324` | `ZoneFill` records source/model revisions and becomes `Stale` on mismatch. | The generalized contract preserves this behavior while adding exact input sets and reasons. |
| `crates/engine/src/substrate/component_instance.rs:344-356` | Placed symbol/package references reject missing or revision-mismatched objects. | Library bindings stay pinned; a newer library object cannot silently replace them. |
| `crates/engine/src/substrate/check_run.rs:64-113` | Check evidence carries model revision, profile basis, coverage, rule revision, and standards basis. | Check freshness must compare all declared bases, not only model revision. |
| `crates/engine/src/substrate/artifact.rs:71-190` | Output jobs bind board/panel, variant, and plan; runs bind model revision/provenance; artifacts retain generator version and file SHA-256 values. | Existing production records are useful inputs, but do not yet constitute a complete reproduction manifest. |

This is an honest inventory. Datum does **not** currently implement the complete
dependency graph, semantic impact evaluator, library-uptake transaction,
regeneration planner, baseline comparison record, hermetic environment capture,
or independent byte-reproduction proof specified below.

### 2.2 External primary and field evidence

| Source | Relevant fact | Datum consequence |
|---|---|---|
| [Reproducible Builds definition](https://reproducible-builds.org/docs/definition/) | Reproducibility requires the same source, environment, and instructions to produce bit-for-bit identical specified artifacts; verification is byte comparison, commonly through cryptographic hashes. | `ByteIdentical` is the release-reproduction result. Semantic or rendered similarity must use a different name. |
| [SLSA provenance v1.2](https://slsa.dev/spec/v1.2/provenance) | Provenance describes where, when, and how an artifact was produced and traces it to its source. | Release evidence needs producer, invocation, inputs, and output subjects, not only filenames and a version string. |
| [in-toto Statement v1](https://in-toto.io/Statement/v1) | A typed statement binds a predicate to immutable subjects identified by algorithm-qualified digests. | Datum's evidence envelope separates digest-qualified subjects from typed provenance/reproduction facts. |
| `PRODUCT_REVISION_ENGINE_STANDARDS_MATRIX.md` | Clause-addressable ECSS, NASA, DoD, and drawing-profile obligations are already classified by source strength and applicability. | REV-C05 maps mechanisms to that matrix and does not claim that software supply-chain patterns create EDA standards conformance. |

Reproducible Builds, SLSA, and in-toto are informative field patterns, not
Datum EDA conformance authorities. Licensed-source gates in the standards
matrix remain unchanged.

## 3. Required vocabulary

| Term | Exact meaning |
|---|---|
| `Changed` | A selected source identity, technical revision, or relevant semantic observation differs between two configurations. |
| `Affected` | A dependency evaluation proves that a change can alter a consumer's governed meaning, validity, or output. |
| `Unaffected` | A dependency evaluation proves that the changed observations are outside a consumer's declared sensitivity. |
| `ImpactUnknown` | Datum cannot prove affected or unaffected because an edge, resolver, semantic delta, or evaluator is missing/unsupported. |
| `Stale` | Derived evidence's recorded input context differs from the exact configuration/context against which it is being evaluated. |
| `Orphaned` | A required producer or input identity cannot resolve in the selected configuration. |
| `Reproducible` | A fresh attempt from the declared source, environment, and instructions produced byte-identical specified outputs. |
| `UnavailableForReproduction` | Required retained input, producer, environment, or instruction cannot be reconstructed. |
| `Standing` | The typed lifecycle fact Current, Superseded, Withdrawn, or Obsolete applied to an immutable issued record. |

`ImpactUnknown` is not `Unaffected`. `CanonicalEquivalent`, visually equal, or
electrically equivalent is not `ByteIdentical`. An orphaned working projection
does not erase a historical baseline member whose retained bytes still resolve.

## 4. Dependency snapshot and impact evaluation

### 4.1 Frozen dependency authority

```text
DependencySnapshot {
  id,
  configuration_ref,
  nodes: sorted Vec<DependencyNode>,
  edges: sorted Vec<DependencyEdge>,
  evaluator_registry_revision,
  completeness,
  unresolved_inputs[],
  digest
}

DependencyNode {
  authority_ref,
  exact_technical_revision,
  semantic_digest,
  node_kind
}

DependencyEdge {
  edge_id,
  source,
  target,
  edge_kind,
  sensitivity: sorted Vec<SemanticObservation>,
  origin,
  evaluator_id,
  evaluator_revision
}
```

Required `edge_kind` families are typed and extensible:

- `UsesDesign`, `UsesLibrary`, `UsesRule`, `UsesTemplate`;
- `BindsIdentity`, `ProjectsInto`, `Checks`, `Generates`;
- `Packages`, `Documents`, `GovernedBy`, and `DerivedFrom`.

Strings may label a typed edge but never create an unreviewed semantic kind.
The snapshot is deterministic, digestible, and exact. A baseline/release
candidate freezes the dependency snapshot used for its analysis.

### 4.2 Semantic delta

```text
SemanticDelta {
  from_configuration,
  to_configuration,
  subject,
  changed_observations[],
  administrative_observations[],
  evaluator_id,
  evaluator_revision,
  digest
}
```

The evaluator reasons from stable identities and typed observations—not file
paths or raw byte difference alone. A rename can be administrative while a
pin-number, layer-stack, rule, population, template field, or generator-policy
change can be semantic. Raw byte change is retained as evidence but does not
pre-decide product impact.

### 4.3 Durable impact result

```text
ImpactEvaluation {
  id,
  governing_change,
  from_configuration,
  to_configuration,
  dependency_snapshot,
  evaluated_at,
  evaluator_registry_revision,
  subjects: sorted Vec<SubjectImpact>,
  graph_completeness,
  digest
}

SubjectImpact {
  subject,
  result: Affected | Unaffected | Unknown | NotApplicable,
  changed_observations[],
  witness_paths[],
  reason_code,
  required_actions[],
  reviewer_disposition?
}
```

An `Affected` or `Unaffected` result must cite at least one machine-readable
witness or evaluator proof. Absence of a discovered path is not proof of
unaffected when the graph is incomplete. `Unknown` remains visible and blocks
release whenever the active profile requires a resolved impact disposition.

Impact traversal crosses Design, library, rules, checks, Publish,
manufacturing, artifacts, controlled documents, and release-package members.
It may route human review, but reviewer membership and UX belong to REV-C06.

## 5. Staleness and historical truth

### 5.1 Evidence input context

```text
EvidenceInputContext {
  configuration_ref,
  dependency_snapshot,
  inputs: sorted Vec<DigestQualifiedInput>,
  policy_refs[],
  producer_ref,
  producer_revision,
  invocation_digest,
  environment_digest?
}
```

Freshness compares every declared input that the evidence contract says can
affect its result. It returns:

```text
EvidenceFreshness =
  Current
  | Stale { differing_inputs[], reasons[] }
  | Orphaned { unresolved_inputs[] }
  | Unknown { unsupported_inputs[], reason }
```

A check run can therefore become stale because its rule/profile basis changed
even when the model did not. A Gerber set can become stale because board,
variant, output-job, manufacturing-plan, generator, or relevant policy changed.
The finding names exactly which inputs differ.

### 5.2 Working versus historical evaluation

- Working and candidate evidence is evaluated against its selected current
  `ConfigurationRef`.
- Release evidence is evaluated against the immutable baseline and context it
  originally covered.
- Successor changes may produce an impact finding against later work, but never
  rewrite the old evidence to `Stale`.
- Inability to rerun historical evidence becomes
  `UnavailableForReproduction`; it does not alter issued bytes, approvals, or
  standing.

## 6. Library uptake

Placed Design objects bind exact library object identities and revisions.
Publication of a newer symbol, footprint, package, model, or part lifecycle
record creates an update opportunity—not an automatic rebind and not, by
itself, stale Design authority.

```text
LibraryUptakeCandidate {
  id,
  binding_subject,
  pinned_library_ref,
  proposed_library_ref,
  semantic_delta,
  compatibility:
    Compatible | RequiresRemap | Breaking | Unknown,
  predicted_impact,
  required_checks[],
  required_regeneration[],
  governing_change?
}
```

Rules:

1. Preview is read-only and does not reserve or change revision identity.
2. Adoption is an explicit typed operation governed by an
   `EngineeringChange` when policy requires it.
3. Pin/pad, gate/unit, layer, 3D-model, lifecycle, provenance, and standards
   annotation differences are independently reportable.
4. `NRND`, `EOL`, `Obsolete`, or changed review annotations can trigger
   findings and routing without silently replacing geometry.
5. An established baseline continues resolving the retained exact library
   bytes. Missing retained bytes refuse reproduction; they do not float to
   `latest`.
6. Working bindings whose exact source cannot resolve are `Orphaned` and cannot
   be released until explicitly repaired or acceptably departed.

## 7. Baseline comparison and change accounting

```text
BaselineComparison {
  id,
  left_baseline,
  right_baseline,
  aligned_members: sorted Vec<MemberComparison>,
  dependency_delta,
  evidence_delta,
  governing_change_coverage,
  unresolved_differences[],
  digest
}

MemberComparison {
  stable_authority_ref,
  result: Added | Removed | Modified | Retargeted | Unchanged,
  technical_revision_delta,
  semantic_delta,
  related_changes[],
  impact_dispositions[]
}
```

Comparison aligns stable authority identities before inspecting labels or
paths. It compares exact members, revisions, dependency snapshots, policy
inputs, controlled evidence, and dispositions. Renames remain visible without
being misrepresented as delete/add when stable identity survives.

Every controlled successor difference must trace to a governing
`EngineeringChange`, approved departure, or typed administrative disposition.
Unexplained differences produce `UnaccountedControlledDifference` and block
release. The comparison does not auto-increment any EngineeringRevision.

## 8. Regeneration

```text
RegenerationPlan {
  id,
  target_configuration,
  basis_impact_evaluation,
  steps: topologically_sorted Vec<RegenerationStep>,
  unchanged_evidence_reused[],
  blockers[],
  digest
}

RegenerationStep {
  output_contract,
  predecessor_evidence?,
  reason_paths[],
  prerequisites[],
  producer_ref,
  producer_revision,
  expected_input_context
}
```

The planner regenerates only evidence that is affected, stale, missing, or
required anew by policy. Steps are ordered by typed dependency edges. A plan
never mutates a baseline or existing evidence; execution creates a new evidence
identity linked to its predecessor. “Refresh in place” is forbidden.

Regeneration of working/candidate outputs and reproduction of a released output
are distinct operations. The former targets a new configuration; the latter
attempts to recreate the old baseline's exact specified bytes.

## 9. Reproducible release evidence

### 9.1 Manifest

```text
ReproductionManifest {
  id,
  release_candidate,
  configuration_ref,
  source_subjects: sorted Vec<DigestQualifiedInput>,
  dependency_snapshot,
  producer: ProducerIdentity,
  invocation: InvocationIdentity,
  environment: EnvironmentIdentity,
  specified_outputs: sorted Vec<OutputSubject>,
  variance_policy,
  canonicalization_policy_refs[],
  created_at,
  digest
}

DigestQualifiedInput { authority_ref, revision, algorithm, digest }
OutputSubject { logical_name, kind, byte_count, algorithm, digest }
```

`ProducerIdentity` includes the Datum/generator build identity and digest, not
only a display version. `InvocationIdentity` includes effective settings and
ordered instructions. `EnvironmentIdentity` records every declared influential
input, including toolchain/dependencies, platform where relevant, locale,
timezone, path policy, timestamp source, random seed/source, and environment
variables. Irrelevant environment must be eliminated or explicitly excluded by
the output contract.

Floating network data, `latest` dependencies, undeclared local files, and
uncontrolled clocks/randomness are prohibited in a release reproduction path.

### 9.2 Attempt and result

```text
ReproductionAttempt {
  id,
  manifest,
  executor_identity,
  attempted_at,
  independently_resolved_environment,
  produced_outputs[],
  result:
    ByteIdentical
    | OutputMismatch { differences[] }
    | Unavailable { missing_requirements[] }
    | ExecutionFailed { findings[] },
  comparison_evidence,
  digest
}
```

For release evidence governed as reproducible, success means all specified
outputs are byte-for-byte identical and their algorithm-qualified digests
match. `CanonicalEquivalent`, render-equivalent, electrically equivalent, or
content-equivalent analyses may be retained as diagnostics but cannot satisfy
the `ByteIdentical` requirement.

Formats containing timestamps, signatures, random identifiers, or path leakage
must normalize those inputs before hashing or separate deterministic product
content from an externally applied delivery/signature envelope. If a selected
output cannot honestly meet byte reproducibility, the profile records that fact
and either excludes it from the reproducible set or refuses release. Datum must
never weaken the word “reproducible” to make a gate green.

Original issued bytes are retained regardless of later reproducibility. A
failed future reproduction creates a new result; it does not rewrite the
Release, its evidence, or its standing.

### 9.3 Authenticity is separate

Digest equality proves byte equality, not signer identity or authorization.
Signatures, trusted timestamps, approvals, and external attestations remain
separate typed evidence governed by REV-C03/REV-C04 and future dependency
ratification. No cryptographic library or algorithm is selected here.

## 10. Typed operations and queries

Required operation families:

- `CaptureDependencySnapshot`
- `EvaluateSemanticDelta`
- `RecordImpactEvaluation`
- `DispositionSubjectImpact`
- `PreviewLibraryUptake`
- `AdoptLibraryRevision`
- `CompareBaselines`
- `CreateRegenerationPlan`
- `ExecuteRegenerationStep`
- `CreateReproductionManifest`
- `RecordReproductionAttempt`

Required queries:

- `impact explain <change> [subject]`
- `impact path <source> <consumer>`
- `evidence freshness <evidence> --configuration <ref>`
- `library uptake preview <binding> <library-revision>`
- `configuration compare <baseline-a> <baseline-b>`
- `regeneration plan <configuration>`
- `release reproduce|verify <release>`
- `reproduction explain <attempt>`

All operations use the one mutation path. Read-only previews return deterministic
candidate records and never silently commit, rebind, allocate revision identity,
or update issued evidence.

## 11. Required findings and refusals

| Code | Trigger |
|---|---|
| `DependencyGraphIncomplete` | Required node/edge/evaluator or resolvable input is absent. |
| `ImpactUnknown` | Affected/unaffected cannot be proved. |
| `ImpactAnalysisIncomplete` | Required subjects lack a disposition. |
| `UnaccountedControlledDifference` | Successor difference lacks governing change/departure/disposition. |
| `LibraryUpdateUnassessed` | Release candidate has a required but undispositioned library uptake. |
| `PinnedLibraryRevisionMissing` | Exact pinned library authority cannot resolve. |
| `DerivedEvidenceStale` | Declared evidence input differs from target context. |
| `DerivedEvidenceOrphaned` | Declared evidence input or producer cannot resolve. |
| `RegenerationPrerequisiteMissing` | A plan step cannot obtain exact required input/producer. |
| `GeneratorIdentityMismatch` | Producer identity/build digest differs from manifest. |
| `EnvironmentUncaptured` | A declared influential environment input is missing. |
| `OutputDigestMismatch` | A specified output differs byte-for-byte. |
| `ReleaseEvidenceNotByteReproducible` | Active policy requires successful byte reproduction and it is absent/failed. |

Each finding/refusal carries stable affected identities, expected/current facts,
witness paths, controlling policy/requirement, and actionable remediation.

## 12. Invariants

1. No floating dependency enters a baseline, impact proof, or reproduction
   manifest.
2. Every affected/unaffected claim has a typed witness; graph incompleteness
   cannot collapse to unaffected.
3. Staleness is evaluated against an explicit configuration and input contract.
4. Historical evidence remains frozen against its historical baseline.
5. Library updates never silently rebind placed Design objects.
6. Regeneration creates successor evidence and never edits prior evidence.
7. Baseline comparison aligns stable identities and accounts for every
   controlled difference.
8. Release reproduction names exact producer, invocation, environment, inputs,
   and outputs.
9. `ByteIdentical` requires byte equality for every specified output.
10. Authenticity, approval, standing, freshness, impact, and reproducibility
    remain separate facts.
11. Git, filenames, timestamps, and display labels cannot substitute for Datum
    authority identities.
12. No profile or future preference can rewrite issued history or relax an
    already-issued evidence claim retrospectively.

## 13. Verification contract

REV-C08 implementation planning must include at least these proofs:

1. A changed source observation with a sensitivity-disjoint consumer produces
   `Unaffected` plus witness evidence.
2. A transitive source change produces `Affected` consumers and complete reason
   paths across Design, Publish, and manufacturing.
3. A missing edge/evaluator produces `ImpactUnknown`, never `Unaffected`.
4. A newer library object produces an uptake candidate without changing the
   placed binding; explicit adoption is journaled and undoable before release.
5. Zone-fill staleness remains compatible with the generalized freshness
   contract.
6. Check, artifact, template, rule, and generator changes identify their exact
   differing inputs.
7. Regeneration is topologically ordered, reuses proven-current evidence, and
   creates immutable successor records.
8. Baseline comparison detects added, removed, modified, retargeted, renamed,
   and unexplained members using stable identity.
9. A released artifact remains unchanged and queryable after successor work.
10. A reproduction succeeds across deliberately different build paths, locale,
    and timezone where those are declared irrelevant/normalized.
11. Uncontrolled timestamp, randomness, order, locale, or path inputs cause a
    deterministic mismatch/finding.
12. Missing historical producer/environment yields
    `UnavailableForReproduction` without altering the Release.
13. External-generator evidence is either fully captured and reproducible or
    honestly excluded/refused by policy.
14. A digest match never impersonates approval or signature verification.

## 14. REV-C06 handoff

The visual study must render, without changing this authority:

- a quiet working state and actionable affected/unknown/stale/orphan findings;
- an impact trace that shows why a document or manufacturing output is affected;
- explicit library update preview and uptake;
- baseline comparison with complete change accounting;
- a regeneration plan that distinguishes reuse from regeneration;
- release evidence showing exact configuration, producer, inputs, and outputs;
- byte-reproduction success, mismatch, and unavailable states; and
- historical release evidence that remains immutable despite successor impact.

REV-C06 may improve language and choreography. Any proposed change to the
meaning or authority in this packet must return to REV-C05/REV-C07 disposition
rather than being selected silently in HTML.
