//! Preserve the existing authored pool; seed only the missing native scaffold.
use anyhow::{Context, Result, ensure};
use eda_engine::{
    api::native_write::{
        self, WriteProvenance, genesis,
        library::{PoolLibraryObjectTarget, PoolLibraryOperationSpec, build_pool_library_write},
    },
    board::{StackupLayer, StackupLayerType},
    substrate::{CommitSource, ProjectResolver, SourceShardKind},
};
use std::path::Path;
pub(super) fn prepare(root: &Path) -> Result<()> {
    ensure!(
        !root.exists(),
        "fresh output root required; source corpus preserved"
    );
    let corpus =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/library/native_authored_baseline_v1");
    let ids = genesis::GenesisRootIds {
        project: uuid::Uuid::from_u128(0x72000000000040008000000000000001),
        schematic: uuid::Uuid::from_u128(0x72000000000040008000000000000002),
        board: uuid::Uuid::from_u128(0x72000000000040008000000000000003),
        rules: Some(uuid::Uuid::from_u128(0x72000000000040008000000000000004)),
    };
    genesis::bootstrap_native_project(
        root,
        genesis::GenesisSpec {
            project_name: "S5A actual library fixture".into(),
            existing_ids: Some(ids),
        },
    )?;
    let mut model = ProjectResolver::new(root).resolve()?;
    let provenance = || {
        WriteProvenance::new(
            "s5a-library-fixture",
            CommitSource::Test,
            "reuse Datum-authored pool through canonical library writes",
        )
    };
    let mut specs = Vec::new();
    let mut groups: Vec<_> =
        std::fs::read_dir(corpus.join("pool"))?.collect::<std::io::Result<_>>()?;
    groups.sort_by_key(|e| e.file_name());
    for group in groups {
        let kind = group.file_name().to_string_lossy().into_owned();
        let mut files: Vec<_> = std::fs::read_dir(group.path())?.collect::<std::io::Result<_>>()?;
        files.sort_by_key(|e| e.file_name());
        for file in files {
            let object: serde_json::Value = serde_json::from_slice(&std::fs::read(file.path())?)?;
            let uuid =
                uuid::Uuid::parse_str(object["uuid"].as_str().context("actual pool object UUID")?)?;
            specs.push(PoolLibraryOperationSpec::Create {
                target: PoolLibraryObjectTarget::new("pool", &kind, uuid),
                object,
            });
        }
    }
    let write = build_pool_library_write(&model, provenance(), Some("pool"), specs)?;
    native_write::commit_prepared(&mut model, root, write)?;
    let sheet: serde_json::Value =
        serde_json::from_slice(&std::fs::read(corpus.join("schematic/sheets/main.json"))?)?;
    let sheet_id = uuid::Uuid::parse_str(sheet["uuid"].as_str().context("actual sheet UUID")?)?;
    let write = native_write::schematic_sheets::build_create_schematic_sheet(
        &model,
        provenance(),
        ids.schematic,
        sheet_id,
        "main.json",
        sheet,
    )?;
    native_write::commit_prepared(&mut model, root, write)?;
    let board = model.materialized_source_shard_value(SourceShardKind::BoardRoot)?;
    let id = uuid::Uuid::parse_str(board["uuid"].as_str().context("actual board UUID")?)?;
    let layers = [
        StackupLayer::new(1, "F.Cu", StackupLayerType::Copper, 35000),
        StackupLayer::new(2, "F.Mask", StackupLayerType::SolderMask, 10000),
        StackupLayer::new(3, "F.SilkS", StackupLayerType::Silkscreen, 10000),
        StackupLayer::new(4, "F.Paste", StackupLayerType::Paste, 10000),
        StackupLayer::new(45, "F.CrtYd", StackupLayerType::Mechanical, 0),
        StackupLayer::new(49, "F.Fab", StackupLayerType::Mechanical, 0),
    ];
    let write = native_write::board_layout::build_set_board_stackup(
        &model,
        WriteProvenance::new(
            "s5a-library-fixture",
            CommitSource::Test,
            "canonical native scaffold for cloned library corpus",
        ),
        id,
        &layers,
    )?;
    native_write::commit_prepared(&mut model, root, write)?;
    println!("{}", root.display());
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn library_source_is_preserved_and_stackup_is_canonical() {
        let root = std::env::temp_dir().join(format!("s5a-library-test-{}", uuid::Uuid::new_v4()));
        prepare(&root).unwrap();
        let corpus = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("testdata/library/native_authored_baseline_v1");
        for name in [
            "pool/parts/90000000-0000-4000-8000-000000000001.json",
            "pool/footprints/60000000-0000-4000-8000-000000000001.json",
            "pool/symbols/30000000-0000-4000-8000-000000000001.json",
        ] {
            let mut actual: serde_json::Value =
                serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap();
            let mut expected: serde_json::Value =
                serde_json::from_slice(&std::fs::read(corpus.join(name)).unwrap()).unwrap();
            actual.as_object_mut().unwrap().remove("object_revision");
            expected.as_object_mut().unwrap().remove("object_revision");
            assert_eq!(
                actual, expected,
                "all actual library definition fields preserved"
            );
        }
        let model = ProjectResolver::new(&root).resolve().unwrap();
        assert_eq!(model.journal.len(), 3);
        assert!(
            model
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .unwrap()["stackup"]["layers"]
                .as_array()
                .unwrap()
                .len()
                >= 6
        );
        assert!(prepare(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
