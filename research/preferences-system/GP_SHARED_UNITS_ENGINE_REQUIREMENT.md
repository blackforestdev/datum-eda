# Shared Units Engine Requirement

> **Status:** Owner-directed Global Preferences specification requirement;
> implementation and dependencies are not authorized.
>
> **Tracker:** `dat-global-preferences-engine-qcv`.

## Authority and current gap

Datum's canonical authored length authority is already exact: coordinates,
dimensions, and distances are signed `i64` nanometers
(`docs/CANONICAL_IR.md:59-77`; `crates/engine/src/ir/geometry.rs:3-15`). The
native-write path consumes normalized typed values, and current MCP board
schemas expose integer `_nm` fields (`mcp-server/tools_catalog_datum.py:194-205`).

The current conversion surface is not one units engine. Engine helpers convert
mm/mil/inch through `f64` (`crates/engine/src/ir/units.rs:1-26`), Gerber owns a
private fixed-six-decimal parser/formatter
(`crates/engine/src/export/formatting.rs:1-30`), and other import, preview,
save, CLI, GUI, and MCP paths retain independent conventions. There is no shared
quantity-aware suffix parser, display-unit resolver, precision policy, or
cross-surface provenance answer.

The working prototype renders the intended user model at
`docs/gui/prototypes/preferences-window.html:110-120`: one measurement system,
per-quantity display choices, display-only precision including exact nanometers,
and unit-suffixed input served by one implementation. This document governs the
engine seam only; it does not ratify every current control label or default.

## Required engine contract

<!-- OWNER-REQUIREMENT:GLOBAL-PREFERENCES:SHARED-UNITS-ENGINE -->

1. Datum must provide one engine-owned Units service used by GUI, CLI, and MCP
   edge adapters. A caller-specific parser or formatter may wrap the service but
   cannot define rival unit semantics.
2. Canonical authored length remains signed `i64` nanometers. Measurement
   system, display unit, and precision are projections only; they never mutate,
   quantize, round, or reserialize stored geometry.
3. `MeasurementSystem` initially distinguishes `Metric` and `Imperial` and
   supplies defaults for quantity descriptors that follow the system. It is not
   storage authority.
4. Each supported physical quantity has a typed quantity kind and a typed
   display-unit choice. Per-quantity overrides may select a unit from either
   measurement system. A cross-system choice is valid but returned and rendered
   as an explicit override rather than silently normalized away.
5. `DisplayPrecision` controls formatting only and includes an exact-nanometer
   mode. Rounded display text cannot be written back as if it were the exact
   stored value.
6. The shared parser accepts locale-independent decimal length expressions with
   explicit supported suffixes, initially including `nm`, `µm`/`um`, `mm`,
   `mil`, and `in`. Inputs such as `5mm`, `200mil`, and `0.1in` use identical
   semantics in GUI, CLI, and MCP.
7. Conversion uses checked integer/rational arithmetic. Overflow, malformed or
   ambiguous suffixes, wrong quantity kinds, and non-integral-nanometer results
   are typed refusals. Any future explicit rounding mode requires a separate
   governed decision and visible provenance.
8. Bare numbers are accepted only when the caller supplies the field
   descriptor's resolved display-unit context. Automation cannot depend on an
   unstated machine preference; CLI/MCP requests must carry or resolve explicit
   context, while a suffix remains context-independent.
9. Parse and format results expose typed provenance: quantity kind, authored
   token, explicit-versus-contextual unit, resolved unit, exact canonical value,
   display precision, cross-system-override state, and refusal reason where
   applicable.
10. Existing native operations and persisted design schemas continue to receive
    integer nanometers. Adding expression input to CLI/MCP requires a compatible
    edge-adapter/schema migration; it cannot silently replace established `_nm`
    integer fields.
11. Length, angle, ratio, percentage, frequency, and other quantities remain
    distinct types. The length suffix set cannot leak into unrelated fields.
12. The final Preferences descriptor catalog must separately classify the
    measurement-system default, each quantity display override, and each
    precision policy while pointing them to this one engine service.

## Required later proof

Implementation planning must include:

- exact equivalence such as `5.08mm == 200mil == 0.2in == 5_080_000nm`;
- negative values, signed limits, overflow, whitespace/case policy, Unicode
  `µm` and ASCII `um`, malformed input, and sub-nanometer refusal;
- format/parse round trips wherever the selected representation is exact;
- proof that display precision and system changes leave stored bytes unchanged;
- identical explicit-suffix results through GUI, CLI, and MCP adapters;
- contextual bare-number proof naming the resolved field/unit;
- migration compatibility for existing integer `_nm` callers.

The intentionally different examples `5mm`, `200mil`, and `0.1in` demonstrate
suffix acceptance; they are not asserted to be equivalent values.

<!-- EVIDENCE:GLOBAL-PREFERENCES-SPEC:SHARED-UNITS-ENGINE -->
