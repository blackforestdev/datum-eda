# GP-C05 Interaction and Visual Contract

> **Status:** owner-adopted refined Option A; activation and onboarding clauses
> amended by the owner GP-CM01 correction of 2026-09-05. The controlling visual surface is
> Claude-owned `docs/gui/prototypes/preferences-window.html` at commit
> `a061fca`. Accessibility and responsive states are controlled by Claude
> commit `e2127fa`; store-operation states remain controlled by `586eb0d`.
> No implementation, dependency, or prototype edit is authorized.
>
> **Tracker:** `dat-global-preferences-engine-qcv`
> **Frontier:** `GLOBAL-PREFERENCES-SPEC / GP-C05`

## 1. Disposition and boundary

The owner adopts refined **Option A**: one two-column Preferences window with a
section list on the left, settings pane on the right, and search pinned over the
settings pane. Search-first discovery, row-complete information, real controls
in results, resolver-owned explanation, and the drawn accessibility/context
states are one contract.

The `#layout=b` full-window search composition remains comparative evidence and
is rejected. This disposition does not reopen GP-C03 Q1–Q11, alter GP-C04
persistence, ratify any clay descriptor in the catalog, publish GP-C08
(historical alias GP-C05A), choose
a native toolkit or dependency, authorize implementation, or permit Codex to
edit any prototype.

## 2. Reviewed evidence

The complete `workspace-documentation-and-revision` evidence route was reviewed
against every listed source and consumer. The controlling interaction evidence
is:

- `preferences-window.html` at `a061fca`: two-column navigation and pinned
  in-pane search (`:33-42,82-88`); real row label, description, control, and
  provenance grammar (`:41-55,89-110`); cross-section matching over labels,
  descriptions, stable keys, and alternate/retired names with match reasons,
  grouping, counts, no-match truth, and real-row navigation (`:380-437`);
  rejected full-width comparison mode (`:438-442`); complete resolver answers,
  explicit absent sources, selection, and stale-panel closure (`:445-522`).
- `search-placement-study.html` at `e1dd9c6` is reviewed comparative evidence:
  A pins search over the content column (`:84-90`), while rejected B spans the
  window (`:91-97`). Behavior is otherwise held constant.
- `preferences-accessibility-study.html` at `e2127fa`: keyboard order and
  Escape/focus return (`:84-93`), reduced-motion invariants (`:94-100`), narrow
  stacking (`:101-107`), color-removed state proof (`:108-114`), and exact
  announcement priorities and accessible-name law (`:115-127`).
- `preference-store-states-study.html` at `586eb0d`: non-blocking corrupt-store
  recovery (`:78-83`), collision/refusal/unknown preservation (`:84-91`),
  migration without silent substitution (`:92-97`), reversible restore
  (`:98-102`), and common preservation law (`:103`).
- Guided setup and the Start page remain separate evidence requiring
  reconciliation under the 2026-09-05 activation amendment before either may
  ship. Revision carry-forward remains subordinate to Product Mechanics 038.
- Approved GP-C03 clauses remain the authority for setup/Q5/Q6/Q7/Q8/Q9/Q10
  and Q11; GP-C04 remains authority for transactions, migration, exchange,
  restore, audit, and the Project-policy seam. GP-C02 accessibility evidence
  (`GP_C02_EXTERNAL_AND_STANDARDS_RESEARCH.md:380-403`) and peer requirements
  (`GP_C02B_DOMAIN_PEER_PREFERENCES_RESEARCH.md:390-410`) constrain the result
  without importing a peer UI or dependency.

No reviewed Revision, Publish, Project, PM-034, PM-035, PM-036, GUI, or
traceability consumer conflicts with this presentation-only contract. Search
can expose Project facts but cannot turn them into machine Preferences; hiding
Revision UI cannot change Revision truth; no store or onboarding state enters
design data.

## 3. Search-first discovery principle

Search is a first-class Preferences capability, not a convenience filter.
Datum shall design descriptor identity, naming, retirement, documentation, and
accessibility so a user can find a setting without already knowing its section.
The section list teaches the available domains and supports browsing; pinned
search is the primary way in.

A production-active stable `PreferenceKey` and its registered retired or
alternate names are searchable vocabulary. Reserved candidates do not enter
the product search index. They remain storage and migration authority only
until separately activated.
Renaming a visible label or reorganizing sections shall not strand documentation,
tutorials, scripts, support guidance, or a user searching an older term.

## 4. Window, section, and row clauses

1. At ordinary widths the window has exactly two primary columns: a persistent
   section list on the left and the selected section's settings pane on the
   right.
2. Search remains pinned above the settings pane while its rows scroll. It does
   not span above the section list. Full-width placement B is rejected.
3. The section list exposes only sections containing production-active settings
   and visibly marks the selected section. **Manage Preferences** appears only
   after its GP-C04 operations are implemented. Project-owned policy is reached
   through the separate Project Preferences doorway and is never a Global
   section.
4. Every setting row presents its current label, plain-language consequence,
   real control or read-only action, and a provenance/status line directly
   beneath it. A tooltip is not the only carrier of any required fact.
5. Writable, managed, and refused states of an active descriptor use the same
   row grammar. Reserved, Unwired, planned, unavailable, and Project-owned rows
   remain absent rather than masquerading as Global settings.
6. Global Preferences never renders a Project-policy row. Project authority is
   inspected or edited only through the separate Project Preferences surface.

## 5. Search clauses

7. Search is always visible and focusable in ordinary navigation and searches
   every active row in every visible section. It never indexes reserved,
   planned, Unwired, or Project-owned candidates.
8. Matching covers the current label, plain description, stable
   `PreferenceKey`, and registered retired or alternate names. Matching is
   case-insensitive and never activates a retired identity.
9. Results are grouped under their section name, show the matched substring,
   retain the row's real control/read-only action and provenance line, and
   expose a live `matched of total` count.
10. When a match comes only through a stable key or former/alternate name, the
    result states that reason and names the vocabulary that matched.
11. Zero results say plainly that no active setting matches and state that
    search includes every visible Global section.
12. Starting or changing a search closes any explanation and clears its row
    selection before displaying new results; an explanation can never remain
    visibly associated with a different query.
13. Escape in search clears the query, exits results mode, and restores ordinary
    section navigation without applying or changing any setting.
14. Activating a result exits results mode, selects its owning section, scrolls
    directly without smooth-motion dependency to the canonical real row, and
    marks that row with the persistent selection treatment.
15. Activating the result's setting name performs clause 14 and opens its
    explanation. Search results never become a shadow settings surface: their
    controls invoke the same typed behavior as the canonical row.

## 6. Resolver-owned explanation clauses

16. Activating a setting name opens an explanation beside the retained list at
    ordinary widths. The list is not replaced, reordered, or recomputed by the
    presentation layer.
17. One resolver query supplies the effective value; every eligible source and
    control contribution; explicit absences such as **no user value**, **no
    organization directive**, and **no installation value**; the contribution
    marked **EFFECTIVE**; the Q4 eligibility → control → ordinary-value reason;
    descriptor identity/type/class/source facts; Context facts; redactions; and
    remaining user actions.
18. An absent source is printed as absent and never masquerades as a value or
    contribution. A displaced user value remains visible and retained; a
    Project-owned fact identifies machine sources as structurally ineligible.
19. GUI, CLI, and MCP render or return the same typed semantic answer. The GUI
    neither reconstructs resolution nor substitutes its provenance line for the
    complete answer.
20. Starting/changing search closes the explanation under clause 12. Escape or
    the close affordance closes it, clears selection, and returns keyboard focus
    to the setting name without moving the list.
21. At narrow widths the section list becomes a section chooser and the
    explanation stacks immediately beneath the selected real row. The row stays
    visible; open-beside becomes open-below, never open-over.

## 7. Editing, reset, refusal, and store-state clauses

22. A writable row uses its real typed control. A successful user change keeps
    focus on that control, updates effective value/provenance, states **set by
    you**, and exposes the exact reset target.
23. Reset is a named typed removal of the eligible user contribution, followed
    by re-resolution. It is not assignment of a guessed default, deletion of
    unknown data, or a Project mutation.
24. An invalid or ineligible attempt is announced as refused, names the law and
    controlling bound/source, retains the user's entry for correction, and
    leaves the last valid effective value truthful. Refusal does not disable
    unrelated rows or authoring.
25. Managed Recommend, Constrain, Pin, and Lock states name the directive,
    organization/provider, reason, accountable actor/role where available,
    offline state, retained user value, remaining freedom, and appeal or
    AuthorityRelease action. Managed values remain focusable and inspectable
    when not writable.
26. **Manage Preferences** is a non-descriptor operations section and remains
    absent until its GP-C04 preview, confirmation, import, backup, restore, and
    recovery paths are implemented and proved. Once active, unreadable data is
    preserved rather than repaired in place; import applies nothing before
    explicit collision choices; Capability/security authority refuses portable
    sources; unknown bytes survive; migration never applies an unchosen
    replacement; and restore is previewed, confirmed, and reversible.
27. No import, reset, migration, backup, restore, sync, explanation, or search
    action touches Project policy or design data, requires a network, or blocks
    Design authoring.

## 8. Revision, setup, Start-page, and Context reconciliation

28. Product Mechanics 038 withdraws Revision visibility, teaching, and policy
    rows from Global Preferences. No Revision descriptor or guidance surface is
    restored by this interaction contract.
29. First-Release guidance remains deferred Revision work. It is not a Global
    setting, active row, or GP-CM01 deliverable.
30. Guided setup remains deferred until all four approved checkpoints—
    Measurement system, Drafting standard, Appearance, and Files & Projects—
    have production-ready controls and consumers. No checkpoint may be skipped,
    renamed, or replaced merely to make setup appear complete.
31. The Q11 Start page remains local, non-tabbed, network-independent, and
    bypassable in principle, but its protected target must first be reconciled
    to the exact active seed authority. It may not advertise reserved settings,
    Revision state, or an unavailable drafting-standard seed.
32. Session, Context, operation input, restartable workspace state, and transient
    state retain approved Q6 separation. Context may explain applicability but
    cannot become a writable preference or authority contribution.

## 9. Accessibility and responsive clauses

33. Tab and Shift+Tab traverse the section navigation, pinned search, each
    setting name/action, and each row control in visual order with exactly one
    visible focus indicator and no trap. Enter/Space operates controls; arrow
    keys choose menu values; Escape cancels or returns as specified above.
34. Every implemented action—including explanation, reset, refusal inspection,
    and any later import/restore review—is available without a pointer. Deferred
    setup and Start-page actions cannot be used as conformance evidence.
35. Reduced motion removes tweening, sliding, smooth autoscroll, and menu/panel
    transitions only. It removes no state, cue, affordance, or information;
    focus never moves automatically, the list never reorders while read, and no
    element flashes for attention.
36. Pinned, refused, stale, verified, protected, blocked, effective, selected,
    and managed states use text plus a non-color cue. Their meaning survives the
    color-removed rendering.
37. Every control exposes programmatic name, role, value, state, provenance, and
    availability. A visible lock/refusal/stale cue is also present in the
    accessible name rather than only adjacent text.
38. Search counts, value changes, managed state, explanation opening, and
    provider state use polite non-focus-stealing announcements. A failed user
    action/refusal is assertive. Focus never moves without a user action.
39. Reflow preserves search, control, provenance, selection, and explanation
    truth. It may collapse the section list and stack explanation as drawn but
    cannot hide or replace information.

## 10. Exclusions and conformance proof

The contract rejects a search field spanning the full window, section-only
filtering, label-only matching, undocumented fuzzy guessing, result cards that
omit controls or provenance, a separate search-edit authority, tooltip-only
state, winner-only explanation, GUI-reconstructed provenance, stale explanation,
modal setup/onboarding, color-only managed/refusal state, automatic focus or
scroll motion, Project-policy editing from Preferences, startup network content,
and any assistant direct-apply path.

It also rejects treating reservation as activation: an Unwired descriptor,
empty section, ordinary aggregate Units seed control, or incomplete management,
setup, or Start-page surface cannot appear merely because its identity or clay
render exists.

Conformance must prove at minimum: every production-active row is reachable by current
label, description, stable key, and each active registered alias; reserved and
Project-owned exclusion; correct grouping/highlighting/count/no-match output; real-control
parity between results and canonical rows; deterministic result-to-row selection;
stale explanation closure; complete Q4 answers including absences across
unmanaged/managed/Context/Project states; GUI/CLI/MCP semantic parity; edit,
reset, refusal, and retained-value behavior; GP-C04 store-state continuity;
keyboard-only completion with focus return; reduced-motion and narrow reflow;
color-removed and screen-reader assertions; no authoring gate; and zero Project
or design mutation from every Preferences-only action.

Any material visual change requires a Claude-owned render and complete evidence
route reconciliation before its clauses can change.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:GP-C05-CONTRACT -->
<!-- EVIDENCE:GLOBAL-PREFERENCES-COMPLETION:GP-C05-CONSUMER-READY-AMENDMENT-20260905 -->
