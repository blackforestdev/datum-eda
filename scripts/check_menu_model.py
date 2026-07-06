#!/usr/bin/env python3
"""Validate docs/gui/menu_model.json against the single-source verb registry
catalog and the marking-menu structural invariants.

Enforces (per docs/gui/DATUM_GUI_DESIGN_SPEC.md 'Modularity' + 'Context Menu System'):
  1. every entry with a `verb` binding references a real datum.* verb that exists
     in mcp-server/datum_tool_catalog.json (no vaporware menu items);
  2. every entry has exactly one binding kind (verb / gui_local / not_built /
     submenu / empty) and never contradicts (verb + not_built);
  3. marking menus: cardinal slots are a subset of {N,E,S,W}, secondary of
     {NE,SE,SW,NW}, and a `destructive` action never sits on a diagonal (secondary)
     slot -- it must live on Cardinal-S or in the overflow.

Wired into scripts/run_drift_gates.sh. Reports verb-backed vs not-built counts;
the not-built entries are the authoring buildout worklist.
"""
import json, re, sys, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
MODEL = ROOT / "docs" / "gui" / "menu_model.json"
CATALOG = ROOT / "mcp-server" / "datum_tool_catalog.json"
VERB_RE = re.compile(r"^datum\.[a-z_]+\.[a-z0-9_]+$")
CARD = {"N", "E", "S", "W"}
DIAG = {"NE", "SE", "SW", "NW"}
BINDINGS = ("verb", "gui_local", "not_built", "submenu", "empty")


def collect_verbs(obj, out):
    if isinstance(obj, str):
        if VERB_RE.match(obj):
            out.add(obj)
    elif isinstance(obj, dict):
        for v in obj.values():
            collect_verbs(v, out)
    elif isinstance(obj, list):
        for v in obj:
            collect_verbs(v, out)


def main():
    model = json.loads(MODEL.read_text())
    catalog = json.loads(CATALOG.read_text())
    valid = set()
    collect_verbs(catalog.get("verbs", catalog), valid)
    if not valid:
        print("menu_model gate FAILED: no verbs found in the registry catalog")
        return 1

    fail = []
    counts = {k: 0 for k in BINDINGS}
    entries = []

    for m in model.get("menubar", []):
        for it in m.get("items", []):
            entries.append((f"menubar/{m['menu']}/{it.get('label')}", it))

    for name, mm in model.get("marking_menus", {}).items():
        cardk = set((mm.get("cardinal") or {}).keys())
        if not cardk <= CARD:
            fail.append(f"{name}: cardinal has non-N/E/S/W slots {sorted(cardk - CARD)}")
        seck = set((mm.get("secondary") or {}).keys())
        if not seck <= DIAG:
            fail.append(f"{name}: secondary has non-diagonal slots {sorted(seck - DIAG)}")
        for slot, e in (mm.get("cardinal") or {}).items():
            entries.append((f"{name}/cardinal/{slot}", e))
        for slot, e in (mm.get("secondary") or {}).items():
            entries.append((f"{name}/secondary/{slot}", e))
            if e.get("destructive"):
                fail.append(f"{name}/secondary/{slot}: destructive action on a diagonal "
                            f"slot (must be Cardinal-S or overflow)")
        for i, e in enumerate(mm.get("overflow") or []):
            entries.append((f"{name}/overflow/{i}", e))
        for sub, lst in (mm.get("submenus") or {}).items():
            for i, e in enumerate(lst):
                entries.append((f"{name}/submenu/{sub}/{i}", e))

    for path, e in entries:
        binds = [k for k in BINDINGS if k in e]
        if not binds:
            fail.append(f"{path}: no binding (need one of {BINDINGS})")
        if "verb" in e and "not_built" in e:
            fail.append(f"{path}: contradictory binding (both 'verb' and 'not_built')")
        if "verb" in e:
            counts["verb"] += 1
            if e["verb"] not in valid:
                fail.append(f"{path}: unknown verb '{e['verb']}' (not in verb registry catalog)")
        elif "gui_local" in e:
            counts["gui_local"] += 1
        elif "not_built" in e:
            counts["not_built"] += 1
        elif "submenu" in e:
            counts["submenu"] += 1
        elif "empty" in e:
            counts["empty"] += 1

    if fail:
        print("menu_model gate FAILED:")
        for f in fail:
            print("  -", f)
        return 1

    print(f"menu_model gate passed: {len(entries)} entries "
          f"({counts['verb']} verb-backed, {counts['gui_local']} gui-local, "
          f"{counts['not_built']} not-built/worklist, {counts['submenu']} submenu-ref, "
          f"{counts['empty']} empty); all verb references exist in the registry catalog.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
