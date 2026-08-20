use crate::codec::{CodecError, CodecLimits, CodecStage, WorkBudget, checked_product};
use crate::{LimitError, LimitKind};

const MAX_BITS: usize = 15;
const MAX_DISTANCE: usize = 32_768;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeflateOutput {
    pub bytes: Vec<u8>,
    pub consumed_bytes: usize,
}

/// Decode one RFC 1951 DEFLATE stream and report its byte-aligned extent.
pub fn decode_deflate(input: &[u8], limits: CodecLimits) -> Result<DeflateOutput, CodecError> {
    let mut work = WorkBudget::new(limits.work);
    decode_deflate_with_work(input, limits, &mut work)
}

pub(crate) fn decode_deflate_with_work(
    input: &[u8],
    limits: CodecLimits,
    work: &mut WorkBudget,
) -> Result<DeflateOutput, CodecError> {
    let ratio_ceiling = checked_product(
        input.len(),
        limits.compression_ratio.get(),
        LimitKind::CompressionRatio,
    )?;
    let (output_ceiling, output_limit_kind) = if ratio_ceiling < limits.decoded_bytes.get() {
        (ratio_ceiling, LimitKind::CompressionRatio)
    } else {
        (limits.decoded_bytes.get(), LimitKind::GraphicDecodedBytes)
    };
    let mut reader = BitReader::new(input, work);
    let mut output = Vec::new();

    loop {
        let final_block = reader.read_bits(1)? != 0;
        match reader.read_bits(2)? {
            0 => decode_stored(&mut reader, &mut output, output_ceiling, output_limit_kind)?,
            1 => {
                let (literal, distance) = fixed_trees()?;
                decode_compressed(
                    &mut reader,
                    &mut output,
                    output_ceiling,
                    output_limit_kind,
                    &literal,
                    &distance,
                )?;
            }
            2 => {
                let (literal, distance) = dynamic_trees(&mut reader)?;
                decode_compressed(
                    &mut reader,
                    &mut output,
                    output_ceiling,
                    output_limit_kind,
                    &literal,
                    &distance,
                )?;
            }
            _ => {
                return Err(CodecError::InvalidDeflate {
                    reason: "reserved block type",
                });
            }
        }
        if final_block {
            break;
        }
    }

    Ok(DeflateOutput {
        bytes: output,
        consumed_bytes: reader.consumed_bytes(),
    })
}

fn decode_stored(
    reader: &mut BitReader<'_, '_>,
    output: &mut Vec<u8>,
    ceiling: usize,
    limit_kind: LimitKind,
) -> Result<(), CodecError> {
    reader.align_byte();
    let length = usize::from(reader.read_u16_le()?);
    let complement = reader.read_u16_le()?;
    if (length as u16) != !complement {
        return Err(CodecError::InvalidDeflate {
            reason: "stored block length complement mismatch",
        });
    }
    reserve_output(output, length, ceiling, limit_kind)?;
    for _ in 0..length {
        let byte = reader.read_aligned_byte()?;
        output.push(byte);
    }
    Ok(())
}

fn decode_compressed(
    reader: &mut BitReader<'_, '_>,
    output: &mut Vec<u8>,
    ceiling: usize,
    limit_kind: LimitKind,
    literal: &Huffman,
    distance: &Huffman,
) -> Result<(), CodecError> {
    loop {
        match literal.decode(reader)? {
            symbol @ 0..=255 => {
                reserve_output(output, 1, ceiling, limit_kind)?;
                output.push(symbol as u8);
            }
            256 => return Ok(()),
            symbol @ 257..=285 => {
                let (base, extra) = length_code(symbol)?;
                let length = base + reader.read_bits(extra)? as usize;
                let distance_symbol = distance.decode(reader)?;
                let (distance_base, distance_extra) = distance_code(distance_symbol)?;
                let distance = distance_base + reader.read_bits(distance_extra)? as usize;
                if distance == 0 || distance > MAX_DISTANCE || distance > output.len() {
                    return Err(CodecError::InvalidDeflate {
                        reason: "distance exceeds decoded history",
                    });
                }
                reserve_output(output, length, ceiling, limit_kind)?;
                for _ in 0..length {
                    let byte = output[output.len() - distance];
                    output.push(byte);
                    reader.work.charge(1)?;
                }
            }
            _ => {
                return Err(CodecError::InvalidDeflate {
                    reason: "reserved literal/length symbol",
                });
            }
        }
    }
}

fn reserve_output(
    output: &[u8],
    additional: usize,
    ceiling: usize,
    limit_kind: LimitKind,
) -> Result<(), CodecError> {
    let requested = output
        .len()
        .checked_add(additional)
        .ok_or(LimitError::ArithmeticOverflow { kind: limit_kind })?;
    if requested > ceiling {
        return Err(LimitError::Exceeded {
            kind: limit_kind,
            requested,
            maximum: ceiling,
        }
        .into());
    }
    Ok(())
}

fn fixed_trees() -> Result<(Huffman, Huffman), CodecError> {
    let mut literal_lengths = vec![0; 288];
    literal_lengths[..144].fill(8);
    literal_lengths[144..256].fill(9);
    literal_lengths[256..280].fill(7);
    literal_lengths[280..].fill(8);
    let distance_lengths = vec![5; 32];
    Ok((
        Huffman::from_lengths(&literal_lengths)?,
        Huffman::from_lengths(&distance_lengths)?,
    ))
}

fn dynamic_trees(reader: &mut BitReader<'_, '_>) -> Result<(Huffman, Huffman), CodecError> {
    const ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let literal_count = reader.read_bits(5)? as usize + 257;
    let distance_count = reader.read_bits(5)? as usize + 1;
    let code_count = reader.read_bits(4)? as usize + 4;
    if literal_count > 286 || distance_count > 32 {
        return Err(CodecError::InvalidDeflate {
            reason: "dynamic tree count exceeds RFC 1951 bounds",
        });
    }

    let mut code_lengths = [0u8; 19];
    for &symbol in &ORDER[..code_count] {
        code_lengths[symbol] = reader.read_bits(3)? as u8;
    }
    let code_tree = Huffman::from_lengths(&code_lengths)?;
    let total = literal_count + distance_count;
    let mut lengths = Vec::with_capacity(total);
    while lengths.len() < total {
        match code_tree.decode(reader)? {
            symbol @ 0..=15 => lengths.push(symbol as u8),
            16 => {
                let previous = *lengths.last().ok_or(CodecError::InvalidDeflate {
                    reason: "repeat code has no previous length",
                })?;
                repeat_length(reader, &mut lengths, total, previous, 3, 2)?;
            }
            17 => repeat_length(reader, &mut lengths, total, 0, 3, 3)?,
            18 => repeat_length(reader, &mut lengths, total, 0, 11, 7)?,
            _ => {
                return Err(CodecError::InvalidDeflate {
                    reason: "invalid code-length symbol",
                });
            }
        }
    }
    if lengths[256] == 0 {
        return Err(CodecError::InvalidDeflate {
            reason: "literal tree omits end-of-block symbol",
        });
    }
    let literal = Huffman::from_lengths(&lengths[..literal_count])?;
    let distance_lengths = &lengths[literal_count..];
    let distance = if distance_lengths.iter().all(|&length| length == 0) {
        // RFC 1951 permits an all-literal dynamic block to declare one
        // zero-bit distance code. It becomes an error only if the stream later
        // attempts to decode a length/distance pair.
        Huffman::empty()
    } else {
        Huffman::from_lengths(distance_lengths)?
    };
    Ok((literal, distance))
}

fn repeat_length(
    reader: &mut BitReader<'_, '_>,
    lengths: &mut Vec<u8>,
    total: usize,
    value: u8,
    base: usize,
    extra_bits: u8,
) -> Result<(), CodecError> {
    let repeat = base + reader.read_bits(extra_bits)? as usize;
    if lengths.len() + repeat > total {
        return Err(CodecError::InvalidDeflate {
            reason: "code-length repeat exceeds declared trees",
        });
    }
    lengths.resize(lengths.len() + repeat, value);
    Ok(())
}

fn length_code(symbol: u16) -> Result<(usize, u8), CodecError> {
    const BASE: [usize; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const EXTRA: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    let index = usize::from(symbol - 257);
    BASE.get(index)
        .copied()
        .zip(EXTRA.get(index).copied())
        .ok_or(CodecError::InvalidDeflate {
            reason: "reserved length code",
        })
}

fn distance_code(symbol: u16) -> Result<(usize, u8), CodecError> {
    const BASE: [usize; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const EXTRA: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];
    let index = usize::from(symbol);
    BASE.get(index)
        .copied()
        .zip(EXTRA.get(index).copied())
        .ok_or(CodecError::InvalidDeflate {
            reason: "reserved distance code",
        })
}

#[derive(Debug)]
struct Huffman {
    counts: [u16; MAX_BITS + 1],
    first_code: [u16; MAX_BITS + 1],
    first_symbol: [usize; MAX_BITS + 1],
    symbols: Vec<u16>,
    maximum_bits: usize,
}

impl Huffman {
    fn empty() -> Self {
        Self {
            counts: [0; MAX_BITS + 1],
            first_code: [0; MAX_BITS + 1],
            first_symbol: [0; MAX_BITS + 1],
            symbols: Vec::new(),
            maximum_bits: 0,
        }
    }

    fn from_lengths(lengths: &[u8]) -> Result<Self, CodecError> {
        let mut counts = [0u16; MAX_BITS + 1];
        for &length in lengths {
            if usize::from(length) > MAX_BITS {
                return Err(CodecError::InvalidDeflate {
                    reason: "Huffman code exceeds 15 bits",
                });
            }
            if length != 0 {
                counts[usize::from(length)] += 1;
            }
        }
        let symbol_count: usize = counts[1..].iter().map(|&value| usize::from(value)).sum();
        if symbol_count == 0 {
            return Err(CodecError::InvalidDeflate {
                reason: "empty Huffman tree",
            });
        }

        let mut left = 1i32;
        for &count in &counts[1..] {
            left = (left << 1) - i32::from(count);
            if left < 0 {
                return Err(CodecError::InvalidDeflate {
                    reason: "oversubscribed Huffman tree",
                });
            }
        }

        let mut first_code = [0u16; MAX_BITS + 1];
        let mut first_symbol = [0usize; MAX_BITS + 1];
        let mut code = 0u16;
        let mut symbol_offset = 0usize;
        for bits in 1..=MAX_BITS {
            code = (code + counts[bits - 1]) << 1;
            first_code[bits] = code;
            first_symbol[bits] = symbol_offset;
            symbol_offset += usize::from(counts[bits]);
        }

        let mut symbols = Vec::with_capacity(symbol_count);
        for bits in 1..=MAX_BITS {
            symbols.extend(
                lengths
                    .iter()
                    .enumerate()
                    .filter(|(_, length)| usize::from(**length) == bits)
                    .map(|(symbol, _)| symbol as u16),
            );
        }
        let maximum_bits = (1..=MAX_BITS).rfind(|&bits| counts[bits] != 0).unwrap_or(1);
        Ok(Self {
            counts,
            first_code,
            first_symbol,
            symbols,
            maximum_bits,
        })
    }

    fn decode(&self, reader: &mut BitReader<'_, '_>) -> Result<u16, CodecError> {
        if self.symbols.is_empty() {
            return Err(CodecError::InvalidDeflate {
                reason: "distance tree is empty",
            });
        }
        let mut code = 0u16;
        for bits in 1..=self.maximum_bits {
            code = (code << 1) | reader.read_bits(1)? as u16;
            let first = self.first_code[bits];
            let offset = code.wrapping_sub(first);
            if code >= first && offset < self.counts[bits] {
                return Ok(self.symbols[self.first_symbol[bits] + usize::from(offset)]);
            }
        }
        Err(CodecError::InvalidDeflate {
            reason: "bit sequence has no Huffman symbol",
        })
    }
}

struct BitReader<'a, 'work> {
    input: &'a [u8],
    bit: usize,
    work: &'work mut WorkBudget,
}

impl<'a, 'work> BitReader<'a, 'work> {
    fn new(input: &'a [u8], work: &'work mut WorkBudget) -> Self {
        Self {
            input,
            bit: 0,
            work,
        }
    }

    fn read_bits(&mut self, count: u8) -> Result<u32, CodecError> {
        self.work.charge(usize::from(count))?;
        let mut value = 0u32;
        for shift in 0..count {
            let byte = *self
                .input
                .get(self.bit / 8)
                .ok_or(CodecError::UnexpectedEnd {
                    stage: CodecStage::Deflate,
                })?;
            value |= u32::from((byte >> (self.bit % 8)) & 1) << shift;
            self.bit += 1;
        }
        Ok(value)
    }

    fn align_byte(&mut self) {
        self.bit = (self.bit + 7) & !7;
    }

    fn read_aligned_byte(&mut self) -> Result<u8, CodecError> {
        debug_assert_eq!(self.bit % 8, 0);
        self.work.charge(1)?;
        let byte = *self
            .input
            .get(self.bit / 8)
            .ok_or(CodecError::UnexpectedEnd {
                stage: CodecStage::Deflate,
            })?;
        self.bit += 8;
        Ok(byte)
    }

    fn read_u16_le(&mut self) -> Result<u16, CodecError> {
        let low = self.read_aligned_byte()?;
        let high = self.read_aligned_byte()?;
        Ok(u16::from_le_bytes([low, high]))
    }

    fn consumed_bytes(&self) -> usize {
        self.bit.div_ceil(8)
    }
}
