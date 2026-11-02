//! Standalone annotation configures the same immutable Move preview.
use super::*;
use eda_engine::api::native_write::schematic_text::TextMoveBasis;
use eda_engine::ir::geometry::Point;

pub struct TextMovePreview {
    pub(super) base: BoardReviewSceneV1,
    pub(super) basis: TextMoveBasis,
    pub(super) authoring: Box<SchematicAuthoring>,
    pub(super) cursor_anchor: PointNm,
    pub(super) instance_path: Vec<Uuid>,
    pub(super) rotation: Option<i32>,
    pub(super) root: PathBuf,
}
impl TextMovePreview {
    pub fn id(&self) -> Uuid {
        self.basis.text.uuid
    }
    pub(super) fn begin(
        base: &BoardReviewSceneV1,
        authoring: &SchematicAuthoring,
        id: &crate::AuthoredSelectionIdentity,
        cursor: PointNm,
        snap: AuthoringSnapState,
        root: PathBuf,
    ) -> Result<Self> {
        let basis = authoring
            .text_move_bases
            .get(&id.id)
            .context("Text Move source unavailable")?
            .as_ref()
            .map_err(|r| anyhow::anyhow!("{r}"))?
            .clone();
        anyhow::ensure!(
            authoring.move_paths.get(&id.id) == Some(&id.instance_path)
                && authoring.sheet == Some(basis.sheet_id),
            "Text occurrence changed; select it again"
        );
        Ok(Self {
            base: base.clone(),
            basis,
            authoring: Box::new(authoring.clone()),
            cursor_anchor: checked_snap_authoring_point(cursor, snap)?,
            instance_path: id.instance_path.clone(),
            rotation: None,
            root,
        })
    }
    pub(super) fn transform(&mut self, transform: PlacementTransform, at: PointNm) -> Result<bool> {
        anyhow::ensure!(
            matches!(transform, PlacementTransform::Rotate),
            "Text supports rotation; mirroring is unavailable"
        );
        let rotation = (self
            .rotation
            .unwrap_or(self.basis.text.rotation)
            .rem_euclid(360)
            + 90)
            .rem_euclid(360);
        self.basis.destination(Point::new(at.x, at.y), rotation)?;
        self.rotation = Some(rotation);
        Ok(true)
    }
    pub(super) fn scene_at(&self, at: PointNm) -> Result<BoardReviewSceneV1> {
        let text = self.basis.destination(
            Point::new(at.x, at.y),
            self.rotation.unwrap_or(self.basis.text.rotation),
        )?;
        let mut source = self
            .authoring
            .source
            .clone()
            .context("Native schematic source unavailable")?;
        source
            .sheets
            .get_mut(&self.basis.sheet_id)
            .context("Native sheet changed")?
            .texts
            .insert(text.uuid, text);
        self.authoring
            .project_source(&source, self.basis.sheet_id, &self.base, &self.root)
    }
}
impl NativeMovePreview {
    pub fn text(
        base: &BoardReviewSceneV1,
        authoring: &SchematicAuthoring,
        id: &crate::AuthoredSelectionIdentity,
        cursor: PointNm,
        snap: AuthoringSnapState,
        root: PathBuf,
    ) -> Result<Self> {
        Ok(Self::Text(TextMovePreview::begin(
            base, authoring, id, cursor, snap, root,
        )?))
    }
}
