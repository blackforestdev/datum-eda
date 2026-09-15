# Product Revision Engine Standards Matrix

> **REV-C02 research artifact — not a conformance claim.** This matrix records
> the primary-source requirements and guidance that can constrain a future
> Datum standards profile. It does not make every listed control universal,
> certify Datum, or authorize implementation. A requirement becomes applicable
> only through the profile, contract, jurisdiction, and tailoring recorded for a
> particular release.

## Reading contract

Each row has a stable identifier so a future audit profile can bind the exact
authority to a Datum control and its evidence. The classifications are:

- **N — normative:** mandatory within the authority's stated applicability.
- **G — official guidance:** useful primary-source guidance, but not a Datum
  conformance requirement by itself.
- **S — scope only:** the publisher confirms the standard's subject and current
  edition, but its licensed normative clauses were not available for this
  research. No clause-level requirement may be inferred from it.
- **D — Datum policy:** an owner-ratified product constraint, kept distinct from
  external standards.

`Adopt-profile` means the control is a candidate requirement when that profile
is enabled. `Core-candidate` means REV-C03 must test whether the control belongs
in Datum's universal substrate. `Blocked-text` means lawful normative text is
required before a conformance profile can be specified. `Reference` means the
source informs design without becoming an audit requirement.

Requirement text below is deliberately paraphrased. The cited authority and
clause—not this summary—remain controlling.

## Primary-source register

| Key | Authority and verified edition/status | Access and classification | Datum applicability |
|---|---|---|---|
| NASA-D | [NASA NPR 7123.1D, Change 2](https://nodis3.gsfc.nasa.gov/displayDir.cfm?Internal_ID=N_PR_7123_001D_&page_name=Chapter3), effective 2023-07-05 through 2028-07-05 | Official public normative NASA procedural requirements (N) | NASA programs/projects only, after the required tailoring and Technical Authority approval; also a useful public control model |
| NASA-SEH | [NASA Systems Engineering Handbook, Rev. 2, §6.5](https://www.nasa.gov/reference/6-5-configuration-management/) | Official public guidance (G) | Reference implementation guidance; not independently mandatory |
| ECSS-40 | [ECSS-M-ST-40C Rev.1](https://ecss.nl/standard/ecss-m-st-40c-rev-1-configuration-and-information-management/), 2009-03-06; still listed as active by ECSS | Official public normative text (N), subject to the ECSS site license/disclaimer | Space-program/project supplier/customer profiles when invoked and tailored under ECSS-S-ST-00 |
| DOD-61 | [MIL-HDBK-61B](https://quicksearch.dla.mil/WMX/Default.aspx?token=5764667), 2020-04-07 | Official public non-mandatory handbook guidance (G) | DoD-oriented design reference; never represented as a contractual requirement by itself |
| DOD-31000 | [MIL-STD-31000B](https://quicksearch.dla.mil/WMX/Default.aspx?token=5754451), dated 2018-10-31 | Official public normative text (N) | A technical-data-package profile only to the extent invoked and tailored by a contract |
| ISO-10007 | [ISO 10007:2017](https://www.iso.org/standard/70400.html), edition 3 | Official scope; normative text paywalled (S) | `Blocked-text`; no ISO 10007 conformance mapping until a licensed copy is reviewed |
| ISO-9001 | [ISO 9001:2015](https://www.iso.org/standard/62085.html), edition 5 plus Amd.1:2024; edition 6 is under publication for 2026-09 | Official scope (S); [ISO/TC 176 guidance on documented information](https://www.iso.org/files/live/sites/isoorg/files/standards/docs/en/iso_9001_2015_guidance_documented_information.pdf) is public but non-normative (G) | Version-pinned QMS profile only; migration must be deliberate when edition 6 publishes |
| SAE-649 | [SAE EIA-649C](https://saemobilus.sae.org/standards/eia649c-configuration-management-standard), revised 2019-02-07 | Official scope; normative text licensed/paywalled (S) | `Blocked-text`; its five-function structure may organize research, not create requirements |
| SAE-649-2 | [SAE EIA-649-2A](https://saemobilus.sae.org/standards/eia649-2a-configuration-management-acquisition-requirements-aeronautics-space), revised 2024-03-01 | Official scope; normative text licensed/paywalled (S) | `Blocked-text` for aeronautics/space acquisition profiles |
| IAQG-9100 | [IAQG 9100:2016 official resource page](https://iaqg.org/standard/9100-qms-requirements-for-aviation-space-and-defense-organizations/) | Official scope/support material; normative text licensed (S) | `Blocked-text`; no AS9100/9100 audit claim from FAQs or presentations |
| ASME-35 | [ASME Y14.35-2025](https://www.asme.org/codes-standards/find-codes-standards/revision-of-engineering-drawings-and-associated-documents) | Official scope; normative text licensed/paywalled (S) | `Blocked-text` for drawing/product-definition revision details |
| ASME-34 | [ASME Y14.34-2013 (R2024)](https://www.asme.org/codes-standards/find-codes-standards/y14-34-associated-lists) | Official scope; normative text licensed/paywalled (S) | `Blocked-text` for associated-list preparation/revision details |
| ASME-100 | [ASME Y14.100-2017](https://www.asme.org/codes-standards/find-codes-standards/y14-100-engineering-drawing-practices) | Official scope; normative text licensed/paywalled (S) | `Blocked-text` for engineering-drawing practices |

### Edition and source findings

- The prior research perimeter's NASA `NPR 7123.1B Appendix C` citation was
  historical. The active public procedural requirement is NPR 7123.1D Change 2,
  whose controlling CM requirement is §3.2.15. Older Appendix-C activity lists
  may be useful history but cannot be labeled current NASA requirements.
- ISO 9001:2015 remains published as of this research, but ISO identifies a
  sixth edition for expected publication in September 2026. Datum must pin the
  edition in every profile and may not silently reinterpret a stored audit.
- ECSS publishes the complete Rev.1 text and lists it as active. Its requirements
  are not universal product rules: the standard expressly allows project
  tailoring under ECSS-S-ST-00.
- MIL-HDBK-61B is guidance, even where it uses forceful language. Contractual
  requirements must come from the contract and incorporated standards.

## Requirement/disposition matrix

### Planning, applicability, and configuration identification

| ID | Authority / clause / class | Applicability and disposition | Requirement and candidate Datum control | Evidence and verification |
|---|---|---|---|---|
| REV-STD-PLAN-001 | NASA-D §3.2.15.1 / N | NASA profile; Adopt-profile | Implement a tailored, Engineering Technical Authority-approved CM process. Datum control: versioned `StandardsProfile` records scope, tailoring, authority, and effective edition. | Signed profile disposition; verify every enabled rule resolves to one edition/clause and approval. |
| REV-STD-PLAN-002 | ECSS-40 §5.2.1 and Annex A / N | ECSS profile; Adopt-profile | Prepare and maintain a CM plan covering actors, responsibilities, interfaces, control levels, information systems, processes, delivery, archive, and retention policy. | Baseline-pinned CM plan plus approvals; schema and completeness audit. |
| REV-STD-ID-001 | NASA-D §3.2.15.2(a-b) / N | NASA profile; Core-candidate | Identify controlled items and be able to identify each item's configuration at selected times. Datum control: stable CI identity plus immutable revision-addressed state. | CI registry and baseline manifests; resolve sampled current/historical configurations and compare hashes. |
| REV-STD-ID-002 | ECSS-40 §§5.3.1.1-5.3.1.3 / N | ECSS profile; Adopt-profile | Define product/CI selection, identifiers, configuration documentation, and applicable baselines in a controlled product structure. | CI selection rationale, product tree, CIDL/baseline membership; uniqueness and referential-integrity checks. |
| REV-STD-ID-003 | ECSS-40 §5.3.7.2.4(a-c) / N | ECSS profile; Core-candidate | Uniquely identify each document and issue; keep mutable facts such as approval level or issue out of the stable document identifier. | Document and issue records; duplicate-ID and mutable-ID lint; round-trip resolution test. |
| REV-STD-ID-004 | DOD-31000 §§3.1.40-3.1.43, 5.14.1 / N | Contract-selected TDP profile; Adopt-profile | Represent an authoritative item description as an indexed package whose members carry nomenclature, document identity, revision, date, and responsible design activity. | TDP/TDPL manifest; required-metadata validation and exact-member reproduction. |

### Baselines, release, and immutable integrity

| ID | Authority / clause / class | Applicability and disposition | Requirement and candidate Datum control | Evidence and verification |
|---|---|---|---|---|
| REV-STD-BL-001 | NASA-D §3.2.15.2(b-d) / N | NASA profile; Core-candidate | Identify configurations over time and maintain their integrity, traceability, and security while systematically controlling changes. | Immutable baseline manifest, provenance chain, authorization records; mutation-refusal and historical-reproduction tests. |
| REV-STD-BL-002 | ECSS-40 §§5.3.1.4-5.3.1.6 / N | ECSS profile; Adopt-profile | Establish defined configuration baselines at lifecycle milestones and control each baseline under assigned authority. | Baseline type, acceptance event, exact member revisions, authority; verify no post-release member rewrite. |
| REV-STD-BL-003 | ECSS-40 §§5.3.7.2.5(d), 5.3.7.3.1(g) / N | ECSS profile; Core-candidate | Protect released information integrity and deliver only after the review cycle and required approvals complete. | Content digest, approval completion, release transaction; tamper detection and incomplete-approval refusal. |
| REV-STD-BL-004 | DOD-31000 §5.2.3 / N | Contract-selected product-level TDP; Adopt-profile | Product data reflects the approved, tested, accepted delivered-item configuration and supports manufacture without recourse to the original design activity. | Released TDP plus qualification/acceptance evidence; independent reconstruction and completeness review. |
| REV-STD-BL-005 | DOD-61 §§5.5.2.1, 7 / G | Reference; Reference | Treat each approved baseline as a point of departure, preserve the controlled path between baselines, and report current/historical state. | Candidate baseline-diff and status-accounting queries; scenario evaluation, not conformance. |

### Change control, impact, departures, and effectivity

| ID | Authority / clause / class | Applicability and disposition | Requirement and candidate Datum control | Evidence and verification |
|---|---|---|---|---|
| REV-STD-CHG-001 | ECSS-40 §§5.3.2.3-5.3.2.5 / N | ECSS profile; Core-candidate | Formally initiate changes, assess technical/programmatic/operational impact, and disposition each as approved with applicability/mode, rejected with rationale, or deferred. | Typed change, impact set, board decision, rationale and applicability; state-machine and missing-impact refusal tests. |
| REV-STD-CHG-002 | ECSS-40 §5.3.2.1(e), §5.3.2.4(b-c) / N | ECSS profile; Core-candidate | Send common-element changes to every affected actor and assess all affected products. | Dependency-derived affected set and reviewer routing; seeded cross-product impact test. |
| REV-STD-CHG-003 | ECSS-40 §5.3.2.6 / N | ECSS profile; Adopt-profile | Distinguish planned deviations from unplanned waivers; process them under competent authority, bound them to items/time, and assess common-element impact. | Typed departure, authority, affected items, validity interval, rationale; expiry and out-of-scope refusal tests. |
| REV-STD-CHG-004 | ECSS-40 §5.3.2.7 / N | ECSS profile; Core-candidate | Update baselines and controlled documents only in accordance with approved changes/deviations. | Change-to-successor-baseline links and affected-document dispositions; orphan-change and unchanged-revision refusal tests. |
| REV-STD-EFF-001 | ECSS-40 §5.3.2.5(a)(1), Annexes H-I / N | ECSS profile; Core-candidate | Record approved change applicability and implementation mode; deviation effectivity can identify model or serial number. | Structured effectivity expression and resolved affected population; boundary-value tests over models/serials. |
| REV-STD-EFF-002 | DOD-61 §5.6.2.1.4, Table I / G | Reference; Reference | Support effectivity expressions that resolve to actual units—serial, lot, block, contract, or date—and report incorporation status. | Candidate effectivity resolver and status report; scenario evaluation, not conformance. |

### Status accounting, verification, and audit

| ID | Authority / clause / class | Applicability and disposition | Requirement and candidate Datum control | Evidence and verification |
|---|---|---|---|---|
| REV-STD-CSA-001 | ECSS-40 §5.3.3.1 / N | ECSS profile; Core-candidate | Record, store, and retrieve baseline, CI design/as-built, document/data-set, change/deviation/waiver approval and implementation, and review/action status. | Queryable status ledger with immutable source links; reconstruct sampled status at arbitrary event/time. |
| REV-STD-CSA-002 | ECSS-40 Annex F / N | ECSS profile; Adopt-profile | Report document/drawing identity, issue/revision/date, obsolete or model applicability, affected items, approval, change, and discrepancy status. | Generated CSA report and source links; field completeness and projection-consistency checks. |
| REV-STD-VER-001 | ECSS-40 §§5.3.4.1-5.3.4.2 / N | ECSS profile; Adopt-profile | Verify configuration definition at selected reviews and systematically compare as-built with as-designed configuration. | Review/audit record, comparison results, discrepancies; seeded mismatch detection and closure trace. |
| REV-STD-AUD-001 | ECSS-40 §5.3.5 / N | ECSS profile; Adopt-profile | Conduct internal audits of the actor's CM requirements and external audits of lower-tier supplier application. | Audit scope, profile/edition, evidence sample, findings, dispositions, closure; replay audit against frozen evidence. |
| REV-STD-AUD-002 | DOD-61 §8.2.2 / G | Reference; Reference | Functional and physical configuration audits compare performance evidence and physical/product definition, record discrepancies, and track actions to closure. | Candidate audit workflow; evaluate with divergent as-designed/as-built fixture. |

### Document/drawing revision and approval/signature

| ID | Authority / clause / class | Applicability and disposition | Requirement and candidate Datum control | Evidence and verification |
|---|---|---|---|---|
| REV-STD-DOC-001 | ECSS-40 §5.3.7.2.5(a-d) / N | ECSS profile; Core-candidate | Justify and trace document changes, route controlled-document updates through CM, and guarantee released-document integrity. | Change justification, predecessor/successor issue, governing change and digest; unauthorized-edit refusal and trace traversal. |
| REV-STD-DOC-002 | ECSS-40 §§4.3.8.3-4.3.8.4 / G, implemented normatively by §§5.3.7.2.5(d), 5.3.7.3.1(g), 5.3.7.4 / N | ECSS profile; Core-candidate | The explanatory lifecycle makes release follow approval and distribution follow release; the normative controls require released integrity, completed approvals before delivery, and controlled delivery. Datum's stronger successor-version rule remains product policy until its exact profile mapping is ratified. | Approval completion, release event, version identity, delivery references; post-release mutation test preserves the released object and creates governed successor work under REV-DATUM-003. |
| REV-STD-DOC-003 | ECSS-40 §5.3.7.2.1(e) / N | ECSS profile; Adopt-profile | Apply the same authorities and approval process when revising information/documents as for prior issues. | Approval-policy identity per issue; compare signer-role requirements across revisions. |
| REV-STD-SIG-001 | ECSS-40 §§5.3.7.3.2-5.3.7.3.3 / N | ECSS profile; Tailoring unresolved | Support the selected compliant approval method. ECSS digital signatures require recognized standards and reject self-signed certificates; process approval has separate visible/provenance requirements. | Signature/approval method, identity, credential chain, time, covered digest; cryptographic/process verification and invalid-credential refusal. |
| REV-STD-SIG-002 | DOD-31000 §5.13 / N | Contract-selected TDP profile; Adopt-profile | Digital approval indicators satisfy contracting-activity requirements for uniqueness, verifiability, and sole control. | Approval identity, verification material, control attestation, covered content; duplicate/forged/replayed approval tests. |
| REV-STD-DRAW-001 | ASME-35 scope / S | Drawing profile; Blocked-text | ASME confirms Y14.35-2025 governs identifying and recording revisions to product-definition datasets and associated documents. Exact practices remain intentionally unspecified here. | Licensed clause matrix and owner-approved mapping required before any `ASME Y14.35` claim. |
| REV-STD-DRAW-002 | ASME-34 and ASME-100 scope / S | Drawing/TDP profile; Blocked-text | Associated-list and engineering-drawing revision details require the licensed standards used together with Y14.35. | Licensed clause matrix, edition pin, and test specimens required before conformance. |

### Records, retention, package delivery, and transmittal

| ID | Authority / clause / class | Applicability and disposition | Requirement and candidate Datum control | Evidence and verification |
|---|---|---|---|---|
| REV-STD-REC-001 | NASA-D §3.2.15.2(d-e) / N | NASA profile; Core-candidate | Maintain integrity, traceability, and security through the CI life and preserve configuration records through the lifecycle under NPR 1441.1 disposition. | Record class, protection, retention/disposition rule, legal hold and destruction event; access/tamper/retention policy tests. |
| REV-STD-REC-002 | ISO/TC 176/SC2 N1286 discussing ISO 9001:2015 §§4.4, 7.5 / G | QMS reference; Reference pending licensed ISO text | Maintain information needed to operate processes and retain enough evidence to show they ran as planned; organization context determines extent. | Candidate configurable record policy; do not claim ISO 9001 conformity from the guidance document. |
| REV-STD-RET-001 | ECSS-40 Annex A §&lt;5.3&gt;(c) / N | ECSS profile; Adopt-profile | Define the information-retention period in the CM plan. | Versioned retention schedule with approving authority and effectivity; boundary, hold, and disposition tests. |
| REV-STD-PKG-001 | ECSS-40 §§5.3.7.4-5.3.7.5, Annex L / N | ECSS exchange profile; Adopt-profile | Deliver controlled information/packages with defined identification, metadata, integrity, format, and distribution authorization. | Frozen package manifest, files, metadata, recipient authorization, delivery event and receipt; hash/manifest and authorization verification. |
| REV-STD-PKG-002 | DOD-31000 §§5.4.1.3(f), 5.14.1 / N | Contract-selected TDP profile; Adopt-profile | Include ownership/control metadata and a TDP index carrying document number, responsible activity, approval, revision, date, and applicable handling statements. | TDPL and element metadata; required-field, classification/export-policy, and exact-content checks. |
| REV-STD-XMIT-001 | No reviewed source creates one universal transmittal schema / D | Core product need; Unresolved for REV-C03 | Datum needs an immutable delivery event distinct from package/document identity, but recipient, receipt, classification, export, supersession, and withdrawal fields must be profile-driven. | Proposed transmittal record and profile schema; resolve against ECSS, DoD, customer, and lightweight scenarios before ratification. |

## Licensed-source gates

The following are deliberate gaps, not permission to approximate a standard:

1. Acquire and review lawful copies of ISO 10007:2017, EIA-649C,
   EIA-649-2A, ASME Y14.35-2025, Y14.34-2013 (R2024), Y14.100-2017,
   and any 9100 edition Datum intends to name in a conformance profile.
2. Record each exact clause, obligation owner, applicability, tailoring rule,
   objective evidence, and verification method without embedding licensed text
   in the repository beyond what its license permits.
3. Ratify a profile only after legal/licensing posture and the complete clause
   crosswalk are owner-reviewed. Official scope pages and secondary summaries
   cannot substitute for normative text.
4. Treat future editions as new authorities. A profile remains pinned to its
   recorded edition until an explicit migration maps changed, added, and
   removed requirements.

## Datum product choices kept outside external authority

These current owner constraints enter REV-C03 as Datum policy rather than
being misattributed to a standard:

| ID | Datum policy | REV-C03 question |
|---|---|---|
| REV-DATUM-001 | The Product Revision Engine works completely offline and standalone; Git is an optional, non-authoritative adapter. | Exact authority objects, persistence, recovery, and Git mapping. |
| REV-DATUM-002 | Every committed Datum mutation has technical revision/provenance, while an issued engineering/document revision is an approved configuration—not an edit count. | Boundary between transaction, technical revision, change, baseline, document issue, and release. |
| REV-DATUM-003 | Released configurations and documents are never silently rewritten; later change creates governed successor work. | Successor allocation timing, affected-document state, and refusal behavior. |
| REV-DATUM-004 | Audit profiles must be machine-queryable, clause-addressable, edition-pinned, evidence-linked, and explicit about tailoring/non-applicability/unresolved findings. | `StandardsProfile`, `RequirementDisposition`, audit execution, and immutable report model. |
| REV-DATUM-005 | Lightweight use stays quiet while regulated profiles can require stronger roles, approvals, evidence, effectivity, and retention without a second architecture. | Universal substrate versus enabled profile controls. |

## REV-C02 conclusions that constrain REV-C03

1. Datum needs one universal identity/provenance/baseline/change/status substrate,
   but not one universal bureaucracy. Applicability and tailoring are first-class
   data, never comments or UI-only preferences.
2. `technical revision`, `document issue`, `configuration baseline`, `release`,
   `package`, and `transmittal` must remain separate identities. The primary
   sources repeatedly distinguish the controlled product state, its documents,
   approval/release, and delivery.
3. Effectivity is a resolved applicability expression, not free text. Profiles
   may enable serial, lot, model, variant, build, customer, site, contract, or
   date selectors, but every selector must resolve to an auditable population.
4. Waivers and deviations do not silently mutate requirements or baselines.
   They are bounded, approved departure records with affected-item and
   effectivity evidence.
5. A repeatable audit requires immutable evidence references and historical
   queries. A generated prose report without a frozen evidence set is
   insufficient.
6. Retention is policy-driven. Datum must preserve records until the applicable
   schedule permits a traceable disposition; it must not bake one aerospace or
   government retention duration into the core.
7. Digital signature requirements vary materially by profile. REV-C03 must
   define a general approval attestation first; specific credential/signature
   mechanisms require explicit profile and dependency decisions.

## Coverage check

| Required REV-C02 domain | Matrix coverage |
|---|---|
| Configuration identification | REV-STD-ID-001..004 |
| Baselines | REV-STD-BL-001..005 |
| Change control | REV-STD-CHG-001..004 |
| Status accounting | REV-STD-CSA-001..002 |
| Audits | REV-STD-VER-001, REV-STD-AUD-001..002 |
| Drawing revision | REV-STD-DOC-001..003, REV-STD-DRAW-001..002 |
| Approval/signature | REV-STD-SIG-001..002 |
| Effectivity | REV-STD-EFF-001..002 |
| Records | REV-STD-REC-001..002 |
| Transmittal/package delivery | REV-STD-PKG-001..002, REV-STD-XMIT-001 |
| Retention | REV-STD-RET-001, REV-STD-REC-001 |
| Release | REV-STD-BL-003..004, REV-STD-DOC-002 |
