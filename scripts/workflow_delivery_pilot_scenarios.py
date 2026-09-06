"""Explicit XTest scripts for the reviewed 1280x768, scale-one pilot.

Coordinates are taken from inspected native captures, not an alternate GUI model.
This names physical inputs only; observations must independently prove outcomes.
"""

VIEW, HELP, WINDOW, EDIT = 189, 595, 543, 148


def menu(x):
    return [["click", x, 16]]


def row_keys(index):
    return [["key", "Down"] for _ in range(index)]


def view_action(index):
    return menu(VIEW) + row_keys(index) + [["key", "Return"]]


def zoom():
    return [["move", 450, 400], ["wheel", 2]]


def capture(name):
    return [["capture", name]]


def missing(keyboard=True):
    steps = []
    for x, index, name in [(HELP, 0, "about"), (WINDOW, 0, "documents"), (VIEW, 5, "layers")]:
        steps += menu(x) + [["move", x + 30, 53 + 32 * index]] + capture(name + "-disabled")
        steps += [["click", x + 30, 53 + 32 * index]] + capture(name + "-pointer-refused")
        if keyboard:
            steps += menu(x) + row_keys(index) + capture(name + "-keyboard-focused")
            steps += [["key", "Return"]] + capture(name + "-keyboard-refused")
    return steps + view_action(21) + capture("refusal-history") + view_action(21)


def actions(scenario):
    if scenario == "PILOT-S03-pointer-regression":
        steps = zoom() + capture("board-zoomed") + [["key", "Tab"]] + menu(VIEW)
        steps += capture("schematic-fit-disabled") + [["click", 230, 53]]
        steps += capture("after-disabled-pointer-attempt")
        # A real canvas click must still acquire Board and select in one gesture.
        return steps + [["click", 369, 351]] + capture("canvas-focus-restored")
    if scenario == "PILOT-S01":
        # Existing square pad in the inspected native fixture; no --select seed.
        steps = [["click", 369, 351]] + capture("selected")
        steps += zoom() + capture("pointer-zoomed") + menu(VIEW)
        steps += [["click", 230, 53]] + capture("pointer-fitted")
        steps += zoom() + capture("shortcut-zoomed") + [["key", "f"]] + capture("shortcut-fitted")
        steps += zoom() + capture("keyboard-zoomed") + menu(VIEW)
        steps += [["key", "Down"], ["key", "Up"]] + capture("fit-keyboard-focused")
        return steps + [["key", "Return"]] + capture("keyboard-fitted")
    if scenario == "PILOT-S02":
        steps = missing()
        steps += view_action(21) + [["move", 350, 630], ["wheel", 3]]
        steps += capture("refusal-history-earlier") + view_action(21)
        # Existing F.Cu layer visibility swatch toggled twice, no missing handler.
        steps += [["click", 18, 531]] + capture("existing-layer-toggle-off")
        steps += [["click", 18, 531]] + capture("existing-layer-toggle-restored")
        steps += [["key", "Tab"]] + capture("existing-pane-next")
        return steps + [["key", "shift+Tab"]] + capture("existing-pane-restored")
    if scenario == "PILOT-S03":
        steps = view_action(17) + zoom() + menu(VIEW) + [["key", "Escape"], ["key", "f"]]
        steps += capture("escape-editor-fit") + [["key", "Tab"]] + menu(VIEW)
        steps += capture("schematic-fit-disabled") + [["key", "Return"], ["key", "Escape"]]
        steps += capture("schematic-refused") + [["key", "shift+Tab"]] + zoom()
        steps += [["key", "f"]] + capture("board-fit-restored") + missing(False)
        steps += view_action(15) + menu(VIEW) + capture("filled-schematic-disabled")
        steps += [["key", "Escape"]] + view_action(14) + menu(VIEW)
        return steps + capture("filled-board-enabled") + [["key", "Escape"]]
    if scenario == "PILOT-S04":
        steps = missing(False) + [["key", "Escape"]] + zoom() + [["key", "f"]]
        steps += capture("recovered-fit") + [["close"], ["reopen"]] + capture("reopened")
        return steps + [["key", "f"]] + capture("reopened-fit") + [["close"]]
    if scenario == "PILOT-S05":
        steps = menu(VIEW) + capture("view-live") + [["key", "Down"]] + capture("down-focus")
        steps += [["key", "Up"]] + capture("fit-focus") + row_keys(5) + capture("layers-focus")
        steps += [["key", "Return"]] + capture("layers-dismissed")
        for x, name in [(HELP, "about"), (WINDOW, "documents")]:
            steps += menu(x) + [["key", "Down"], ["key", "Up"]] + capture(name + "-focus")
            steps += [["key", "Return"]] + capture(name + "-dismissed")
        steps += menu(VIEW) + [["key", "Escape"]] + zoom() + [["key", "f"]]
        steps += capture("escape-fit") + view_action(15) + menu(VIEW) + capture("schematic-live")
        steps += [["key", "Escape"]] + view_action(14) + menu(VIEW) + capture("board-live")
        steps += [["key", "Escape"]] + capture("menus-absent-terminal-retained")
        steps += menu(EDIT) + row_keys(6) + [["key", "Right"], ["key", "Return"]]
        steps += capture("preferences-terminal-retained") + [["focus-window", "Global Preferences — Datum"],
                    ["key", "Escape"], ["focus-window", "Datum EDA"]]
        return steps + menu(VIEW) + capture("menu-after-preferences") + [["key", "Escape"]]
    raise ValueError("unreviewed scenario: " + scenario)
