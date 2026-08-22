use crate::{
    Action, ControlCode, ControlString, ControlStringKind, CoreLimits, CsiParameter, CsiSequence,
    EscapeSequence, LimitKind, ParseError, ParserStateKind, StringTerminator,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FeedReport {
    pub consumed: usize,
    pub actions: usize,
    pub work_exhausted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    Ground,
    Escape(SequenceBytes),
    Csi(CsiBuilder),
    String(StringBuilder),
    Discard(DiscardState),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct SequenceBytes {
    intermediates: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ParameterBuilder {
    subparameters: Vec<Option<usize>>,
    value: Option<usize>,
    digits: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CsiBuilder {
    private_markers: Vec<u8>,
    parameters: Vec<CsiParameter>,
    current: ParameterBuilder,
    saw_parameter_bytes: bool,
    intermediates: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StringBuilder {
    kind: ControlStringKind,
    bytes: Vec<u8>,
    escape_pending: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DiscardState {
    Escape,
    Csi,
    String {
        kind: ControlStringKind,
        escape_pending: bool,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Utf8Decoder {
    bytes: [u8; 4],
    len: usize,
    expected: usize,
}

impl Utf8Decoder {
    fn is_pending(&self) -> bool {
        self.len != 0
    }

    fn reset(&mut self) {
        self.len = 0;
        self.expected = 0;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamingParser {
    limits: CoreLimits,
    state: State,
    utf8: Utf8Decoder,
}

impl StreamingParser {
    pub fn new(limits: CoreLimits) -> Self {
        Self {
            limits,
            state: State::Ground,
            utf8: Utf8Decoder::default(),
        }
    }

    pub fn state(&self) -> ParserStateKind {
        state_kind(&self.state)
    }

    pub fn can_fast_path_ascii(&self) -> bool {
        matches!(self.state, State::Ground) && !self.utf8.is_pending()
    }

    pub fn feed(&mut self, input: &[u8], mut emit: impl FnMut(Action)) -> FeedReport {
        let take = input.len().min(self.limits.parser_work.get());
        let mut actions = 0;
        for &byte in &input[..take] {
            self.process_byte(byte, &mut |action| {
                actions += 1;
                emit(action);
            });
        }
        FeedReport {
            consumed: take,
            actions,
            work_exhausted: take < input.len(),
        }
    }

    pub fn finish(&mut self, mut emit: impl FnMut(Action)) -> usize {
        let mut actions = 0;
        if self.utf8.is_pending() {
            self.utf8.reset();
            emit(Action::Error(ParseError::MalformedUtf8));
            emit(Action::Print('\u{fffd}'));
            actions += 2;
        }
        if !matches!(self.state, State::Ground) {
            let state = state_kind(&self.state);
            self.state = State::Ground;
            emit(Action::Error(ParseError::IncompleteSequence { state }));
            actions += 1;
        }
        actions
    }

    fn process_byte(&mut self, byte: u8, emit: &mut impl FnMut(Action)) {
        if matches!(self.state, State::Ground) {
            self.process_ground(byte, emit);
            return;
        }

        if byte == 0x18 || byte == 0x1a {
            let state = state_kind(&self.state);
            self.state = State::Ground;
            emit(Action::Cancelled {
                state,
                by: ControlCode::new(byte).expect("CAN and SUB are control codes"),
            });
            return;
        }
        if byte == 0x1b
            && !matches!(
                self.state,
                State::String(_) | State::Discard(DiscardState::String { .. })
            )
        {
            self.state = State::Escape(SequenceBytes::default());
            return;
        }

        let state = std::mem::replace(&mut self.state, State::Ground);
        self.state = match state {
            State::Escape(sequence) => self.process_escape(sequence, byte, emit),
            State::Csi(sequence) => self.process_csi(sequence, byte, emit),
            State::String(sequence) => self.process_string(sequence, byte, emit),
            State::Discard(discard) => self.process_discard(discard, byte),
            State::Ground => unreachable!(),
        };
    }

    fn process_ground(&mut self, byte: u8, emit: &mut impl FnMut(Action)) {
        if self.utf8.is_pending() {
            if (0x80..=0xbf).contains(&byte) {
                self.utf8.bytes[self.utf8.len] = byte;
                self.utf8.len += 1;
                if self.utf8.len == self.utf8.expected {
                    let bytes = &self.utf8.bytes[..self.utf8.len];
                    match std::str::from_utf8(bytes)
                        .ok()
                        .and_then(|text| text.chars().next())
                    {
                        Some(character) => emit(Action::Print(character)),
                        None => malformed_utf8(emit),
                    }
                    self.utf8.reset();
                }
                return;
            }
            self.utf8.reset();
            malformed_utf8(emit);
            self.process_ground(byte, emit);
            return;
        }

        match byte {
            0x1b => self.state = State::Escape(SequenceBytes::default()),
            0x9b => self.state = State::Csi(CsiBuilder::default()),
            0x90 => self.start_string(ControlStringKind::Dcs),
            0x9d => self.start_string(ControlStringKind::Osc),
            0x98 => self.start_string(ControlStringKind::Sos),
            0x9e => self.start_string(ControlStringKind::Pm),
            0x9f => self.start_string(ControlStringKind::Apc),
            0x00..=0x1f | 0x7f..=0x9f => emit(Action::Execute(
                ControlCode::new(byte).expect("matched byte is a control code"),
            )),
            0x20..=0x7e => emit(Action::Print(char::from(byte))),
            0xc2..=0xdf => self.start_utf8(byte, 2),
            0xe0..=0xef => self.start_utf8(byte, 3),
            0xf0..=0xf4 => self.start_utf8(byte, 4),
            _ => malformed_utf8(emit),
        }
    }

    fn process_escape(
        &mut self,
        mut sequence: SequenceBytes,
        byte: u8,
        emit: &mut impl FnMut(Action),
    ) -> State {
        match byte {
            0x00..=0x1f | 0x7f => {
                emit_control(byte, emit);
                State::Escape(sequence)
            }
            0x20..=0x2f => match push_bounded(
                &mut sequence.intermediates,
                byte,
                self.limits.intermediate_bytes.get(),
            ) {
                Ok(()) => State::Escape(sequence),
                Err(()) => limit_discard(LimitKind::IntermediateBytes, DiscardState::Escape, emit),
            },
            0x30..=0x7e => {
                match byte {
                    _ if !sequence.intermediates.is_empty() => {}
                    b'[' => return State::Csi(CsiBuilder::default()),
                    b'P' => return self.string_state(ControlStringKind::Dcs),
                    b']' => return self.string_state(ControlStringKind::Osc),
                    b'X' => return self.string_state(ControlStringKind::Sos),
                    b'^' => return self.string_state(ControlStringKind::Pm),
                    b'_' => return self.string_state(ControlStringKind::Apc),
                    _ => {}
                }
                emit(Action::Escape(EscapeSequence {
                    intermediates: sequence.intermediates,
                    final_byte: byte,
                }));
                State::Ground
            }
            _ => unexpected(ParserStateKind::Escape, byte, emit),
        }
    }

    fn process_csi(
        &mut self,
        mut sequence: CsiBuilder,
        byte: u8,
        emit: &mut impl FnMut(Action),
    ) -> State {
        match byte {
            0x00..=0x1f | 0x7f => {
                emit_control(byte, emit);
                State::Csi(sequence)
            }
            b'0'..=b'9' if sequence.intermediates.is_empty() => {
                match sequence.push_digit(byte - b'0', &self.limits) {
                    Ok(()) => State::Csi(sequence),
                    Err(kind) => limit_discard(kind, DiscardState::Csi, emit),
                }
            }
            b';' if sequence.intermediates.is_empty() => {
                match sequence.next_parameter(&self.limits) {
                    Ok(()) => State::Csi(sequence),
                    Err(kind) => limit_discard(kind, DiscardState::Csi, emit),
                }
            }
            b':' if sequence.intermediates.is_empty() => {
                match sequence.next_subparameter(&self.limits) {
                    Ok(()) => State::Csi(sequence),
                    Err(kind) => limit_discard(kind, DiscardState::Csi, emit),
                }
            }
            0x3c..=0x3f if !sequence.saw_parameter_bytes && sequence.intermediates.is_empty() => {
                if push_bounded(
                    &mut sequence.private_markers,
                    byte,
                    self.limits.parameter_count.get(),
                )
                .is_ok()
                {
                    State::Csi(sequence)
                } else {
                    limit_discard(LimitKind::ParameterCount, DiscardState::Csi, emit)
                }
            }
            0x20..=0x2f => {
                if push_bounded(
                    &mut sequence.intermediates,
                    byte,
                    self.limits.intermediate_bytes.get(),
                )
                .is_ok()
                {
                    State::Csi(sequence)
                } else {
                    limit_discard(LimitKind::IntermediateBytes, DiscardState::Csi, emit)
                }
            }
            0x40..=0x7e => match sequence.finish(byte, &self.limits) {
                Ok(action) => {
                    emit(Action::Csi(action));
                    State::Ground
                }
                Err(kind) => {
                    emit(Action::Error(ParseError::LimitExceeded(kind)));
                    State::Ground
                }
            },
            _ => {
                emit(Action::Error(ParseError::UnexpectedByte {
                    state: ParserStateKind::Csi,
                    byte,
                }));
                State::Discard(DiscardState::Csi)
            }
        }
    }

    fn process_string(
        &mut self,
        mut sequence: StringBuilder,
        byte: u8,
        emit: &mut impl FnMut(Action),
    ) -> State {
        if sequence.escape_pending {
            sequence.escape_pending = false;
            if byte == b'\\' {
                emit_string(sequence, StringTerminator::StringTerminator, emit);
                return State::Ground;
            }
            if self.push_string_byte(&mut sequence, 0x1b, emit).is_err() {
                return State::Discard(DiscardState::String {
                    kind: sequence.kind,
                    escape_pending: false,
                });
            }
        }
        match byte {
            0x1b => {
                sequence.escape_pending = true;
                State::String(sequence)
            }
            0x9c => {
                emit_string(sequence, StringTerminator::StringTerminator, emit);
                State::Ground
            }
            0x07 if sequence.kind == ControlStringKind::Osc => {
                emit_string(sequence, StringTerminator::Bell, emit);
                State::Ground
            }
            _ => match self.push_string_byte(&mut sequence, byte, emit) {
                Ok(()) => State::String(sequence),
                Err(()) => State::Discard(DiscardState::String {
                    kind: sequence.kind,
                    escape_pending: false,
                }),
            },
        }
    }

    fn process_discard(&self, discard: DiscardState, byte: u8) -> State {
        match discard {
            DiscardState::Escape => {
                if (0x30..=0x7e).contains(&byte) {
                    State::Ground
                } else {
                    State::Discard(discard)
                }
            }
            DiscardState::Csi => {
                if (0x40..=0x7e).contains(&byte) {
                    State::Ground
                } else {
                    State::Discard(discard)
                }
            }
            DiscardState::String {
                kind,
                escape_pending,
            } => {
                if byte == 0x9c
                    || (kind == ControlStringKind::Osc && byte == 0x07)
                    || (escape_pending && byte == b'\\')
                {
                    State::Ground
                } else {
                    State::Discard(DiscardState::String {
                        kind,
                        escape_pending: byte == 0x1b,
                    })
                }
            }
        }
    }

    fn start_utf8(&mut self, byte: u8, expected: usize) {
        self.utf8.bytes[0] = byte;
        self.utf8.len = 1;
        self.utf8.expected = expected;
    }

    fn start_string(&mut self, kind: ControlStringKind) {
        self.state = self.string_state(kind);
    }

    fn string_state(&self, kind: ControlStringKind) -> State {
        State::String(StringBuilder {
            kind,
            bytes: Vec::new(),
            escape_pending: false,
        })
    }

    fn push_string_byte(
        &self,
        sequence: &mut StringBuilder,
        byte: u8,
        emit: &mut impl FnMut(Action),
    ) -> Result<(), ()> {
        if push_bounded(
            &mut sequence.bytes,
            byte,
            self.limits.control_string_bytes.get(),
        )
        .is_err()
        {
            emit(Action::Error(ParseError::LimitExceeded(
                LimitKind::ControlStringBytes,
            )));
            Err(())
        } else {
            Ok(())
        }
    }
}

impl CsiBuilder {
    fn push_digit(&mut self, digit: u8, limits: &CoreLimits) -> Result<(), LimitKind> {
        self.saw_parameter_bytes = true;
        self.current.digits += 1;
        if self.current.digits > limits.parameter_digits.get() {
            return Err(LimitKind::ParameterDigits);
        }
        let value = self.current.value.unwrap_or(0);
        let value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(usize::from(digit)))
            .ok_or(LimitKind::ParameterValue)?;
        if value > limits.parameter_value.get() {
            return Err(LimitKind::ParameterValue);
        }
        self.current.value = Some(value);
        Ok(())
    }

    fn next_parameter(&mut self, limits: &CoreLimits) -> Result<(), LimitKind> {
        self.saw_parameter_bytes = true;
        self.finish_current(limits)?;
        if self.parameters.len() >= limits.parameter_count.get() {
            return Err(LimitKind::ParameterCount);
        }
        self.parameters.push(CsiParameter {
            subparameters: std::mem::take(&mut self.current.subparameters),
        });
        self.current = ParameterBuilder::default();
        Ok(())
    }

    fn next_subparameter(&mut self, limits: &CoreLimits) -> Result<(), LimitKind> {
        self.saw_parameter_bytes = true;
        self.push_current_subparameter(limits)
    }

    fn finish(mut self, final_byte: u8, limits: &CoreLimits) -> Result<CsiSequence, LimitKind> {
        if self.saw_parameter_bytes {
            self.finish_current(limits)?;
            if self.parameters.len() >= limits.parameter_count.get() {
                return Err(LimitKind::ParameterCount);
            }
            self.parameters.push(CsiParameter {
                subparameters: self.current.subparameters,
            });
        }
        Ok(CsiSequence {
            private_markers: self.private_markers,
            parameters: self.parameters,
            intermediates: self.intermediates,
            final_byte,
        })
    }

    fn finish_current(&mut self, limits: &CoreLimits) -> Result<(), LimitKind> {
        self.push_current_subparameter(limits)
    }

    fn push_current_subparameter(&mut self, limits: &CoreLimits) -> Result<(), LimitKind> {
        if self.current.subparameters.len() >= limits.subparameter_count.get() {
            return Err(LimitKind::SubparameterCount);
        }
        self.current.subparameters.push(self.current.value.take());
        self.current.digits = 0;
        Ok(())
    }
}

fn state_kind(state: &State) -> ParserStateKind {
    match state {
        State::Ground => ParserStateKind::Ground,
        State::Escape(_) | State::Discard(DiscardState::Escape) => ParserStateKind::Escape,
        State::Csi(_) | State::Discard(DiscardState::Csi) => ParserStateKind::Csi,
        State::String(sequence) => sequence.kind.state(),
        State::Discard(DiscardState::String { kind, .. }) => kind.state(),
    }
}

fn malformed_utf8(emit: &mut impl FnMut(Action)) {
    emit(Action::Error(ParseError::MalformedUtf8));
    emit(Action::Print('\u{fffd}'));
}

fn emit_control(byte: u8, emit: &mut impl FnMut(Action)) {
    emit(Action::Execute(
        ControlCode::new(byte).expect("matched byte is a control code"),
    ));
}

fn emit_string(
    sequence: StringBuilder,
    terminator: StringTerminator,
    emit: &mut impl FnMut(Action),
) {
    emit(Action::ControlString(ControlString {
        kind: sequence.kind,
        bytes: sequence.bytes,
        terminator,
    }));
}

fn push_bounded(bytes: &mut Vec<u8>, byte: u8, limit: usize) -> Result<(), ()> {
    if bytes.len() >= limit {
        Err(())
    } else {
        bytes.push(byte);
        Ok(())
    }
}

fn unexpected(state: ParserStateKind, byte: u8, emit: &mut impl FnMut(Action)) -> State {
    emit(Action::Error(ParseError::UnexpectedByte { state, byte }));
    State::Ground
}

fn limit_discard(kind: LimitKind, discard: DiscardState, emit: &mut impl FnMut(Action)) -> State {
    emit(Action::Error(ParseError::LimitExceeded(kind)));
    State::Discard(discard)
}
