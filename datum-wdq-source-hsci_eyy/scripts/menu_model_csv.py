#!/usr/bin/env python3
"""Round-trip docs/gui/menu_model.json <-> docs/gui/menu_model.csv.

The CSV is the human authoring surface (a spreadsheet); the JSON is the built
artifact the GUI reads and scripts/check_menu_model.py gates. Workflow:

    edit docs/gui/menu_model.csv  ->  python3 scripts/menu_model_csv.py build
    ->  docs/gui/menu_model.json  ->  python3 scripts/check_menu_model.py

Columns (one row per menu leaf; nesting via `wheel` + `slot`):
  surface        menubar | marking
  context        menu name (File) for menubar, object type (pcb.component) for marking
  active_editor  menubar-only: schematic | pcb | ''  (gates the menu to a mode)
  wheel          '' for a top-level slot (menubar item, marking cardinal/secondary/overflow);
                 a submenu-wheel name (Align, Align Horizontal, Distribute) for nested items
  slot           menubar: order int; marking top-level: N/E/S/W/NE/SE/SW/NW/overflow;
                 submenu wheel: order int
  label, icon
  verb           datum.* id            (binding: verb)
  gui_local      view/shell action id  (binding: gui_local)
  not_built      reason string         (binding: not_built = worklist)
  submenu        target wheel name     (binding: submenu; may co-exist with not_built)
  destructive    y | ''
  shortcut       e.g. Ctrl+Z
"""
import csv, json, sys, pathlib
from collections import OrderedDict

ROOT = pathlib.Path(__file__).resolve().parents[1]
JSON = ROOT / "docs" / "gui" / "menu_model.json"
CSV = ROOT / "docs" / "gui" / "menu_model.csv"
COLS = ["surface", "context", "active_editor", "wheel", "slot", "label", "icon",
        "verb", "gui_local", "not_built", "submenu", "destructive", "shortcut"]
CARD = ("N", "E", "S", "W")
DIAG = ("NE", "SE", "SW", "NW")


def entry_to_row(e, **base):
    r = {c: "" for c in COLS}
    r.update(base)
    r["label"] = e.get("label", "")
    r["icon"] = e.get("icon", "")
    for k in ("verb", "gui_local", "not_built", "submenu", "shortcut"):
        if k in e:
            r[k] = e[k]
    if e.get("destructive"):
        r["destructive"] = "y"
    return r


def export():
    m = json.loads(JSON.read_text())
    rows = []
    for menu in m.get("menubar", []):
        for i, it in enumerate(menu.get("items", [])):
            rows.append(entry_to_row(it, surface="menubar", context=menu["menu"],
                                     active_editor=menu.get("active_editor", ""), wheel="", slot=str(i)))
    for name, mm in m.get("marking_menus", {}).items():
        for slot, e in (mm.get("cardinal") or {}).items():
            rows.append(entry_to_row(e, surface="marking", context=name, wheel="", slot=slot))
        for slot, e in (mm.get("secondary") or {}).items():
            rows.append(entry_to_row(e, surface="marking", context=name, wheel="", slot=slot))
        for e in (mm.get("overflow") or []):
            rows.append(entry_to_row(e, surface="marking", context=name, wheel="", slot="overflow"))
        for wheel, lst in (mm.get("submenus") or {}).items():
            for i, e in enumerate(lst):
                rows.append(entry_to_row(e, surface="marking", context=name, wheel=wheel, slot=str(i)))
    with CSV.open("w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=COLS)
        w.writeheader()
        w.writerows(rows)
    print(f"exported {len(rows)} rows -> {CSV.relative_to(ROOT)}")


def row_to_entry(r):
    e = OrderedDict()
    e["label"] = r["label"]
    if r["icon"]:
        e["icon"] = r["icon"]
    for k in ("verb", "gui_local", "not_built", "submenu"):
        if r.get(k):
            e[k] = r[k]
    if r.get("destructive", "").strip().lower() in ("y", "yes", "true", "1"):
        e["destructive"] = True
    if r.get("shortcut"):
        e["shortcut"] = r["shortcut"]
    return e


def build_obj():
    rows = list(csv.DictReader(CSV.open()))
    base = json.loads(JSON.read_text())  # preserve schema + notes
    menubar = OrderedDict()
    marking = OrderedDict()
    for r in rows:
        if r["surface"] == "menubar":
            menu = menubar.setdefault(r["context"], {"active_editor": r.get("active_editor", ""), "items": []})
            if r.get("active_editor"):
                menu["active_editor"] = r["active_editor"]
            menu["items"].append((int(r["slot"]), row_to_entry(r)))
        elif r["surface"] == "marking":
            mm = marking.setdefault(r["context"], {"cardinal": OrderedDict(), "secondary": OrderedDict(),
                                                   "overflow": [], "submenus": OrderedDict()})
            if not r["wheel"]:
                if r["slot"] in CARD:
                    mm["cardinal"][r["slot"]] = row_to_entry(r)
                elif r["slot"] in DIAG:
                    mm["secondary"][r["slot"]] = row_to_entry(r)
                elif r["slot"] == "overflow":
                    mm["overflow"].append(row_to_entry(r))
            else:
                mm["submenus"].setdefault(r["wheel"], []).append((int(r["slot"]), row_to_entry(r)))
    out_menubar = []
    for name, menu in menubar.items():
        d = {"menu": name}
        if menu["active_editor"]:
            d["active_editor"] = menu["active_editor"]
        d["items"] = [e for _, e in sorted(menu["items"], key=lambda x: x[0])]
        out_menubar.append(d)
    out_marking = OrderedDict()
    for name, mm in marking.items():
        out_marking[name] = {
            "cardinal": mm["cardinal"],
            "secondary": mm["secondary"],
            "overflow": mm["overflow"],
            "submenus": OrderedDict((w, [e for _, e in sorted(lst, key=lambda x: x[0])])
                                    for w, lst in mm["submenus"].items()),
        }
    base["menubar"] = out_menubar
    base["marking_menus"] = out_marking
    return base


def build(target=JSON):
    pathlib.Path(target).write_text(json.dumps(build_obj(), indent=2) + "\n")
    print(f"built {target} from {CSV.relative_to(ROOT)}")


def check():
    """Fail if menu_model.json is out of sync with menu_model.csv (drift gate)."""
    want = json.dumps(build_obj(), indent=2) + "\n"
    if JSON.read_text() != want:
        print("OUT OF SYNC: docs/gui/menu_model.json != build(docs/gui/menu_model.csv). "
              "Run: python3 scripts/menu_model_csv.py build")
        return 1
    print("in sync: menu_model.json == build(menu_model.csv)")
    return 0


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else "export"
    if mode == "export":
        export()
    elif mode == "build":
        build()
    elif mode == "check":
        sys.exit(check())
    else:
        print("usage: menu_model_csv.py [export|build|check]")
        sys.exit(2)
