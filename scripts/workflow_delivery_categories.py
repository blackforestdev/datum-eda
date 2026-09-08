"""Category transition requirements supplement, never replace, PM025 validation."""

from workflow_delivery_shapes import require


def validate_categories(items, coverage, enrolled, authority):
    readiness = []
    promoted = {i["key"]: i for i in authority.json("specs/active_frontier.json")["frontier"]}
    for row in coverage["rows"]:
        key, category = row["frontier_key"], row["category"]
        item = items[key]
        if category == "historical":
            continue  # Baseline/evidence immutability checked by classifications.
        completion = item.get("completion", {})
        steps = completion.get("steps", [])
        executing = item.get("authorization") == "execution" or any(
            s.get("kind") == "execution" and s.get("status") in ("in_progress", "complete")
            for s in steps)
        if category == "deferred":
            require(item.get("state") == "deferred" and
                    item.get("authorization") == "none" and not item.get("claim")
                    and not executing, key, "deferred work requires reclassification before execution",
                    "WDQ-COVERAGE")
            continue
        require(bool(steps), key, "classified active work requires a completion plan", "WDQ-COVERAGE")
        if category == "specification":
            require(not executing and all(s.get("kind") != "execution" for s in steps), key,
                    "specification classification cannot authorize implementation", "WDQ-COVERAGE")
        elif category in ("product", "infrastructure"):
            # Without a declaration there is no trusted ready mapping. Refuse
            # completion of preflight rather than allowing that omission to
            # postpone readiness indefinitely behind an owner-step label.
            preflight_done = any(s.get("kind") in ("planning", "governance") and
                                 s.get("status") == "complete" for s in steps)
            require(not (executing or preflight_done) or "delivery" in completion,
                    key, "readiness/execution requires a delivery declaration", "WDQ-COVERAGE")
            require(not executing or key in enrolled, key,
                    "execution requires owner-promoted enrollment", "WDQ-COVERAGE")
            if executing or preflight_done:
                readiness.append(key)
        elif category == "external_lane":
            prior = promoted.get(key, {})
            old = prior.get("completion", {})
            require(item.get("authorization") == prior.get("authorization"), key,
                    "external-lane authorization changed; coordinated promotion required",
                    "WDQ-COVERAGE")
            require(all(item.get(field) == prior.get(field)
                        for field in ("dependencies", "governing_docs")), key,
                    "external-lane governing boundary changed; coordinated promotion required",
                    "WDQ-COVERAGE")
            require({k: v for k, v in completion.items() if k != "steps"} ==
                    {k: v for k, v in old.items() if k != "steps"}, key,
                    "external-lane completion boundary changed; coordinated promotion required",
                    "WDQ-COVERAGE")
            selected = completion.get("canonical_next_step_id")
            require(selected == old.get("canonical_next_step_id"), key,
                    "external-lane selected boundary changed; coordinated promotion required",
                    "WDQ-COVERAGE")
            before = {s["id"]: s for s in old.get("steps", []) if s["id"] != selected}
            after = {s["id"]: s for s in steps if s["id"] != selected}
            require(before == after, key, "external-lane work crossed the promoted boundary",
                    "WDQ-COVERAGE")
            old_step = next((s for s in old.get("steps", []) if s["id"] == selected), {})
            new_step = next((s for s in steps if s["id"] == selected), {})
            mutable = {"status", "completion_evidence"}
            require({k: v for k, v in old_step.items() if k not in mutable} ==
                    {k: v for k, v in new_step.items() if k not in mutable}, key,
                    "external-lane selected requirement/authority changed", "WDQ-COVERAGE")
    return readiness
