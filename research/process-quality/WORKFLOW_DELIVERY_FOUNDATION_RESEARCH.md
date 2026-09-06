# Workflow delivery: bounded foundational research dispositions

Date: 2026-09-05
Status: WDQ-C02 research; recommendations, not amended CAD authority
Owner route: `workflow-delivery-quality`

## Question and review boundary

Why can Datum have substantial engine code and governed specifications while
ordinary manual workflows remain unavailable or require repeated owner correction?
The C01 audit supplies bounded observations, not an estimate of total code quality.
This report resolves the delivery-process question and assigns specific remaining
CAD questions; it does not claim to finish their domain research in this lane.

Reviewed in full: the owner brief, capability baseline and planning consumer in
this route; Product Mechanics 001 (canonical edit model), 002 (manual baseline),
and 012 (quality bar). The three decisions were not assigned to an evidence route
in the current manifest. Their explicit ratified subsections remain controlling;
their open questions are not silently resolved here. Other foundation ownership
below is a **routing inventory**, read from the evidence manifest and Frontier,
not a semantic re-review or approval of those routes. Their complete sources and
consumers must be read by the scheduled follow-on before specification changes.
In particular, this report does not refresh the large active
`workspace-documentation-and-revision` route or review Preferences on its behalf.

## Primary-source findings

Sources accessed 2026-09-05. External practice informs recommendations; it is not
a Datum requirement or permission to copy implementation or add dependencies.

1. NASA's Systems Engineering Handbook separates requirement verification from
   validation of stakeholder use in the intended environment. It explicitly
   discusses systems passing verification but failing validation and recommends
   early operational scenarios and user involvement. Applicability: define and
   exercise Datum's manual paths before treating engine proof as delivery.
   NASA mission certification and its organizational scale are not adopted.
   [Product realization, sections 5.3–5.4](https://www.nasa.gov/reference/5-0-product-realization/).
2. NASA describes bidirectional requirements traceability, including links to
   verification and validation evidence. Applicability: use existing Frontier
   requirements and evidence routes, adding scenario-to-consumer links instead
   of creating another task authority. A linked document alone does not establish
   that its assertions are correct.
   [Requirements management](https://www.nasa.gov/reference/6-2-requirements-management/).
3. KiCad's pinned PCB Editor 9.0 manual (page identifies 9.0.9) distinguishes
   display/entry units, expression conversion, dimension display precision and
   grid visibility versus snapping. Rule units are independent of editor display
   units. Applicability: these are useful counterexamples to treating “units” or
   “grid” as one cosmetic toggle. They are comparative behavior, not a mandate
   to copy KiCad's choices or override Datum PM040. Review bounded scenarios for
   unit changes, explicit suffixes and hidden-grid editing with Datum's owners.
   [Units, expressions and grid behavior](https://docs.kicad.org/9.0/en/pcbnew/pcbnew.html).

## Findings and recommended dispositions

| Finding | Evidence and authority | Disposition |
| --- | --- | --- |
| Manual-first doctrine does not itself deliver a usable path | PM002 requires manual operations, bindings, outputs and reopen; C01 observes partial viewing but unavailable authoring on the tested path | Require native input and visible/persisted result proof separately from engine coverage. Do not revise PM002 downward to match the implementation |
| Availability can overstate dispatch readiness | C01 actual About activation refuses despite an enabled menu item; `dat-gui-local-readiness-uev` owns the defect | Pilot must compare the actual production consumer registry and context eligibility with all pilot entry surfaces; an action-class label is insufficient |
| Foundational contracts cross apparent feature boundaries | PM001's journal/identity requirements and PM002's direct-edit workflow; comparative unit/grid behavior above | Require a consuming-workflow foundation matrix before implementation, with explicit unknowns and one accountable owner per question |
| Source classification can conceal open mechanism | PM012 header says draft hypothesis while its governance entry is `doctrine`; PM001/002 retain explicit open questions | Record the ambiguity for WDQ-F01 clause-level reconciliation. Do not infer that every open question is ratified, or discard explicit ratifications because of a broad draft header |
| Successful inspection is narrower than delivery | C01 selected a pad, zoomed and reopened unchanged source; it did not perform a durable edit or generate manufacturing outputs | Preserve observed capability and explicit unverified/unavailable states. No percentage or broad “EDA complete” claim |
| Checks cannot judge intent unaided | A traceability digest proves the reviewed input set changed, not that an operator's expectations are met | Independent replay/review plus explicit owner acceptance binds the precise reviewed packet; no self-written reviewer name can supply that authority |

## Foundation-to-consumer research map

The following are **questions**, not new numeric policies or findings that a
particular existing specification is missing. Each exits with a clause-level
answer, reviewed examples and tests, or an explicit owner decision blocking the
affected scenario. An unresolved mandatory answer may not become `N/A`.

| ID / user scenario and risk | Existing authority route / accountable scheduled owner | Required bounded output and completion boundary |
| --- | --- | --- |
| F1: change working/display units before and during exact entry; avoid geometry rescaling, silent rounding and stale labels | `workspace-documentation-and-revision`; GLOBAL-PREFERENCES-COMPLETION and PROJECT-PREFERENCES-SPEC; PM040 owner retains semantics | Units/precision/coordinate table covering storage, transforms, display, explicit and implicit input, output, overflow and round-trip examples. Reuse accepted units decisions; request owner disposition only for genuine gaps |
| F2: type a negative, out-of-range, mixed-unit or temporarily incomplete coordinate; avoid silent commit of unintended values | Same route for unit authority; NATIVE-AUTHORING owns field-to-operation use | Grammar and parse/preview/commit states, locale boundary, error location, rounding and cancel behavior, named consumers and exact-value tests. Separate display precision from permissible authored precision |
| F3: select a pad within a component, change focus/view, then edit; avoid mutating the wrong stable object | `gui-selection` and `prototype-selection`; UVT-S5A-BUILD, with PM026 authority | Identity/granularity/filter/focus/inspector mapping and selection-change tests. Compare visual and engine target identities, including stale selection; preserve current Claude visual truth |
| F4: hide grid, zoom or rotate a view, then place at an exact location; avoid screen/model and snap ambiguities | UVT-S5A-BUILD plus NATIVE-AUTHORING; inspect their complete routed specs before adjudication | Coordinate frames, origin, transform inverses, snap precedence/tolerance and visibility semantics, boundary-value fixture. Display zoom must not substitute for geometry proof |
| F5: begin/preview/cancel/commit/undo/redo an edit; avoid hidden writes or unusable history | PM001/002, GUI-WRITE-PATH, writer-lock issue `dat-project-write-ownership-lock-0ne` | One transaction timeline with no-write cancel and durable undo checks, error/partial-commit behavior and multi-surface parity. Resolve applicable direct-versus-proposal open questions explicitly |
| F6: create/open/reopen, encounter an unavailable directory or second writer, recover after interruption; avoid loss or unowned writes | New doorway issue `dat-native-project-startup-vrf`; GUI-WRITE-PATH and existing revision/recovery owners in `workspace-documentation-and-revision` | Ordinary launch/chooser/context contract and storage/writer-state matrix, crash-injection proof plan and provenance. Do not revive withdrawn Revision product concepts or equate clean unchanged reopen with edit recovery |
| F7: resolve a project-local part, place a symbol, connect it and realize a board component; avoid missing/mismatched pins or string-based joins | GUI-SURFACE-SPECS and NATIVE-AUTHORING; PM001 ComponentInstance and PM002 binding baseline | Small local fixture, missing/stale binding and connectivity cases, cross-domain handoff and exactly scoped first proof. Broad library service, full router and enterprise PLM are excluded |
| F8: change a setting with two projects/documents open; avoid preferences modifying design authority or taking effect in the wrong scope | `workspace-documentation-and-revision`; GLOBAL-PREFERENCES-COMPLETION, PROJECT-PREFERENCES-SPEC, existing product-revision owners | Scope/default/inheritance/override/timing/reset/persistence table with affected consumers. Review complete current route; no second preferences store, implementation or speculative prototype edits in WDQ |

## Scheduling and stopping rules

`FOUNDATION-WORKFLOW-SPEC` / `dat-manual-foundation-contracts-fsw` collects F1–F8
into **one bounded pre-implementation contract**, governed by WDQ-F01–F03 in the
WDQ plan. At C02 it was blocked on WDQ adoption (subsequently approved at C05);
it is not canonical-next
or an execution authorization. The existing owners above are consumers and
review boundaries, not newly assigned parallel work. Their claims, completion
criteria and implementation dependencies remain unchanged.

The existing doorway intake `dat-native-project-startup-vrf` depends on this
contract before implementation promotion. It is related to GUI-WRITE-PATH and
NATIVE-AUTHORING; this does not claim that either existing owner has accepted a
changed schedule. The About/readiness defect is linked to WDQ for the proposed
pilot and remains a separate correction, not silently fixed by this report.

WDQ-F01 must read complete identified routes, reconcile clause authority and
confirm owners against fresh selectors. WDQ-F02 produces concrete examples and
explicit decisions for F1–F8, using additional primary research only where the
current authority does not answer the question. WDQ-F03 presents the consuming
owners' reconciled contract for owner disposition, then proposes exact changes
to existing completion prerequisites. This is the deliberate stop before those
dependencies become blocking policy. No new standards/compliance or “enterprise
class” certification is implied.

WDQ-C02 exits here: the process recommendation is supported, every foundation
question has a scenario, risk, owner, output and decision boundary, and bounded
specification work is registered. CAD questions F1–F8 and clause-status cleanup
are deliberately **not claimed resolved** until that scheduled work is reviewed.
