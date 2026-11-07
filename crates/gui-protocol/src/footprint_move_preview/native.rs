//! One native Move session, configured by its authored target surface.
use super::*;
use crate::native_edit_client::NativeEditAction;
use crate::schematic_authoring::{PlacementTransform, SchematicAuthoring};
use eda_engine::api::native_write::{
    schematic_junction_move::JunctionMoveBasis,
    schematic_label_move::LabelMoveBasis,
    schematic_symbol_move::SymbolMoveBasis,
    schematic_wire_edit::{WireEditBasis, WireHandle},
    schematic_wire_edit_adapter::WireEditParams,
};
use std::path::PathBuf;
use uuid::Uuid;

#[path = "group.rs"]
pub mod group;
use group::SymbolGroupPreview;
#[path = "text.rs"]
mod text;
use text::TextMovePreview;
#[path = "wiring.rs"]
pub mod wiring;
use wiring::WiringGroupPreview;

pub enum NativeMovePreview {
    Text(TextMovePreview),
    Wiring(WiringGroupPreview),
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

/// Immutable source/configuration of one qualified connectivity edit.
#[derive(Clone)]
pub enum ConnectivityMoveTarget {
    Wiring {
        basis: WireEditBasis,
        edit: WireEditParams,
        anchor: eda_engine::ir::geometry::Point,
        identity: crate::AuthoredSelectionIdentity,
        locked_members: std::collections::BTreeSet<Uuid>,
    },
    Junction(JunctionMoveBasis),
    Port(eda_engine::schematic::HierarchicalPort),
}
impl ConnectivityMoveTarget {
    fn id(&self) -> Uuid {
        match self {
            Self::Wiring { identity, .. } => identity.id,
            Self::Junction(basis) => basis.junction.uuid,
            Self::Port(port) => port.uuid,
        }
    }
    fn class(&self) -> crate::AuthoredSelectionClass {
        match self {
            Self::Wiring { identity, .. } => identity.class,
            Self::Junction(_) => crate::AuthoredSelectionClass::Junction,
            Self::Port(_) => crate::AuthoredSelectionClass::Port,
        }
    }
    /// The anchor the engine verb's absolute destination applies to.
    fn anchor(&self) -> eda_engine::ir::geometry::Point {
        match self {
            Self::Wiring { anchor, .. } => *anchor,
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
        Self::connectivity_with_wire_edit(base, authoring, id, cursor, snap, root, None)
    }
    pub fn connectivity_with_wire_edit(
        base: &BoardReviewSceneV1,
        authoring: &SchematicAuthoring,
        id: &crate::AuthoredSelectionIdentity,
        cursor: PointNm,
        snap: AuthoringSnapState,
        root: PathBuf,
        wire_edit: Option<WireEditParams>,
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
            crate::AuthoredSelectionClass::Wire => {
                let edit = wire_edit
                    .clone()
                    .context("Choose a wire endpoint, vertex or segment handle")?;
                let basis = WireEditBasis::capture(sheet, id.id, edit.edit_seed)?;
                let anchor = basis.anchor(&edit.handle)?;
                ConnectivityMoveTarget::Wiring {
                    basis,
                    edit,
                    anchor,
                    identity: id.clone(),
                    locked_members: Default::default(),
                }
            }
            crate::AuthoredSelectionClass::Junction => {
                if let Some(edit) = wire_edit.clone() {
                    let marker = sheet
                        .junctions
                        .get(&id.id)
                        .context("Junction Move source unavailable")?;
                    anyhow::ensure!(
                        matches!(edit.handle, WireHandle::Vertex { point } if point == marker.position),
                        "Choose the captured junction vertex"
                    );
                    let basis = WireEditBasis::capture_junction(sheet, id.id, edit.edit_seed)?;
                    let anchor = basis.anchor(&edit.handle)?;
                    ConnectivityMoveTarget::Wiring {
                        basis,
                        edit,
                        anchor,
                        identity: id.clone(),
                        locked_members: Default::default(),
                    }
                } else {
                    ConnectivityMoveTarget::Junction(JunctionMoveBasis::capture(sheet, id.id)?)
                }
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
        anyhow::ensure!(
            wire_edit.is_none() || matches!(target, ConnectivityMoveTarget::Wiring { .. }),
            "Wire editing requires a Wire source"
        );
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
            Self::Wiring(p) => &p.base,
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
                | Self::Wiring(_)
                | Self::Symbols(_)
                | Self::Symbol { .. }
                | Self::Label { .. }
                | Self::Connectivity { .. }
        )
    }
    /// Locks are captured from the same immutable source as the gesture.
    /// This is view admission only; canonical writes validate raw locks again.
    pub fn set_wire_locked_members(&mut self, locked: std::collections::BTreeSet<Uuid>) {
        if let Self::Connectivity {
            target: ConnectivityMoveTarget::Wiring { locked_members, .. },
            ..
        } = self
        {
            *locked_members = locked;
        }
    }
    pub fn uses_electrical_pin_snap(&self) -> bool {
        if let Self::Wiring(group) = self {
            return group.uses_electrical_pin_snap();
        }
        matches!(self, Self::Connectivity { target: ConnectivityMoveTarget::Wiring { edit, .. }, .. }
            if matches!(edit.handle, WireHandle::Endpoint { .. }))
    }
    pub fn instance_path(&self) -> Vec<Uuid> {
        match self {
            Self::Text(p) => p.instance_path.clone(),
            Self::Wiring(p) => p.instance_path(),
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
            Self::Wiring(group) => group.singleton_identity()?,
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
            Self::Wiring(p) => p.matches_subject(subject),
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
            Self::Wiring(_) => NativeEditAction::MoveDesign {
                x_nm: at.x,
                y_nm: at.y,
            },
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
                wire_edit: match target {
                    ConnectivityMoveTarget::Wiring {
                        basis,
                        edit,
                        locked_members,
                        ..
                    } => {
                        let plan =
                            edit.plan(basis, eda_engine::ir::geometry::Point { x: at.x, y: at.y })?;
                        validate_wire_locks(&plan, locked_members)?;
                        Some(*edit)
                    }
                    _ => None,
                },
            },
        })
    }
    /// Validate the complete connected geometry before changing the desired pose.
    /// Repeated keys always transform the immutable original, never rounded previews.
    pub fn transform_symbol(&mut self, transform: PlacementTransform, at: PointNm) -> Result<bool> {
        if matches!(
            self,
            Self::Connectivity {
                target: ConnectivityMoveTarget::Wiring { .. },
                ..
            }
        ) {
            return self.transform_selected_wiring(transform, at);
        }
        if let Self::Wiring(p) = self {
            return p.transform(transform, at);
        }
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
            Self::Wiring(group) => group.destination(cursor, snap),
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
                target: ConnectivityMoveTarget::Wiring { edit, .. },
                ..
            } if matches!(
                edit.handle,
                WireHandle::Endpoint { .. } | WireHandle::InsertVertex { .. }
            ) =>
            {
                checked_snap_authoring_point(cursor, snap)
            }
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
            Self::Wiring(p) => p.scene_at(at),
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
                    ConnectivityMoveTarget::Wiring {
                        basis,
                        edit,
                        locked_members,
                        ..
                    } => {
                        let plan =
                            edit.plan(basis, eda_engine::ir::geometry::Point { x: at.x, y: at.y })?;
                        validate_wire_locks(&plan, locked_members)?;
                        *sheet = plan.apply_to_sheet(sheet);
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

fn validate_wire_locks(
    plan: &eda_engine::api::native_write::schematic_wire_edit::WireEditPlan,
    locked: &std::collections::BTreeSet<Uuid>,
) -> Result<()> {
    let affected = plan
        .wires
        .keys()
        .chain(plan.removed_wire_ids.iter())
        .chain(plan.new_wire_origins.values())
        .chain(plan.junctions.keys())
        .chain(plan.removed_junction_ids.iter());
    anyhow::ensure!(
        !affected.into_iter().any(|id| locked.contains(id)),
        "Wire edit includes a locked wire or junction"
    );
    Ok(())
}
