# Source-Health Operational Policy

Status: active

This document is the operational companion to
`docs/decisions/PRODUCT_MECHANICS_022_SOURCE_HEALTH_GOVERNANCE.md`. Decision 022
is controlling. If this document, a manifest, a script, or a pull-request
instruction conflicts with it, the weaker instruction is invalid and the
conflict is a blocking governance defect.

## Contributor Workflow

Before changing source:

1. Run the source-health gate and note whether any intended write target is in
   the legacy debt ledger.
2. If a target is oversized, identify a cohesive ownership boundary that can be
   extracted without changing behavior.
3. Keep every new child below its applicable normal budget.

Before landing:

1. Run the source-health and policy-integrity regression tests.
2. Run the complete drift battery.
3. Record before/after line counts and structural evidence for every touched
   legacy file.
4. Ratchet its baseline and ceiling downward in the same change.
5. Remove a legacy entry in the same change when the file returns to its normal
   budget.

Registration is never the response to a new oversized file. Decompose it before
landing.

Current debt inventory: 93 legacy source entries remain after UNIT-I03B moved
Project-root operation application into its cohesive dispatch owner and returned
`operation_application.rs` to the normal production budget. Its stale ledger
entry and the no-longer-needed `replay.rs` rustfmt exemption were removed in the
same change.

PM045 S3 moved Project/Layers composition and shared panel chrome into normal
Rust modules. `gui-render/src/side_panels.rs` drops from 91 to 52 physical lines
and from 1,685 to 1,393 expanded lines; its remaining include debt stays open
with an exact downward ceiling. Panel clipping uses the shared content owner.

PM045 S3 control-label layout moved text-buffer keys and layout extents from
`render/geometry.rs` into the existing shared `text_buffer_cache` module. Its
pre-test lines decrease from 1,250 to 1,215; the root expanded ceiling becomes
7,375. Text identity/signature tests move into their own test module, reducing
`render/tests.rs` from 1,159 to 1,070 lines. All three limits ratchet downward;
remaining debt stays open.

PM045 S3 shell-layout adoption moves the shell solver and existing bounded
layout cache into `gui-render/src/render/shell_layout.rs`, shared by frame
preparation, retained-scene preparation and Runtime input geometry. The private
app cache is retired. The renderer root include expansion falls from 7,375 to
7,191 lines with an exact downward ceiling; the new owner stays within normal
budgets. The root module map and existing panel solvers remain unchanged.

PM045 S3 Project panel retention extracts its Taffy row solver into
`gui-render/src/side_panels/project_layout.rs`, with a bounded two-entry cache
keyed only by geometry inputs. `side_panels.rs` keeps its 52-line module root;
its include expansion falls from 1,393 to 1,261 lines. The exact ceiling ratchets
downward; Inspector and Layers row solvers remain separate and unchanged.

PM045 S3 Layers geometry retention moves its row solver into
`gui-render/src/side_panels/filters_layout.rs`. Shell, Project and Layers use one
bounded two-entry cache implementation; Layers retains its row vector by shared
ownership. The side-panel include ceiling falls from 1,261 to 1,160 lines;
its root remains 52 lines and Inspector layout remains unchanged.

PM045 S3 Inspector geometry retention extracts its outer and detail solvers into
`gui-render/src/side_panels/inspector_layout.rs`, using the shared layout cache
policy. The side-panel include ceiling falls from 1,160 to 1,011 lines;
the root remains 52 lines. Inspector composition remains include debt; solver
extraction and cache adoption do not close that remaining ownership work.

PM045 S4 text-GPU ownership introduces the normal `gui-render/src/text_gpu`
module and moves text-area construction into the existing `text_buffer_cache`
owner. The renderer root include ceiling falls from 7,191 to 7,167 lines;
`render/geometry.rs` pre-test lines fall from 1,215 to 1,189. Exact ceilings
ratchet downward; existing include debt remains open. Renderer unit tests,
installed-renderer pixel parity and production GPU tests cover the migration.

PM045 S4 retained text shaping/layout moves into the normal `text_layout`
module. Its cache entry moves from `render/types.rs` into `text_buffer_cache`;
the renderer root include ceiling falls from 7,167 to 7,159 lines. Shared
immutable shape payloads and independent visible layout rows replace opaque
retained buffers. Unit, extent-reuse and installed-renderer pixel comparisons
cover this ownership change; private font/scratch accounting remains open.

PM045 S4 scoped private-text accounting keeps the allocator implementation in
normal `cpu_alloc` and moves the existing text-color conversion beside font/text
attributes in `render/text_metrics`. Native allocator installation stays at the
existing shared native GPU boundary. The renderer include ceiling falls from
7,159 to 7,153 lines and geometry production from 1,189 to 1,183; no exception or
limit increase is introduced. Allocator conformance and text pixel parity support
this ownership change; full overhead/resource qualification remains separate.

## What Counts as Structural Evidence

Qualifying evidence names the ownership moved, the real module boundary created,
the new destination files, and behavior-preserving tests. Good boundaries follow
domain responsibility, lifecycle, or stable type ownership.

The following do not qualify:

- `include!` or equivalent textual concatenation;
- import/re-export-only, forwarding-only, or registration-only shells;
- numbered or generically named continuation files;
- whitespace compression, statement joining, or deletion without ownership
  transfer;
- moving implementation into tests, fixtures, examples, or generated-looking
  authored files; or
- splitting only to place each child immediately at the maximum budget.

Thin roots are allowed when they remain readable module maps with visible
ownership boundaries. A Rust file is measured at 700 pre-test lines, with an
inline `#[cfg(test)]` tail measured separately at 350 lines; dedicated Rust test
files use 700 total physical lines. Recursive literal `include!` content is also
expanded and measured as one logical module with a 700-line limit, so physically
small trampolines cannot conceal a logical monolith. Other authored languages
use their physical-file limits from decision 022.

## Debt-Ledger Operations

For a legacy file:

- each value in `limits` is the last approved landed count and hard ceiling for
  that metric;
- live growth above any metric limit is a failure;
- live shrinkage requires that metric limit to ratchet exactly to the live count;
- A move or rename requires atomic ledger reconciliation and retains history.
- Reaching the normal budget requires immediate de-listing.

Ledger entries also state source kind, trigger commit/date, responsible owner,
target boundary, intended shards, lifecycle status, and evidence references.
Policy constants never belong in the ledger.

## Tripwire Response

When the gate reports a new oversized file, split it into cohesive modules; do
not add a debt entry.

When it reports growth in a known monolith, extract enough real ownership that
the original ends smaller than merge base, then ratchet its ledger values.

When it reports an oversized extracted child, redesign the boundary; moving the
same concentration into a different filename does not resolve the defect.

When it reports stale debt, reconcile the move/deletion or remove a now-compliant
entry in the same change.

When it reports policy weakening, restore the controlling contract. If a genuine
permanent policy change is intended, stop and use a later numbered Product
Mechanics amendment with project-owner approval.

## Temporary Emergency Exception

Do not create an exception merely to keep feature work moving. If an operational
emergency truly prevents immediate decomposition, prepare a later numbered
Product Mechanics amendment containing the exact temporary delta, reason,
expiry, retirement evidence, and affected tests. Obtain project-owner approval
through the protected CODEOWNERS review path. An expired or incomplete exception
is blocking.

## Review Checklist

Reviewers verify:

- discovery includes every authored source file added or moved;
- no new source exceeds its applicable budget;
- every touched legacy file is smaller than merge base;
- extraction owns cohesive implementation rather than textual/facade indirection;
- new children and test destinations remain healthy;
- ledger ceilings only move downward and compliant entries are removed;
- behavior tests cover structural movement;
- no policy, coverage, workflow, or required-check weakening is hidden in the
  change; and
- protected governance changes have project-owner approval.

PM045 S4 consolidates raw legacy scene-buffer initialization and bindings into
`render/uniform_buffer.rs`. The renderer root no longer supplies a broad
`DeviceExt` import to that path; its exact expanded ceiling decreases from
7,153 to 7,152. The shared owner retains bindings, deferred uniform updates and
submission references. Remaining include debt and normal budgets are unchanged.

## Required Enforcement Surface

The standard drift battery must invoke blocking checks for source discovery and
budgets (including Rust pre-test, inline-test-tail, dedicated-test, and recursive
literal-`include!` measures), touched-monolith comparison against merge base,
ledger schema and downward ratcheting, architectural-evasion detection, and
policy integrity. The
checker regression suite must exercise tracked, staged, untracked, renamed,
decomposed, regrown, test-moved, expired-exception, and policy-tampering cases.

The controlling decision, this policy, the checkers and tests, debt ledger,
standard drift wiring, CI workflow, and governance classifications must remain
under CODEOWNERS and protected-branch/ruleset review. In-repository checks are a
backstop; repository protection supplies the human authorization boundary.
