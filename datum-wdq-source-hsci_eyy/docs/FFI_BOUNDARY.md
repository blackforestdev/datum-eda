# C++ / FFI Boundary Strategy

> **Status**: Non-normative design rationale.
> No formal FFI spec exists in `specs/`. This document captures research
> and design reasoning about when and how C++ libraries should enter the
> build. Decisions here are proposals until ratified through implementation
> milestones.

## Purpose
Identifies where the project should use existing C++ libraries via FFI
rather than reimplementing in Rust, defines the integration architecture,
and specifies when each dependency enters the build.

## Design Principle
Rust owns the architecture. C++ enters only where **proven libraries
solve hard numerical/geometric problems** that would take person-years
to reimplement. The FFI boundary is explicit, narrow, and testable.

---

## 1. Where Rust Is Sufficient

The following subsystems are well-served by Rust and its ecosystem.
No C++ dependency is needed or beneficial:

| Subsystem | Rust approach | Notes |
|-----------|--------------|-------|
| Canonical IR | Native Rust structs + serde | Core strength |
| JSON serialization | serde_json | Best-in-class |
| SQLite pool index | rusqlite (bundled) | Mature, widely used |
| UUID generation | uuid crate | v4 + v5, production quality |
| KiCad S-expression parser | Custom parser or nom | Format is simple enough |
| Eagle XML parser | quick-xml or roxmltree | Well-supported |
| Rule engine / expression evaluator | Native Rust enum + match | Pattern matching is ideal |
| Operation model / undo-redo | Native Rust traits | Ownership model fits perfectly |
| ERC | Native Rust | Graph analysis on in-memory data |
| DRC (checks) | Native Rust | Geometry comparisons on integer coords |
| CLI | clap | Mature |
| Error handling | thiserror / anyhow | Standard |
| Property-based testing | proptest | Excellent |
| Board connectivity graph | petgraph or custom | Well-suited |
| MCP server (Python side) | N/A — Python, not Rust | MCP SDK is Python |
| Python bindings | PyO3 | Mature Rust↔Python bridge |

---

## 2. Where C++ Libraries Add Value

### 2.1 Polygon Boolean Operations (Clipper2)

**What**: Union, intersection, difference, XOR of polygons. Polygon
offsetting (inflation/deflation).

**Where used**:
- Copper pour fill computation (M4): subtract clearance-inflated obstacles
  from zone boundary to get fill geometry
- Clearance corridor construction (M5): inflate obstacles by clearance
  distance for routing obstacle map
- DRC clearance checking: compute distance between complex polygon shapes
- Board outline operations: combine polygon cutouts with outline

**Why not reimplement**: Polygon clipping with robust handling of
degeneracies (coincident edges, touching vertices, self-intersecting
input) is notoriously difficult. Clipper2 has been refined for over a
decade and is used by KiCad, Horizon EDA, and many other tools.

**Library**: [Clipper2](https://github.com/AngusJohnson/Clipper2)
- License: Boost Software License 1.0 (permissive, no copyleft)
- Language: C++ with C API (clipper2c)
- Size: ~15K LOC
- Maturity: Production-proven in multiple EDA tools

**Rust integration options**:
1. **clipper2c-sys**: Rust bindings to Clipper2's C API. Minimal FFI surface.
2. **i_overlay**: Pure Rust polygon boolean library. Less mature than Clipper2
   but avoids FFI entirely. Worth evaluating at M4 to see if it handles
   EDA-scale polygon operations reliably.

**Recommendation**: Start with i_overlay (pure Rust). If it fails on
edge cases from real designs in the test corpus, fall back to Clipper2
via clipper2c-sys.

**Enters at**: M4 (copper pour) or M5 (routing obstacle inflation)

### 2.2 Computational Geometry (CGAL — conditional)

**What**: Voronoi diagrams, Delaunay triangulation, Minkowski sums,
convex hull, arrangements.

**Where used**:
- Minkowski sum for clearance corridor computation (M5): inflate obstacles
  by clearance amount for routing search
- Delaunay triangulation for routing graph construction (M5): Eagle and
  Horizon both use Delaunay for routing topology
- Voronoi for congestion estimation (M5): partition board into routing
  regions

**Why not reimplement**: CGAL represents decades of computational geometry
research with rigorous handling of numerical precision. Implementing
robust Minkowski sums on integer coordinates is a research-level task.

**Library**: [CGAL](https://www.cgal.org/)
- License: LGPL v3 (some packages GPL, but Minkowski/Delaunay are LGPL)
- Language: C++ (header-heavy, template-heavy)
- Size: Large (only link needed packages)
- Maturity: The gold standard

**Rust integration**:
- CGAL is template-heavy C++ — direct FFI is impractical
- Approach: write a thin C wrapper exposing only the functions needed
  (minkowski_sum_polygon, delaunay_triangulation, voronoi_diagram)
- Use `cc` or `cmake` crate to build the C wrapper
- Alternative: evaluate Rust-native options first (see below)

**Rust alternatives**:
- `spade`: Rust Delaunay triangulation. Mature, well-tested.
- `geo`: Rust geometry crate. Convex hull, bounding rect, area, etc.
  Does NOT have Minkowski sum.
- `robust`: Rust exact arithmetic predicates (orient2d, incircle).
  Foundation for robust geometric algorithms.

**Recommendation**: Use `spade` for Delaunay, `geo` for basic geometry.
Evaluate whether Minkowski sum can be decomposed into polygon offset
(Clipper2) for the routing use case. Only bring CGAL if Minkowski sum
on non-convex polygons is truly needed and cannot be approximated.

**Enters at**: M5 (layout engine), and only if pure-Rust alternatives
prove insufficient

### 2.3 BREP Kernel / STEP Export (OpenCASCADE)

**What**: Boundary representation solid modeling. STEP file read/write.
3D shape construction from 2D profiles (extrude, revolve, boolean).

**Where used**:
- STEP export (M8): generate 3D model of the PCB assembly
- 3D collision checking (M8): component height restrictions
- 3D viewer (M7): render the board in 3D

**Why not reimplement**: BREP kernels are among the most complex software
systems in existence. STEP is an ISO standard with thousands of pages.
Nobody writes a BREP kernel from scratch for a PCB tool.

**Library**: [OpenCASCADE Technology (OCCT)](https://dev.opencascade.org/)
- License: LGPL v2.1 (compatible with any project license via dynamic linking)
- Language: C++
- Size: Very large (but only link needed modules)
- Maturity: Industry standard, used by FreeCAD, Horizon EDA, gmsh

**Rust integration**:
- `opencascade-rs`: Existing Rust bindings (cxx-based). Actively maintained.
- Wraps OCCT's shape construction, STEP import/export, meshing

**Recommendation**: Use opencascade-rs when STEP export is implemented.
No earlier dependency needed.

**Enters at**: M8 (3D/STEP export)

### 2.4 Field Solver (conditional)

**What**: 2D electromagnetic field solver for transmission line impedance
calculation from cross-section geometry.

**Where used**:
- Impedance-aware layer stack manager (M8): compute trace width for
  target impedance given stackup geometry and materials
- Impedance validation (M6): verify that routed traces meet impedance
  targets

**Options**:
- **Closed-form equations** (IPC-2141, Wadell): adequate for standard
  microstrip/stripline/coplanar structures. Pure Rust, no FFI needed.
  Accuracy: ±5-10% vs. field solver.
- **atlc2** (C): 2D Laplace solver. Simple, fast, open-source.
  License: GPL v2 — would need process isolation (subprocess, not linking).
- **openEMS** (C++/Matlab): full 3D FDTD solver. Overkill for PCB
  impedance but can handle exotic structures.
- **Custom MoM solver**: Method of Moments 2D solver for transmission
  line impedance. ~500 LOC of numerical code. Feasible in Rust.

**Recommendation**: Use closed-form equations (Wadell) for M6. They
cover 95% of real-world cases (microstrip, stripline, coplanar waveguide).
Consider a custom Rust MoM solver for M8 if higher accuracy is needed.
No C++ dependency for impedance calculation.

**Enters at**: M6 (closed-form, pure Rust), M8 (field solver if needed)

---

## 3. FFI Architecture

When a C++ library is used, the integration follows this pattern:

```
Rust code (engine crate)
    │
    │ Rust-safe wrapper (safe API, Rust types)
    │
    ├── sys crate (unsafe FFI bindings)
    │   │
    │   └── C API wrapper (if library has no C API)
    │       │
    │       └── C++ library (Clipper2, CGAL, OCCT)
    │
    └── build.rs (compile C/C++ code, link library)
```

### Rules for FFI boundaries:

1. **Narrow interface**: expose only the functions actually needed, not
   the entire library API
2. **Rust types at the boundary**: convert to/from Rust types (Point, Polygon,
   etc.) immediately at the FFI layer — no C++ types leak into Rust code
3. **Error handling**: C++ exceptions caught at the FFI boundary, converted
   to Rust Results
4. **Memory ownership**: clear ownership rules — Rust allocates and frees
   Rust memory, C++ allocates and frees C++ memory, no cross-boundary
   ownership transfer
5. **No C++ in the hot path if avoidable**: batch operations across FFI
   (send a polygon, get a polygon back) rather than per-vertex calls
6. **Separate sys crate**: FFI bindings live in a `*-sys` crate, isolated
   from engine logic
7. **Feature-gated**: C++ dependencies are cargo features, not mandatory.
   The engine compiles and passes core tests without any C++ dependency.

### Workspace impact

```
crates/
├── engine/              # Pure Rust. No C++ dependency.
├── engine-geo/          # Geometry operations. May depend on clipper2-sys.
│                        # Feature-gated: "clipper2" or "pure-rust"
├── engine-3d/           # STEP export. Depends on opencascade-rs.
│                        # Separate crate, not compiled unless needed.
├── clipper2-sys/        # Raw FFI bindings to Clipper2 (if needed)
├── cli/
├── engine-daemon/
└── test-harness/
```

The core engine crate never directly depends on C++ code. Geometry
operations that may use C++ are in a separate crate with feature flags.

---

## 4. Build System Implications

### C++ toolchain requirement
When C++ dependencies are enabled:
- Requires a C++ compiler (g++ or clang++)
- Requires cmake (for OCCT, CGAL)
- Documented in build prerequisites

### Feature flags
```toml
[features]
default = ["pure-rust-geo"]
pure-rust-geo = ["dep:i_overlay", "dep:spade"]
clipper2 = ["dep:clipper2-sys"]
step-export = ["dep:opencascade-rs"]
```

### CI builds
- Default CI: pure Rust, no C++ toolchain needed
- Extended CI: with C++ features enabled, tests run against C++ libraries
- Golden tests compare pure-Rust and C++ geometry results for parity

---

## 5. Decision Timeline

| Milestone | Decision | Dependencies |
|-----------|----------|-------------|
| M0-M2 | None — pure Rust | No C++ |
| M4 | Evaluate i_overlay vs. Clipper2 for copper pour | Clipper2 only if i_overlay fails |
| M5 | Evaluate spade vs. CGAL for routing graph | CGAL only if spade + custom code insufficient |
| M6 | Closed-form impedance (Wadell equations) | No C++ |
| M7 | wgpu for rendering (pure Rust) | No C++ |
| M8 | OpenCASCADE for STEP export | opencascade-rs |
| M8 | Field solver decision | Rust MoM or external solver |

The project can reach M7 (GUI) without any C++ dependency. C++ enters
definitively only at M8 (STEP export via OpenCASCADE), and even earlier
dependencies are conditional on pure-Rust alternatives proving insufficient.

---

## Milestone Position
- M0-M3: Pure Rust. No C++ in the build.
- M4: First geometry evaluation point (copper pour). Prefer pure Rust.
- M5: Second geometry evaluation point (routing). Prefer pure Rust.
- M8: OpenCASCADE for STEP. Definitive C++ dependency.
