//! Bounded acknowledgement framing, derived from the exact producer wire type.
use super::*;
use anyhow::Result;
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read};

// Diagnostic storage only: retain the final 4 KiB while always draining the
// pipe. This memory ceiling is neither an acknowledgement nor a result limit.
const DIAGNOSTIC_TAIL_BYTES: usize = 4096;

pub(super) struct StreamReceipt {
    pub accepted: bool,
    pub error: Option<String>,
    pub stderr_bytes: u64,
    pub diagnostic_tail: VecDeque<u8>,
}

/// Non-authoritative stderr is drained, counted and retained as a bounded tail.
/// Candidate frames alone retain bytes, up to prefix + exact JSON + CR delimiter.
pub(super) fn read(
    input: impl Read,
    request: &CheckProducerRequest,
    mut accepted: impl FnMut(CheckProducerAcceptance),
) -> Result<StreamReceipt> {
    let prefix = ACCEPTED_PREFIX.as_bytes();
    let maximum = prefix
        .len()
        .checked_add(serde_json::to_vec(&request.acceptance())?.len())
        .and_then(|length| length.checked_add(1))
        .ok_or_else(|| anyhow::anyhow!("ERC acknowledgement size overflow"))?;
    let mut candidate = Vec::with_capacity(maximum);
    let mut discard = false;
    let mut receipt = StreamReceipt {
        accepted: false,
        error: None,
        stderr_bytes: 0,
        diagnostic_tail: VecDeque::with_capacity(DIAGNOSTIC_TAIL_BYTES),
    };
    let mut reader = BufReader::new(input);
    loop {
        let bytes = reader.fill_buf()?;
        if bytes.is_empty() {
            break;
        }
        for &byte in bytes {
            receipt.stderr_bytes = receipt.stderr_bytes.saturating_add(1);
            if receipt.diagnostic_tail.len() == DIAGNOSTIC_TAIL_BYTES {
                receipt.diagnostic_tail.pop_front();
            }
            receipt.diagnostic_tail.push_back(byte);
            if byte == b'\n' {
                if !discard && candidate.starts_with(prefix) {
                    if candidate.last() == Some(&b'\r') {
                        candidate.pop();
                    }
                    let value = std::str::from_utf8(&candidate)
                        .map_err(anyhow::Error::from)
                        .and_then(|line| CheckProducerAcceptance::decode(line, request));
                    match value {
                        Ok(Some(value)) if !receipt.accepted && receipt.error.is_none() => {
                            receipt.accepted = true;
                            accepted(value);
                        }
                        Ok(Some(_)) => {
                            receipt
                                .error
                                .get_or_insert_with(|| "Duplicate ERC acknowledgement".into());
                        }
                        Ok(None) => unreachable!("candidate has full prefix"),
                        Err(error) => {
                            receipt.error.get_or_insert_with(|| error.to_string());
                        }
                    }
                }
                candidate.clear();
                discard = false;
            } else if !discard {
                if candidate.len() < prefix.len() && byte != prefix[candidate.len()] {
                    discard = true;
                    candidate.clear();
                } else if candidate.len() == maximum {
                    receipt
                        .error
                        .get_or_insert_with(|| "Oversized ERC acknowledgement".into());
                    candidate.clear();
                    discard = true;
                } else {
                    candidate.push(byte);
                }
            }
        }
        let count = bytes.len();
        reader.consume(count);
    }
    if candidate.starts_with(prefix) {
        receipt
            .error
            .get_or_insert_with(|| "ERC acknowledgement has no terminating delimiter".into());
    }
    Ok(receipt)
}

#[cfg(test)]
#[path = "acceptance_stream_tests.rs"]
mod tests;
