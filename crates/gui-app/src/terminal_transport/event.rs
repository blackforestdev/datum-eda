#[derive(Debug)]
pub(crate) enum TerminalTransportEvent {
    Output(Vec<u8>),
    Exited(Option<i32>),
}
