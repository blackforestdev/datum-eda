use super::*;
use crate::api::native_write::{board_layout, commit_prepared, test_support};
use crate::board::{StackupLayer, StackupLayerType, graphics::GraphicGeometry};
use crate::import::kicad::{board_graphic_import_key, native_graphic_sources};
use crate::substrate::{CommitSource, ImportMapEntryStatus, ProjectResolver};
use std::path::PathBuf;

struct Fixture {
    root: PathBuf,
    model: DesignModel,
    board: ObjectId,
    source: PathBuf,
}
fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "import-test",
        CommitSource::Test,
        "PM057 imported graphic history",
    )
}
impl Fixture {
    fn new() -> Self {
        let root = test_support::temp_project_root("graphic-import");
        let created = crate::api::native_write::genesis::bootstrap_native_project(
            &root,
            crate::api::native_write::genesis::GenesisSpec {
                project_name: "graphic-import".into(),
                existing_ids: None,
            },
        )
        .unwrap();
        let board = created.board_uuid;
        let mut model = ProjectResolver::new(&root).resolve().unwrap();
        let write = board_layout::build_set_board_stackup(
            &model,
            provenance(),
            board,
            &[StackupLayer::new(
                37,
                "F.SilkS",
                StackupLayerType::Silkscreen,
                0,
            )],
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        let write = board_layout::source_profile::build_adopt_board_source_profile(
            &model,
            provenance(),
            board,
        )
        .unwrap();
        commit_prepared(&mut model, &root, write).unwrap();
        let source = root.join("input.kicad_pcb");
        Self {
            root,
            model,
            board,
            source,
        }
    }
    fn source(&self, end: i64) {
        std::fs::write(&self.source,format!("(kicad_pcb (gr_line (start 0 0) (end {end} 0) (width 0.1) (layer \"F.SilkS\") (uuid {})) (gr_rect (start 0 1) (end 2 3) (fill solid) (width 0) (layer \"F.SilkS\") (uuid {})))",Uuid::from_u128(1),Uuid::from_u128(2))).unwrap();
    }
    fn imported_board(&self) -> Board {
        let (mut board, _) = crate::import::kicad::import_board_document(&self.source).unwrap();
        board.stackup = serde_json::from_value(
            self.model
                .materialized_source_shard_value(SourceShardKind::BoardRoot)
                .unwrap()["stackup"]
                .clone(),
        )
        .unwrap();
        board
    }
    fn prepared(&self) -> ImportWrite {
        let board = self.imported_board();
        let plan = native_graphic_sources(&self.source, &self.model.import_map, &board).unwrap();
        let entries: Vec<_> = plan
            .identities
            .iter()
            .map(|i| ImportMapEntry {
                import_key: i.import_key.clone(),
                object_id: i.object_id,
                source_shard_id: self.model.objects[&self.board].source_shard_id,
                status: ImportMapEntryStatus::Active,
                source_tool: "kicad".into(),
                source_path: self.source.display().to_string(),
                source_object_ref: format!("board-graphic:{}", i.source_uuid),
                source_hash: crate::import::ids_sidecar::compute_source_hash_file(&self.source)
                    .unwrap(),
            })
            .collect();
        let operation = super::super::import_map_shard_operation(
            kicad_board_import_map_relative_path(&self.source),
            entries.clone(),
        )
        .unwrap();
        let changed = entries
            .iter()
            .any(|e| self.model.import_map.get(&e.import_key) != Some(e));
        let base = ImportWrite {
            prepared: changed.then(|| {
                BatchComposer::compose(&self.model, provenance())
                    .push_op(operation)
                    .finish()
                    .unwrap()
            }),
            created_object_count: 0,
        };
        build_kicad_graphic_import(&self.model, provenance(), base, self.board, &plan.graphics)
            .unwrap()
    }
    fn import(&mut self) -> usize {
        let write = self.prepared();
        if let Some(prepared) = write.prepared {
            commit_prepared(&mut self.model, &self.root, prepared).unwrap();
        }
        write.created_object_count
    }
    fn reopen(&mut self) {
        let old = self.model.model_revision.clone();
        let count = self.model.journal.len();
        self.model = ProjectResolver::new(&self.root).resolve().unwrap();
        assert_eq!(self.model.model_revision, old);
        assert_eq!(self.model.journal.len(), count);
    }
    fn board_bytes(&self) -> Vec<u8> {
        std::fs::read(self.root.join("board/board.json")).unwrap()
    }
    fn map_bytes(&self) -> Vec<u8> {
        std::fs::read(
            self.root
                .join(kicad_board_import_map_relative_path(&self.source)),
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
#[test]
fn graphic_import_and_source_refresh_restore_board_and_map_preimages_through_reopened_history() {
    let mut f = Fixture::new();
    f.source(2);
    let adopted = f.board_bytes();
    assert_eq!(f.import(), 2);
    f.reopen();
    let first_board = f.board_bytes();
    let first_map = f.map_bytes();
    let first_bindings = f.model.import_map.clone();
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.reopen();
    assert_eq!(f.board_bytes(), adopted);
    assert!(
        !f.root
            .join(kicad_board_import_map_relative_path(&f.source))
            .exists()
    );
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.reopen();
    assert_eq!(f.board_bytes(), first_board);
    assert_eq!(f.map_bytes(), first_map);
    let journal_count = f.model.journal.len();
    assert_eq!(f.import(), 0);
    assert_eq!(
        f.model.journal.len(),
        journal_count,
        "unchanged reimport is read-only"
    );
    f.source(4);
    assert_eq!(f.import(), 0);
    f.reopen();
    let refreshed = f.board_bytes();
    let refreshed_map = f.map_bytes();
    assert_ne!(refreshed, first_board);
    assert_ne!(refreshed_map, first_map);
    assert_eq!(
        f.model.import_map.keys().collect::<Vec<_>>(),
        first_bindings.keys().collect::<Vec<_>>()
    );
    for (key, old) in &first_bindings {
        assert_eq!(f.model.import_map[key].object_id, old.object_id);
    }
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.reopen();
    assert_eq!(f.board_bytes(), first_board);
    assert_eq!(f.map_bytes(), first_map);
    assert_eq!(f.model.import_map, first_bindings);
    f.model
        .commit_journal_redo(&f.root, provenance().into())
        .unwrap();
    f.reopen();
    assert_eq!(f.board_bytes(), refreshed);
    assert_eq!(f.map_bytes(), refreshed_map);
}
#[test]
fn reimport_refuses_native_graphic_edits_before_any_writer_or_artifact_change() {
    let mut f = Fixture::new();
    f.source(2);
    f.import();
    let key = board_graphic_import_key(&f.source, Uuid::from_u128(1));
    let id = f.model.import_map[&key].object_id;
    let root = f
        .model
        .materialized_source_shard_value(SourceShardKind::BoardRoot)
        .unwrap();
    let mut graphic: BoardGraphic =
        serde_json::from_value(root["graphics"][id.to_string()].clone()).unwrap();
    let GraphicGeometry::Line { ref mut to, .. } = graphic.geometry else {
        unreachable!()
    };
    to.y = 1_000_000;
    let write = build_set_board_graphic(&f.model, provenance(), f.board, &graphic).unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    let before = f.board_bytes();
    let map = f.map_bytes();
    let journal = f.model.journal.len();
    f.source(4);
    let board = f.imported_board();
    let plan = native_graphic_sources(&f.source, &f.model.import_map, &board).unwrap();
    let result = build_kicad_graphic_import(
        &f.model,
        provenance(),
        ImportWrite {
            prepared: None,
            created_object_count: 0,
        },
        f.board,
        &plan.graphics,
    );
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("conflicts with native edits")
    );
    assert_eq!(f.board_bytes(), before);
    assert_eq!(f.map_bytes(), map);
    assert_eq!(f.model.journal.len(), journal);
}

#[test]
fn missing_external_source_retains_native_graphic_and_binding_through_delete_undo() {
    let mut f = Fixture::new();
    f.source(2);
    f.import();
    f.reopen();
    let id = f.model.import_map[&board_graphic_import_key(&f.source, Uuid::from_u128(1))].object_id;
    let mut entries: Vec<_> = f.model.import_map.values().cloned().collect();
    for entry in &mut entries {
        entry.status = ImportMapEntryStatus::MissingInSource;
    }
    let operation = super::super::import_map_shard_operation(
        kicad_board_import_map_relative_path(&f.source),
        entries,
    )
    .unwrap();
    let base = ImportWrite {
        prepared: Some(
            BatchComposer::compose(&f.model, provenance())
                .push_op(operation)
                .finish()
                .unwrap(),
        ),
        created_object_count: 0,
    };
    let write = build_kicad_graphic_import(&f.model, provenance(), base, f.board, &[])
        .unwrap()
        .prepared
        .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.reopen();
    let before = f.board_bytes();
    let map = f.map_bytes();
    assert!(f.model.objects.contains_key(&id));
    let write = crate::api::native_write::board_annotations::graphics::build_delete_board_graphic(
        &f.model,
        provenance(),
        f.board,
        id,
    )
    .unwrap();
    commit_prepared(&mut f.model, &f.root, write).unwrap();
    f.reopen();
    f.model
        .commit_journal_undo(&f.root, provenance().into())
        .unwrap();
    f.reopen();
    assert_eq!(f.board_bytes(), before);
    assert_eq!(f.map_bytes(), map);
    assert_eq!(
        f.model.import_map[&board_graphic_import_key(&f.source, Uuid::from_u128(1))].status,
        ImportMapEntryStatus::MissingInSource
    );
}
