# Process & Quality — Industry Survey & Datum EDA Implementation Strategy

> Phase 2 deep-dive on Domain 8 of the 8-domain standards audit — the
> FINAL deep-dive. Continues from
> `research/standards-audit/STANDARDS_AUDIT.md § 8`.
> **Consolidates the audit-trail / signature / authentication-context
> substrate** that Domain 2 (encrypted-content gate logging),
> Domain 4 (substrate-vs-certification with 21 CFR Part 11 / ITAR /
> EAR / CMMC / ISO 27001 dependency), Domain 5 (compliance-posture
> lint, substance-list pinning, signed materials declarations),
> Domain 6 (length-match-group authoring, PHY-profile assignment,
> rule-pack version pinning as authored ops, AI-output guard-rail),
> Domain 7 (the nine explicit handoff items: ECO grouping signature,
> AS9102 evidence package signature backing, LibraryAuditEntry as
> audit-log feed, refresh_supply_chain Transaction with
> `data_egress_policy_decision` metadata, ActorIdentity OIDC/SAML,
> lifecycle event feed, pessimistic vault-style check-out deferred
> until Domain 8 user identity, tamper-evident hash chain,
> `data_egress_policy` audit log) have all deferred here.
>
> Reads against the post-Standards-Audit-Batch-1 spec baseline
> (PR #1 merged 2026-04-18); Batches 3/4/5/6 (Domains 4/5/6/7)
> edits are in flight per the project owner's integration cadence.
> The contract surfaces this report builds on (`Operation`, `OpDiff`,
> `Transaction` in `ENGINE_SPEC.md` § 3; `Encrypted Content Handling
> Policy` in `MCP_API_SPEC.md`; `STANDARDS_COMPLIANCE_SPEC.md` § 4.8
> Domain-8 dispositions; `STANDARDS_COMPLIANCE_SPEC.md` § 8 Audit
> Trail And Review Contracts target-state list) are the load-bearing
> baseline.
>
> Cross-references all seven prior Phase-2 reports for tone,
> structure, depth, and source-citation style:
> `research/component-modeling/`, `research/data-exchange-interop/`,
> `research/schematic-drawing-conventions/`,
> `research/industry-vertical-compliance/`,
> `research/materials-environmental/`,
> `research/emc-signal-integrity/`,
> `research/plm-lifecycle-integration/`. Domain-8's role is
> **integrative**: the audit-trail / signature / authentication-context
> substrate it specifies is consumed by all five upstream domains and
> by every authored operation in the engine.
>
> **The Phase-1 audit's signature insight on Domain 8** (worth quoting
> verbatim because it frames the entire deep-dive):
>
> > Datum's transaction model is consistently better-positioned than
> > its other surfaces for compliance work. ISO 9001 audit trail,
> > 21 CFR Part 11 electronic records, AS9102 FAI capture all hang
> > naturally on the existing Operation/OpDiff/Transaction primitives
> > (`specs/ENGINE_SPEC.md:684-731`, `docs/CANONICAL_IR.md:122-188`).
> > The data is being captured; the export surface and authentication-
> > context fields are missing.
>
> This is the framing of the entire report: Domain 8 is **less
> greenfield than other domains**. The substrate already exists. Domain
> 8 specifies the **export contract**, the **authentication-context
> layer**, the **signature primitive**, and the **lint-and-evidence-
> emit surfaces** that the other domains consume.

> **Pending Exclusions Policy (verbatim, ratified 2026-04-17):**
>
> > The audit's "Recommended low-priority / skip" list is an
> > **advisory exclusion** for Phase 2 work. Phase 2 agents MUST NOT
> > re-investigate these standards. Final ratification of skips into
> > binding scope documents happens in a single consolidated pass
> > after Domain 8 lands, when full cross-domain context is available.
>
> For Domain 8 specifically, the advisory exclusion list contains:
>
> - **AS9100** (aerospace QMS) — process certification; Datum is
>   substrate
> - **IATF 16949** (automotive QMS) — process certification; Datum is
>   substrate
> - **CMMI** (capability maturity model) — organisation-process
>   assessment; not a tool feature
> - **ISO 13485** (medical-device QMS) — process certification; Datum
>   is substrate
>
> For each, the recommendation pattern follows Domain 4: Datum is
> substrate, certification is organisational, never tool-level. Each
> gets a "what Datum's substrate provides" paragraph under § "Pending
> Exclusions (re-affirmed)". None has hidden cross-cutting value that
> would justify re-opening the deep-dive.

## Executive Summary

- **The substrate is there. Domain 8 specifies the missing surfaces.**
  Datum's `Operation` / `OpDiff` / `Transaction` types
  (`ENGINE_SPEC.md` § 3; `CANONICAL_IR.md` § 4) are already a
  deterministic, undoable, JSON-serialisable record of every authored
  modification. Add four small extensions to that substrate (acting
  user identity, wall-clock timestamp, structured rationale, optional
  signature blob) and a queryable export surface, and the resulting
  audit trail satisfies the records-and-signatures clauses of every
  audit regime in scope: ISO 9001:2015 § 7.5 (Documented information),
  ISO 13485:2016 § 4.2.5 (Control of records), 21 CFR Part 11 §§ 11.10 /
  11.30 / 11.50 / 11.70 / 11.100-300, EU Annex 11, EIA-649C, ISO
  10007:2017, AS9100D § 8.5.6, IATF 16949 § 8.3.6. The work is
  bounded, the contract is clear, and the substrate behind it has
  been hardened across M0-M5.

- **Audit-trail-as-deterministic-replay is the central differentiator,
  and no incumbent EDA tool can match it.** Datum's deterministic
  serialisation (`CANONICAL_IR.md` § 5) and its purely-functional
  Operation model mean that every captured `Transaction` can be
  replayed against the prior state to produce byte-identical output.
  An auditor does not just read an event log — the auditor can
  reconstruct the design state at any point in history and verify it
  against the persisted state. This is **stronger evidence** than the
  append-only event logs every other commercial EDA tool produces;
  Altium Vault, Cadence Allegro Design Workbench, PADS / Xpedition +
  Teamcenter, and KiCad's git history all stop at "here are the events
  in order". None replay deterministically. The framing is "the audit
  trail IS the design history, not a sidecar to it" and it is the
  single biggest cross-cutting Datum positioning win.

- **The `Signature` primitive is small, well-defined, and
  protocol-agnostic.** Five required fields plus one optional list —
  `signer: ActorIdentity`, `algorithm: SignatureAlgorithm`,
  `signature_blob: Vec<u8>`, `signed_record_hash: [u8; 32]`,
  `timestamp: DateTime<Utc>`, `countersignatures: Vec<Signature>` (so
  multi-party sign-off composes naturally without a separate type).
  The signature is detached (the signed payload — the SHA-256 of the
  Transaction's canonical-JSON serialisation — is recovered from the
  audit-log entry, not embedded in the signature). This shape supports
  21 CFR Part 11 § 11.50 manifestation (signer + meaning + timestamp),
  § 11.70 record-linking (the `signed_record_hash` cryptographically
  binds the signature to the record state), eIDAS QES (the
  `algorithm` enum carries the X.509 certificate chain reference for
  qualified signatures), and PKCS#11 hardware tokens (the
  `signature_blob` is opaque to Datum — Datum never sees the private
  key, the token does the signing).

- **The `ActorIdentity` model decouples engine identity from IdP
  identity.** Datum needs one type — `ActorIdentity { id: Uuid,
  display_name: String, identity_provider: IdpKind, idp_user_id:
  String, signature_capable: bool, attached_at: DateTime<Utc> }` — and
  needs to thread it through every authored operation. The IdP layer
  (OAuth 2.1 / OIDC 1.0 / SAML 2.0 / WebAuthn / PKCS#11) is pluggable
  and lives in a daemon-side identity-provider module; the engine
  treats `ActorIdentity` as opaque post-authentication. This means
  Datum can ship a "Local" identity provider that suffices for solo /
  air-gapped use (the `id` is generated locally, the `idp_user_id` is
  the OS username), and federated identity providers can plug in
  later without changing the engine. This is the correct
  abstraction-layer split: identity establishment is the daemon's
  problem; identity attribution is the engine's contract.

- **The AI-output guard-rail is the single most important Domain-8
  finding for an AI-native EDA tool, and it requires a typed
  `ProvenanceTag` enum.** AI explanations, AI suggestions, AI-narrated
  diagnostics, and AI-rephrased rule rationales are all
  **non-authoritative metadata** by definition; they are convenience
  to the human, not engineering evidence. Domain 6's research called
  this out explicitly ("AI-explanation outputs are not authoritative
  records"). Domain 8 specifies the data marker that distinguishes
  authoritative from non-authoritative bytes:
  `ProvenanceTag::{Authored, Derived, AiSuggestion, UserValidatedAi}`.
  Authored bytes are the user's intent; Derived bytes are the engine's
  deterministic computation; `AiSuggestion` bytes are an LLM's output
  with no engineering authority; `UserValidatedAi` is the canonical
  promotion path: a user accepted an LLM suggestion, and the audit
  trail captures both the original AI suggestion (model name, prompt
  hash) and the user's acceptance event. Once accepted, the data
  becomes `Authored` for downstream purposes but retains a
  back-pointer to its AI origin. **No incumbent EDA tool has this
  distinction in its data model**; every commercial tool that has
  added "AI assistance" features (Altium AD24's AI co-pilot, OrCAD X
  Presto's AI co-pilot, KiCad-AI extensions) treats AI output the
  same as user input. Datum can ship the only formally-typed
  AI-output-vs-engineering-record distinction in the industry.

- **The recommended audit-log export contract is JSON-Lines per
  Transaction, with two adapter exporters for compliance-specific
  formats.** JSON-Lines (one JSON object per line) is the canonical
  default — diffable, streamable, AI-readable, deterministic,
  trivially queryable from `jq` or any JSON-aware tool. For
  21 CFR Part 11 / ISO 13485 reviewer hand-off, a CSV adapter
  exports a flat denormalised view (one row per
  Transaction × ChangedObject pairing) with column headers an FDA
  reviewer recognises. For multi-Transaction sign-off bundles, a
  PAdES (PDF Advanced Electronic Signatures, ETSI EN 319 142) adapter
  emits a PDF with embedded signature blocks suitable for
  organisational document control. JSON-Lines is the source of truth;
  CSV and PAdES are derivations.

- **Tamper-evidence comes free from the existing JSON-determinism
  property — augment with a Merkle hash chain over the Transaction
  sequence.** Each persisted Transaction carries `prev_hash:
  Option<[u8; 32]>` (the SHA-256 of the immediately preceding
  Transaction's canonical-JSON form). This makes the audit log a
  cryptographic chain — any modification to a historical Transaction
  invalidates every subsequent Transaction's `prev_hash`. The
  invariant is verifiable in O(N) by replay; the daily-recompute hook
  is one CLI command (`datum-eda audit verify`). For organisations
  that want external anchoring, the chain head's hash can be
  optionally submitted to an RFC 3161 Time-Stamp Authority or to
  sigstore's rekor verifiable log; both are detachable, both are
  out-of-band, neither requires Datum to operate as a network service.

- **The audit-log surface integrates with every other domain's
  deferred-here items via one consistent contract: every authored
  Operation produces a `TransactionEntry` with a typed `event` field,
  and every domain's "this should be audit-logged" item is one new
  `AuditEventKind` variant.** The Domain-2 encrypted-content
  extraction-attempt logging becomes
  `AuditEventKind::EncryptedExtractionAttempt`. The Domain-5
  substance-list-version pin-update becomes
  `AuditEventKind::SubstanceListPinUpdated`. The Domain-6 PHY-profile
  assignment becomes `AuditEventKind::PhyProfileAssigned`. The
  Domain-7 `refresh_supply_chain` invocation becomes
  `AuditEventKind::SupplyChainRefreshed { data_egress_policy_decision }`.
  The Domain-7 ECO state transition becomes
  `AuditEventKind::EcoStateChanged { from, to, by_signature }`. One
  enum, one log, one query surface, one export contract. All eight
  domains' compliance evidence flows through the same pipe. This is
  the integrative payoff of having Domain 8 land last.

- **Workflow gates (ECO state machine, library approval, sign-off)
  are state-machine constraints over signatures, not new mechanisms.**
  The ECO state machine specified in Domain 7
  (Draft → Submitted → InReview → Approved → Implemented → Closed) is
  encoded as a state-transition table where each transition specifies
  required signature meanings and required approver roles. The
  `EcoSignaturePolicy` (per-project, lives on `ProjectCompliance`)
  binds named workflow steps to required signature meanings:
  `{ submission_requires: [Authored], review_requires: [Reviewed],
  approval_requires: [Approved x N (where N >= reviewer_minimum)],
  implementation_requires: [Released] }`. This composes the ISO 9001
  / ISO 13485 / AS9100 / IATF 16949 review-and-approve clauses into
  one mechanism, parameterised per project.

- **Lint diagnostics and validator findings are first-class audit
  events.** Domain 5's compliance-posture lint (RoHS-status missing on
  required-halogen-free project), Domain 6's controlled-impedance lint
  (Stackup Dk missing for high-speed NetClass), Domain 4's ITAR-flag
  diagnostics, and the standard ERC/DRC findings all emit
  `AuditEventKind::LintFinding { severity, rule_id, finding_id }` and
  `AuditEventKind::WaiverGranted { finding_id, signature, rationale }`
  events. The waiver carries an embedded `Signature` because the act
  of granting a waiver is itself an electronic signature ("I, this
  user, accept the risk of this finding for this rationale"). This
  mechanism scales from single-line CSV exports to fully-signed
  PAdES waiver dossiers without any API change.

- **Data-egress audit is the single most important field for
  AI-native compliance posture, and Domain 4 / Domain 7 already
  specified the policy enum.** Every external-network call (Octopart
  / distributor / PLM connector / cloud sim / external LLM API call)
  carries `data_egress_policy_decision: AllowedExplicit |
  AllowedByDefault | Blocked` in its Transaction metadata, plus a
  snapshot of the current policy state. Auditors get
  one-query visibility into "did any tool ever make an external call
  on this ITAR-marked project?" — and the answer is cryptographically
  bound to the engine state at the moment the call was made. No
  incumbent tool surfaces this evidence; most commercial tools do not
  even capture the policy-decision snapshot at call time.

- **The export contract for compliance-export bundles is a single
  signed PAdES PDF or a directory of detached XAdES signatures.**
  PAdES (ETSI EN 319 142) is the EU-recognised PDF signature format
  with regulatory-equivalence to handwritten signatures under eIDAS.
  XAdES (ETSI EN 319 132) is the XML signature format used for
  IPC-2581, IPC-1752A, IPC-1755 XML payloads. CAdES (ETSI EN 319 122)
  is the CMS signature format for arbitrary binary payloads. Datum's
  signature substrate emits all three via three thin adapters; the
  underlying signature primitive is identical across all three.
  Algorithm choices follow ETSI TS 119 312 recommendations
  (RSA-PSS-3072, ECDSA-P-256, ECDSA-P-384, EdDSA-Ed25519, all
  SHA-256 / SHA-384 / SHA-512 hashed).

- **Most of the certification-grade QMS standards (AS9100D, IATF
  16949, ISO 13485, CMMI) are correctly out of Datum's scope, and the
  consolidated post-Domain-8 ratification pass should formally
  exclude them.** The substrate-vs-certifier framing applies cleanly:
  Datum can be the substrate that an organisation's QMS leverages,
  but the certifying authority is always external (registrar, FDA
  inspector, IAQG audit, IATF audit). The audit's `Recommended
  low-priority / skip` list for Domain 8 contains four items
  (AS9100, IATF 16949, CMMI, ISO 13485 process-grade); all four
  hold up after the deep-dive. Each gets a paragraph under § "Pending
  Exclusions (re-affirmed)" documenting what Datum's substrate
  provides each one.

- **The deep-dive surfaces 28 recommended spec edits across six
  files** (`STANDARDS_COMPLIANCE_SPEC.md`, `ENGINE_SPEC.md`,
  `MCP_API_SPEC.md`, `NATIVE_FORMAT_SPEC.md`,
  `docs/POOL_ARCHITECTURE.md`, `docs/INTEROP_SCOPE.md`), divided into
  four passes: Pass 0 disposition refresh, Pass 1 schema bedrock
  (ActorIdentity, Signature, ProvenanceTag, AuditEntry, EcoSignaturePolicy),
  Pass 2 transaction extension and tamper-evident chain, Pass 3 MCP
  query/export/sign tools (~14 new tools). Comparable to Domain 7's
  12-edit count and Domain 6's 15-edit count; this report's count is
  higher because Domain 8 owns the contract for five upstream domains
  in addition to its own scope.

- **The biggest unexpected finding is that the eIDAS-Qualified
  Electronic Signature (QES) regime is more accessible to an
  open-source tool than the audit's framing assumed.** Qualified
  Trust Service Providers (QTSPs) sell certificate-signing services
  for EUR 50-200/year per signer, the relevant ETSI TS 119 312
  algorithm specifications are free, and PKCS#11 hardware tokens
  (smart cards, USB HSMs) follow a published OASIS standard.
  The Datum-side work is bounded: implement the `Signature`
  primitive against a PKCS#11 token, route through any QTSP-issued
  certificate, and the resulting signatures carry the same legal
  weight as handwritten signatures across the EU. **eIDAS QES is
  achievable as a Datum capability for any project that chooses to
  enable it**, with no Datum-side certification overhead — the QTSP
  is the certifying party. This is a genuine differentiator: no
  open-source EDA tool currently supports eIDAS QES.

## The Audit-Trail-as-Deterministic-Replay Position

This section is the central positioning claim of the entire report.
Every other section either supports it or specifies the substrate that
implements it.

### The claim

Datum's design history is not a log of events; it is a sequence of
deterministic transformations whose intermediate states are
re-derivable from authored inputs alone. An auditor does not have to
trust the log — the auditor verifies the log by replay.

### Why this is stronger than incumbent practice

Every commercial EDA tool surveyed (Altium Vault, Cadence Allegro
Design Workbench, Mentor Xpedition + Teamcenter, OrCAD CIS) and every
open-source tool (KiCad git history, Horizon git history, LibrePCB
git history) records design history as an **append-only event log**.
The log is the source of truth for "what happened"; the design state
is the source of truth for "what is now". The two are independent
artifacts; the relationship between them is asserted, not proven.

The classic failure mode of this architecture is **log/state drift**:
a side-effect modifies state without going through the log (a manual
file edit, a script bypassing the API, a third-party tool writing to
the database), and the log no longer accurately describes how the
state was reached. Auditors catch this by sampling — picking a
historical Transaction and verifying that applying it to the prior
state would have produced the recorded subsequent state. In incumbent
tools this verification is impractical because the operations are
not deterministic (rename a net, re-route a track — Cadence Allegro
will produce different intermediate routing on a re-run because the
auto-router is non-deterministic).

Datum's architecture eliminates this failure mode at the source. The
canonical IR's determinism invariant (`CANONICAL_IR.md` § 5) means
every Operation's `execute()` produces the same `OpDiff` against the
same prior state, byte-identical, on every platform, in every run.
The persisted `OpDiff` is sufficient evidence that the recorded
Transaction was the cause of the recorded state change. An auditor
can pick **any** historical Transaction, replay it against the
recoverable prior state, and verify that the recorded diff matches
the replay-produced diff exactly.

### What this means for compliance evidence

For 21 CFR Part 11 § 11.10(e) ("audit trail generated independently
of the user, time-stamped, secure"): the deterministic-replay
property satisfies the "secure" clause cryptographically. For
ISO 9001:2015 § 8.5.6 ("control of changes"): the replay property
gives the same evidence to an external registrar as to an internal
QA function. For AS9102 First Article Inspection (Domain 7
deliverable): the replay-derived state at the moment of FAI sign-off
is the FAI-evidence baseline, cryptographically bound to the
authored history. For ECO-replay scenarios: an ECO captured against
revision A can be replayed against revision B and the engine
guarantees byte-identical results in the unaffected parts of the
design (the determinism invariant extends across replay boundaries).

### The two implementation pieces required

The replay capability needs two pieces beyond what is already in
M0-M5:

1. **Operation-log persistence**: today the operation log is in-memory
   and discarded at engine close. Persisting it (one JSON file per
   Transaction in `<project>/audit_log/<transaction-uuid>.json`) is
   modest engineering work — the serialisation contract already
   exists. Effort: ~3 days.

2. **Replay verifier**: a CLI subcommand `datum-eda audit verify` that
   walks the operation log forward from the project's initial state,
   replays each Transaction, and asserts byte-equality with the
   persisted intermediate states. Effort: ~5 days for the verifier,
   including the test suite that proves the verifier itself is sound
   (i.e., that a corrupted log is detected, that a corrupted state is
   detected, that the verifier's claim of "passes" is itself
   reproducible).

That is the entire deterministic-replay-as-audit-evidence work. It
ships in the Domain 8 batch.

### The qualification

The replay claim has one boundary condition that must be documented:
the claim holds for **authored data**. Derived data (airwires, DRC
markers, zone fills) is recomputed from authored data on demand;
its values are deterministic given the authored state but they are
not themselves replayed because they are not part of the operation
log. This is the correct partition (authored = audited, derived =
verifiable on demand) and matches the established Datum invariant
that derived data is always rebuildable from authored data alone
(`CANONICAL_IR.md` § 3).

## Standards Catalog

### Quality Management Systems (substrate-relevant)

#### ISO 9001:2015

**Full title.** **ISO 9001:2015** — *Quality management systems —
Requirements*. Current revision is the fifth edition (2015), with a
2020 amendment for COVID-relevant guidance and a confirmed-2024
review (ISO TC 176/SC 2) that left the substantive requirements
unchanged. The 2030 revision is in committee discussion as of
2026-04 with no published draft.

**Issuing body.** **ISO** (International Organization for
Standardization), TC 176/SC 2 (Quality systems). Adoption is global;
the standard is available in 12 official ISO languages.

**Scope.** ISO 9001 is the generic QMS standard against which most
quality-management-system certifications are issued. Sector-specific
QMS standards (AS9100D for aerospace, IATF 16949 for automotive,
ISO 13485 for medical devices, TL 9000 for telecommunications) are
all derivatives that add sector-specific requirements onto an
ISO 9001 base. The clauses Datum's substrate must support are:

- **§ 4 Context of the organization** — establishes scope, no Datum
  intersection.
- **§ 5 Leadership** — organisational responsibilities, no Datum
  intersection.
- **§ 6 Planning** — risk-based thinking, no direct Datum intersection.
- **§ 7 Support** — including § 7.1.5 (Monitoring and measuring
  resources, traceability), § 7.5 (**Documented information** —
  this is the key clause for Datum), § 7.5.2 (Creating and updating
  documented information), § 7.5.3 (**Control of documented
  information** — version control, distribution, access, retrieval,
  storage, preservation, change control, retention, disposition).
- **§ 8 Operation** — including § 8.3 (Design and development), § 8.5
  (Production and service provision), § 8.5.6 (**Control of changes**
  — every design change must be reviewed, controlled, and documented).
- **§ 9 Performance evaluation** — internal audit (§ 9.2), management
  review (§ 9.3), no direct Datum intersection beyond providing
  evidence on demand.
- **§ 10 Improvement** — corrective action (§ 10.2), continual
  improvement (§ 10.3).

The two clauses that map directly onto Datum's substrate are **§ 7.5
Documented information** and **§ 8.5.6 Control of changes**. Both are
satisfied by the Datum transaction model + audit-log export surface
specified in this report.

**Adoption status (2026).** **Mainstream-mandatory** for any
organisation seeking to demonstrate quality-management-system maturity
to customers, regulators, or accreditation bodies. Approximately one
million ISO 9001-certified organisations worldwide as of 2024 (ISO
Survey 2023 figures, latest published).

**License / IP.** **Paywalled.** Full PDF from ISO Webstore at ~CHF
138; Annex SL high-level structure is excerpted in many free
sources. Many national standards bodies (BSI, ANSI, DIN, AFNOR)
re-publish national-prefix versions (BS EN ISO 9001, ANSI/ASQ
9001) with similar pricing.

**EDA tool support matrix.** **No EDA tool offers ISO 9001
certification — the certification is conferred on the organisation,
not the tool.** All commercial EDA tools provide audit-log
substrate; certification posture comes from the user's deployment.

**Datum coverage status.** **`Deferred with prerequisite`** per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.8 ("ISO 9001 / ISO 13485 / 21 CFR
Part 11 conformance claims — Prerequisite: exported audit-trail
completeness plus user/signature metadata"). **Confirm**; this report
specifies that prerequisite. Once landed, Datum can move ISO 9001
substrate-claim to `Reference-only` (Datum is substrate, not
certifier — never `Implemented`).

**Datum implementation cost.** Covered by the consolidated audit-log
export + `Signature` + `ActorIdentity` work specified later. No
ISO-9001-specific work is needed beyond the substrate.

**Strategic recommendation.** **Reference-only**. The right Datum
positioning is "Datum's audit-log export and signature substrate are
suitable evidence for organisations pursuing ISO 9001 certification".
Datum makes no certification claim of its own.

**Risks.** **AI-surface risk:** if an AI agent claims "your project
is ISO 9001 compliant" the user could rely on that statement in a
customer-facing document. The MCP wording rule MUST be: "your
project uses Datum's ISO 9001 substrate features (audit-log export +
signed sign-off); ISO 9001 certification is conferred on your
organisation by an accredited registrar, not by Datum".

#### ISO 13485:2016

**Full title.** **ISO 13485:2016** — *Medical devices — Quality
management systems — Requirements for regulatory purposes*. Current
revision is the third edition (2016), with a 2020 amendment for
clarifications. A 2024 confirmed-without-changes review applies.

**Issuing body.** **ISO** TC 210 (Quality management and
corresponding general aspects for medical devices).

**Scope.** Medical-device QMS — derived from ISO 9001 with extensive
medical-device-specific additions. The clauses that map onto Datum's
substrate are:

- **§ 4.2.5 Control of records** — records must be legible,
  identifiable, retrievable, retained for a defined period, and
  protected from unauthorised modification. **This is the
  audit-trail clause Datum's substrate satisfies.**
- **§ 7.3 Design and development** — § 7.3.7 (Design and development
  changes) requires every change to be reviewed, verified, validated,
  approved, and documented.
- **§ 7.5.6 Validation of processes for production and service
  provision** — process validation; no direct Datum intersection at
  the EDA tool level.
- **§ 8.2.6 Monitoring and measurement of product** — pre-shipment
  evidence; cross-references AS9102 / FAI workflows.

The Datum-relevant clauses (§ 4.2.5 + § 7.3.7) are satisfied by the
same substrate that satisfies ISO 9001 § 7.5 + § 8.5.6 — no
ISO-13485-specific Datum work beyond the cross-cutting substrate.

**Adoption status (2026).** **Mainstream-mandatory** for medical-
device manufacturers seeking FDA QSR (21 CFR Part 820), EU MDR
(2017/745), or similar regulator alignment. The 2025 FDA Quality
Management System Regulation (QMSR) Final Rule effectively
incorporates ISO 13485:2016 by reference (effective February 2026),
collapsing the historical FDA Part 820 / ISO 13485 dual-track for
US medical-device companies.

**License / IP.** **Paywalled.** ISO Webstore ~CHF 158.

**EDA tool support matrix.** Same as ISO 9001 — substrate-only,
certification is organisational.

**Datum coverage status.** **`Reference-only`** per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4 (medical vertical: "ISO 13485,
FDA Part 820, EU MDR — Reference-only or metadata-only at the core
engine layer"). **Confirm**. The substrate that satisfies ISO 13485
§ 4.2.5 is the same substrate this Domain-8 report specifies.

**Datum implementation cost.** Covered by the cross-cutting substrate.

**Strategic recommendation.** **Reference-only**. Same wording rule
as ISO 9001 — Datum is substrate, certification is organisational.
Document the post-2026 FDA QMSR alignment so medical-device users
understand a single substrate posture serves both FDA QSR and
ISO 13485.

**Risks.** Same AI-surface guard-rail as ISO 9001.

#### AS9100D / AS9110 / AS9120 (skip — substrate paragraph)

**Full title.**
- **AS9100D (2016)** — *Quality Management Systems — Requirements
  for Aviation, Space and Defense Organizations*. Issued by SAE
  International on behalf of IAQG (International Aerospace Quality
  Group); the EU equivalent is **EN 9100:2018**, the Asia-Pacific
  equivalent is **JISQ 9100:2016 / SJAC 9100:2017**.
- **AS9110C (2016)** — *Quality Management Systems — Requirements
  for Aviation Maintenance Organizations*.
- **AS9120B (2016)** — *Quality Management Systems — Requirements
  for Aviation, Space, and Defense Distributors*.

**Issuing body.** **SAE International** on behalf of **IAQG**
(International Aerospace Quality Group); national counterparts via
ASQ (US), TÜV (Germany), JIS (Japan).

**Scope.** Aerospace QMS — derived from ISO 9001 with extensive
aerospace-specific additions: counterfeit prevention (§ 8.1.4 — see
cross-ref Domain 5), risk management, configuration management,
verification of purchased product, control of nonconforming product,
First Article Inspection (FAI — see cross-ref Domain 7 / AS9102),
foreign object debris (FOD) prevention.

**Adoption status (2026).** **Mainstream-mandatory** for any
organisation participating in the aerospace supply chain (Tier 1 to
Tier N suppliers). IAQG OASIS database lists ~24,000 AS9100-certified
sites worldwide (2025 figures).

**License / IP.** **Paywalled.** SAE store ~USD 207 per standard.

**Why advisory exclusion.** **Process-grade certification, conferred
on the organisation by an IAQG-accredited registrar.** The certifying
party is the registrar; the EDA tool is one of many evidence sources
the registrar samples. AS9100D's clauses that intersect Datum's scope
(§ 8.5.6 Control of changes; § 8.1.4 Counterfeit prevention; AS9102
FAI) are satisfied by the cross-cutting substrate this report
specifies plus Domain 7's AS9102 evidence-package contract plus
Domain 5's component-compliance fields. No AS9100-specific Datum
work is needed.

**Substrate positioning.** Datum's contribution to an organisation's
AS9100 audit posture:
1. **Deterministic transaction log** (this report) satisfies § 8.5.6
   Control of changes.
2. **Component-lifecycle tracking** (Domain 7 — `Lifecycle::{Active,
   Nrnd, Eol, Obsolete}`, supersede chains, AVL) supports § 8.1.4
   Counterfeit prevention by surfacing high-risk parts for
   procurement-side verification.
3. **AS9102 evidence package** (Domain 7 + Domain 8 signature
   substrate) supports the First Article Inspection clause.
4. **Variant data model** (`specs/ENGINE_SPEC.md:440-444`) supports
   AS9102 fitted-component reporting.

**Re-affirmed exclusion.** No hidden cross-cutting value. AS9100D
should be promoted to formal `Out of scope` in
`STANDARDS_COMPLIANCE_SPEC.md` § 4.8 in the consolidated post-Domain-8
ratification pass, with a single-paragraph substrate-positioning note.

#### IATF 16949 (skip — substrate paragraph)

**Full title.** **IATF 16949:2016** — *Quality management system
requirements for automotive production and relevant service parts
organizations*. Current revision the first IATF-published edition
(2016, replacing ISO/TS 16949:2009).

**Issuing body.** **IATF** (International Automotive Task Force);
co-published with ISO/TC 176 alignment. Certification by IATF-
recognised certification bodies.

**Scope.** Automotive QMS — derived from ISO 9001 with extensive
automotive-OEM-specific additions: customer-specific requirements
(per OEM), production part approval process (PPAP), advanced product
quality planning (APQP), production trial run, error proofing,
control plans, FMEA, MSA (measurement system analysis), SPC (statistical
process control).

**Adoption status (2026).** **Mainstream-mandatory** for any
organisation supplying to global OEMs (VW, Toyota, GM, Ford,
Stellantis, Hyundai, Renault, BMW, Mercedes, Honda, Nissan, etc.).

**License / IP.** **Paywalled.** AIAG store ~USD 195.

**Why advisory exclusion.** Same framing as AS9100 — process-grade
certification conferred on the organisation. The Datum-relevant
clauses (§ 8.3.6 Engineering changes, § 7.5.3.1 Records retention)
are satisfied by the cross-cutting substrate.

**Substrate positioning.** Same shape as AS9100 — Datum's audit-trail
+ signature substrate supports an IATF 16949 audit posture; the
certifying party is the IATF-recognised certification body.

**Re-affirmed exclusion.** No hidden cross-cutting value. Promote to
formal `Out of scope` in the ratification pass.

#### AS9100D § 8.1.4 counterfeit prevention (cross-ref Domain 5)

AS9100D's counterfeit-prevention clause (§ 8.1.4) intersects three
Datum surfaces:

1. **Component lifecycle metadata** (Domain 7) — `Lifecycle::Eol`
   parts have higher counterfeit risk; supersede chains
   (Domain 7) help users navigate to safer alternatives.

2. **AVL (Approved Vendor List) management** (Domain 7) — AS9100D
   § 8.1.4 requires "control of suppliers and verification of
   purchased product"; the AVL surface specifies the per-Part
   approved-supplier set and gates BOM exports against
   `data_egress_policy` to prevent unverified-source procurement.

3. **Authoritative provenance** (this report) — every authored Part
   addition carries `ProvenanceTag::Authored` with `actor:
   ActorIdentity`; supply-chain-refresh-derived data carries
   `ProvenanceTag::Derived { from: [supply_chain_refresh_transaction] }`.
   An auditor can verify which Parts were vetted by a known librarian
   versus auto-populated from an external API.

Datum's substrate provides the data shape; counterfeit-prevention
**process** is organisational. This is consistent with the broader
substrate-vs-certifier position.

### Electronic Records & Signatures

#### FDA 21 CFR Part 11 (the canonical spec)

**Full title.** **21 CFR Part 11** — *Electronic Records;
Electronic Signatures*. Effective 1997-03-20; FDA Guidance for
Industry "Part 11, Electronic Records; Electronic Signatures —
Scope and Application" (2003) provides the modern risk-based
interpretation that most pharmaceutical and medical-device firms
operate under. Current 2024 FDA Draft Guidance "Electronic Systems,
Electronic Records, and Electronic Signatures in Clinical
Investigations" extends application context.

**Issuing body.** **US FDA** (Food and Drug Administration), Center
for Drug Evaluation and Research / Center for Devices and
Radiological Health.

**Scope.** Specifies criteria under which electronic records and
electronic signatures are considered trustworthy, reliable, and
generally equivalent to paper records and handwritten signatures.
Applies to any FDA-regulated industry (pharmaceutical, medical-
device, biologics, food). Three subparts:

- **Subpart A — General Provisions** (§§ 11.1-11.3):
  - § 11.1 Scope.
  - § 11.2 Implementation (closed vs open systems).
  - § 11.3 Definitions.

- **Subpart B — Electronic Records** (§§ 11.10-11.30):
  - § 11.10 **Controls for closed systems** — validation, ability to
    generate accurate and complete copies, record protection,
    limiting system access, secure computer-generated time-stamped
    audit trails, operational system checks, authority checks,
    device checks, training, accountability for actions, controls
    over systems documentation. **This is the section Datum's
    substrate maps to.**
  - § 11.30 Controls for open systems — adds digital signatures to
    the § 11.10 list to provide record authenticity, integrity, and
    confidentiality during transmission. (Open systems = "an
    environment in which system access is not controlled by persons
    who are responsible for the content of electronic records on the
    system" — relevant if Datum is deployed in a multi-tenant
    cloud; less relevant for typical desktop deployments.)

- **Subpart C — Electronic Signatures** (§§ 11.50-11.300):
  - § 11.50 **Signature manifestations** — every signed record must
    contain (1) the printed name of the signer, (2) the date and
    time of signing, (3) the meaning associated with the signature
    (e.g., review, approval, responsibility, authorship). The
    information must be subject to the same controls as the record
    itself and must be included in any human-readable output.
  - § 11.70 **Signature/record linking** — electronic signatures
    must be linked to their respective electronic records to ensure
    that the signatures cannot be excised, copied, or transferred to
    falsify an electronic record.
  - § 11.100 **General requirements** — each electronic signature is
    unique to one individual; an organisation must verify identity
    before establishing the signer's electronic signature; certify
    in writing to the FDA that signatures are equivalent to
    handwritten ones.
  - § 11.200 **Electronic signature components and controls** — for
    non-biometric signatures, the signature consists of two distinct
    identification components (e.g., user ID + password); the first
    signing in a session uses both, subsequent signings use one but
    only if the components are not used by anyone else.
  - § 11.300 **Controls for identification codes/passwords** —
    uniqueness of identification codes, password aging, transaction
    safeguards, periodic testing of devices.

**Adoption status (2026).** **Mainstream-mandatory** for any
electronic record or electronic signature used in an FDA-regulated
context.

**License / IP.** **Free.** Public-domain US federal regulation at
`ecfr.gov/current/title-21/chapter-I/subchapter-A/part-11`. The 2003
Guidance for Industry (free PDF from `fda.gov`) is the de-facto
interpretation reference.

**EDA tool support matrix.**
- **Altium Designer + Altium Vault / Altium 365 Workspace** — claims
  21 CFR Part 11 substrate support via Vault sign-off workflow.
  Requires Vault paid subscription, organisational deployment, and
  validation effort by the customer. The certifying party remains
  the customer's QMS, not Altium.
- **Cadence Allegro + Pulse/Allegro Design Workbench** — integrates
  with Windchill / Teamcenter for 21 CFR Part 11 evidence; the EDA
  tool itself does not claim certification.
- **Mentor Xpedition + Teamcenter** — same pattern as Cadence.
- **OrCAD CIS** — provides change-tracking infrastructure; PLM
  integration adds signature workflow.
- **KiCad / Eagle/Fusion / Horizon / LibrePCB / DipTrace / EasyEDA** —
  no built-in 21 CFR Part 11 sign-off feature; reliance on git
  history is the closest substrate.
- **Datum (current spec)** — substrate exists (transaction model);
  no signature surface yet. The Phase 1 audit identified this as
  "21 CFR Part 11 / electronic-signature audit-trail compliance is
  implicitly enabled by Datum's deterministic transaction model but
  not explicitly claimed".

**Datum coverage status.** **`Deferred with prerequisite`** per
`STANDARDS_COMPLIANCE_SPEC.md` § 4.4 and § 4.8. **Promote to
`Planned` (substrate-only)** once this report's recommended spec
edits land. The substrate matches every § 11.10 closed-systems
control and every § 11.50 / § 11.70 signature requirement; the
remaining § 11.100 / § 11.200 / § 11.300 controls (signature
component management, password aging, organisational FDA
certification) are organisational/process-level and out of Datum's
scope.

**Per-clause Datum substrate mapping:**

| 21 CFR Part 11 clause | Datum substrate satisfying it |
|-----------------------|--------------------------------|
| § 11.10(a) Validation | Deterministic-replay verifier (this report) |
| § 11.10(b) Accurate complete copies | JSON serialisation + canonical-form determinism |
| § 11.10(c) Record protection | Per-Transaction `prev_hash` chain (this report) |
| § 11.10(d) Limit system access | OS-level access control + ActorIdentity binding |
| § 11.10(e) Audit trails | `Transaction { id, timestamp, actor, prev_hash, ops, description }` (this report) |
| § 11.10(f) Operational system checks | Validator + replay-verify |
| § 11.10(g) Authority checks | `EcoSignaturePolicy` per-state-transition role check (this report) |
| § 11.10(h) Device checks | Out of Datum scope — OS / IdP layer |
| § 11.10(i) Training | Out of Datum scope — organisational |
| § 11.10(j) Accountability | `ActorIdentity` on every Transaction (this report) |
| § 11.10(k) Systems documentation | This spec corpus |
| § 11.30 Open systems | XAdES / CAdES detached signatures for inter-system transmission |
| § 11.50(a)(1) Signer name | `Signature.signer.display_name` |
| § 11.50(a)(2) Date/time | `Signature.timestamp` (UTC) |
| § 11.50(a)(3) Signature meaning | `Signature.meaning: SignatureMeaning` enum |
| § 11.50(b) Subject to same controls | Signatures live in same chain as Transactions |
| § 11.50(c) Included in human-readable output | PAdES PDF export adapter (this report) |
| § 11.70 Signature/record linking | `Signature.signed_record_hash` cryptographically binds |
| § 11.100 Unique to individual | ActorIdentity uniqueness invariant + IdP delegation |
| § 11.200 Signature components | Out of Datum scope — IdP layer |
| § 11.300 Identification controls | Out of Datum scope — IdP layer |

**Datum implementation cost.** Covered by the consolidated
substrate work specified later. The 21 CFR Part 11 substrate is
the audit-log + Signature + ActorIdentity work; no Part-11-
specific code beyond the Signature meaning enum.

**Strategic recommendation.** **`Planned (substrate)`** after this
report's edits land. Marketing-grade wording: "21 CFR Part 11
substrate-compatible" — never "21 CFR Part 11 compliant", which is
reserved for the certifying party's organisational claim.

**Risks.** **AI-surface risk** as noted under ISO 9001. The MCP
wording rule MUST be: "your project uses Datum's 21 CFR Part 11
substrate features (signed audit trail with manifested signature
meanings); 21 CFR Part 11 compliance certification is conferred on
your organisation by your QMS, not by Datum."

#### EU Annex 11 (computerised systems — EU GMP equivalent)

**Full title.** **EudraLex Volume 4, Annex 11** — *Computerised
Systems*. Issued by the European Commission as part of the EU
Guidelines to Good Manufacturing Practice. Current revision 2011-06-30
(effective 2011-06-30, no substantive update since); is under EU GMP
modernisation review with no published draft amendment as of 2026-04.

**Issuing body.** **European Commission**, DG Health and Food
Safety. National GMP inspectorates enforce.

**Scope.** EU GMP equivalent of 21 CFR Part 11. Covers risk
management, personnel, suppliers and service providers, validation,
data, accuracy checks, data storage, printouts, audit trails,
change and configuration management, periodic evaluation, security,
incident management, electronic signature, batch release, business
continuity, archiving.

**Datum substrate alignment.** Substantively identical to 21 CFR
Part 11 § 11.10 + § 11.50 — same audit trail + signature + access
control requirements. The Datum substrate satisfying 21 CFR Part 11
satisfies EU Annex 11 by the same construction.

**Adoption status.** **Mainstream-mandatory** for pharmaceutical
manufacturers selling into the EU. Less commonly invoked for
medical devices (those follow MDR + ISO 13485 instead).

**License / IP.** **Free.** Public at
`health.ec.europa.eu/medicinal-products/eudralex/eudralex-volume-4_en`.

**Datum coverage status.** **`Reference-only`** under the same
substrate as 21 CFR Part 11. No separate Datum work.

**Strategic recommendation.** Document the substantive equivalence
to 21 CFR Part 11 in the spec; deliver the same substrate; let
users invoke either regulation.

#### eIDAS Regulation (EU 910/2014)

**Full title.** **Regulation (EU) No 910/2014** — *electronic
IDentification, Authentication and trust Services*. Current as
amended by Regulation (EU) 2024/1183 (eIDAS 2.0, in force from
2024-05-20 with phased implementation through 2026-05-20).

**Issuing body.** **EU** (European Parliament and Council); ENISA
(European Union Agency for Cybersecurity) provides technical
guidance.

**Scope.** Establishes the EU framework for electronic identification
and trust services for electronic transactions. Defines three tiers
of electronic signature:

- **Simple Electronic Signature (SES)** — any data in electronic
  form attached to or logically associated with other data which is
  used by the signatory to sign. Lowest legal weight; not equivalent
  to handwritten signature.
- **Advanced Electronic Signature (AES)** — uniquely linked to the
  signatory, capable of identifying the signatory, created using
  electronic signature creation data that the signatory can use under
  their sole control, and linked to the signed data such that any
  subsequent change is detectable. The technical realisations are
  defined by ETSI TS 119 312 (algorithm recommendations), ETSI TS
  119 102 (procedures), ETSI EN 319 132 (XAdES), ETSI EN 319 122
  (CAdES), ETSI EN 319 142 (PAdES).
- **Qualified Electronic Signature (QES)** — an AES created by a
  Qualified Signature Creation Device (QSCD — typically a smart
  card or USB HSM following the EN 419241 series Common Criteria
  Protection Profile) and based on a Qualified Certificate issued
  by a Qualified Trust Service Provider (QTSP — listed in the EU
  Trusted Lists at `webgate.ec.europa.eu/tl-browser/`). **A QES has
  the same legal effect as a handwritten signature throughout the
  EU** (Article 25(2)).

eIDAS 2.0 (2024) adds the **European Digital Identity Wallet** as a
unified citizen-credential framework, with QES issuance to wallet
holders.

**Adoption status (2026).** **Mainstream** in EU regulatory and
inter-organisational contexts. QTSPs include Italian InfoCert, German
D-Trust, Spanish Camerfirma, French CertEurope, Belgian QuoVadis,
Swedish Steria, etc. Cross-border recognition is enforced via the
EU Trusted Lists.

**License / IP.** **Free.** EU regulations at `eur-lex.europa.eu`;
ETSI standards free at `www.etsi.org/standards-search` (no
registration required for most ETSI EN documents).

**EDA tool support matrix.** **No EDA tool surveyed natively
supports eIDAS QES.** Most rely on external PDF signing (Adobe
Acrobat with QSCD; DocuSign EU; foreign PKCS#11 signing utilities)
applied to exported PDFs after the fact. This is a real gap and a
genuine differentiator opportunity for Datum.

**Datum coverage status.** **`Planned`** with this report's
`Signature` primitive. The signature primitive's algorithm enum
includes the ETSI TS 119 312 recommended set; PAdES / XAdES / CAdES
adapters route signatures into the canonical formats.

**Datum implementation cost.**
- **AES**: covered by the cross-cutting `Signature` primitive
  (any X.509 cert plus PKCS#11 token suffices). ~5 days for the
  PAdES/XAdES/CAdES adapter triplet plus ~3 days for the PKCS#11
  shim.
- **QES**: requires the user to obtain a QSCD and a Qualified
  Certificate from a QTSP, then point Datum at the QSCD via PKCS#11.
  No incremental Datum-side work beyond AES; the QES distinction is
  carried by the certificate's policy OID, which the PAdES adapter
  surfaces.

**Strategic recommendation.** **Implement the AES substrate now**
(part of the Domain 8 batch); document QES capability via PKCS#11 +
external QSCD/QTSP arrangement. This makes Datum the **only
open-source EDA tool with eIDAS QES capability** as a substrate.

**Risks.** **Cryptographic risk** if Datum claims QES capability
without actually invoking the PKCS#11 token correctly. Mitigation:
the QES path is opt-in (off by default); the validator suite tests
against the EU Trusted Lists API to verify certificate validity at
sign time. **Legal risk** if Datum presents a non-QES signature as
QES. Mitigation: the `Signature.assurance_level: AssuranceLevel`
field carries the explicit tier (`Simple | Advanced | Qualified`) and
the PAdES / XAdES / CAdES adapters surface that tier in the embedded
signature properties.

#### PKCS#11 (cross-ref Domain 4)

**Full title.** **OASIS PKCS #11 Cryptographic Token Interface
Standard, Version 3.0** (current). Originally RSA Security PKCS #11;
adopted by OASIS in 2013; v3.0 published 2020-06-23. Active TC.

**Issuing body.** **OASIS** PKCS 11 TC.

**Scope.** Defines a platform-independent C API for cryptographic
tokens (smart cards, HSMs, USB tokens). The standard interface for
delegating cryptographic operations (key generation, signing,
encryption, decryption, hashing) to external hardware.

**Datum integration.** Datum's `Signature` primitive treats the
signing operation as opaque — the engine produces the canonical-form
hash to be signed, hands it to a `SigningProvider` trait, and stores
the returned `signature_blob`. The `SigningProvider` is implemented
by:

- **Local-software provider** — Ed25519 / ECDSA via `ring` crate
  (BSD-style license). Suitable for non-regulated workflows.
- **PKCS#11 provider** — wraps any PKCS#11-compatible token via the
  `cryptoki` Rust crate (Apache-2.0/MIT). Suitable for AES and the
  pre-QES path.
- **External-process provider** — invokes a subprocess (`pkcs11-tool`
  / `signtool` / `osslsigncode`) for signature creation. Useful for
  air-gapped HSMs or organisations with established signing tooling.

**License / IP.** **Free.** OASIS standard at
`docs.oasis-open.org/pkcs11/pkcs11-base/v3.0/`. Reference
implementations (SoftHSM, OpenSC) under permissive licences.

**Adoption status (2026).** **Mainstream-mandatory** for any signed
content in a regulated workflow.

**Strategic recommendation.** **Implement** the PKCS#11 provider
behind the `SigningProvider` trait. Hardware-token support is the
table-stakes feature for 21 CFR Part 11 / eIDAS QES / FIPS 140
compliance.

#### PKCS#12 (keystore format)

**Full title.** **PKCS #12: Personal Information Exchange Syntax
Standard, Version 1.1** (RFC 7292; February 2014).

**Issuing body.** Originally RSA, now **IETF** (RFC 7292).

**Scope.** A binary format for storing private keys and X.509
certificates together with a passphrase. The dominant format for
exporting/importing certificates between PKI tools.

**Datum integration.** Datum reads PKCS#12 keystores via the `p12`
Rust crate (Apache-2.0/MIT) or via a subprocess shim to `openssl
pkcs12`. The keystore is a user-side artifact; Datum loads it on
demand for non-PKCS#11 signature workflows. Importantly, **Datum
never persists the private key** — the keystore is loaded into
memory, used to sign, then dropped.

**Strategic recommendation.** **Implement** as the lightweight
alternative to PKCS#11 for software-key signing. PKCS#11 is the
preferred path for hardware tokens; PKCS#12 covers the
software-keystore case.

#### X.509 (certificate format)

**Full title.** **ITU-T X.509 (10/2019)** — *Information technology
— Open Systems Interconnection — The Directory: Public-key and
attribute certificate frameworks*. The IETF profile is
**RFC 5280** (May 2008) — *Internet X.509 Public Key Infrastructure
Certificate and Certificate Revocation List (CRL) Profile*.

**Issuing body.** **ITU-T** (ITU Telecommunication Standardization
Sector); IETF profile via PKIX working group.

**Scope.** The certificate format that binds a public key to an
identity (a Distinguished Name + Subject Alternative Names) via a
trusted Certificate Authority (CA) signature. Foundation of every
real-world public-key infrastructure (TLS, S/MIME, code signing,
document signing, eIDAS QES).

**Datum integration.** Datum's `Signature.signer` carries the
certificate's Subject DN, the CA's Issuer DN, the certificate
serial number, and the not-after expiry date as denormalised
display fields. The full certificate chain is stored as part of the
signature blob (per PAdES / XAdES / CAdES requirements). Validation
follows RFC 5280: build the chain to a trust anchor, verify
signatures up the chain, check Certificate Revocation Lists (CRL)
or Online Certificate Status Protocol (OCSP) status, check Extended
Key Usage extensions for the document-signing OID
(`id-kp-emailProtection 1.3.6.1.5.5.7.3.4` for general document
signing; ETSI-defined OIDs for QES).

**License / IP.** **Free.** ITU-T standards free at `itu.int`; IETF
RFCs always free.

**Implementation libraries.** `x509-parser` (Rust, MIT), `webpki`
(Rust, ISC), `openssl` crate (Apache-2.0). Subprocess to `openssl`
binary for fallback.

**Strategic recommendation.** **Implement** as the certificate-
parsing backbone. RFC 5280 chain validation is non-trivial but
well-libraried; do not roll bespoke crypto.

#### PAdES (PDF Advanced Electronic Signatures, ETSI EN 319 142)

**Full title.** **ETSI EN 319 142-1 V1.2.1 (2024-09)** — *Electronic
Signatures and Infrastructures (ESI); PAdES digital signatures;
Part 1: Building blocks and PAdES baseline signatures*.
**ETSI EN 319 142-2** covers extended PAdES profiles.

**Issuing body.** **ETSI** (European Telecommunications Standards
Institute), TC ESI (Electronic Signatures and Infrastructures).

**Scope.** PDF-embedded electronic signatures, building on ISO 32000
(PDF format). PAdES baseline profiles (B-B / B-T / B-LT / B-LTA)
provide increasing levels of long-term validation evidence:
- **B-B** — basic signature with signing certificate.
- **B-T** — adds a timestamp.
- **B-LT** — adds full revocation data (CRL/OCSP) for long-term
  verification.
- **B-LTA** — adds long-term archival timestamps for extended
  preservation.

PAdES is the canonical signature format for human-readable
documents in EU regulatory contexts and is widely adopted in non-EU
contexts as well (US, UK, Asia).

**Datum integration.** Datum exports compliance-evidence bundles as
PAdES PDFs via a `pdf-writer` (Apache-2.0) + `signature` adapter.
The PDF embeds:
- The audit-log entries for the signed Transaction range.
- The user-readable summary (per § 11.50(c) "human-readable output").
- Embedded signatures at each B-B / B-T / B-LT / B-LTA level the
  user requests.

**License / IP.** **Free.** ETSI standards free at `etsi.org`.
PDF format ISO 32000 is paywalled (~CHF 200 from ISO Webstore) but
the PDF specification is also freely published as a "snapshot"
PDF on Adobe's website (PDF 1.7 reference).

**Strategic recommendation.** **Implement** as the primary
human-readable signed-evidence export format. PAdES is the
right answer for "I need a single signed PDF I can hand to my QMS".

#### CAdES (CMS Advanced Electronic Signatures, ETSI EN 319 122)

**Full title.** **ETSI EN 319 122-1 V1.2.1 (2024-09)** — *Electronic
Signatures and Infrastructures (ESI); CAdES digital signatures;
Part 1: Building blocks and CAdES baseline signatures*.

**Issuing body.** **ETSI** TC ESI.

**Scope.** CMS (RFC 5652 Cryptographic Message Syntax) detached
signatures with ETSI long-term-validation profile additions
(B-B / B-T / B-LT / B-LTA, mirroring PAdES). The signature is a
separate `.p7s` file alongside the original payload; the payload
itself is unmodified.

**Datum integration.** CAdES is the canonical detached-signature
format for arbitrary binary payloads. Datum uses CAdES for:
- Project-state-snapshot signatures (the canonical-JSON form of the
  full project state at sign time, hashed and signed).
- Per-Transaction signatures (the canonical-JSON form of a single
  Transaction, hashed and signed).
- Audit-log-bundle signatures (a JSONL audit-log file, hashed and
  signed in CAdES format alongside).

**License / IP.** **Free** at ETSI; underlying CMS RFC 5652 always
free.

**Strategic recommendation.** **Implement** as the canonical
detached-signature format for all non-PDF, non-XML payloads.

#### XAdES (XML Advanced Electronic Signatures, ETSI EN 319 132)

**Full title.** **ETSI EN 319 132-1 V1.2.1 (2024-09)** — *Electronic
Signatures and Infrastructures (ESI); XAdES digital signatures;
Part 1: Building blocks and XAdES baseline signatures*.

**Issuing body.** **ETSI** TC ESI.

**Scope.** XML-DSig (W3C Recommendation, 2008) plus ETSI long-term-
validation profile additions. The signature is embedded inside the
XML document being signed.

**Datum integration.** XAdES is the canonical signature format for
XML payloads. Datum uses XAdES for:
- IPC-2581 fab-data export (Domain 1) when sign-on-export is
  configured.
- IPC-1752A material-declaration export (Domain 5) when the project
  requires signed materials evidence.
- IPC-1755 supply-chain-traceability export (Domain 5) when AS9100
  supply-chain attestation is configured.

**License / IP.** **Free** at ETSI; underlying XML-DSig is W3C
Recommendation, always free.

**Strategic recommendation.** **Implement** as the XAdES adapter for
IPC-2581 / IPC-1752A / IPC-1755 export integration. The work is
~3 days once the cross-cutting `Signature` primitive lands.

#### ETSI TS 119 312 (algorithm and key-length recommendations)

**Full title.** **ETSI TS 119 312 V1.5.1 (2024-09)** — *Electronic
Signatures and Infrastructures (ESI); Cryptographic Suites*.

**Issuing body.** **ETSI** TC ESI.

**Scope.** Specifies cryptographic algorithm choices and key-length
minima for AES / QES signatures. Periodically refreshed as algorithms
weaken or new attacks emerge. The 2024 revision (V1.5.1) recommends:

- **Hash algorithms**: SHA-256, SHA-384, SHA-512, SHA3-256, SHA3-384,
  SHA3-512. SHA-1 and MD5 explicitly forbidden.
- **Signature algorithms**: RSA-PSS with 3072-bit keys (general-
  purpose), 4096-bit (high-assurance, long-term); ECDSA with NIST
  P-256 / P-384 / P-521; EdDSA with Ed25519 / Ed448. PKCS#1 v1.5 RSA
  acceptable for legacy interop only.
- **Symmetric algorithms** (only for hybrid encryption schemes,
  not directly relevant to signing): AES-256-GCM, ChaCha20-Poly1305.

**Datum integration.** Datum's `SignatureAlgorithm` enum matches
the ETSI TS 119 312 recommended set (RSA-PSS-3072, RSA-PSS-4096,
ECDSA-P-256, ECDSA-P-384, EdDSA-Ed25519). Algorithms outside the
recommended set return a validation warning at sign time.

**License / IP.** **Free** at ETSI.

**Strategic recommendation.** **Implement** the algorithm enum
matching ETSI TS 119 312; the implementation work is bounded by the
underlying crypto crates (`ring`, `ed25519-dalek`, `rsa`).

### Audit-Trail / Change-Control

#### CMII (cross-ref Domain 7)

**Full title.** **CMII (Configuration Management II)** — methodology
maintained by the **Institute for Configuration Management (ICM)**.
Current revision **CMII-100J (2018)**.

**Issuing body.** **Institute for Configuration Management (ICM)**;
not an ISO/IEC body. Methodology rather than standard.

**Scope.** A configuration-management methodology that emphasises
keeping the documented configuration the unique authoritative
source-of-truth for "what is to be built" — through closed-loop
change management, requirements traceability, and identity-of-truth
between requirements / design / build artifacts.

**Datum positioning.** CMII is methodology, not toolable directly.
Datum's substrate (deterministic transaction log, JSON-diffable
state, ECO grouping) is consistent with CMII principles but Datum
does not enforce CMII. **Domain 7 already classified this as
`Reference-only` (methodology, not toolable enforcement).**

**License / IP.** Methodology overview free at `icminstitute.org`;
the methodology itself is taught via paid ICM training and
certification.

**Strategic recommendation.** **`Reference-only`**. Document the
substrate-methodology alignment in the spec; do not attempt to
enforce CMII compliance algorithmically.

#### ISO 10007:2017

**Full title.** **ISO 10007:2017** — *Quality management —
Guidelines for configuration management*. Current revision the third
edition (2017).

**Issuing body.** **ISO** TC 176/SC 2.

**Scope.** Generic guidelines for configuration management, aligned
with ISO 9001 § 7.5 / § 8.5.6 and parallel to EIA-649C. Defines five
configuration-management activities: configuration-management
planning, configuration identification, change control, configuration
status accounting, configuration audit.

**Datum substrate alignment.** All five CM activities map onto
existing or this-report-recommended Datum primitives:
- **Configuration-management planning** — `ProjectCompliance.eco_workflow_required`
  (Domain 7) + this report's `EcoSignaturePolicy`.
- **Configuration identification** — UUIDs (`CANONICAL_IR.md` § 1).
- **Change control** — Transaction model + ECO grouping + signed
  approvals.
- **Configuration status accounting** — `query_audit_log` MCP tool
  (this report) + per-Object change-history queries.
- **Configuration audit** — deterministic-replay verifier (this
  report).

**License / IP.** **Paywalled** ~CHF 138 from ISO Webstore.

**Adoption status (2026).** Mainstream as a guideline reference;
rarely a certification target on its own (organisations align
configuration management to ISO 9001 / AS9100 / IATF 16949 directly).

**Datum coverage status.** **`Reference-only`**. Document the
substrate alignment in the spec.

**Strategic recommendation.** **Reference-only**; the cross-cutting
substrate satisfies it.

#### EIA-649C (Configuration Management — current US revision)

**Full title.** **EIA-649C (2019)** — *Configuration Management
Standard*. Current revision the third (2019); successor to EIA-649B
(2011).

**Issuing body.** Originally **Electronic Industries Alliance**;
maintained by **SAE International** since the EIA dissolved in 2011.

**Scope.** US-equivalent configuration-management standard. Five
principles parallel to ISO 10007: identification, change management,
status accounting, configuration verification and audit, document
management. Widely cited in US-defence procurements and aerospace
supply chains.

**Datum substrate alignment.** Same as ISO 10007 — same substrate
satisfies both.

**License / IP.** **Paywalled** ~USD 100 from SAE store. The historical
NIST cross-reference summary is a free reading aid.

**Datum coverage status.** **`Reference-only`**.

**Strategic recommendation.** Document substrate alignment alongside
ISO 10007.

#### MIL-HDBK-61A (note only)

**Full title.** **MIL-HDBK-61A (2001)** — *Military Handbook —
Configuration Management Guidance*. US Department of Defense.

**Adoption status.** Reference document for US-defence configuration
management; references EIA-649 as the underlying standard.

**License / IP.** **Free** at `everyspec.com`.

**Strategic recommendation.** **`Reference-only`** — note in the
spec as a US-defence configuration-management reference; substrate
covered via EIA-649C.

### Process Maturity (skip — organisational)

#### CMMI (Capability Maturity Model Integration for Development)

**Full title.** **CMMI for Development (CMMI-DEV) v3.0 (2023)**.
Current revision the v3 baseline (2023), maintained by **ISACA**
(formerly CMMI Institute).

**Issuing body.** **ISACA** (Information Systems Audit and Control
Association).

**Scope.** Process-maturity assessment framework. Five maturity
levels (Initial, Managed, Defined, Quantitatively Managed, Optimizing)
across multiple practice areas. Used by US DoD acquisitions and
many large IT/engineering organisations to assess supplier-process
maturity.

**Why advisory exclusion.** **Organisational process assessment
framework, not a tool feature.** CMMI assessments are conducted by
SCAMPI-certified assessors against the organisation's processes,
not its tools. A CMMI Level-3 organisation uses tools that
support its processes; CMMI does not specify tool requirements.

**Substrate positioning.** Datum's substrate is consistent with
CMMI Level-3 practice areas (Configuration Management, Verification,
Validation, Process and Product Quality Assurance) but provides no
algorithmic CMMI capability.

**License / IP.** **Paywalled** — CMMI v3 model document available
to ISACA members; non-members purchase per assessment.

**Re-affirmed exclusion.** No hidden cross-cutting value. Promote to
formal `Out of scope` in the consolidated ratification pass.

#### ISO/IEC 33000 series (process assessment — note only)

**Full title.** **ISO/IEC 33001:2015** — *Information technology —
Process assessment — Concepts and terminology*. Plus the ISO/IEC
330xx family (33002, 33003, 33004, 33020 etc.).

**Issuing body.** **ISO/IEC** JTC 1/SC 7.

**Scope.** Successor to ISO/IEC 15504 (SPICE — Software Process
Improvement and Capability dEtermination). Generic process-assessment
framework adopted by Automotive SPICE (ASPICE) for the automotive
industry.

**Datum positioning.** Same as CMMI — organisational process
framework, not tool feature.

**Strategic recommendation.** **`Out of scope`**. Note only.

#### Automotive SPICE (ASPICE) (note only)

**Full title.** **Automotive SPICE Process Reference Model and
Process Assessment Model V4.0 (2023)**.

**Issuing body.** **VDA** (Verband der Automobilindustrie) Quality
Management Center.

**Scope.** Automotive-industry adaptation of ISO/IEC 33020 process
assessment. Used by European automotive OEMs (especially German) to
assess Tier-N supplier process maturity for embedded-software /
electronic-system development.

**Datum positioning.** Software-process assessment; Datum is an EDA
tool. Cross-cutting only at the level of "the supplier uses Datum as
part of its tooling" — supplier-process certification is independent.

**Strategic recommendation.** **`Out of scope`**. Note only.

### Audit-Log Patterns

#### Tamper-evident logs (Merkle trees, hash chains)

**Concept.** A tamper-evident log is a log where any modification to
historical entries is detectable by verification of cryptographic
linkages between entries. Two canonical patterns:

- **Hash chain**: each entry includes the cryptographic hash of the
  immediately preceding entry. Modification of any historical entry
  invalidates every subsequent entry's hash-link. O(N) verification.
- **Merkle tree**: entries are leaves of a binary hash tree; the root
  hash summarises all entries. Verification of a single entry's
  inclusion is O(log N) via a Merkle proof. Used by certificate
  transparency logs and Trillian.

**Datum integration.** **Hash chain over Transactions** is the
recommended pattern: each persisted Transaction carries `prev_hash:
Option<[u8; 32]>` (the SHA-256 of the immediately preceding
Transaction's canonical-JSON form, or `None` for the first
Transaction). The chain head's hash uniquely summarises the entire
audit-log state. Verification is O(N) via replay; for typical
project sizes (hundreds to low-thousands of Transactions per
project) this is fast enough that interactive verification is
practical.

A Merkle tree is **over-engineered** for typical EDA-project audit
logs: the entry count is too small to benefit from O(log N)
inclusion proofs, and the simpler hash-chain is sufficient for
detection. Merkle trees become attractive only at the
Trillian / certificate-transparency scale (millions of entries).

**Adoption status.** Hash chains are mainstream in audit-log /
ledger / blockchain contexts; specific implementations vary.
git's commit graph is itself a hash chain (and Datum can leverage
this when project files live in a git repository).

**Strategic recommendation.** **Implement** the Transaction
`prev_hash` field; the chain is the substrate for tamper-evidence.
Optionally extend to a Merkle root summary for large projects (defer
until project size warrants).

#### Append-only logs

**Concept.** A log where entries can only be added, never modified
or removed. Implementation strategies range from filesystem-level
(POSIX `O_APPEND` plus chattr +a immutable bit on Linux) to
application-level (a write-only API surface) to storage-level (cloud
object storage with object-lock / write-once-read-many policies).

**Datum integration.** Datum's audit log is application-level
append-only: the engine API has no `delete_transaction` or
`modify_transaction` operation. The persisted file layout
(`<project>/audit_log/<transaction-uuid>.json`, one file per
Transaction) supports filesystem-level append-only enforcement if
the deployment environment configures it (chattr +a or equivalent).

**Combined with hash-chain**: append-only is the access-control
property; hash-chain is the cryptographic property. Both are
recommended; neither alone is sufficient.

**Strategic recommendation.** **Document** the application-level
append-only invariant; **document** the optional filesystem-level
enforcement; **link** to the hash-chain primitive for cryptographic
detection.

#### Git-as-audit-log

**Concept.** Use a git repository as the audit-log substrate.
git's commit DAG is itself a hash chain; signed git tags
(`git tag -s`) provide GnuPG-signed referential integrity over
commit ranges; the content-addressed object store gives
deduplication and tamper-detection.

**Datum integration.** Datum's project file format is JSON, deliberately
designed to be git-friendly (sorted map keys, deterministic
serialisation per `CANONICAL_IR.md` § 5). A git repository tracking
a Datum project IS a tamper-evident audit log for free. The Datum
audit-log export can map one-to-one onto git commits (one commit
per Transaction) with the commit message carrying the Transaction's
`description` and the Transaction `id` in the commit trailer.

**Recommendation flow.**
- **Default mode**: Datum writes Transaction files to
  `<project>/audit_log/<transaction-uuid>.json`; git is not required.
- **Git-integrated mode**: a CLI subcommand `datum-eda audit
  git-commit` materialises the audit log as git commits in the
  project's git repository. Each commit has trailer
  `Datum-Transaction-Id: <uuid>` and `Datum-Prev-Hash: <hex>`.
  Signed commits via `git commit -S` or `git tag -s` add the
  cryptographic signature layer.

**Strategic recommendation.** **Document and ship** the
git-integrated mode as an optional adapter. The git ecosystem is
universal; integrating with it gives users the option to leverage
existing organisational git infrastructure.

**License/IP.** git is GPL-2 — but invoking git as a subprocess is
license-clean (no linkage). The `git2` Rust crate (libgit2 bindings)
is GPL-2 with linking exception; the linking-exception terms are
permissive enough for Datum's distributable-binary requirements,
but the safe pattern is subprocess invocation.

#### Trillian / sigstore / rekor

**Concept.** **Trillian** (Google, Apache-2.0) is the
verifiable-log infrastructure underpinning **certificate transparency
logs** (Google CT, Cloudflare CT, etc.) and **rekor** (the sigstore
project's public verifiable log for code-signing transparency).
Trillian provides a public append-only Merkle-tree log with
public verifiability — anyone can audit the log's consistency
without trusting the operator.

**Datum integration.** Datum's per-project audit log is private by
default; an optional opt-in path can submit Transaction hashes to
**rekor** as a public timestamp anchor. The submission is minimal
(send the SHA-256 of the canonical-form Transaction; rekor returns a
signed inclusion proof). The rekor-submitted hash provides a
tamper-evident timestamp **without revealing the contents** of the
Transaction — useful for long-term audit posture in regulated
industries.

**Strategic recommendation.** **Optional opt-in**. Document the
rekor submission flow; ship as a thin adapter. Most users will not
use this; for those who do, the public-verifiable timestamp is a
strong evidence augmentation for high-stakes compliance contexts
(eIDAS QES augmentation, archival evidence).

**License/IP.** Trillian / rekor / sigstore are Apache-2.0; the
client libraries are equally permissive.

#### In-toto attestation framework

**Concept.** **in-toto** (CNCF graduated project, 2017+,
Apache-2.0) is a framework for cryptographically attesting to the
provenance of software artifacts through a build/release pipeline.
The attestation format (in-toto Attestation Framework v1.0) is
JSON-based; predicates are pluggable.

**Datum integration.** Cross-references Domain 5 (supply chain) and
the broader software-supply-chain story. Datum can emit in-toto
attestations covering the design-to-fab handoff: an attestation
predicate `https://datum-eda.org/attestations/design-snapshot/v1`
that binds the project state hash to the signing user's
ActorIdentity at a specific timestamp. Consumers of the attestation
(downstream fab houses, customers performing supply-chain audit)
can verify the predicate without needing to consume Datum's full
project state.

**Strategic recommendation.** **Optional adapter**. Document the
in-toto integration path; ship if a customer requests SLSA-track
software-supply-chain attestation for design artifacts.

**License/IP.** in-toto and its libraries are Apache-2.0.

#### SLSA (Supply-chain Levels for Software Artifacts)

**Concept.** **SLSA v1.0 (2023)** is a framework specifying
Supply-chain Levels for Software Artifacts — four levels (L0-L4) of
build-pipeline security. Hosted by **OpenSSF** under the Linux
Foundation.

**Datum relevance.** SLSA is software-supply-chain (build pipeline);
Datum is an EDA tool. Cross-cutting only at the level of "the Datum
binary itself is built with SLSA-compliant pipelines" — a Datum-
distribution concern, not a Datum-feature concern.

**Strategic recommendation.** **Out of Datum scope** at the feature
level; relevant to Datum's own build-and-release engineering as the
project matures. Note.

### Identity / Authentication (cross-ref Domain 7)

#### OAuth 2.0 / OIDC

**Full title.**
- **OAuth 2.0** — RFC 6749 (2012) + RFC 7636 PKCE (2015) +
  RFC 8252 (2017, native apps) + RFC 9700 (2024, security best
  practice). **OAuth 2.1** is the consolidated revision in IETF
  draft as of 2026-04 (`draft-ietf-oauth-v2-1`); near final.
- **OIDC** — OpenID Connect Core 1.0 (2014, with errata set 2 in
  2020). The OIDC Working Group (OpenID Foundation) maintains.

**Issuing body.** **IETF** (OAuth) and **OpenID Foundation** (OIDC).

**Scope.** OAuth 2 is the authorisation-delegation framework;
OIDC layers identity-assertion (the ID Token) on top. Together they
are the modern web-identity-federation stack.

**Datum integration.** Datum's identity layer (daemon-side) supports
OAuth 2.1 / OIDC 1.0 with PKCE for the authorisation code flow.
Implementations:
- **Octopart / Nexar** — OAuth 2.0 client credentials.
- **Digi-Key API** — OAuth 2.0.
- **Mouser API** — API-key (legacy; no OAuth at the time of writing).
- **Most cloud PLM (Arena, OpenBOM)** — OAuth 2.0 / OIDC.
- **Generic OIDC IdP** (Google, Microsoft Entra, Okta, Auth0,
  Keycloak) — OIDC 1.0.

The daemon-side identity provider implements the OIDC Authorization
Code Flow with PKCE; the resulting ID Token populates `ActorIdentity`
(`identity_provider: IdpKind::Oidc`, `idp_user_id` from the
`sub` claim, `display_name` from the `name` claim).

**License / IP.** **Free.** RFCs and OIDC specifications free at
`ietf.org` and `openid.net`.

**Adoption status (2026).** **Mainstream-mandatory** for any web
identity federation. Every major cloud IdP and SaaS authenticates
via OAuth 2.0 / OIDC.

**Implementation libraries.** `oauth2` Rust crate (MIT-OR-Apache-
2.0), `openidconnect` Rust crate (MIT-OR-Apache-2.0).

**Strategic recommendation.** **Implement** as the primary
federated-identity path. The daemon hosts the authentication state;
the engine consumes the resulting `ActorIdentity` opaque record.

#### SAML 2.0

**Full title.** **SAML 2.0** — *Security Assertion Markup Language
2.0* (OASIS Standard, March 2005). The dominant enterprise-SSO
protocol pre-OIDC.

**Issuing body.** **OASIS** Security Services Technical Committee.

**Scope.** XML-based assertion framework for cross-domain identity
federation. Used by enterprise SSO (Active Directory Federation
Services, Okta, OneLogin, Auth0 SAML mode, Shibboleth).

**Datum integration.** SAML is the dominant identity protocol in
enterprise environments (especially aerospace, defence, large
corporations). Datum's daemon-side identity provider can implement
SAML 2.0 SP (Service Provider) mode for enterprise-SSO integration.

**License / IP.** **Free.** OASIS standards at
`docs.oasis-open.org/security/saml/v2.0/`.

**Adoption status (2026).** **Mainstream** in enterprise contexts;
declining in favour of OIDC for new deployments but well-entrenched
in Windchill / Teamcenter / Aras / large-corporation environments.

**Implementation libraries.** `samael` Rust crate (Apache-2.0 / MIT)
for SAML SP / IdP. **Note:** SAML implementation is significantly
more complex than OIDC due to the XML signature/canonicalisation
requirements and the historical XML-DSig vulnerability surface
(XML signature wrapping attacks). Use a well-tested library; do not
roll bespoke XML signature handling.

**Strategic recommendation.** **`Deferred with prerequisite`**. The
SAML SP integration is significant work; ship after OIDC is
established and only when a customer demands it. Domain 7 already
classified vendor-PLM connectors (Windchill, Teamcenter, Aras) as
`Out of scope`, and the SAML use case is largely those connectors;
ship SAML when the connectors are commissioned.

#### WebAuthn / FIDO2 (emerging passwordless)

**Full title.** **WebAuthn Level 3 (W3C Recommendation, 2024)** +
**FIDO2 CTAP 2.2 (FIDO Alliance, 2023)**.

**Issuing body.** **W3C** (WebAuthn) + **FIDO Alliance** (FIDO2 / CTAP).

**Scope.** Passwordless authentication framework using public-key
cryptography. The user authenticates by demonstrating possession of
a private key (typically held in a hardware authenticator — a USB
key, a phone secure enclave, a platform authenticator like Apple
Touch ID / Windows Hello). The relying party verifies the public
key signature without ever seeing the private key.

**Datum integration.** WebAuthn is an emerging signature-binding
mechanism. For Datum's signature substrate, WebAuthn provides:
- A **strong second factor** for ActorIdentity establishment (login
  to Datum requires WebAuthn assertion in addition to OIDC).
- An **alternative signing pathway** for environments where PKCS#11
  hardware tokens are inconvenient — a USB FIDO2 key can sign Datum
  artifacts via the WebAuthn assertion mechanism.

The cryptographic primitives are similar to PKCS#11 (the key never
leaves the hardware), but the abstractions are different (WebAuthn
is web-browser-centric; PKCS#11 is C-API-centric).

**Adoption status (2026).** **Mainstream-emerging** for consumer
authentication; beginning to penetrate enterprise environments
(Microsoft Entra Conditional Access, Okta WebAuthn). FIDO2 hardware
keys (YubiKey 5 series, SoloKeys, Google Titan) are widely
available.

**Implementation libraries.** `webauthn-rs` Rust crate (MPL-2.0);
not a hard fit for Datum's existing license posture, but the MPL is
compatible at the file boundary. Alternative: subprocess to a
WebAuthn shim.

**Strategic recommendation.** **`Optional, deferred`**. WebAuthn is
emerging; the value-add is real but the user demand is low. Ship as
an optional second-factor for Datum's identity provider and as an
alternative signing pathway when a customer requests it.

#### mTLS (mutual TLS)

**Full title.** Mutual TLS authentication — the variant of TLS
(TLS 1.3, RFC 8446) where both client and server authenticate each
other via X.509 certificates.

**Scope.** Service-to-service authentication. Used in
service-mesh / microservices contexts and in some PLM-connector
integrations.

**Datum integration.** Datum's daemon talks to backend services
(supply-chain APIs, PLM connectors) via TLS; mTLS adds the daemon's
own certificate to the server's authentication path. Useful for
PLM-connector deployments where the PLM enforces certificate-based
service identity.

**Strategic recommendation.** **Document** the mTLS support path;
ship as a configuration option on the daemon's HTTP client. No
substantial Datum-side work beyond the configuration knob.

#### PKCS#11 — hardware-token auth (cross-ref Domain 4)

Already covered above under Electronic Records & Signatures. Cross-
references this section as the hardware-token authentication
mechanism for both signing and identity establishment.

### Sign-Off / Approval Workflow

#### Single approver / multi-signature / role-based patterns

**Patterns surveyed:**

- **Single-approver workflow** — one approver's signature is
  sufficient. The simplest pattern, common for low-risk changes
  (consumer-electronics review-and-release).
- **N-of-M multi-signature** — at least N of M designated approvers
  must sign before the change is approved. Used in high-risk
  contexts (medical device design control, safety-critical avionics
  — N typically 2 of 3 or 2 of 4).
- **Role-based** — approval requires signatures from specific roles
  (designer, reviewer, approver, releaser). Used in formal QMS
  contexts (AS9100D, ISO 13485). Roles map to distinct
  ActorIdentity records via IdP group membership.
- **Sequential vs concurrent** — sequential approvals occur in
  order (designer signs, then reviewer, then approver); concurrent
  approvals can occur in any order.

**Datum integration.** All four patterns are expressed by the
`EcoSignaturePolicy` data shape:

```rust
pub struct EcoSignaturePolicy {
    /// Per-state-transition required signatures.
    pub transitions: HashMap<EcoTransition, Vec<RequiredSignature>>,
}

pub enum EcoTransition {
    SubmitForReview,    // Draft -> Submitted
    BeginReview,        // Submitted -> InReview
    Approve,            // InReview -> Approved
    Reject,             // InReview -> Rejected
    MarkImplemented,    // Approved -> Implemented
    Close,              // Implemented -> Closed
}

pub struct RequiredSignature {
    pub meaning: SignatureMeaning,        // e.g., Reviewed, Approved
    pub role: Option<String>,             // e.g., "SI/PI Engineer"
    pub minimum_count: u32,               // N for N-of-M
    pub from_role_pool: Option<Vec<String>>, // M designated roles for N-of-M
    pub allow_self_approval: bool,        // can the submitter also approve?
}
```

This shape covers single-approver (`minimum_count: 1`), N-of-M
(`minimum_count: N`, `from_role_pool: Some([role1, role2, ...])`),
role-based (`role: Some("Approver")`), and sequential (multiple
`RequiredSignature` entries with `meaning` ordered) cases. The
`allow_self_approval: false` flag prevents the submitter from being
counted toward the approval threshold (a common segregation-of-
duties requirement).

**Strategic recommendation.** **Implement** the
`EcoSignaturePolicy` data shape; default policies (single-approver,
2-of-3, role-based) ship as documented templates. Custom policies
are project-defined.

#### Workflow tool integration patterns (JIRA / ServiceNow / Salesforce)

**Patterns.** External workflow tools (JIRA, ServiceNow, Salesforce
CRM, ProjectWise, Polarion) typically integrate with EDA via:
- **Inbound webhook** — the workflow tool calls Datum's MCP API to
  query state or trigger an action ("query open ECOs", "mark this
  ECO approved").
- **Outbound event feed** — Datum's event log is polled / subscribed
  to by the workflow tool for change notifications ("new ECO
  submitted in Datum").

**Datum integration.** Datum's MCP / daemon API is the inbound
interface; the audit-log JSONL files (with optional rekor anchoring
or git commit) are the outbound interface. **Datum does not
re-implement workflow tooling** — Domain 7 already classified
workflow-engine integration as `Out of scope`. Datum exposes the
data hooks; the workflow tool plugs in.

**Strategic recommendation.** **Document** the inbound MCP +
outbound JSONL integration patterns; ship a sample webhook
integration recipe as documentation. Per-tool connectors are
out of scope (per Domain 7).

#### Pull-request-as-sign-off (git-native pattern)

**Pattern.** A pull request (or merge request) is the sign-off
mechanism: the author proposes a change, reviewers comment and
approve, and a maintainer merges. The git history captures the
authoring + review + approval signatures via signed commits and
signed merge commits.

**Datum positioning.** This pattern is **incompatible with Datum's
direct-to-main posture** per the project owner's standing
preference (`feedback_no_pull_requests` memory). Datum projects
operate on direct commits to main with prior agreement; the PR
ceremony is not used.

**However**, Datum's ECO substrate **emulates the same workflow
shape** without requiring git PR ceremony: an ECO is the proposed
change, ECO reviewers add `Reviewed` signatures, the ECO approver
adds the `Approved` signature, and the ECO transitions to
Implemented. The audit-trail evidence is identical to a git PR
review's evidence — and crucially, it works the same way for
non-git deployments.

**Strategic recommendation.** **Document** the conceptual parallel;
**implement** the ECO substrate as the canonical mechanism;
**optionally bridge** to git via the git-integrated audit-log mode
(above) for organisations that want both mechanisms in parallel.

### AI-Native Audit Considerations

#### AI-explanation surface as informational metadata

**Pattern.** AI-narrated explanations of engine output (rule
diagnostics, supply-chain refresh results, length-match tolerance
derivations) are useful to humans but are **non-authoritative**
records — the LLM's text is convenience, not engineering evidence.

**Datum integration.** AI-explanation outputs carry
`ProvenanceTag::AiSuggestion { model: String, prompt_hash:
String }` and are stored separately from canonical engine outputs.
The audit-log surface marks them with a distinct `event` field
(`AuditEventKind::AiExplanationEmitted`) so a compliance-reviewer
filter can include or exclude them. **The reviewer's default is to
exclude AI explanations from the canonical record**; the AI text is
a sidecar to the engineering evidence.

**Strategic recommendation.** **Specify** the data marker; **ship**
the audit-log filter; **document** the wording rule for AI agents
("explanations are convenience text; the rule output / engineering
result is the authoritative record").

#### MCP-tool egress consultation

**Pattern.** Every MCP tool with external-network or external-AI
side effects must consult the project's `data_egress_policy` before
execution and audit-log the decision. This is the cross-cutting
data-egress audit specified by Domain 4 and ratified by Domain 7;
Domain 8 owns the audit-log entry contract.

**Strategic recommendation.** **Implement** the
`AuditEventKind::ExternalCallGated` event type with required
sub-fields `{ tool, policy_at_call_time, decision, snapshot_hash }`.
This is the mechanism by which an auditor verifies "did any
external call occur on this ITAR-marked project?" — a query
against the audit log returns a definitive answer.

## The AI-Output Guard-Rail

This is the **CRITICAL** Domain-8 specification section. The Domain 6
research surfaced the underlying observation; Domain 8 specifies the
data shape that enforces it.

### The observation

LLM output is **categorically different** from human-authored input
or engine-derived data. Three reasons:

1. **Non-determinism.** Two invocations of the same LLM with the
   same input may produce different outputs. This is incompatible
   with Datum's determinism invariant for canonical state.
2. **Ungrounded confidence.** LLMs can produce confidently-stated
   wrong answers (hallucinations). Engineering evidence requires
   grounded statements.
3. **Authority absence.** An LLM is not a licensed engineer, not a
   designated reviewer, not a signature-capable ActorIdentity. The
   LLM cannot **authorise** anything.

These three properties make AI output **incompatible with the
authoritative-record role** in any compliance regime. The audit-trail
data model must distinguish authoritative from non-authoritative
bytes — and must do so at the type level, so a reviewer querying the
audit log gets the distinction without ambiguity.

### The Domain-6 finding (verbatim)

> **AI-explanation outputs are not authoritative records.** When the
> AI agent emits "the length-match tolerance is 5 mil because IBIS
> attachment X says R_pkg=2.5 nH", that text is a **convenience
> explanation**, not an authoritative engineering record. The
> authoritative record is the rule-pack JSON + the IBIS attachment +
> the rule-engine output. Domain 8's audit-log treatment of
> AI-explanation text should treat it as informational metadata, not
> as audit-trail evidence. (Strong guard-rail: AI agents are
> observers + assistants, not authority.)

### The data marker — `ProvenanceTag` enum

```rust
/// Tracks the origin of every data element in Datum's authored or
/// derived state. Carried as a field on every authored object that
/// can be the result of an AI suggestion, on every derived object
/// for traceability, and on every audit-log entry.
pub enum ProvenanceTag {
    /// User-entered, the result of an authored Operation by a
    /// human ActorIdentity. Authoritative.
    Authored,

    /// Deterministically computed from authored data. Authoritative
    /// because it is reproducible from authored inputs alone.
    Derived {
        /// The authored Transaction(s) this derived value depends on.
        from: Vec<Uuid>,
    },

    /// LLM-generated suggestion that has NOT been validated by a
    /// human ActorIdentity. NOT authoritative. Stored for context;
    /// must not be mistaken for authoritative engineering evidence.
    AiSuggestion {
        /// Model identifier (e.g., "claude-opus-4-7", "gpt-5",
        /// "local/llama-3-70b-instruct").
        model: String,
        /// SHA-256 of the prompt that produced this suggestion;
        /// allows the suggestion to be reproduced offline.
        prompt_hash: [u8; 32],
        /// Wall-clock timestamp of suggestion emission.
        at: DateTime<Utc>,
    },

    /// LLM-suggested data that has been explicitly validated by a
    /// human ActorIdentity. Treated as Authored for downstream
    /// purposes BUT retains the AI-origin pointer for audit
    /// traceability — a reviewer can answer "where did this data
    /// originally come from?" by following the back-pointer.
    UserValidatedAi {
        /// Reference to the originating AiSuggestion record.
        ai_origin: AiSuggestionRef,
        /// The ActorIdentity that validated.
        validated_by: ActorIdentity,
        /// Timestamp of validation.
        validated_at: DateTime<Utc>,
        /// Optional rationale for accepting the AI suggestion.
        rationale: Option<String>,
    },
}

pub struct AiSuggestionRef {
    pub suggestion_id: Uuid,             // pointer to the AiSuggestion record
    pub model: String,                   // model snapshot at suggestion time
    pub prompt_hash: [u8; 32],
    pub originally_at: DateTime<Utc>,
}
```

### Behavioural rules

The `ProvenanceTag` is not just metadata — it drives engine and MCP
behaviour:

1. **AI-suggested data is never written to canonical state.** When
   an AI agent emits a suggestion that would modify canonical state
   (e.g., "set this NetClass's PHY profile to DDR4_3200"), the
   engine **does not apply the change**. Instead, it stores the
   suggestion in a sidecar `pending_ai_suggestions` collection and
   surfaces it for user review. The user's `accept_ai_suggestion`
   MCP call promotes the suggestion: a new Operation is executed
   with the suggested values, and the OpDiff carries
   `provenance: ProvenanceTag::UserValidatedAi { ai_origin, ... }`.

2. **AI-explanation text is a sidecar, not an audit entry.** When an
   AI agent emits explanatory text ("the length-match tolerance is
   5 mil because..."), the text is stored in a sidecar
   `ai_explanations/<event-id>.txt` file or returned as MCP tool
   output. It does NOT become an audit-log entry. The
   audit-log entry instead carries
   `AuditEventKind::AiExplanationEmitted { event_id, model,
   prompt_hash, at }` — the existence of an explanation is logged,
   the text content is sidecarred. Compliance reviewers can find
   the text on demand but it never pollutes the canonical record.

3. **AI agents cannot grant approvals.** The
   `EcoSignaturePolicy.RequiredSignature.role` cannot be satisfied
   by an AI agent's "signature" — only an ActorIdentity with
   `signature_capable: true` (typically requiring a PKCS#11 token
   or a certificate-bound IdP credential) can produce a
   `Signature`. AI agents do not have ActorIdentities with
   signature capability; the type system enforces this at the API
   boundary.

4. **AI-validated data is auditable end-to-end.** The
   `UserValidatedAi { ai_origin: AiSuggestionRef, validated_by,
   validated_at, rationale }` record provides a one-call answer to
   "where did this canonical data originally come from?" — the
   reviewer can trace from the canonical Datum state to the AI
   suggestion, including the prompt hash (so the suggestion can be
   reproduced offline by replaying the prompt against the same
   model), the model identifier, and the validating user's
   identity.

### MCP wording rules

When the AI agent emits any output that could be mistaken for
authoritative engineering evidence:

- "**Suggestion**: this NetClass should use the DDR4_3200 PHY
  profile" — clearly marked as a suggestion, not as a directive.
- "**Explanation**: the rule pack derived this tolerance from the
  IBIS attachment's `[Pin Mapping]` block; this is convenience
  text, not the authoritative rule output" — clearly marked as
  explanation, with reference to the authoritative source.
- "**Cannot perform**: I cannot grant approval signatures; please
  ask an authorised reviewer with a signing certificate to
  approve this ECO" — refusal of authority requests.

### Why this matters

The AI-output guard-rail is the **only** mechanism that lets
Datum credibly position itself as **AI-native AND
compliance-substrate-ready**. Every other commercial EDA tool that
has added AI features (Altium AD24's Co-Pilot, OrCAD X Presto's AI
features, Cadence's Allegro AI Assistant) treats AI output as
indistinguishable from user input — which is a category error for
regulated workflows. Datum's `ProvenanceTag` formalises the
distinction, and once formalised, downstream consumers (audit-log
reviewers, compliance officers, FDA inspectors) can rely on the
distinction.

This is a **type-system enforcement of an organisational
requirement** — the kind of thing that distinguishes serious
engineering tools from ones that bolted on AI features without
thinking through the compliance implications.

## Cross-Cutting Patterns

### Substrate-vs-certifier (extended to QMS)

The substrate-vs-certifier framing has now been applied across
five domains (4, 5, 6, 7, 8). The position is consistent:

- Datum is the **substrate**.
- The **certifying party** is always external (registrar, FDA
  inspector, IAQG audit, IATF audit, internal QA function under a
  recognised QMS).
- Datum makes substrate-ready claims ("21 CFR Part 11
  substrate-compatible") — never compliance claims ("21 CFR Part 11
  compliant").
- AI agents using Datum surfaces inherit the same wording rules.

### Audit-trail-as-deterministic-replay

(Detailed above.) The central differentiator. Datum can produce
**stronger evidence** than any incumbent EDA tool because the
deterministic-replay property lets auditors re-derive state, not
just read events.

### Audit-log export contract

The canonical export format is **JSON-Lines, one entry per
Transaction**, persisted to `<project>/audit_log/<transaction-uuid>.json`
with a daily-rolled-up `audit_log/index.jsonl` for fast scanning.
Adapters export to:
- **CSV** — flat denormalised view, one row per
  Transaction × ChangedObject pair, columns matching common QMS
  reviewer expectations.
- **PAdES PDF** — single signed PDF carrying a date-range slice of
  audit log entries, signature manifestation per § 11.50, and
  embedded long-term-validation data per PAdES B-LTA.
- **In-toto attestation** (optional) — predicate
  `https://datum-eda.org/attestations/audit-snapshot/v1` for
  software-supply-chain-aware consumers.

### Signature substrate (Signature primitive)

```rust
pub struct Signature {
    /// Who signed.
    pub signer: ActorIdentity,

    /// Algorithm used.
    pub algorithm: SignatureAlgorithm,

    /// The opaque signature blob (PKCS#11 / PKCS#12 / WebAuthn /
    /// software signing produces this).
    pub signature_blob: Vec<u8>,

    /// SHA-256 of the canonical-form serialisation of the record
    /// being signed. Cryptographically binds the signature to the
    /// record state; satisfies 21 CFR Part 11 § 11.70.
    pub signed_record_hash: [u8; 32],

    /// What the signature means (Authored / Reviewed / Approved /
    /// Released / Witnessed); satisfies 21 CFR Part 11 § 11.50.
    pub meaning: SignatureMeaning,

    /// Wall-clock timestamp.
    pub timestamp: DateTime<Utc>,

    /// Optional: countersignatures by other parties. Composes
    /// multi-signature workflows naturally without a separate
    /// type. The order of countersignatures is the order in which
    /// they were applied.
    pub countersignatures: Vec<Signature>,

    /// Optional: assurance-level claim (Simple / Advanced /
    /// Qualified per eIDAS).
    pub assurance_level: AssuranceLevel,

    /// Optional: external timestamp anchor (RFC 3161 TSA token or
    /// rekor inclusion proof).
    pub external_anchor: Option<ExternalTimestampAnchor>,
}

pub enum SignatureAlgorithm {
    RsaPssSha256,    // RSA-PSS-3072 SHA-256
    RsaPssSha384,    // RSA-PSS-3072 SHA-384
    RsaPssSha512,    // RSA-PSS-4096 SHA-512
    EcdsaP256Sha256,
    EcdsaP384Sha384,
    EcdsaP521Sha512,
    EdDsaEd25519,
    EdDsaEd448,
}

pub enum SignatureMeaning {
    Authored,
    Reviewed,
    Approved,
    Released,
    Witnessed,
    /// Custom meaning (e.g., "SI/PI Engineer Sign-Off"); free-form
    /// per organisation.
    Custom(String),
}

pub enum AssuranceLevel {
    Simple,      // SES per eIDAS
    Advanced,    // AES per eIDAS
    Qualified,   // QES per eIDAS — requires QSCD + QTSP cert
}

pub enum ExternalTimestampAnchor {
    Rfc3161 {
        tsa_url: String,
        tsa_response: Vec<u8>,        // RFC 3161 TimeStampResp
    },
    SigstoreRekor {
        log_url: String,
        log_index: u64,
        inclusion_proof: Vec<[u8; 32]>,
    },
}
```

### ActorIdentity model

```rust
pub struct ActorIdentity {
    /// Datum-internal stable UUID for this actor. Survives IdP
    /// rotation, display-name changes, etc.
    pub id: Uuid,

    /// Human-readable display name (e.g., "Jane Doe").
    pub display_name: String,

    /// IdP that authenticated this actor.
    pub identity_provider: IdpKind,

    /// Stable user identifier within the IdP (e.g., OIDC `sub`
    /// claim, SAML NameID, OS username for Local).
    pub idp_user_id: String,

    /// Whether this actor can produce Signature records (i.e., has
    /// an associated signing certificate or hardware token).
    pub signature_capable: bool,

    /// When this ActorIdentity was first attached to the project.
    pub attached_at: DateTime<Utc>,

    /// Optional: roles assigned to this actor (used by
    /// EcoSignaturePolicy role-based approval rules).
    pub roles: Vec<String>,
}

pub enum IdpKind {
    /// Local OS-username-based identity. No federation. Suitable
    /// for solo / air-gapped use.
    Local,

    /// OIDC 1.0 federation.
    Oidc { issuer_url: String },

    /// SAML 2.0 federation.
    Saml { entity_id: String },

    /// PKCS#11 hardware-token-bound identity (the certificate's
    /// Subject DN is the identity).
    Pkcs11 { token_label: String, key_label: String },

    /// WebAuthn / FIDO2 authenticator binding.
    WebAuthn { credential_id: Vec<u8> },
}
```

### Workflow gates (cross-ref Domain 7 ECO substrate)

ECO state machine (Domain 7):

```
Draft -> Submitted -> InReview -> { Approved -> Implemented -> Closed
                                  | Rejected -> Draft (re-author) }
```

State transitions are constrained by `EcoSignaturePolicy` (above):
each transition specifies required signature meanings and roles;
the engine refuses the transition if the required signatures are
missing.

The Signature records on an ECO are detached: they live in
`<project>/ecos/<eco-uuid>/signatures/<signature-uuid>.signature.json`
alongside the ECO body. The ECO's `signed_record_hash` covers the
ECO body's canonical-JSON form. Modifying an ECO post-approval
invalidates the signature; this is detectable.

### Lint diagnostics as audit events (cross-ref Domain 5)

Every lint diagnostic emitted by a validator (`validate_project_compliance`,
`validate_controlled_impedance_audit`, `validate_length_match_audit`,
`validate_project_rohs_compliance`, etc.) produces an audit event:

```rust
AuditEventKind::LintFinding {
    severity: Severity,                // Error | Warning | Info
    rule_id: String,                   // e.g., "M5-RoHS-001"
    finding_id: Uuid,                  // unique to this finding occurrence
    object_uuid: Option<Uuid>,         // the design object the finding applies to
    message_canonical: String,         // canonical text — NOT AI-generated
}
```

Waivers produce a parallel event:

```rust
AuditEventKind::WaiverGranted {
    finding_id: Uuid,                  // the LintFinding being waived
    waiver_signature: Signature,       // the waiver IS a signature event
    rationale: String,                 // mandatory free-text justification
    expires_at: Option<DateTime<Utc>>, // optional expiry for re-review
}
```

This gives compliance reviewers a single query (`query_audit_log
--event-type LintFinding,WaiverGranted --object project_uuid`) that
returns the entire compliance posture trail for a project.

### Tamper-evidence model

(Detailed above.) Hash chain over Transactions via `prev_hash:
Option<[u8; 32]>` field; verification is O(N) replay; optional
external anchoring via RFC 3161 TSA or sigstore rekor.

### AI-output guard-rail data marker

(Detailed in § "The AI-Output Guard-Rail" above.) `ProvenanceTag`
enum with four variants (`Authored | Derived | AiSuggestion |
UserValidatedAi`); enforced at the type level; AI agents cannot
write canonical state without user validation.

### Data-egress audit (cross-ref Domains 4, 7)

Every external-network MCP tool consults
`Project.compliance.data_egress_policy` before execution and produces
an audit-log entry capturing the policy state at call time:

```rust
AuditEventKind::ExternalCallGated {
    tool: String,                      // e.g., "lookup_part_octopart"
    policy_at_call_time: DataEgressPolicy,
    policy_snapshot_hash: [u8; 32],    // SHA-256 of the policy serialisation
    decision: GateDecision,            // AllowedExplicit | AllowedByDefault | Blocked
    target_url: Option<String>,        // if Allowed; the URL that was contacted
    response_summary: Option<String>,  // brief: "received N parts" — NEVER full payload
}

pub enum GateDecision {
    AllowedExplicit,    // policy explicitly permits this tool
    AllowedByDefault,   // policy is Unrestricted (default)
    Blocked,            // policy refuses; tool returned an error
}
```

Auditors verifying ITAR / EAR / EU dual-use compliance posture have
a one-query answer: "did any external-network call occur on this
project, and was the policy state consistent with the project's
declared posture?"

## Cross-Domain Consolidation (Domain 8 Owns)

This is the **integrative payoff** of the entire 8-domain sequence.
Domain 8 owns the contracts that Domains 2, 4, 5, 6, 7 specified
their dependencies on. Each is specified concretely with type
signatures.

### The Signature primitive (consumed by Domains 4, 5, 7)

Specified above under § "Signature substrate". Consumers:

- **Domain 4** — ECO approval signatures (when
  `Project.compliance.eco_workflow_required` is set);
  21 CFR Part 11 § 11.50 manifestation (when
  `Project.compliance.audit_overlay: SignatureRequired*`);
  ITAR / EAR-marked record sign-off.
- **Domain 5** — IPC-1752A / IPC-1755 / CMRT-export signatures;
  signed REACH SVHC declarations; signed CMRT 3TG attestations.
  Per Domain 5's recommendation ("signed materials declarations
  require electronic-signature substrate"), the export adapter
  embeds the `Signature` into the produced XML / PDF artifact.
- **Domain 7** — ECO state-transition signatures (Submit / Review /
  Approve / Implement / Close); AS9102 First Article Inspection
  evidence-package signatures; `LibraryAuditEntry.signature` for
  library-object approvals; AVL-policy-update signatures.

### The ActorIdentity model (consumed by Domains 7 and all authored ops)

Specified above under § "ActorIdentity model". Consumers:

- **Every Operation** — the engine's `Engine::execute()` wraps the
  Operation in a Transaction whose `actor: ActorIdentity` field is
  populated from the daemon's authenticated session.
- **Domain 7** — `LibraryAuditEntry.actor` field; `EngineeringChangeOrder.author`,
  `.reviewers[].actor`, `.approvers[].actor`; `PoolLock.acquired_by`.
- **Domain 4** — 21 CFR Part 11 § 11.10(j) accountability; ITAR/EAR
  per-record actor identity for export-control evidence.

### The audit-log export contract (consumed by Domains 4, 5, 6, 7)

Specified above under § "Audit-log export contract". Consumers:

- **Domain 4** — ITAR / EAR posture audit; 21 CFR Part 11 §§ 11.10(e)
  + 11.50 + 11.70 evidence; ISO 13485 § 4.2.5 records control.
- **Domain 5** — substance-list-version-pin update history;
  per-Part `compliance` field change history; CMRT export evidence.
- **Domain 6** — length-match-group authoring history; PHY-profile
  assignment history; rule-pack version pinning history; EMC
  waiver history.
- **Domain 7** — ECO state-transition history; library-object-
  status-change history; supply-chain-refresh history with
  `data_egress_policy_decision`.

### The ProvenanceTag enum (consumed by Domain 6 and AI surfaces)

Specified above under § "The AI-Output Guard-Rail". Consumers:

- **Domain 6** — AI-explanation outputs for length-match tolerance,
  controlled-impedance violation explanation, PHY-profile selection
  rationale; all marked
  `ProvenanceTag::AiSuggestion { model, prompt_hash, at }`.
- **All MCP tools** — every tool that returns AI-generated text
  embeds the `ProvenanceTag` in the response; clients must respect
  the tag.
- **All authored Operations** — when an AI suggestion is accepted
  by the user, the resulting Operation carries
  `provenance: ProvenanceTag::UserValidatedAi { ai_origin, ... }`
  on its OpDiff; the audit-log entry preserves the AI-origin
  pointer.

### The data_egress_policy audit (consumed by Domains 4, 7)

Specified above under § "Data-egress audit". Consumers:

- **Domain 4** — ITAR / EAR / EU dual-use posture; the
  `Project.compliance.data_egress_policy` field is the project-
  level policy state; every external-network MCP tool consults it.
- **Domain 7** — Octopart / Nexar / Digi-Key / Mouser / LCSC
  supply-chain refresh; PLM connectors when they ship; AS9102
  FAI evidence-package signing if it involves external resources.

### Summary table of cross-domain consumption

| Contract | Owner | Domain 2 | Domain 4 | Domain 5 | Domain 6 | Domain 7 |
|---|---|---|---|---|---|---|
| `ActorIdentity` | Domain 8 | n/a | yes (Part 11 § 11.10(j)) | n/a | n/a | yes (LibraryAuditEntry, ECO) |
| `Signature` | Domain 8 | n/a | yes (Part 11 § 11.50) | yes (signed declarations) | n/a | yes (ECO, AS9102) |
| `ProvenanceTag` | Domain 8 | n/a | n/a | n/a | yes (AI explanations) | n/a |
| `AuditEntry` JSONL export | Domain 8 | yes (encrypted-extract logging) | yes (ITAR/EAR audit) | yes (substance pin updates) | yes (PHY-profile, rule-pack pinning) | yes (ECO history, refresh history) |
| `DataEgressPolicy` audit | Domain 8 | yes (encrypted-extract gate) | yes (ITAR/EAR posture) | n/a | n/a | yes (refresh_supply_chain) |
| `EcoSignaturePolicy` | Domain 8 | n/a | yes (workflow gates) | n/a | n/a | yes (ECO state machine) |
| `prev_hash` chain | Domain 8 | n/a | yes (Part 11 § 11.10(c) record protection) | n/a | n/a | n/a |
| Deterministic-replay verifier | Domain 8 | n/a | yes (Part 11 § 11.10(a) validation) | n/a | n/a | yes (AS9102 evidence verification) |

This consolidation completes the 8-domain audit. Every cross-domain
deferred-here item from Domains 2 / 4 / 5 / 6 / 7 is satisfied by
one of the contracts above.

## EDA Tool Support Matrix

The matrix is **conspicuously empty** for open-source tools and
modest for commercial tools. This is a real differentiation
opportunity for Datum.

| Tool | Audit-log export | E-sig support | ECO workflow | ActorIdentity | Part-11-style trail | AI-output provenance |
|------|------------------|---------------|--------------|---------------|---------------------|----------------------|
| **Altium Designer + Vault / Altium 365** | yes, via Vault | yes (paid) | yes (paid) | yes (Vault SSO) | substrate; user validates | no |
| **OrCAD Capture + CIS + Allegro** | partial; PLM-mediated | via PLM (Windchill) | via PLM | via SSO | substrate; PLM validates | no |
| **PADS / Xpedition + Teamcenter** | via Teamcenter | via Teamcenter | via Teamcenter | via Teamcenter SSO | substrate; PLM validates | no |
| **Cadence Allegro + Pulse / DWB** | partial; PLM-mediated | via PLM | via PLM | via SSO | substrate | no |
| **KiCad** | git history only | none built-in (signed git tags optional) | none | OS-user only | git-substrate | no |
| **Eagle / Fusion Electronics** | partial via Fusion (cloud) | none built-in | none | Autodesk SSO (cloud only) | none | no |
| **Horizon EDA** | git history only | none | none | OS-user | git-substrate | no |
| **LibrePCB** | git history only | none | none | OS-user | git-substrate | no |
| **DipTrace** | none | none | none | OS-user | none | no |
| **EasyEDA / EasyEDA Pro** | partial (cloud) | none | none | EasyEDA cloud SSO | none | no |
| **Datum (current)** | substrate exists; export not landed | none | none | engine assumes single user | substrate exists; no surface | no |
| **Datum (post-Domain-8)** | yes (JSONL + CSV + PAdES) | yes (PKCS#11 / PKCS#12 / WebAuthn) | yes (ECO state machine) | yes (ActorIdentity + IdP plug-ins) | yes (Part-11 substrate-compatible) | yes (ProvenanceTag enum) |

**Observations.**

1. **The "AI-output provenance" column is universally `no` across
   commercial and open-source incumbents.** Datum's planned
   `ProvenanceTag` enum is a genuine industry-first.
2. **The "ECO workflow" column is universally tied to PLM
   integration** for commercial tools. Datum's planned ECO
   substrate is **first-class engine-level**, not deferred to a
   PLM bolt-on.
3. **The open-source landscape (KiCad, Horizon, LibrePCB, DipTrace,
   EasyEDA-FOSS) has zero audit-trail / signature / ECO support
   beyond git history.** Datum's planned Domain-8 capabilities
   put it ahead of the entire open-source EDA category and at
   parity with commercial-PLM-integrated workflows.

## Pending Exclusions (re-affirmed)

Per the Pending Exclusions Policy, the following Domain-8 advisory
exclusions are re-affirmed for promotion to formal `Out of scope`
in the consolidated post-Domain-8 ratification pass.

### AS9100D / AS9110 / AS9120 (aerospace QMS)

**Why excluded.** Process-grade certification conferred on the
organisation by an IAQG-accredited registrar.

**Substrate Datum provides.**
- Deterministic transaction log → § 8.5.6 Control of changes.
- Per-Part lifecycle + supersede chains (Domain 7) → § 8.1.4
  Counterfeit prevention substrate.
- AS9102 evidence package (Domain 7 + Domain 8 signatures) → AS9102
  First Article Inspection.
- Variant data model → fitted-component reporting.
- Audit-log JSONL/CSV/PAdES export (Domain 8) → all clause-7
  documented-information requirements.

**Recommendation:** promote to formal `Out of scope` with substrate
paragraph in `STANDARDS_COMPLIANCE_SPEC.md` § 4.4.

### IATF 16949 (automotive QMS)

**Why excluded.** Process-grade certification conferred on the
organisation by an IATF-recognised certification body.

**Substrate Datum provides.**
- Same as AS9100 — transaction log, audit-trail export, ECO
  workflow, signature substrate, deterministic replay.
- Per-Part `qualification: PartQualification` (Domain 4) → AEC-Q
  metadata for automotive supplier declaration.
- `Project.compliance.intended_asil` (Domain 4) → ASIL declaration
  for ISO 26262 cross-reference.

**Recommendation:** promote to formal `Out of scope` with substrate
paragraph in `STANDARDS_COMPLIANCE_SPEC.md` § 4.4.

### CMMI (capability maturity model)

**Why excluded.** Organisational process-maturity assessment;
SCAMPI-certified assessors evaluate the organisation's processes,
not its tools.

**Substrate Datum provides.** Datum's substrate is consistent with
CMMI Level-3 practice areas (Configuration Management, Verification,
Validation, Process and Product Quality Assurance) but provides no
algorithmic CMMI capability.

**Recommendation:** promote to formal `Out of scope` in
`STANDARDS_COMPLIANCE_SPEC.md` § 4.8.

### ISO 13485 (medical-device QMS)

**Why excluded as process-grade.** ISO 13485 certification is
conferred on the organisation by an accredited registrar. Datum can
be substrate but cannot certify.

**Substrate Datum provides.**
- Same as AS9100 / IATF 16949 — transaction log, audit-trail export,
  ECO workflow, signature substrate, deterministic replay.
- 21 CFR Part 11 substrate-compatible (this report) → ISO 13485
  § 4.2.5 control of records.
- Project-level medical-vertical metadata (Domain 4) → ISO 13485
  scope declaration.

**Recommendation:** promote to formal `Reference-only` (already
classified there in `STANDARDS_COMPLIANCE_SPEC.md` § 4.4); confirm
in the ratification pass.

### Additional exclusions surfaced by this deep-dive

The deep-dive surfaces no new exclusions beyond the four already
on the advisory list. Two items considered and **kept** in scope:

- **eIDAS QES** — initial framing was "deferred indefinitely" but
  the deep-dive shows it is achievable as substrate with bounded
  Datum-side work + organisational QSCD + QTSP arrangement.
  Recommend `Planned` (substrate).
- **WebAuthn / FIDO2** — emerging but real; recommend `Optional,
  deferred` (substrate ready when a customer demands it).

## User Pain Points & Wishlist Items

Distilled from forums, industry discussion, and competitor
documentation:

1. **"My QMS auditor wants the design history but Altium Vault is
   too expensive for our team size."** A common refrain in EEVblog,
   r/PrintedCircuitBoard, and KiCad forum threads from medical-
   device startups, automotive Tier-3 suppliers, and aerospace
   contract designers. The sub-$50k/year tier of compliance-grade
   audit support is wholly unserved. **Datum opportunity:** the
   audit-log + signature substrate is engine-level, not gated on a
   paid Vault subscription. Open-source (or eventually open-core)
   pricing fits the underserved market.

2. **"I have to maintain a separate spreadsheet of which version
   of which schematic each ECO references because git tags don't
   carry the change rationale."** Common in automotive Tier-2 and
   medical-device shops using KiCad. The ECO-as-bundle-of-
   transactions-plus-rationale shape (Domain 7 + Domain 8) is
   exactly the missing mechanism.

3. **"Our compliance officer wants a PDF of every change with
   approving-engineer signature; we generate them by hand from git
   diffs."** A real workflow. **Datum opportunity:** PAdES PDF
   export from the audit log is one CLI command.

4. **"Our IT department blocks Altium 365 cloud sync because of
   data-residency rules; we can't use modern Vault features."**
   Defence, aerospace, financial-services-adjacent customers. The
   `data_egress_policy` field plus engine-level audit support
   (no cloud dependency) is the answer.

5. **"AI suggested a part substitute and a junior engineer approved
   it without checking the AEC-Q grade; the part shipped, the
   project failed automotive qualification."** Real-world failure
   mode reported in industry forums. The `ProvenanceTag::UserValidatedAi`
   record + the EcoSignaturePolicy role-based approval rules
   (preventing junior engineers from approving compliance-relevant
   substitutions) directly address this.

6. **"We need to prove our designs were not modified after FDA
   submission but our git history doesn't include cryptographic
   signatures by default."** The `Signature` primitive +
   external-anchor support (rekor / RFC 3161) addresses this.

7. **"Octopart told us a part was active but our supplier audit
   later showed it was actually EOL three months earlier; we have
   no way to prove what state Octopart returned at the time we
   queried it."** The `ExternalCallGated` audit event with
   `response_summary` + the deterministic-replay-friendly
   `refresh_supply_chain` Transaction (Domain 7) addresses this:
   the auditor can reconstruct the state of the supply-chain data
   at the moment of query, with cryptographic evidence.

8. **"Our reviewer commented on a design last month but I can't
   find the comment chain because it's in our chat tool."** The
   ECO comment / rationale field (Domain 7) with audit-trail
   capture preserves the discussion history alongside the design
   change.

These pain points are universally underserved by the
EDA-tool-as-product category; the workaround is bolt-on PLM. The
Datum opportunity is to make engine-level what others sell as
add-ons.

## Datum EDA Implementation Strategy

### Hard Requirements (must support)

The HR set is the **substrate** that all five upstream domains
depend on. Without these, Domains 2 / 4 / 5 / 6 / 7's deferred-
here items remain blocked.

#### HR-1: `ActorIdentity` type and identity-provider abstraction

**Engine spec.** New types in `ENGINE_SPEC.md` § 1.1a:
`ActorIdentity`, `IdpKind`. Implementation in
`crates/engine/src/identity/`.

**Daemon spec.** New module `crates/engine-daemon/src/identity/`
hosting OAuth 2.1 / OIDC client + Local-OS-user-provider + (later)
SAML SP / WebAuthn / PKCS#11.

**MCP API surface.** New tools (`MCP_API_SPEC.md` new "Identity"
section):
- `set_session_actor_identity(actor: ActorIdentity)` — daemon-
  authenticated; engine consumes.
- `get_session_actor_identity()` — returns current session actor.
- `list_known_actors()` — for project-history filtering.

**Effort.** ~5 days for the engine type and Local provider; ~5 days
for the OIDC provider; ~10 days for SAML SP (deferred); ~5 days
for WebAuthn (deferred).

#### HR-2: `Transaction` extension with actor + timestamp + prev_hash

**Engine spec.** Extend `Transaction` (`ENGINE_SPEC.md` § 3):

```rust
pub struct Transaction {
    pub id: Uuid,
    pub operations: Vec<(Box<dyn Operation>, OpDiff)>,
    pub description: String,
    /// NEW: who initiated this transaction
    pub actor: ActorIdentity,
    /// NEW: wall-clock time at commit
    pub timestamp: DateTime<Utc>,
    /// NEW: SHA-256 of the immediately preceding Transaction's
    /// canonical-JSON form; None for the first Transaction.
    pub prev_hash: Option<[u8; 32]>,
    /// NEW: structured rationale beyond description
    pub rationale: Option<String>,
    /// NEW: provenance of this transaction
    pub provenance: ProvenanceTag,
    /// NEW: optional signatures attached at commit time
    pub signatures: Vec<Signature>,
}
```

**Native format.** Extend `<project>/audit_log/<txn-uuid>.json` to
include the new fields per the canonical-JSON serialisation
contract.

**Effort.** ~3 days for the type + serialisation + tests.

#### HR-3: `Signature` primitive + signing providers

**Engine spec.** New types in `ENGINE_SPEC.md` § 1.1a:
`Signature`, `SignatureAlgorithm`, `SignatureMeaning`,
`AssuranceLevel`, `ExternalTimestampAnchor`. New trait
`SigningProvider`.

**Implementation.** `crates/engine/src/signature/` with three
provider implementations: `LocalSoftwareProvider` (`ring` crate),
`Pkcs11Provider` (`cryptoki` crate, Apache-2.0), `ExternalProcessProvider`
(subprocess shim).

**MCP API surface.** New tools (`MCP_API_SPEC.md` new "Signatures"
section):
- `sign_transaction(txn_uuid, meaning, provider_config) -> Signature`
- `sign_eco(eco_uuid, meaning, provider_config) -> Signature`
- `verify_signature(signature) -> VerificationResult`
- `list_signature_providers() -> Vec<ProviderInfo>`
- `request_pkcs11_pin(token_label) -> SessionHandle` (interactive)

**Effort.** ~5 days for Local; ~7 days for PKCS#11; ~3 days for
external-process; ~5 days for the verifier and provider abstraction.
Total: ~20 days.

#### HR-4: `ProvenanceTag` enum + AI-output guard-rail

**Engine spec.** New types in `ENGINE_SPEC.md` § 1.1a:
`ProvenanceTag`, `AiSuggestionRef`. Field added to `OpDiff`,
`Transaction`, and any AI-emitting MCP tool's response shape.

**MCP API surface.** Extension across all MCP tools — every tool's
response schema gains `provenance: ProvenanceTag`. AI agents read
the tag; humans see it in the tool output.

**Engine behaviour.** AI-suggested data is never written to canonical
state directly; it lives in `pending_ai_suggestions` until promoted
via `accept_ai_suggestion(suggestion_uuid)` which executes a real
Operation with `provenance: UserValidatedAi { ai_origin, ... }`.

**MCP API surface.** New tools (`MCP_API_SPEC.md` new "AI
Provenance" section):
- `record_ai_suggestion(prompt_hash, model, suggestion) -> SuggestionUuid`
- `list_pending_ai_suggestions(project_uuid) -> Vec<Suggestion>`
- `accept_ai_suggestion(suggestion_uuid, rationale) -> TransactionId`
- `reject_ai_suggestion(suggestion_uuid, rationale)`
- `query_ai_provenance_chain(object_uuid) -> ProvenanceChain`

**Effort.** ~5 days for the type + serialisation; ~3 days for the
suggestion-promotion mechanism; ~5 days for cross-MCP-tool
integration. Total: ~13 days.

#### HR-5: Audit-log persistence + JSON-Lines export

**Native format.** New directory contract
`<project>/audit_log/<txn-uuid>.json` (one file per Transaction);
optional `<project>/audit_log/index.jsonl` daily roll-up.

**Engine.** `crates/engine/src/audit_log/` for write side;
read-side query API.

**MCP API surface.** New tools (`MCP_API_SPEC.md` new "Audit Log"
section):
- `query_audit_log(filter: AuditQueryFilter) -> Vec<AuditEntry>`
- `export_audit_log_jsonl(filter, output_path) -> ExportSummary`
- `export_audit_log_csv(filter, output_path) -> ExportSummary`
- `export_audit_log_pades(filter, signing_config, output_path) -> ExportSummary`
- `verify_audit_log_chain(project_uuid) -> ChainVerificationResult`
- `replay_audit_log(project_uuid, target_txn_uuid?) -> ReplayResult`

**Effort.** ~5 days for persistence; ~5 days for query + filter
language; ~3 days for JSONL export; ~5 days for CSV adapter; ~10
days for PAdES adapter (with PDF + embedded signature); ~5 days for
chain verifier; ~7 days for replay verifier. Total: ~40 days.

#### HR-6: ECO state machine + EcoSignaturePolicy

**Engine spec.** Domain 7 introduced `EngineeringChangeOrder`,
`EcoStatus`, `EcoTransition`. Domain 8 adds:

```rust
pub struct EcoSignaturePolicy { ... }  // (above)
pub struct RequiredSignature { ... }   // (above)
```

Add `Project.compliance.eco_signature_policy: EcoSignaturePolicy` to
the `ProjectCompliance` block (Domain 4).

**Engine behaviour.** Each ECO state-transition operation
(`SubmitEcoForReview`, `ApproveEco`, `MarkEcoImplemented`, etc.)
checks the project's `eco_signature_policy` and refuses if the
required signatures are missing.

**MCP API surface.** Existing Domain-7 tools (`open_eco`, etc.)
gain optional `signatures: Vec<Signature>` arguments. New tool:
- `validate_eco_against_policy(eco_uuid) -> PolicyValidationResult`

**Effort.** ~5 days for the policy type + state-machine integration;
~3 days for the validator; ~3 days for documentation templates.
Total: ~11 days.

### Should Support (post-M7)

#### SS-1: Lint-finding-as-audit-event integration

**Status.** Domain 5 + Domain 6 already specified the validators;
Domain 8 specifies the audit-log integration shape.

**Effort.** ~3 days to wire validators into the audit-log emission
path; ~2 days for the `WaiverGranted` event shape and the waiver-
signing integration. Total: ~5 days.

#### SS-2: PAdES / XAdES / CAdES adapters for compliance-export bundles

**Status.** Adapters depend on HR-3 (Signature primitive) and
HR-5 (audit-log persistence).

**Effort.** ~10 days for PAdES (covered under HR-5); ~3 days for
XAdES (extends Domain 5's IPC-1752A export with signature embedding);
~3 days for CAdES (covers project-state-snapshot signing). Total:
~16 days; PAdES is in the HR-5 budget already, the other 6 days
are new.

#### SS-3: External timestamp anchoring (RFC 3161 + sigstore rekor)

**Status.** Optional augmentation of HR-3 signatures.

**Effort.** ~3 days for RFC 3161 TSA client (`tsp` crate, Apache-2.0);
~3 days for rekor client (`sigstore` crate, Apache-2.0). Total:
~6 days.

#### SS-4: Git-integrated audit-log mode

**Status.** Optional adapter that materialises the audit log as git
commits.

**Effort.** ~5 days for the `datum-eda audit git-commit` CLI
subcommand and the canonical git-trailer format; ~3 days for
documentation.

#### SS-5: WebAuthn / FIDO2 second-factor + signing pathway

**Status.** Emerging; ship when a customer requests.

**Effort.** ~10 days for the `webauthn-rs` integration + daemon-
side flow; ~5 days for the signing-pathway integration.

### On-Demand Only

#### OD-1: SAML 2.0 SP integration

**Trigger.** When a customer with a SAML-only enterprise IdP
commissions integration. Effort: ~15 days for `samael` integration +
testing.

#### OD-2: in-toto attestation export

**Trigger.** When a customer with SLSA-tracked software-supply-chain
infrastructure requests it. Effort: ~5 days.

#### OD-3: Vendor PLM connectors (Windchill / Teamcenter / Aras / Arena)

**Trigger.** Per-customer paid integration. Already classified by
Domain 7 as `Out of scope` for pre-emptive work. Effort: ~30-60
days per connector.

### Out of Scope (recommend formal exclusion)

- **AS9100D / AS9110 / AS9120** — promote to formal `Out of scope`
  with substrate paragraph.
- **IATF 16949** — promote to formal `Out of scope` with substrate
  paragraph.
- **CMMI** — promote to formal `Out of scope`.
- **ISO/IEC 33000 series / ASPICE** — promote to formal `Out of
  scope`.
- **Workflow-engine integration (JIRA / ServiceNow / Salesforce
  workflow tools)** — Datum exposes data hooks; per-tool connectors
  are user-side / consultant-side work.

### Minimum-Viable vs Full Implementation

**Minimum-viable Domain-8 batch (~30 days engineering)**:
- HR-1 (ActorIdentity + Local provider only, OIDC deferred)
- HR-2 (Transaction extension)
- HR-3 (Signature primitive + Local provider only, PKCS#11 deferred)
- HR-4 (ProvenanceTag — minimum: enum + serialisation)
- HR-5 (audit-log persistence + JSONL export only, CSV/PAdES deferred)

This delivers the substrate that satisfies most of the
`STANDARDS_COMPLIANCE_SPEC.md` § 4.8 deferred items, even without
the IdP federation and the regulated-grade signature providers. The
deferred pieces add ~50 more engineering days as customer demand
materialises.

**Full Domain-8 batch (~80 days engineering)**:
- All HR + SS items above.
- OIDC, PKCS#11, PAdES, CSV, RFC 3161, git-integrated mode all
  delivered.
- Regulated workflows fully supported.

### Partner / library dependencies (license-clean)

| Dependency | Purpose | License | Risk |
|------------|---------|---------|------|
| `ring` | Crypto primitives (Ed25519, ECDSA, SHA-2) | ISC + BSD | clean |
| `ed25519-dalek` | Ed25519 signing | BSD-3-Clause | clean |
| `rsa` | RSA-PSS signing | MIT/Apache-2.0 | clean |
| `cryptoki` | PKCS#11 binding | MIT/Apache-2.0 | clean |
| `oauth2` | OAuth 2.1 client | MIT/Apache-2.0 | clean |
| `openidconnect` | OIDC client | MIT/Apache-2.0 | clean |
| `samael` | SAML 2.0 SP | MIT/Apache-2.0 | clean |
| `webauthn-rs` | WebAuthn server | MPL-2.0 | file-boundary OK; verify before merge |
| `x509-parser` | X.509 parsing | MIT | clean |
| `webpki` | X.509 chain validation | ISC | clean |
| `pdf-writer` | PDF emission | Apache-2.0 | clean |
| `tsp` | RFC 3161 TSA client | unverified | check before adoption |
| `sigstore` | Rekor client | Apache-2.0 | clean |
| `git2` (libgit2 bindings) | Git integration | GPL-2 with linking exception | acceptable; subprocess preferred |

The git2 binding's GPL-2-with-linking-exception is the only
non-permissive license in the dependency stack. The safe pattern is
subprocess invocation of the `git` binary (license-clean) rather
than linking libgit2 directly. The subprocess pattern is documented
in Domain 7's PLM-substrate framing.

### Effort estimate summary

| Item | Engineering days |
|------|------------------|
| HR-1 ActorIdentity + Local provider | 5 |
| HR-1 OIDC provider | 5 |
| HR-2 Transaction extension | 3 |
| HR-3 Signature primitive + Local provider | 10 |
| HR-3 PKCS#11 provider | 7 |
| HR-3 verifier | 5 |
| HR-4 ProvenanceTag | 13 |
| HR-5 Audit-log persistence + JSONL | 10 |
| HR-5 CSV adapter | 5 |
| HR-5 PAdES adapter | 10 |
| HR-5 Chain verifier | 5 |
| HR-5 Replay verifier | 7 |
| HR-6 ECO state machine + policy | 11 |
| SS-1 Lint-as-audit | 5 |
| SS-2 XAdES + CAdES adapters | 6 |
| SS-3 External timestamp anchoring | 6 |
| SS-4 Git-integrated mode | 8 |
| SS-5 WebAuthn second-factor + signing | 15 |
| **Hard Requirements subtotal** | **96** |
| **Should Support subtotal** | **40** |
| **Total Domain-8 batch** | **136** |

This is a substantial batch — comparable in size to Domain 5's
material-environmental implementation (Domain 5 estimated ~30 days
across substance metadata + posture + IPC-1752A + CMRT). The
difference is that Domain 8 is **the consolidation** of five
upstream domains' deferred items, so the per-domain marginal cost
is lower than it appears: every other domain's deferred-here items
become "ready to ship" once Domain 8 lands.

Realistic sequencing: Pass 0 + HR-1 + HR-2 + HR-3 (Local) + HR-4 +
HR-5 (JSONL) as the minimum-viable batch (~50 days); the rest as
follow-on batches as customer demand materialises.

### Datum Differentiators

The big positioning win for Domain 8 — Datum can be the **only**
EDA tool with **ALL** of:

1. **Deterministic-replay audit** (no incumbent has this; even
   Altium Vault's audit log is event-based, not replay-verifiable).
2. **AI-output provenance** in the data model (`ProvenanceTag`
   enum; no incumbent has this).
3. **MCP-queryable compliance evidence** (every audit-trail event
   queryable via one MCP tool with structured filters; no
   incumbent has comparable AI-agent-friendly audit access).
4. **Substrate ready for 21 CFR Part 11 / ISO 9001 / AS9100 /
   ISO 13485 audits without bolting on a second PLM** (Altium needs
   Vault, Cadence needs Pulse + Windchill, Mentor needs Teamcenter;
   Datum's substrate is engine-level).
5. **eIDAS QES capable as substrate** with PKCS#11 + QSCD + QTSP
   arrangement (no open-source EDA tool offers this; Datum becomes
   the first).
6. **Git-native** by file format design, optionally git-integrated
   for audit-log materialisation (most commercial tools fight git;
   Datum embraces it).
7. **Data-egress-aware AI integration** with policy-gated MCP tools
   and audit-logged egress decisions (no incumbent surfaces this
   evidence).

This combination is **unique in the EDA-tool category**. The
Phase-1 audit's signature insight ("Datum's transaction model is
consistently better-positioned for compliance work than any other
surface") was correct, and Domain 8 specifies the surfaces that
make that latent advantage visible.

Quantification: the closest competitor (Altium Designer + Vault +
Altium 365 Workspace) costs $10,000-$25,000/year per seat for the
audit / signature / Vault feature set, and still requires a separate
PLM bolt-on for full ECO workflow. Datum delivers the substrate at
engine level with no per-seat licence cost (open source) and no
PLM dependency. For any organisation in the underserved sub-50-seat
regulated-EDA market, the value differential is substantial.

### Recommended Spec Edits

The Domain 8 batch surfaces 28 recommended spec edits across six
files. **NOTE: Claude is in research-only mode per `feedback_research_only_mode`
memory rule; these are recommendations only, NOT to be applied by
the agent.** The user reviews, prioritises, and applies via the
standard spec-edit process.

| # | Source | Target file | Substance |
|---|--------|-------------|-----------|
| **Pass 0 — disposition refresh** | | | |
| D8-0a | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.8 | Refresh Domain-8 dispositions: ISO 9001 / ISO 13485 / 21 CFR Part 11 substrate-claim move from `Deferred with prerequisite` to `Planned (substrate)` once Pass 1 + Pass 2 + Pass 3 land; AS9100 / IATF 16949 / CMMI / ISO/IEC 33000 / ASPICE promote to `Out of scope` with substrate paragraphs; eIDAS / PAdES / XAdES / CAdES / PKCS#11 / X.509 add as `Planned (substrate)`; OAuth 2.1 / OIDC add as `Planned`; SAML 2.0 / WebAuthn / in-toto add as `Optional, deferred` |
| D8-0b | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 8 (Audit Trail And Review Contracts) | Promote target-state list to concrete contract: enumerate the `Transaction` extension fields (actor, timestamp, prev_hash, rationale, provenance, signatures), the audit-log JSONL persistence layout, the JSONL/CSV/PAdES export adapters, the deterministic-replay verifier; cross-reference `ENGINE_SPEC.md` § 3 and § 1.1a additions |
| D8-0c | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` new § 8.x "AI-Output Guard-Rail" | New subsection formalising the `ProvenanceTag` enum, the AI-suggestion-vs-canonical-state distinction, the user-validation promotion path, the AI-cannot-grant-approvals invariant, and the MCP wording rules |
| D8-0d | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` § 7 (Project-Level Compliance Metadata) | Confirm the `audit_overlay` field placement and add `eco_signature_policy: EcoSignaturePolicy` as a sub-field of `ProjectCompliance` |
| **Pass 1 — `specs/ENGINE_SPEC.md` schema bedrock** | | | |
| D8-1 | this report | `specs/ENGINE_SPEC.md` § 1.1a | New shared types: `ActorIdentity`, `IdpKind`, `Signature`, `SignatureAlgorithm`, `SignatureMeaning`, `AssuranceLevel`, `ExternalTimestampAnchor`, `ProvenanceTag`, `AiSuggestionRef` |
| D8-2 | this report | `specs/ENGINE_SPEC.md` § 3 (Transaction extension) | Extend `Transaction` with `actor: ActorIdentity`, `timestamp: DateTime<Utc>`, `prev_hash: Option<[u8; 32]>`, `rationale: Option<String>`, `provenance: ProvenanceTag`, `signatures: Vec<Signature>` fields; document the canonical-JSON serialisation order to preserve determinism |
| D8-3 | this report | `specs/ENGINE_SPEC.md` § 3 (OpDiff extension) | Extend `OpDiff` with `provenance: ProvenanceTag` field |
| D8-4 | this report | `specs/ENGINE_SPEC.md` § 1.x (ProjectCompliance — Domain 4) | Extend `ProjectCompliance` (Domain 4 introduces) with `eco_signature_policy: EcoSignaturePolicy`, `audit_overlay: AuditOverlayMode`, `tamper_evidence_required: bool`. New types: `EcoSignaturePolicy`, `RequiredSignature`, `EcoTransition`, `AuditOverlayMode` |
| D8-5 | this report | `specs/ENGINE_SPEC.md` § 3 (Operations) | New operations: `RecordAiSuggestion`, `AcceptAiSuggestion`, `RejectAiSuggestion`, `SignTransaction`, `SignEco`, `RotateActorIdentity`, `SetEcoSignaturePolicy`, `SetAuditOverlay` |
| D8-6 | this report | `specs/ENGINE_SPEC.md` § 5.x (Engine API extension) | Engine API gains `current_actor() -> ActorIdentity`, `set_session_actor(actor)`, `query_audit_log(filter)`, `verify_audit_chain()`, `replay_audit_log(target?)` methods |
| **Pass 2 — `specs/NATIVE_FORMAT_SPEC.md` persistence** | | | |
| D8-7 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6 (new subsection) | `<project>/audit_log/<txn-uuid>.json` per-Transaction file contract; canonical-JSON form; `prev_hash` linkage |
| D8-8 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6 (new subsection) | `<project>/audit_log/index.jsonl` daily-roll-up index file contract |
| D8-9 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6 (new subsection) | `<project>/signatures/<txn-uuid>/<sig-uuid>.signature.json` per-Signature file contract |
| D8-10 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6 (new subsection) | `<project>/ai_suggestions/<sug-uuid>.json` pending-AI-suggestion file contract |
| D8-11 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6 (new subsection) | `<project>/ai_explanations/<event-id>.txt` AI-explanation sidecar contract |
| D8-12 | this report | `specs/NATIVE_FORMAT_SPEC.md` § 6.1 (project.json) | `project.json` `compliance` block gains `eco_signature_policy`, `audit_overlay`, `tamper_evidence_required`, `data_egress_policy` (the data_egress_policy field is Domain 4's Batch 3; cross-reference) |
| **Pass 3 — `specs/MCP_API_SPEC.md` (Identity / Signatures / Audit Log / AI Provenance / ECO Workflow Sections)** | | | |
| D8-13 | this report | `specs/MCP_API_SPEC.md` new "Identity Tools" section | Tool stubs: `set_session_actor_identity`, `get_session_actor_identity`, `list_known_actors`, `rotate_actor_identity` |
| D8-14 | this report | `specs/MCP_API_SPEC.md` new "Signature Tools" section | Tool stubs: `sign_transaction`, `sign_eco`, `sign_audit_log_bundle`, `verify_signature`, `list_signature_providers`, `request_pkcs11_pin` |
| D8-15 | this report | `specs/MCP_API_SPEC.md` new "Audit Log Tools" section | Tool stubs: `query_audit_log`, `export_audit_log_jsonl`, `export_audit_log_csv`, `export_audit_log_pades`, `verify_audit_log_chain`, `replay_audit_log`, `query_audit_event_types` |
| D8-16 | this report | `specs/MCP_API_SPEC.md` new "AI Provenance Tools" section | Tool stubs: `record_ai_suggestion`, `list_pending_ai_suggestions`, `accept_ai_suggestion`, `reject_ai_suggestion`, `query_ai_provenance_chain`, `query_ai_explanation_for_event` |
| D8-17 | this report | `specs/MCP_API_SPEC.md` extension to existing ECO tools (Domain 7) | `open_eco`, `submit_eco_for_review`, `review_eco`, `approve_eco`, `mark_eco_implemented`, `close_eco` extended with optional `signatures: [Signature]` and `actor_override: Option<ActorIdentity>` arguments; `validate_eco_against_policy` new tool |
| D8-18 | this report | `specs/MCP_API_SPEC.md` extension to all MCP tool response shapes | Every MCP tool response gains `provenance: ProvenanceTag` field; AI agents and clients respect the tag |
| D8-19 | this report | `specs/MCP_API_SPEC.md` § Encrypted Content Handling Policy (extend) | The existing policy at line 1653 is extended with the audit-log-event contract: every encrypted-extraction-attempt produces an `AuditEventKind::EncryptedExtractionAttempt { encryption_scheme, requested_handling, decision }` event |
| **Pass 4 — `docs/POOL_ARCHITECTURE.md` and `docs/INTEROP_SCOPE.md`** | | | |
| D8-20 | this report | `docs/POOL_ARCHITECTURE.md` § 5 (extend) | New subsection "Pool Object Lifecycle Audit": each pool object's lifecycle transitions emit `LibraryAuditEntry` (Domain 7 specified the data shape; Domain 8 specifies the audit-log integration) |
| D8-21 | this report | `docs/POOL_ARCHITECTURE.md` new § "Audit Log Integration" | New section documenting how the project's `audit_log/` directory layout interacts with pool changes (pool-object edits via authoring ops produce Transactions just like board / schematic edits) |
| D8-22 | this report | `docs/INTEROP_SCOPE.md` | Add explicit rows: **eIDAS QES**: Planned substrate via PKCS#11 + QSCD + QTSP; **PAdES / XAdES / CAdES**: Planned substrate; **OAuth 2.1 / OIDC**: Planned (substrate); **SAML 2.0 SP**: Optional, deferred (per-customer); **WebAuthn / FIDO2**: Optional, deferred; **PKCS#11 / PKCS#12 / X.509**: Planned; **rekor / sigstore**: Optional, deferred (external timestamp anchor); **RFC 3161 TSA**: Optional, deferred (external timestamp anchor); **in-toto attestation**: Optional, deferred |
| D8-23 | this report | `docs/INTEROP_SCOPE.md` | Add explicit rows: **AS9100 / IATF 16949 / CMMI / ISO/IEC 33000 / ASPICE**: Out of scope (process-grade certification); substrate paragraphs cross-referenced from STANDARDS_COMPLIANCE_SPEC.md § 4.8 |
| **Pass 5 — `docs/STANDARDS_AUDIT_BATCH_X_GUIDANCE.md` (NEW Batch-7 bridging doc)** | | | |
| D8-24 | this report | `docs/STANDARDS_AUDIT_BATCH_7_GUIDANCE.md` (NEW) | Batch-7 bridging doc following the Batch-1 / 2 / 3 / 4 / 5 / 6 pattern (must-land vs deferred, apply order, dependence on Domain 4's Batch 3 ProjectCompliance and Domain 7's Batch 6 ECO substrate landing first, Pass 0 disposition refresh, Cross-Spec Update Rule compliance, advisory-exclusion ratification framing). Recommended split: Batch 7.0 = Pass 0 + Pass 1 + Pass 2 substrate; Batch 7.1 = Pass 3 MCP surface + minimum-viable signing providers (Local); Batch 7.2 = full signing providers (PKCS#11, PAdES, XAdES, CAdES) + replay verifier; Batch 7.3 = OIDC / SAML / WebAuthn IdP federation + external timestamp anchors |
| D8-25 | this report | `docs/STANDARDS_COMPLIANCE_INTEGRATION_GUIDANCE.md` (extend) | Add Domain 8 row to the integration-guidance table: the audit-trail-as-deterministic-replay claim is the central differentiator; the Signature primitive consumes from Domains 4 / 5 / 7; the ProvenanceTag enum consumes from Domain 6 + AI surfaces; the data-egress audit consumes from Domains 4 + 7 |
| **Pass 6 — Final consolidated ratification synthesis (cross-cutting; the LAST of the eight)** | | | |
| D8-26 | this report | `specs/STANDARDS_COMPLIANCE_SPEC.md` final consolidated ratification | Promote all eight advisory-exclusion lists (Domains 1-8) from "Recommended low-priority / skip" advisory status to formal `Out of scope` in the relevant § 4.x dispositions; add the substrate-positioning paragraphs ratified by each domain's deep-dive; reference the eight Phase-2 deep-dives as supporting material |
| D8-27 | this report | `specs/PROGRESS.md` | Add a "Standards Audit (8-domain Phase 2 deep-dive)" milestone status row marking all eight Phase-2 deep-dives delivered; M-x.y substrate-batch sequencing for Domain 8's HR / SS items |
| D8-28 | this report | `docs/RESEARCH_TRACEABILITY.md` | Add the Domain 8 deep-dive to the traceability ledger with its derived-guidance and spec-edit pointers; mark the 8-domain audit complete; close the loop on the Phase-2 cycle |

**Total recommended spec edits: 28** (across six spec/doc files
plus one new bridging-guidance doc and two consolidation entries).

**Comparable to Domain 7's 12-edit count and Domain 6's 15-edit
count.** Domain 8's count is higher because the deep-dive owns the
contracts for five upstream domains in addition to its own scope —
each upstream domain's deferred-here items become a Domain 8 spec
edit.

**Dependency note.** Pass 0 + Pass 1 + Pass 2 are the substrate
landing (the minimum-viable batch). Pass 3 + Pass 4 + Pass 5 deliver
the full surface. Pass 6 (final consolidated ratification) **must
land last across the entire 8-domain audit** — it is the formal
conclusion of the Phase-2 cycle and depends on every prior domain's
ratification work being applied.

The user can split into multiple PRs if the batch is judged too
large for a single review pass; the suggested split is in the
Batch-7 guidance doc above.

## Closing the Standards Audit

This is the final Phase-2 deep-dive. With Domain 8 delivered:

- **All 8 domains delivered.** Phase 2 of the Standards & Compliance
  Audit is complete:
  - Domain 1 (Data exchange & interop) — delivered
  - Domain 2 (Component modelling) — delivered
  - Domain 3 (Schematic & drawing conventions) — delivered
  - Domain 4 (Industry-vertical compliance) — delivered
  - Domain 5 (Materials & environmental) — delivered
  - Domain 6 (EMC & signal integrity) — delivered
  - Domain 7 (PLM & lifecycle integration) — delivered
  - Domain 8 (Process & quality) — delivered
- **The integrated substrate is now specified.** The
  `ActorIdentity` / `Signature` / `ProvenanceTag` / audit-log /
  ECO-state-machine / data-egress-audit contracts that all five
  upstream domains (2, 4, 5, 6, 7) deferred to Domain 8 are
  defined here. The five upstream deferred items can move from
  `Deferred with prerequisite` to `Planned` once Domain 8's
  Pass 0 + Pass 1 + Pass 2 land.
- **Next is the consolidated skip-ratification synthesis pass.**
  The Pending Exclusions Policy (ratified 2026-04-17) deferred
  formal exclusion ratification to a single consolidated pass
  after Domain 8 lands. Domain 8's Pass 6 spec edit (D8-26 above)
  is that consolidated pass — promoting all eight domains'
  advisory-exclusion lists to formal `Out of scope` with substrate-
  positioning paragraphs.
- **The 3 deferred Batch-1 edits remain queued.** Standards Audit
  Batch 1 (PR #1, merged 2026-04-18) deferred three follow-up
  edits per the project owner's integration cadence; they are not
  Domain-8-specific and remain in the user's edit queue
  independent of this report.
- **The cross-domain insights have all landed in Domain 8 as
  concrete contracts.** Every deferred-here item from Domains
  2 / 4 / 5 / 6 / 7 is satisfied by a specific Domain 8 contract
  surface (see § "Cross-Domain Consolidation"). The integrative
  payoff of having Domain 8 land last has been delivered.

The Phase-2 cycle ends here. The Phase-3 work — translating these
research deliverables into spec edits and engine implementation —
is the project owner's prerogative to sequence per the standard
research-to-spec flow specified in
`specs/STANDARDS_COMPLIANCE_SPEC.md` § 10.

The Phase-1 audit's signature insight on Domain 8 — quoted at the
top of this report — held up under the deep-dive. The substrate is
there. The export and authentication-context layers are missing but
specified. The transaction model is consistently better-positioned
for compliance work than any other surface, and Domain 8 specifies
the surfaces that make that latent advantage visible to compliance
reviewers, regulators, and AI agents.

## Sources

### Primary regulatory and standards references

- [21 CFR Part 11](https://www.ecfr.gov/current/title-21/chapter-I/subchapter-A/part-11) — *Electronic Records; Electronic Signatures*. US Code of Federal Regulations; **free** at ecfr.gov. The canonical electronic-records-and-signatures specification.
- [FDA Guidance for Industry, Part 11 — Scope and Application (2003)](https://www.fda.gov/regulatory-information/search-fda-guidance-documents/part-11-electronic-records-electronic-signatures-scope-and-application) — **Free** PDF. The risk-based interpretation reference.
- [FDA Draft Guidance — Electronic Systems, Electronic Records, and Electronic Signatures in Clinical Investigations (2024)](https://www.fda.gov/regulatory-information/search-fda-guidance-documents/electronic-systems-electronic-records-and-electronic-signatures-clinical-investigations) — **Free** PDF. Modern application context.
- [EudraLex Volume 4, Annex 11 — Computerised Systems](https://health.ec.europa.eu/medicinal-products/eudralex/eudralex-volume-4_en) — EU GMP equivalent of 21 CFR Part 11. **Free**.
- [Regulation (EU) 910/2014 (eIDAS)](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=uriserv%3AOJ.L_.2014.257.01.0073.01.ENG) — *electronic IDentification, Authentication and trust Services*. **Free** at eur-lex.
- [Regulation (EU) 2024/1183 (eIDAS 2.0)](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=OJ%3AL_202401183) — eIDAS 2.0 amendment. **Free**.
- [ISO 9001:2015](https://www.iso.org/standard/62085.html) — *Quality management systems — Requirements*. Paywalled, ~CHF 138 from ISO Webstore.
- [ISO 13485:2016](https://www.iso.org/standard/59752.html) — *Medical devices — Quality management systems*. Paywalled, ~CHF 158.
- [ISO 10007:2017](https://www.iso.org/standard/70400.html) — *Quality management — Guidelines for configuration management*. Paywalled, ~CHF 138.
- [AS9100D (2016)](https://www.sae.org/standards/content/as9100d/) — *Quality Management Systems — Requirements for Aviation, Space and Defense Organizations*. Paywalled, ~USD 207 from SAE.
- [IATF 16949:2016](https://www.iatfglobaloversight.org/iatf-169492016/about/) — Automotive QMS. Paywalled ~USD 195 from AIAG store.
- [EIA-649C (2019)](https://www.sae.org/standards/content/eia649c/) — *Configuration Management Standard*. Paywalled, ~USD 100 from SAE.
- [MIL-HDBK-61A (2001)](https://everyspec.com/MIL-HDBK/MIL-HDBK-0001-0099/MIL-HDBK-61A_11531/) — US-defence configuration-management handbook. **Free** at everyspec.com.
- [CMMI for Development v3.0 (2023)](https://cmmiinstitute.com/) — Process-maturity model. ISACA-published; member access.
- [ISO/IEC 33001:2015](https://www.iso.org/standard/54175.html) — Process assessment concepts. Paywalled.
- [Automotive SPICE V4.0 (2023)](https://vda-qmc.de/en/automotive-spice/) — VDA-QMC. Free overview at vda-qmc.de.

### ETSI signature and trust standards

- [ETSI EN 319 142-1 V1.2.1 (2024-09)](https://www.etsi.org/deliver/etsi_en/319100_319199/31914201/) — PAdES baseline signatures. **Free** at etsi.org.
- [ETSI EN 319 132-1 V1.2.1 (2024-09)](https://www.etsi.org/deliver/etsi_en/319100_319199/31913201/) — XAdES baseline signatures. **Free**.
- [ETSI EN 319 122-1 V1.2.1 (2024-09)](https://www.etsi.org/deliver/etsi_en/319100_319199/31912201/) — CAdES baseline signatures. **Free**.
- [ETSI TS 119 312 V1.5.1 (2024-09)](https://www.etsi.org/deliver/etsi_ts/119300_119399/119312/) — Cryptographic suites for AES / QES. **Free**.
- [ETSI TS 119 102](https://www.etsi.org/deliver/etsi_ts/119100_119199/119102/) — Procedures for creation and validation of AdES digital signatures. **Free**.
- [EN 419241 series](https://www.cencenelec.eu/) — Common Criteria Protection Profiles for Qualified Signature Creation Devices. Paywalled at CEN.
- [EU Trusted Lists Browser](https://webgate.ec.europa.eu/tl-browser/) — Authoritative list of QTSPs. **Free**.

### IETF and OASIS standards

- [RFC 5280](https://www.rfc-editor.org/rfc/rfc5280) — *Internet X.509 Public Key Infrastructure Certificate and CRL Profile*. **Free**.
- [RFC 5652](https://www.rfc-editor.org/rfc/rfc5652) — *Cryptographic Message Syntax (CMS)*. **Free**.
- [RFC 6749](https://www.rfc-editor.org/rfc/rfc6749) — *OAuth 2.0 Authorization Framework*. **Free**.
- [RFC 7636](https://www.rfc-editor.org/rfc/rfc7636) — *Proof Key for Code Exchange (PKCE)*. **Free**.
- [RFC 8252](https://www.rfc-editor.org/rfc/rfc8252) — *OAuth 2.0 for Native Apps*. **Free**.
- [RFC 8446](https://www.rfc-editor.org/rfc/rfc8446) — *TLS 1.3*. **Free**.
- [RFC 9700 (2024)](https://www.rfc-editor.org/rfc/rfc9700) — *OAuth 2.0 Security Best Current Practice*. **Free**.
- [draft-ietf-oauth-v2-1](https://datatracker.ietf.org/doc/draft-ietf-oauth-v2-1/) — OAuth 2.1 consolidated draft. **Free**.
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html) — **Free**.
- [SAML 2.0 — OASIS Standard](http://docs.oasis-open.org/security/saml/v2.0/) — **Free**.
- [PKCS #11 v3.0 — OASIS](https://docs.oasis-open.org/pkcs11/pkcs11-base/v3.0/) — Cryptographic Token Interface Standard. **Free**.
- [RFC 7292](https://www.rfc-editor.org/rfc/rfc7292) — *PKCS #12: Personal Information Exchange Syntax*. **Free**.
- [RFC 3161](https://www.rfc-editor.org/rfc/rfc3161) — *Internet X.509 Public Key Infrastructure Time-Stamp Protocol (TSP)*. **Free**.
- [WebAuthn Level 3 W3C Recommendation (2024)](https://www.w3.org/TR/webauthn-3/) — **Free**.
- [FIDO2 CTAP 2.2 (2023)](https://fidoalliance.org/specs/fido-v2.2-rd-20230321/fido-client-to-authenticator-protocol-v2.2-rd-20230321.html) — **Free**.

### Audit-log / supply-chain provenance ecosystem

- [Trillian](https://github.com/google/trillian) — Google verifiable-log infrastructure. Apache-2.0.
- [sigstore](https://www.sigstore.dev/) — Code-signing transparency platform. Apache-2.0 across the project.
- [rekor](https://docs.sigstore.dev/logging/overview/) — Sigstore's verifiable transparency log. Apache-2.0.
- [in-toto](https://in-toto.io/) — Software-supply-chain attestation framework. CNCF graduated; Apache-2.0.
- [SLSA v1.0 (2023)](https://slsa.dev/spec/v1.0/) — Supply-chain Levels for Software Artifacts. OpenSSF / Linux Foundation; Apache-2.0.
- [in-toto Attestation Framework v1.0](https://github.com/in-toto/attestation/tree/main/spec/v1) — Predicate / statement format. Apache-2.0.

### Reference implementations and Rust ecosystem

- [ring (Rust)](https://github.com/briansmith/ring) — Crypto primitives. ISC + BSD.
- [ed25519-dalek (Rust)](https://github.com/dalek-cryptography/ed25519-dalek) — BSD-3-Clause.
- [rsa (Rust)](https://github.com/RustCrypto/RSA) — MIT/Apache-2.0.
- [cryptoki (Rust)](https://github.com/parallaxsecond/rust-cryptoki) — PKCS#11 binding. MIT/Apache-2.0.
- [oauth2 (Rust)](https://github.com/ramosbugs/oauth2-rs) — OAuth 2.x client. MIT/Apache-2.0.
- [openidconnect (Rust)](https://github.com/ramosbugs/openidconnect-rs) — OIDC client. MIT/Apache-2.0.
- [samael (Rust)](https://github.com/njaremko/samael) — SAML 2.0 SP. MIT/Apache-2.0.
- [webauthn-rs](https://github.com/kanidm/webauthn-rs) — WebAuthn server. MPL-2.0.
- [x509-parser (Rust)](https://github.com/rusticata/x509-parser) — MIT.
- [webpki (Rust)](https://github.com/briansmith/webpki) — ISC.
- [pdf-writer (Rust)](https://github.com/typst/pdf-writer) — Apache-2.0.
- [sigstore (Rust)](https://github.com/sigstore/sigstore-rs) — Apache-2.0.
- [SoftHSM](https://github.com/opendnssec/SoftHSM) — Software PKCS#11 token for testing. BSD-2.
- [OpenSC](https://github.com/OpenSC/OpenSC) — PKCS#11 token middleware. LGPL-2.1 (subprocess invocation pattern).

### EDA tool documentation (used for support-matrix construction)

- [Altium Designer Documentation — Vault Sign-Off](https://www.altium.com/documentation/altium-designer/vault-sign-off-process) — Vault sign-off workflow reference.
- [Altium 365 Workspace Documentation](https://www.altium.com/documentation/altium-365) — Cloud-vault reference.
- [Cadence Allegro Design Workbench](https://www.cadence.com/en_US/home/tools/pcb-design-and-analysis/allegro-design-authoring.html) — Allegro design-management reference.
- [Mentor / Siemens Xpedition + Teamcenter](https://eda.sw.siemens.com/en-US/pcb/xpedition-enterprise/) — Xpedition Enterprise + Teamcenter integration reference.
- [OrCAD CIS](https://www.cadence.com/en_US/home/tools/pcb-design-and-analysis/orcad-capture-cis.html) — Component Information System reference.
- [KiCad — Git Workflow Documentation](https://www.kicad.org/help/contribute/learn/) — KiCad's recommended git-history pattern as audit-log substrate.
- [Horizon EDA — Documentation](https://horizon-eda.readthedocs.io/) — Horizon EDA reference (no audit-trail / signature features).
- [LibrePCB Documentation](https://librepcb.org/docs/) — LibrePCB reference (no audit-trail / signature features).
- [DipTrace User Guide](https://diptrace.com/books/) — DipTrace reference.
- [EasyEDA Documentation](https://docs.easyeda.com/) — EasyEDA reference.

### Forum / industry discussion

- [r/PrintedCircuitBoard](https://www.reddit.com/r/PrintedCircuitBoard/) — Practitioner pain-points distilled in § "User Pain Points & Wishlist Items".
- [EEVblog forum: PCB compliance / audit](https://www.eevblog.com/forum/) — Community discussion of regulated-EDA workflows.
- [KiCad forum: traceability discussions](https://forum.kicad.info/) — Community-side audit / sign-off discussions.
- [LinkedIn EDA / PCB groups](https://www.linkedin.com/groups/) — Industry discussion of compliance posture.
- [SAE Standards Mailing List](https://www.sae.org/standards/) — Discussion of AS9100 / EIA-649 evolution.

### Cross-references to prior research (Datum-internal)

- `research/standards-audit/STANDARDS_AUDIT.md` § 8 (Per-Domain Audit → 8. Process & quality) — Phase-1 inventory; the foundation for this deep-dive.
- `research/standards-audit/STANDARDS_AUDIT.md` § Cross-Cutting Observations (transaction-model insight) — quoted at top of this report.
- `research/component-modeling/COMPONENT_MODELING_RESEARCH.md` — Domain 2 deep-dive; the encrypted-content gate logging requirement that Domain 8 owns the audit-log entry contract for.
- `research/data-exchange-interop/DATA_EXCHANGE_INTEROP_RESEARCH.md` — Domain 1 deep-dive; the IPC-2581 / IPC-1755 export adapters that Domain 8's XAdES adapter consumes.
- `research/schematic-drawing-conventions/SCHEMATIC_DRAWING_CONVENTIONS_RESEARCH.md` — Domain 3 deep-dive; the title-block reviewer/approver placeholders that Domain 8's signature substrate populates.
- `research/industry-vertical-compliance/INDUSTRY_VERTICAL_COMPLIANCE_RESEARCH.md` — Domain 4 deep-dive; the substrate-vs-certification framing extended to QMS in this report; the `data_egress_policy` field; the `audit_overlay` enum; the 21 CFR Part 11 substrate analysis.
- `research/materials-environmental/MATERIALS_ENVIRONMENTAL_RESEARCH.md` — Domain 5 deep-dive; the substance-list-version pinning as audit event; the signed materials declarations requiring the Domain 8 signature primitive.
- `research/emc-signal-integrity/EMC_SIGNAL_INTEGRITY_RESEARCH.md` — Domain 6 deep-dive; the AI-output guard-rail finding (verbatim); the length-match-group / PHY-profile / rule-pack pinning as authored ops captured in audit-trail; EMC waivers requiring separate-category sign-off.
- `research/plm-lifecycle-integration/PLM_LIFECYCLE_INTEGRATION_RESEARCH.md` — Domain 7 deep-dive; the nine explicit Domain-8 handoffs (ECO grouping signature, AS9102 evidence package signature backing, LibraryAuditEntry as audit-log feed, refresh_supply_chain Transaction with data_egress_policy_decision metadata, ActorIdentity OIDC/SAML, lifecycle event feed, pessimistic vault-style check-out deferred until Domain 8 user identity, tamper-evident hash chain, data_egress_policy audit log).
- `research/ipc-compliance/IPC_COMPLIANCE_RESEARCH.md` — IPC compliance survey; cross-referenced for IPC-1755 / IPC-1782 traceability framework (note: IPC-1782 is the supply-chain traceability cross-reference standard for AS9100 § 8.1.4 counterfeit-prevention).
- `specs/STANDARDS_COMPLIANCE_SPEC.md` § 4.8 — current Domain 8 dispositions (post-Batch-1 baseline).
- `specs/STANDARDS_COMPLIANCE_SPEC.md` § 8 (Audit Trail And Review Contracts) — target-state list this report promotes to concrete contracts.
- `specs/ENGINE_SPEC.md` § 3 (Operations / Transaction model) — Batch-1 contract surface this report extends.
- `specs/MCP_API_SPEC.md` § Encrypted Content Handling Policy — Batch-1 contract surface this report extends with audit-log-event integration.
- `docs/CANONICAL_IR.md` § 4 (Transaction Model) and § 5 (Serialization) — the determinism invariant that the audit-trail-as-deterministic-replay claim depends on.
- `docs/POOL_ARCHITECTURE.md` — pool-object-lifecycle audit feed.

### Project-internal context

- `CLAUDE.md` — project framing, attribution policy (Co-Authored-By, "Generated by", and emoji markers explicitly forbidden in research artifacts).
- `~/.claude/projects/-home-bfadmin-Documents-datum-eda/memory/MEMORY.md` — the project owner's standing rules, including:
  - `feedback_no_pull_requests` — Datum projects use direct-to-main; the pull-request-as-sign-off pattern is incompatible (acknowledged in this report).
  - `feedback_research_only_mode` — Claude is in research-only mode; this report is recommendations only, NOT to be applied by the agent.
  - `feedback_no_architectural_decisions` — product/workflow-impact choices surface to the user; the recommendations in this report explicitly defer to the project owner for prioritisation.

### Acknowledgement

This is the FINAL Phase-2 deep-dive in the 8-domain Standards &
Compliance Audit. The cycle began on 2026-04-17 with the Phase-1
inventory; eight per-domain deep-dives followed; the synthesis is
in this report. The cycle ends here.
