//! One owner for declared painter order and its shared stage walk.
/// Declared render-stack policy (`docs/gui/M7_RENDER_LAYER_DISCIPLINE_MEMO.md`,
/// 2026-04-16 rule): layer type group first, then back-to-front side, with
/// scene `render_order` only as a stable in-stage tie-breaker. Declaration
/// order IS the draw order; `render_stage_priority` derives from it. Do not
/// reintroduce a second ordering encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RenderStage {
    BottomCopper,
    UnresolvedCopper,
    InnerCopper,
    TopCopper,
    BottomMask,
    TopMask,
    BottomPaste,
    TopPaste,
    BottomSilk,
    TopSilk,
    Mechanical,
    Edge,
    Other,
}

/// The shared post-copper stage walk, in declared draw order. Both retained
/// scene assembly and board-graphics emission iterate this one list.
pub(crate) const POST_COPPER_STAGES: [RenderStage; 9] = [
    RenderStage::BottomMask,
    RenderStage::TopMask,
    RenderStage::BottomPaste,
    RenderStage::TopPaste,
    RenderStage::BottomSilk,
    RenderStage::TopSilk,
    RenderStage::Mechanical,
    RenderStage::Edge,
    RenderStage::Other,
];

pub(crate) fn render_stage_priority(stage: RenderStage) -> u32 {
    // The enum declaration order is the single encoding of the declared
    // render-stack policy; priority is its discriminant.
    stage as u32
}
