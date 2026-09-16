"""Compare complete raw trials against fixed limits; never trust supplied summaries."""

from math import ceil

from .inputs import array, canonical, closed, integer, require, unique, validation
from .inventory import BUDGETS, TIMED_VARIANTS
from .matrix import gui_coordinates


def percentile(values, fraction=0.95):
    require(bool(values), "percentile requires samples")
    return sorted(values)[ceil(len(values) * fraction) - 1]


def measurement_inventory():
    expected = set()
    for budget, variants in TIMED_VARIANTS.items():
        coordinates = gui_coordinates() if budget in ("gui-feedback", "native-window-open") else (None,)
        expected.update((budget, variant, coordinate) for variant in variants for coordinate in coordinates)
    expected.update(("storage-growth", variant, None)
                    for variant in ("factory", "global-missing", "global-populated", "generation"))
    expected.update(("window-lifecycle", "cycles", coordinate) for coordinate in gui_coordinates())
    return expected


@validation
def validate_trials(measurement, bundle):
    budget = measurement["budget_id"]
    require(budget in BUDGETS, "unknown measurement budget")
    limits = BUDGETS[budget]
    trials = array(measurement["trials"], "trials")
    require(len(trials) == 3 and [t["index"] for t in trials] == [0, 1, 2],
            f"{budget}: three ordered trials required")
    previous_start = 0
    for trial in trials:
        closed(trial, "index warmup_count warmups baseline_rss_kib baseline_resources cold_open samples log", "trial")
        integer(trial["index"], "trial index")
        require(type(trial["warmup_count"]) is int and trial["warmup_count"] == 10,
                "ten observed warm-ups required")
        raw_trial = bundle.json(trial["log"])
        expected_raw = {k: v for k, v in trial.items() if k != "log"}
        expected_raw.update(schema="datum.preferences.measurement-trial.v2",
                            measurement={k: v for k, v in measurement.items() if k != "trials"})
        require(canonical(raw_trial) == canonical(expected_raw),
                "reported trial differs from retained raw observations")
        if budget == "window-lifecycle":
            integer(trial["baseline_rss_kib"], "warmed RSS baseline", 1)
            closed(trial["baseline_resources"], "owned_windows writer_leases", "baseline resources")
            for resources in trial["baseline_resources"].values():
                _resources(resources)
        else:
            require(trial["baseline_rss_kib"] is None, "baseline is lifecycle-only")
            require(trial["baseline_resources"] is None, "resource baseline is lifecycle-only")
        if budget == "native-window-open":
            _timed_sample(trial["cold_open"], budget, limits, bundle)
            require(trial["cold_open"]["index"] == 0, "cold-open index must be zero")
        else:
            require(trial["cold_open"] is None, "cold open is window-open-only")
        samples = array(trial["samples"], "raw samples")
        require(len(samples) == 100 and [s["index"] for s in samples] == list(range(100)),
                "each trial needs all 100 ordered raw samples")
        warmups = array(trial["warmups"], "warm-up observations")
        require(len(warmups) == 10 and [s["index"] for s in warmups] == list(range(10)),
                "all ten warm-up observations required")
        for sample in warmups:
            require(type(sample) is dict and set(sample) == set(samples[0]), "warm-up shape differs")
            # Warm-ups retain the full observed shape; only their numeric budgets
            # are excluded. Failed operations and leaked resources are never ignored.
            unbounded = {key: (float("inf") if key in {"p95_ms", "max_ms"} else value)
                         for key, value in limits.items()}
            if budget == "storage-growth":
                _storage_sample(sample, measurement["variant_id"], unbounded, bundle)
            elif budget == "window-lifecycle":
                _lifecycle_sample(sample, trial["baseline_rss_kib"], unbounded, bundle,
                                  trial["baseline_resources"])
            else:
                _timed_sample(sample, budget, unbounded, bundle)
                if budget == "durable-mutation":
                    require((sample["daemon_rss_kib"] is not None) == measurement["variant_id"].endswith("/daemon"),
                            "warm-up daemon accounting differs")
        ordered = ([trial["cold_open"]] if trial["cold_open"] else []) + warmups + samples
        starts = [s["started_monotonic_ns"] for s in ordered]
        require(all(a < b for a, b in zip(starts, starts[1:])), "measurement executions repeated or out of order")
        require(starts[0] > previous_start, "measurement trials repeat or overlap executions")
        previous_start = starts[-1]
        if budget == "window-lifecycle":
            require(trial["baseline_rss_kib"] == warmups[-1]["post_close_rss_kib"] and
                    all(trial["baseline_resources"][k] == warmups[-1][k]
                        for k in ("owned_windows", "writer_leases")),
                    "lifecycle baseline differs from the final warmed observation")
        if budget == "storage-growth" and measurement["variant_id"] == "generation":
            mutations = warmups + samples
            require(all(a["after"] == b["before"] for a, b in zip(mutations, mutations[1:])),
                    "successive storage mutations do not form a continuous retained history")
        elapsed = []
        for sample in samples:
            if budget == "storage-growth":
                _storage_sample(sample, measurement["variant_id"], limits, bundle)
            elif budget == "window-lifecycle":
                _lifecycle_sample(sample, trial["baseline_rss_kib"], limits, bundle,
                                  trial["baseline_resources"])
            else:
                elapsed.append(_timed_sample(sample, budget, limits, bundle))
                if budget == "durable-mutation":
                    require((sample["daemon_rss_kib"] is not None) ==
                            measurement["variant_id"].endswith("/daemon"),
                            "daemon measurements require separate observed daemon RSS")
        if elapsed:
            require(percentile(elapsed) <= limits["p95_ms"] * 1_000_000,
                    f"{budget}: trial {trial['index']} p95 exceeds budget")


def _identity(sample, bundle):
    integer(sample["index"], "sample index")
    integer(sample["started_monotonic_ns"], "monotonic start", 1)
    require(sample["outcome"] == "success", "failed or skipped measurement")
    bundle.read(sample["evidence"])


def _timed_sample(sample, budget, limits, bundle):
    closed(sample, "kind index started_monotonic_ns elapsed_ns rss_kib daemon_rss_kib launch_to_response_ns "
           "presented_elapsed_ns outcome evidence", "timed sample")
    require(sample["kind"] == "timed", "timed sample required")
    _identity(sample, bundle)
    elapsed = integer(sample["elapsed_ns"], "elapsed nanoseconds")
    integer(sample["rss_kib"], "process RSS", 1)
    require(elapsed <= limits["max_ms"] * 1_000_000, f"{budget}: maximum exceeded")
    if "max_rss_mib" in limits:
        require(sample["rss_kib"] <= limits["max_rss_mib"] * 1024, f"{budget}: RSS exceeded")
    if budget == "durable-mutation":
        integer(sample["launch_to_response_ns"], "launch-to-response nanoseconds")
        require(sample["launch_to_response_ns"] >= elapsed, "mutation interval exceeds full interval")
        if sample["daemon_rss_kib"] is not None:
            integer(sample["daemon_rss_kib"], "daemon RSS", 1)
            require(sample["daemon_rss_kib"] <= limits["max_rss_mib"] * 1024,
                    "daemon RSS exceeded")
    else:
        require(sample["launch_to_response_ns"] is None and sample["daemon_rss_kib"] is None,
                "mutation-only accounting present on other measurement")
    if budget in ("gui-feedback", "native-window-open"):
        integer(sample["presented_elapsed_ns"], "presented-frame nanoseconds")
        require(sample["presented_elapsed_ns"] >= elapsed if budget == "gui-feedback"
                else sample["presented_elapsed_ns"] <= elapsed,
                "native timing endpoints contradict presentation/interaction order")
        require(sample["presented_elapsed_ns"] <= limits["max_ms"] * 1_000_000,
                "native presentation maximum exceeded")
    else:
        require(sample["presented_elapsed_ns"] is None, "presentation is GUI-only")
    return elapsed


def _storage_sample(sample, variant, limits, bundle):
    closed(sample, "kind index started_monotonic_ns project_bytes generation_bytes receipt_bytes request_index_bytes "
           "fixed_overhead_bytes before after outcome evidence", "storage sample")
    require(sample["kind"] == "storage", "storage sample required")
    _identity(sample, bundle)
    for field in ("project_bytes", "generation_bytes", "receipt_bytes", "request_index_bytes",
                  "fixed_overhead_bytes"):
        integer(sample[field], field)
    from .observations import state_manifest
    before, after = (state_manifest(sample[field], bundle) for field in ("before", "after"))
    require(before["root"] == after["root"], "storage roots differ")
    # A capture recipe uses these virtual roots to inventory both physical stores.
    categories = {"project": "project_bytes", "generations": "generation_bytes",
                  "receipts": "receipt_bytes", "request-index": "request_index_bytes",
                  "fixed": "fixed_overhead_bytes"}
    totals = []
    for state in (before, after):
        sizes = {field: 0 for field in categories.values()}
        for entry in state["files"]:
            category = entry["path"].split("/", 1)[0]
            require(category in categories, "unclassified storage file")
            sizes[categories[category]] += entry["bytes"]
        totals.append(sizes)
    for field in categories.values():
        require(sample[field] == totals[1][field] - totals[0][field],
                "reported storage growth differs from per-file manifests: " + field)
    before_files = {e["path"]: e for e in before["files"] if e["kind"] == "file"}
    after_files = {e["path"]: e for e in after["files"] if e["kind"] == "file"}
    for name, entry in before_files.items():
        if name.split("/", 1)[0] != "fixed":
            require(after_files.get(name) == entry, "retained immutable storage was removed or rewritten")
    if variant == "generation":
        require(sample["project_bytes"] == 0, "preference mutation cannot grow a Project")
        require(sample["generation_bytes"] > 0, "durable mutation generation is missing")
        require(sum(sample[f] for f in ("generation_bytes", "receipt_bytes", "request_index_bytes"))
                <= limits["per_preference_generation_kib"] * 1024,
                "individual preference generation exceeds storage budget")
    else:
        require(0 < sample["project_bytes"] <= limits["empty_project_kib"] * 1024,
                "empty Project exceeds storage budget or is missing")
        require(all(sample[f] == 0 for f in ("generation_bytes", "receipt_bytes", "request_index_bytes")),
                "genesis must not mutate preference storage")


def _lifecycle_sample(sample, baseline, limits, bundle, baseline_resources):
    closed(sample, "kind index started_monotonic_ns post_close_rss_kib orphan_windows orphan_writer_leases "
           "owned_windows writer_leases outcome evidence", "lifecycle sample")
    require(sample["kind"] == "lifecycle", "lifecycle sample required")
    _identity(sample, bundle)
    integer(sample["post_close_rss_kib"], "post-close RSS", 1)
    require(sample["post_close_rss_kib"] - baseline <= limits["max_rss_growth_mib"] * 1024,
            "post-close peak RSS growth exceeded")
    for name in ("orphan_windows", "orphan_writer_leases"):
        integer(sample[name], name)
        require(sample[name] == limits[name], f"{name}: orphan remains")
    for name in ("owned_windows", "writer_leases"):
        _resources(sample[name])
        require(canonical(sample[name]) == canonical(baseline_resources[name]),
                "post-close resources differ from observed warmed baseline")


def _resources(values):
    for value in array(values, "resources", nonempty=False):
        closed(value, "identity owner", "resource")
        from .inputs import text
        text(value["identity"], "resource identity")
        text(value["owner"], "resource owner")
    unique([v["identity"] for v in values], "resource identities")
