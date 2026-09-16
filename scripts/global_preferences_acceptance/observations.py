"""Validate case/native observations against frozen fixture expectations and raw records."""

from .identity import command_line
from .inputs import array, canonical, closed, integer, require, sha, text, unique
from .inventory import GATES, NATIVE_ASSERTIONS
from .matrix import case_inventory, gui_coordinates


def identity(row, report, environments, binaries):
    require(row["candidate_revision"] == report["candidate"]["revision"], "observation candidate differs")
    require(row["input_manifest_sha256"] == report["input_manifest"]["sha256"], "observation inputs differ")
    require(row["matrix_sha256"] == report["matrix_sha256"], "observation matrix differs")
    role = row.get("surface")
    if role is None:
        role = "gui" if "scenario_id" in row or row.get("budget_id") in {
            "gui-feedback", "native-window-open", "window-lifecycle"} else "cli"
    require(role in binaries and sha(row["binary_sha256"], "observation binary") == binaries[role],
            "observation executable does not match its surface")
    require(row["environment_id"] in environments, "unknown observation environment")
    return environments[row["environment_id"]]


def state_manifest(reference, bundle):
    state = bundle.json(reference)
    closed(state, "schema root files", "state manifest")
    require(state["schema"] == "datum.preferences.state.v2", "unknown state schema")
    text(state["root"], "state root identity")
    paths = []
    for entry in array(state["files"], "state files", nonempty=False):
        closed(entry, "path kind bytes sha256", "state file")
        from .inputs import relative
        paths.append(str(relative(entry["path"])))
        require(entry["kind"] in ("file", "absent"), "unsupported captured state kind")
        integer(entry["bytes"], "file size")
        if entry["kind"] == "file":
            sha(entry["sha256"], "state file digest")
            # Bundle construction verified every artifact's actual bytes. Reuse
            # that size instead of rereading all retained history each sample.
            require(bundle.validated_sizes.get(entry["sha256"]) == entry["bytes"],
                    "state bytes are not retained or their declared size differs")
        else:
            require(entry["bytes"] == 0 and entry["sha256"] is None, "absence has no bytes/digest")
    require(paths == sorted(set(paths)), "state paths must be sorted and unique")
    return state


def observation_context(row):
    return {k: row[k] for k in ("candidate_revision", "input_manifest_sha256", "matrix_sha256",
                                "binary_sha256", "environment_id", "fixture")}


def assertions(rows, expected, bundle, context):
    require(type(expected) is dict and bool(expected), "fixture expectations required")
    values = array(rows, "assertions")
    unique([r["id"] for r in values], "assertion ids")
    require({r["id"] for r in values} == set(expected), "missing or unknown assertion")
    for row in values:
        closed(row, "id expected observed evidence", "assertion")
        require(canonical(row["expected"]) == canonical(expected[row["id"]]),
                "assertion expectation differs from frozen fixture")
        predicate(row["expected"], row["observed"])
        observed = bundle.json(row["evidence"])
        closed(observed, "schema context id observed", "raw assertion")
        require(observed["schema"] == "datum.preferences.assertion.v2" and
                canonical(observed["context"]) == canonical(context) and
                observed["id"] == row["id"] and
                canonical(observed["observed"]) == canonical(row["observed"]),
                "assertion differs from retained raw observation")


def predicate(expected, observed):
    closed(expected, "operator value", "expectation")
    operator, value = expected["operator"], expected["value"]
    if operator == "equal":
        require(canonical(observed) == canonical(value), "assertion equality failed")
    elif operator == "one_of":
        array(value, "allowed outcomes")
        require(any(canonical(observed) == canonical(v) for v in value), "assertion outside allowed outcomes")
    elif operator in ("at_most", "at_least"):
        integer(value, "expected bound")
        integer(observed, "observed value")
        require(observed <= value if operator == "at_most" else observed >= value, "assertion bound failed")
    else:
        require(False, "unknown assertion operator")


def fixture(reference, bundle, required, report, coverage):
    value = bundle.json(reference)
    closed(value, "schema source_revision recipe parameters expected_assertions initial_state", "fixture")
    require(value["schema"] == "datum.preferences.fixture.v2" and
            value["source_revision"] == report["candidate"]["revision"], "fixture identity differs")
    closed(value["recipe"], "path sha256", "fixture recipe")
    require(value["recipe"] in bundle.json(report["input_manifest"]),
            "fixture recipe is not in candidate input closure")
    require(type(value["parameters"]) is dict, "explicit fixture parameters required")
    require(canonical(value["parameters"].get("coverage")) == canonical(coverage),
            "fixture parameters do not select this exact coverage row")
    require(type(value["expected_assertions"]) is dict and
            set(value["expected_assertions"]) == set(required), "fixture assertion inventory differs")
    require(all(canonical(v) == canonical({"operator": "equal", "value": True})
                for v in value["expected_assertions"].values()),
            "required affirmative predicates cannot be weakened by fixture expectations")
    state_manifest(value["initial_state"], bundle)
    return value


def command_observation(row, bundle, *, tests=None):
    command_line(row["command"])
    require(type(row["exit_code"]) is int and row["exit_code"] == 0,
            "failed or skipped proof command")
    raw = bundle.json(row["log"])
    closed(raw, "schema command exit_code executed_tests stdout stderr candidate_revision "
           "input_manifest_sha256", "command observation")
    require(raw["schema"] == "datum.preferences.command-observation.v2", "unknown command observation")
    for name in ("command", "exit_code", "candidate_revision", "input_manifest_sha256"):
        require(canonical(raw[name]) == canonical(row[name]), "raw command identity/result differs")
    stdout = bundle.read(raw["stdout"])
    bundle.read(raw["stderr"])
    integer(raw["executed_tests"], "executed tests")
    if tests is not None:
        integer(tests, "case executed tests", 1)
        require(raw["executed_tests"] == tests, "case test count differs from command capture")
        result = bundle.json(raw["stdout"])
        closed(result, "schema executed_tests observations", "case result output")
        require(result["schema"] == "datum.preferences.case-observations.v2" and
                type(result["executed_tests"]) is int and result["executed_tests"] == tests,
                "case output has no matching executed result count")
        require(type(result["observations"]) is dict and
                canonical(result["observations"]) == canonical({r["id"]: r["observed"] for r in row["assertions"]}),
                "reported assertions differ from actual case output")
    return stdout


def validate_cases(report, bundle, environments, binaries):
    required = case_inventory()
    require(len(array(report["case_results"], "case results")) == len(required), "case result count differs")
    observed = set()
    fields = ("case_id variant_id subcase_id surface environment_id candidate_revision binary_sha256 "
              "input_manifest_sha256 matrix_sha256 fixture command exit_code executed_tests assertions before after log")
    for row in array(report["case_results"], "case results"):
        closed(row, fields, "case result")
        coordinate = identity(row, report, environments, binaries)
        key = (row["case_id"], row["variant_id"], row["subcase_id"], row["surface"], coordinate)
        require(key in required and key not in observed, "unknown or duplicate case coverage")
        observed.add(key)
        frozen = fixture(row["fixture"], bundle, required[key], report, key)
        require(row["before"] == frozen["initial_state"], "case did not start from frozen fixture state")
        assertions(row["assertions"], frozen["expected_assertions"], bundle, observation_context(row))
        state_manifest(row["before"], bundle)
        state_manifest(row["after"], bundle)
        command_observation(row, bundle, tests=row["executed_tests"])
    require(observed == set(required), f"missing case results: {len(set(required) - observed)}")


def validate_native(report, bundle, environments, binaries):
    required = {(scenario, coordinate) for scenario in NATIVE_ASSERTIONS for coordinate in gui_coordinates()}
    require(len(array(report["native_observations"], "native observations")) == len(required),
            "native observation count differs")
    seen = set()
    for row in array(report["native_observations"], "native observations"):
        closed(row, "scenario_id environment_id candidate_revision binary_sha256 input_manifest_sha256 "
               "matrix_sha256 fixture actions assertions capture accessibility_capture state_evidence", "native observation")
        coordinate = identity(row, report, environments, binaries)
        key = (row["scenario_id"], coordinate)
        require(key in required and key not in seen, "unknown or duplicate native scenario")
        seen.add(key)
        require(row["binary_sha256"] == report["candidate"]["binaries"]["gui"]["sha256"],
                "native observation did not use candidate GUI")
        frozen = fixture(row["fixture"], bundle, NATIVE_ASSERTIONS[row["scenario_id"]], report, key)
        for action in array(row["actions"], "actual user actions"):
            text(action, "user action")
        context = observation_context(row)
        assertions(row["assertions"], frozen["expected_assertions"], bundle, context)
        for field, kind in (("capture", "native"), ("accessibility_capture", "at-spi")):
            capture = bundle.json(row[field])
            closed(capture, "schema context kind actions tool payload", "native capture manifest")
            require(capture["schema"] == "datum.preferences.native-capture.v2" and
                    capture["kind"] == kind and canonical(capture["context"]) == canonical(context)
                    and capture["actions"] == row["actions"], "native capture execution identity differs")
            text(capture["tool"], "native capture tool/version")
            require(bool(bundle.read(capture["payload"])), "native capture payload missing")
        state_manifest(row["state_evidence"], bundle)
    require(seen == required, f"missing native observations: {len(required - seen)}")


def validate_gates(report, bundle, root):
    from .gates import commands
    required = commands(root)
    require(len(array(report["gate_results"], "gate results")) == len(required), "mandatory gate count differs")
    seen = set()
    for row in array(report["gate_results"], "gate results"):
        closed(row, "id candidate_revision input_manifest_sha256 commands", "gate result")
        require(row["id"] in GATES and row["id"] not in seen, "unknown or duplicate gate")
        seen.add(row["id"])
        require(row["candidate_revision"] == report["candidate"]["revision"] and
                row["input_manifest_sha256"] == report["input_manifest"]["sha256"], "gate identity differs")
        observations = array(row["commands"], "gate commands")
        require([o["command"] for o in observations] == required[row["id"]],
                "gate substituted or omitted a mandatory command")
        for observation in observations:
            closed(observation, "command candidate_revision input_manifest_sha256 exit_code log", "gate command")
            require(observation["candidate_revision"] == row["candidate_revision"] and
                    observation["input_manifest_sha256"] == row["input_manifest_sha256"],
                    "gate command input identity differs")
            stdout = command_observation(observation, bundle)
            if "unittest" in observation["command"] or "test" in observation["command"]:
                from .capture import executed_tests
                raw = bundle.json(observation["log"])
                count = executed_tests(stdout + b"\n" + bundle.read(raw["stderr"]))
                require(count > 0 and count == raw["executed_tests"],
                        "mandatory test gate executed zero tests or its count differs")
    require(seen == set(GATES), "missing mandatory gates")
