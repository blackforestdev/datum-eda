//! One native Move session, configured by its authored target surface.
use super::*;
use crate::native_edit_client::NativeEditAction;
use crate::schematic_authoring::{PlacementTransform, SchematicAuthoring};
use eda_engine::api::native_write::{
    schematic_label_move::LabelMoveBasis, schematic_symbol_move::SymbolMoveBasis,
};
use std::path::PathBuf;
use uuid::Uuid;

pub enum NativeMovePreview {
    Footprint(FootprintMovePreview),
    Label {
        base: BoardReviewSceneV1,
        basis: LabelMoveBasis,
        authoring: Box<SchematicAuthoring>,
        cursor_anchor: PointNm,
        instance_path: Vec<Uuid>,
        root: PathBuf,
    },
    Symbol {
        base: BoardReviewSceneV1,
        basis: SymbolMoveBasis,
        authoring: Box<SchematicAuthoring>,
        cursor_anchor: PointNm,
        rotation_deg: Option<i32>,
        mirrored: Option<bool>,
        instance_path: Vec<Uuid>,
        root: PathBuf,
    },
}
impl NativeMovePreview {
    pub fn symbol(
        base: &BoardReviewSceneV1,
        authoring: &SchematicAuthoring,
        id: &crate::AuthoredSelectionIdentity,
        cursor: PointNm,
        snap: AuthoringSnapState,
        root: PathBuf,
    ) -> Result<Self> {
        let basis = authoring
            .move_bases
            .get(&id.id)
            .context("Symbol Move source unavailable")?
            .as_ref()
            .map_err(|reason| anyhow::anyhow!("{reason}"))?
            .clone();
        anyhow::ensure!(
            authoring.move_paths.get(&id.id) == Some(&id.instance_path),
            "Symbol occurrence changed; select it again"
        );
        anyhow::ensure!(
            authoring.sheet == Some(basis.sheet_id),
            "Select a Symbol on the current native sheet"
        );
        Ok(Self::Symbol {
            base: base.clone(),
            basis,
            authoring: Box::new(authoring.clone()),
            cursor_anchor: checked_snap_authoring_point(cursor, snap)?,
            instance_path: id.instance_path.clone(),
            root,
            rotation_deg: None,
            mirrored: None,
        })
    }
    pub fn label(
        base: &BoardReviewSceneV1,
        authoring: &SchematicAuthoring,
        id: &crate::AuthoredSelectionIdentity,
        cursor: PointNm,
        snap: AuthoringSnapState,
        root: PathBuf,
    ) -> Result<Self> {
        let basis = authoring
            .label_move_bases
            .get(&id.id)
            .context("Label Move source unavailable")?
            .as_ref()
            .map_err(|reason| anyhow::anyhow!("{reason}"))?
            .clone();
        anyhow::ensure!(
            authoring.move_paths.get(&id.id) == Some(&id.instance_path),
            "Label occurrence changed; select it again"
        );
        anyhow::ensure!(
            authoring.sheet == Some(basis.sheet_id),
            "Select a Label on the current native sheet"
        );
        Ok(Self::Label {
            base: base.clone(),
            basis,
            authoring: Box::new(authoring.clone()),
            cursor_anchor: checked_snap_authoring_point(cursor, snap)?,
            instance_path: id.instance_path.clone(),
            root,
        })
    }
    pub fn base(&self) -> &BoardReviewSceneV1 {
        match self {
            Self::Footprint(p) => &p.base,
            Self::Symbol { base, .. } | Self::Label { base, .. } => base,
        }
    }
    pub fn footprint(&self) -> Option<&FootprintMovePreview> {
        match self {
            Self::Footprint(p) => Some(p),
            _ => None,
        }
    }
    pub fn is_schematic(&self) -> bool {
        matches!(self, Self::Symbol { .. } | Self::Label { .. })
    }
    pub fn instance_path(&self) -> Vec<Uuid> {
        match self {
            Self::Symbol { instance_path, .. } | Self::Label { instance_path, .. } => {
                instance_path.clone()
            }
            _ => vec![],
        }
    }
    pub fn identity(&self) -> Result<crate::AuthoredSelectionIdentity> {
        Ok(match self {
            Self::Footprint(p) => crate::AuthoredSelectionIdentity {
                class: crate::AuthoredSelectionClass::Footprint,
                id: p.component.component_uuid.parse()?,
                instance_path: vec![],
            },
            Self::Label {
                basis,
                instance_path,
                ..
            } => crate::AuthoredSelectionIdentity {
                class: crate::AuthoredSelectionClass::Label,
                id: basis.label.uuid,
                instance_path: instance_path.clone(),
            },
            Self::Symbol {
                basis,
                instance_path,
                ..
            } => crate::AuthoredSelectionIdentity {
                class: crate::AuthoredSelectionClass::Symbol,
                id: basis.symbol.uuid,
                instance_path: instance_path.clone(),
            },
        })
    }
    pub fn action(&self, at: PointNm) -> Result<NativeEditAction> {
        Ok(match self {
            Self::Footprint(p) => NativeEditAction::Move {
                component: p.component.component_uuid.parse()?,
                x_nm: at.x,
                y_nm: at.y,
            },
            Self::Label { basis, .. } => NativeEditAction::MoveLabel {
                label: basis.label.uuid,
                x_nm: at.x,
                y_nm: at.y,
            },
            Self::Symbol {
                basis,
                rotation_deg,
                mirrored,
                ..
            } => NativeEditAction::MoveSymbol {
                symbol: basis.symbol.uuid,
                x_nm: at.x,
                y_nm: at.y,
                rotation_deg: *rotation_deg,
                mirrored: *mirrored,
            },
        })
    }
    /// Validate the complete connected geometry before changing the desired pose.
    /// Repeated keys always transform the immutable original, never rounded previews.
    pub fn transform_symbol(&mut self, transform: PlacementTransform, at: PointNm) -> Result<bool> {
        let Self::Symbol {
            basis,
            rotation_deg,
            mirrored,
            ..
        } = self
        else {
            return Ok(false);
        };
        let (rotation, mirror) = transform.apply(
            rotation_deg.unwrap_or(basis.symbol.rotation),
            mirrored.unwrap_or(basis.symbol.mirrored),
        );
        basis.destination_with_pose(
            eda_engine::ir::geometry::Point { x: at.x, y: at.y },
            Some(rotation),
            Some(mirror),
        )?;
        *rotation_deg = Some(rotation);
        *mirrored = Some(mirror);
        Ok(true)
    }
    pub fn destination(&self, cursor: PointNm, snap: AuthoringSnapState) -> Result<PointNm> {
        match self {
            Self::Footprint(p) => p.destination(cursor, snap),
            Self::Symbol {
                basis,
                cursor_anchor,
                ..
            } => translated_destination(basis.symbol.position, *cursor_anchor, cursor, snap),
            Self::Label {
                basis,
                cursor_anchor,
                ..
            } => translated_destination(basis.label.position, *cursor_anchor, cursor, snap),
        }
    }
    pub fn scene_at(&self, at: PointNm) -> Result<BoardReviewSceneV1> {
        match self {
            Self::Footprint(p) => p.scene_at(at),
            Self::Label {
                base,
                basis,
                authoring,
                root,
                ..
            } => {
                let moved =
                    basis.destination_edit(eda_engine::ir::geometry::Point { x: at.x, y: at.y })?;
                let mut source = authoring
                    .source
                    .clone()
                    .context("Native schematic source unavailable")?;
                let sheet = source
                    .sheets
                    .get_mut(&basis.sheet_id)
                    .context("Native sheet changed")?;
                sheet.labels.insert(moved.label.uuid, moved.label);
                for wire in moved.wires {
                    sheet.wires.insert(wire.uuid, wire);
                }
                authoring.project_source(&source, basis.sheet_id, base, root)
            }
            Self::Symbol {
                base,
                basis,
                authoring,
                root,
                rotation_deg,
                mirrored,
                ..
            } => {
                let moved = basis.destination_with_pose(
                    eda_engine::ir::geometry::Point { x: at.x, y: at.y },
                    *rotation_deg,
                    *mirrored,
                )?;
                let mut source = authoring
                    .source
                    .clone()
                    .context("Native schematic source unavailable")?;
                let sheet = source
                    .sheets
                    .get_mut(&basis.sheet_id)
                    .context("Native sheet changed")?;
                sheet.symbols.insert(moved.symbol.uuid, moved.symbol);
                for wire in moved.wires {
                    sheet.wires.insert(wire.uuid, wire);
                }
                authoring.project_source(&source, basis.sheet_id, base, root)
            }
        }
    }
}

fn translated_destination(
    position: eda_engine::ir::geometry::Point,
    anchor: PointNm,
    cursor: PointNm,
    snap: AuthoringSnapState,
) -> Result<PointNm> {
    let at = checked_snap_authoring_point(cursor, snap)?;
    let translate = |original: i64, current: i64, anchor: i64| {
        i64::try_from(i128::from(original) + i128::from(current) - i128::from(anchor))
            .context("Move exceeds integer coordinates")
    };
    Ok(PointNm {
        x: translate(position.x, at.x, anchor.x)?,
        y: translate(position.y, at.y, anchor.y)?,
    })
}
