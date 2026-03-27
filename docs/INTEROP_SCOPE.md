# Interoperability Scope

Scope terminology follows `specs/PROGRAM_SPEC.md` §Scope Integrity Terms.

`docs/R1_G0_FOUNDATION.md` defines the minimum history/context gate for
downstream interop claims. This document consumes that foundation; it does not
replace it.

## Principle
For v1, imported formats are first-class citizens in the current execution
slice. The engine does not
require designs to be in its native format. A KiCad project opened via
import is as fully functional as a native project for all supported
operations (query, DRC, limited modification, export).

This interoperability-first slice is an implementation strategy, not the
long-term product boundary.

Native format is introduced in M4. Until then, the canonical IR exists
only in memory and in golden test files.

Interop interpretation rule:
- distinguish the currently implemented slice from broad migration claims
- classify fidelity as `exact`, `approximated`, `preserved-as-metadata`, or
  `unsupported`
- do not treat "file opens" as sufficient evidence of faithful migration

## Import Formats

### KiCad (Priority 1)
- **Files**: `.kicad_pcb`, `.kicad_sch`, `.kicad_pro`, `.kicad_sym`, `.kicad_mod`
- **Versions**: KiCad 7, 8, 9 (S-expression format, version-tagged)
- **Parser**: Custom S-expression parser in Rust
- **Fidelity target**: 95%+ of design data imports correctly for boards
  with ≤ 4 layers and standard features
- **Known gaps** (acceptable for v1):
  - Custom constraint rules (KiCad's advanced rule system)
  - Teardrops (KiCad 8+, can import as track geometry)
  - 3D model assignments (deferred — no 3D in v1)
  - Custom fonts (substitute with default)

### Eagle (Priority 1)
- **Files**: `.brd`, `.sch`, `.lbr` (all XML, DTD documented in research)
- **Versions**: Eagle 6.x through 9.6.2
- **Parser**: XML parser using eagle.dtd schema (CC BY-ND 3.0 licensed)
- **Fidelity target**: 90%+ for board geometry and connectivity
- **Known gaps**:
  - ULP-generated content (no ULP runtime)
  - Fusion 360 cloud references (URN-based assets, inaccessible)
  - DRC rules (Eagle's rule system is different; import as approximate)

### Altium (Future — M5+)
- **Files**: `.PcbDoc`, `.SchDoc` (OLE compound documents)
- **Complexity**: High — binary format, poorly documented
- **Approach**: staged commercial migration path; see
  `docs/COMMERCIAL_INTEROP_STRATEGY.md` and `docs/R1_G0_FOUNDATION.md`
- **Not in v1 scope**

### Commercial Tools (Future — M5+)
- **Targets**: Altium, PADS, OrCAD/Allegro
- **Priority**: migration support after KiCad/Eagle foundation is proven
- **Approach**:
  - research and corpus gathering before implementation
  - library extraction before full design import
  - read-only migration before write-back claims
  - explicit fidelity/loss reporting
- **Reference**: `docs/COMMERCIAL_INTEROP_STRATEGY.md`,
  `docs/R1_G0_FOUNDATION.md`

## Export Formats

### Manufacturing (M4)
- **Gerber RS-274X**: Copper, mask, silk, paste layers
- **Gerber X2**: Extended attributes (net names, component refs)
- **Excellon**: Drill files (plated + non-plated)
- **BOM**: CSV and JSON
- **Pick and Place**: CSV (reference, value, x, y, rotation, layer)

### Design Interchange (M3-M4)
- **KiCad .kicad_pcb write-back**: Save modifications back to KiCad format
  - Goal: import → modify → export → opens cleanly in KiCad
  - Acceptable: formatting changes, reordered sections
  - Unacceptable: data loss, coordinate drift, broken connectivity
  - Fidelity framing: apply `R1-G0` policy categories; successful open is
    necessary but not sufficient
- **Native JSON format**: Canonical IR serialized (introduced in M4)

### Future (M5+)
- ODB++
- STEP (3D)
- PDF (documentation)
- IPC-2581

## Library/Pool Interop

### Eagle Library Import (M0)
- Parse .lbr XML → create pool items (unit, entity, part, package)
- 300+ libraries ship with Eagle 9.6.2 (available in research/eagle-analysis)
- Validates the pool data model against a large real-world library

### KiCad Library Import (M1)
- Parse .kicad_sym (symbols) and .kicad_mod (footprints)
- Map to pool structure (KiCad's flat model → pool's entity/part model)
- This mapping is lossy by definition (KiCad has no "Part" concept)
  — create Parts from symbol+footprint pairs found in designs

### Pool as interchange
The pool's JSON format could become a library interchange format, but
this is not a v1 goal. For now, the pool is an internal database
populated by importers.

## Fidelity Measurement

Each importer tracks:
- **Features tested**: how many KiCad/Eagle features are covered by tests
- **Features passing**: how many import correctly
- **Fidelity %**: passing / tested
- **Untested features**: known features with no test coverage

Fidelity is reported per-design in the test corpus:
```
DOA2526.kicad_pcb:
  Components: 24/24 (100%)
  Tracks: 187/187 (100%)
  Vias: 12/12 (100%)
  Zones: 4/4 (100%)
  Net classes: 2/2 (100%)
  Design rules: 3/5 (60% — custom constraints not imported)
  Overall: 97%
```
