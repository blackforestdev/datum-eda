# GP-CM05E S4 — P1 presented-frame-ack capture helper (follow-up)

**Producer session:** frontend-dev-01 · **Date:** 2026-10-30 · **Status:** OBSERVATION CAPTURE /
capture-tool development, NOT ACCEPTANCE. Follow-up to `gp-cm05e-s4-mechanism-feasibility.md`
(af0af52f), resolving that record's one named measurement gap: the gui-feedback presented-frame
acknowledgement (`presented_elapsed_ns`) at 100 ms-bound precision. Bounded diagnostic under the
PM-scheduled cadence; no scored samples, no readiness/budget claim. Routed to **audit-agent-01**.

PM029 / first-party: every helper binds SYSTEM libraries via `ctypes` only (libX11, libXdamage,
libxcb, libxcb-present). Nothing is fetched, installed or vendored. Where a header is absent the
protocol ABI is reconstructed from the published spec. A missing library/extension is recorded as an
environment limitation, never acquired. Helpers live under `scripts/global_preferences_acceptance/`.

## Problem
`wp_presentation` feedback is client-delivered (an external harness cannot receive another client's
timestamps), KWin exposes no external per-frame timing, and the product's internal timing is a
diagnostic-gated *submission* value explicitly "not compositor sampling or visible-output
acceptance." The S4 record carried presented-ack for gui-feedback as a named gap with a
no-fabrication resolution path. This follow-up walks that path to ground.

## Three approaches, measured

### 1. XDamage (core X DAMAGE) — RULED OUT for this GUI (keepable finding)
Helper: `scripts/global_preferences_acceptance/xdamage_present_watch.py` (libX11 + libXdamage).
- **Negative control PASS:** against `xclock -update 1` (core-X rendering) the helper captured 6
  `DamageNotify` events at precise ~1 s intervals (40 ms, 1041, 2040, …), ms-grade — the binding is
  sound.
- **datum-gui: ZERO `DamageNotify`** despite a real visible change. datum-gui renders via
  Vulkan/WGPU and presents through **DRI3/Present (GPU buffer flip), which bypasses core X DAMAGE**.
  So XDamage cannot observe this GUI. Do not re-walk this path. Evidence: `probe-xdamage2/report.json`
  + the xclock control.

### 2. XGetImage front-buffer poll — VALIDATED fallback
Helper: `scripts/global_preferences_acceptance/present_image_poll.py` (libX11 `XGetImage`).
Reads the Present-updated front buffer directly, no per-sample process spawn (unlike `import`).
- **Clean-floor cadence: median ~6.8 ms / min ~5.7 ms** per full-window grab (vs the ~93 ms
  `import -window` baseline). (An earlier 21–25 ms run was CPU-contended by a concurrent authorized
  cargo batch — a floor-check miss, owned and corrected with a hard-stop rule; the clean numbers
  stand.)
- **Disambiguation:** Tab (4/4) and Space (2/2) changes DETECTED at **median 27.4 ms** input→change;
  search-type letters (3/3) undetected = a focus artifact (the dialog opens focused on the Appearance
  panel, not the search entry; lone letters drove no visible change), NOT XGetImage returning a
  stale buffer (it sees Present-flipped front-buffer changes fine). Evidence:
  `probe-xgetimage-clean/`, `probe-xgetimage-disambig/`.
- Verdict: a viable external presented-pixel signal at ~27 ms detection, well under the 100 ms bound.
  **Scored note:** the search-update action must focus the search field first so the result change
  is driven and detectable; the poll resolves change no finer than its cadence (~6 ms here).

### 3. X Present `PresentCompleteNotify` (via xcb) — THE IDEAL PATH, PROVEN WORKING
Helper: `scripts/global_preferences_acceptance/xcb_present_watch.py` (libxcb + libxcb-present;
`libXpresent` is absent so the Present extension is reached through xcb; `xcb/present.h` absent, ABI
reconstructed from the protocol spec).
- Present extension present (major opcode 145). **`idle_presents_in_1s = 0`** — datum-gui presents
  ON-DEMAND, so a present after input is a clean signal with no idle flood.
- input(Tab)→present detected 6/6: **recv latency median 32.8 ms / min 17.2 ms**. The event UST
  appears to be CLOCK_MONOTONIC microseconds: on all 6 samples `ust_minus_t0` tracks `recv_latency`
  within <1 ms with UST consistently slightly EARLIER than receipt (the flip precedes our receipt —
  physically correct), which is strong evidence for the monotonic domain. (The code's coarse
  `all(-50<=u<=500)` gate alone only rules out REALTIME; the tight correctly-signed tracking in the
  data is what supports the domain claim — per independent review.)
- Verdict: `PresentCompleteNotify` is a **true external per-present completion timestamp** (the X
  server/compositor attests the flip), ms-grade, ordinary, no product edit. At FEASIBILITY level it
  **retires the gui-feedback presented-ack gap** — even the clock-domain-INDEPENDENT recv latency
  (~32.8 ms median) is already a valid <100 ms external signal. The SCORED milestone must RE-VERIFY
  UST↔CLOCK_MONOTONIC alignment on the pinned candidate host (do not inherit it as settled), and fall
  back to the local monotonic receipt time or the XGetImage poll if it ever differs.
  Evidence: `probe-xcbpresent/report.json`.
- `msc` note (non-blocking, unused): the returned `msc` reads as k·2³²+2 (high word increments, low
  word fixed) — atypical for a vblank counter, likely a word-order/field-offset quirk in the
  reconstructed ABI. No claim depends on `msc` (only `ust` and recv time are used); verify the `msc`
  parse before ever relying on it.

## Outcome for the P1 line
The S4 record's P1 line flips from "prototype at milestone" to **"helper proven; apply at
milestone."**

**Milestone method of record (PM-endorsed 2026-10-30)** for `presented_elapsed_ns`: X Present
`PresentCompleteNotify` (ideal true per-present timestamp) as primary, with the XGetImage poll as a
validated fallback; the pixel-watch covers native-window-open comfortably too. The scored milestone
run must additionally: (a) RE-VERIFY UST↔CLOCK_MONOTONIC alignment on the pinned candidate host
before using ust-based latency (else recv/poll fallback); (b) RETAIN the XDamage xclock negative
control as a kept evidence artifact alongside the datum-gui zero-run (this feasibility pass recorded
the control numbers in prose/stdout and independent review reproduced them live — 3 `DamageNotify`
@ ~1 s — but the scored record needs its own retained control report, not transient numbers).
XDamage is ruled out for this GPU-present GUI (DRI3/Present bypasses core DAMAGE).

**Separate, still-open:** prepared-state `elapsed_ns` (the p95-bearing gui-feedback metric) has no
ordinary EXTERNAL observable — presented-ack being solved does not address it; it remains on the
GP-CM05A owner-visible packet for its own ruling.

## Independent review
PASS as a feasibility/capture-tooling record (PM029-clean) by **audit-agent-01** (session 45a27fb5),
which reviewed the artifact + helper source + evidence and independently REPRODUCED the XDamage
xclock control through this helper's own `PresentWatch` binding (3 `DamageNotify` @ ~1 s), confirming
the datum-gui zero is a real DRI3/Present bypass. Receipt:
`…/45a27fb5-…/scratchpad/gp-cm05e-s4-p1-helper-review.md`. Three non-blocking items folded above:
"retires outright" tightened to feasibility-level + milestone UST re-verify; xclock control to be a
retained milestone artifact; `msc` marked unverified-anomalous/unused.

## Scope / discipline
Capture tooling inside the granted acceptance-scripts scope; PM-coordinated within GP-CM05E; additive
review evidence, not a product/scripts claim under the lane's product scope. No new dependency; no
nested compositor used for any timing; all failures/contaminated samples preserved and labeled; no
retry-to-green. Raw probe logs retained once under `.datum/scratch/frontend-dev-01/` (probe-xdamage2,
probe-xgetimage-clean, probe-xgetimage-disambig, probe-xcbpresent), referenced not committed.
