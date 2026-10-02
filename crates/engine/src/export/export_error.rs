use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error(
        "Track {source_id} manufacturing refused: {witness:?}; nonzero-deviation allowance or certified construction unavailable; no chord output is permitted"
    )]
    DeferredArcManufacturing {
        source_id: uuid::Uuid,
        witness: Box<super::ArcOutputWitness>,
    },
    #[error("Track {source_id} manufacturing witness failed: {reason:?}")]
    InvalidArcManufacturing {
        source_id: uuid::Uuid,
        reason: super::ArcWitnessError,
    },
    #[error("board outline export requires at least two vertices")]
    OutlineTooShort,
    #[error("outline aperture diameter must be positive")]
    InvalidAperture,
    #[error("copper-layer export requires positive track widths")]
    InvalidTrackWidth,
    #[error("copper-layer export requires positive via diameters")]
    InvalidViaDiameter,
    #[error("copper-layer export requires positive pad diameters")]
    InvalidPadDiameter,
    #[error("copper-layer export requires positive pad rectangle widths")]
    InvalidPadWidth,
    #[error("copper-layer export requires positive pad rectangle heights")]
    InvalidPadHeight,
    #[error("silkscreen export requires positive text heights")]
    InvalidTextHeight,
    #[error("silkscreen export requires positive text stroke widths")]
    InvalidTextStrokeWidth,
    #[error("text mesh tessellation failed")]
    TextMeshTessellationFailed,
    #[error("silkscreen export encountered unsupported text character: {0}")]
    UnsupportedSilkscreenTextCharacter(char),
    #[error("drill export requires positive via drill diameters")]
    InvalidViaDrill,
}
