# GP-CM05E S4 — native GUI capture mechanism feasibility (consolidated P1–P5)

**Producer session:** frontend-dev-01 · **Date:** 2026-10-30 · **Status:** OBSERVATION CAPTURE,
NOT ACCEPTANCE. No `development_ready` / `candidate_ready` / `production_accepted` claim is made or
implied here. This records whether the S4 native-GUI capture *mechanism* is feasible and honest,
under the PM-scheduled "bounded diagnostic until the mechanism is ready" cadence (CLAUDE.md
verification-cadence §; GP contract §2; owner-verified per audit receipt
gp-cm05e-cadence-provenance-verification.md). The scored 48-native corpus + statistical budget
runs are the PM-scheduled milestone qualification, not this record.

Routed to **audit-agent-01** for independent review (new typed piece). Failures/refusals are
preserved as-is; nothing was retried-to-green.

## Independent review
PASS (honest feasibility/observation-capture record; no overclaim; no required changes) by
**audit-agent-01** (session 45a27fb5), verified against the evidence bundles first-hand, not the
markdown. Receipt: `…/45a27fb5-…/scratchpad/gp-cm05e-s4-mechanism-feasibility-review.md`. Three
non-blocking coordination notes folded as clarifications here: N03 refusal negatives TRACKED in
dev-agent-02's lane (PM handling); scored corpus REBUILDS datum-gui at the pinned candidate (not
stale 8afa4062); prepared-state `elapsed_ns` on the PM's GP-CM05A owner-visible packet list.

## Scope and governing sources
- Contract: `specs/GLOBAL_PREFERENCES_PRODUCTION_ACCEPTANCE_CONTRACT.md` (§7 budgets, §11 V2
  schemas). Matrix: `specs/global_preferences_production_acceptance_matrix.json`.
- S4 corpus shape (verified against `scripts/global_preferences_acceptance/measurements.py`
  `measurement_inventory()`): 64 GUI measurement rows (48 gui-feedback + 8 native-window-open + 8
  window-lifecycle) + 48 native-capture.v2 observations (N01–N06 × 8 coordinates) + env array
  (1 headless + 8 backend/scale/size) + candidate.binaries `gui` role.
- Lane claim is dev-agent-02's; frontend-dev-01 delivers S4 capture-mechanism craft under PM
  coordination. Reuses dev's assembler in `measurements_producer.py` (no parallel schema).

## Environment and binaries (honest, recorded — not assumed)
- Host: KDE Plasma, `WAYLAND_DISPLAY=wayland-0` + Xwayland `DISPLAY=:1`; a11y stack up
  (`org.a11y.Bus`, `at-spi2-registryd`, `org.freedesktop.a11y.Manager` on kwin). Tools present:
  busctl, orca, xdotool, wayland-info, weston, weston-screenshooter, `import` (ImageMagick), PIL,
  `gi.repository.Atspi`. Absent: python-xlib, mss (bears on P1 helper — see below).
- **Binary used (and SOURCE SKEW, recorded honestly, not assumed unchanged):**
  `target/release/datum-gui` sha256 `47a9082f…` built from source **8afa4062** (Oct 29). The
  `datum-eda` CLI (`c889c5b5…`) and `eda-engine-daemon` (`22df1c9f…`) are Oct-30 builds at a newer
  source (dev's S3 build). **The GUI binary drives every workflow and commit; the newer CLI is used
  only as a READ cross-check** (`preferences explain/get`) against the GUI-written store — GUI
  AT-SPI + store are primary. "Unchanged product inputs since 8afa4062" is a claim to VERIFY via
  the input manifest at milestone pinning, not assumed here.
- Pre-existing competing load recorded (PM ruling, non-blocking): four stale debug
  `eda-engine-daemon` pids 3203866/3212801/3233384/3237135 (idle; cleanup blocked in the producer
  session by the auto-mode classifier — owner/PM to resolve).

## Findings

### P1 — presented-frame acknowledgement (presented_elapsed_ns) — FEASIBLE (coarse) + named gap
- The product exposes **no ordinary true presented-frame ack**: all presentation timing is behind
  `DATUM_TRACE_TIMING`/`DATUM_GUI_VERBOSE_LOG` (stripped in ordinary runs), and the loggable value
  is *submission*, explicitly "not GPU completion, compositor sampling, or visible-output
  acceptance" (`crates/gui-app/src/rendering/runtime_present.rs` ~L208). `wp_presentation` feedback
  is client-delivered — an external harness cannot receive another client's timestamps. KWin
  `/Compositor` dbus exposes no timing methods; KWin QML scripting is geometry/lifecycle-scoped.
- **External pixel-change watch (X11) WORKS** (PM-proposed ordinary alternative): keyboard input →
  detected window-pixel change at **145 ms upper bound**, capture cadence **~93 ms/frame** via
  `import -window` + PIL hash (evidence: `…/probe-p1c/report.json`).
- The validator puts **no p95 on `presented_elapsed_ns`** — only `>= elapsed` and `<= max_ms`
  (stalled-loop guard). So native-window-open (max 1000 ms) is **provable** by the coarse watch;
  gui-feedback (max 100 ms) is **not** at ~93 ms cadence.
- **PM-endorsed split treatment:** pixel-change watch as presented-ack for native-window-open; a
  **first-party XDamage/XShm capture helper** (X11, ctypes against system libXdamage/libX11 —
  **PM029: no new dependency, no pip**; capture-tooling, not a product edit) prototyped at the
  milestone for gui-feedback; **declared honest incompleteness** as the fallback. Nested compositor
  never (it would make the measured binary non-ordinary). If system libs are absent that is an env
  limitation to record, not a dependency to acquire.
- **Separate named finding (own ruling, possibly owner-visible at GP-CM05A):** prepared-state
  `elapsed_ns` — the p95-bearing gui-feedback metric — has **no ordinary EXTERNAL observable** (an
  external watcher sees visible output, not internal "prepared state"). Distinct from the
  presented-ack question.

### P2 — AT-SPI exposure of the Global Preferences surface — PASS (both backends)
- `datum-gui` registers on the live AT-SPI bus under ordinary conditions (no screen-reader forced):
  `[application] "Datum EDA"`, exposing a genuine tree (Wayland: terminal + Navigator + Inspector
  with real `[entry]`/`[button]` name-role-value-state; evidence `…/probe-run-1/report.json`).
- The Global Preferences dialog (X11) is a distinct top-level `[dialog] "Global Preferences"` with
  real controls: `[entry] "Search Appearance settings"`, `[combo box] "Console feedback duration"`,
  `[toggle button] "Reduced motion"`, `[toggle button] "High contrast and non-color cues"`, focused
  `[panel] "Appearance"` (evidence `…/probe-x11-dialog/report.json` + `dialog.png`). No AccessKit;
  Datum uses a **first-party AT-SPI platform** (PM029-consistent).

### Item (2) — AT-SPI events + screen-reader spoken output — PASS (last AT-SPI risk closed)
- 178 AT-SPI events fired during search-type + Tab + Space (evidence `…/probe-item2/report.json`):
  `object:property-change:accessible-name` (123) / `:accessible-description` (40) [N06
  name-role-value-state]; `object:text-changed:insert/delete` on the search field [N02]; and
  **`object:announcement` events carrying REAL live-region text** from src "Datum EDA" — "11
  preference search results." / "1 …" / "0 …" as the search filtered [N06 polite-assertive-updates].
- **Orca screen-reader transcript capture is PROVEN as a mechanism** (`orca --debug-file` →
  `SPEECH OUTPUT` lines). In this bounded run Orca echoed keystrokes (`'tab'`/`'space'`,
  `established:False`) rather than reading dialog content — **Orca focus-establishment tuning is an
  OPEN named item** carried into the milestone capture plan: scored N06 runs attempt the tuned Orca
  transcript FIRST, fall back to app-announcement events + a declared limitation ONLY if tuning
  genuinely fails. App announcement events are the deterministic **app-side** evidence, not a
  substitute for the screen-reader claim (keeps audit's no-substitution bar met).

### Item (4) — responsive render at scale/size — PASS
- At narrow **720×760 + scale 2.0** (X11) the dialog exposes the identical full control set (no
  clipping, all controls reachable), matching 1024×900/scale1 (evidence
  `…/probe-item4-narrow/report.json` + `dialog.png`) [N05 responsive-render / narrow].

### Item (5) — four ordinary workflows end-to-end via GENUINE OS input — PASS
Harness: governed **StartupQA owned-Weston X11-input envelope** (MM-C06/crossprobe pattern; datum-gui
native Wayland/Vulkan under it). The nested Weston is used **only** for genuine-input workflow
observation — **never** for any P1 presentation-timing claim. Evidence: `…/probe-item5/result.json`
+ weston captures (`5a-open-read`, `5b-set`, `5c-persist`, `5d-reset`, menu-reachable frames).
Key: `datum.accessibility.reduced_motion`; store isolated at `<root>/config/datum/preferences`.
- **5a OPEN+READ** — dialog opened via GENUINE menu activation (`StartupQA.menu_item` clicks the
  Edit title @ (147,17), keyboard-navigates Preferences → Right → "Global Preferences…" → Return —
  NOT the `--open-global-preferences` affordance). Read sections/controls/defaults from AT-SPI.
- **5b SET** — genuine Space on the focused toggle committed **generation 0** via the real journal:
  receipt `operation=CommitPreferenceMutation`, `actor_kind=human_gui`
  (`human_gui:bfadmin:datum-gui-24338`), `affected_keys=[reduced_motion]`, `before_digests={}` →
  `after_digests{user:…reduced_motion}`, `reason="Global Preferences control activation"`. `explain`:
  `effective_value=true`, "effective from user:…", descriptor-default(false) "loses to higher-ranked
  User contribution". ONE User-partition change, committed + resolver-explained + receipted.
- **5c PERSIST** — after a full GUI **process restart**, `explain` still `effective_value=true` from
  the user contribution: persisted across processes as the immutable generation.
- **5d RESET** — the real "Reset Reduced motion" control (genuine input) committed **generation 1**
  (parent 0); `explain`: `effective_value=false`, "effective from descriptor-default",
  `absent_sources` include "user". **ResetUser removed the User contribution only, wrote no
  descriptor default, touched no Project** — exact contract semantics. Store holds g0+g1 (append-only).
- These map line-for-line onto the frontier item's `ordinary_workflow` enumeration that GP-CM05A is
  walked against.

### N04 — New Project form — PASS (AT-SPI feasible)
- `--open-new-project` shows `[dialog] "New Project"` fully AT-SPI-exposed (Project/Folder/Location
  `[entry]`s, `[check box] "Use project name"`, `[button]` Create/Cancel/Browse/Up/Home, file
  `[list]` + `[list item]`s; evidence `…/probe-item3-np/report.json` + `newproject.png`). The form
  is deliberately units-free (`StartupQA.assert_new_has_no_units`); genesis **modes** are a function
  of global state, observable via the genesis receipt (consistent with item (5)'s project evidence).

### OPEN-3 — window-lifecycle resource observables — CLOSED
- **owned_windows**: countable via AT-SPI top-level `[dialog]`/`[frame]` children of the app (present
  when open, gone when closed).
- **writer leases / orphan_writer_leases**: `writer.lock` in the preferences store is the writer-lease
  observable; content (15 bytes) = `"datum-gui-24468"` (owner = writer-instance id). It PERSISTED
  after the SIGTERM'd process exited → **orphan-lease detection is observable** (orphan = a
  `writer.lock` whose owner instance is no longer alive). Note: this is the preferences-store write
  lease; a native Project adds its own `ProjectWriteLease` (out of GP-dialog scope).

## Recorded capture-craft rules (for the milestone capture tooling)
1. **AT-SPI reference lifetime:** Datum is immediate-mode and rebuilds its accessible tree ~per
   frame, so a held node reference goes stale ("unknown accessible object"). **Resolve + read
   (role/name/state/extents/events) INLINE in one descent; never hold a node reference.** Geometry
   (`Component.get_extents`) works only when read inline.
2. **Value source:** the AT-SPI CHECKED/toggle *state* read is unreliable under immediate-mode
   (stayed false while the store/resolver showed user=true). **Read preference VALUE from the
   resolver/store (`preferences explain/get`); AT-SPI is authoritative for tree/role/name/events,
   the store/resolver for VALUE.**
3. **Input:** the top menu bar is NOT in AT-SPI — genuine top-menu activation needs a coordinate
   click on the title (then keyboard submenu nav). Keyboard input is more robust/backend-uniform
   than coordinate clicking for in-dialog controls.
4. **Affordance discipline:** `--open-global-preferences` / `--open-new-project` are capture/test
   affordances (seed the real production forms open) — fine for AT-SPI/pixel feasibility; anything
   SCORED (native-window-open timing) must use GENUINE menu activation.

## Open / named-remainder items carried to the milestone capture plan
- **P1 gui-feedback presented_elapsed_ns**: prototype first-party XDamage/XShm helper (ctypes, no
  dep); fall back to declared incompleteness. **prepared-state elapsed_ns**: no external observable —
  needs its own ruling, possibly owner-visible at GP-CM05A.
- **N06 Orca**: focus-establishment tuning (tuned transcript first, announcements+limitation fallback).
- **N03 set-reset refusal** (invalid/stale-writer/agent, draft retention): engine-boundary negative;
  human_gui set/reset is PROVEN (item 5), but the refusal negatives are best staged in the engine
  corpus (dev-agent-02's lane) — GUI-surface draft/refusal capture is a named milestone item, not a
  categorical infeasibility. **Must be TRACKED in dev-agent-02's lane (per audit note) so
  GUI-surface refusals do not fall between lanes.**
- **Scored window-open measurement** must use genuine menu activation + the pixel-watch presented-ack.
- **Candidate rebuild (per audit note):** the scored 48-native corpus must REBUILD `datum-gui` at the
  PINNED candidate revision — it must not measure the stale 8afa4062 binary used for this bounded
  mechanism proof. The input manifest verifies the unchanged-inputs claim at that pinning.

## Bottom line
The S4 native-GUI capture **mechanism is feasible and honest**: AT-SPI tree + events + app
announcements + Orca transcript, all four ordinary workflows through the real journal (commit →
receipt → resolver → cross-process persistence → ResetUser), responsive at scale/size, with
owned-window and writer-lease observables. The one genuine measurement gap is gui-feedback
`presented_elapsed_ns`/`elapsed_ns` (P1), carried as a named item with an honest, no-fabrication
resolution path. No fabrication; nested compositor never used for timing; failures preserved.
