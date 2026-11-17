//! Six real physical history families over the same two-user managed project.
//! Generated fixture proof supplements the separately owned real GUI workflow.
use super::*;

fn cold_geometry(f: &mut Fixture) -> (BTreeMap<Uuid, Value>, BTreeMap<Uuid, Option<Uuid>>) {
    let revision = f.model.model_revision.clone();
    let canonical = files(&f.root);
    f.model = ProjectResolver::new(&f.root).resolve().unwrap();
    assert_eq!(f.model.model_revision, revision);
    let geometry = qualified_geometry(f);
    assert_eq!(files(&f.root), canonical, "cold read must remain read-only");
    geometry
}

fn run_sequence() {
    let mut f = two_users_with_copper();
    let before = board(&f);
    let schematic = files(&f.root)
        .into_iter()
        .filter(|(path, _)| path.starts_with("schematic"))
        .collect();
    let initial_ids = qualified_geometry(&f).0.keys().copied().collect::<Vec<_>>();
    for footprint in [true, false] {
        let old_geometry = qualified_geometry(&f);
        let old_lock = std::fs::read(f.root.join("library/lock.json")).unwrap();
        let id = if footprint { f.footprint } else { f.stack };
        let base = f.active(id);
        let mut payload = snapshot(&f.root, f.limits).assets()[&base]
            .payload()
            .value()
            .clone();
        let mut expected = old_geometry.clone();
        if footprint {
            for pad in payload["pads"].as_object_mut().unwrap().values_mut() {
                let x = pad["position"]["x"].as_i64().unwrap();
                pad["position"]["x"] = json!(x + 100000);
            }
            for pad in expected.0.values_mut() {
                let x = pad["position"]["x"].as_i64().unwrap();
                pad["position"]["x"] = json!(x + 100000);
            }
        } else {
            payload["aperture"]["circle"]["diameter_nm"] = json!(1200000);
            for pad in expected.0.values_mut() {
                pad["aperture"]["circle"]["diameter_nm"] = json!(1200000);
            }
        }
        let candidate = f.revise(&base, payload);
        assert_eq!(qualified_geometry(&f), old_geometry);
        assert_eq!(
            std::fs::read(f.root.join("library/lock.json")).unwrap(),
            old_lock
        );
        let mut request = f.request(&candidate);
        let canonical = files(&f.root);
        let context = native::registry::NativeWriteContext {
            model: &f.model,
            project_root: &f.root,
        };
        let review = native::library_assets::inspect_geometry_selection(
            &context,
            request["request"].clone(),
        )
        .unwrap();
        assert_eq!(
            review["physical_geometry"]["placements"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        request["request"]["physical_geometry"] = review["physical_geometry"].clone();
        let change =
            native::library_assets::prepare_library_change(&context, public_provenance(), request)
                .unwrap();
        assert_eq!(files(&f.root), canonical);
        native::commit_prepared(&mut f.model, &f.root, change.prepared).unwrap();
        assert_eq!(cold_geometry(&mut f), expected);
        let adopted_lock = std::fs::read(f.root.join("library/lock.json")).unwrap();
        assert_eq!(f.active(id), candidate);
        unchanged_design(&f, &before, &schematic);

        f.model
            .commit_journal_undo(&f.root, public_provenance().into())
            .unwrap();
        assert_eq!(cold_geometry(&mut f), old_geometry);
        assert_eq!(f.active(id), base);
        assert_eq!(
            std::fs::read(f.root.join("library/lock.json")).unwrap(),
            old_lock
        );
        unchanged_design(&f, &before, &schematic);

        f.model
            .commit_journal_redo(&f.root, public_provenance().into())
            .unwrap();
        assert_eq!(cold_geometry(&mut f), expected);
        assert_eq!(f.active(id), candidate);
        assert_eq!(
            std::fs::read(f.root.join("library/lock.json")).unwrap(),
            adopted_lock
        );
        assert_eq!(
            qualified_geometry(&f).0.keys().copied().collect::<Vec<_>>(),
            initial_ids
        );
        unchanged_design(&f, &before, &schematic);
    }
    crate::substrate::library_asset_journal_ops::history::Proof::assert_cache_identity(
        &f.model.journal,
    );
    std::fs::remove_dir_all(f.root).unwrap();
}

#[test]
fn public_sequential_footprint_padstack_history_preserves_users_and_cold_readback() {
    const NAME: &str = concat!(
        module_path!(),
        "::public_sequential_footprint_padstack_history_preserves_users_and_cold_readback"
    );
    if !isolated(NAME) {
        run_sequence();
    }
}
