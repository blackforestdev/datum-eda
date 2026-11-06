//! One native Move session, configured by its authored target surface.
use super::*;
use crate::native_edit_client::NativeEditAction;
use crate::schematic_authoring::{PlacementTransform, SchematicAuthoring};
use eda_engine::api::native_write::{
    schematic_junction_move::JunctionMoveBasis, schematic_label_move::LabelMoveBasis,
    schematic_symbol_move::SymbolMoveBasis,
};
use std::path::PathBuf;
use uuid::Uuid;

#[path = "group.rs"]
pub mod group;
use group::SymbolGroupPreview;
#[path = "text.rs"]
mod text;
use text::TextMovePreview;

pub enum NativeMovePreview {
    Text(TextMovePreview),
    Symbols(SymbolGroupPreview),
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
    Connectivity {
        base: BoardReviewSceneV1,
        target: ConnectivityMoveTarget,
        sheet_id: Uuid,
        authoring: Box<SchematicAuthoring>,
        cursor_anchor: PointNm,
        instance_path: Vec<Uuid>,
        root: PathBuf,
    },
}

/// Immutable source copy of one rigid connectivity move subject.
#[derive(Clone)]
pub enum ConnectivityMoveTarget {
    Wire(eda_engine::schematic::SchematicWire),
    Junction(JunctionMoveBasis),
    Port(eda_engine::schematic::HierarchicalPort),
}
impl ConnectivityMoveTarget {
    fn id(&self) -> Uuid {
        match self {
            Self::Wire(wire) => wire.uuid,
            Self::Junction(basis) => basis.junction.uuid,
            Self::Port(port) => port.uuid,
        }
    }
    fn class(&self) -> crate::AuthoredSelectionClass {
        match self {
            Self::Wire(_) => crate::AuthoredSelectionClass::Wire,
            Self::Junction(_) => crate::AuthoredSelectionClass::Junction,
            Self::Port(_) => crate::AuthoredSelectionClass::Port,
        }
    }
    /// The anchor the engine verb's absolute destination applies to.
    fn anchor(&self) -> eda_engine::ir::geometry::Point {
        match self {
            Self::Wire(wire) => wire.from,
            Self::Junction(basis) => basis.junction.position,
            Self::Port(port) => port.position,
        }
    }
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
            base.schematic_document
                .as_ref()
                .is_some_and(|document| document.source_sheet == basis.sheet_id
                    && document.instance_path == id.instance_path),
            "Select a Symbol in the captured schematic occurrence"
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
            base.schematic_document
                .as_ref()
                .is_some_and(|document| document.source_sheet == basis.sheet_id
                    && document.instance_path == id.instance_path),
            "Select a Label in the captured schematic occurrence"
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
    /// One rigid Wire/Junction move subject acquired from the focused
    /// qualified occurrence; the engine `datum.schematic.move` adapter
    /// resolves the kind again from its own source.
    pub fn connectivity(
        base: &BoardReviewSceneV1,
        authoring: &SchematicAuthoring,
        id: &crate::AuthoredSelectionIdentity,
        cursor: PointNm,
        snap: AuthoringSnapState,
        root: PathBuf,
    ) -> Result<Self> {
        let document = base
            .schematic_document
            .as_ref()
            .context("Move requires its captured schematic occurrence")?;
        anyhow::ensure!(
            document.instance_path == id.instance_path,
            "Select a Wire or Junction in the captured schematic occurrence"
        );
        let sheet_id = document.source_sheet;
        let sheet = authoring
            .source
            .as_ref()
            .and_then(|source| source.sheets.get(&sheet_id))
            .context("Native schematic source unavailable")?;
        let target = match id.class {
            crate::AuthoredSelectionClass::Wire => ConnectivityMoveTarget::Wire(
                sheet
                    .wires
                    .get(&id.id)
                    .context("Wire Move source unavailable")?
                    .clone(),
            ),
            crate::AuthoredSelectionClass::Junction => {
                ConnectivityMoveTarget::Junction(JunctionMoveBasis::capture(sheet, id.id)?)
            }
            crate::AuthoredSelectionClass::Port => ConnectivityMoveTarget::Port(
                sheet
                    .ports
                    .get(&id.id)
                    .context("Port Move source unavailable")?
                    .clone(),
            ),
            _ => anyhow::bail!("Move supports a selected Wire, Junction or Port here"),
        };
        if let ConnectivityMoveTarget::Wire(wire) = &target {
            anyhow::ensure!(
                !sheet
                    .bus_entries
                    .values()
                    .any(|entry| entry.wire == Some(wire.uuid)),
                "Wire is referenced by a bus entry; remove the entry first"
            );
        }
        Ok(Self::Connectivity {
            base: base.clone(),
            target,
            sheet_id,
            authoring: Box::new(authoring.clone()),
            cursor_anchor: checked_snap_authoring_point(cursor, snap)?,
            instance_path: id.instance_path.clone(),
            root,
        })
    }
    pub fn base(&self) -> &BoardReviewSceneV1 {
        match self {
            Self::Footprint(p) => &p.base,
            Self::Text(p) => &p.base,
            Self::Symbols(p) => &p.base,
            Self::Symbol { base, .. }
            | Self::Label { base, .. }
            | Self::Connectivity { base, .. } => base,
        }
    }
    pub fn footprint(&self) -> Option<&FootprintMovePreview> {
        match self {
            Self::Footprint(p) => Some(p),
            _ => None,
        }
    }
    pub fn is_schematic(&self) -> bool {
        matches!(
            self,
            Self::Text(_)
                | Self::Symbols(_)
                | Self::Symbol { .. }
                | Self::Label { .. }
                | Self::Connectivity { .. }
        )
    }
    pub fn instance_path(&self) -> Vec<Uuid> {
        match self {
            Self::Text(p) => p.instance_path.clone(),
            Self::Symbols(p) => p.members[0].instance_path.clone(),
            Self::Symbol { instance_path, .. }
            | Self::Label { instance_path, .. }
            | Self::Connectivity { instance_path, .. } => instance_path.clone(),
            _ => vec![],
        }
    }
    pub fn identity(&self) -> Result<crate::AuthoredSelectionIdentity> {
        Ok(match self {
            Self::Text(p) => crate::AuthoredSelectionIdentity {
                class: crate::AuthoredSelectionClass::SchematicText,
                id: p.id(),
                instance_path: p.instance_path.clone(),
            },
            Self::Symbols(_) => anyhow::bail!("A compound has no privileged single identity"),
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
            Self::Connectivity {
                target,
                instance_path,
                ..
            } => crate::AuthoredSelectionIdentity {
                class: target.class(),
                id: target.id(),
                instance_path: instance_path.clone(),
            },
        })
    }
    pub fn matches_subject(&self, subject: &crate::SelectionSubject) -> bool {
        match self {
            Self::Symbols(p) => p.matches_subject(subject),
            _ => self
                .identity()
                .is_ok_and(|id| subject == &crate::SelectionSubject::Object(id)),
        }
    }
    pub fn group_request(
        &self,
        at: PointNm,
    ) -> Option<eda_engine::api::native_write::schematic_symbol_move::SymbolMoveSetRequest> {
        match self {
            Self::Symbols(p) => Some(p.request(at)),
            _ => None,
        }
    }
    pub fn action(&self, at: PointNm) -> Result<NativeEditAction> {
        Ok(match self {
            Self::Text(p) => NativeEditAction::MoveText {
                text: p.id(),
                sheet: p.basis.sheet_id,
                x_nm: at.x,
                y_nm: at.y,
                rotation_deg: p.rotation,
            },
            Self::Symbols(p) => NativeEditAction::MoveSymbols {
                x_nm: at.x,
                y_nm: at.y,
                rotation_deg: p.rotation,
                mirrored: p.mirrored,
            },
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
            Self::Connectivity { target, .. } => NativeEditAction::MoveConnectivity {
                object: target.id(),
                x_nm: at.x,
                y_nm: at.y,
            },
        })
    }
    /// Validate the complete connected geometry before changing the desired pose.
    /// Repeated keys always transform the immutable original, never rounded previews.
    pub fn transform_symbol(&mut self, transform: PlacementTransform, at: PointNm) -> Result<bool> {
        if let Self::Text(p) = self {
            return p.transform(transform, at);
        }
        if let Self::Symbols(p) = self {
            return p.transform(transform, at);
        }
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
            Self::Text(p) => {
                translated_destination(p.basis.text.position, p.cursor_anchor, cursor, snap)
            }
            Self::Symbols(_) => checked_snap_authoring_point(cursor, snap),
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
            Self::Connectivity {
                target,
                cursor_anchor,
                ..
            } => translated_destination(target.anchor(), *cursor_anchor, cursor, snap),
        }
    }
    pub fn scene_at(&self, at: PointNm) -> Result<BoardReviewSceneV1> {
        match self {
            Self::Footprint(p) => p.scene_at(at),
            Self::Text(p) => p.scene_at(at),
            Self::Symbols(p) => p.scene_at(at),
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
            Self::Connectivity {
                base,
                target,
                sheet_id,
                authoring,
                root,
                ..
            } => {
                let mut source = authoring
                    .source
                    .clone()
                    .context("Native schematic source unavailable")?;
                let sheet = source
                    .sheets
                    .get_mut(sheet_id)
                    .context("Native sheet changed")?;
                let translate = |point: eda_engine::ir::geometry::Point,
                                 anchor: eda_engine::ir::geometry::Point|
                 -> Result<eda_engine::ir::geometry::Point> {
                    let shift = |p: i64, a: i64, c: i64| {
                        i64::try_from(i128::from(p) + i128::from(c) - i128::from(a))
                            .context("Move exceeds integer coordinates")
                    };
                    Ok(eda_engine::ir::geometry::Point {
                        x: shift(point.x, anchor.x, at.x)?,
                        y: shift(point.y, anchor.y, at.y)?,
                    })
                };
                match target {
                    ConnectivityMoveTarget::Wire(wire) => {
                        let moved = eda_engine::schematic::SchematicWire {
                            uuid: wire.uuid,
                            from: translate(wire.from, wire.from)?,
                            to: translate(wire.to, wire.from)?,
                        };
                        sheet.wires.insert(moved.uuid, moved);
                    }
                    ConnectivityMoveTarget::Junction(basis) => {
                        let moved = basis.destination_edit(eda_engine::ir::geometry::Point {
                            x: at.x,
                            y: at.y,
                        })?;
                        sheet.junctions.insert(moved.junction.uuid, moved.junction);
                        for wire in moved.wires {
                            sheet.wires.insert(wire.uuid, wire);
                        }
                    }
                    ConnectivityMoveTarget::Port(port) => {
                        let mut moved = port.clone();
                        moved.position = translate(port.position, port.position)?;
                        sheet.ports.insert(moved.uuid, moved);
                    }
                }
                authoring.project_source(&source, *sheet_id, base, root)
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
