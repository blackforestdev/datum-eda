"""Category transition requirements supplement, never replace, PM025 validation."""

from workflow_delivery_shapes import require


def terminal_owner_closeout(item, prior):
    """Permit only completion of the last already-promoted owner decision.

    PM025 still validates owner evidence, tracker closure and landing commit.
    This creates no execution, source permission or product acceptance itself.
    """
    old = prior.get("completion", {})
    current = item.get("completion", {})
    selected = old.get("canonical_next_step_id")
    old_steps = old.get("steps", [])
    old_step = next((s for s in old_steps if s.get("id") == selected), {})
    step = next((s for s in current.get("steps", []) if s.get("id") == selected), {})
    return (
        prior.get("authorization") == "owner_decision"
        and not prior.get("claim")
        and old_step.get("kind") == "owner_decision"
        and old_step.get("status") == "pending"
        and all(s.get("status") == "complete" for s in old_steps if s.get("id") != selected)
        and item.get("state") == "landed"
        and item.get("authorization") == "none"
        and not item.get("claim")
        and current.get("canonical_next_step_id") is None
        and step.get("status") == "complete"
        and bool(step.get("completion_evidence"))
    )


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
            prior = promoted.get(key, {})
            # Dormancy can retain completed work from before the owner hold.
            # Preserve that exact record; do not mistake it for new execution
            # or allow candidate-only completion/requirement changes under it.
            live_execution = any(s.get("kind") == "execution" and
                                 s.get("status") == "in_progress" for s in steps)
            require(item.get("state") == promoted.get(key, {}).get("state") and
                    item.get("authorization") == "none" and not item.get("claim")
                    and not live_execution and completion == prior.get("completion", {}),
                    key, "deferred work requires reclassification before execution or completion changes",
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
            terminal = terminal_owner_closeout(item, prior)
            require(terminal or item.get("authorization") == prior.get("authorization"), key,
                    "external-lane authorization changed; coordinated promotion required",
                    "WDQ-COVERAGE")
            require(all(item.get(field) == prior.get(field)
                        for field in ("dependencies", "governing_docs")), key,
                    "external-lane governing boundary changed; coordinated promotion required",
                    "WDQ-COVERAGE")
            mutable_fields = {"steps", "canonical_next_step_id"} if terminal else {"steps"}
            require({k: v for k, v in completion.items() if k not in mutable_fields} ==
                    {k: v for k, v in old.items() if k not in mutable_fields}, key,
                    "external-lane completion boundary changed; coordinated promotion required",
                    "WDQ-COVERAGE")
            selected = old.get("canonical_next_step_id")
            require(terminal or completion.get("canonical_next_step_id") == selected, key,
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
