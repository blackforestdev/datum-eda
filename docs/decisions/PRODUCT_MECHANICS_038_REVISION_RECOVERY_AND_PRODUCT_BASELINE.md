# Product Mechanics 038: Revision Recovery and Product Baseline

Status: ratified corrective doctrine

## Context

The owner rejected the running REV-I10 result on 2026-08-30 and stopped the
Revision Engine execution sequence. The review exposed a product-level failure,
not merely a visual-conformance defect. The prior contract required permanent,
expanded Revision navigation from Project creation while also promising that
unmanaged designers could ignore Revision completely. The implementation made
that contradiction concrete: ordinary Projects displayed invented Change,
baseline, Release, controlled-document, and evidence records; selecting Project
navigation opened Revision panes; and Revision presentation displaced the
manual Design workspace before a Project had deliberately adopted revision
control.

The owner also reported that the earlier approval process was not
comprehensible enough to support informed consent. A large combined evidence
route coupled workspace, Revision, Global Preferences, Units, enterprise
authority, and many visual studies into one review surface. Prior approvals
remain historical evidence, but they do not compel continuation of a product
direction the owner has now explicitly rejected.

## Corrective decision

Datum remains a manual-first EDA application. Library, schematic, and PCB work
are the ordinary product path. Revision control is an optional governed
capability that appears when a Project deliberately adopts it or when the user
explicitly invokes a Revision action. It is not permanent application chrome,
an onboarding requirement, or a teaching surface imposed on an unmanaged
Project.

The following baseline supersedes conflicting portions of Product Mechanics 034,
Product Mechanics 037, the Product Revision Engine specification and
implementation plan, and the approved REV-C06 visual dispositions:

1. An unmanaged Project has no permanent Revision group, pane, badge, empty
   state, placeholder record, or revision-specific teaching strip in its normal
   Navigator or Design workspace.
2. Runtime UI may render only records resolved from the open Project. Examples,
   demonstrations, fixtures, and screenshots must be unmistakably isolated from
   production project data. Hard-coded fictional Change, baseline, Release,
   controlled-document, or evidence identities are forbidden in the running
   product.
3. Revision becomes discoverable through explicit user intent (for example,
   Prepare Revision, compare to a real baseline, or open a real controlled
   record) or through a Project policy/record that the Project deliberately
   adopted. Discovery must not interrupt or displace ordinary authoring.
4. Global Preferences may control machine-local presentation defaults and may
   seed a Project choice at creation. It does not own operational Revision
   workflow, create Project records, or make Revision visible in an unmanaged
   Project. No organization directive may force teaching chrome into an
   unmanaged personal workflow.
5. Formal revision policy and its governed surfaces are available only after
   deliberate adoption. Existing technical journal, integrity, recovery, and
   immutable-evidence mechanisms may operate below the UI without pretending
   that an unmanaged Project has engineering Change or Release authority.
6. Revision presentation must be subordinate to a readable Design workspace.
   Generic pane clipping, minimum usable size, scrolling, selection, and
   activation behavior remain reusable GUI requirements, but no Revision
   surface may be used to justify permanent navigation or layout occupation.

## Recovery disposition

The existing work is classified before further implementation:

- **Retain:** the separation between technical history and human-issued
  revision; exact immutable baseline/reproduction concepts; append-only
  integrity, recovery, and audit primitives; typed preference identity,
  provenance, migration, and Project-versus-machine separation; and generic GUI
  clipping, scrolling, and pane-usability fixes.
- **Rework:** Revision scope, default discovery, navigation, onboarding,
  preferences integration, sample-data handling, roadmap sequencing, and the
  owner-review process.
- **Quarantine:** the broad authority-family implementation, mandatory
  journal-commit integration, public Revision operation/query catalog,
  enterprise role/adaptor workflow, and Project revision gate until each is
  separately justified against a real user workflow and explicitly
  reauthorized.
- **Remove from production presentation:** every hard-coded or fictional
  Revision record and every default Revision surface in unmanaged Projects.

Quarantine means no successor execution, production-acceptance claim, or
downstream dependency may rely on the quarantined behavior. It does not direct
a wholesale Git revert: Revision work is interleaved with reusable terminal,
layout, and substrate changes and must be separated by bounded corrective work.

## Execution stop and replacement gate

REV-I10 is rejected as a production UI result. REV-I17 is closed as rejected,
not approved. REV-I18, REV-I10A, REV-I19, REV-I11, REV-I12, and REV-I20 are
suspended and unauthorized. The Product Revision Engine build is deferred and
no longer unblocks Publish or MCAD work.

<!-- REQ:REVISION-PRODUCT-RECOVERY:RVR-A01 -->
The only active Revision work is the bounded recovery Frontier item. This
governance transaction establishes its controlling baseline and then it must:

<!-- REQ:REVISION-PRODUCT-RECOVERY:RVR-A02 -->
1. reconcile the Claude-owned visual-truth files through their owning lane,
   marking the permanent-Q1 contract superseded and drawing contextual/opt-in
   behavior;
<!-- REQ:REVISION-PRODUCT-RECOVERY:RVR-A03 -->
2. inventory runtime and engine behavior as retain, rework, quarantine, or
   remove, with tests that distinguish real Project data from examples;
<!-- REQ:REVISION-PRODUCT-RECOVERY:RVR-I01 -->
3. remove fictional runtime records and unmanaged default Revision surfaces;
4. prove ordinary library, schematic, and PCB authoring remains readable,
   prompt-free, and unaffected in a real unmanaged Project; and
<!-- REQ:REVISION-PRODUCT-RECOVERY:RVR-O01 -->
<!-- OWNER:REVISION-PRODUCT-RECOVERY:RVR-O01:RVR-O01 -->
5. return to the owner with a small, comprehensible running-app review packet
   before any wider Revision scope is reauthorized.

Global Preferences implementation and the shared Units execution remain
blocked. Their reusable authority primitives are preserved, but Revision
visibility, default profile, teaching-pin, and onboarding descriptors are
pending recovery and cannot be implemented from the old catalog.

## Claude-owned visual handoff

RVR-A02 is a bounded handoff to the protected prototype lane. Codex must not
edit these files.

- `revision-ux-shell-study.html`, Q1/Navigator region: mark permanent expanded
  Revision navigation as superseded and compare contextual discovery through
  explicit Revision intent or real adopted Project authority. Preserve one
  native window, ordinary Design/Publish navigation, single-select behavior,
  open-beside semantics, readable pane bounds, and accessibility.
- `revision-ux-study-index.html`, Q1 disposition/index region: identify the old
  Q1 result as superseded by Product Mechanics 038 and link the replacement
  study without implying that Q2-Q6 or engine breadth is reapproved.
- `preferences-window.html`, Revision region: remove factory-visible Revision
  and managed teaching-pin assumptions. Show that Global Preferences cannot
  adopt Project revision policy or expose Revision chrome in an unmanaged
  Project. Preserve typed provenance, Project-seed separation, and the rest of
  the ratified Preferences window.
- `revision-carryforward-study.html`, unmanaged/managed comparison: reconcile
  the unmanaged state to silence by default and remove any implication that
  organization teaching policy can force chrome into it. Preserve the
  distinction between machine onboarding state and deliberately adopted
  Project authority.
- `project-preferences-revision-gate.html`, Revision policy row and guidance
  strip: keep the artifact clay, preserve Unmanaged as the zero-setup state,
  and remove the implication that authoring may prompt or that opening Project
  Preferences itself adopts formal revision control.

Proof is a Claude-owned commit plus renders of each changed state, a concise
before/after reconciliation list, keyboard/non-color/narrow-layout review, and
owner confirmation of the contextual default. No visual marker may be treated
as runtime implementation authorization.

## Approval-process correction

Future owner decisions must be bounded to one understandable product question,
show the ordinary user workflow first, state what becomes visible by default,
and identify concrete runtime consequences. A decision packet may not treat a
large evidence digest, an earlier prototype marker, or agent-authored wording as
substitute for informed owner understanding. Approval of a mechanism does not
approve all later visual projections of that mechanism.

## Owner evidence

On 2026-08-30 the owner explicitly refused REV-I10, paused the other coding
session, accepted the proposed recovery sequence and product baseline, and then
directed Codex to proceed with the governance-only recovery transaction. This
decision records that correction; it does not infer approval of a replacement
prototype or authorize corrective runtime implementation.

<!-- EVIDENCE:REVISION-RECOVERY:PM-038-OWNER-APPROVED -->

## Dependency and licensing impact

None. This decision adds no dependency or license exception under Product
Mechanics 029.
