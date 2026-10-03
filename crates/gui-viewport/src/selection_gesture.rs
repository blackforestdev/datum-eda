//! One press/motion/release owner; cancellation never publishes selection.
use crate::selection_acquisition::AcquisitionMode;
use datum_gui_protocol::{PaneId, PointNm};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionKind {
    Rectangle,
    Lasso,
}
#[derive(Debug, Clone)]
pub struct SelectionGesture {
    pub pane: PaneId,
    pub mode: AcquisitionMode,
    pub kind: Option<RegionKind>,
    pub origin: PointNm,
    pub current: PointNm,
    pub samples: Vec<PointNm>,
    start: (f32, f32),
    pub source_revision: String,
}
impl SelectionGesture {
    pub fn begin(
        pane: PaneId,
        screen: (f32, f32),
        world: PointNm,
        shift: bool,
        control: bool,
        space: bool,
        revision: String,
    ) -> Option<Self> {
        if space || (shift && control) {
            return None;
        }
        Some(Self {
            pane,
            mode: if control {
                AcquisitionMode::Remove
            } else if shift {
                AcquisitionMode::Add
            } else {
                AcquisitionMode::Replace
            },
            kind: None,
            origin: world,
            current: world,
            samples: vec![world],
            start: screen,
            source_revision: revision,
        })
    }
    pub fn motion(&mut self, screen: (f32, f32), world: PointNm) -> bool {
        if self.kind.is_none() {
            let dx = screen.0 - self.start.0;
            let dy = screen.1 - self.start.1;
            if dx.hypot(dy) < 4.0 {
                return false;
            }
            self.kind = Some(if dx >= 0.0 {
                RegionKind::Rectangle
            } else {
                RegionKind::Lasso
            });
        }
        self.current = world;
        if self.kind == Some(RegionKind::Lasso) && self.samples.last() != Some(&world) {
            self.samples.push(world);
        }
        true
    }
    pub fn boundary(&self) -> Vec<PointNm> {
        match self.kind {
            Some(RegionKind::Rectangle) => vec![
                self.origin,
                PointNm {
                    x: self.current.x,
                    y: self.origin.y,
                },
                self.current,
                PointNm {
                    x: self.origin.x,
                    y: self.current.y,
                },
            ],
            Some(RegionKind::Lasso) => self.samples.clone(),
            None => Vec::new(),
        }
    }
    pub fn edge_pan(&self, screen: (f32, f32), rect: [f32; 4]) -> (f32, f32) {
        if self.kind.is_none() {
            return (0.0, 0.0);
        }
        let axis = |p: f32, min: f32, length: f32| {
            if p < min + 24.0 {
                ((min + 24.0 - p) / 24.0).min(3.0) * 6.0
            } else if p > min + length - 24.0 {
                -((p - (min + length - 24.0)) / 24.0).min(3.0) * 6.0
            } else {
                0.0
            }
        };
        (
            axis(screen.0, rect[0], rect[2]),
            axis(screen.1, rect[1], rect[3]),
        )
    }
}
