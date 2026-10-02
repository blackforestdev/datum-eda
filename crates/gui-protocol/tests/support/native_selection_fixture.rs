//! Owned on-disk fixtures through actual canonical native producers.
#![allow(dead_code)]
use eda_engine::{
    api::native_write::{self, BatchComposer, WriteProvenance, genesis},
    board::{Net, Track, Zone},
    ir::geometry::{Point, Polygon},
    substrate::{CommitSource, DesignModel, Operation, ProjectResolver, ZoneFill, ZoneFillState},
};
use std::path::PathBuf;

pub type Id = eda_engine::substrate::ObjectId;
pub fn id(n: u128) -> Id {
    Id::from_u128(n)
}
pub fn provenance() -> WriteProvenance {
    WriteProvenance::new(
        "focused-selection-proof",
        CommitSource::Test,
        "S5A native adapter proof",
    )
}
pub struct Fixture {
    pub root: PathBuf,
    pub model: DesignModel,
    pub sheet: Id,
    pub schematic: Id,
    pub board: Id,
    pub net: Id,
}
impl Fixture {
    pub fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "s5a-native-adapter-{}-{}",
            std::process::id(),
            Id::new_v4()
        ));
        let report = genesis::bootstrap_native_project(
            &root,
            genesis::GenesisSpec {
                project_name: name.into(),
                existing_ids: Some(genesis::GenesisRootIds {
                    project: id(900),
                    schematic: id(901),
                    board: id(902),
                    rules: Some(id(903)),
                }),
            },
        )
        .unwrap();
        let model = ProjectResolver::new(&root).resolve().unwrap();
        let mut result = Self {
            root,
            model,
            sheet: id(904),
            schematic: report.schematic_uuid,
            board: report.board_uuid,
            net: id(905),
        };
        let source = serde_json::json!({"schema_version":1,"uuid":result.sheet,"name":"Main","symbols":{},"wires":{},"labels":{},"ports":{},"junctions":{},"buses":{},"bus_entries":{},"noconnects":{},"texts":{},"drawings":{}});
        let write = native_write::schematic_sheets::build_create_schematic_sheet(
            &result.model,
            provenance(),
            result.schematic,
            result.sheet,
            "sheets/main.json",
            source,
        )
        .unwrap();
        native_write::commit_prepared(&mut result.model, &result.root, write).unwrap();
        let net = Net::new(result.net, "N", Id::nil());
        let write =
            native_write::board_routing::build_place_board_net(&result.model, provenance(), &net)
                .unwrap();
        native_write::commit_prepared(&mut result.model, &result.root, write).unwrap();
        result
    }
    pub fn commit(&mut self, operations: impl IntoIterator<Item = Operation>) {
        let write = BatchComposer::compose(&self.model, provenance())
            .push_ops(operations)
            .finish()
            .unwrap();
        native_write::commit_prepared(&mut self.model, &self.root, write).unwrap();
    }
    pub fn line(&mut self, source: Id, from: Point, to: Point) -> Track {
        let track = Track::straight(source, self.net, from, to, 2, 1);
        self.commit([Operation::CreateBoardTrack {
            track_id: source,
            track: serde_json::to_value(&track).unwrap(),
        }]);
        track
    }
    pub fn zone(&mut self, source: Id) -> Zone {
        let zone = Zone {
            uuid: source,
            net: self.net,
            polygon: rectangle(-10, -10, 100, 30),
            layer: 1,
            priority: 0,
            thermal_relief: false,
            thermal_gap: 0,
            thermal_spoke_width: 0,
        };
        let write =
            native_write::board_routing::build_place_board_zone(&self.model, provenance(), &zone)
                .unwrap();
        native_write::commit_prepared(&mut self.model, &self.root, write).unwrap();
        zone
    }
    pub fn fill(&mut self, zone: &Zone, state: ZoneFillState, islands: Vec<Polygon>) {
        let fill = ZoneFill {
            schema_version: 2,
            zone_id: zone.uuid,
            state,
            source_zone_revision: self.model.objects[&zone.uuid].object_revision,
            model_revision: self.model.model_revision.clone(),
            islands,
            provenance: Some("native certified test producer result".into()),
        };
        let write =
            native_write::board_routing::build_set_zone_fills(&self.model, provenance(), &[fill])
                .unwrap();
        native_write::commit_prepared(&mut self.model, &self.root, write).unwrap();
    }
    pub fn undo(&mut self) {
        self.model
            .commit_journal_undo(&self.root, provenance().into())
            .unwrap();
    }
    pub fn reopen(&self) -> DesignModel {
        ProjectResolver::new(&self.root).resolve().unwrap()
    }
    pub fn source_bytes(&self) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files: Vec<_> = self
            .model
            .source_shards
            .iter()
            .map(|s| (s.path.clone(), std::fs::read(&s.path).unwrap()))
            .collect();
        let journal = eda_engine::substrate::transaction_journal_path(&self.root);
        files.push((journal.clone(), std::fs::read(journal).unwrap()));
        files
    }
    pub fn assert_bytes(files: &[(PathBuf, Vec<u8>)]) {
        for (path, bytes) in files {
            assert_eq!(&std::fs::read(path).unwrap(), bytes);
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
pub fn rectangle(x: i64, y: i64, w: i64, h: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + w, y),
        Point::new(x + w, y + h),
        Point::new(x, y + h),
    ])
}
