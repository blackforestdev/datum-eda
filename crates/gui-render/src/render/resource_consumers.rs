//! Logical production consumers of shared renderer resources; bytes count once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Consumer {
    Main,
    Board,
    Schematic,
    Revision,
    Global,
    Project,
    New,
    Layers,
    Navigator,
    Inspector,
    Menu,
    Console,
    Terminal,
    TerminalOverlay,
}

impl Consumer {
    pub const ALL: [Self; 14] = [
        Self::Main,
        Self::Board,
        Self::Schematic,
        Self::Revision,
        Self::Global,
        Self::Project,
        Self::New,
        Self::Layers,
        Self::Navigator,
        Self::Inspector,
        Self::Menu,
        Self::Console,
        Self::Terminal,
        Self::TerminalOverlay,
    ];

    pub fn adoption_id(self) -> &'static str {
        match self {
            Self::Main => "MAIN",
            Self::Board => "BOARD",
            Self::Schematic => "SCHEMATIC",
            Self::Revision => "REVISION",
            Self::Global => "GLOBAL",
            Self::Project => "PROJECT",
            Self::New => "NEW",
            Self::Layers => "LAYERS",
            Self::Navigator => "NAV",
            Self::Inspector => "INSPECTOR",
            Self::Menu => "MENU",
            Self::Console => "CONSOLE",
            Self::Terminal => "TERMINAL",
            Self::TerminalOverlay => "TERM_OVERLAY",
        }
    }
}

/// Incidence, not apportioned bytes: a shared allocation can have several consumers.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Consumers(u16);
impl std::fmt::Debug for Consumers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set()
            .entries(self.iter().map(Consumer::adoption_id))
            .finish()
    }
}
impl Consumers {
    pub fn contains(self, consumer: Consumer) -> bool {
        self.0 & (1 << consumer as u8) != 0
    }
    pub fn iter(self) -> impl Iterator<Item = Consumer> {
        Consumer::ALL.into_iter().filter(move |c| self.contains(*c))
    }
    pub(crate) fn insert(&mut self, consumer: Consumer) {
        self.0 |= 1 << consumer as u8;
    }
    pub(crate) fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub(crate) fn bits(self) -> u16 {
        self.0
    }
    pub(crate) fn from_bits(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<Consumer> for Consumers {
    fn from(value: Consumer) -> Self {
        Self(1 << value as u8)
    }
}

#[derive(Clone, Copy)]
#[repr(usize)]
pub(crate) enum Stream {
    Panel,
    Overlay,
    Text,
    OverlayText,
    Underlay,
    ViewportOverlay,
    BoardInteraction,
    Console,
    SchematicUnderlay,
    SchematicOverlay,
    Grid,
    BoardWorld,
    SchematicWorld,
    TerminalGraphics,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FrameConsumers([Consumers; 14]);
impl FrameConsumers {
    pub fn dialog(consumer: Consumer) -> Self {
        let mut result = Self::default();
        result.0[Stream::Overlay as usize] = consumer.into();
        result.0[Stream::OverlayText as usize] = consumer.into();
        result
    }
    pub fn note(&mut self, stream: Stream, consumer: Consumer, before: usize, after: usize) {
        if after > before {
            self.0[stream as usize].insert(consumer);
        }
    }
    pub fn include(&mut self, stream: Stream, consumers: Consumers) {
        self.0[stream as usize] = self.0[stream as usize].union(consumers);
    }
    pub fn get(self, stream: Stream) -> Consumers {
        self.0[stream as usize]
    }
    pub fn all(self) -> Consumers {
        self.0
            .into_iter()
            .fold(Consumers::default(), Consumers::union)
    }
}

impl crate::PreparedScene {
    pub(crate) fn consumer_incidence(&self) -> FrameConsumers {
        let mut frame = self.consumers;
        // Camera application can rebuild these immediate streams after composition.
        for (stream, consumer, count) in [
            (
                Stream::BoardInteraction,
                Consumer::Board,
                self.board_interaction_vertices.len(),
            ),
            (
                Stream::SchematicUnderlay,
                Consumer::Schematic,
                self.schematic_underlay_vertices.len(),
            ),
            (
                Stream::SchematicOverlay,
                Consumer::Schematic,
                self.schematic_overlay_vertices.len(),
            ),
        ] {
            frame.0[stream as usize] = if count == 0 {
                Consumers::default()
            } else {
                consumer.into()
            };
        }
        frame
    }

    /// Select the actual native dialog adapter when Preferences share a painter.
    pub fn set_native_consumer(&mut self, consumer: Consumer) {
        assert!(matches!(
            consumer,
            Consumer::Global | Consumer::Project | Consumer::New
        ));
        self.consumers = FrameConsumers::dialog(consumer);
    }
}

impl crate::Renderer {
    /// Publish producer incidence before preparing upload receipts or GPU holds.
    /// Whole shared streams carry their producer set; bytes are never multiplied.
    pub(crate) fn publish_resource_consumers(&self) {
        let frame = self.frame_consumers;
        self.panel_gpu.set_consumers(frame.get(Stream::Panel));
        self.menu_overlay_gpu
            .set_consumers(frame.get(Stream::Overlay));
        self.viewport_underlay_gpu
            .set_consumers(frame.get(Stream::Underlay));
        self.viewport_overlay_gpu
            .set_consumers(frame.get(Stream::ViewportOverlay));
        self.board_interaction_gpu
            .set_consumers(frame.get(Stream::BoardInteraction));
        self.console_gpu
            .vertices
            .set_consumers(frame.get(Stream::Console));
        self.schematic_underlay_gpu
            .set_consumers(frame.get(Stream::SchematicUnderlay));
        self.schematic_overlay_gpu
            .set_consumers(frame.get(Stream::SchematicOverlay));
        self.surface_grid_gpu.set_consumers(frame.get(Stream::Grid));
        self.world_vertices_gpu
            .set_consumers(frame.get(Stream::BoardWorld));
        self.world_strokes_gpu
            .set_consumers(frame.get(Stream::BoardWorld));
        self.schematic_world_vertices_gpu
            .set_consumers(frame.get(Stream::SchematicWorld));
        self.schematic_world_strokes_gpu
            .set_consumers(frame.get(Stream::SchematicWorld));
        self.scene_bind_group
            .buffer
            .set_consumers(frame.get(Stream::BoardWorld));
        self.schematic_scene_bind_group
            .buffer
            .set_consumers(frame.get(Stream::SchematicWorld));
        self.uniform_buffer.set_consumers(frame.all());
        self.surface_attachments.set_consumers(frame.all());
        self.text_renderer.set_consumers(frame.get(Stream::Text));
        self.menu_overlay_text_renderer
            .set_consumers(frame.get(Stream::OverlayText));
        self.atlas.set_consumers(
            frame
                .get(Stream::Text)
                .union(frame.get(Stream::OverlayText)),
        );
        self.terminal_graphics
            .set_consumers(frame.get(Stream::TerminalGraphics));
    }
}
