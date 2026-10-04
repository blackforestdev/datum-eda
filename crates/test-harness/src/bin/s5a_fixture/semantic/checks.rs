//! Exact independently enumerated occurrence sets; complete source/journal reuse.
use super::*;
use anyhow::ensure;
use serde_json::json;
pub(super) fn check(path: &Path, net: Uuid) -> Result<serde_json::Value> {
    let model = ProjectResolver::new(path).resolve()?;
    let mut files = model
        .source_shards
        .iter()
        .map(|s| Ok((s.path.clone(), std::fs::read(&s.path)?)))
        .collect::<Result<Vec<_>>>()?;
    let journal = eda_engine::substrate::transaction_journal_path(path);
    files.push((journal.clone(), std::fs::read(journal)?));
    let snap = ElectricalSelectionSnapshot::capture(&model)
        .map_err(|e| anyhow::anyhow!("capture: {e:?}"))?;
    let expected_local: BTreeSet<_> = [
        ("wires", 8),
        ("wires", 9),
        ("labels", 10),
        ("junctions", 11),
        ("ports", 12),
    ]
    .into_iter()
    .map(|(c, n)| occurrence(c, n, &[]))
    .collect();
    let mut expected_global = BTreeSet::new();
    for path in [vec![], vec![id(31)], vec![id(32)]] {
        for (c, n) in [
            ("wires", 8),
            ("wires", 9),
            ("wires", 13),
            ("labels", 10),
            ("labels", 14),
            ("junctions", 11),
            ("ports", 12),
        ] {
            expected_global.insert(occurrence(
                c,
                n + if path.is_empty() { 0 } else { 100 },
                &path,
            ));
        }
    }
    let local = snap
        .schematic_run(&model.model_revision, &occurrence("wires", 8, &[]))
        .map_err(|e| anyhow::anyhow!("Run: {e:?}"))?;
    ensure!(
        local.owned == expected_local,
        "complete local physical Wire Run"
    );
    let global = snap
        .global_net(&model.model_revision, net)
        .map_err(|e| anyhow::anyhow!("Global Net: {e:?}"))?;
    ensure!(
        global.owned == expected_global,
        "complete Global Net across disconnected geometry and repeated sheet occurrences"
    );
    let origin = occurrence("buses", 1, &[]);
    let run = snap
        .bus_run(&model.model_revision, &origin)
        .map_err(|e| anyhow::anyhow!("Bus Run: {e:?}"))?;
    let expected_run: BTreeSet<_> = [("buses", 1), ("buses", 2), ("bus_entries", 5)]
        .into_iter()
        .map(|(c, n)| occurrence(c, n, &[]))
        .collect();
    ensure!(
        run.membership.owned == expected_run,
        "Bus Run constrained to origin declaration; entry source association retained"
    );
    ensure!(
        run.contacts.len() == 1 && run.contacts.first().unwrap().foreign_bus == id(42),
        "distinct declaration touching spine remains separate foreign contact"
    );
    let mut expected_bus = BTreeSet::new();
    for path in [vec![], vec![id(31)]] {
        expected_bus.extend(reps(
            &path,
            &if path.is_empty() {
                [1, 2, 3]
            } else {
                [101, 102, 103]
            },
        ));
        expected_bus.insert(occurrence(
            "bus_entries",
            if path.is_empty() { 5 } else { 105 },
            &path,
        ));
    }
    let bus = snap
        .bus(&model.model_revision, id(41))
        .map_err(|e| anyhow::anyhow!("Bus: {e:?}"))?;
    ensure!(
        bus.owned == expected_bus,
        "complete explicit identity-bound Bus membership"
    );
    let mut expected_related = expected_global.clone();
    expected_related.extend(reps(&[id(32)], &[101, 102, 103]));
    expected_related.insert(occurrence("bus_entries", 105, &[id(32)]));
    ensure!(
        bus.related == expected_related && bus.related_subjects == BTreeSet::from([net, id(43)]),
        "scalar Net and distinct interface declaration remain related only"
    );
    let inventory = snap
        .authored_sources()
        .map_err(|e| anyhow::anyhow!("sources: {e:?}"))?;
    for path in [vec![], vec![id(31)], vec![id(32)]] {
        for (c, n) in [
            ("ports", 12),
            ("junctions", 11),
            ("labels", 10),
            ("buses", 1),
        ] {
            ensure!(
                inventory.iter().any(|s| s.occurrence
                    == occurrence(c, n + if path.is_empty() { 0 } else { 100 }, &path)),
                "each authored occurrence retained"
            );
        }
    }
    for (path, bytes) in files {
        ensure!(
            std::fs::read(&path)? == bytes,
            "read-only source/journal unchanged: {}",
            path.display()
        );
    }
    let reopen = ProjectResolver::new(path).resolve()?;
    ensure!(
        reopen.model_revision == model.model_revision,
        "reopen source basis"
    );
    Ok(
        json!({"path":path,"model_revision":model.model_revision,"project_id":model.project.project_id,"source_basis":snap.source_basis(),"sheet_instances":[id(31),id(32)],"logical_net":net,"primary_bus":id(41),"foreign_bus":id(42),"related_bus":id(43),"expected_wire_run_members":expected_local,"expected_global_net_members":expected_global,"expected_bus_run_members":expected_run,"expected_bus_members":expected_bus,"expected_related":expected_related,"contact_evidence":run.contacts,"nonmutation":"complete source shards and full journal compared byte-for-byte around captured queries/reopen","native_input":"pending: three acquisition tiers, pre-acquisition hidden/filter, modifier refusal/no-op and unprojected qualified-members disclosure","acceptance":"partial C03 fixture preparation; no C04/C05/C06 acceptance"}),
    )
}
