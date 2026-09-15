# Revision Sequencing & Origin — Research

> **Status:** Claude-owned research artifact (2026-08-24), commissioned by the
> owner after halting REV-C03-Q2. Evidence base for the revision-identity
> scheme decision that was never ratified (alpha-sequential entered through
> examples, not a decision — title-block research open question 3, unanswered
> since 2026-07). Feeds the owner decision packet; ratifies nothing.
> Four evidence streams: formal CM standards (ASME Y14.35, EIA-649,
> MIL-HDBK-61, ECSS-M-ST-40C) · ISO 19650 · PLM/PDM systems (Teamcenter,
> Windchill, Arena/Agile, Onshape, SolidWorks PDM, AllSpice) · hardware field
> practice (EVT/DVT/PVT, board spins, certification). Source links in §8.

## The three commissioned questions

1. Which revision sequencing fits the industry standards Datum targets?
2. **Origin:** when does the clock start on revision identification — first
   contribution (a single resistor placed) or periodic milestones?
3. What defines a milestone?

## 1. The unanimous architecture: two clocks, never one

Every surveyed standard and every surveyed system separates:

- a **dense, automatic, machine-owned clock** that ticks on every save/commit
  (Windchill iterations `A.1→A.2`, Teamcenter dataset versions, SolidWorks PDM
  check-in versions, ISO 19650 WIP versions `P01.01→P01.02`, git commits,
  EIA-649 "working data"/file versions) — never user-minted, often prunable,
  never contractually meaningful; and
- a **sparse, human-owned revision identity** minted only by an explicit
  event (Revise, ECO release, workflow transition, release candidate, share,
  fab release) — immutable once issued.

EIA-649 states this as principle: *document revision identifiers* and *file
version identifiers* are distinct, and the mapping between them must be
maintained. Git proves the extreme form: content hashes carry no human
meaning; tags are sparse human names layered on by explicit action.

**Datum already owns clock one.** The journal, `ObjectRevision`, and
`ModelRevision` begin at the first committed contribution — the first
resistor — automatically, silently, at no cost to the designer (verified in
the REV-C01 baseline). So the answer to "does the clock start at the first
resistor?" is: **the technical clock does, and must; the human revision clock
must not.** The entire open design question concerns clock two only.

## 2. What the targeted standards actually require (sequencing)

| Standard (Datum matrix profile) | Scheme | Origin rule | Composition rule |
|---|---|---|---|
| **ASME Y14.35** (drawing profile) | Uppercase letters, omit I·O·Q·S·X·Z, then AA…; ≤3 chars | **Initial release carries NO letter — it is "−" (dash). Letter A is minted by the first change AFTER release.** Pre-release drafts have no revision vocabulary at all | Drawing-level, sheet-level, or all-sheets-same methods; associated lists "need not be revised … solely to maintain a common revision level" |
| **EIA-649/649C** (core CM) | Scheme-agnostic: unique identifiers; letters/numbers are org policy | Release ("designation … approved by an appropriate authority") mints; everything before is *working data* (file versions) | Baseline = manifest of documents **each at its own specific revision**; baselines established "by agreement — any agreed point of departure" |
| **MIL-HDBK-61** (DoD profile) | PIN + dash numbers; non-interchangeable change ⇒ **new identity, not new revision** | *Engineering release* (including internal releases of the evolving "developmental configuration"); formal control attaches at functional/allocated/product baselines | Multi-level tree; tabulated/dash numbers; next-higher-assembly traceability required; roll-up is interchangeability judgment |
| **ECSS-M-ST-40C** (space profile) | **Two-tier issue.revision** (issue increments for re-issue, revision resets; e.g. 2.3) | "In Preparation" versions are non-binding; "after the document is released, any modification implies a new version" | Milestone-pegged baselines (MOB/FCB/DCB/DB/PCB at PRR/SRR/PDR/CDR/QR); per-CI CIDL; as-built ABCL per serial |
| **ISO 19650** (BIM/exchange profile) | **Two-axis: revision (P01…/C01…) × suitability status (S/A codes)** per information container | Provisional `P01.01` reserved at first save (private, revertible); **revision proper minted at first share beyond the authoring team**; publication re-mints as C01 under a separate sequence | Strictly per-container; **no project-wide revision exists**; renditions revise independently of sources |

**Conclusion for question 1:** no standard Datum targets requires — or even
describes — a single project-wide linear letter sequence. All of them mint
identity at release-class events, support per-item identity, and identify
compositions through baselines/manifests, not one letter. Alpha-sequential is
one *label format* inside one profile family (ASME), whose own origin rule
(dash, then A at first post-release change) contradicts the "start at A"
folklore. The engine must model the minting event and the namespace; the
label format is profile policy — exactly the pattern of
`DrawingStandardProfile`.

## 3. Origin — the spectrum of answers found in the field

Ordered from earliest to latest minting:

1. **At first contribution (automatic)** — only ever the *technical* clock.
   No surveyed standard or product mints human revision identity per edit.
2. **Provisional-from-birth** — Teamcenter/Windchill school: the object is
   born at `A.1`; the letter exists but is cheap, mutable in meaning, frozen
   only by release. ISO 19650's `P01.01` at first save is the same idea with
   honest marking (the suffix declares it a draft of a future revision).
   Risk: an unreleased "Rev A" *looks* authoritative before it means anything.
3. **At first share** — ISO 19650's WIP→Shared transition: identity is minted
   by crossing the authoring boundary; the version suffix drops and P01
   becomes durable. Sharing, not saving, is the event.
4. **At first release** — Onshape / SolidWorks PDM / AllSpice / electronics
   field practice: **no revision exists before first release**; "unreleased"
   is a first-class state with only version history. Y14.35's dash is the
   drafting-standard form of the same rule.
5. **At first post-release change** — Y14.35 strictly: the released original
   is "−"; letter A identifies the first *change*. Successor identity is
   literally defined as departure from a release.

Field hybrid worth noting: electronics teams commonly run **numeric prototype
revs minted per fab spin** (Rev 0/1/2 or P1/P2), then **reset to Rev A at
production release** — two sequences, one boundary, mirroring ISO 19650's
independent P and C counters.

## 4. What defines a milestone (question 3)

Across all four streams, a revision-minting milestone is always a **declared
crossing of a boundary** — never an edit count, a timer, or a calendar tick:

| Boundary crossed | Event examples | Evidence |
|---|---|---|
| To manufacturing | Fab release (board spin), assembly/BOM release | "A release package freezes the exact files sent to a manufacturer"; the spin is the revision because the physical artifact population must be discriminable (silkscreen rev, HWID straps) |
| To another party | ISO 19650 share (P0n) and publish (C0n); contractual CDRL delivery; customer submission | Identity minted when work leaves the authoring team's control |
| Through a phase gate | Proto/EVT1/EVT2/DVT/PVT builds; PDR/CDR review baselines (allocated/product baseline) | The build name ("the DVT1 config") is the *system-level* identity teams actually use — a named snapshot binding each item's revision |
| Into certification/regulatory scope | FCC grant + Class II permissive changes; CE substantial modification; UL re-evaluation | A controlled identity exists per submission even when Gerbers did not change |
| Into a controlled baseline by agreement | EIA-649: "any agreed point of departure can represent a baseline"; ECSS milestone baselines | Formality varies; declaration does not |
| Through a change disposition | Arena/Agile: **the ECO is the minting transaction** (old rev → new rev + effectivity recorded on the order) | Revision identity is inseparable from the documented reason it exists |

Two structural facts: a **phase is not a build, and a build is not a spin**
(EVT contains EVT1/EVT2; each build consumes spins) — so milestone identity
is itself hierarchical; and at least **four decoupled revision axes** exist on
a real product (bare board, assembly/BOM, design documents, system
configuration) plus firmware — they revise independently by design.

## 5. Complex products — composition identity (the owner's objection, confirmed)

The owner's objection to a single linear product score is confirmed
everywhere: nothing in the standards or the field auto-rolls a child revision
into a parent letter. Released parents **pin exact child revisions**
(Teamcenter precise BOMs, baseline manifests); in-work parents float by
policy (revision rules), and rolling a parent is a form/fit/function
judgment, not identity mechanics. The composition's identity is the
**baseline manifest** — which is precisely what Datum's already-drawn G4 seam
provides (`ConfigurationBaseline` as immutable manifest of exact member
revisions) and what the authority model's per-CI `revision_namespace`
gestures at. Non-interchangeable change re-identifies (new number), never
merely revises — the interchangeability rule is the only universal roll-up
law.

## 6. Implications for Datum (carried to the decision packet, not ratified)

1. **Model the minting event, not the label.** `EngineeringRevision` =
   (namespace, scheme-policy label) allocated by an explicit release-class
   event. The scheme registry is profile policy: ASME alpha (with dash origin
   and skip-set), numeric spin-based, ECSS issue.revision, ISO 19650 P/C +
   status axis, organization-custom, with optional prototype→production
   sequence reset. No unratified default.
2. **"Unreleased" must be a first-class, honest state.** Title blocks and
   chrome must be able to show "−"/unreleased instead of a fake Rev A — the
   Y14.35 dash is the standards-blessed precedent. (Every existing "REV A"
   in committed prototypes is scheme-placeholder pending the owner decision.)
3. **The technical clock stays silent and universal** (already built), with
   the EIA-649-mandated mapping exposed: issued revision ↔ journal
   transactions/baseline, never displayed as revision identity itself.
4. **Milestones are declared boundary crossings, plural in kind.** The engine
   should accept multiple release-class event kinds — manufacturing release,
   share/delivery, phase build, review baseline, certification submission,
   declared baseline — as first-class minting events, profile-configurable.
   The designer (or the org's gate policy) declares; the engine records.
   This directly answers "who starts the clock": the designer does, by
   crossing a boundary they chose to cross — never the editor.
5. **Phase builds deserve first-class support.** "EVT2" as a release-scope
   label binding each CI's revision is how real hardware teams identify
   system states; Datum's baseline manifest + release-scope revision
   namespace can make the build name a governed identity instead of a
   spreadsheet convention — a genuine differentiator.
6. **Successor-work collection must bind no identity.** Auto-collection of
   post-release transactions (REV-C03-Q2) is acceptable only decoupled from
   any presumed next-in-sequence label; label allocation happens at the next
   minting event under the scheme in force. (The authority model's separate
   expiring `RevisionReservation` is compatible with this.)
7. **The status/suitability axis (ISO 19650) is worth a profile.** "Which
   issue" and "what it is fit for" are different facts; conflating them into
   one letter is the folklore failure the owner identified.

## 7. Owner questions the decision packet must present (with rendered candidates)

1. Which scheme families ship in the v1 registry, and is there a lightweight
   default (candidate: numeric spin-based with first-class "unreleased",
   ASME alpha and ECSS/ISO schemes as profiles)?
2. Provisional labels before release (Windchill/19650 school, honestly
   marked) or none-until-release (Onshape/Y14.35 school)?
3. Which minting-event kinds are enabled by default (manufacturing release,
   phase build, share/delivery, review baseline, submission)?
4. Are phase-build names (EVT1, DVT2) first-class release-scope identities?
5. Does the v1 lightweight profile expose the suitability/status axis, or is
   it profile-only?
6. Prototype→production sequence reset: supported, default-on, or per-org?

Each question should be answered against rendered title-block/UI candidates,
not prose alone.

## 8. Sources

Formal CM: ASME Y14.35 public ballot draft (tsp.esta.org); NDIA/DoD Y14.35
deck (ndia.dtic.mil); EIA-649 (1998) full text (ncsx.pppl.gov); CMstat on
EIA-649C; MIL-HDBK-61A (acqnotes.com); ECSS-M-ST-40C (ecss.nl); Arena BOM
roll-up guidance; Eng-Tips threads on omitted letters/dash convention/assembly
roll-up. ISO 19650: UK BIM Framework Guidance Part C; CDBB National Annex
Guidance; Man and Machine, GlobalCAD, BibLus, BIM Corner, goto.archi,
Construction Management. PLM/PDM: PTC Windchill help + community (revisions/
iterations/lifecycles); Saratech, PLM Coach, Swoosh Tech (Teamcenter revise/
revision rules); Arena REST API + ECO template + best practices; PROSTEP;
Onshape release-management help + tech tips; GoEngineer/SolidWorks Help/MLC
CAD (PDM versions vs revisions); AllSpice revision-control concepts + "The
dangerous art of versioning"; semver.org; calver.org. Hardware practice:
Instrumental EVT/DVT/PVT handbook + "engineers speak in code"; AppleInsider/
The Apple Wiki prototype stages; agile.how; belitechnologies; fictiv;
pcbvault release packages; Embedded Artistry "Versioning PCBs";
electronicdesign.com on spins; Elsmar PCB/PCBA revision threads;
electrondepot PCB version-control thread; element14; Fasal Engineering and
hwe.design on HWID; JLCPCB/Protoexpress on markings; Cadence "ECOs Made
Easy"; pcdandf.com; acqnotes CDR / DAU PDR; FCC permissive-change policy;
Silex; Spilma/EMC FastPass (CE); UL; OpenBOM/Arena/PLM Advisors/Visure on
multi-level BOM revisioning.
