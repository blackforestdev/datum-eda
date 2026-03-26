# Pool Architecture — Design Rationale

> **Status**: Non-normative design rationale.
> The controlling pool types are defined in `specs/ENGINE_SPEC.md` §1.2.
> The controlling pool index schema is in `docs/CANONICAL_IR.md` §8.
> This document provides architectural context, storage design reasoning,
> and query interface design. If promoted, it should move to `specs/`.

## Purpose
Documents the design of the component library system — how parts,
packages, entities, symbols, and units are stored, indexed, queried,
versioned, and shared. The pool is the foundation that connects schematic
symbols to physical footprints to purchasable components.

## Architectural Lineage
- **Horizon EDA**: Pool concept, entity hierarchy, SQLite index, JSON files
- **Eagle**: deviceset→device→connect chain (explicit pin-to-pad binding)
- **Altium**: Unified component model, lifecycle metadata, supply chain data

This pool takes Horizon's structural separation (Unit/Entity/Part/Package)
and extends it with Eagle's explicit binding model and Altium's
professional metadata (lifecycle, supply chain, parametric search).

---

## 1. Entity Hierarchy

The pool separates electrical identity from physical form from purchasable
identity. This separation enables:
- One entity (e.g., "op-amp") to have multiple packages (SOIC-8, DFN-8, MSOP-8)
- One package (e.g., 0402) to serve thousands of different parts
- Multi-gate components (e.g., quad NAND — one entity, four gates, one package)
- Part variants (same entity+package, different values or manufacturers)

```
Unit (electrical identity of one gate)
  │  "What pins does it have?"
  │
  ├──→ Symbol (schematic visual representation)
  │     "How does it look on a schematic?"
  │
  └──→ Entity (multi-gate logical component)
        │  "How many gates, what prefix?"
        │
        ├──→ Package (physical footprint)
        │     │  "What pads, what courtyard, what silkscreen?"
        │     │
        │     └──→ Padstack (pad geometry — through-hole, SMD, etc.)
        │
        └──→ Part (the purchasable thing)
              "What MPN, what value, what package, which pin goes where?"
```

### Unit
The electrical identity of a single gate. Defines pins with direction,
swap groups, and alternate names.

- **Identity**: UUID
- **Pins**: ordered list with electrical semantics (Input, Output, etc.)
- **Swap groups**: pins within the same non-zero swap group can be swapped
  by the optimizer without changing function (e.g., AND gate inputs)
- **Alternate names**: pins may have alternate function names (e.g., a GPIO
  pin that can also function as SPI_MOSI)

### Symbol
The visual representation of a Unit on a schematic. Graphics primitives
(lines, arcs, text) plus pin positions and orientations.

- **Identity**: UUID
- **References**: Unit UUID (one symbol represents one unit)
- **Content**: graphics primitives + pin stubs with pin UUIDs matching the Unit

### Entity
A multi-gate logical component. Maps gates to Units.

- **Identity**: UUID
- **Prefix**: reference designator prefix (R, C, U, Q, etc.)
- **Gates**: named gates, each referencing a Unit and a Symbol
- **Example**: A 74HC00 has Entity with 4 gates, each gate references the
  same "NAND2" Unit with the same Symbol. A TL074 has 4 gates each
  referencing the same "op-amp" Unit.
- **Single-gate**: Most components (resistors, capacitors, simple ICs)
  have one Entity with one Gate. The Entity→Gate→Unit chain still exists
  but is just one deep.

### Package
The physical footprint. Defines pads, courtyard, silkscreen, 3D models.

- **Identity**: UUID
- **Pads**: positioned pads, each with a padstack (geometry, layers)
- **Courtyard**: bounding polygon for assembly clearance checking
- **Silkscreen**: graphics for board markings
- **3D models**: references to STEP files (paths, not embedded)

### Padstack
The geometry of a single pad. Defines the copper shape, mask openings,
paste pattern per layer.

- **Types**: Through-hole (round, oblong), SMD (rectangular, rounded-rect,
  custom polygon), BGA (ball)
- **Per-layer**: Different copper shape on different layers (e.g.,
  through-hole pad with different annular ring on inner vs. outer layers)

### Part
The purchasable component. THE key abstraction — this is what gets placed
on a schematic, ordered from a distributor, and assembled onto a board.

- **Identity**: UUID
- **References**: Entity UUID + Package UUID
- **Pad map**: explicit mapping from Package Pad UUIDs to (Gate UUID, Pin UUID)
  pairs. This is the Eagle-style explicit binding that KiCad lacks.
- **Attributes**: MPN, value, manufacturer, datasheet URL, description
- **Parametric**: key-value pairs for parametric search (resistance,
  capacitance, voltage, tolerance, temperature coefficient, etc.)
- **Supply chain**: orderable MPNs, lifecycle status
- **Inheritance**: optional base Part UUID. Derived parts inherit all
  attributes from the base and override specific fields. Useful for
  value variants (same resistor series, different values).

---

## 2. Storage Model

### JSON files (source of truth)

Each pool object is stored as an individual JSON file in a directory tree:

```
pool/
├── units/
│   ├── resistor.json
│   ├── capacitor.json
│   └── opamp.json
├── entities/
│   ├── resistor.json
│   └── 74hc00.json
├── symbols/
│   ├── resistor.json
│   └── opamp.json
├── packages/
│   ├── 0402.json
│   ├── 0603.json
│   └── soic-8.json
├── padstacks/
│   ├── smd-rect.json
│   └── th-round.json
└── parts/
    ├── generic/
    │   └── resistor-0402-10k.json
    └── murata/
        └── grm155r71c104ka88j.json
```

**Why individual files:**
- Git-friendly: each part is a separate diff
- AI-parseable: read one file, get one complete object
- Merge-friendly: concurrent pool edits don't conflict unless they touch
  the same object
- Simple: no database migration, no schema versioning for storage

**File format:**
- JSON with deterministic serialization (sorted keys, stable format)
- Schema version in each file
- UUIDs as hyphenated lowercase strings
- Coordinates in nanometers

### SQLite index (derived, rebuildable)

The pool index is a SQLite database built from the JSON files. It exists
for fast querying — keyword search, parametric filtering, tag browsing.
It is **always rebuildable** from the JSON files. If the index is lost or
corrupted, rebuild from source.

```sql
-- Core tables
CREATE TABLE units (
    uuid TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    manufacturer TEXT,
    filename TEXT NOT NULL,
    n_pins INTEGER
);

CREATE TABLE entities (
    uuid TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    manufacturer TEXT,
    prefix TEXT NOT NULL,
    n_gates INTEGER NOT NULL,
    filename TEXT NOT NULL
);

CREATE TABLE symbols (
    uuid TEXT PRIMARY KEY,
    unit_uuid TEXT NOT NULL REFERENCES units(uuid),
    name TEXT NOT NULL,
    filename TEXT NOT NULL
);

CREATE TABLE packages (
    uuid TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    manufacturer TEXT,
    n_pads INTEGER NOT NULL,
    filename TEXT NOT NULL
);

CREATE TABLE padstacks (
    uuid TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    package_uuid TEXT REFERENCES packages(uuid),
    filename TEXT NOT NULL
);

CREATE TABLE parts (
    uuid TEXT PRIMARY KEY,
    mpn TEXT,
    manufacturer TEXT,
    entity_uuid TEXT NOT NULL REFERENCES entities(uuid),
    package_uuid TEXT NOT NULL REFERENCES packages(uuid),
    description TEXT,
    datasheet TEXT,
    value TEXT,
    parametric_table TEXT,  -- which parametric category (resistor, capacitor, etc.)
    base_uuid TEXT REFERENCES parts(uuid),
    lifecycle TEXT DEFAULT 'unknown',
    filename TEXT NOT NULL
);

CREATE TABLE orderable_mpns (
    part_uuid TEXT NOT NULL REFERENCES parts(uuid),
    mpn TEXT NOT NULL
);

-- Tagging (FTS for fast search)
CREATE TABLE tags (
    uuid TEXT NOT NULL,
    type TEXT NOT NULL,     -- 'unit', 'entity', 'package', 'part'
    tag TEXT NOT NULL
);

CREATE VIRTUAL TABLE tags_fts USING fts5(tag, content=tags, content_rowid=rowid);

-- Parametric tables (one per category)
CREATE TABLE parametric_resistors (
    part_uuid TEXT PRIMARY KEY REFERENCES parts(uuid),
    resistance TEXT,        -- stored as string for unit-aware search
    tolerance TEXT,
    power_rating TEXT,
    temperature_coefficient TEXT,
    package TEXT
);

CREATE TABLE parametric_capacitors (
    part_uuid TEXT PRIMARY KEY REFERENCES parts(uuid),
    capacitance TEXT,
    voltage TEXT,
    tolerance TEXT,
    dielectric TEXT,        -- C0G, X7R, X5R, Y5V
    package TEXT
);

-- Additional parametric tables for inductors, ICs, connectors, etc.
-- Created on demand as new parametric categories appear.
```

### Index rebuild

```
pool_index_rebuild(pool_path):
  1. Walk pool directory tree
  2. For each JSON file:
     a. Parse and validate
     b. Insert/update index tables
  3. Rebuild FTS index
  4. Verify referential integrity (all Entity→Unit, Part→Entity,
     Part→Package references are valid)
```

Rebuild is idempotent and safe to run at any time.

---

## 3. Query Interface

### Keyword search
Full-text search across name, MPN, manufacturer, description, and tags.

```
search_pool("100nF 0402 ceramic")
→ matches parts where FTS matches across all text fields
→ ranked by relevance
```

### Parametric search
Filtered search within a parametric category.

```
search_pool_parametric("resistor", {
    resistance: "10k",
    tolerance: "1%",
    package: "0402"
})
→ matches parts in parametric_resistors table
→ resistance comparison is unit-aware (10k = 10000)
```

### Tag browsing
Browse by tag hierarchy.

```
get_tags("part") → ["passive", "active", "connector", "mechanical", ...]
search_by_tag("passive", "resistor") → all parts tagged passive+resistor
```

### Reference lookup
Direct UUID lookup for any pool object.

```
get_part(uuid) → full Part with resolved Entity and Package
get_entity(uuid) → Entity with Gates and Units
get_package(uuid) → Package with Pads and Padstacks
```

---

## 4. Pool Population

### From imports
When a KiCad or Eagle design is imported:
1. Library data from the design creates pool objects
2. Eagle .lbr: deviceset→device→connect maps to Entity→Part→Package+pad_map
3. KiCad: symbol+footprint pairs create synthetic Parts
4. Imported pool objects get deterministic UUIDs (v5, import namespace)
5. Duplicate detection: if a pool object with the same UUID already exists,
   the import skips it (idempotent)

### From Eagle .lbr (M0)
Eagle's library format maps cleanly to the pool:

| Eagle concept | Pool concept |
|--------------|-------------|
| symbol | Symbol |
| package | Package |
| deviceset | Entity |
| gate | Gate |
| device | Part (one per device variant) |
| connect | pad_map entry |
| technology | Part variant (different attributes, same Entity+Package) |

### From KiCad libraries (M1)
KiCad's library model is flatter:

| KiCad concept | Pool concept |
|--------------|-------------|
| .kicad_sym symbol | Unit + Symbol + Entity (single-gate) |
| .kicad_mod footprint | Package |
| symbol+footprint pair (from design) | Part |

KiCad does not have a "Part" concept — the binding of symbol to footprint
happens at the design level, not the library level. The importer creates
Parts from the symbol-footprint pairs it finds in actual designs.

### Manual / scripted
Pool objects can be created directly:
- CLI: `tool pool create-part --entity ... --package ... --mpn ...`
- MCP: `create_part` tool (M4+)
- Python: pool API via PyO3

---

## 5. Pool Sharing and Distribution

### Local pool
Default: pool directory alongside the project. Self-contained, no
external dependencies.

### Shared pool (future)
A pool can be shared across projects via:
- Filesystem path reference in project settings
- Git submodule (pool is a git repository)
- Network pool (future: REST API for pool queries)

### Pool layering
A project can reference multiple pools with priority ordering:
1. Project-local pool (highest priority — project-specific overrides)
2. Organization pool (shared team library)
3. Base pool (shipped default library)

When searching, results are merged across all pools. When a UUID exists
in multiple pools, the highest-priority pool wins.

### Pool versioning
Pool JSON files are designed for git. Each modification to a pool object
is a file change that git tracks. Pool versioning IS git versioning — no
separate versioning system.

---

## 6. Part Inheritance

Parts can inherit from a base Part. This models families of components
that share everything except one or two attributes (typically value).

```
Base Part: "Generic 0402 1% resistor"
  Entity: resistor
  Package: 0402
  Manufacturer: (any)
  Tolerance: 1%
  Parametric table: resistor

Derived Part: "Generic 0402 1% 10kΩ resistor"
  Base: → base part UUID
  Value: "10kΩ"
  Parametric: { resistance: "10k" }
  (everything else inherited from base)

Derived Part: "Generic 0402 1% 4.7kΩ resistor"
  Base: → base part UUID
  Value: "4.7kΩ"
  Parametric: { resistance: "4.7k" }
```

Inheritance is single-level only (no chains). A derived part can
override any field. Non-overridden fields are read from the base.

---

## 7. Lifecycle and Supply Chain

Each Part tracks:

| Field | Values | Source |
|-------|--------|--------|
| lifecycle | active, nrnd, eol, obsolete, unknown | Manual or supply chain API |
| orderable_mpns | List of purchasable MPN strings | Manual or supply chain API |
| supply_chain | (future) pricing, stock, lead time | Supply chain API (M8) |

Lifecycle status affects:
- Pool search ranking (active parts ranked higher)
- BOM export warnings (EOL/obsolete parts flagged)
- AI recommendations ("this part is going EOL, consider alternatives")

---

## Milestone Position
- M0: Pool types implemented (Unit, Pin, Entity, Gate, Package, Pad, Part, Symbol)
- M0: SQLite index (create, insert, keyword search, parametric search)
- M0: Eagle .lbr → pool import
- M0: JSON serialization with round-trip tests
- M1: KiCad library → pool import
- M1: Design import populates pool from embedded libraries
- M2: Pool search via MCP and CLI
- M4: Manual pool creation (create-part wizard)
- M8: Supply chain integration
