# Eagle 9.6.2 Architectural Blueprint
## Reverse-Engineered from libSuits.so Symbols, DTD, and File Analysis

### Purpose
This document captures the organizational DNA of Eagle's EDA engine as a reference
for building a modern, AI-native, Linux-first EDA tool. This is not a crack or
reimplementation — it's an architectural study of what made Eagle work.

---

## 1. Core Engine Architecture (libSuits.so — 9.5MB, unstripped)

### Complexity Map — Methods per Class (Top 20)
The method count reveals where the real engineering complexity lives:

| Class | Methods | Role |
|-------|---------|------|
| **CGeoComputer** | 187 | Computational geometry engine — the mathematical core |
| **CPush** | 140 | Push-and-shove routing engine |
| **CCMDDrive** | 118 | **Command interpreter** — Eagle's CLI brain |
| **CEditer** | 116 | Interactive editor / UI state machine |
| **CRouter** | 104 | Autorouter |
| **CCriticer** | 96 | DRC engine (design rule checker) |
| **CSelecter** | 78 | Selection/pick engine |
| **CNet** | 78 | Net/connectivity management |
| **CPairPostProcess** | 76 | Differential pair post-processing |
| **Checker** | 71 | ERC engine (electrical rule checker) |
| **CEqualLength** | 59 | Length-matched routing |
| **CTBFanout** | 57 | BGA/QFP fanout engine |
| **CTune** | 52 | Interactive trace tuning (meander) |
| **CPinsTemplate** | 52 | Pin template/pattern management |
| **CDelaunayManager** | 49 | Delaunay triangulation (for routing graph) |
| **CGridBoxTable** | 46 | Spatial indexing (grid-based acceleration) |
| **CCriticerCtrl** | 46 | DRC rule configuration/control |

### Key Architectural Insight
Eagle's engine has **6 major subsystems**:

1. **Geometry Engine** (CGeoComputer, CDelaunayManager, CTriangle, CTriangleObj)
   - Delaunay triangulation for routing topology
   - Computational geometry for shape operations
   - Spatial indexing via CGridBoxTable, CQuadTree

2. **Routing Engine** (CRouter, CPush, CRouteControler, CRouteEdge, CBackTrace)
   - A* pathfinding (CAStar)
   - Push-and-shove (CPush — 140 methods, very sophisticated)
   - Template routing (CTemplateRoute)
   - Differential pair routing (CPairPostProcess)
   - Length matching (CEqualLength)
   - Interactive tuning (CTune — meander)
   - BGA fanout (CTBFanout)
   - Guide-based routing (CGuide, CGuideTree, CGuideZone)

3. **Design Rule Engine** (CCriticer, CCriticerCtrl, CRuleManager, CRule, CRuleTable)
   - Per-net-class clearance computation
   - Region-based rule overrides
   - Shape-to-shape clearance calculations
   - Width-by-net-layer-point resolution

4. **Command System** (CCMDDrive — 118 methods)
   - Text command parser and executor
   - This IS Eagle's CLI — the thing that made it fast
   - Drives the editor state machine

5. **Data Model** (CNet, CPin, CVia, CWire, CComponent, CLibrary, CPCB, CPadStack)
   - PCB object hierarchy
   - Net connectivity with island detection
   - Component/footprint/symbol relationships
   - Layer management (CLayer, CLayerManager)

6. **Editor System** (CEditer, CSelecter, various *Editor classes)
   - Interactive editing with undo
   - Component editing (CCompEditor, CPKGEditor)
   - Boundary editing (CBoundaryEditor)
   - Image/padstack editing

---

## 2. Data Model (from eagle.dtd — formal file format)

### Entity Hierarchy

```
eagle (root)
├── drawing
│   ├── settings
│   ├── grid
│   ├── layers[]
│   │   └── layer (number, name, color, fill, visible, active)
│   │
│   ├── library (standalone .lbr files)
│   │   ├── packages[]
│   │   │   └── package (name)
│   │   │       └── polygon | wire | text | dimension | circle | rectangle | hole | pad | smd
│   │   ├── symbols[]
│   │   │   └── symbol (name)
│   │   │       └── polygon | wire | text | dimension | pin | circle | rectangle
│   │   └── devicesets[]
│   │       └── deviceset (name, prefix, uservalue)
│   │           ├── gates[]
│   │           │   └── gate (name → symbol, x, y, addlevel, swaplevel)
│   │           ├── devices[]
│   │           │   └── device (name → package)
│   │           │       ├── connects[] (gate.pin → pad)
│   │           │       └── technologies[] (variants, attributes)
│   │           └── spice (pinmapping, model)
│   │
│   ├── schematic (.sch files)
│   │   ├── libraries[] (embedded copies)
│   │   ├── attributes[]
│   │   ├── variantdefs[]
│   │   ├── classes[] (net classes with clearance matrix)
│   │   ├── parts[]
│   │   │   └── part (name → library.deviceset.device.technology)
│   │   └── sheets[]
│   │       └── sheet
│   │           ├── instances[] (part.gate placed at x,y)
│   │           ├── nets[]
│   │           │   └── net (name, class)
│   │           │       └── segment[] → pinref | wire | junction | label
│   │           └── busses[]
│   │
│   └── board (.brd files)
│       ├── libraries[] (embedded copies)
│       ├── designrules (named params)
│       ├── classes[] (net classes with clearance matrix)
│       ├── elements[]
│       │   └── element (name → library.package, x, y, value, rot, locked, smashed)
│       └── signals[]
│           └── signal (name, class)
│               └── contactref | polygon | wire | via
```

### The deviceset→device→connect Chain (Eagle's Part Model)

This is Eagle's version of what Horizon calls the pool system:

```
deviceset "RESISTOR" (prefix="R")
  ├── gate "G$1" → symbol "R-US" (the schematic symbol)
  └── device "" → package "R0402"
      └── connect: gate "G$1" pin "1" → pad "1"
      └── connect: gate "G$1" pin "2" → pad "2"
  └── device "" → package "R0603"
      └── connect: gate "G$1" pin "1" → pad "1"
      └── connect: gate "G$1" pin "2" → pad "2"
```

**Key difference from KiCad:** Eagle explicitly binds pin-to-pad inside the device.
KiCad requires you to create separate symbols or use awkward pin mapping.
This is the architectural choice that Eagle got right and KiCad still gets wrong.

---

## 3. Command Language (extracted from binary strings)

### Core Commands (Eagle CLI)
```
ADD       - Place component from library
ARC       - Draw arc
ATTRIBUTE - Edit part attributes
AUTO      - Autorouter
BOARD     - Switch to board editor / create board from schematic
BUS       - Draw bus
CHANGE    - Modify object properties
CIRCLE    - Draw circle
CLASS     - Define/edit net classes
CLOSE     - Close editor
CONNECT   - Connect gate pins to pads (in library editor)
COPY      - Copy objects
CUT       - Cut to clipboard
DELETE    - Delete objects
DESCRIPTION - Edit descriptions
DISPLAY   - Control layer visibility
DRC       - Run design rule check
EDIT      - Open/create symbol/package/device
ERC       - Run electrical rule check
EXPORT    - Export data (netlist, BOM, image, etc.)
GATE      - Add gate to deviceset
GROUP     - Create selection group
HOLE      - Place mounting hole
INFO      - Show object properties
INVOKE    - Place additional gates
JUNCTION  - Place net junction
LABEL     - Place net label
LAYER     - Create/modify layers
LOCK      - Lock component position
MARK      - Set reference mark
MEANDER   - Create meander trace (length tuning)
MITER     - Miter wire corners
MIRROR    - Mirror objects
MOVE      - Move objects
NAME      - Rename objects
NET       - Draw net
OPEN      - Open file
OPTIMIZE  - Optimize wire paths
PAD       - Place through-hole pad
PACKAGE   - Define package
PASTE     - Paste from clipboard
PIN       - Place pin (in symbol editor)
PINSWAP   - Swap electrically equivalent pins
POLYGON   - Draw polygon (copper pour)
PREFIX    - Set reference prefix
RATSNEST  - Recalculate airwires
RECT      - Draw rectangle
REDO      - Redo last undo
REMOVE    - Remove from library
RENAME    - Rename objects
REPLACE   - Replace package
RIPUP     - Remove routed traces
ROTATE    - Rotate objects
ROUTE     - Route airwire
RUN       - Execute ULP script
SCRIPT    - Execute command script
SET       - Set editor parameters (SET TOGGLE_CLI;)
SHOW      - Highlight nets/objects
SIGNAL    - Define signal in board
SMD       - Place SMD pad
SPLIT     - Split wire
SUPPLY    - Define supply symbol
SYMBOL    - Define symbol
TECHNOLOGY - Define technology variants
TEXT      - Place text
UNDO      - Undo last action
UPDATE    - Update library parts
USE       - Load library
VALUE     - Set component value
VIA       - Place via
WINDOW    - Control view
WIRE      - Draw wire/trace
WRITE     - Save file
```

### Command Pattern
Commands follow: `COMMAND [qualifier] [target] [parameters]`
Examples from binary strings:
- `SET TOGGLE_CLI;` — toggle command line
- `CHANGE %1` — change property
- `NET %1 will be added to the BUS`
- `GROUP ALL`

---

## 4. ULP Scripting Object Model

### Accessible Objects (UL_ prefix types)
```
UL_ARC           UL_LIBRARY       UL_POLYGON
UL_AREA          UL_MODULE        UL_RECTANGLE
UL_ATTRIBUTE     UL_MODULEINST    UL_SCHEMATIC
UL_BOARD         UL_NET           UL_SHEET
UL_BUS           UL_PACKAGE       UL_SIGNAL
UL_CIRCLE        UL_PACKAGE3D     UL_SMD
UL_CLASS         UL_PAD           UL_SYMBOL
UL_CONTACT       UL_PART          UL_TEXT
UL_CONTACTREF    UL_PIN           UL_VARIANT
UL_DEVICE        UL_PINREF        UL_VARIANTDEF
UL_DEVICESET     UL_POLYGON       UL_VIA
UL_ELEMENT       UL_RECTANGLE     UL_WIRE
UL_GATE
UL_HOLE
UL_INSTANCE
UL_LAYER
```

### ULP Language Characteristics (from example ULPs)
- C-like syntax with `string`, `int`, `real` types
- `#require` version directives
- `#usage` documentation strings (multilingual)
- `board()`, `schematic()`, `library()` context functions
- Iterator pattern: `board(B) { B.elements(E) { E.package.contacts(C) { ... } } }`
- Dialog system for GUI: `dlgDialog()`, `dlgListView()`, etc.
- File I/O: `output()`, `fileerror()`
- Command execution: `exit()` with command string argument
- `argc`/`argv` for command line arguments

---

## 5. File Formats Summary

| Format | Type | Structure | Status |
|--------|------|-----------|--------|
| .sch | XML | DTD-defined, human-readable | Fully documented |
| .brd | XML | DTD-defined, human-readable | Fully documented |
| .lbr | XML | DTD-defined, human-readable | Fully documented |
| .dru | Text | Named params (design rules) | Embedded in .brd as `<designrules>` |
| .cam | ? | CAM job definition | Needs analysis |
| .ulp | Text | C-like scripting language | Fully readable source |
| .scr | Text | Eagle command scripts | Fully readable source |

**All design data is XML with a formal DTD.** This is 100% parseable and convertible.

---

## 6. What a Modern Successor Needs

Based on this architectural analysis, a "liberated Eagle" needs:

### Must Have (Eagle's DNA)
1. **Interactive command line** (CCMDDrive equivalent) — natural language evolution
2. **Proper part model** (deviceset→device→connect) — not KiCad's flat mapping
3. **Push-and-shove routing** (CPush) — 140-method sophistication
4. **Integrated DRC** (CCriticer) — rule-aware from the start
5. **Scriptable** (ULP equivalent) — Python, not a custom language
6. **XML/structured data** — machine-readable, diffable, AI-parseable

### Must Improve (Eagle's Weaknesses)
1. **Library management** — Eagle's was file-based; use Horizon's pool model
2. **Multi-board / hierarchy** — Eagle's module system was bolted on late
3. **3D integration** — Eagle's was Fusion-dependent; use STEP natively
4. **Version control** — XML diffs are ugly; consider structured formats
5. **Collaboration** — Eagle had none; build for multi-user from start

### Must Add (2026+ Requirements)
1. **AI command interface** — "route these diff pairs 100Ω" not `ROUTE`
2. **AI-assisted placement** — intent-driven, constraint-based
3. **AI DRC explanation** — "why does this fail?" not just error markers
4. **Structured data model** — graph database, not flat files (for AI reasoning)
5. **API-first** — every operation available programmatically
6. **Linux-native** — Wayland, GPU-accelerated rendering
7. **Import everything** — Eagle XML, KiCad, Altium (via intermediate)

---

## 7. Technology Stack Recommendation

For a modern reimplementation:

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Language | Rust + Python bindings | Memory safety for geometry, Python for scripting |
| GUI | wgpu + egui or Slint | GPU-accelerated, Wayland-native, no Qt dependency |
| Data model | SQLite + structured schema | Queryable, transactional, AI-friendly |
| File format | SQLite (native) + XML (interchange) | Fast native, portable interchange |
| Geometry | Rust port of CGAL concepts | Computational geometry foundation |
| Routing | Custom (Delaunay + A* + push-shove) | Eagle's proven architecture |
| CLI | Tree-sitter grammar + LLM adapter | Traditional commands + natural language |
| Scripting | Python (via PyO3) | Universal, AI-ecosystem compatible |
| Library mgmt | Pool model (Horizon-inspired) | Proper entity relationships |
| 3D | OpenCascade (via opencascade-rs) | STEP native, no external deps |

---

## Appendix: File Inventory

- 1785 files total in distribution
- 741 example files (ULPs, projects, scripts, CAM jobs, design rules)
- 306 cached library files (.lbr, XML)
- 383 bin/ files (resources, translations, icons)
- 156 ngspice files (bundled simulation)
- 22MB main binary (eagle, stripped ELF x86-64)
- 9.5MB libSuits.so (EDA engine, NOT stripped — 240 classes, ~2400 methods)
- 63 shared libraries (bundled Qt5, ICU, X11, NSS)
- 1 eagle.dtd (formal file format specification, CC BY-ND 3.0 licensed)
