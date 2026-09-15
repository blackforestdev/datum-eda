# Test Strategy

## Principle
The engine is trusted when it produces correct results on real designs,
not when unit tests pass on synthetic data. Real designs are the test.

## Test Layers

### 1. Unit Tests (per-module)
Standard Rust `#[test]` functions. Cover:
- Serialization round-trips (struct → JSON → struct == original)
- Coordinate math (unit conversion, rotation, mirroring)
- Pool queries (insert → search → verify results)
- Rule matching (scope expression → matching objects)

### 2. Golden Tests (per-design)
A golden test takes a real design file, processes it, and compares
the output against a checked-in reference ("golden") file.

```
input: designs/doa2526.kicad_pcb
operation: import → serialize to canonical JSON
golden: golden/doa2526.canonical.json
test: output == golden (byte-identical)
```

Golden tests catch:
- Import regressions (parser changes break existing designs)
- Serialization drift (output format changes unexpectedly)
- Derived data changes (connectivity, airwires computed differently)

Golden files are updated deliberately, never automatically.

### 3. DRC Comparison Tests
Import a design, run DRC, compare violations against a reference.

For KiCad designs: also run KiCad's DRC on the same file and compare
results. Differences are either our bugs or KiCad's bugs — investigate
each one.

### 3b. ERC Comparison Tests
Import a schematic, resolve schematic connectivity, run ERC, compare
violations against a checked-in reference.

For vetted KiCad schematics: compare against KiCad ERC when feature parity
exists. Differences are investigated individually.

### 4. Round-Trip Tests
Import → export → re-import → compare.
The re-imported design must be structurally identical to the first import.
This validates that export doesn't lose information.

For KiCad: import .kicad_pcb → export .kicad_pcb → diff.
Acceptable differences: formatting, comment ordering, whitespace.
Unacceptable: any data difference.

### 5. Property-Based Tests (geometry)
Use proptest (Rust property testing) for geometry operations:
- Random polygons: clip result is valid polygon
- Random point sets: Delaunay triangulation is valid
- Random track pairs: clearance computation is symmetric
- Random coordinates: unit conversion round-trips exactly

## Test Corpus

### Minimum viable corpus (M0-M1)
- 20 Eagle .lbr files (from Eagle 9.6.2 shipped libraries)
- 5 KiCad designs of varying complexity:
  - DOA2526 (discrete opamp — primary test case)
  - A simple single-sided board (< 20 components)
  - A medium 2-layer board (50-100 components)
  - A 4-layer board with planes and diff pairs
  - A large board (500+ components) for performance testing
- 5 Eagle designs (.brd + .sch)
- 5 schematics with known ERC edge cases:
  - output-to-output conflict
  - undriven input
  - power net with no source
  - hierarchical port mismatch
  - no-connect pin actually connected

### Growth target
50+ real designs by M2. Sourced from:
- Open-source hardware projects
- User-contributed designs (with permission)
- Synthetic test cases for edge cases

### Corpus management
Test designs live in `tests/corpus/`. Each design has:
- Source files (original format)
- Golden files (expected canonical output)
- Metadata (source, format version, known issues)
- DRC reference (expected violations or clean pass)

### 6. Routing Benchmarks (M5+)
Objective metrics for routing quality, measured on the test corpus:

```
Per-design routing report:
  Completion rate:      187/187 nets (100%)
  Total vias:           24 (reference design: 22)
  Total wire length:    1,847mm (reference: 1,792mm — +3.1%)
  Max clearance viol:   0
  Length match skew:    DQ byte 0: 0.3mm (target ±0.5mm) ✓
  Diff pair skew:       USB: 0.02mm (target ±0.1mm) ✓
  Runtime:              4.2s on [hardware spec]
  Stability:            identical output on 10 runs with same seed
```

Benchmarks tracked over time. Regressions block merge.
Comparison against the same design routed by KiCad or hand-routed
reference provides the quality baseline.

## CI Requirements
- All tests pass on every commit
- Golden test failures block merge (deliberate update required)
- DRC comparison tests run nightly (too slow for every commit)
- ERC comparison tests run nightly (too slow for every commit)
- Performance benchmarks tracked over time (import time, DRC time)

## Fidelity Tracking

Each import format has a fidelity matrix documenting which features
import correctly:

```
KiCad Import Fidelity:
  [x] Components (position, rotation, layer, reference, value)
  [x] Tracks (width, layer, net assignment)
  [x] Vias (drill, diameter, net, layer span)
  [x] Zones (polygon, net, priority, thermal settings)
  [x] Net classes (clearance, width rules)
  [ ] Custom design rules (KiCad's constraint system)
  [x] Board outline
  [x] Keepouts
  [ ] Teardrops (KiCad 8+)
  [ ] Tuning patterns (stored as tracks, lose tuning metadata)
  ...
```

This matrix is a living document updated as import coverage improves.
