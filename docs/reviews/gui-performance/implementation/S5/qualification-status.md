# S5 completed tests and remaining acceptance

This checklist records existing validated evidence from `implementation-map.json`.
**An agent handoff does not reset a completed test.** Done applies to the exact
test scope and candidate in the linked evidence; it does not turn a partial
requirement into full S5 acceptance. Ignored tests are not passes. Test-group
counts overlap and must not be summed. No tests were rerun to create this list.

## Done — carry forward

- [x] **Console regression tests** — 9 passed; 0 failed. [Evidence](console-reconciliation/result.json)
  Scope: Console behavior on recorded candidate.
- [x] **Console static goldens** — 10 scale cases across seven manifests; exact zero-pixel differences. [Evidence](console-reconciliation/result.json)
  Scope: Named Console corpus; broader visual rows remain separate.
- [x] **Canonical native shell screenshot** — 0.320% differing pixels and0.2914 mean channel delta; both within existing limits. [Evidence](native-parity/result.json)
  Scope: Recorded1680x1050 fixture and backend; not every native configuration.
- [x] **Reference and custom-icon gates** — Board-editor reference accepted;11custom icon checks pass. [Evidence](native-parity/result.json)
  Scope: Recorded static references, not full product parity.
- [x] **Renderer unit regression suite** — 353passed;89ignored were not executed. [Evidence](resource-snapshots/result.json)
  Scope: Completed serial visual-feature suite; ignored tests not counted as passed.
- [x] **Application unit regression suite** — 400passed;17ignored;0failed. [Evidence](native-owner-focus/verification.json)
  Scope: Completed application suite on focus-fix candidate; ignored tests not counted.
- [x] **Serial GPU regression group** — 33serial GPU tests pass. [Evidence](resource-snapshots/result.json)
  Scope: Named existing group; this does not erase separately tracked concurrent-GPU failure.
- [x] **Missing-final-resolve negative and restored positive** — Injected missing resolve fails pixel oracle; restored candidate passes1/1. [Evidence](final-msaa-resolve/result.json)
  Scope: Final-stage ordering/resolve sensitivity, unchanged8xquality; not absoluteGPU budget.
- [x] **Native menu output regression** — Six normal runs;60menu cycles; exact baseline pixel matches. [Evidence](final-msaa-resolve/result.json)
  Scope: Bounded native menu replay, not complete W-WINDOWS or performance qualification.
- [x] **X11 auxiliary focus positives and negatives** — All3missing-owner-hint negatives fail focus; all3production positives restore focus;6exact captures and normal exits. [Evidence](native-owner-focus/result.json)
  Scope: Global/Project/New X11/1x owner-focus behavior.
- [x] **Wayland native-entry close regression** — 1test passes;12host/profile close cases. [Evidence](native-owner-focus/verification.json)
  Scope: Direct native-handler invocation, not OS delivery; constrained2xheight limitation retained.
- [x] **Global Preferences30-cycle controls sequence** — 30cycles;510actions;540image checks;90drag/cancel motion receipts. [Evidence](controls-30-cycle-batch/result.json)
  Scope: Actual X11/1x host functional sequence; numerical/full configuration/negative matrix still separate.
- [x] **Project Preferences30-cycle controls sequence** — 30cycles;510actions;540image checks;90drag/cancel motion receipts. [Evidence](controls-30-cycle-batch/result.json)
  Scope: Actual X11/1x host functional sequence; completed Project sequence was not rerun for supplemental clip probe.
- [x] **Hidden-control/Search overlap probes** — 30actual overlap probes per host preserve fullclient pixels and do not activate hidden control. [Evidence](controls-30-cycle-batch/result.json)
  Scope: 60supplemental native probes; original inert-header-only evidence retained separately.
- [x] **Native pane transition sequence** — 60actions/15cycles over30seconds;PaneId/camera retention;exact Board interior;normal drain. [Evidence](pane-native-sequence/review.json)
  Scope: One X11/1x functional sequence; not three-trial/all-scale performance.
- [x] **Pane tracked-allocation lifetime reconciliation** — 606registrations/606releases;25040events;341snapshots;final tracked capacity0. [Evidence](pane-resource-sequence/review.json)
  Scope: Tracked application allocations only; not complete driver/private/API-instant peaks or endurance.
- [x] **Private-call observer controls** — 2explicit serial observer tests pass, including transient peaks,overlap,overflow and abandoned calls. [Evidence](private-call-observation/result.json)
  Scope: Observer conformance, not full private-memory qualification.
- [x] **Private-call native delivery and refusal controls** — 6373calls/12746events reconcile;native overflow/existing-file controls also retained. [Evidence](private-call-observation/result.json)
  Scope: Recorded9cycle run;66historical null measurement origins retained, addressed by separate origin test.
- [x] **GPU allocation observer controls** — 2explicit serial observer tests pass. [Evidence](gpu-allocation-observation/result.json)
  Scope: Identity,release,shared-reservation peak and overflow delivery conformance.
- [x] **GPU allocation native delivery** — 6961events;184allocations/releases;no unattributed origin;malformed-history controls rejected. [Evidence](gpu-allocation-observation/result.json)
  Scope: Tracked-event delivery, not complete GPU residency or numerical caps.
- [x] **Four-host measurement-origin/refusal/retry test** — 1explicit GPU test passes all4actual adapters;all measurement calls attributed;zero dropped events. [Evidence](measurement-origin/result.json)
  Scope: Direct adapter proof; original native null-origin evidence remains historical.
- [x] **Resource observer replacement/retirement conformance** — RealGPU observer test passes shared budget IDs,retirement and metadata-only ownership. [Evidence](resource-snapshots/result.json)
  Scope: Observer semantics; full instantaneous accounting remains outstanding.
- [x] **Endurance harness method and cleanup review** — Independent method findings corrected and verified. [Evidence](independent-replay/method-review.json)
  Scope: Method review only. Both incomplete native attempts remain failed; no endurance pass.
- [x] **Fractional pointer offline controls** — Schedule plus6simulated portal cases pass; independent source review verified corrections. [Evidence](independent-replay/pointer-method-preparation.json)
  Scope: No native pointer stream executed; no delivered-input or performance pass.

## Still open — do not reset the done list

- [ ] Resolve recorded pointer CPU/GPU, clamp CPU and warm Project-open budget failures; verify affected corrections.
- [ ] Complete missing native backend/scale, input, semantic-output and required negative cases; reuse already completed host sequences.
- [ ] Complete fixture admission and instantaneous private/scratch/driver/client resource accounting, including observer overhead.
- [ ] Complete required60-minute endurance,200window cycles and20recoveries. Both archived incomplete attempts remain failures.
- [ ] Complete independent actual replay against the stable final candidate, followed by separate owner UX/product acceptance.
- [ ] Resolve applicable recorded drift failures; preserve separately tracked concurrency and golden failures.

S0–S4 implementation exits remain complete. Their carried qualification is part
of the outstanding original predicates, not a reason to repeat implementation.
Pinned resize/temporal,T2 and unadmitted schematic exclusions remain unchanged.

## Current change awaiting focused verification

Lazy world-pipeline construction is an uncommitted production change. Its
initialization and affected pixel checks are pending. Existing unrelated proof
stays done; neither that patch nor this checklist claims S5 closure.
