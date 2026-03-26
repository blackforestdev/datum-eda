# Competitive Analysis

> **Status**: Non-normative research and positioning analysis.
> This document does not define any contracts or API surfaces.

## Purpose
Maps the existing Linux EDA landscape, identifies what's broken or
missing, and articulates where this project fits. This is not a feature
comparison chart — it's an analysis of why a new tool is justified.

---

## 1. Existing Linux EDA Tools

### KiCad (v9, 2025)
**Status**: The dominant open-source EDA tool. Active development, large community.

**Strengths**:
- Full schematic + PCB workflow
- Interactive push-and-shove router (PNS)
- 3D viewer (OpenCASCADE)
- Large symbol/footprint library (community-maintained)
- KiCad 8+ added custom DRC constraint language
- Free, open-source (GPL v3)

**Weaknesses**:
- No real CLI — `kicad-cli` exists but is limited to export operations
  (plot, export-bom). Cannot import, query, run DRC, or modify designs
  from the command line.
- No scriptable design modification (no Python API for editing designs)
- Python scripting (pcbnew module) is poorly documented, unstable across
  versions, and cannot be used headless reliably
- No MCP or structured AI integration
- File format changes between versions, breaking parsers
- DRC constraint language is powerful but has no query API — you can
  write rules but cannot ask "what rules apply to this net?"
- Net class system has improved but remains less expressive than Altium's
  query language
- No impedance-aware stackup manager (impedance must be computed externally)
- Library management is a known pain point (symbol/footprint binding)
- GUI-first architecture — headless operation is an afterthought

**The gap**: KiCad has no answer for "AI agent analyzes my design" or
"CI pipeline runs DRC and fails the build with structured output" or
"script moves components and saves back." The CLI and Python story is
weak enough that professional automation workflows are effectively
impossible without fragile hacks.

### Horizon EDA (v2.7, 2026)
**Status**: Excellent architecture, limited adoption. Single primary developer.

**Strengths**:
- Best-in-class pool/library system (the model this project studies)
- Clean C++ codebase (~169K LOC)
- JSON file format (diffable, version-control friendly)
- SQLite pool index with FTS search
- Bundled KiCad PNS router
- ODB++ export (professional manufacturing format)

**Weaknesses**:
- No CLI at all — GUI-only application
- No scripting API for design modification (Python module is export-only)
- No AI integration path
- Router frozen at KiCad 6.0.4 (4 years behind)
- GTK3 (maintenance mode — should be GTK4 or other)
- No command line input (unlike Eagle)
- Bus factor = 1 (single primary developer)
- No impedance calculator
- No autorouter (only interactive PNS)
- Small user community — limited testing on diverse designs

**The gap**: Horizon has the best data model architecture of any open-source
EDA tool but is trapped in a GUI-only, single-developer situation with no
path to programmatic access.

### gEDA / PCB (legacy)
**Status**: Effectively unmaintained. Last significant development activity
years ago.

**Strengths**:
- Scriptable (Scheme-based scripting in PCB)
- Unix philosophy (separate tools: gschem, PCB, gnetlist)

**Weaknesses**:
- No active development
- No modern routing
- No modern DRC
- Antiquated UI
- File formats are legacy

**The gap**: gEDA proved that Unix-philosophy EDA (separate tools, pipes,
scripting) had value, but the project died from lack of maintenance and
modern capabilities. The philosophy was right; the execution didn't survive.

### LibrePCB (v1.x, 2025)
**Status**: Modern, clean, but early-stage for professional use.

**Strengths**:
- Excellent library management (similar philosophy to Horizon's pool)
- Clean codebase (C++/Qt)
- Cross-platform
- Version-controlled libraries

**Weaknesses**:
- No routing engine (must export to other tools for routing)
- Limited DRC
- Small feature set compared to KiCad
- No CLI, no scripting, no AI integration
- Small community

**The gap**: LibrePCB focused on library management but doesn't have
enough of the workflow to be a standalone tool for professional use.

### Fritzing
**Status**: Hobbyist tool. Not relevant for professional use.

### Commercial Tools on Linux
- **Altium**: No Linux version. Wine compatibility is poor.
- **OrCAD/Allegro**: No Linux version.
- **PADS**: No Linux version.
- **Pulsonix**: No Linux version.
- **Zuken CR-8000**: Linux version exists but extremely expensive and
  enterprise-only.

The professional EDA market has effectively zero Linux-native options
beyond KiCad.

---

## 2. The Gap This Project Fills

### What doesn't exist on Linux today

1. **No EDA tool with a real CLI for design operations**
   - KiCad's CLI is export-only
   - Horizon has no CLI
   - No tool supports `import → query → modify → DRC → save` from the command line

2. **No EDA tool with structured AI integration**
   - No MCP server for any EDA tool
   - No tool exposes design data in a format an AI agent can consume and act on
   - The closest is KiCad's Python module, which is fragile and GUI-dependent

3. **No EDA tool designed for CI/CD**
   - No tool can run DRC headlessly with structured JSON output and meaningful exit codes
   - Hardware teams cannot integrate design verification into their CI pipeline
   - Manual design review is the only option

4. **No EDA tool with an import-first, analyze-first philosophy**
   - Every tool assumes you create designs in it
   - No tool is designed to open someone else's KiCad/Eagle project and
     provide structured analysis without first converting to its native format

5. **No headless-first EDA engine**
   - Every existing tool is GUI-first with CLI bolted on (or absent)
   - The engine/GUI separation doesn't exist — you can't use the design
     engine without the GUI

### What this project provides that nothing else does

| Capability | This project | Nearest competitor |
|-----------|-------------|-------------------|
| Headless engine (no GUI dependency) | Core architecture | None |
| CLI: import + query + ERC/DRC + modify + export | M2-M4 | KiCad CLI (export only) |
| MCP server for AI agents | M2 | None |
| Structured JSON DRC output | M2 | None |
| CI/CD integration (exit codes, JSON reports) | M2 | None |
| Import KiCad/Eagle and query programmatically | M1-M2 | None (without GUI) |
| Python scripting with full engine access | M4 (PyO3) | KiCad pcbnew (unstable, GUI-dependent) |
| ERC with explanation and waiver model | M2 | KiCad ERC (GUI only, no explanation API) |
| AI-assisted layout with proposal/review | M5-M6 | None |

---

## 3. Why Not Just Improve KiCad?

This is the obvious question. KiCad is mature, has a large community,
and is actively developed. Why build something new?

**Architectural reasons**:
- KiCad's architecture is GUI-first. The design engine (`pcbnew`,
  `eeschema`) are GUI applications, not libraries. There is no clean
  engine/GUI separation. Making KiCad headless-first would require
  rewriting the core architecture.
- KiCad's Python API is a thin wrapper over the GUI application's
  internals. It is not designed for headless use, breaks across versions,
  and cannot be used in a serverless/CI context reliably.
- KiCad's file format is an S-expression dialect that changes between
  versions. Building a stable parser that works across KiCad 7/8/9
  is feasible but maintaining it against future changes is ongoing work.

**Philosophical reasons**:
- KiCad optimizes for the GUI user. This project optimizes for the
  AI agent, the CLI user, and the CI pipeline.
- KiCad's development is community-driven with many contributors.
  This project makes opinionated architectural decisions that would
  not survive KiCad's consensus process.
- KiCad is a complete tool. This project starts as a complement
  (analyze KiCad designs better than KiCad can) and grows into an
  alternative over time.

**Practical reasons**:
- This project can import KiCad designs. It doesn't replace KiCad — it
  extends what you can do with KiCad designs.
- v1 (M2) is useful alongside KiCad, not instead of it. Design in KiCad,
  analyze/automate/CI with this tool.
- The trajectory toward full CAD (M4+) is a long-term ambition, not a
  launch promise.

---

## 4. Positioning Strategy

### Phase 1 (M0-M2): Complement
"The best way to analyze, query, and check your KiCad/Eagle designs
from the command line or via AI."

- Not a replacement for anything
- Works with existing designs
- Adds capabilities no existing tool has
- Zero cost to try (import your existing project)

### Phase 2 (M3-M4): Alternative for automation
"Create and modify PCB designs without a GUI. AI-native design
automation for Linux."

- Can create designs from scratch
- Full schematic + board workflow via CLI/MCP
- Manufacturing output (Gerber, BOM)
- Still imports KiCad/Eagle designs

### Phase 3 (M5-M7): Contender
"A complete PCB design environment with AI-assisted layout, CLI-first
workflow, and visual editing."

- Layout engine with AI strategy
- GUI for visual editing and review
- Competitive with KiCad on features
- Differentiated by AI integration and CLI workflow

---

## 5. Risks of This Positioning

| Risk | Mitigation |
|------|-----------|
| KiCad adds a real CLI/MCP | Our engine architecture is still differentiated. KiCad adding CLI validates the need. |
| Nobody uses CLI for EDA | CI/CD use case doesn't require individual CLI users. AI agent use case is CLI-native. |
| Import fidelity isn't good enough | Golden test corpus, fidelity tracking, accept partial import with warnings |
| Takes too long to reach useful (M2) | M2 scope is deliberately narrow. Analysis-only is useful without authoring. |
| Horizon EDA adds Python API | Horizon's architecture could support it, but single-developer constraint makes it unlikely soon |
| Commercial tool adds Linux + AI | Validates the market. Our open-source + CLI-first model serves a different user. |
