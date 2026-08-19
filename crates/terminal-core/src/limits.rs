use std::error::Error;
use std::fmt;
use std::num::NonZeroUsize;

/// Every resource family that the core must account independently.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitKind {
    ParameterCount,
    ParameterDigits,
    ParameterValue,
    SubparameterCount,
    IntermediateBytes,
    ControlStringBytes,
    ClusterBytes,
    TitleBytes,
    WorkingDirectoryBytes,
    ClipboardBytes,
    NotificationBytes,
    ReplyBytes,
    PendingEvents,
    PendingDamage,
    HistoryLines,
    HistoryBytes,
    GraphicObjects,
    GraphicPixels,
    GraphicDecodedBytes,
    GraphicFrames,
    CompressionRatio,
    ParserWork,
    SearchWork,
    ReflowWork,
    ScreenCells,
    SnapshotCells,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LimitError {
    Zero {
        kind: LimitKind,
    },
    Exceeded {
        kind: LimitKind,
        requested: usize,
        maximum: usize,
    },
    ArithmeticOverflow {
        kind: LimitKind,
    },
}

impl fmt::Display for LimitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero { kind } => write!(formatter, "{kind:?} limit must be nonzero"),
            Self::Exceeded {
                kind,
                requested,
                maximum,
            } => write!(
                formatter,
                "{kind:?} request {requested} exceeds configured maximum {maximum}"
            ),
            Self::ArithmeticOverflow { kind } => {
                write!(formatter, "{kind:?} accounting overflowed")
            }
        }
    }
}

impl Error for LimitError {}

macro_rules! checked_limit {
    ($name:ident, $kind:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(NonZeroUsize);

        impl $name {
            pub fn new(value: usize) -> Result<Self, LimitError> {
                NonZeroUsize::new(value).map(Self).ok_or(LimitError::Zero {
                    kind: LimitKind::$kind,
                })
            }

            pub const fn get(self) -> usize {
                self.0.get()
            }

            pub fn check(self, requested: usize) -> Result<(), LimitError> {
                if requested <= self.get() {
                    Ok(())
                } else {
                    Err(LimitError::Exceeded {
                        kind: LimitKind::$kind,
                        requested,
                        maximum: self.get(),
                    })
                }
            }

            pub fn checked_total(self, left: usize, right: usize) -> Result<usize, LimitError> {
                let total = left
                    .checked_add(right)
                    .ok_or(LimitError::ArithmeticOverflow {
                        kind: LimitKind::$kind,
                    })?;
                self.check(total)?;
                Ok(total)
            }
        }
    };
}

checked_limit!(ParameterCountLimit, ParameterCount);
checked_limit!(ParameterDigitsLimit, ParameterDigits);
checked_limit!(ParameterValueLimit, ParameterValue);
checked_limit!(SubparameterCountLimit, SubparameterCount);
checked_limit!(IntermediateBytesLimit, IntermediateBytes);
checked_limit!(ControlStringBytesLimit, ControlStringBytes);
checked_limit!(ClusterBytesLimit, ClusterBytes);
checked_limit!(TitleBytesLimit, TitleBytes);
checked_limit!(WorkingDirectoryBytesLimit, WorkingDirectoryBytes);
checked_limit!(ClipboardBytesLimit, ClipboardBytes);
checked_limit!(NotificationBytesLimit, NotificationBytes);
checked_limit!(ReplyBytesLimit, ReplyBytes);
checked_limit!(PendingEventsLimit, PendingEvents);
checked_limit!(PendingDamageLimit, PendingDamage);
checked_limit!(HistoryLinesLimit, HistoryLines);
checked_limit!(HistoryBytesLimit, HistoryBytes);
checked_limit!(GraphicObjectsLimit, GraphicObjects);
checked_limit!(GraphicPixelsLimit, GraphicPixels);
checked_limit!(GraphicDecodedBytesLimit, GraphicDecodedBytes);
checked_limit!(GraphicFramesLimit, GraphicFrames);
checked_limit!(CompressionRatioLimit, CompressionRatio);
checked_limit!(ParserWorkLimit, ParserWork);
checked_limit!(SearchWorkLimit, SearchWork);
checked_limit!(ReflowWorkLimit, ReflowWork);
checked_limit!(ScreenCellsLimit, ScreenCells);
checked_limit!(SnapshotCellsLimit, SnapshotCells);

/// Raw owner-supplied values. DTC-P07 deliberately defines no numeric default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreLimitValues {
    pub parameter_count: usize,
    pub parameter_digits: usize,
    pub parameter_value: usize,
    pub subparameter_count: usize,
    pub intermediate_bytes: usize,
    pub control_string_bytes: usize,
    pub cluster_bytes: usize,
    pub title_bytes: usize,
    pub working_directory_bytes: usize,
    pub clipboard_bytes: usize,
    pub notification_bytes: usize,
    pub reply_bytes: usize,
    pub pending_events: usize,
    pub pending_damage: usize,
    pub history_lines: usize,
    pub history_bytes: usize,
    pub graphic_objects: usize,
    pub graphic_pixels: usize,
    pub graphic_decoded_bytes: usize,
    pub graphic_frames: usize,
    pub compression_ratio: usize,
    pub parser_work: usize,
    pub search_work: usize,
    pub reflow_work: usize,
    pub screen_cells: usize,
    pub snapshot_cells: usize,
}

/// Checked resource policy supplied by the application owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoreLimits {
    pub parameter_count: ParameterCountLimit,
    pub parameter_digits: ParameterDigitsLimit,
    pub parameter_value: ParameterValueLimit,
    pub subparameter_count: SubparameterCountLimit,
    pub intermediate_bytes: IntermediateBytesLimit,
    pub control_string_bytes: ControlStringBytesLimit,
    pub cluster_bytes: ClusterBytesLimit,
    pub title_bytes: TitleBytesLimit,
    pub working_directory_bytes: WorkingDirectoryBytesLimit,
    pub clipboard_bytes: ClipboardBytesLimit,
    pub notification_bytes: NotificationBytesLimit,
    pub reply_bytes: ReplyBytesLimit,
    pub pending_events: PendingEventsLimit,
    pub pending_damage: PendingDamageLimit,
    pub history_lines: HistoryLinesLimit,
    pub history_bytes: HistoryBytesLimit,
    pub graphic_objects: GraphicObjectsLimit,
    pub graphic_pixels: GraphicPixelsLimit,
    pub graphic_decoded_bytes: GraphicDecodedBytesLimit,
    pub graphic_frames: GraphicFramesLimit,
    pub compression_ratio: CompressionRatioLimit,
    pub parser_work: ParserWorkLimit,
    pub search_work: SearchWorkLimit,
    pub reflow_work: ReflowWorkLimit,
    pub screen_cells: ScreenCellsLimit,
    pub snapshot_cells: SnapshotCellsLimit,
}

impl TryFrom<CoreLimitValues> for CoreLimits {
    type Error = LimitError;

    fn try_from(raw: CoreLimitValues) -> Result<Self, Self::Error> {
        Ok(Self {
            parameter_count: ParameterCountLimit::new(raw.parameter_count)?,
            parameter_digits: ParameterDigitsLimit::new(raw.parameter_digits)?,
            parameter_value: ParameterValueLimit::new(raw.parameter_value)?,
            subparameter_count: SubparameterCountLimit::new(raw.subparameter_count)?,
            intermediate_bytes: IntermediateBytesLimit::new(raw.intermediate_bytes)?,
            control_string_bytes: ControlStringBytesLimit::new(raw.control_string_bytes)?,
            cluster_bytes: ClusterBytesLimit::new(raw.cluster_bytes)?,
            title_bytes: TitleBytesLimit::new(raw.title_bytes)?,
            working_directory_bytes: WorkingDirectoryBytesLimit::new(raw.working_directory_bytes)?,
            clipboard_bytes: ClipboardBytesLimit::new(raw.clipboard_bytes)?,
            notification_bytes: NotificationBytesLimit::new(raw.notification_bytes)?,
            reply_bytes: ReplyBytesLimit::new(raw.reply_bytes)?,
            pending_events: PendingEventsLimit::new(raw.pending_events)?,
            pending_damage: PendingDamageLimit::new(raw.pending_damage)?,
            history_lines: HistoryLinesLimit::new(raw.history_lines)?,
            history_bytes: HistoryBytesLimit::new(raw.history_bytes)?,
            graphic_objects: GraphicObjectsLimit::new(raw.graphic_objects)?,
            graphic_pixels: GraphicPixelsLimit::new(raw.graphic_pixels)?,
            graphic_decoded_bytes: GraphicDecodedBytesLimit::new(raw.graphic_decoded_bytes)?,
            graphic_frames: GraphicFramesLimit::new(raw.graphic_frames)?,
            compression_ratio: CompressionRatioLimit::new(raw.compression_ratio)?,
            parser_work: ParserWorkLimit::new(raw.parser_work)?,
            search_work: SearchWorkLimit::new(raw.search_work)?,
            reflow_work: ReflowWorkLimit::new(raw.reflow_work)?,
            screen_cells: ScreenCellsLimit::new(raw.screen_cells)?,
            snapshot_cells: SnapshotCellsLimit::new(raw.snapshot_cells)?,
        })
    }
}
