//! Complete Symbol compound carried unchanged from acquisition to one native edit.
use super::*;
use crate::{AuthoredSelectionClass, AuthoredSelectionIdentity, SelectionSubject};
use eda_engine::api::native_write::schematic_symbol_move::{
    SymbolMoveSetBasis, SymbolMoveSetRequest,
};
use eda_engine::ir::geometry::Point;

pub struct SymbolGroupPreview {
    pub(super) base: BoardReviewSceneV1,
    pub(super) basis: SymbolMoveSetBasis,
    pub(super) authoring: Box<SchematicAuthoring>,
    pub reference: PointNm,
    pub(super) members: Vec<AuthoredSelectionIdentity>,
    pub(super) rotation: i32,
    pub(super) mirrored: bool,
    pub(super) root: PathBuf,
}

/// A compound is indivisible. Inspector focus and per-type scopes never filter it.
pub fn symbol_members(subject: &SelectionSubject) -> Result<Vec<AuthoredSelectionIdentity>> {
    let SelectionSubject::Compound(compound) = subject else {
        anyhow::bail!("Select a complete Symbol compound before moving");
    };
    let members: Vec<_> = compound.members.iter().cloned().collect();
    anyhow::ensure!(!members.is_empty(), "Select Symbols before moving");
    let path = &members[0].instance_path;
    anyhow::ensure!(
        members
            .iter()
            .all(|id| id.class == AuthoredSelectionClass::Symbol && &id.instance_path == path),
        "Move requires only Symbols from one sheet occurrence; the complete selection is unchanged"
    );
    Ok(members)
}

pub fn subject_contains(subject: &SelectionSubject, id: &AuthoredSelectionIdentity) -> bool {
    match subject {
        SelectionSubject::Object(target) => target == id,
        SelectionSubject::Compound(group) => group.members.contains(id),
        _ => false,
    }
}

impl SymbolGroupPreview {
    pub fn begin(
        base: &BoardReviewSceneV1,
        authoring: &SchematicAuthoring,
        subject: &SelectionSubject,
        basis: SymbolMoveSetBasis,
        cursor: PointNm,
        snap: AuthoringSnapState,
        root: PathBuf,
    ) -> Result<Self> {
        let members = symbol_members(subject)?;
        let ids: std::collections::BTreeSet<_> = members.iter().map(|id| id.id).collect();
        anyhow::ensure!(
            basis
                .symbols
                .iter()
                .map(|s| s.uuid)
                .collect::<std::collections::BTreeSet<_>>()
                == ids,
            "Move basis must contain the complete selected Symbol set"
        );
        anyhow::ensure!(
            authoring.sheet == Some(basis.sheet_id),
            "Select Symbols on the current sheet"
        );
        anyhow::ensure!(
            members
                .iter()
                .all(|id| authoring.move_paths.get(&id.id) == Some(&id.instance_path)),
            "Symbol occurrence changed; select the complete set again"
        );
        Ok(Self {
            base: base.clone(),
            basis,
            authoring: Box::new(authoring.clone()),
            reference: checked_snap_authoring_point(cursor, snap)?,
            members,
            rotation: 0,
            mirrored: false,
            root,
        })
    }
    pub fn request(&self, at: PointNm) -> SymbolMoveSetRequest {
        SymbolMoveSetRequest {
            symbols: self.members.iter().map(|id| id.id).collect(),
            instance_path: self.members[0].instance_path.clone(),
            reference: Point::new(self.reference.x, self.reference.y),
            x_nm: at.x,
            y_nm: at.y,
            rotation_deg: self.rotation,
            mirrored: self.mirrored,
        }
    }
    pub fn matches_subject(&self, subject: &SelectionSubject) -> bool {
        symbol_members(subject).is_ok_and(|members| members == self.members)
    }
    pub fn transform(&mut self, transform: PlacementTransform, at: PointNm) -> Result<bool> {
        let (rotation, mirrored) = transform.apply(self.rotation, self.mirrored);
        self.basis.destination_with_transform(
            Point::new(self.reference.x, self.reference.y),
            Point::new(at.x, at.y),
            rotation,
            mirrored,
        )?;
        self.rotation = rotation;
        self.mirrored = mirrored;
        Ok(true)
    }
    pub fn scene_at(&self, at: PointNm) -> Result<BoardReviewSceneV1> {
        let moved = self.basis.destination_with_transform(
            Point::new(self.reference.x, self.reference.y),
            Point::new(at.x, at.y),
            self.rotation,
            self.mirrored,
        )?;
        let mut source = self
            .authoring
            .source
            .clone()
            .context("Native schematic source unavailable")?;
        let sheet = source
            .sheets
            .get_mut(&self.basis.sheet_id)
            .context("Native sheet changed")?;
        for symbol in moved.symbols {
            sheet.symbols.insert(symbol.uuid, symbol);
        }
        for wire in moved.wires {
            sheet.wires.insert(wire.uuid, wire);
        }
        self.authoring
            .project_source(&source, self.basis.sheet_id, &self.base, &self.root)
    }
}
