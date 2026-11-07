//! Explicit wiring selection configures the shared native Move lifecycle.
use super::*;
use crate::schematic_authoring::{PlacementTransform, SchematicAuthoring, SchematicEditRequest};
use crate::{AuthoredSelectionClass as Class, AuthoredSelectionIdentity, SelectionSubject};
use eda_engine::api::native_write::{
    design_clipboard::{ClipboardClass, ClipboardPlacement, ClipboardTarget},
    design_clipboard_edit::{DesignEditIntent, ExistingTransformContactPolicy},
    schematic_wire_edit::WiringTransformBasis,
};
use eda_engine::ir::geometry::Point;
use std::path::PathBuf;
use uuid::Uuid;

pub struct WiringGroupPreview {
    pub(super) base: BoardReviewSceneV1,
    basis: WiringTransformBasis,
    authoring: Box<SchematicAuthoring>,
    pub reference: PointNm,
    cursor_anchor: PointNm,
    endpoint_snap: bool,
    members: Vec<AuthoredSelectionIdentity>,
    sheet: Uuid,
    rotation: i32,
    mirrored: bool,
    root: PathBuf,
}

/// No derived Run expansion, Inspector filtering or partial mixed selection.
pub fn wiring_members(subject: &SelectionSubject) -> Result<Vec<AuthoredSelectionIdentity>> {
    let members = match subject {
        SelectionSubject::Object(id) => vec![id.clone()],
        SelectionSubject::Compound(group) => group.members.iter().cloned().collect(),
        _ => anyhow::bail!("Choose explicit Wire and Junction objects to transform"),
    };
    anyhow::ensure!(!members.is_empty(), "Select wiring before moving");
    let path = &members[0].instance_path;
    anyhow::ensure!(
        members.iter().all(
            |id| matches!(id.class, Class::Wire | Class::Junction) && &id.instance_path == path
        ),
        "Move requires the complete wiring selection from one occurrence"
    );
    Ok(members)
}

impl WiringGroupPreview {
    pub fn begin(
        base: &BoardReviewSceneV1,
        authoring: &SchematicAuthoring,
        subject: &SelectionSubject,
        reference: PointNm,
        root: PathBuf,
        locked: &std::collections::BTreeSet<Uuid>,
    ) -> Result<Self> {
        let members = wiring_members(subject)?;
        let document = base
            .schematic_document
            .as_ref()
            .context("Move source unavailable")?;
        anyhow::ensure!(
            members[0].instance_path == document.instance_path,
            "Select wiring in the captured schematic occurrence"
        );
        anyhow::ensure!(
            members.iter().all(|id| !locked.contains(&id.id)),
            "Selection contains locked wiring"
        );
        let sheet = authoring
            .source
            .as_ref()
            .and_then(|source| source.sheets.get(&document.source_sheet))
            .context("Move source unavailable")?;
        let targets = members.iter().map(target).collect::<Vec<_>>();
        let basis = WiringTransformBasis::capture(sheet, &targets)?;
        Ok(Self {
            base: base.clone(),
            basis,
            authoring: Box::new(authoring.clone()),
            reference,
            cursor_anchor: reference,
            endpoint_snap: false,
            members,
            sheet: sheet.uuid,
            rotation: 0,
            mirrored: false,
            root,
        })
    }

    pub(super) fn destination(&self, cursor: PointNm, snap: AuthoringSnapState) -> Result<PointNm> {
        super::translated_destination(
            Point::new(self.reference.x, self.reference.y),
            self.cursor_anchor,
            cursor,
            snap,
        )
    }
    fn pose(&self, at: PointNm) -> ClipboardPlacement {
        ClipboardPlacement {
            position: Point::new(at.x, at.y),
            rotation_deg: self.rotation,
            mirrored: self.mirrored,
        }
    }
    pub fn matches_subject(&self, subject: &SelectionSubject) -> bool {
        wiring_members(subject).is_ok_and(|members| members == self.members)
    }
    pub(super) fn singleton_identity(&self) -> Result<AuthoredSelectionIdentity> {
        anyhow::ensure!(
            self.members.len() == 1,
            "A compound has no privileged single identity"
        );
        Ok(self.members[0].clone())
    }
    pub(super) fn uses_electrical_pin_snap(&self) -> bool {
        self.endpoint_snap
    }
    pub fn instance_path(&self) -> Vec<Uuid> {
        self.members[0].instance_path.clone()
    }
    pub fn request(&self, at: PointNm) -> Result<SchematicEditRequest> {
        self.basis.plan(
            Point::new(self.reference.x, self.reference.y),
            self.pose(at),
        )?;
        Ok(SchematicEditRequest::DesignEdit {
            sheet: self.sheet,
            instance_path: self.instance_path(),
            edit: DesignEditIntent::TransformExisting {
                targets: self.members.iter().map(target).collect(),
                reference: Point::new(self.reference.x, self.reference.y),
                placement: self.pose(at),
                contact_policy: ExistingTransformContactPolicy::PreserveFixedContacts,
            },
        })
    }
    pub fn transform(&mut self, transform: PlacementTransform, at: PointNm) -> Result<bool> {
        let (rotation, mirrored) = transform.apply(self.rotation, self.mirrored);
        let pose = ClipboardPlacement {
            rotation_deg: rotation,
            mirrored,
            ..self.pose(at)
        };
        self.basis
            .plan(Point::new(self.reference.x, self.reference.y), pose)?;
        self.rotation = rotation;
        self.mirrored = mirrored;
        Ok(true)
    }
    pub fn scene_at(&self, at: PointNm) -> Result<BoardReviewSceneV1> {
        let plan = self.basis.plan(
            Point::new(self.reference.x, self.reference.y),
            self.pose(at),
        )?;
        let mut source = self
            .authoring
            .source
            .clone()
            .context("Move source unavailable")?;
        let sheet = source
            .sheets
            .get_mut(&self.sheet)
            .context("Move source changed")?;
        *sheet = plan.apply_to_sheet(sheet);
        self.authoring
            .project_source(&source, self.sheet, &self.base, &self.root)
    }
}
fn target(id: &AuthoredSelectionIdentity) -> ClipboardTarget {
    ClipboardTarget {
        id: id.id,
        class: if id.class == Class::Wire {
            ClipboardClass::Wire
        } else {
            ClipboardClass::Junction
        },
    }
}

impl super::NativeMovePreview {
    /// Rotation upgrades the explicit selected edge, never a derived net/run.
    pub(super) fn transform_selected_wiring(
        &mut self,
        transform: PlacementTransform,
        at: PointNm,
    ) -> Result<bool> {
        use super::{ConnectivityMoveTarget, NativeMovePreview};
        if let NativeMovePreview::Connectivity {
            base,
            target:
                ConnectivityMoveTarget::Wiring {
                    identity,
                    anchor,
                    locked_members,
                    edit,
                    ..
                },
            authoring,
            root,
            cursor_anchor,
            ..
        } = self
        {
            let mut group = WiringGroupPreview::begin(
                base,
                authoring,
                &SelectionSubject::Object(identity.clone()),
                PointNm {
                    x: anchor.x,
                    y: anchor.y,
                },
                root.clone(),
                locked_members,
            )?;
            // Endpoint and insertion gestures already follow absolute cursor
            // positions; span/vertex gestures retain their captured pickup offset.
            if !matches!(edit.handle, eda_engine::api::native_write::schematic_wire_edit::WireHandle::Endpoint { .. }
                | eda_engine::api::native_write::schematic_wire_edit::WireHandle::InsertVertex { .. }) {
                group.cursor_anchor = *cursor_anchor;
            }
            group.endpoint_snap = matches!(
                edit.handle,
                eda_engine::api::native_write::schematic_wire_edit::WireHandle::Endpoint { .. }
            );
            group.transform(transform, at)?;
            *self = NativeMovePreview::Wiring(group);
            return Ok(true);
        }
        Ok(false)
    }
    pub fn design_request(&self, at: PointNm) -> Result<SchematicEditRequest> {
        match self {
            Self::Wiring(group) => group.request(at),
            _ => anyhow::bail!("The captured Move is not a wiring transform"),
        }
    }
}
