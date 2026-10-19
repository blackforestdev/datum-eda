# Specification Progress Tracker

> **Purpose**: Track requirement implementation status; update with code changes.
> Keep checkpoint summaries here and detail/history in linked `specs/progress/` shards; only the generated Frontier selects work.
> Legend: `[x]` done, `[~]` partial, `[ ]` not started, `[—]` deferred/N/A

---

## Active Frontier — Next Development Steps (canonical)

> Canonical development order comes from the generated Frontier below; details remain linked under each checkpoint (CLAUDE.md → Roadmap Wayfinding).
> Preserve dependency/governing links and update the projection with every specification or course correction.

<!-- ACTIVE FRONTIER:START -->
> Generated from `specs/active_frontier.json` by `scripts/project_status.py`; do not hand-edit.

<details>
<summary>Completed history (21 landed records)</summary>

- **Align project-state authority and tooling** (`STATE-ALIGNMENT`; `dat-project-management-alignment-pua`).
   Install the structured roadmap, deterministic next-task selector, cross-ledger checks, and reconciled tracker state without weakening source-health governance. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* one reproducible roadmap answer for every agent. *Governing:* `docs/decisions/PRODUCT_MECHANICS_025_PROJECT_STATE_AUTHORITY.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md`.
- **Complete the S5 selection contract and identity decision** (`UVT-S5-SPEC`; `dat-s5-selection-visual-contract-zid`).
   Complete the exhaustive stable-ID S5 closure contract: identity matrix, bounded queries, lifetime, compound outputs, authority/atomicity/cross-probe/overlay reconciliation, evidence, owner review, decision, and synchronized governance; S5A execution remains unauthorized. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* dat-uvt-s5a-build-1wv. *Governing:* `docs/decisions/PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING.md`, `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`, `docs/gui/DATUM_SELECTION_VISUAL_LANGUAGE_GUIDANCE.md`, `docs/gui/DATUM_SELECTION_COMPOUND_EDITING_GUIDANCE.md`, `docs/gui/DATUM_RENDERING_BOOK.md`, `docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md`, `docs/gui/DATUM_GUI_PHASE_2_SPEC.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/decisions/PRODUCT_MECHANICS_026_SELECTION_IDENTITY.md`.
- **Course-correct the full native terminal product contract** (`FULL-NATIVE-TERMINAL-CONTRACT`; `dat-full-native-terminal-contract-e2d`).
   Replace the terminal-lite plan with a governed fully fledged native terminal product, Datum-owned terminal semantics, complete daily-driver capability matrix, and production agent/CLI/MCP proof. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md`.
- **Deliver the fully fledged Datum-native terminal** (`TERMINAL-EPIC`; `dat-terminal-emulator-epic-jzv`).
   Deliver T0-T4e: shell-screen truth, Datum-owned core and PTY, native rendering/input, daily-driver UX/protocols, native agent discovery/MCP/context/workflows, and final production proof. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* trustworthy daily-driver embedded terminal and agent surface. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_028_TERMINAL_AGENT_INTEROPERABILITY.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`.
- **Ratify terminal agent discovery and interoperability** (`TERMINAL-AGENT-INTEROP-CONTRACT`; `dat-terminal-emulator-epic-jzv.7`).
   Replace prompt-memory discovery with client-native adapters, standard MCP, pinned context, scoped authority, portable workflows, and production interoperability proof before terminal execution continues. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_028_TERMINAL_AGENT_INTEROPERABILITY.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`.
- **Restore foreign-shell screen authority and visible-output proof** (`TERMINAL-T0-SHELL-TRUTH`; `dat-pan-trace-terminal-pollution-0j0`).
   Make PTY-derived state the sole terminal-cell authority and prove real typed shell output through the production display path. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-full-native-terminal-contract-e2d`, `dat-terminal-emulator-epic-jzv.7`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`.
- **Complete terminal focus and input authority** (`TERMINAL-P0-FOCUS`; `dat-terminal-focus-authority-6aw`).
   Preserve TF-01 as focus-owner extraction only, then complete content focus, focus reports, cursor affordance, key timing, and leak tests without claiming the whole terminal repaired. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-pan-trace-terminal-pollution-0j0`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/decisions/PRODUCT_MECHANICS_024_NATIVE_TERMINAL_EMULATOR.md`, `docs/decisions/PRODUCT_MECHANICS_021_WORKSPACE_PANE_TILING.md`.
- **Collapse the rival terminal input models** (`TERMINAL-T1-INPUT`; `dat-terminal-dual-input-model-75y`).
   Keep raw PTY/TerminalCore input as the sole attached model and make detached state explicit. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-pan-trace-terminal-pollution-0j0`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`.
- **Harden the Datum-owned PTY transport** (`TERMINAL-T1-PTY`; `dat-terminal-emulator-epic-jzv.1`).
   Implement and prove the bounded Datum-owned Linux PTY/session transport through DTC-P03..P06E with no third-party terminal dependency. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-pan-trace-terminal-pollution-0j0`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`, `docs/decisions/PRODUCT_MECHANICS_030_DATUM_TERMINAL_CORE_ARCHITECTURE.md`, `docs/decisions/PRODUCT_MECHANICS_031_TERMINAL_TYPOGRAPHY_EXCEPTION.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_CORE_RESEARCH.md`, `docs/gui/DATUM_TERMINAL_CORE_IMPLEMENTATION_PLAN.md`.
- **Build the Datum-owned TerminalCore** (`TERMINAL-T1-CORE`; `dat-terminal-emulator-epic-jzv.2`).
   Implement Datum-owned VT parsing, state, cells, modes, replies, damage, history, reflow, selection, search, and protocols without third-party terminal code. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-pan-trace-terminal-pollution-0j0`, `dat-terminal-emulator-epic-jzv.1`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`, `docs/decisions/PRODUCT_MECHANICS_030_DATUM_TERMINAL_CORE_ARCHITECTURE.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_CORE_RESEARCH.md`, `docs/gui/DATUM_TERMINAL_CORE_IMPLEMENTATION_PLAN.md`.
- **Complete native renderer, focus, input, IME and text interaction** (`TERMINAL-T2-NATIVE`; `dat-terminal-emulator-epic-jzv.3`).
   Replace the bespoke screen with TerminalCore and complete lossless rendering, Unicode/fonts, focus/input/IME/mouse, selection/clipboard, damage, and accessibility foundation. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-pan-trace-terminal-pollution-0j0`, `dat-terminal-dual-input-model-75y`, `dat-terminal-emulator-epic-jzv.1`, `dat-terminal-emulator-epic-jzv.2`, `dat-terminal-focus-authority-6aw`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`, `docs/decisions/PRODUCT_MECHANICS_030_DATUM_TERMINAL_CORE_ARCHITECTURE.md`, `docs/decisions/PRODUCT_MECHANICS_032_DATUM_OWNED_AT_SPI_BRIDGE.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_CORE_RESEARCH.md`, `docs/gui/DATUM_TERMINAL_CORE_IMPLEMENTATION_PLAN.md`.
- **Deliver scrollback, reflow, search, links and daily-driver session UX** (`TERMINAL-T3-UX`; `dat-terminal-emulator-epic-jzv.4`).
   Deliver complete scrollback/reflow/search/link behavior plus tabs, splits, profiles, themes, fonts, terminate-only lifecycle, maximize, shell, and terminfo integration without detached Datum PTYs. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-terminal-emulator-epic-jzv.3`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_CORE_IMPLEMENTATION_PLAN.md`.
- **Complete modern protocols, graphics, security and accessibility** (`TERMINAL-T3-PROTOCOLS`; `dat-terminal-emulator-epic-jzv.5`).
   Close synchronized output, OSC 8/52, kitty keyboard, kitty graphics/sixel, security policy, and complete accessibility behavior. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-terminal-emulator-epic-jzv.4`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_CORE_IMPLEMENTATION_PLAN.md`.
- **Build agent launcher and discovery adapters** (`TERMINAL-T4A-AGENT-LAUNCH`; `dat-terminal-emulator-epic-jzv.8`).
   Implement native discovery adapters and datum-eda agent list/doctor/launch with protected per-session configuration for Codex, Claude Code, Cursor-compatible CLI, and generic local agents. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-terminal-emulator-epic-jzv.3`, `dat-terminal-emulator-epic-jzv.7`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_028_TERMINAL_AGENT_INTEROPERABILITY.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`.
- **Provide the standard MCP broker, resources, and prompts** (`TERMINAL-T4B-MCP`; `dat-terminal-emulator-epic-jzv.9`).
   Expose Datum through a protocol-clean MCP stdio broker, optional secured loopback transport, typed tools, stable resources, templates, notifications, and portable prompts. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-terminal-emulator-epic-jzv.3`, `dat-terminal-emulator-epic-jzv.7`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_028_TERMINAL_AGENT_INTEROPERABILITY.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`.
- **Enforce pinned context and scoped agent authority** (`TERMINAL-T4C-CONTEXT-AUTHORITY`; `dat-terminal-emulator-epic-jzv.10`).
   Separate live and immutable pinned context, fence mutations by revision, and grant short-lived inspect/propose/apply-approved authority with complete audit provenance. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-terminal-emulator-epic-jzv.8`, `dat-terminal-emulator-epic-jzv.9`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_028_TERMINAL_AGENT_INTEROPERABILITY.md`, `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`.
- **Project portable workflows and verify agent parity** (`TERMINAL-T4D-WORKFLOW-PARITY`; `dat-terminal-emulator-epic-jzv.11`).
   Keep workflows canonical in CLI/MCP, parity-check thin client projections, enforce the OSC metadata boundary, and prove the named agent round trip. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-terminal-emulator-epic-jzv.5`, `dat-terminal-emulator-epic-jzv.10`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_028_TERMINAL_AGENT_INTEROPERABILITY.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`.
- **Verify full terminal and agent interoperability** (`TERMINAL-T4-VERIFY`; `dat-terminal-emulator-epic-jzv.6`).
   Close only after T4a-T4d and quantified conformance/performance/security/accessibility evidence prove real native discovery and end-to-end Codex, Claude, Cursor/local-agent, CLI/MCP, shell, and TUI operation. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-terminal-emulator-epic-jzv.5`, `dat-terminal-emulator-epic-jzv.11`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_028_TERMINAL_AGENT_INTEROPERABILITY.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/gui/DATUM_TERMINAL_AGENT_INTEROP_SPEC.md`.
- **Research and ratify the Console and action-feedback contract** (`CONSOLE-CONTRACT-RECOVERY`; `dat-output-lane-t6v`).
   Remove legacy Console conflation, research the actual feedback and optional typed-command needs, establish an owner-reviewed visual contract, and ratify the implementation boundary before any visible surface is built. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_033_OUTPUT_ONLY_DATUM_CONSOLE.md`, `docs/gui/DATUM_GUI_DESIGN_SPEC.md`, `docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md`.
- **Build the output-only Datum Console** (`DATUM-CONSOLE-OUTPUT`; `dat-datum-console-output-djd`).
   Replace the legacy untyped narration sink with the typed, bounded, output-only Datum Console and render the owner-selected focused-pane feedback overlay without creating a new command, notification, diagnostic, or mutation authority. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_033_OUTPUT_ONLY_DATUM_CONSOLE.md`, `docs/gui/DATUM_GUI_DESIGN_SPEC.md`, `docs/gui/DATUM_GUI_CONFORMANCE_SPEC.md`.
- **Specify the performance-first shared GUI foundation** (`GUI-PERFORMANCE-SPEC`; `dat-gui-performance-spec-u9n`).
   GPS-C01-C06 complete and owner-ratified. Owner removed GPS-C01R from specification completion to the existing unresolved, nonblocking resize investigation. Specification closes without resize or runtime acceptance. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* dat-gui-performance-implementation-vkq, dat-gui-baseline-budgets-j9w8. *Governing:* `specs/GUI_PERFORMANCE_RECOVERY_PLAN.md`, `research/gui-performance/SHARED_GUI_PERFORMANCE_AUDIT.md`, `specs/GUI_SHARED_ENGINEERING_CONTRACT.md`, `docs/reviews/gui-performance/zoom-noop-slice.md`, `docs/reviews/gui-performance/rendering-reopening.json`, `specs/GUI_PERFORMANCE_ACCEPTANCE_MATRIX.md`, `docs/reviews/gui-performance/initial-qualification-owner-direction.json`, `specs/GUI_PERFORMANCE_IMPLEMENTATION_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_045_SHARED_GUI_PERFORMANCE.md`, `docs/reviews/gui-performance/final-specification-review.json`, `docs/reviews/gui-performance/owner-packet.json`, `docs/reviews/gui-performance/final-owner-direction.json`, `docs/reviews/gui-performance/resize-deferral-owner-direction.json`.

</details>

- **Implement the approved shared GUI performance specification** (`GUI-PERFORMANCE-IMPLEMENTATION`; `dat-gui-performance-implementation-vkq`).
   Owner resets CPU investigation to current-host horizontal/vertical native-window resize. Historical resize cost and current shared-source changes reviewed; one finite reproduction/conditional diagnosis proposal, budgets undecided. Prior CPU audit/derivation and approved ordinary pointer observation retained, no repeats or overhead claim. S4 closed; S5/GBB-P05 separate; no resize execution yet. *state `specified`; authorization `owner_decision`; parallel lane.*
   *Dependencies:* `dat-gui-performance-spec-u9n`. *Unblocks:* none. *Governing:* `specs/GUI_PERFORMANCE_RECOVERY_PLAN.md`, `specs/GUI_SHARED_ENGINEERING_CONTRACT.md`, `specs/GUI_PERFORMANCE_IMPLEMENTATION_CONTRACT.md`, `docs/reviews/gui-performance/implementation-map.json`, `docs/reviews/gui-performance/resize-deferral-owner-direction.json`, `docs/decisions/PRODUCT_MECHANICS_047_PRIVATE_TEXT_CONSTRUCTION_GUARD.md`, `docs/reviews/gui-performance/gpu-redraw-s4-owner-direction.json`, `docs/decisions/PRODUCT_MECHANICS_045_SHARED_GUI_PERFORMANCE.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r1.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/independent-review-r1.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-clarification-r2.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/reassessment-r2.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r2.json`, `docs/decisions/PRODUCT_MECHANICS_049_SHARED_RENDER_SESSION.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/proposal-r3.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/r3-review/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r3.json`, `docs/decisions/PRODUCT_MECHANICS_050_DAMAGED_SAMPLE_RESTORATION.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/r3-stopped-native/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r3-lineage-correction/validation-proposal.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r3-lineage.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r3-lineage-stopped-native/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r3-lineage-stopped-native/diagnostic-proposal.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/drm-finalization-source-review/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r3-crosshair.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r3-crosshair-replay/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/observer-correction-planning.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/crosshair-recovery-plan.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/crosshair-recovery-reassessment.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-crosshair-focused.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/focused-recovery-preparation/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/focused-native-result/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/timed-recovery/plan.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/timed-recovery/incidence-analysis.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-timed-recovery.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/timed-native-result/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/timed-native-result/report.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/painter-hover-correction/proposal.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/painter-hover-correction/review.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/drm-finalization-source-review/wait-api-followup.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-painter-hover.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/painter-hover-stopped-validation/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/painter-hover-stopped-validation/corrective-validation.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-sample-readback.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/sample-readback-validation/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/painter-hover-timed-preparation/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/painter-hover-timed-result/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/painter-hover-timed-result/report.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/resolve-source-assessment/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/resolve-source-assessment/assessment.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/proposal-r4.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/proposal-r4-source-review.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/owner-approval-r4.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/independent-review-r4.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-target-contract/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-image-admission/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-regional-graph/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-offline-readiness/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-proof-preparation/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-proof-preparation/gpu-batch-declaration.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-gpu-proof/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-gpu-proof/batch-state.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-gpu-proof/baseline-source.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-disposition/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-disposition/completion-assessment.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-disposition/declaration.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-disposition/build-state.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-disposition/campaign-state.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-disposition/native-result.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-correction/owner-direction.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-result/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-result/assessment.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-result/declaration.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-result/build-state.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-result/campaign-state.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-result/analysis.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/r4-native-result/native-result.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/packet.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/inputs.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/owner-request.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/source-review.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/owner-approval.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/disposition.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/result.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/readback-correction.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/readback-owner-approval.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/readback-result/report.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/readback-result/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/readback-result/execution-state.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/scale-relaunch/owner-approval.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/scale-relaunch/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/scale-relaunch/report.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/completion/owner-approval.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/completion/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/completion/report.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/completion/declaration.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/completion/preflight.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/completion/execution-state.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/completion/analysis.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/setup-correction/owner-direction.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/setup-correction/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/setup-correction/report.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/setup-correction/declaration.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/setup-correction/comparison.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/setup-correction/independent-review.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/setup-correction/preflight.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/setup-correction/execution-state.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/owner-direction.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/receipt.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/report.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/declaration.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/analysis.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/completion-consistency.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/comparison.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/chrome-assessment.md`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/setup-mismatch.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/preflight.json`, `docs/reviews/gui-performance/gpu-redraw-proposal/attribution-a1/corrected-result/execution-state.json`, `docs/decisions/PRODUCT_MECHANICS_051_EVIDENCE_DERIVED_PERFORMANCE_REQUIREMENTS.md`, `specs/GUI_PERFORMANCE_BASELINE_BUDGET_PLAN.md`, `docs/reviews/gui-performance/baseline-budget/owner-direction.json`, `docs/reviews/gui-performance/baseline-budget/handoff.md`, `docs/reviews/gui-performance/s4-closure/checklist.md`, `docs/reviews/gui-performance/s4-closure/owner-direction.json`, `docs/reviews/gui-performance/s4-closure/evidence-reconciliation.json`, `docs/reviews/gui-performance/s4-closure/owner-approval.json`, `docs/reviews/gui-performance/s4-closure/closure.md`, `docs/reviews/gui-performance/s5-qualification/owner-direction.json`, `docs/reviews/gui-performance/s5-qualification/plan.md`, `docs/reviews/gui-performance/s5-qualification/inventory.json`, `docs/reviews/gui-performance/cpu-reconciliation/owner-direction.json`, `docs/reviews/gui-performance/cpu-reconciliation/audit.md`, `docs/reviews/gui-performance/cpu-reconciliation/evidence.json`, `docs/reviews/gui-performance/cpu-reconciliation/derivation.md`, `docs/reviews/gui-performance/cpu-reconciliation/resize-investigation.md`, `docs/reviews/gui-performance/cpu-reconciliation/ordinary-observation.json`.
- **Establish reference baseline and derive defensible performance budgets** (`GUI-PERFORMANCE-BASELINE`; `dat-gui-baseline-budgets-j9w8`).
   Owner accepted only the limited pointer assessment at GBB-P04. Replacement budgets remain undecided. GBB-P05 schedules seven workload families, total instrumentation effects, displayed responsiveness, supported-hardware expectations and complete DRM/resource accounting, nonblocking for functionality. No additional measurement or renderer optimization authorized; representative baseline and qualification incomplete. *state `specified`; authorization `planning`; parallel lane.*
   *Dependencies:* `dat-gui-performance-spec-u9n`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_051_EVIDENCE_DERIVED_PERFORMANCE_REQUIREMENTS.md`, `specs/GUI_PERFORMANCE_BASELINE_BUDGET_PLAN.md`, `docs/reviews/gui-performance/baseline-budget/owner-direction.json`, `docs/reviews/gui-performance/baseline-budget/baseline-inventory.json`, `docs/reviews/gui-performance/baseline-budget/threshold-audit.md`, `docs/reviews/gui-performance/baseline-budget/handoff.md`, `docs/reviews/gui-performance/baseline-budget/collection-protocol.md`, `docs/reviews/gui-performance/baseline-budget/pointer-declaration.json`, `docs/reviews/gui-performance/baseline-budget/pointer-report.md`, `docs/reviews/gui-performance/baseline-budget/pointer-analysis.json`, `docs/reviews/gui-performance/baseline-budget/pointer-receipt.json`, `docs/reviews/gui-performance/baseline-budget/pointer-owner-disposition.json`.
<details>
<summary>Completed history (3 landed records)</summary>

- **Complete schematic and library GUI surface specifications** (`GUI-SURFACE-SPECS`; `dat-gui-surface-specs-usb`).
   Delegated SURFACE-S05 disposition ratified and landed in fc287786 after actual corrected distinct policy/contract and governance review. PM062 records B1/O1, shared library composition and truthful warning/readiness/narrow presentation; broader placement/pool/source/standards/HUMAN obligations remain. Explicit finite LIBRARY-GUI-AUTHORING execution selected separately; this specification issue closes without runtime qualification. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* `dat-manual-foundation-contracts-fsw`. *Unblocks:* dat-native-authoring-depth-sf9, dat-sheet-interaction-reentry-9ee. *Governing:* `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md`, `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`, `specs/WORKFLOW_DELIVERY_SURFACE_SPECIFICATION_REVIEW.md`, `docs/reviews/workflow-delivery-rollout/native-file-types-proposal.md`, `docs/reviews/workflow-delivery-rollout/library-foundation-investigation.json`, `docs/reviews/workflow-delivery-rollout/library-surface-inventory-review.json`, `docs/reviews/workflow-delivery-rollout/schematic-surface-contract-review.json`, `docs/reviews/workflow-delivery-rollout/library-surface-contract-review.json`, `docs/reviews/workflow-delivery-rollout/surface-visual-review.json`, `docs/reviews/workflow-delivery-rollout/surface-reconciliation-visual-addendum.json`, `docs/reviews/workflow-delivery-rollout/surface-s04-manager-acceptance.json`, `docs/reviews/workflow-delivery-rollout/surface-specification-combined-review.json`, `docs/reviews/workflow-delivery-rollout/surface-review-manager-acceptance.json`, `docs/reviews/workflow-delivery-rollout/surface-s05-manager-disposition.json`, `docs/decisions/PRODUCT_MECHANICS_062_SCHEMATIC_LIBRARY_SURFACE_DISPOSITION.md`.
- **Complete bounded engine authority for S5A derived selection** (`S5A-DERIVED-AUTHORITY`; `dat-s5a-derived-authority-hhsw`).
   Selection prerequisite accepted by owner after reserved independent PASS for source 91b56957 / packet e298e2df / blob cc39cd23; 102 Rust and 5 Python replay passes, no mandatory findings. DA-A4-R and DA-A4-D complete. Explicitly restore S5A-C03 under retained C02 execution approval. Positive rational-center CAM/T04 remains open dat-rational-center-cam-ir1t; capability limits preserved; no S5A UI/native acceptance. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* dat-uvt-s5a-build-1wv. *Governing:* `specs/S5A_DERIVED_AUTHORITY_PLAN.md`, `docs/reviews/workflow-delivery-rollout/s5a/foundation-gap-amendment.md`, `specs/ENGINE_SPEC.md`, `specs/SCHEMATIC_CONNECTIVITY_SPEC.md`, `docs/decisions/PRODUCT_MECHANICS_026_SELECTION_IDENTITY.md`, `specs/FOUNDATION_CONSUMER_COMPLETION_PLAN.md`, `docs/decisions/PRODUCT_MECHANICS_053_ELECTRICAL_IDENTITY_AUTHORITY.md`, `docs/decisions/PRODUCT_MECHANICS_054_NOMINAL_COPPER_DERIVED_AUTHORITY.md`, `docs/decisions/PRODUCT_MECHANICS_055_BUS_RUN_CONTACT_AUTHORITY.md`, `docs/contracts/MANUFACTURING_OUTPUT_TOOL_CONTRACT.md`.
- **Deliver the first ordinary board edit and user feedback build** (`USER-EDIT-DELIVERY`; `dat-first-user-edit-p9f1`).
   Reviewed first-edit development delivery landed; objective manager availability at a365ae5e and owner successor selection resolve publication. Claim released. Historical crash cause, ~1s confirmation and full qualification remain open. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md`, `docs/reviews/specification-history/feature-delivery-records.md`, `docs/decisions/PRODUCT_MECHANICS_060_PROJECT_WRITER_OWNERSHIP.md`.

</details>

- **Build UVT S5A selection and compound inspection** (`UVT-S5A-BUILD`; `dat-uvt-s5a-build-1wv`).
   Owner-directed 2026-10-18 resumption of S5A-C04 under its retained execution grant. C03 and accepted C04 cases remain credited; finish only genuine native proof gaps and necessary existing-contract corrections, including current diagnostic reader migration before affected proof. Management coordinates the requested dev-agent-02 worker, actual review and C04 acceptance; exact worker MCP identity/claim transfer is pending. Current claim is management metadata only. Wayland/wgpu QA delivery, honest scoped reuse and source/journal preservation are required. C05 reserved replay, C06 owner acceptance and other qualification remain distinct; no automatic successor. *state `in_progress`; authorization `execution`; **CANONICAL NEXT**.*
   *Dependencies:* `dat-s5-selection-visual-contract-zid`, `dat-s5a-derived-authority-hhsw`. *Unblocks:* dat-gui-p2-cross-probe-27z. *Governing:* `docs/decisions/PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING.md`, `docs/gui/DATUM_UNIVERSAL_VIEWPORT_TOOLING_SPEC.md`, `specs/FOUNDATION_CONSUMER_COMPLETION_PLAN.md`, `docs/reviews/workflow-delivery-rollout/s5a/preparation-plan.md`, `docs/decisions/PRODUCT_MECHANICS_051_EVIDENCE_DERIVED_PERFORMANCE_REQUIREMENTS.md`, `docs/reviews/workflow-delivery-rollout/s5a/foundation-gap-amendment.md`, `docs/decisions/PRODUCT_MECHANICS_056_IMPORTED_ZONE_FILL_ADOPTION.md`, `docs/decisions/PRODUCT_MECHANICS_057_C03_SOURCE_AND_ACTION_AUTHORITY.md`, `docs/reviews/workflow-delivery-rollout/s5a/thirty-degree-proposal.md`, `docs/decisions/PRODUCT_MECHANICS_058_EXACT_ROTATED_PAD_QUERIES.md`, `docs/reviews/specification-history/specification-reorganization.md`.
   *Completion plan:* `python3 scripts/project_status.py details`.
- **Build GUI Phase 2 P2.3 cross-probe** (`GUI-P2-CROSSPROBE`; `dat-gui-p2-cross-probe-27z`).
   Project one selection identity into Board and Schematic panes after S5A lands; execution remains separately authorized. *state `blocked`; authorization `planning`.*
   *Dependencies:* `dat-uvt-s5a-build-1wv`. *Unblocks:* dat-gui-p2-full-inspector-0ye, dat-native-authoring-depth-sf9. *Governing:* `docs/gui/DATUM_GUI_PHASE_2_SPEC.md`, `docs/decisions/PRODUCT_MECHANICS_023_UNIVERSAL_VIEWPORT_TOOLING.md`, `specs/WORKFLOW_DELIVERY_PHASE2_COMPLETION_PLAN.md`, `docs/decisions/PRODUCT_MECHANICS_051_EVIDENCE_DERIVED_PERFORMANCE_REQUIREMENTS.md`.
- **Build GUI Phase 2 P2.4 full inspector** (`GUI-P2-INSPECTOR`; `dat-gui-p2-full-inspector-0ye`).
   Complete Identity, Placement, and Checks inspector sections after the shared cross-probe substrate lands. *state `blocked`; authorization `planning`.*
   *Dependencies:* `dat-gui-p2-cross-probe-27z`. *Unblocks:* none. *Governing:* `docs/gui/DATUM_GUI_PHASE_2_SPEC.md`, `specs/WORKFLOW_DELIVERY_PHASE2_COMPLETION_PLAN.md`, `docs/gui/DATUM_GUI_DESIGN_SPEC.md`, `docs/decisions/PRODUCT_MECHANICS_051_EVIDENCE_DERIVED_PERFORMANCE_REQUIREMENTS.md`.
- **Build the inert marking-menu shell** (`GUI-MARKING-MENU`; `dat-marking-menu-shell-g0w`).
   Render the menu_model-driven radial shell with disabled mutation entries; execution requires an explicit go. *state `specified`; authorization `planning`; parallel lane.*
   *Dependencies:* none. *Unblocks:* dat-gui-write-path-qiu. *Governing:* `docs/gui/DATUM_GUI_CONTEXT_MENU_CONTENT.md`, `docs/gui/DATUM_GUI_PARAMETRIC_TOOLING.md`, `specs/WORKFLOW_DELIVERY_MARKING_MENU_COMPLETION_PLAN.md`, `docs/decisions/PRODUCT_MECHANICS_051_EVIDENCE_DERIVED_PERFORMANCE_REQUIREMENTS.md`.
- **Enable the GUI journaled write path** (`GUI-WRITE-PATH`; `dat-gui-write-path-qiu`).
   Execute the P0-W3 direct typed-operation path after its actual write-authority prerequisites land and the owner authorizes execution; no passive feedback viewer is an implicit blocker. *state `blocked`; authorization `planning`.*
   *Dependencies:* `dat-marking-menu-shell-g0w`, `dat-project-write-ownership-lock-0ne`, `dat-manual-foundation-contracts-fsw`. *Unblocks:* dat-native-authoring-depth-sf9. *Governing:* `docs/gui/DATUM_GUI_WRITE_PATH_PLAN.md`, `docs/decisions/PRODUCT_MECHANICS_019_GUI_PRODUCT_MODEL.md`, `docs/decisions/PRODUCT_MECHANICS_017_VERB_REGISTRY.md`, `specs/FOUNDATION_CONSUMER_COMPLETION_PLAN.md`, `docs/decisions/PRODUCT_MECHANICS_051_EVIDENCE_DERIVED_PERFORMANCE_REQUIREMENTS.md`.
- **Build native schematic and PCB authoring depth** (`NATIVE-AUTHORING`; `dat-native-authoring-depth-sf9`).
   Build manual-first native editors over the shared tooling and canonical GUI commit path after their prerequisites land. *state `blocked`; authorization `planning`.*
   *Dependencies:* `dat-gui-p2-cross-probe-27z`, `dat-gui-surface-specs-usb`, `dat-gui-write-path-qiu`, `dat-manual-foundation-contracts-fsw`. *Unblocks:* none. *Governing:* `docs/contracts/SCHEMATIC_AUTHORING_TOOL_CONTRACT.md`, `docs/contracts/PCB_LAYOUT_TOOL_CONTRACT.md`, `specs/FOUNDATION_CONSUMER_COMPLETION_PLAN.md`.
<details>
<summary>Completed history (4 landed records)</summary>

- **Specify the documentation-system object model** (`DOC-SYSTEM-SPEC`; `dat-documentation-system-spec-y8z`).
   Specify Sheet, Viewport, annotation, SheetSet, and the v1 paper-space scope without authorizing implementation. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* dat-dimension-selection-reentry-kxk, dat-distributed-collaboration-architecture-lt1. *Governing:* `docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md`, `specs/PUBLISH_SPACE_SPEC.md`.
- **Expose operational ownership in canonical next-task output** (`STATE-NEXT-OWNERSHIP`; `dat-next-output-ownership-uqz`).
   Make the selector report tracker status, assignee, and live-claim state so agents cannot silently reintroduce stale conversational ownership. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* memory-independent next-task narration. *Governing:* `docs/decisions/PRODUCT_MECHANICS_025_PROJECT_STATE_AUTHORITY.md`, `docs/PROJECT_STATE_POLICY.md`.
- **Make canonical task completion plans deterministic** (`STATE-TASK-DETAILS`; `dat-task-completion-alignment-4jp`).
   Add validated structured completion plans and one deterministic task-details query so agents cannot omit controlling requirements or infer post-completion ordering. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_025_PROJECT_STATE_AUTHORITY.md`, `docs/PROJECT_STATE_POLICY.md`.
- **Make task-detail presentation verbatim and non-concurrent** (`STATE-DETAIL-PRESENTATION`; `dat-task-detail-presentation-bjk`).
   Make the human task-detail output a verbatim response contract with fixed order and explicit one-active-step, no-inferred-parallelism semantics. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_025_PROJECT_STATE_AUTHORITY.md`, `docs/PROJECT_STATE_POLICY.md`.

</details>

- **Specify distributed project collaboration and semantic merge** (`DISTRIBUTED-COLLAB-SPEC`; `dat-distributed-collaboration-architecture-lt1`).
   Research and specify local-first, Git-compatible semantic collaboration for simultaneous, remote, air-gapped, and high-latency teams before any multi-user implementation is authorized. *state `planned`; authorization `planning`.*
   *Dependencies:* `dat-documentation-system-spec-y8z`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_007_PROJECT_WORKSPACE_MODEL.md`, `docs/decisions/PRODUCT_MECHANICS_001_CANONICAL_EDIT_MODEL.md`, `specs/WORKFLOW_DELIVERY_COLLABORATION_COMPLETION_PLAN.md`.
<details>
<summary>Completed history (5 landed records)</summary>

- **Specify the Datum Product Revision Engine** (`PRODUCT-REVISION-SPEC`; `dat-product-revision-engine-k9f`).
   Research and specify Datum-owned configuration, change, baseline, revision, release, approval, effectivity, status-accounting, reproduction, and audit authority before any Revision or Publish implementation is authorized. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* dat-product-revision-engine-build-18f. *Governing:* `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md`, `specs/PRODUCT_REVISION_ENGINE_SPEC.md`, `research/documentation-system/PRODUCT_REVISION_ENGINE_RESEARCH.md`, `research/documentation-system/REV_C07_PRODUCT_REVISION_ENGINE_RATIFICATION_PACKET.md`, `specs/PUBLISH_SPACE_SPEC.md`.
- **Specify the Datum Global Preferences Engine** (`GLOBAL-PREFERENCES-SPEC`; `dat-global-preferences-engine-qcv`).
   Research and specify typed system/user, organization, Project, and contextual preference authority, including precedence, provenance, policy seeding, schema migration, persistence/recovery, managed settings, synchronization boundaries, and accessible GUI, without conflating machine-local preferences with governed Project policy. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-product-revision-engine-k9f`, `dat-visual-truth-file-lane-enforcement-jfl`. *Unblocks:* dat-product-revision-engine-build-18f. *Governing:* `research/preferences-system/GLOBAL_PREFERENCES_ENGINE_RESEARCH.md`, `research/preferences-system/GP_C02B_DOMAIN_PEER_PREFERENCES_RESEARCH.md`, `research/preferences-system/GP_C03_TYPED_AUTHORITY_DECISION_PACKET.md`, `research/preferences-system/GP_C03_PROJECT_SEED_AND_CONTEXT_DECISION_PACKET.md`, `research/preferences-system/GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md`, `research/preferences-system/GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md`, `research/preferences-system/GP_C06_CONSOLIDATED_RATIFICATION_PACKET.md`, `specs/GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md`, `research/preferences-system/GP_DRAFT_SETTINGS_CATALOG_SEED.md`, `research/preferences-system/GP_SHARED_UNITS_ENGINE_REQUIREMENT.md`, `research/documentation-system/DRAFTING_STANDARD_OWNERSHIP_DECISION_PACKET.md`, `docs/decisions/PRODUCT_MECHANICS_036_SCHEMATIC_DRAWING_THEMES.md`, `docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md`, `specs/GLOBAL_PREFERENCES_ENGINE_IMPLEMENTATION_PLAN.md`, `docs/gui/prototypes/preferences-ux-study.html`, `docs/gui/prototypes/preferences-window.html`, `research/documentation-system/PRODUCT_REVISION_ENGINE_RESEARCH.md`, `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`.
- **Build the shared Units exact core** (`SHARED-UNITS-ENGINE`; `dat-shared-units-engine-build-915`).
   Implement the engine-owned exact integer-nanometer and typed parse/format core before the first real Global Preferences slice; surface parity and Units production acceptance occur later through that real Preferences surface. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-global-preferences-engine-qcv`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_039_PREFERENCES_FIRST_SETTINGS_ENTRY.md`, `docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md`, `research/preferences-system/GP_SHARED_UNITS_ENGINE_REQUIREMENT.md`, `specs/GLOBAL_PREFERENCES_ENGINE_IMPLEMENTATION_PLAN.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`.
- **Build the Global Preferences foundation and first GUI slice** (`GLOBAL-PREFERENCES-ENGINE`; `dat-global-preferences-engine-build-vge`).
   Implement the typed resolver and durable repository foundation, then deliver a production-grade three-row Appearance slice through the real nested Edit > Preferences > Global Preferences doorway, with typed presentation metadata, live consumers, durability, accessibility, recovery honesty, and zero Project mutation. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-global-preferences-engine-qcv`, `dat-shared-units-engine-build-915`. *Unblocks:* dat-global-preferences-completion-f84. *Governing:* `docs/decisions/PRODUCT_MECHANICS_039_PREFERENCES_FIRST_SETTINGS_ENTRY.md`, `docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md`, `specs/GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md`, `research/preferences-system/GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md`, `research/preferences-system/GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md`, `specs/GLOBAL_PREFERENCES_ENGINE_IMPLEMENTATION_PLAN.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`, `docs/gui/prototypes/preferences-window.html`.
- **Complete shared Units surface parity through Preferences** (`SHARED-UNITS-SURFACE-PARITY`; `dat-shared-units-surface-parity-s9x`).
   Reconcile the shared Units requirement around eight typed Global defaults, Project Working Units, per-quantity precision, lossless editing, exact scientific notation, decimal-degree V1 behavior, deterministic migration, and separate Publish/document authority; obtain approved Global and Project targets before staged execution and owner acceptance. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-global-preferences-engine-build-vge`. *Unblocks:* dat-global-preferences-completion-f84. *Governing:* `docs/decisions/PRODUCT_MECHANICS_040_PROJECT_WORKING_UNITS_AUTHORITY.md`, `docs/decisions/PRODUCT_MECHANICS_039_PREFERENCES_FIRST_SETTINGS_ENTRY.md`, `docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md`, `research/preferences-system/GP_SHARED_UNITS_ENGINE_REQUIREMENT.md`, `specs/GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md`, `specs/GLOBAL_PREFERENCES_ENGINE_IMPLEMENTATION_PLAN.md`, `specs/PROJECT_PREFERENCES_SPEC.md`, `docs/gui/prototypes/preferences-window.html`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`.

</details>

- **Complete and production-accept Global Preferences** (`GLOBAL-PREFERENCES-COMPLETION`; `dat-global-preferences-completion-f84`).
   Complete Global Preferences by activating only consumer-ready descriptors, preserving incomplete identities as reserved candidates, and proving truthful controls, timing, accessibility, recovery, and exclusions before later parity and production-acceptance gates. *state `ready`; authorization `execution`.*
   *Dependencies:* `dat-global-preferences-engine-build-vge`, `dat-shared-units-surface-parity-s9x`. *Unblocks:* dat-project-preferences-spec-kiw. *Governing:* `docs/decisions/PRODUCT_MECHANICS_039_PREFERENCES_FIRST_SETTINGS_ENTRY.md`, `docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md`, `docs/decisions/PRODUCT_MECHANICS_040_PROJECT_WORKING_UNITS_AUTHORITY.md`, `research/preferences-system/GP_SHARED_UNITS_ENGINE_REQUIREMENT.md`, `specs/GLOBAL_PREFERENCES_V1_DESCRIPTOR_CATALOG.md`, `research/preferences-system/GP_C04_STORAGE_MIGRATION_EXCHANGE_RECOVERY_CONTRACT.md`, `research/preferences-system/GP_C05_INTERACTION_AND_VISUAL_CONTRACT.md`, `specs/GLOBAL_PREFERENCES_ENGINE_IMPLEMENTATION_PLAN.md`, `specs/GLOBAL_PREFERENCES_PRODUCT_SURFACE_CONTRACT.md`, `specs/global_preferences_production_acceptance_matrix.json`, `specs/MCP_API_SPEC.md`, `docs/contracts/AI_CLI_MCP_TOOL_SURFACE.md`, `docs/gui/prototypes/preferences-window.html`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`, `docs/reviews/preferences/gp-cm05-implementation/owner-direction.md`.
- **Specify the Global and Project Preferences doorway** (`PROJECT-PREFERENCES-SPEC`; `dat-project-preferences-spec-kiw`).
   Complete the exact Project-owned settings inventory, canonical query/mutation contract, conventional menu doorway, interaction/accessibility specification, and protected visual truth after Global Preferences is stable. *state `blocked`; authorization `planning`.*
   *Dependencies:* `dat-global-preferences-completion-f84`. *Unblocks:* dat-project-preferences-build-tlq. *Governing:* `docs/decisions/PRODUCT_MECHANICS_039_PREFERENCES_FIRST_SETTINGS_ENTRY.md`, `specs/PROJECT_PREFERENCES_SPEC.md`, `research/preferences-system/PREFERENCES_FIRST_REENTRY_AUDIT.md`, `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`, `docs/gui/DATUM_GUI_MENU_BINDINGS.md`, `docs/gui/prototypes/preferences-window.html`, `specs/WORKFLOW_DELIVERY_SURFACE_SPECIFICATION_REVIEW.md`.
- **Build and stabilize Project Preferences** (`PROJECT-PREFERENCES-BUILD`; `dat-project-preferences-build-tlq`).
   Implement Project Preferences through real Project queries and canonical journaled mutations after specification and protected visual ratification; Revision configuration remains a separate later decision. *state `blocked`; authorization `owner_decision`.*
   *Dependencies:* `dat-project-preferences-spec-kiw`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_039_PREFERENCES_FIRST_SETTINGS_ENTRY.md`, `specs/PROJECT_PREFERENCES_SPEC.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`, `docs/gui/prototypes/preferences-window.html`, `docs/decisions/PRODUCT_MECHANICS_051_EVIDENCE_DERIVED_PERFORMANCE_REQUIREMENTS.md`.
- **Build and production-accept the Datum Product Revision Engine** (`PRODUCT-REVISION-ENGINE`; `dat-product-revision-engine-build-18f`).
   Historical Product Revision Engine execution item. Product Mechanics 038 rejects REV-I10, suspends every successor slice, quarantines the broad implementation, and removes this item from production-acceptance and downstream-unblock authority pending bounded recovery. *state `deferred`; authorization `none`.*
   *Dependencies:* `dat-product-revision-engine-k9f`, `dat-global-preferences-engine-qcv`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_038_REVISION_RECOVERY_AND_PRODUCT_BASELINE.md`, `docs/decisions/PRODUCT_MECHANICS_039_PREFERENCES_FIRST_SETTINGS_ENTRY.md`, `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md`, `specs/PRODUCT_REVISION_ENGINE_SPEC.md`, `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md`, `research/documentation-system/REV_I00_EXECUTION_AUTHORIZATION_PACKET.md`, `research/documentation-system/REV_I02_EXECUTION_AUTHORIZATION_PACKET.md`, `research/documentation-system/REV_I03_EXECUTION_AUTHORIZATION_PACKET.md`, `research/documentation-system/REV_I04_EXECUTION_AUTHORIZATION_PACKET.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`.
- **Plan native ECAD-MCAD interoperability** (`MCAD-INTEROP-PLACEHOLDER`; `dat-mcad-interop-l3p`).
   Scheduled placeholder for three-rung native ECAD-MCAD interoperability: STEP assembly export, mechanical outline/constraint import, and IDX-class proposal/quarantine/evidence exchange. Datum never embeds a mechanical modeling kernel or permits an external MCAD master. Geometry/STEP-import dependencies require exact Product Mechanics 029 owner approval. Sequence only after Revision Engine production acceptance and Global Preferences, approximately with future 3D-viewer work; no research, specification, dependency, or implementation is authorized. *state `planned`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `specs/STANDARDS_COMPLIANCE_SPEC.md`, `docs/DATUM_SHARED_TOOLING_TAXONOMY.md`, `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md`, `docs/decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md`.
<details>
<summary>Completed history (2 landed records)</summary>

- **Decide adopted drafting-standard object ownership** (`DRAFTING-STANDARD-AUTHORITY`; `dat-drafting-standard-authority-jcz`).
   Owner decision packet for the authority that adopts a drafting-standard object for a Project. The packet distinguishes projection angle, dimensioning conventions, and sheet symbology from Product Mechanics 034 revision coding; it authorizes no implementation or dependency. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* dat-global-preferences-engine-qcv, dat-adopted-drafting-standard-object-er9. *Governing:* `research/documentation-system/DRAFTING_STANDARD_OWNERSHIP_DECISION_PACKET.md`, `docs/decisions/PRODUCT_MECHANICS_035_ADOPTED_DRAFTING_STANDARD_AUTHORITY.md`, `docs/decisions/PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md`, `docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md`, `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md`, `specs/PUBLISH_SPACE_SPEC.md`, `specs/STANDARDS_COMPLIANCE_SPEC.md`, `docs/gui/DATUM_RENDERING_BOOK.md`, `docs/gui/prototypes/preferences-window.html`.
- **Resolve schematic canvas background authority** (`SCHEMATIC-BACKGROUND-OWNER-DECISION`; `dat-schematic-background-owner-decision-gom`).
   Ratify Candidate B as one persisted machine Presentation preference selecting the complete governed Dark or Light schematic drawing theme, while preserving the Dark factory default, dark chrome, board color-scheme boundary, and Publish/print independence. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_036_SCHEMATIC_DRAWING_THEMES.md`, `docs/gui/prototypes/canvas-background-decision.html`, `docs/gui/prototypes/preferences-window.html`, `docs/gui/DATUM_RENDERING_BOOK.md`.

</details>

- **Specify the adopted drafting-standard object** (`ADOPTED-DRAFTING-STANDARD-SPEC`; `dat-adopted-drafting-standard-object-er9`).
   Specify the documentation-system-owned Project drafting-standard object, its StandardsRegistry references, typed authority, Publish/template seams, migration, provenance, and receipted Global Preferences new-Project seed without authorizing implementation. *state `planned`; authorization `planning`.*
   *Dependencies:* `dat-drafting-standard-authority-jcz`. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_035_ADOPTED_DRAFTING_STANDARD_AUTHORITY.md`, `research/documentation-system/DRAFTING_STANDARD_OWNERSHIP_DECISION_PACKET.md`, `docs/decisions/PRODUCT_MECHANICS_010_INDUSTRY_STANDARDS_COMPLIANCE.md`, `docs/decisions/PRODUCT_MECHANICS_020_PAPER_SPACE_AND_VIEWPORTS.md`, `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md`, `specs/PUBLISH_SPACE_SPEC.md`, `specs/STANDARDS_COMPLIANCE_SPEC.md`, `specs/WORKFLOW_DELIVERY_DRAFTING_STANDARD_COMPLETION_PLAN.md`.
<details>
<summary>Completed history (6 landed records)</summary>

- **Enforce Claude ownership of GUI visual-truth files** (`VISUAL-TRUTH-FILE-LANE`; `dat-visual-truth-file-lane-enforcement-jfl`).
   Codify the owner-directed Claude-only visual-truth file lane, define fail-closed staged commit enforcement and gate-integrity behavior, obtain owner approval of the marker design, and only then implement and prove the hook. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `research/process-quality/VISUAL_TRUTH_FILE_LANE_ENFORCEMENT_PACKET.md`, `docs/PROJECT_STATE_POLICY.md`.
- **Recover the Revision product baseline and contain rejected REV-I10** (`REVISION-PRODUCT-RECOVERY`; `dat-revision-product-recovery-dap`).
   Apply the owner-approved manual-first product baseline, supersede permanent unmanaged Revision presentation, quarantine unvalidated implementation breadth, remove fictional runtime records, and require a small comprehensible owner review before any wider Revision execution resumes. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_038_REVISION_RECOVERY_AND_PRODUCT_BASELINE.md`, `docs/decisions/PRODUCT_MECHANICS_039_PREFERENCES_FIRST_SETTINGS_ENTRY.md`, `docs/decisions/PRODUCT_MECHANICS_034_PRODUCT_REVISION_ENGINE.md`, `docs/decisions/PRODUCT_MECHANICS_037_GLOBAL_PREFERENCES_ENGINE.md`, `specs/PRODUCT_REVISION_ENGINE_SPEC.md`, `specs/PRODUCT_REVISION_ENGINE_IMPLEMENTATION_PLAN.md`, `research/documentation-system/REVISION_RECOVERY_IMPLEMENTATION_INVENTORY.md`, `research/preferences-system/PREFERENCES_FIRST_REENTRY_AUDIT.md`, `specs/PROJECT_PREFERENCES_SPEC.md`, `docs/PROJECT_STATE_POLICY.md`.
- **Establish workflow delivery readiness and acceptance governance** (`WORKFLOW-DELIVERY-QUALITY`; `dat-workflow-delivery-quality-xgj`).
   Audit manual workflow capability, research missing CAD foundations, and prepare a concrete readiness/acceptance gate adoption packet; preserve active Preferences ownership and select this planning work only through an explicit coordinated handoff. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `specs/WORKFLOW_DELIVERY_QUALITY_PLAN.md`.
- **Specify the first manual workflow foundation contracts** (`FOUNDATION-WORKFLOW-SPEC`; `dat-manual-foundation-contracts-fsw`).
   Resolve the bounded F1-F8 consumer questions and authority ambiguities before promoting the native project doorway and proposing changes to existing authoring prerequisites; no product implementation or automatic successor selection. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* `dat-workflow-delivery-quality-xgj`. *Unblocks:* dat-gui-write-path-qiu, dat-native-authoring-depth-sf9. *Governing:* `specs/WORKFLOW_DELIVERY_QUALITY_PLAN.md`, `docs/reviews/workflow-foundation/authority-review.md`, `docs/decisions/PRODUCT_MECHANICS_012_APPLICATION_QUALITY_BAR.md`, `specs/FOUNDATION_MANUAL_WORKFLOW_CONTRACT.md`, `docs/reviews/workflow-foundation/handoff.md`.
- **Implement and validate the bounded workflow delivery gate pilot** (`WORKFLOW-DELIVERY-GATE-PILOT`; `dat-workflow-gate-pilot-b3s`).
   Prepare a fresh authorized handoff, build the scoped delivery gate and actual action-consumer pilot, and obtain independent replay plus owner activation; no blanket rollout or EDA authoring acceptance. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* `dat-workflow-delivery-quality-xgj`. *Unblocks:* none. *Governing:* `specs/WORKFLOW_DELIVERY_ADOPTION_PACKET.md`, `specs/WORKFLOW_DELIVERY_GATE_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_041_WORKFLOW_DELIVERY_QUALITY.md`, `docs/reviews/workflow-delivery-pilot/handoff.md`.
- **Prepare broader workflow delivery enforcement and adoption rollout** (`WORKFLOW-DELIVERY-ROLLOUT`; `dat-wdq-broader-rollout-tqh`).
   Owner-approved separate workflow-quality planning: schedule foundation specification follow-ups and prepare exact broader enforcement, promotion and three-slice adoption proposals without taking over product implementation. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `specs/WORKFLOW_DELIVERY_ROLLOUT_PLAN.md`, `docs/reviews/workflow-foundation/handoff.md`.

</details>

- **Ratify project writer ownership and recovery serialization** (`PROJECT-WRITER-OWNERSHIP`; `dat-project-write-ownership-lock-0ne`).
   Bounded PM060 decision, actual independent review/delegated UED ratification and implemented engine-owned lease/fresh revision-tip validation are credited. Pending PWO-C01/C02/REVIEW/C03 reconcile full writer/consumer/platform/concurrency/recovery clauses and any genuinely broader ratification; no repeated bounded lock choice or whole PWO/GWP qualification. Both hard dependents remain gated. *state `planned`; authorization `planning`; parallel lane.*
   *Dependencies:* none. *Unblocks:* dat-multi-agent-proposals-first-f49, dat-gui-write-path-qiu. *Governing:* `specs/FOUNDATION_FOLLOWUP_PLANS.md`, `docs/decisions/PRODUCT_MECHANICS_060_PROJECT_WRITER_OWNERSHIP.md`.
- **Reconcile native connectivity identity, formation and gesture authority** (`NATIVE-CONNECTIVITY-CONTRACT`; `dat-grid-connectivity-authority-5s6`).
   Owner-ratified connectivity and gesture clauses consistently distinguish electrical identity, geometric formation and attached movement; complete owning-lane reconciliation preserves shared tooling and journal authority. *state `planned`; authorization `planning`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `specs/FOUNDATION_FOLLOWUP_PLANS.md`.
<details>
<summary>Completed history (1 landed records)</summary>

- **Define and deliver the ordinary native GUI project startup doorway** (`NATIVE-PROJECT-DOORWAY`; `dat-native-project-startup-vrf`).
   Accepted C01-C09 native startup delivery: shared New/Open browsing, automatic Units, protected New Folder/Rename, preserved terminal panel and selected Project-root session, manifest activation and successive path completion. C09 implementation 35d5e014 and authorized visual commit 93e94e0c have actual independent review, owner QA and manager objective acceptance. Existing qualification defects and S5A hold remain separate; no successor selected. *state `landed`; authorization `none`.*
   *Dependencies:* `dat-manual-foundation-contracts-fsw`. *Unblocks:* none. *Governing:* `specs/FOUNDATION_FOLLOWUP_PLANS.md`, `docs/reviews/workflow-delivery-rollout/native-project-doorway.md`, `docs/decisions/PRODUCT_MECHANICS_005_EMBEDDED_TERMINAL.md`, `docs/decisions/PRODUCT_MECHANICS_027_FULL_NATIVE_TERMINAL.md`, `docs/gui/DATUM_NATIVE_TERMINAL_SPEC.md`, `docs/decisions/PRODUCT_MECHANICS_063_SHARED_FILE_BROWSER_ACTIONS.md`.

</details>

- **Define native workflow fixtures and ratify performance measurement budgets** (`NATIVE-WORKFLOW-BUDGETS`; `dat-native-workflow-budgets-0h4`).
   Owner-ratified native fixtures and performance measurement budgets specify exact populations, timing boundaries, environment and failure policy; no product performance or harness implementation is claimed. *state `planned`; authorization `planning`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `specs/FOUNDATION_FOLLOWUP_PLANS.md`.
<details>
<summary>Completed history (1 landed records)</summary>

- **Implement broader workflow delivery enforcement and measured adoption** (`WORKFLOW-DELIVERY-IMPLEMENTATION`; `dat-wdq-rollout-implementation-ffy`).
   All eleven workflow milestones complete. Owner accepted the installed and independently verified guardrails and product-owner handoffs. No execution claim or workflow step remains. Product adoption, native proof and product acceptance remain open in their existing lanes; no successor is implicitly authorized. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `specs/WORKFLOW_DELIVERY_EXECUTION_PLAN.md`, `specs/WORKFLOW_DELIVERY_INFRASTRUCTURE_CONTRACT.md`, `specs/workflow_delivery/rollout.contract.json`, `specs/WORKFLOW_DELIVERY_COVERAGE_PROPOSAL.md`, `docs/reviews/workflow-delivery-rollout/coverage-proposal.json`, `research/process-quality/WORKFLOW_COHORT_CONTRACT_PREPARATION.md`, `docs/reviews/workflow-delivery-rollout/cohort-mappings.json`, `specs/workflow_delivery/s5a.contract.json`, `specs/workflow_delivery/gui-write-path.contract.json`, `specs/workflow_delivery/native-authoring.contract.json`, `specs/WORKFLOW_DELIVERY_ROLLOUT_IMPLEMENTATION.md`, `specs/WORKFLOW_DELIVERY_ROLLOUT_PLAN.md`, `docs/reviews/workflow-delivery-rollout/infrastructure/independent/review-completion.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/owner-dispositions.md`, `docs/reviews/workflow-delivery-rollout/infrastructure/final-publication/producer-inspection.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/workspace-repair-authorization.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/producer-closeout.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/index-repair-authorization.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/proof-renewal-authorization.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/index-repair-producer-closeout.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/sequencing-repair-authorization.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/sequencing-producer-closeout.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/sequencing-independent-closeout.json`, `docs/reviews/workflow-delivery-rollout/infrastructure/workspace-renewal/sequencing-owner-dispositions.md`.

</details>

- **Repair the owner-directed workflow revision path** (`WORKFLOW-OWNER-REVISION`; `dat-workflow-owner-revision-71j`).
   Admit, claim and finish the owner-directed revision repair with usable preparation, durable commits, independent proof and exact promotion. *state `specified`; authorization `planning`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `specs/WORKFLOW_OWNER_REVISION_REPAIR_PLAN.md`, `research/process-quality/WORKFLOW_OWNER_REVISION_ADMISSION.md`, `docs/reviews/workflow-owner-revision/admission-owner-20260916.json`, `specs/WORKFLOW_OWNER_REVISION_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_044_INCREMENTAL_DELIVERY_BOUNDARIES.md`.
<details>
<summary>Completed history (39 landed records)</summary>

- **Resolve current-host window resize resource consumption** (`GUI-RESIZE-REPAIR`; `dat-gui-vertical-resize-cpu-toj`).
   Owner-approved buffer reuse is now the ordinary main workspace default; no special launch option. No-option two-axis visible checks match the opt-in version exactly. Exact fallback, diagnostics, budgets and S4/broad qualification boundaries preserved. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/reviews/gui-performance/cpu-reconciliation/resize-repair.md`, `docs/decisions/PRODUCT_MECHANICS_045_SHARED_GUI_PERFORMANCE.md`, `docs/decisions/PRODUCT_MECHANICS_051_EVIDENCE_DERIVED_PERFORMANCE_REQUIREMENTS.md`, `docs/decisions/PRODUCT_MECHANICS_052_CURRENT_HOST_RESIZE_BUFFER_REUSE.md`.
- **Enforce ordinary feature delivery and continuation in the selector** (`FEATURE-DELIVERY-ENFORCEMENT`; `dat-feature-delivery-enforcement-kux5`).
   PM059 ordinary-workflow, concrete prerequisite and development-evidence validation implemented in 27e98908 with 60 focused tests and governance checks passed. First-edit manager ownership, actual writer review and qualification boundaries unchanged; no automatic grant or acceptance. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md`, `docs/reviews/specification-history/feature-delivery-records.md`.
- **Deliver first native symbol placement and wire connection** (`USER-SCHEMATIC-DELIVERY`; `dat-first-native-schematic-948o`).
   FIRST-SYMBOL-WIRE-01 bounded development delivery accepted at94c95ae6 with actual independent PASS. Owner-appointed project manager records explicit delegated NSD-C06 successor selection in f7a4a465: UVT-S5A-BUILD/S5A-C04 under existing execution authorization. C01-C06 complete; no implementation claim. Full qualification and disclosed Clippy/parity defects remain unpassed and unwaived. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md`, `docs/reviews/specification-history/feature-delivery-records.md`.
- **Deliver native schematic Symbol Move** (`USER-SCHEMATIC-SYMBOL-MOVE`; `dat-native-symbol-move-ydza`).
   Bounded Symbol Move behavior landed and independently reviewed on disclosed synthetic test data; no approved production library inferred. Manager explicitly selects existing GUI-SURFACE-SPECS planning to reconcile owner-identified library foundation gap. Full qualification remains open. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md`, `docs/reviews/specification-history/feature-delivery-records.md`.
- **Compact progress tracker and preserve status evidence** (`PROGRESS-COMPACTION`; `dat-progress-compaction-7lmq`).
   Owner-directed compact-tracker correction landed: PROGRESS reduced from3521 to685lines, detailed status/history preserved in linked cohesive shards. Actual independent structural/content/link review PASS after performance-summary correction; coverage, ERC/connectivity, selector/projection, governance and whitespace checks PASS. Maintenance claim released and issue closed. GUI-SURFACE-SPECS/SURFACE-S05 remains selected/unanswered; no product/runtime/source/standards or successor authority changed. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `specs/PROGRAM_SPEC.md`, `specs/PROGRESS.md`.
- **Share existing library definition validation through the engine** (`LIBRARY-VALIDATION-CONVERGENCE`; `dat-library-validation-convergence-88z1`).
   Landed engine-owned existing definition validation and complete CLI handler extraction;15focused guarded tests plus independent code/document reviews PASS; three downward debt ratchets and owning inventories reconciled. PM059 manager accepts bounded outcome. Full drift FAIL13unchanged baseline Clippy and unrelated GUI parity remain open; no global qualification, waiver, source adoption, manufacturing rules or S05 disposition. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Decompose oversized visual study sources without changing candidate meaning** (`SURFACE-STUDY-SOURCE-HEALTH`; `dat-surface-study-source-health-3aio`).
   Landed cohesive study pages/shared CSS preserving25section blocks and29anchors;12normal/narrow actual captures independently inspected, structural/presentation and registration reviews PASS. Historical receipts remain scoped to their original snapshots. PM059 manager accepts bounded outcome. Full drift FAIL13unchanged baseline Clippy and unrelated GUI parity remain open; no global qualification, waiver, source adoption, manufacturing rules or S05 disposition. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/SOURCE_HEALTH_POLICY.md`.
- **Enforce existing Footprint references in generic manual JSON edits** (`FOOTPRINT-JSON-REFERENCE-VALIDATION`; `dat-footprint-json-reference-validation-cn2p`).
   Existing-contract manual generic Footprint JSON Set repair implemented and independently reviewed code/behavior PASS. Four reproduced absent/wrong-kind Package/Padstack cases now refuse with byte-identical Footprint/journal/query/resolved state; six provenance controls match baseline. Three focused CLI tests PASS after retained fixture correction. Same-model canonical Set/history and unknown fields preserved; complete edit lifecycle extracted, ceiling2596to2565. Owning docs independently reviewed, route/state gates PASS. Required full drift failed13existing Clippy diagnostics in five unchanged files; later gates unreached, no full qualification/waiver/S05/newrules. Landed with actual manager acceptance and tracker closure; no automatic successor authorization. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Enforce existing references in generic manual Footprint JSON creation** (`FOOTPRINT-JSON-CREATE-REFERENCES`; `dat-footprint-json-create-references-5ds6`).
   Under owner continuous-development directive, appointed manager explicitly selects reproduced sibling existing-contract CREATE defect after closing SET repair. Deliver manual generic Footprint Create Package/Padstack existence-kind checks, preserving13actual baseline controls including root/leaf collision, shape and provenance precedence, source ownership, ensurepool and canonical create/inverse/reopen behavior. Genuine lifecycle extraction/downward ratchet; focused actual proof, independent review and landing. No automatic predecessor grant, general graph promotion, generic Delete, proposal rules, GUI/S05/source/standards or manufacturing semantics. Implementer frozen handoff transferred explicitly to appointed manager for landing only. Actual independent final code/behavior PASS; manager accepts bounded existing criteria: four focused tests, four fresh CLI bad-reference refusals without shard/journal mutation, thirteen error/provenance and two source-path controls byte-identical, valid ordinary Create/reopen/query/Undo/Redo. Required full drift exit101 retains thirteen baseline Clippy errors in five unchanged files and later gates unreached; no full qualification PASS. Landed in be7761d130416c31030f2c6db2b374c40345a9c5; bounded task closed. No successor implementation authorization is inferred. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Enforce existing reference integrity in generic manual Part creation and editing** (`PART-JSON-REFERENCE-VALIDATION`; `dat-part-json-reference-validation-rvzg`).
   Owner continuous-development directive; appointed manager explicitly selects reproduced existing Tool1 criterion6 Part default refs resolve repair. Actual current CLI audit: absent/wrong-kind entity/package/default_footprint/default_pin_pad_map accepted by generic Create and Set (16 writes), refused by typed controls (8), later project validate dangling_reference; Undo restores validity. Implement engine-owned shared checks for those existing four fields only, consumed by generic manual Create/Set using canonical same-model mutation. Preserve schema/shape/provenance/source path/root and foreign-leaf collision precedence, optional null refs, unknown fields and canonical history. No default_symbol invention, map completeness policy, library graph promotion, Delete/cascade, GUI/S05, import/source/seed/license/standards or manufacturing semantics. Ordinary valid create/edit/reopen/query/Undo/Redo and byte-exact refusal proof, focused controls and independent review required; broader qualification separate. Final actual independent code/behavior PASS at frozen c79 test/source candidate. Appointed manager accepts unchanged bounded criteria: four focused tests; fresh binary16162536 exact16refusal outcomes and24baseline controls; five exact payload comparisons and20positivecommands. Exact completed harness receipt c31b8d08 proves executed in-memory journal/Part/manifest invariance and no-create-shard assertions; AFTER snapshots were not saved and no independent AFTER filecomparison is claimed. Optional clear/omit and alternate existing Package/Footprint history proven; no alternate Entity reassignment claim. Superseded original passing test version retained, not counted final; updatedversion PASS. Full drift exit101/db4750369a14c70bfc37b5d463603656824c21d8c9670b91822a022d707cbe88 retains13known Clippy errors in5unchangedfiles, latergatesunreached. No global qualification PASS/waiver. Frozen implementer handoff explicitly transferred to manager for landing only; Actual landing 33e2fbd73637c4e9a473dad7dc8b4e293e1f16c5; bounded task closed, lease released. No automatic successor grant. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Enforce existing PinPadMap validation in generic manual creation and editing** (`PIN-PAD-MAP-JSON-ADMISSION`; `dat-pin-pad-map-json-target-validation-szcb`).
   Owner-appointed manager accepts bounded PinPadMap admission delivery against existing Tool1criterion6 under PM059; lib explicitly freezes six source/test paths for root landing. Actual independent corrected source and behavior PASS: engine05976ea5, CLI216626f7, Create6de34db6, Set6a2d6d32, tests6e71ac3a, registration8f20a54a. Twelve unique scoped tests pass (2 corrected regressions +10 existing typed/proposal/legacy); initial2PASS retained superseded. Rebuilt CLI fc0dcb53 ordinary7refusals preserve all project file hashes,27controls exactly preserve baseline errors/provenance and authoritative files; four legacy collision controls retain only baseline-equivalent stage scratch.18positive commands/five payload queries verify Create/Set/reopen/UndoRedo/extensions/omittedtags/null-or-omittedFootprint Packagepads; validate0. Independent documentation/API241 review PASS; owning route406e7b48 reconciled. One full drift exits101 on same13Clippy errors/locations in five unchanged enginefiles, latergatesunreached; no global qualification. Sourcehealth3040, policy13/resource/format/projection/governance/traceability PASS. Existing optional/membership/duplicate/samePackage rules and canonical shape/source/UUID/provenance/history retained; full error chains and raw alias mapping keys repaired after review. No new rule, graph/delete/placed update, dependency, GUI/S05, seed/license/standards/manufacturing acceptance. Actual landing 803e98d7aa71cc25c38f840385d2a36b4d243eca; bounded task closed and landing lease released. Symbol hmjx remains intake until its own explicit selection; no automatic successor grant. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Enforce existing Unit references in generic manual Symbol creation and editing** (`SYMBOL-JSON-UNIT-REFERENCES`; `dat-symbol-json-unit-references-hmjx`).
   Owner continuous-development directive; appointed manager explicitly selects independently reproduced existing Symbol Unit reference admission repair after PinPadMap delivery803e98d7/closure4f90ffcb. Generic manual Symbol JSON Create AND Set accept missing or wrong-kind Package Unit references, four ordinary writes persist dangling data; typed Create two controls reject existing missing pool unit diagnostic. Independent reproduction e996d150 pins fc0dcb53/frozen first-party fixture copies and exact fullstate/query/journal/validate evidence; unrelated archived board schema error disclosed. Fresh valid native CLI project baseline17controls ed839867 and positive10commands/fourpayloadqueries5eaed0d8 reused; seed unchanged. Add engine-owned generic manual Create/Set guards sharing existing pool domain/units-kind lookup, canonical schema/shape/source/root and leaf UUID collision/provenance precedence and same prepared model. Preserve typed/proposal diagnostics, raw JSON/unknown fields, default fields, valid unbound Symbol relink to alternate Unit and canonical revisions/history. Avoid legacy library.rs/proposals.rs edits; no new bound-gate compatibility/pin-anchor/completeness/graph/delete rule, Entity/Package/ModelAttachment scope, import/GUI/S05, source/seed/license/standards or manufacturing adoption. Deliver ordinary fourrefusals/stateinvariance,17baselineerror controls and positiveCreate/Set/query/reopen/UndoRedo, focused proof, independent review and landing; full qualification remains separate. Existing manager selection is explicit for this scope, not automatic successor authority. Manager routine unchanged-scope proof correction: two existing generic Symbol lifecycle tests create dangling Unit fixtures. Extend scope for cohesive extraction of those complete tests from legacy main_tests_project_pool_library.rs931lines into owned Symbol test module, replace fixture links with ordinary UnitCreate and retain names/assertions/history. Root owns exact downward sourceledger ratchet and source-health structural note/parity reconciliation; child remains normal. No production policy/waiver/legacy library.rs/proposals.rs edit; initial focused candidate stays frozen during activecompile and retained as superseded if test ownership changes. PM059 appointed manager accepts bounded existing-rule Symbol delivery after actual independent final code/behavior and documentation/source-health PASS. Frozen enginefc301c9b/Create8080045b/Setbbef197c/parent559f3ef6/child1c6022dd; CLIe6816111. Eight unique current-source tests PASS (4 owned+2 generic+2 proposal); initial2/intermediate4 versions superseded, not counted. Actual afterfd77a530 preserves all files for four exact refusals;17controls match baseline exit/stdout/stderr/authority and four legacy scratch hashes. Positiveb0ceb66a verifies10ordinarycommands/four exact payload queries/Create/Unit relink/UndoRedo/one journal per write/validate0; completed execution receipt a0396291. Parent931 to724 with real nested normal497line test ownership and lawful Unit fixtures retains17prior assertions; sourcehealth3041/policy13/resource/fmt/diff PASS. Actual full drift40027 completed exit101 receipt e2ca85/hash3e17161e retains same13Clippy errors/locations in five unchanged files; later gates unreached, no global qualification PASS. Known GUI parity9vs2 remains separately open. Actual frozen implementer handoff explicitly transfers landing-only ownership to manager; no production edits pending, no automatic successor grant or new product/source/standards/GUI authority. Actual landing df56e8f67d5c7740fd36a15c3717322e2384599a; bounded Symbol delivery closed after independent code/behavior/docs/records PASS and PM059 manager acceptance. Landing-only lease released. Full qualification and unrelated retained Clippy/GUI parity remain open; no automatic successor. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Author native Footprint fabrication graphics through existing CLI actions** (`FOOTPRINT-FAB-GRAPHICS`; `dat-footprint-fab-graphics-pfgg`).
   Owner continuing native-development directive; appointed manager explicitly selects existing manual Footprint fabrication graphics authoring after reviewed Symbol deliverydf56e8f6/closure535b232a. Independent authority assessment confirms PM008105-110 and Tool1 fab data/authoring fields already ratified; existing native fab Vec<Primitive>. Extend same four graphics actions line/rect/circle/polygon with target silkscreen or fab, default silkscreen, visible generic aliases retaining existing command names/argv/diagnostics. Shared engine target/input/build/append owner; one prepared model and canonical Set/commit/journal only. Complete graphics lifecycle extraction from legacy footprint.rs992 into real graphics child and exact downward source-health reconciliation; no forwarding decomposition or cap exception. Preserve existing proposal silk adapters, raw unknown JSON/otherfields/IDs/history and Tool/Assistant direct refusal. Ordinary allfourfab/defaultsilk/alias edits/query/reopen/UndoRedo and invalidtarget/geometry/provenance nonmutation, independent source/runtime/docs review and actual landing required. Reuse actual first-party freshCLI baseline fouroldpositives and16controls46601fd3/binarye6816111. Q5/Q6 graph promotion/placed-impact, GUI/S05, seed/source/license/standards/IPC dimensions/manufacturing adoption remain owner-reserved/open; no new rule/dependency or qualification waiver. Route preflight must finish before specification-derived source or documentation edits; root owns metadata/docs and digest reconciliation. Owning route preflight complete before implementation: independent review honestly reuses actual complete prior S03 research/IPC and subsequent S03/S04/combined/manager consumer reviews; independently verifies all13 parametric route members byte-identical803e98d7, digest17b352ee08dcd25822b2b64d349e38369000cf5e01746372ebb320d72cc62c8b. Focused current PM008/Tool1/schema review confirms existing fab target; no fresh standards corpus or13freshfullread claim. Foundation23members actual unchanged from reviewed Symbol landingdf56e8f6. Root accepts bounded authority/scope review; implementation edits released under current grant. Only factual Tool1 runtime baseline after actual proof, ENGINE/API/source-health debt rows as measured; research/IPC/GUI/history receipts untouched. Existing proposal silk semantics and Q5/Q6/S05 remain open. Actual implementer frozen seven-source handoff transferred to manager for landing-only; independent final source/behavior and docs PASS. Eleven scopedtests3new+8old; freshCLI34d87f86 ordinary16writes/fourprofiles/exactpayloadrevision/4UndoRedo/reopen/validate0 and50fullfileinvariantrefusals30exactlegacy+20Fab, hashes47907643/a3e0ccd0/currentactualreceipt75b6a77e. One-journal checks are executed assertions, not independent savedjournal comparisons; malformedcollection onlyhelperproof. Actualfull8639exit101/chunk3cd98d/ceb25b38 same13Clippy messageslocations/fiveHEADunchangedfiles; latergatesunreached/fullqualificationFAIL retained. FailedsupplementBEFOREharness5cf11ef3 retained+canonicalUndo. Routine proof correction fills declared standalonePackage/FootprintCreate item oncurrentbinary, withoutsource/build/testrerun. Extend rootrecordscope for durable ordinaryapplication feature-delivery record after actualsourcecommit; development_ready staysfalse untilalldeclaredordinaryevidence sourcepinned. No newproductrule/waiver/S05/Q5Q6/source/standards/dependency grant. Final supplemental independent review PASS: fresh-create results585471bf and original completed execution79920/d70f25dd demonstrate12successful ordinarycommands, named Package/unplaced Footprint creation, exactFabLine/query/UndoRedo/reopen/validation and13protected file hashes unchanged. Earlier creation-proof pending checkpoint is superseded. Appointed manager accepts bounded existing-contract delivery against all declared workflow and safety criteria; durable source-pinned availability record follows actual source landing. FullqualificationFAIL and owner-reserved limits retained. Actual source landing 5300ec192c274456168a2d3e0df34dfe7793c5d7; all declared workflow and safety results source-pinned in docs/reviews/workflow-delivery-rollout/footprint-fab-graphics-delivery.json. Bounded development availability accepted; landing-only lease released and issue closure synchronized. Full qualification remains FAIL separately; no automatic successor. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Edit native Symbol definition fields through the existing Set action** (`SYMBOL-DEFINITION-FIELDS`; `dat-symbol-definition-fields-qycq`).
   Owner-appointed manager explicitly selects this bounded existing PM008105-108/190-213 and Tool1 manual Symbol field authoring scope after actual Fab source5300ec19 and source-pinned independently reviewed closure1590b1ce. Optional --section fields on existing set-pool-library-object, symbols only; omitted flag preserves full JSON behavior and precedence. Validate existing LibrarySymbolField shape but persist original raw ordered array, duplicate field-key values, unknown row metadata and absent defaults. Materialize raw Symbol in same fresh model, replace fields only and use existing source/reference admission and canonical Set/history. No new uniqueness/upsert/patch/default policy. Extract full SetArgs.run lifecycle from library.rs2537 into existing object_edit owner; extract cohesive generic object argument types from project_library.rs976 into normal project_library_object owner with actual module registration/reexports, exact downward ledger ratchets, no exception or forwarding decomposition. Focused shared engine helper belongs native Symbol authority; avoid legacy engine library.rs. Actual independent authority and complete route reuse assessment PASS: parametric13members12unchanged vs reviewed Symbol source, only delivered Fab Tool1 delta reviewed; foundation23members21unchanged with complete ENGINE/source-health deltas reviewed; manual-workflow4unchanged. Reuse adequate complete prior sources/consumers reviews, not39freshreads or new standards corpus. Root owns docs/metadata/ledger/parity/digests and acceptance; lib owns claimed production/test edits. Ordinary designer add/change/remove/clear/query/raw-sibling/history/reopen and nonmutating refusal proof required with actual independent source/runtime review and landing. Reuse adequate old evidence; do not repeatedly build for admin changes. No production PASS claimed yet, no new dependency/waiver, no Q5Q6/S05/placed/GUI/MCP/import/seed/standards grant. Implementer explicitly confirms frozen seven-source landing-only transfer to appointed manager; no outstanding writes/builds. Actual final source and behavior independent PASS, docs/ownership final PASS; manager accepts bounded existing-contract delivery. Six unique scopedtests2new+4existing PASS after retaining two fixture-only failed attempts101; finaltesta12f3987/224, sixproductionunchanged. FreshCLI9a7f67b3, actualordinary20730/abda3689 covers63commands40positive23refusal, allfourfieldarrays exactpayload/revision/4UndoRedoqueries/freshUnitSymbol/reopen/validate0. Required22refusals fullprojectunchanged; additionalforeignUUIDmetadata refusal preservesauthored/journal with one recorded canonicalstagefile, no scratch cleanup/newrule. Actualcompat65809/7343c0bc21legacyoutcomes byte-exact with4oldstagepayload hashes preserved. Executedjournal/unplaced assertions only, no independent savedjournalcomparison. Actualfull7577exit101/aa226189 and updatedexecution9d89db3f retains exact13Clippydiagnosticlocations/fiveHEADunchangedfiles/latergatesunreached; no globalPASS. Exactparent976→932/2537→2511 genuineArgs/Setownership, health3045/policy13PASS; API246/debt68twoownedrows. RootPROGRESS historicalCLI condensation699→692 reviewedretains8commands/exactformat-only/exit limits, generatedprojectionunchanged. Development availability remainsfalse until actualsourcecommit and source-pinned ordinaryrecord; no successorgrant. Actual source landing 0e66feecfe9cd260ac6e01af6d2f3634c6c5fca4; source-pinned ordinary record docs/reviews/workflow-delivery-rollout/symbol-definition-fields-delivery.json establishes bounded development availability after independent reviews. Landing-only claim released; close issue with exact landing. Fullqualification remains separateFAIL; no automatic successor. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Correct and remove native Symbol graphics through the existing Set action** (`SYMBOL-DRAWING-EDIT`; `dat-symbol-drawing-edit-dxc9`).
   Owner-appointed manager explicitly selects existing PM008107 and Tool1 manual Symbol graphics editing after actual source0e66feec and independently reviewed source-pinned fields closure4201dece. Current six append verbs support creation but targeted correction/removal requires wholeJSON workaround. Extend existing Symbol Set --section with drawings; same one fresh raw Symbol/model/source/reference admission and canonical Set/history. Admit existing Vec<Primitive> sixserde shapes and persist ORIGINALorderedarray, extensions, repeatedgraphics and absentdefaults. Preserve current fields helper signature/diagnostic/order through private shared typedarray/rawmerge owner and add focused drawings helper; no extra writer or duplicated CLIlifecycle. Preserve generic wholeJSON numeric/schema semantics, do not add typedappend positivity/texttrim/blanktext rules, primitiveUUID/indexselection/upsert/geometrythreshold/defaultnormalization. Current normal owners Argschild56/object_edit135/engineDefinition644 and newnormaltests; leave legacy932/2511 parents untouched. Independent exactauthority/route assessment PASS: allcurrentmembers match reviewed fields source0e66feec; versusFab5300ec19 parametric12unchanged/onecompleteTool1delta, foundation21unchanged/twoENGINE/sourcehealthdeltas reviewed; manualfoundation4unchanged. Reuse adequate complete prior sources/consumers review with actualbytecomparisons, no fresh wholecorpus claim. Root metadata/docs/parity/digests and acceptance; lib claimedproduction/testscope. Deliver fresh ordinary sixgraphics/correct-remove-reorder-clear/rawsibling/query/UndoRedo/reopen and shape/kind/section/writerrefusal invariance; existingfields/fullJSON/appendcompatible, actual independent source/runtime/docsreview and landing. Reuse adequate proof, one justified serial guardedbatch, retain fullqualification failures; no new dependency/waiver/productpolicy, Q5Q6/S05/placed/MCP/GUI/import/seedrights/standards/manufacturing grant. Availability false until actual delivery. Actual implementer frozen five-source landing-only handoff accepted by appointed manager, no outstanding Cargo or mutations. Independent actual source/behavior review PASS:5unique tests, ordinary85commands56positive/6oldappendrefusals/23new with22fullprojectinvariant and extraUUIDcanonical scratch/authjournal invariant; four exact payload/revision/UndoRedo pairs, six shape-only wholeJSON equivalence, generic21byteexact and fields63commands retained with explicit path/UUID diagnostic rebasing and sectionchoices expansion. Final execution6bd6f47d and full242cc407 actual101 retain same13diagnostics/fiveunchangedfiles/laterunreached; no globalPASS. Independent final Tool1/ENGINE engineering and PROGRESS695-to666 condensation PASS. PM059 manager accepts bounded existing-contract delivery; source-pinned availability follows actual landing. No automatic successor. Actual source5e682e93 landed; independently reviewed source-pinned ordinary delivery record95cd1300 establishes bounded development availability. Manager records actual record PASS and closes issue, releases landing-only lease. Full qualification FAIL and owner-reserved scope retained; no automatic successor. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Correct and remove native Footprint silkscreen and fabrication graphics** (`FOOTPRINT-GRAPHIC-EDIT`; `dat-footprint-graphic-edit-f12q`).
   Owner-appointed manager explicitly selects existing PM008105-110/Tool1 Footprint graphic editing after actual Symbol drawings5e682e93 and independently reviewed source-pinned closure74133df3. Independent current authority assessment PASS; all route members match actually reviewed landed corpus, Tool163969136/ENGINEf7ce8f48 deltas fully reviewed. Reuse adequate complete unchanged sources/consumer review, no fresh standards corpus claim. Add silkscreen/fab sections to existing footprints Set. Share internal typed-array admission/raw replacement with existing Symbol wrappers, preserving exact Symbol diagnostics and raw sixPrimitive genericwholeJSON semantics. Restrict Footprint section via existing target enum; same fresh model/source/reference admission/canonical Set/history. Preserve pads/Package/court/process/models/othercollection/IDs/extensions. No arbitrary patch/upsert/graphicID/positivity/texttrim/normalization/threshold/ArcTextproducer/renderer grant. Normal owners Args56/edit144/Definition660/Footprint360 plus newnormaltest; no legacy monolith touches. Root docs/metadata/acceptance, lib production/tests. Actual ordinary fresh native setup/fourappendshapes bothcollections/correct-remove-reorder-clear/query/rawsiblings/revisions/UndoRedo/reopen/refusal invariance and oldSymbol/fullJSON/append compatibility required, actual independent review and landing. Reuse adequate proof, serial guarded builds only; retain current fullqualification failures, no retry-until-green. No owner-reserved Q5Q6/S05/GUI/MCP/proposal/placed/source/standards/seedrights/dependency rule or waiver. This is explicit bounded successor authorization, not automatic from closure. Implementer explicitly transfers frozen six-source candidate landing-only ownership to appointed manager; no outstanding writes/Cargo. Actual independent final source/runtime/docs PASS; PM059 manager accepts bounded existing-contract delivery. Eight unique tests PASS/no failed attempts; current ordinary159commands104positive/8oldappendrefusals/47new with45fullprojectinvariant+2UUIDcanonical scratch/authjournal invariant;8exactedits/revisions/history and23command both-nonempty directional sibling supplement. Symbol85/fields63 currentreplays+46diagnostics explicitly path/UUIDrebased with intentional4sectionchoices expansion; generic21byteexact. Finalexecutiond95ab2f2 and full23c86766 actual101 retain13exactClippy messages/locations/fiveunchangedfiles/laterunreached. EngineeringAPI248/33798029 normalowners/sourcehealth3047, Tool1a3cbc6f9/ENGINEfbd0409b independentPASS; deliberate reviewedroute refresh only. Source-pinned availability follows actual landing, no fullqualification PASS or automatic successor. Actual source88d5e68c and independently reviewed source-pinned record26fa4127 establish bounded development availability. Final record review corrected only diagnostic event count:two affected comparison rows with supported values expanding two-to-four. Manager receives actual final record PASS, closes issue/releases landing-only lease. All qualification failures/owner-reserved boundaries remain; no automatic successor. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Edit native Padstack copper apertures through existing Set** (`PADSTACK-APERTURE-EDIT`; `dat-padstack-aperture-edit-2dwa`).
   Owner-appointed manager explicitly selects existing PM008105-110/280-307 and Tool1 manual Padstack copper editing after actual Footprint source88d5e68c and independently reviewed source-pinned closuredda829b9. Actual current authority audit PASS; parametric13members match reviewed Tool1a3cbc6f9 plus12unchanged complete corpus, ENGINEfbd0409b actual sharedarray delta reviewed. Reuse adequate exact complete reviews, no fresh standards corpus claim. Current typed Create only plus wholeJSON Set workaround; add optional aperture section to existing padstacks Set. Admit current Option<PadstackAperture> circle/rect/null with i64dimensions, retain raw originalvalue/extensions and replace only aperture on one fresh rawPadstack/model/canonicalSet/history. Share generic internal typed-value/rawfield admission with existing array wrappers; preserve old Symbol/Footprint wrappers/signatures/errors and fullJSON ordering. Explicit null is current Option representation, no autogenerated default or fielddeletionpolicy. Preserve name/drill/plating/span/maskpaste/annular/thermal/antipad/unknownmetadata. Typed Create positivity/drill rules unchanged; section admits existing wholeJSON numeric/schema semantics, not new standards/annular/drillcomparisonpolicy. Normal Args56/edit155/Definition661 plus newnormaltest/modulehook, no legacy monolith edit or exception. Deliver fresh unplaced resize/change/clear/restore/query/UndoRedo/reopen and subsequent newFootprintpad reference to edited sameUUID; no placed impact claim. Actual malformed/kind/section/missing/source refusal authjournal invariance and oldwholeJSON/Create/sections compatibility; disclose canonical scratch separately. Root docs/metadata/acceptance, lib claimedsource/tests, serial guardedproof and actual independent source/runtime/docsreview/landing. No new graph/Q5Q6/S05/GUI/MCP/proposal/perlayercopper/process/standards/seedrights/source/dependency rule or waiver. Explicit bounded successor authorization, not automatic from closure. Actual explicit lib-agent landing-only transfer of five frozen dirty source/test paths to appointed manager; freeze11469ee5 and final execution7eb2e50c retained. Root owns docs and landing; actual independent final runtime/docs review remains pending. No further compilation for administrative changes. PM059 owner-appointed admin manager session01a122e5-f602-7f61-a144-0d1ce7116a77 accepts exact frozen5source11469ee5/actualexecution7eb2e50c candidate after distinct management_review final source/behavior PASS and code_audit Tool1 factual delta PASS. Exact six ordinary workflows/five safety criteria met with8unique tests,101-command aperture ordinary proof and all affected compatibility profiles. Actual twelve completion records retain wrong-package101 and full101/13unchangedClippy/latergatesunreached. Saved journal entries independently compared; increment/unplaced assertions remain script-backed. No global/GUI/HUMAN/placed/standards/manufacturing qualification. Availability/source pin and closure remain next after landing. Landed510fe31ef2316cd793af0e57f29a7959d3739362; distinct independent tracked150line delivery record c259af0f review PASS with all17hashes/fivecommitpins/fourpayload+persistedjournal hashes/protected13 map verified. Manager records source-pinned development availability and closes exact bounded unit; retained fullqualificationFAIL and all owner-reserved boundaries unchanged. Lease released by explicit completed landing disposition. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Edit and remove native Footprint pads through existing Set** (`FOOTPRINT-PADS-EDIT`; `dat-footprint-pads-edit-lrm2`).
   Owner-appointed manager explicitly selects bounded existing PM008105-110/246-277 and Tool1 manual Footprint pads correction after actual Padstack source510fe31e, independently reviewed source-pinned deliveryc259af0f and reconciled closure39b52f39. Distinct actual code audit and lib read-only current schema/seam audit agree: onepad Set already moves/rebinds/upserts; removal/clear currently require fullJSON. Add pads section to existing footprints Set, current HashMap<Uuid,Pad> temporary typed admission then original raw map replacement, empty object clears. Focused footprint_with_pads alongside current Definition helpers reuses private typed-value seam; no arbitrary public patch, new writer or mutation class. Same fresh model, Footprint Package/allPadstack reference admission and canonical source/UUID guard/Set/history. Preserve current mapkey/rowUUID, duplicate name, numeric/layer/text wholeJSON semantics and absent defaults/extensions; onepad scalar checks separate. Fresh unplaced/unbound add/edit/remove/clear with stable retained IDs and nonempty graphics/courtyard/process/model siblings, exact payload/revisions/persisted history/UndoRedo/reopen/validate and continued onepad edit. Required malformed/missing/wrong reference/kind/section/source refusals preserve authored/journal; qualify canonical scratch separately. Normal Definition680 must remain under700; normal focused test/modulehook; root docs/metadata and lib source/tests13scope. Actual independent source/runtime/docs/record review, serial guarded proof and source-pinned delivery required; reuse adequate prior corpus/proof with exact changed deltas. No PinPadMap or placed impact promise; Q5Q6/S05/GUI/MCP/proposal/source/import/seed rights/standards/manufacturing/dependencies/unique names/new identity or numeric rules. Explicit retained scope execution selection, not automatic successor from closure. Actual explicit landing-only transfer of five frozen dirty source/test paths to appointed manager; freeze213740e7/executionf847523b retained. PM059 owner-appointed manager session01a122e5-f602-7f61-a144-0d1ce7116a77 accepts bounded candidate after distinct management final source/behavior PASS and codeaudit Tool1 factual PASS, meeting exactfiveworkflow/fivesafety. Sixunique tests/109commands/5persistedcanonicaljournalpairs/all5affectedcompatprofiles verified; elevenoriginalcompletions10zero/full25715exit101 retains13unchangedClippy/laterunreached. Onepadcontrols4byteexact1explicitUUIDrebase and canonicalstagescratch qualified; Packagecontrols reused at unchangedhelper, not freshsectionclaim. No global/placed/GUI/HUMAN/source/standards/manufacturing qualification. Root landing/docs; availability/sourcepin remains aftercommit. Landed2c34dac0195688dc686f372dce5de29ed2971a67; distinct tracked154line delivery record6f05a955 independentPASS with all15hashes/5commitpins/5payload+persistedjournal hashes/protected13map verified. PM059 manager records source-pinned development availability, closes bounded task and explicitly releases completed lease; fullqualificationFAIL/ownerreserved boundaries retained. Read-only materialprocess feasibility audit separately exposes policy→frozen/materialization/DRC/CAM gap and owner-reserved inheritance/source/profile/Q4Q5 choices; no implementation successor inferred. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`.
- **Deliver effective native Padstack mask and paste geometry** (`EFFECTIVE-PADSTACK-PROCESS`; `dat-effective-padstack-process-vgno`).
   PM061 bounded native process feature landed in ef32fbe1: manual authoring/effective inspection, frozen placement/transform/update/history, separately adopted capability and shared check/render/supported CAM. Actual ordinary workflows and eight two-root scene/CAM comparisons passed distinct review; documentation and source-health gates passed. Availability source pins and qualification limits recorded in effective-process-owner-delegation.json. Issue closed and ownership explicitly released. Standards/seed rights, unsupported CAM, HUMAN/S05, existing global Clippy/GUI parity and full qualification remain separate. No successor authorization. *state `landed`; authorization `none`; parallel lane.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_008_LIBRARY_COMPONENT_SYSTEM.md`, `docs/decisions/PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md`, `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/reviews/workflow-delivery-rollout/effective-process-owner-delegation.json`, `docs/decisions/PRODUCT_MECHANICS_061_EFFECTIVE_PADSTACK_PROCESS_GEOMETRY.md`.
- **Deliver native library browser and Footprint Padstack GUI authoring** (`LIBRARY-GUI-AUTHORING`; `dat-library-gui-authoring-1ix7`).
   PM062 bounded named eight-kind library browser and manual Footprint/Padstack authoring landed61b5b409. Actual source/runtime/rendered/documentation and closure reviews accepted by owner-appointed PM059 manager; landed50 pins and56 artifacts verified. Source-pinned development availability recorded; all completion steps complete, delivery issue closed and source ownership explicitly released. Full targets/qualification and separate baseline failures retained; no successor authorization. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_062_SCHEMATIC_LIBRARY_SURFACE_DISPOSITION.md`, `docs/reviews/workflow-delivery-rollout/surface-s05-manager-disposition.json`, `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`.
- **Complete owner-directed repository maintenance** (`REPOSITORY-MAINTENANCE`; `dat-repository-maintenance-2e18`).
   Owner-directed five-priority maintenance delivered. Temporary pilot fixture and parity/coverage checks repaired; raw proof retired and prevented; released local output/retired backups pruned. Owner-approved exact history replacement published and locally installed with verified source map, all historical-source tags and preserved rollback mirror. Local Git pack48.95MiB; fresh GitHub clone57.02MiB. All steps complete; no S5A resumption or successor authorization. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/maintenance-records.md`.
- **Complete five owner-directed repository organization improvements** (`REPOSITORY-ORGANIZATION`; `dat-repository-organization-t83n`).
   Owner-selected five organization improvements. Reconcile docs, compact unnecessary review JSON, group GUI source, deduplicate verification and extract cohesive GUI protocol debt with exact ratchets. S5A remains paused; pre-maintenance selector will be restored with its unchanged grant at closure. No product, dependency, acceptance or visual-authority changes. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md`, `docs/reviews/specification-history/maintenance-records.md`.
- **Complete five additional owner-directed cleanup improvements** (`ADDITIONAL-CLEANUP`; `dat-additional-cleanup-1p3q`).
   Owner authorized five further improvements: released local output pruning, external history rollback preservation, retention enforcement, reviewed raw evidence compaction and selective registered source decomposition. S5A remains paused and its unchanged pointer/grant will be restored at closure. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md`, `docs/reviews/specification-history/maintenance-records.md`.
- **Complete five bounded specification and fixture workspace improvements** (`SPEC-WORKSPACE-CLEANUP`; `dat-spec-workspace-cleanup-o0od`).
   All five owner-selected bounded improvements landed with independent review: existing specification ownership, preserved historical program records, coherent inherited authority, reversible external fixture preservation and external managed fixture generation. Original S5A record/grant restored exactly without a claim; working sessions remain paused. No product or qualification authority changes. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md`, `docs/reviews/specification-history/maintenance-records.md`.
- **Audit documentation fragmentation and retire redundant shards** (`DOCUMENTATION-CONSOLIDATION`; `dat-documentation-consolidation-bznj`).
   Repository-wide structural inventory and first four verified retirements landed with independent review; semantic documentation reconciliation remains open as intake dat-documentation-semantic-reconciliation-olnn. Current authority/approvals and fixture data preserved. Original paused S5A task/grant restored exactly without claim or resumption. No full documentation-cleanup or qualification PASS. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/maintenance-records.md`.
- **Separate native workspaces from disposable Cargo output** (`TARGET-WORKSPACE-SEPARATION`; `dat-target-workspace-separation-1v8o`).
   All four owner-selected cleanup tasks delivered in b3920995: Cargo-only target, 33 preserved external collections, six released temporary collections retired, reviewed recovery and bounded verification. Original S5A task/grant restored; sessions remain paused and full qualification remains open. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/maintenance-records.md`.
- **Organize review evidence and retire obsolete development records** (`ORGANIZATION-EVIDENCE-CLEANUP`; `dat-organization-evidence-cleanup-kd9d`).
   Owner-selected organization/evidence cleanup delivered in 6a973551: exact duplicate proof retired, historical planning consolidated, two native delivery groups preserved externally with scoped recovery, disposable caches pruned, current executables retained and independent review passed. Original paused S5A task/grant restored exactly. Broader semantic consolidation remains open intake. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/maintenance-records.md`.
- **Clarify documentation ownership and externalize remaining audit test output** (`DOCUMENTATION-OWNERSHIP-ORGANIZATION`; `dat-documentation-ownership-organization-rhm7`).
   All three owner-selected organization improvements delivered in 73b4be15: one documentation navigation map; duplicate engine schema/API and IR version-support wording delegate to existing owners; eleven audit test producers use shared external scratch. All32 affected tests and independent review pass; old audit inputs preserved and released compiler/scratch output pruned. Original paused S5A task/grant restored exactly; broader semantic intake remains open. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/maintenance-records.md`, `docs/README.md`, `docs/CANONICAL_IR.md`, `docs/ENGINE_DESIGN.md`.
- **Reconcile checking rationale, historical proposals and portable tooling references** (`CHECKING-TOOLING-DOCUMENTATION`; `dat-checking-tooling-doc-organization-96qj`).
   All four owner-selected documentation improvements delivered in 5fe2c4ba: checking companions delegate owned contracts, historical proposals separate from adopted authority, existing tools navigable, research instructions and107 machine-specific links repaired. Focused checks and independent review pass; four evidence routes reconciled at reviewed scopes. Original paused S5A task/grant restored exactly; remaining semantic intake stays open. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/maintenance-records.md`, `docs/README.md`, `docs/ERC_SPEC.md`, `docs/SCHEMATIC_CONNECTIVITY_SPEC.md`, `docs/LICENSING.md`.
- **Reconcile specification governance and separate active documentation from history** (`SPECIFICATION-GOVERNANCE-RECONCILIATION`; `dat-deep-governance-reconciliation-grpa`).
   Owner-selected five-item deep organization pass. Preserve semantic authority and original paused S5A; explicit decisions remain unresolved until actual disposition. Only justified presentation/procedure/reference reconciliation; no new product rules. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/governance-reconciliation.md`.
- **Align native product delivery through the bounded initial reorganization** (`REORGANIZATION-INITIAL-CHECKPOINT`; `dat-native-delivery-alignment-1vy6`).
   Owner-appointed reorganization manager selects initial RGO-01..05 planning checkpoint. Correct named stale baselines, map twenty unfinished tasks, prepare honest first-workflow contract and owner decisions. Preserve paused S5A; no runtime, product/rights/dependency/waiver grant. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/native-delivery-reorganization.md`.
- **Reconcile bounded specification ownership, delivery history and direction maps** (`SPECIFICATION-REORGANIZATION`; `dat-specification-reorganization-vqot`).
   Owner-selected RSP-01..04 bounded specification cleanup accepted by the appointed manager after actual distinct source-review PASS and focused checks at f35cdee0. Exact history/authority and current direction maps reconciled; broader intake remains open. Original paused/unclaimed S5A record restored exactly; no product or successor execution. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/specification-reorganization.md`.
- **Correct bounded retired enforcement, proof storage and import identity documentation** (`SPECIFICATION-RESIDUAL-CLEANUP`; `dat-specification-residual-cleanup-vjub`).
   Actual owner-selected RC01-03 accepted after two distinct reviewed/landed batches be576a8b and6e26fe8b. Three retired-WDQ references, evidence-storage ownership and imported seed/mapping scope corrected with all algorithms/proof duties preserved. Final bounded cleanup closes; original paused/unclaimed S5A restored exactly. Broad intake open; no optional/product/successor work. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/specification-reorganization.md`.
- **Clarify four historical authority and portable verification navigation gaps** (`SPECIFICATION-NAVIGATION-CLEANUP`; `dat-specification-navigation-cleanup-aiud`).
   All four owner-selected navigation corrections are accepted at 8b740e00 after actual independent v2 review and manager verification. Six exact spans in five documents preserve original authority and requirements. The bounded cleanup closes; all 95 prior task records and paused S5A are restored exactly. Broad intake remains open. No product work or successor is authorized. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/specification-reorganization.md`.
- **Consolidate accepted cleanup and precisely close the assessed reorganization phase** (`SPECIFICATION-CLEANUP-CLOSEOUT`; `dat-specification-cleanup-closeout-oisy`).
   All four closeout deliverables and bounded phase exit accepted after actual independent review and verified content landing 0810f0c3. Current summary records exact retained limits and existing product owners. All 96 prior task records and paused S5A restored exactly; only bounded issue closes. Broad intake remains open/unselected. No product or successor work authorized. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/specification-reorganization.md`.
- **Repair five production-process authority and state defects** (`PRODUCTION-PROCESS-CORRECTION`; `dat-production-process-correction-enxp`).
   Five owner-authorized process corrections accepted after actual independent v2 PASS and exact local content landing 11afd4b5. PPC-R1–R3 resolved. Hold, purpose, explicit idle, scoped decision routing and normalized claim overlap are enforced. Bounded issue closes and claim releases; all original product grants and qualification remain. S5A held/internal C04 retained. Explicit idle selects no successor or product work. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/specification-reorganization.md`.
- **Correct the three audit findings and verify Symbol Move transport behavior** (`CORRECT-AUDIT-FINDINGS`; `dat-private-writer-journal-gate-0u2x`).
   AUDIT-01/02/03 corrected and QUESTION-01 verified at independently reviewed content8948efe9 with actual manager landing/cleanup acceptance. PWO bounded credit reconciled without closing broader obligations; journal guard repaired with negative controls; duplicate discovery intake consolidated. Verified Symbol Move gap616t stays open/unselected. Claim released and explicit idle restored; S5A hold and all product/qualification boundaries remain. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/specification-reorganization.md`, `docs/decisions/PRODUCT_MECHANICS_060_PROJECT_WRITER_OWNERSHIP.md`, `specs/FOUNDATION_FOLLOWUP_PLANS.md`.
- **Replace required external replay inputs and retire obsolete collections** (`REPRODUCIBLE-REPLAY-INPUTS`; `dat-reproducible-replay-inputs-g2mc`).
   Standalone exact replay inputs landed at2b7fe4ef with independent bounded source review and manager acceptance; all54 retired collections,26external metadata files and released task scratch removed after reviewed safe checks. Manager verified protected data and757-file provisioning after removal. Explicit idle; no product/S5A qualification or successor. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `CLAUDE.md`, `AGENTS.md`, `docs/PROJECT_STATE_POLICY.md`, `docs/reviews/specification-history/maintenance-records.md`.
- **Create Packages through the native library GUI** (`LIBRARY-PACKAGE-GUI-CREATE`; `dat-library-package-gui-create-2pdx`).
   Creation-only Package source remains landed at1459e21f with its bounded creation/history/Footprint review. NPD-C07 startup repair at0a8e8b5d now verifies ordinary GUI-created Project-to-Package-draft reachability on native Wayland and restores bounded development availability. No new full Package, platform or HUMAN qualification; S5A held. *state `landed`; authorization `none`.*
   *Dependencies:* none. *Unblocks:* none. *Governing:* `docs/decisions/PRODUCT_MECHANICS_059_DELEGATED_FEATURE_DELIVERY.md`, `docs/decisions/PRODUCT_MECHANICS_062_SCHEMATIC_LIBRARY_SURFACE_DISPOSITION.md`, `docs/contracts/LIBRARY_AUTHORING_TOOL_CONTRACT.md`, `docs/gui/DATUM_GUI_PRODUCT_SPEC.md`, `docs/reviews/workflow-delivery-rollout/package-gui-creation.json`.

</details>

<!-- ACTIVE FRONTIER:END -->

### Workflow repair admission transaction

Admission and repair evidence are historical; current selection and authorization come from the Active Frontier. [Detailed evidence](progress/workflow_governance_details.md#workflow-repair-admission-transaction).

## Native product outcomes

[Delivered foundations and twenty-task outcome map](progress/production_foundation_details.md#native-product-outcomes--initial-reorganization) connects library → schematic → PCB → manufacturing to existing owners and actual gaps. It is navigation, not selection, execution authority or full qualification.

## WORKFLOW-DELIVERY-QUALITY — workflow evidence and readiness adoption

### WORKFLOW-DELIVERY-IMPLEMENTATION — prepared infrastructure contract

Eleven infrastructure milestones were completed and owner-accepted; subsequent repairs remain separately tracked. [Detailed evidence](progress/workflow_governance_details.md#workflow-delivery-implementation--prepared-infrastructure-contract).

## FOUNDATION-WORKFLOW-SPEC — bounded pre-implementation foundation contract

Foundation contract preparation is distinct from native product delivery and qualification. [Detailed evidence](progress/workflow_governance_details.md#foundation-workflow-spec--bounded-pre-implementation-foundation-contract).

## FOUNDATION-CONSUMER-COMPLETION — existing lane delivery contracts

Existing lane contracts retain their documented delivery and owner-acceptance boundaries. [Detailed evidence](progress/workflow_governance_details.md#foundation-consumer-completion--existing-lane-delivery-contracts).

## WORKFLOW-DELIVERY-ROLLOUT — broader enforcement and adoption preparation

Broader adoption evidence does not substitute for product-lane delivery or qualification. [Detailed evidence](progress/workflow_governance_details.md#workflow-delivery-rollout--broader-enforcement-and-adoption-preparation).

## GUI-PERFORMANCE-SPEC — performance-first shared GUI specification

Specification preparation is owner-ratified. Current-host horizontal/vertical resize investigation awaits execution disposition; S4 is closed, S5/GBB-P05 remain separate, and budgets are undecided. [Detailed evidence](progress/workflow_governance_details.md#gui-performance-spec--performance-first-shared-gui-specification).

## Spec Governance Coverage

Checkpoint evidence and retained boundaries remain in the linked detail; current selection comes from the Active Frontier. [Detailed evidence](progress/workflow_governance_details.md#spec-governance-coverage).

## Scope Integration / Substrate Readiness

Promote target to current evidence only with implementation, verification and applicable CLI/MCP/API anchors.
Imported-board fidelity additionally requires its substrate contracts or an explicit governance exemption.

### Library Foundation Investigation — Repository-Owned Planning

SURFACE-S01–S04 and the distinct combined planning review are accepted. SURFACE-S05 owner disposition remains pending; synthetic studies and fixtures establish neither production-library approval nor native visual delivery. [Detailed evidence](progress/workflow_governance_details.md#library-foundation-investigation--repository-owned-planning).

### Native File-Type Catalog — Owner-Requested Proposal

Current native loading uses manifest/path context and `.json`. Proposed file types remain pending owner disposition. [Detailed evidence](progress/workflow_governance_details.md#native-file-type-catalog--owner-requested-proposal).

### Native Write Facade — Write-Surface Convergence (2026-07-04, COMPLETE)

Facade convergence is recorded as complete for its bounded slice; all consumers still share the canonical commit/journal path. [Detailed evidence](progress/workflow_governance_details.md#native-write-facade--write-surface-convergence-2026-07-04-complete).

### Single-Source Verb Registry (Decision 017)

GUI terminal catalog is registry-derived; CLI clap and daemon dispatch remain target projections. [Detailed evidence](progress/workflow_governance_details.md#single-source-verb-registry-decision-017).

### Production Goals — Current / Target Ledger

| Capability | Current checkpoint | Remaining boundary |
|---|---|---|
| Pin electrical taxonomy | [~] Library-owned Input, Output, Bidirectional, Passive, PowerIn, PowerOut, OpenCollector, OpenEmitter, TriState, NoConnect | Complete consumer and evidence adoption |
| Symbol schema and pin style | [~] Typed fields, pin anchors, provenance and standards basis | Full rendering/check/import/export coverage |
| Gate-aware PinPadMap | [~] First-class maps and typed authoring; legacy Part.pad_map is compatibility input | Complete migration and consumer authority |
| Padstack process policy | [~] Aperture, drill, layer, mask/paste and thermal fields | Complete materialization/check/export consumption |
| LibraryGraph | [~] Registration, dependency/duplicate diagnostics and validation reports | Unified resolution, layering and materialization policy |
| Model attachments | [~] Part behavioural models carry hashes, provenance and review state | Unified cross-target roles and lifecycle |
| Schematic binding | [~] Journaled placement and revisioned Entity/Gate/Part binding evidence | Broader editor and production-library qualification |
| Production artifacts | [~] Canonical run evidence, artifact inspection and bounded GUI preview | Richer family viewers and complete GUI drill-down |

Foundation and production capabilities remain partial where the detailed ledger identifies missing consumers, broader semantics or qualification. This ledger never selects tasks. [Production foundation and public surfaces](progress/production_foundation_details.md); [Terminal and GUI integration history](progress/terminal_integration_details.md); [Source context, checks and production artifacts](progress/production_checks_artifacts_details.md).

## Current Repo Health

Retained health results are dated evidence, not a fresh whole-workspace PASS. [Detailed evidence](progress/repository_health_details.md#current-repo-health).

### Drift RCA + Prevention (2026-03-25)

Dated drift findings and prevention work are retained without implying current qualification. [Detailed evidence](progress/repository_health_details.md#drift-rca--prevention-2026-03-25).

## Legacy Implementation Evidence

Legacy coverage below records scoped implementation evidence; current scheduling comes only from the generated Frontier. [Detailed evidence](progress/repository_health_details.md#legacy-implementation-evidence).

Original milestone requirements are preserved in [program history](../docs/reviews/specification-history/program-milestones.md); their recorded statuses below do not select or authorize current work.

## PROGRAM_SPEC.md — M0 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Pool types: Unit, Pin, Entity, Gate, Package, Pad, Padstack, Part, Symbol | [x] | `pool/mod.rs` |
| Pool serialization round-trip (100% of types) | [x] | serde derive + golden tests |
| Pool SQLite index: create, insert, keyword search, parametric search | [x] | `pool/mod.rs` PoolIndex |
| Deterministic serialization (byte-identical 3 runs) | [x] | `ir/serialization.rs` |
| Eagle .lbr import (20 libraries, 0 errors) | [x] | `import/eagle/mod.rs` |
| Eagle canonicalization round-trip | [x] | import → pool JSON → deserialize |
| Eagle deterministic re-import (identical UUIDs) | [x] | UUID v5 with eagle namespace |
| RuleScope IR compiles, serializes, leaf evaluator works | [x] | `rules/ast.rs`, `rules/eval.rs` |
| UUID v5 import identity | [x] | `ir/ids.rs` |
| .ids.json sidecar (write on import, restore on re-import) | [x] | `import/ids_sidecar.rs` |
| **No .lbr write-back** | [x] | import-only, confirmed |

**M0 overall**: [x] Complete (closed by architect)

---

## PROGRAM_SPEC.md — M1 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| KiCad .kicad_pcb import: DOA2526 + 4 designs, 0 errors | [~] | Skeleton parser exists; warns "full geometry not implemented" |
| KiCad .kicad_sch import: DOA2526 + 2 schematics, 0 errors | [~] | Skeleton parser exists; warns "full symbol connectivity not implemented" |
| Eagle .brd/.sch import: 3 designs, 0 errors | [ ] | Returns "not implemented" error |
| Schematic connectivity: hierarchy, labels, power symbols | [~] | Union-find solver in `connectivity/mod.rs`; imported KiCad child-sheet hierarchical linking and basic bus/member expansion now feed existing net/query/check surfaces for the current supported subset, while wider import fidelity and advanced bus syntax remain open |
| Board connectivity: net-to-pin resolution | [x] | `board/mod.rs` net_pins(), net_pad_points() |
| Airwire computation: matches KiCad on DOA2526 | [~] | Algorithm implemented; DOA2526 validation not confirmed |
| Board diagnostics: net_without_copper, via_only, partially_routed | [x] | `board/mod.rs` diagnostics() |
| Golden tests: 8+ designs with checked-in golden files | [ ] | `tests/corpus/` is empty |
| Deterministic import: byte-identical on 3 runs | [~] | Current KiCad import/query corpus is now gated by `scripts/check_import_query_determinism.py` over checked-in query/check fixtures and passes 3-run stability for the current supported subset; broader import fidelity and non-query/save-backed import determinism remain open |
| Import fidelity KiCad ≥ 90% | [ ] | No fidelity matrix exists |
| Import fidelity Eagle ≥ 85% | [ ] | Eagle design import not implemented |

### PROGRAM_SPEC.md — M1 Query API

Recorded coverage: 68 [x], 1 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/m2_details.md#program_specmd--m1-query-api).

## PROGRAM_SPEC.md — M2 Exit Criteria

### ERC Rules (specs/ERC_SPEC.md §4)

| Check | Status | Notes |
|-------|--------|-------|
| output_to_output_conflict | [x] | `erc/mod.rs` |
| undriven_input | [x] | `erc/mod.rs` (undriven_input_pin + input_without_explicit_driver) |
| power_without_source | [x] | `erc/mod.rs` (power_in_without_source) |
| noconnect_connected | [x] | `erc/mod.rs` |
| unconnected_required_pin | [x] | `erc/mod.rs` (unconnected_component_pin) |
| passive_only_net | [~] | No distinct `passive_only_net` rule/code ships in `erc/mod.rs` (0 grep hits 2026-06-22). Passive pins are only consumed as a modifier that softens `input_without_explicit_driver` messaging; a standalone passive-only-net check is target work. |
| hierarchical_connectivity_mismatch | [x] | Implemented as sheet-level hierarchical-label vs port mismatch check in `erc/mod.rs` |

ERC correctness: [x] `m2_quality` harness reports 0.0% FP / 0.0% FN on current ERC fixtures (2026-03-25)

### DRC Rules

| Check | Status | Notes |
|-------|--------|-------|
| clearance_copper | [x] | Implemented in `drc/mod.rs` (track-track same-layer clearance) |
| track_width | [x] | Implemented in `drc/mod.rs` (`track_width_below_min`) |
| via_hole | [x] | Implemented in `drc/mod.rs` (`via_hole_out_of_range`) |
| via_annular | [x] | Implemented in `drc/mod.rs` (`via_annular_below_min`) |
| connectivity | [x] | Implemented in `drc/mod.rs` (no-copper + unrouted-net checks) |
| unconnected_pins | [x] | Implemented in `drc/mod.rs` (`connectivity_unconnected_pin`) |
| silk_clearance | [x] | Implemented in `drc/mod.rs` (`silk_clearance_copper`) |

DRC correctness: [x] `m2_quality` harness reports 0.0% FP / 0.0% FN on current DRC fixtures (2026-03-25)

### MCP Tools (specs/MCP_API_SPEC.md — M2)

| Tool | Status | Notes |
|------|--------|-------|
| open_project | [x] | Current implementation contract |
| close_project | [x] | Current implementation contract |
| search_pool | [x] | Current implementation contract |
| get_part | [x] | Current implementation contract |
| get_package | [x] | Current implementation contract |
| get_package_change_candidates | [x] | Current implementation contract |
| get_board_summary | [x] | Current implementation contract |
| get_schematic_summary | [x] | Current implementation contract |
| get_sheets | [x] | Current implementation contract |
| get_symbols | [x] | Current (all-sheets only) |
| get_ports | [x] | Current (all-sheets only) |
| get_labels | [x] | Current (all-sheets only) |
| get_buses | [x] | Current (all-sheets only) |
| get_bus_entries | [x] | Current implementation contract |
| get_noconnects | [x] | Current (all-sheets only) |
| get_symbol_fields | [x] | Current implementation contract |
| get_hierarchy | [x] | Current implementation contract |
| get_netlist | [x] | Current implementation contract |
| get_components | [x] | Current implementation contract |
| get_net_info | [x] | Current (full inventory, not single-net selector) |
| get_schematic_net_info | [x] | Current (full inventory, not single-net selector) |
| get_connectivity_diagnostics | [x] | Current implementation contract |
| get_design_rules | [x] | Current implementation contract |
| get_unrouted | [x] | Current implementation contract |
| get_check_report | [x] | Current implementation contract (not in M2 exit list but implemented) |
| run_erc | [x] | Current implementation contract |
| run_drc | [x] | Current implementation contract |
| explain_violation | [x] | Current implementation contract |

MCP tools: M2 slice 26/26 implemented. Current MCP runtime catalog: 181
methods (daemon-dispatched + CLI-bridged via `mcp-server/server_runtime.py`),
locked via `specs/SPEC_PARITY.md` → `mcp_runtime_methods`.

### CLI Commands (specs/PROGRAM_SPEC.md — M2)

Historical M2 CLI coverage: 8 completed commands—`tool import` (Eagle .lbr only),
`tool query --nets/--components/--summary`, `tool erc` (.kicad_sch only), `tool drc`
(JSON/text and nonzero on violations), `tool pool search`, and unified `tool check`.
Query modes count as three commands. Checking commands return 1 for violations
and 2 for execution errors; supported CLI exit codes are 0/1/2. Current Project
command identities remain locked by `SPEC_PARITY.md` → `cli_project_commands`;
`check_spec_parity.py --print` enumerates them. This is historical slice evidence.

### Other M2 Requirements

| Criterion | Status | Notes |
|-----------|--------|-------|
| MCP registration in active MCP host config | [x] | Configured `datum-eda` MCP entry with `EDA_ENGINE_SOCKET=/tmp/datum-eda-engine.sock` (2026-03-25) |
| Test corpus: 10+ designs with ERC/DRC golden files | [x] | `m2_quality` reports 11 unique designs (erc=5, drc=6) |
| Quality-rate harness (ERC/DRC FP/FN + corpus gate) | [x] | `m2_quality` implemented with checked-in manifest; current gate state is `pass=true` |
| ERC on DOA2526 < 3 seconds | [x] | `m2_perf` median ERC = 2ms (3-iteration run, 2026-03-25) |
| DRC on DOA2526 < 5 seconds | [x] | `m2_perf` median DRC = 18ms baseline, 20ms compare run (2026-03-25) |
| MCP transport (daemon ↔ MCP server) | [x] | Unix socket JSON-RPC implemented in daemon + Python client; live transport smoke is tracked separately in unrestricted CI |

**M2 overall**: [x] Complete for the current implementation slice — ERC/DRC checks, MCP/CLI parity, quality/performance harnesses, and local MCP host registration are all in place.

### M2 Pre-Freeze Closeout Checklist (Ordered)

Recorded coverage: 8 [x]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/m2_details.md#m2-pre-freeze-closeout-checklist-ordered).

### 2026-03-25 Contract Alignment Pass

Recorded coverage: 18 [x]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/m2_details.md#2026-03-25-contract-alignment-pass).

## PROGRAM_SPEC.md — R1 Research Gates

| Criterion | Status | Notes |
|-----------|--------|-------|
| R1-G0 Foundation Gate | [x] | Minimum history/context gate is now evidenced in `docs/R1_G0_FOUNDATION.md` and blocks downstream milestone completion claims unless maintained |
| R1-G0 tool lineage + format evolution map | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §1 covers KiCad/Eagle/Altium/PADS/OrCAD lineage, likely ingestion surfaces, and roadmap implications |
| R1-G0 migration pain taxonomy | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §2 classifies migration failure modes using the current KiCad/Eagle design/library corpus and current interop workflow evidence |
| R1-G0 fidelity boundary policy | [x] | Evidence: `docs/R1_G0_FOUNDATION.md` §3 defines exactness, approximation, preservation-as-metadata, and unsupported-loss boundaries for future interop claims |

**R1 overall**: [~] Minimal context gate is complete, but broader commercial-interop research exit criteria remain open (corpus, legal posture, prototypes, recommendation)

---

## PROGRAM_SPEC.md — M3 Exit Criteria

Recorded coverage: 28 completed criteria for the implemented imported-board
write slice, including edits, replacements, introspection, history, fidelity and
CLI/MCP writes. [Criterion-level status and evidence](progress/m3_details.md)
remain unchanged. **M3 overall**: [x] complete for that recorded slice; this
historical record grants no current import execution or broader qualification.

## PROGRAM_SPEC.md — M4 Exit Criteria

| Criterion | Status | Notes |
|-----------|--------|-------|
| Native format | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#native-format`. |
| Schematic operations | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#schematic-operations`. |
| Board operations | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#board-operations`. |
| Schematic query parity | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#schematic-query-parity`. |
| Forward annotation | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#forward-annotation`. |
| Gerber export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#gerber-export`. |
| Drill export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#drill-export`. |
| BOM export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#bom-export`. |
| PnP export | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#pnp-export`. |
| Gerber comparison | [x] | Detailed status and evidence anchors are maintained in `specs/progress/m4_details.md#gerber-comparison`. |

**M4 overall**: [x] Closed for scope; details remain in `specs/progress/m4_details.md`.

## M5 Opening Charter

Status: [~] Routing-kernel scope complete; closure review pending. Opening authority remains `specs/progress/m5_opening.md`. [Detailed evidence](progress/m5_details.md#m5-opening-charter).

## M6 Opening Charter

Status: [~] Frozen pending evidence. Opening authority remains `specs/progress/m6_opening.md`. [Detailed evidence](progress/m6_opening.md#m6-opening-charter).

## M7 Opening Charter

Status: [~] Opened narrowly as a read-only route-proposal review layer. Opening authority remains `specs/progress/m7_opening.md`. [Detailed evidence](progress/m7_opening.md#m7-opening-charter).

## Standards Audit Batch 1 — Spec Stubs Awaiting Implementation

Recorded coverage: 2 [x], 2 [ ]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#standards-audit-batch-1--spec-stubs-awaiting-implementation).

### Pass 0 — Standards Compliance Disposition Refresh

Recorded coverage: 2 [x], 2 [—]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#pass-0--standards-compliance-disposition-refresh).

### Pass 1 — Engine Schema Bedrock

Recorded coverage: 6 [x], 6 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#pass-1--engine-schema-bedrock).

### Pass 2 — Pool and Native Persistence

Recorded coverage: 2 [x], 2 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#pass-2--pool-and-native-persistence).

### Pass 3 — MCP API Stubs

Recorded coverage: 3 [x], 2 [ ], 1 [—]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#pass-3--mcp-api-stubs).

### Pass 4 — Import Spec

Recorded coverage: 2 [x], 2 [ ]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#pass-4--import-spec).

### Pass 5 — Architecture and Scope Docs

Recorded coverage: 3 [x], 3 [—]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#pass-5--architecture-and-scope-docs).

### Deferred — Held For A Later Batch

Recorded coverage: 1 [x], 1 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#deferred--held-for-a-later-batch).

## ENGINE_SPEC.md — Core Types

Checkpoint evidence and retained boundaries remain in the linked detail; current selection comes from the Active Frontier. [Detailed evidence](progress/contract_coverage_details.md#engine_specmd--core-types).

### §1.1 Geometry Primitives

Recorded coverage: 5 [x]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#11-geometry-primitives).

### §1.1a Shared Enums

Recorded coverage: 5 [x]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#11a-shared-enums).

### §1.2 Pool Types

Recorded coverage: 8 [x]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#12-pool-types).

### §1.3 Board Types

Recorded coverage: 11 [x]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#13-board-types).

### §1.4 Schematic Types

Recorded coverage: 13 [x]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#14-schematic-types).

### §1.5 Rule Types

Recorded coverage: 5 [x], 1 [—]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#15-rule-types).

### §2 Object Invariants

Recorded coverage: 5 [x]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#2-object-invariants).

### §3 Operations

Recorded coverage: 5 [ ]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#3-operations).

### §5 Engine API

Recorded coverage: 45 [x], 3 [~], 1 [ ]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#5-engine-api).

## CHECKING_ARCHITECTURE_SPEC.md

Recorded coverage: 8 [x]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#checking_architecture_specmd).

## ERC_SPEC.md

Recorded coverage: 11 [x], 3 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#erc_specmd).

## IMPORT_SPEC.md

Recorded coverage: 7 [x], 3 [~], 2 [ ]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#import_specmd).

## SCHEMATIC_CONNECTIVITY_SPEC.md

Recorded coverage: 9 [x], 1 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/contract_coverage_details.md#schematic_connectivity_specmd).

## SCHEMATIC_EDITOR_SPEC.md — M4 Operations

Status: [x] Closed for scoped M4 slice [Detailed evidence](progress/contract_coverage_details.md#schematic_editor_specmd--m4-operations).

## NATIVE_FORMAT_SPEC.md

Status: [x] Closed for scoped M4 slice [Detailed evidence](progress/contract_coverage_details.md#native_format_specmd). PM061 bounded process implementation landed in `ef32fbe1`; [source-pinned disposition](../docs/reviews/workflow-delivery-rollout/effective-process-owner-delegation.json) retains its exact delivery and availability limits. Standards and broader qualification remain open.

## Infrastructure

| Item | Status | Notes |
|------|--------|-------|
| Rust workspace (10 crates) | [x] | engine, cli, engine-daemon, test-harness, gui-protocol, gui-render, gui-app, gui-viewport, terminal-core, verb-registry (locked via `specs/SPEC_PARITY.md` → `workspace_crates`) |
| Engine compiles without GUI deps | [x] | |
| Test harness (golden file utilities) | [x] | test-harness crate |
| Test corpus (real designs) | [ ] | tests/corpus/ empty |
| Daemon JSON-RPC dispatch | [x] | 39 methods in `dispatch.rs`, with the 11 retired imported-board writer arms removed (Retired tombstones point at datum.pcb.*/datum.proposal.*) and the 4 terminally frozen imported-session writers fenced as hidden compatibility, with coverage in daemon tests |
| Daemon socket transport | [x] | `main()` parses `--socket` and serves Unix socket; live smoke is environment-gated because sandboxed local runs deny socket IPC |
| MCP Python server (tool host) | [x] | Tool definitions + stdio dispatch |
| MCP→daemon transport | [x] | `EngineDaemonClient.call()` uses Unix socket JSON-RPC; behavioral parity remains covered separately from live socket smoke |
| Git repository initialized | [x] | `main` branch with GitHub remote configured |
| CI pipeline | [x] | `.github/workflows/alignment.yml` runs alignment and file-size budget checks |

## GP-CM05E production acceptance correction

Production acceptance corrections and retained qualification boundaries are recorded in the detailed delivery history. [Detailed evidence](progress/gui_delivery_details.md#gp-cm05e-production-acceptance-correction).

### PM056 — imported Zone generated-fill repair

Recorded coverage: 1 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/gui_delivery_details.md#pm056--imported-zone-generated-fill-repair).

### S5A-C03 — ordinary eligibility and overlap Select All

Recorded coverage: 1 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/gui_delivery_details.md#s5a-c03--ordinary-eligibility-and-overlap-select-all).

### S5A-C03 — governed selection-lock-anchor cue

Recorded coverage: 1 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/gui_delivery_details.md#s5a-c03--governed-selection-lock-anchor-cue).

### S5A-C03 — footprint-owned graphic acquisition (existing sources)

Recorded coverage: 1 [~]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/gui_delivery_details.md#s5a-c03--footprint-owned-graphic-acquisition-existing-sources).

### S5A-C03 — remaining authority disposition packet (pending owner)

Recorded coverage: 1 [ ]. Detailed scope and limitations remain in the linked evidence. [Detailed evidence](progress/gui_delivery_details.md#s5a-c03--remaining-authority-disposition-packet-pending-owner).

### S5A-C03 — exact 30-degree query execution ratified

Checkpoint evidence and retained boundaries remain in the linked detail; current selection comes from the Active Frontier. [Detailed evidence](progress/gui_delivery_details.md#s5a-c03--exact-30-degree-query-execution-ratified).

## DELIVERY-MANAGEMENT — owner-directed workflow correction

Owner-appointed management applies PM059 delegation; product rules, waivers and reserved HUMAN decisions remain owner-only. [Detailed evidence](progress/gui_delivery_details.md#delivery-management--owner-directed-workflow-correction).

## INSTRUCTION-CONSOLIDATION — task-scoped agent reading

Task-scoped reading uses CLAUDE.md and applicable operational procedures; it does not waive owning authority or review. [Detailed evidence](progress/gui_delivery_details.md#instruction-consolidation--task-scoped-agent-reading).

## USER-EDIT-DELIVERY bounded writer prerequisite

Bounded writer prerequisite history is retained; live selection and execution authority come from the Active Frontier. [Detailed evidence](progress/gui_delivery_details.md#user-edit-delivery-bounded-writer-prerequisite).
