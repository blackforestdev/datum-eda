# Product Mechanics 046: Bounded text dependency patches

Status: historical. Ratified on 2026-09-24; the bounded trial subsequently
stopped without an integrated implementation. Source-patch authority is retired
under the owner's full-cleanup instruction. No active dependency exception remains.
Issue and Frontier: `dat-gui-performance-implementation-vkq`, GPI-S4.

## Owner decision

The owner replied `approved` to the exact package-and-license proposal in
commit `85d99f31`, recorded at
`docs/reviews/gui-performance/implementation/S4/dependency-patch-review/result.json`.
This decision is the PM029 DA-001/007 exception for that bounded trial, not
acceptance of its feasibility, runtime results, S4 completion or S5 qualification.

## Exact authority

| Package | Version | Selected license | Permitted source location |
|---|---|---|---|
| cosmic-text | 0.15.0 | MIT | third_party/cosmic-text |
| swash | 0.2.7 | MIT | third_party/swash |
| zeno | 0.3.3 | MIT | third_party/zeno |

Existing installed registry packages are the source baseline. Preserve source
provenance, original license files and copyright notices, record local changes,
and retain applicable notices in any distribution containing these packages.
Use the MIT option; no Apache option is selected. Package approval is not a
whole-project or transitive-dependency licensing clearance.

Local Cargo patches may redirect exactly these existing packages to the named
paths. Do not upgrade versions, introduce another dependency, modify glyphon,
change registry sources in place, or add network build/download steps. Modified
sources remain third-party code, not independently authored Datum code.

## Implementation boundary

Add fallible font selection/cache construction and raster scratch/output
admission, with existing Datum host/process budgets as authority. Cover retained
capacity, temporary growth and old/new overlap. Refusal must preserve source
text, font identity and valid prior state, propagate distinctly from a missing
glyph, and permit successful retry after resources retire. Preserve pixels,
fallback, shaping, quality and existing compatibility entry points.

Before production adoption, demonstrate one integrated cold and cache-growth
font/raster path with pre-allocation refusal, concurrent-owner reservations,
successful retry, release and pixel fidelity. Counters alone, measured peaks,
arbitrary reservations, post-call eviction, allocator abort and allocator unwind
do not establish this admission guarantee.

Stop rather than expand the exception if correct admission requires modifying
another package, increasing budgets, a broad algorithm replacement or unsafe
allocation-failure recovery. HarfRust font construction and Skrifa outline/hinting
remain explicit feasibility risks; this approval does not authorize their
modification. Preserve useful incomplete work and report the concrete boundary.

## Delivery and unchanged requirements

After the integrated proof succeeds, migrate the default shared consumers and
retire competing unbounded production calls. Record consumers, retired paths,
proof and limitations in the existing adoption map. All four remaining S4
implementation predicates and the complete S4 exit review still precede S5.
All original S5 qualification, GEFN preservation and nonblocking resize
exclusions remain. No terminal implementation, UI styling or backend policy
change is authorized by this exception.

## Trial disposition

The feasibility review did not establish safe construction admission across the
unmodified HarfRust/Skrifa calls. No resource guarantee or S4 completion follows.
All trial source copies, overrides, trial-only enforcement and build artifacts
are removed; the preexisting registry text dependencies remain. Details and
restored-build proof are recorded in
`docs/reviews/gui-performance/implementation/S4/dependency-patch-review/disposition.json`.
Reopening or expanding this exception requires a new owner decision.
