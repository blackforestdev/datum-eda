# GP-C05 Interaction and Visual Contract

> **Status:** owner-adopted refined Option A. The controlling visual surface is
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
- The settled setup, Start-page, and Revision carry-forward renders remain
  controlling: `guided-setup-study.html:78-96`,
  `start-page-study.html:78-82`, and
  `revision-carryforward-study.html:78-116`.
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

A stable `PreferenceKey` and every registered retired or alternate name are
searchable vocabulary. They are not merely storage or migration machinery.
Renaming a visible label or reorganizing sections shall not strand documentation,
tutorials, scripts, support guidance, or a user searching an older term.

## 4. Window, section, and row clauses

1. At ordinary widths the window has exactly two primary columns: a persistent
   section list on the left and the selected section's settings pane on the
   right.
2. Search remains pinned above the settings pane while its rows scroll. It does
   not span above the section list. Full-width placement B is rejected.
3. The section list exposes every current section, including **Manage
   preferences** and **Project Policy (Read-Only)**, and visibly marks the
   selected section.
4. Every setting row presents its current label, plain-language consequence,
   real control or read-only action, and a provenance/status line directly
   beneath it. A tooltip is not the only carrier of any required fact.
5. Writable, managed, refused, planned, unavailable, and Project-owned rows use
   the same row grammar. State changes the available action and provenance, not
   whether the row remains discoverable or explainable.
6. A Project-policy row remains inspection plus **view in Project** navigation;
   no machine-preference control may write the Project-owned fact.

## 5. Search clauses

7. Search is always visible and focusable in ordinary navigation and searches
   every row in every section, including planned rows and read-only Project
   policy rows.
8. Matching covers the current label, plain description, stable
   `PreferenceKey`, and registered retired or alternate names. Matching is
   case-insensitive and never activates a retired identity.
9. Results are grouped under their section name, show the matched substring,
   retain the row's real control/read-only action and provenance line, and
   expose a live `matched of total` count.
10. When a match comes only through a stable key or former/alternate name, the
    result states that reason and names the vocabulary that matched.
11. Zero results say plainly that no setting matches and state that search
    includes every section, including planned and read-only rows.
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
26. **Manage preferences** uses the GP-C04 rendered states unchanged: unreadable
    data is preserved rather than repaired in place; import applies nothing
    before explicit collision choices; Capability/security authority refuses
    portable sources; unknown bytes survive; migration never applies an
    unchosen replacement; and restore is previewed, confirmed, and reversible.
27. No import, reset, migration, backup, restore, sync, explanation, or search
    action touches Project policy or design data, requires a network, or blocks
    Design authoring.

## 8. Revision, setup, Start-page, and Context reconciliation

28. Unmanaged Revision visibility remains the user's Presentation choice.
    Eligible organization Pin state may keep it on only within the approved
    `AuthorityRelease`; the row and explanation disclose the Pin and retained
    user value. Either state changes projection only, never Revision truth,
    journal, Project policy, or Release gates.
29. First-Release guidance uses the approved Q10-A/S2/R1 state: ordinary
    completion/dismissal is machine-local and replayable; an eligible managed
    teaching directive may require dismissible, non-gating replay once per
    Project. Guidance remains beside the unchanged arm-then-confirm surface,
    labels first use versus replay, and never applies a setting or gates issue.
30. Initial setup remains Q5A-B+C: the real Preferences rows are the sole manual
    setup surface, zero choices are mandatory, Skip/Escape remain available,
    **Run setup again** exposes machine-local completion/dismissal/never-run
    state, and optional assistant help creates typed proposals that apply only
    after acceptance.
31. The Q11 Start page remains local, non-tabbed, and bypassable by **Last
    session** or **Empty**. It shows canonical New/Open/Import, engine-truth
    Recents, honest missing paths, and a read-only pre-creation Q5 seed rail; it
    has no news/marketing/alerts, telemetry, or startup network load.
32. Session, Context, operation input, restartable workspace state, and transient
    state retain approved Q6 separation. Context may explain applicability but
    cannot become a writable preference or authority contribution.

## 9. Accessibility and responsive clauses

33. Tab and Shift+Tab traverse the section navigation, pinned search, each
    setting name/action, and each row control in visual order with exactly one
    visible focus indicator and no trap. Enter/Space operates controls; arrow
    keys choose menu values; Escape cancels or returns as specified above.
34. Every action—including setup coach actions, explanation, reset, refusal
    inspection, import/restore review, and Start-page actions—is available
    without a pointer.
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

Conformance must prove at minimum: every registered row is reachable by current
label, description, stable key, and each registered alias; planned and read-only
coverage; correct grouping/highlighting/count/no-match output; real-control
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
