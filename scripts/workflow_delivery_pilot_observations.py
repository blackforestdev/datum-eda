"""Evaluate recorded machine observations, not visual quality or pilot acceptance.

Screenshots still require producer inspection and independent replay; a machine
check pass cannot override the recorded Console readability defect.
"""

import json
import argparse
from pathlib import Path
import re

PREFIX = "DATUM_ACTION_EVIDENCE "


def read(path):
    return json.loads(path.read_text())


def state(root, capture):
    return [row["value"] for row in read(root / (capture + "-state.json"))
            if row["event"] == "state"][-1]


def records(root):
    result = []
    for log in sorted(root.glob("gui-*.log")):
        result.extend(json.loads(line.removeprefix(PREFIX))
                      for line in log.read_text().splitlines() if line.startswith(PREFIX))
    return result


def menu_node(root, capture, identity):
    path = "/org/a11y/atspi/accessible/menus/n" + identity.encode().hex()
    return next(node for node in read(root / (capture + "-atspi.json")) if node["path"] == path)


def flag(node, bit):
    words = re.findall(r"\d+", node["states"].replace("uint32", ""))
    return bool(int(words[0]) & (1 << bit))


def evaluate(root, receipts):
    sid = root.name
    findings = []

    def check(name, condition, observed):
        findings.append({"check": name, "pass": bool(condition), "observed": observed})

    identity = read(root / "run-identity.json")
    check("identified GUI and CLI", all(identity[k + "_sha256"] == receipts[k]["binary_sha256"]
          for k in ("gui", "cli")), {k: identity[k + "_sha256"] for k in ("gui", "cli")})
    diff = read(root / "source-diff.json")
    check("archived source and journal unchanged", not diff["changed_or_removed"], diff)
    permitted = (".datum/terminal-contexts/", ".datum/tool-sessions/")
    check("new paths are enumerated runtime sidecars only",
          all(p == ".datum/gui-terminal-context.json" or p.startswith(permitted)
              for p in diff["new_paths"]), sorted(diff["new_paths"]))
    before, after = read(root / "preferences-before.json"), read(root / "preferences-after.json")
    check("no preferences-file changes", before == after, {"before": before, "after": after})
    trace = records(root)
    states = [row["value"] for row in trace if row["event"] == "state"]
    dispatches = [row["value"] for row in trace if row["event"] == "dispatch"]
    check("actual scale and dimensions", bool(states) and all(s["scale"] == 1
          and s["window_size"] == [1280, 768] for s in states), "1280x768 / scale 1.0")
    check("no unavailable invocation", all(d["enabled"] or not d["invoked"] for d in dispatches), dispatches)
    check("normal close recorded", read(root / "inputs.json")[-1]["action"] == ["close"],
          [x for x in read(root / "inputs.json") if x["action"][0] in ("close", "reopen")])

    if sid == "PILOT-S01":
        selected = state(root, "selected")
        check("native pad selection", "pad:" in selected["selection"], selected["selection"])
        for method in ("pointer", "shortcut", "keyboard"):
            zoomed, fitted = state(root, method + "-zoomed"), state(root, method + "-fitted")
            check(method + " non-fitted prerequisite", zoomed["board_camera"]["zoom"] > 1, zoomed["board_camera"])
            check(method + " exact fitted camera", fitted["board_camera"] == selected["board_camera"], fitted["board_camera"])
            check(method + " preserved selection/layout/other camera",
                  all(fitted[k] == selected[k] for k in ("selection", "layout"))
                  and fitted["pane_cameras"][1:] == selected["pane_cameras"][1:], fitted["selection"])
        bounds = selected["scene_bounds"]
        center = selected["board_camera"]
        check("fit center agrees with actual scene bounds",
              center["center_x_nm"] == (bounds["min_x"] + bounds["max_x"]) / 2
              and center["center_y_nm"] == (bounds["min_y"] + bounds["max_y"]) / 2
              and center["zoom"] == 1, {"bounds": bounds, "camera": center})
        check("all three actual Fit entry surfaces", {d["entry_surface"] for d in dispatches} == {
            "view-fit-menu-pointer", "view-fit-menu-keyboard", "view-fit-shortcut"}
            and all(d["invoked"] for d in dispatches), dispatches)
    elif sid == "PILOT-S02":
        check("six pointer/keyboard refusals", len(dispatches) == 6
              and all(not d["enabled"] and not d["invoked"] for d in dispatches), dispatches)
        off, restored = state(root, "existing-layer-toggle-off"), state(root, "existing-layer-toggle-restored")
        check("existing layer toggle remains functional", off["layer_filters"] != restored["layer_filters"],
              {"off": off["layer_filters"], "restored": restored["layer_filters"]})
        next_pane, back = state(root, "existing-pane-next"), state(root, "existing-pane-restored")
        check("existing pane navigation remains functional", next_pane["focused_pane"] != back["focused_pane"],
              {"next": next_pane["focused_pane"], "restored": back["focused_pane"]})
    elif sid == "PILOT-S03":
        disabled = state(root, "schematic-fit-disabled")
        refused = state(root, "schematic-refused")
        check("unresolved Fit does not refit other pane", disabled["board_camera"] == refused["board_camera"]
              and any(not d["enabled"] and d["dispatch_key"] == "view.fit" for d in dispatches), refused)
        for name, enabled in [("schematic-fit-disabled", False), ("filled-schematic-disabled", False),
                              ("filled-board-enabled", True)]:
            node = menu_node(root, name, "menu:View/view.fit")
            check(name + " availability", flag(node, 8) == enabled and flag(node, 24) == enabled, node)
        for name in ("escape-editor-fit", "board-fit-restored"):
            value = state(root, name)
            check(name + " editor camera restored", value["focus"].startswith("Editor")
                  and value["board_camera"]["zoom"] == 1, value["board_camera"])
        regression = root.parent / "PILOT-S03-pointer-regression"
        if (regression / "after-disabled-pointer-attempt-state.json").exists():
            before = state(regression, "schematic-fit-disabled")
            after = state(regression, "after-disabled-pointer-attempt")
            attempts = [row["value"] for row in records(regression) if row["event"] == "dispatch"]
            check("disabled pointer Fit preserves context and does not invoke",
                  before["focused_pane"] == after["focused_pane"]
                  and before["board_camera"] == after["board_camera"]
                  and bool(attempts) and all(not a["invoked"] for a in attempts),
                  {"before_pane": before["focused_pane"], "after_pane": after["focused_pane"],
                   "before_camera": before["board_camera"], "after_camera": after["board_camera"],
                   "dispatches": attempts})
        else:
            check("disabled pointer Fit preserves context and does not invoke", False,
                  "native pointer regression capture is required")
    elif sid == "PILOT-S04":
        inputs = read(root / "inputs.json")
        check("two normal closes and one reopen", sum(x["action"] == ["close"] for x in inputs) == 2
              and sum(x["action"] == ["reopen"] for x in inputs) == 1, "two launches, two exit-zero closes")
        for name in ("recovered-fit", "reopened-fit"):
            value = state(root, name)
            check(name, value["board_camera"]["zoom"] == 1 and value["focus"].startswith("Editor"), value["board_camera"])
    elif sid == "PILOT-S05":
        for name, key, enabled in [("fit-focus", "View/view.fit", True),
                                   ("layers-focus", "View/view.layers", False),
                                   ("about-focus", "Help/help.about", False),
                                   ("documents-focus", "Window/window.documents", False)]:
            node = menu_node(root, name, "menu:" + key)
            check(name + " live focus/availability", flag(node, 12) and flag(node, 8) == enabled
                  and flag(node, 24) == enabled, node)
            if not enabled:
                check(name + " live reason", "unavailable in this build" in node["properties"], node["properties"])
        for name, enabled in [("schematic-live", False), ("board-live", True)]:
            node = menu_node(root, name, "menu:View/view.fit")
            check(name, flag(node, 8) == enabled and flag(node, 24) == enabled, node)
        for name in ("layers-dismissed", "about-dismissed", "documents-dismissed", "menus-absent-terminal-retained"):
            nodes = read(root / (name + "-atspi.json"))
            check(name, not any("/menus/" in n["path"] for n in nodes)
                  and any(n["path"].endswith("/terminal") for n in nodes), [n["path"] for n in nodes])
        preferences = read(root / "preferences-terminal-retained-atspi.json")
        check("Preferences and Terminal retained", any("Global Preferences" in n["properties"] for n in preferences)
              and any(n["path"].endswith("/terminal") for n in preferences), [n["path"] for n in preferences])
        restored = state(root, "menu-after-preferences")
        check("native Preferences dismissal returns menu operability", not restored["preferences_open"]
              and restored["menu"] == "View", {"preferences_open": restored["preferences_open"], "menu": restored["menu"]})
        value = state(root, "escape-fit")
        check("Escape restores editor F", value["board_camera"]["zoom"] == 1 and value["focus"].startswith("Editor"), value["focus"])
    else:
        raise ValueError("unreviewed scenario")
    return {"scenario_id": sid, "machine_checks_pass": all(f["pass"] for f in findings), "findings": findings}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--compact-state-snapshots", action="store_true",
                        help="drop redundant cumulative snapshot copies; retain full raw GUI logs")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1] / "docs/reviews/workflow-delivery-pilot"
    if args.compact_state_snapshots:
        for snapshot in sorted((root / "native").glob("PILOT-S*/*-state.json")):
            rows = read(snapshot)
            latest = [row for row in rows if row["event"] == "state"][-1:]
            if len(latest) != 1:
                raise ValueError("required snapshot state missing: " + str(snapshot))
            # The original observation is still present byte-for-byte as a JSON
            # value in the raw production log. Refuse an orphan projection.
            if latest[0] not in records(snapshot.parent):
                raise ValueError("snapshot not backed by raw production log")
            snapshot.write_text(json.dumps(latest, indent=2) + "\n")
        print("Compacted derived snapshots; full production logs retained.")
        raise SystemExit(0)
    receipts = {key: read(root / "native-build-verified" / (key + "-receipt.json")) for key in ("gui", "cli")}
    results = [evaluate(root / "native" / f"PILOT-S0{i}", receipts) for i in range(1, 6)]
    if args.output:
        with args.output.open("x") as stream:
            stream.write(json.dumps(results, indent=2) + "\n")
        for result in results:
            failed = [f["check"] for f in result["findings"] if not f["pass"]]
            print(result["scenario_id"], "machine checks:", "FAIL" if failed else "PASS", failed)
        print("Visual readability and pilot acceptance are NOT asserted by these machine checks.")
    else:
        print(json.dumps(results, indent=2))
    raise SystemExit(0 if all(r["machine_checks_pass"] for r in results) else 1)
