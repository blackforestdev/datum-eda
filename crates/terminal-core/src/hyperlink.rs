use crate::{HyperlinkBytesLimit, HyperlinkId, LimitError, LimitKind};
use std::collections::VecDeque;
use std::num::NonZeroU64;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hyperlink {
    id: HyperlinkId,
    parameters: String,
    uri: String,
}

impl Hyperlink {
    pub const fn id(&self) -> HyperlinkId {
        self.id
    }
    pub fn parameters(&self) -> &str {
        &self.parameters
    }
    pub fn uri(&self) -> &str {
        &self.uri
    }
    fn payload_bytes(&self) -> usize {
        self.parameters.len() + self.uri.len()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct HyperlinkRegistry {
    entries: VecDeque<Hyperlink>,
    payload_bytes: usize,
    next_id: u64,
    limit: HyperlinkBytesLimit,
}

impl HyperlinkRegistry {
    pub(crate) fn new(limit: HyperlinkBytesLimit) -> Self {
        Self {
            entries: VecDeque::new(),
            payload_bytes: 0,
            next_id: 1,
            limit,
        }
    }

    pub(crate) fn insert(
        &mut self,
        parameters: String,
        uri: String,
    ) -> Result<HyperlinkId, LimitError> {
        let requested =
            parameters
                .len()
                .checked_add(uri.len())
                .ok_or(LimitError::ArithmeticOverflow {
                    kind: LimitKind::HyperlinkBytes,
                })?;
        self.limit.check(requested)?;
        while self
            .payload_bytes
            .checked_add(requested)
            .is_none_or(|total| total > self.limit.get())
        {
            let Some(evicted) = self.entries.pop_front() else {
                break;
            };
            self.payload_bytes -= evicted.payload_bytes();
        }
        let raw_id = NonZeroU64::new(self.next_id).ok_or(LimitError::ArithmeticOverflow {
            kind: LimitKind::HyperlinkBytes,
        })?;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(LimitError::ArithmeticOverflow {
                kind: LimitKind::HyperlinkBytes,
            })?;
        let id = HyperlinkId::new(raw_id);
        self.entries.push_back(Hyperlink {
            id,
            parameters,
            uri,
        });
        self.payload_bytes += requested;
        Ok(id)
    }

    pub(crate) fn get(&self, id: HyperlinkId) -> Option<&Hyperlink> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.payload_bytes = 0;
    }
}

impl crate::TerminalCore {
    pub fn uri_open_request(&self, id: HyperlinkId) -> Option<crate::CoreEvent> {
        self.state
            .hyperlinks
            .get(id)
            .cloned()
            .map(crate::CoreEvent::OpenUriRequest)
    }
}
