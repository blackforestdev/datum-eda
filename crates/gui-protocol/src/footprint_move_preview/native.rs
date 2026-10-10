//! One native Move session, configured by its authored target surface.
use super::*;
use crate::native_edit_client::NativeEditAction;
use crate::schematic_authoring::SchematicAuthoring;
use eda_engine::api::native_write::schematic_symbol_move::SymbolMoveBasis;
use std::path::PathBuf;
use uuid::Uuid;

pub enum NativeMovePreview {
    Footprint(FootprintMovePreview),
    Symbol {
        base: BoardReviewSceneV1,
        basis: SymbolMoveBasis,
        authoring: Box<SchematicAuthoring>,
        cursor_anchor: PointNm,
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
        })
    }
    pub fn base(&self) -> &BoardReviewSceneV1 {
        match self {
            Self::Footprint(p) => &p.base,
            Self::Symbol { base, .. } => base,
        }
    }
    pub fn footprint(&self) -> Option<&FootprintMovePreview> {
        match self {
            Self::Footprint(p) => Some(p),
            _ => None,
        }
    }
    pub fn is_schematic(&self) -> bool {
        matches!(self, Self::Symbol { .. })
    }
    pub fn instance_path(&self) -> Vec<Uuid> {
        match self {
            Self::Symbol { instance_path, .. } => instance_path.clone(),
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
            Self::Symbol { basis, .. } => NativeEditAction::MoveSymbol {
                symbol: basis.symbol.uuid,
                x_nm: at.x,
                y_nm: at.y,
            },
        })
    }
    pub fn destination(&self, cursor: PointNm, snap: AuthoringSnapState) -> Result<PointNm> {
        match self {
            Self::Footprint(p) => p.destination(cursor, snap),
            Self::Symbol {
                basis,
                cursor_anchor,
                ..
            } => {
                let at = checked_snap_authoring_point(cursor, snap)?;
                let translate = |original: i64, current: i64, anchor: i64| {
                    i64::try_from(i128::from(original) + i128::from(current) - i128::from(anchor))
                        .context("Move exceeds integer coordinates")
                };
                Ok(PointNm {
                    x: translate(basis.symbol.position.x, at.x, cursor_anchor.x)?,
                    y: translate(basis.symbol.position.y, at.y, cursor_anchor.y)?,
                })
            }
        }
    }
    pub fn scene_at(&self, at: PointNm) -> Result<BoardReviewSceneV1> {
        match self {
            Self::Footprint(p) => p.scene_at(at),
            Self::Symbol {
                base,
                basis,
                authoring,
                root,
                ..
            } => {
                let moved =
                    basis.destination(eda_engine::ir::geometry::Point { x: at.x, y: at.y })?;
                let mut source = authoring
                    .source
                    .clone()
                    .context("Native schematic source unavailable")?;
                source
                    .sheets
                    .get_mut(&basis.sheet_id)
                    .context("Native sheet changed")?
                    .symbols
                    .insert(moved.uuid, moved);
                authoring.project_source(&source, basis.sheet_id, base, root)
            }
        }
    }
}
