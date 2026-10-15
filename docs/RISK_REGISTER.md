# Historical risk analysis

Status: historical planning hypotheses. The R1–R9 analysis below preserves its
original rationale and mitigation ideas; its "not started", "accepted for v1"
and "resolved" labels are not current implementation state, owner acceptance
or a qualification waiver. Current work and recorded defects come from the
[Frontier](../specs/active_frontier.json), [progress](../specs/PROGRESS.md) and
canonical tracker export, with their actual evidence and claims.

| Original assumption | Current authority to consult |
| --- | --- |
| Import-first MVP; delay GUI/routing; M0–M8 timing | [Product mechanics](DATUM_PRODUCT_MECHANICS.md) and selected completion contract; original milestone limits are historical. |
| R3 resolved because AI replaces manual routing | Manual-first, optional-AI product doctrine. The old rationale establishes no product exclusion or dependency decision. |
| Use geometry crates or isolate a GPL component | [PM029](decisions/PRODUCT_MECHANICS_029_DEPENDENCY_AUTHORITY.md); references and mitigation suggestions do not authorize additions or accepted obligations. |
| "Not started" source/corpus/manufacturing work | Current progress and task evidence; old labels are not an audit of today's code. |
| Numerical corpus/performance targets | Existing governed acceptance criteria; original planning numbers below are not new blocking thresholds. |

[PROGRAM_SPEC](../specs/PROGRAM_SPEC.md#scope-integrity-terms) owns scope terms.
All original analysis follows unchanged. Do not infer closure or current risk
acceptance from preservation of these notes.

## R1: Import Fidelity
**Risk**: Imported KiCad/Eagle designs don't round-trip correctly.
Misaligned pads, missing nets, wrong layer mapping, lost design rules.

**Impact**: Critical — the current execution slice depends on accurate import.
If import is lossy, DRC results are wrong, queries return garbage,
and the tool is useless.

**Mitigation**:
- Golden test corpus of 50+ real designs with known-good DRC results
- Round-trip test: import → export → diff against original
- Per-format fidelity tracker (which features import cleanly, which don't)
- Accept partial import with explicit warnings over silent data loss

**Status**: Not started. Corpus assembly is an M0 task.

## R2: Single Architect / Bus Factor
**Risk**: One person designing and building the entire system.
If that person stops, the project dies.

**Impact**: High for long-term viability, low for initial development.
The single-architect model is correct for establishing coherent
architecture, but becomes a liability at scale.

**Mitigation**:
- Thorough documentation (this document set)
- Clear architectural decisions with rationale
- Deterministic tests that encode design intent
- If the project gains traction, recruit contributors for specific
  subsystems (exporters, importers, DRC rules) not core architecture

**Status**: Inherent. Accepted for v1.

## R3: ~~GPL Router Contamination~~ (Resolved)
**Original risk**: Bundling KiCad PNS infects the codebase with GPL v3.

**Resolution**: KiCad PNS is not needed. The routing paradigm is
AI-proposes/human-reviews, not interactive push-and-shove. Custom
constraint-based routing engine, no GPL dependency. See LICENSING.md.

**Status**: Resolved. No GPL router in the architecture.

## R4: Geometry Engine Robustness
**Risk**: Computational geometry is hard. Edge cases in polygon clipping,
zone fill, clearance checking, and coordinate transforms cause crashes
or incorrect results.

**Impact**: High — DRC correctness and manufacturing output depend on
geometry being bulletproof.

**Mitigation**:
- Use proven Rust geometry crates (geo, geo-clipper) where possible
- Integer coordinates (nanometers) eliminate floating-point edge cases
- Exhaustive property-based testing (proptest) for geometry operations
- Compare DRC results against KiCad's DRC on the same design

**Status**: Not started. Architecture supports mitigation.

## R5: Scope Creep
**Risk**: The full-CAD ambition pulls effort away from the current execution
slice.
Features like GUI, routing, 3D, supply chain get started before the
foundation (import, query, DRC) is solid.

**Impact**: High — a half-built everything is worse than a complete
analysis tool.

**Mitigation**:
- Milestones are sequenced so each is useful on its own
- M2 deliverable is the minimum viable product
- Non-goals for v1 are explicitly documented in CLAUDE.md
- Resist adding GUI or routing until M4 is complete

**Status**: Active. Discipline required.

## R6: KiCad Format Instability
**Risk**: KiCad changes its file format between versions (it has,
multiple times). Importers break on new versions.

**Impact**: Medium — KiCad import is a key feature, but format changes
are incremental and well-documented.

**Mitigation**:
- Parse KiCad S-expression format generically, not version-specifically
- Test against KiCad 7, 8, and 9 files
- Monitor KiCad release notes for format changes

**Status**: Not started.

## R7: Performance at Scale
**Risk**: Large boards (1000+ components, 5000+ nets) are slow to import,
query, or DRC. JSON serialization becomes a bottleneck.

**Impact**: Medium — most designs are smaller, but professional credibility
requires handling complex boards.

**Mitigation**:
- Profile early with large designs in the test corpus
- SQLite for pool queries (already fast)
- Incremental DRC (only recheck affected objects)
- JSON streaming/lazy loading if needed
- Binary format option for large designs (future)

**Status**: Not started. Monitor from M1.

## R8: Manufacturing Output Correctness
**Risk**: Gerber/drill output has errors — wrong apertures, missing layers,
incorrect drill coordinates. Fabrication house rejects the files or
(worse) manufactures a defective board.

**Impact**: Critical — manufacturing trust is non-negotiable. One bad
Gerber export destroys credibility permanently.

**Mitigation**:
- Compare output against KiCad's Gerber for the same design
- Validate with open-source Gerber viewers (gerbv, tracespace)
- Test with real fabrication houses (JLCPCB, PCBWay) on test designs
- Gerber export is M4, not M1 — plenty of time to validate

**Status**: Not started. M4 task.

## R9: AI Operation Determinism
**Risk**: AI-driven operations produce different results on different runs
(non-deterministic placement, routing, or rule application). This makes
designs unreproducible.

**Impact**: Medium — reproducibility is important for professional use.

**Mitigation**:
- All operations are deterministic given the same inputs
- Random elements (UUID generation) use seeded RNG in test mode
- AI suggestions are presented as proposed operations, not auto-applied
- Operation log enables exact replay

**Status**: Architecture supports this. Verify during M3.
