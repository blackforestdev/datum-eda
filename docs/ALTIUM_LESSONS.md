# Altium Designer — Lessons for Horizon-AI

## Purpose
Altium Designer is the industry benchmark for professional PCB design UX.
This document identifies specific Altium design decisions that should inform
the Horizon-AI fork, mapped against Horizon's current capabilities. This is
not about cloning Altium — it's about understanding WHY professionals choose
it and bringing those principles to an AI-native, Linux-native tool.

---

## Priority 1: Immediate Impact / Moderate Effort

### 1.1 Properties Panel (Altium's Killer UX Feature)

**What Altium does**: Context-sensitive, always-visible panel that dynamically
adapts to the selected object(s). Select a resistor — see value, footprint,
net connections. Select a track — see width, net, layer, length. Select 50
resistors — edit their shared properties in one operation. No dialogs.

**Why it matters**: The dialog-per-object workflow (select → open dialog → edit
→ close → repeat) is the single biggest time sink in KiCad and older tools.
Altium's properties panel eliminates this entirely. Every property change is
one click, not four.

**Horizon's current state**: Has a `PropertyPanels` system (`src/property_panels/`)
with `PropertyPanel` and `PropertyEditor` classes. It exists but is basic —
a GTK expander-based panel for property editing. It does NOT support:
- Multi-object editing (select many, edit shared properties)
- Dynamic context switching (different panel for different object types)
- Inline editing of all properties without dialogs

**What to build**:
- Extend PropertyPanels to handle multi-select with shared-property detection
- Add type-specific panel configurations for every object type
- Make it the PRIMARY editing interface, not a supplement to dialogs
- AI integration: properties panel state becomes context for AI queries
  ("what's selected?", "what are its properties?")

### 1.2 Keyboard-Centric Routing Workflow

**What Altium does**: During active routing, single-key presses adjust
parameters without breaking the routing operation:
- `Shift+R`: Cycle conflict resolution (walkaround → push → hug → ignore)
- `Shift+Space`: Cycle corner style (45° → any angle → arc)
- `3`: Cycle track width (min → preferred → max from rule)
- `4`: Cycle via size
- `Shift+G`: Toggle net length gauge
- `Shift+F`: Follow mode (route along contours)
- `Ctrl+W`: Toggle clearance boundary display
- `B`/`Shift+B`: Adjust bus spacing (multi-route)

**Why it matters**: Every time a designer opens a dialog during routing,
they lose flow state. Altium's approach keeps hands on the keyboard and
eyes on the board. This is the same philosophy Eagle had with its command
line — minimize interruption.

**Horizon's current state**: The PNS router supports walkaround, shove, and
basic corner modes. But parameter changes during routing require menu
interaction. No width cycling, no via cycling, no inline length display.

**What to build**:
- In-route keyboard shortcuts for width/via/mode cycling
- HUD overlay showing current routing parameters (width, clearance, length)
- Net length gauge (real-time, visual, during routing)
- Clearance boundary visualization toggle
- AI integration: "route this net at 100Ω impedance" translates to
  correct width selection per layer from rule system

### 1.3 Route State Visualization

**What Altium does**: Three visual states for track segments during routing:
- **Hatched**: Under router control, tentative
- **Solid**: Soft-committed, can be undone
- **Hollow**: Look-ahead preview, won't place on click

**Why it matters**: The designer always knows what is committed vs. tentative.
This visual language prevents the "did that actually place?" confusion common
in other tools.

**Horizon's current state**: The PNS router renders the trace being routed
but has no visual distinction between committed, tentative, and preview
segments.

**What to build**:
- Shader-based rendering of three route states (Horizon's OpenGL canvas
  already has custom shaders — extend for hatching/hollow)
- Apply to diff pair routing and length tuning as well

---

## Priority 2: Significant Impact / Significant Effort

### 2.1 Rule Query Language

**What Altium does**: Rules are scoped with an expression language:
```
InNet('CLK') And OnLayer('Top Layer')
InNetClass('HighSpeed') And Not InComponent('U1')
HasFootprint('QFP-*')
```
Operators: And, Or, Not, brackets. Functions: InNet, InNetClass, OnLayer,
InComponent, HasFootprint, InRoom, IsVia, IsPad, etc.

Priority resolution: rules are ordered, first matching scope wins. This
allows general rules with specific overrides.

**Why it matters**: KiCad's net class system is flat — you can set rules per
net class, but you can't express "clearance between high-speed nets and
anything on the bottom layer" without creating artificial net classes.
Altium's query language makes the rule system composable and expressive.

Horizon's current RuleMatch is similarly limited:
```cpp
enum class Mode { ALL, NET, NETS, NET_CLASS, NET_NAME_REGEX, NET_CLASS_REGEX };
```
Six fixed scoping modes. No expression composition. No layer-based scoping.
No component-based scoping.

**What to build**:
- Expression parser for rule scoping (simple recursive descent)
- Functions: InNet, InNetClass, OnLayer, InComponent, HasPackage, IsVia,
  IsPad, IsSMD, InArea/InRoom
- Priority ordering with first-match-wins semantics
- Query builder UI (Altium's is a dialog — ours should be a command)
- AI integration: natural language rule definition
  ("keep USB signals 10 mil from anything on the bottom layer")

### 2.2 ECO System with Per-Change Granularity

**What Altium does**: When syncing schematic↔PCB, a comparator generates an
Engineering Change Order listing every individual difference:
- Added component R15
- Removed net VCC_OLD
- Changed footprint on U3 from QFP-48 to QFP-64
- Renamed net DATA_IN to SPI_MOSI

The designer reviews the list and checks/unchecks individual changes.
Changes can flow in either direction (schematic→PCB or PCB→schematic).

**Why it matters**: KiCad's "Update PCB from Schematic" is bulk — you can't
selectively accept changes. This is dangerous on complex boards where a
schematic change might break carefully-done layout work. Altium's ECO gives
the designer full control over what gets applied.

**Horizon's current state**: Has backannotation support (ZMQ `backannotate`
op) and netlist reload. But no granular ECO comparator with per-change
accept/reject.

**What to build**:
- Diff engine between block (schematic netlist) and board state
- ECO dialog showing individual changes with accept/reject checkboxes
- Bidirectional sync (forward and backward annotation)
- AI integration: "what changed since last sync?" returns structured diff
  that AI can reason about

### 2.3 Impedance-Aware Layer Stack Manager

**What Altium does**: Dedicated stack manager with:
- Material properties database (Dk, Df, copper weight per layer)
- Integrated impedance calculator (Simbeor SFS field solver)
- Automatic trace width computation per layer for target impedance
- Supports microstrip, stripline, coplanar structures
- Back-drill configuration for via stub removal

**Why it matters**: Controlled impedance is non-negotiable for any design
above 50 MHz. Currently this requires external tools (Saturn PCB Toolkit,
Si9000, online calculators). Integrating the impedance solver into the
stackup editor means trace widths update automatically when the stackup
changes.

**Horizon's current state**: Has a stackup editor (`EDIT_STACKUP` tool) with
layer definitions, but no material properties database, no impedance
calculator, no automatic width computation.

**What to build**:
- Material properties per dielectric layer (Dk, Df, thickness)
- Copper properties per layer (thickness, roughness)
- 2D field solver for impedance computation (open-source options:
  openEMS, atlc2, or custom MoM solver)
- Impedance profiles linked to routing width rules
- AI integration: "configure this for 100Ω differential on layers 1/2 with
  FR4" sets up the entire stackup + rules automatically

### 2.4 Supply Chain Integration

**What Altium does**: ActiveBOM provides real-time pricing, stock levels,
lifecycle status, and supplier availability for every component during design.
Not a post-processing step — designers see "this part is going EOL" or
"this part has 6-week lead time" while they're still designing.

**Why it matters**: Component availability has been the #1 supply chain
problem since 2020. Discovering a part is unavailable after layout is complete
means redesign. Early visibility prevents this.

**Horizon's current state**: Has DigiKey stock info integration
(`stock_info_provider_digikey`) with a cache database. Basic but functional.
Also has orderable MPNs in the part model.

**What to build**:
- Extend stock provider to multiple sources (Mouser, LCSC, Farnell)
- Lifecycle status indicator per part (active/NRND/EOL/obsolete)
- BOM-level supply chain summary (total cost, worst-case lead time,
  single-source risk)
- AI integration: "are there any supply chain risks in this design?"
  returns structured analysis

### 2.5 Schematic Hierarchy and Annotation UX

**What Altium does**: Hierarchical sheets, ports, buses, net labels, and
annotation all participate in a coherent editing workflow. Designers can
rename ports, update sheet symbols, re-annotate, and review ECO changes
without losing confidence in object identity.

**Why it matters**: Professional schematic work is not just placing symbols
and wires. The hard parts are reusable hierarchy, stable multi-sheet intent,
annotation discipline, and cross-probing changes into PCB without accidental
breakage.

**What to build**:
- A hierarchy panel showing sheet definitions, instances, and port mappings
- Deterministic annotation controls with scope preview before commit
- Properties-panel editing for labels, ports, buses, and fields
- Explicit visualization of hidden power behavior and promoted power objects
- AI integration: "show me why this port does not connect through hierarchy"
  uses schematic connectivity diagnostics, not guesswork

### 2.6 Schematic Bus and Harness Workflow

**What Altium does**: Bus constructs are editable, inspectable, and visibly
bound to scalar members. Bus entries and labels are not treated as loose
graphics; they are first-class connectivity-shaping objects.

**Why it matters**: High-pin-count digital designs become unreadable if the
editor treats buses as afterthoughts. A professional schematic editor must
make bus membership, expansion, and hierarchy behavior obvious.

**What to build**:
- Bus member inspection and edit tools
- Bus-entry-aware snapping and validation
- Connectivity diagnostics for ambiguous expansion and broken hierarchy links
- AI integration: "which scalar nets are carried by this bus across sheets?"

### 2.7 Cross-Probe and Selection Parity

**What Altium does**: Cross-probing between schematic and PCB is immediate and
predictable. Selecting a component, net, or violation in one domain reveals
the corresponding objects in the other.

**Why it matters**: If board workflows are richer than schematic workflows,
the schematic becomes a source artifact instead of a live engineering surface.
That is exactly the parity failure this project must avoid.

**What to build**:
- Selection and properties parity for schematic objects
- Cross-probe hooks for symbols, ports, nets, and ERC findings
- ECO review that presents schematic-side and board-side context together
- AI integration: "show me every schematic object involved in this board net"

---

## Priority 3: High Impact / High Effort (Long-term)

### 3.1 Glossy Routing / Route Optimization

**What Altium does**: Three glossing modes (off/weak/strong) that continuously
optimize the route path during placement. Separate "neighbor glossing"
smooths adjacent routes displaced by push-and-shove. Three hugging styles
(45°, mixed, rounded) control how routes conform to obstacles.

**Why it matters**: Glossing produces cleaner routes with fewer corners and
shorter paths, improving signal integrity and reducing manufacturing risk.
Without it, routes follow the exact cursor path, requiring manual cleanup.

**Horizon's current state**: The bundled PNS has a basic `pns_optimizer.cpp`
that does line simplification and cost-based path optimization. But it's
KiCad 6.0.4-era code — KiCad 9's optimizer is significantly improved.

**What to build**:
- Update PNS router to KiCad 9 version (major undertaking)
- OR: implement independent optimization passes (merge collinear segments,
  remove unnecessary corners, minimize total length)
- Add glossing modes accessible during interactive routing
- AI integration: "clean up routing on this net" runs optimization passes

### 3.2 Multi-Track / Bus Routing

**What Altium does**: Select multiple source pads, route all connections
simultaneously with adjustable bus spacing. Respects all design rules
across all parallel tracks.

**What to build**:
- Multi-net interactive routing mode
- Dynamic spacing adjustment during routing
- Respects per-net width and clearance rules
- AI integration: "route this 8-bit data bus from U1 to U2"

### 3.3 Design Reuse System

**What Altium does**: Three tiers — snippets (simple copy-paste), device
sheets (reusable circuit blocks), managed sheets (revision-controlled).
Reuse blocks contain both schematic AND PCB representations.

**Why it matters**: Professional designs reuse the same power supply, USB
interface, or debug connector across projects. A proper reuse system
prevents re-designing proven circuits.

**What to build**:
- Circuit snippet save/load for both schematic and board
- Block-level reuse with both domains (schematic + routed PCB)
- Pool integration (reuse blocks stored in pool alongside parts)
- AI integration: "insert a 3.3V LDO circuit using the MCP1700"
  retrieves a proven reuse block and places it

### 3.4 Output Job System

**What Altium does**: Pre-configured output sets (OutJob files) that define
all manufacturing deliverables — Gerber settings, drill files, BOM format,
assembly drawings, PDFs. Run once to generate everything.

**Horizon's current state**: Has Gerber/ODB++/PDF/PnP/STEP export settings
stored per-board. No unified output job concept.

**What to build**:
- Output job definitions (JSON-based, part of project)
- One-click manufacturing package generation
- AI integration: "generate manufacturing files for JLCPCB" selects
  correct Gerber layers, drill format, BOM template

---

## Priority 4: Philosophical Differences Worth Studying

### 4.1 Unified Data Model

**Altium's approach**: A component is ONE entity containing symbol, footprint,
3D model, simulation model, supply chain data. All representations travel
together.

**Horizon's approach**: The pool system separates Unit (electrical) → Entity
(multi-gate) → Part (purchasable, links entity + package + pad map). This
is architecturally cleaner than Altium's monolithic approach, but the
indirection has a cost — creating a new part requires touching 3-4 separate
pool items.

**Assessment**: Horizon's pool model is actually BETTER than Altium's for a
large shared library. The separation allows one package to serve many parts
and one entity to have multiple packages. But the creation workflow needs
to be streamlined — a wizard that creates unit + entity + part + package
in one flow, not four separate editors.

### 4.2 Variant Management

**Altium's approach**: Fitted/not-fitted/alternate per component per variant.
Assembly variants (same board, different stuffing) and fabrication variants
(different board builds).

**Horizon's current state**: Basic variant support exists in the block model
but is not as developed as Altium's.

### 4.3 Real-time 3D

**Altium's approach**: Press `3` to toggle 2D/3D view. Real-time rendering
in the same editor window. 3D clearance checking.

**Horizon's current state**: Separate 3D viewer window (canvas3d). Not
integrated into the main editor canvas. No 3D DRC.

---

## Summary: What to Build, In Order

### Immediate (M1-M2 timeframe)
1. **Keyboard-centric routing** — single-key width/via/mode cycling during route
2. **Net length gauge** — real-time HUD during routing
3. **Route state visualization** — hatched/solid/hollow segment rendering
4. **Multi-object property editing** — select many, edit shared properties

### Near-term (M3-M4 timeframe)
5. **Rule query language** — expression-based rule scoping
6. **ECO system** — per-change accept/reject schematic↔PCB sync
7. **Clearance boundary visualization** — toggle during routing
8. **Part creation wizard** — streamlined unit→entity→part→package flow
9. **Schematic hierarchy UX** — sheet definitions, instances, ports, annotation
10. **Bus-aware schematic editing** — member inspection and diagnostics

### Medium-term (M5+ timeframe)
11. **Impedance-aware layer stack** — material properties + field solver
12. **Supply chain dashboard** — multi-source pricing, lifecycle, lead time
13. **Output job system** — one-click manufacturing package
14. **Glossy routing** — continuous route optimization

### Long-term
15. **Multi-track bus routing**
16. **Design reuse blocks** (schematic + PCB paired)
17. **Integrated 2D/3D view** with 3D DRC
18. **Follow mode** routing along contours

---

## What NOT to Copy from Altium

1. **DelphiScript** — Altium's scripting is its weakest point. Use Python.
2. **Cloud lock-in** — No mandatory cloud. Self-hosted, git-based.
3. **Subscription pressure** — Open source, GPL v3. No artificial restrictions.
4. **Windows-only** — Linux-native is the entire point.
5. **Monolithic UI** — Altium's settings are labyrinthine. Keep it discoverable.
6. **Situs autorouter** — Not competitive. Focus on AI-assisted interactive routing.

---

## The AI Angle on Every Feature

Every Altium feature listed above has a natural AI extension:

| Altium Feature | AI Evolution |
|---------------|-------------|
| Properties panel | "What's wrong with this component?" contextual AI |
| Width cycling `3` | "Route at 50Ω impedance" — AI selects correct width |
| Rule queries | "Keep USB 10 mil from power" — natural language rules |
| ECO system | "What changed and should I accept it?" — AI-assisted review |
| Impedance stack | "Set up for DDR4 routing" — AI configures entire stackup |
| Supply chain | "Any risks?" — AI analyzes BOM availability |
| Glossing | "Clean up this routing" — AI runs optimization |
| Multi-track | "Route this bus" — AI handles parallel routing |
| Design reuse | "Insert a USB-C connector circuit" — AI retrieves block |
| Output jobs | "Generate for JLCPCB" — AI configures all exports |

This is the fundamental insight: **Altium's keyboard shortcuts are a human
optimization for parameter selection. AI eliminates parameter selection
entirely by understanding intent.** "Route this at 100Ω" is strictly better
than pressing `3` three times to cycle to the right width.

The professional features Altium provides are the DOMAIN KNOWLEDGE that an
AI integration layer needs. You can't build "route at 100Ω" without having
an impedance-aware layer stack. You can't build "keep USB away from power"
without a rule query language. Altium's features are the vocabulary that
makes AI-driven design possible.
