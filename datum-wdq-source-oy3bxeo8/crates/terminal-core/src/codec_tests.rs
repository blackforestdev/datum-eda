use crate::{
    Base64Limits, ChecksumKind, ClipboardBytesLimit, CodecError, CodecLimits,
    GraphicDecodedBytesLimit, GraphicPixelsLimit, LimitError, LimitKind, ParserWorkLimit, Rgba8,
    adler32, crc32, decode_base64, decode_deflate, decode_png, decode_zlib,
};

fn limits(decoded: usize, pixels: usize, ratio: usize, work: usize) -> CodecLimits {
    CodecLimits {
        decoded_bytes: GraphicDecodedBytesLimit::new(decoded).unwrap(),
        pixels: GraphicPixelsLimit::new(pixels).unwrap(),
        compression_ratio: crate::CompressionRatioLimit::new(ratio).unwrap(),
        work: ParserWorkLimit::new(work).unwrap(),
    }
}

fn base64_limits(decoded: usize, work: usize) -> Base64Limits {
    Base64Limits::graphics(
        GraphicDecodedBytesLimit::new(decoded).unwrap(),
        ParserWorkLimit::new(work).unwrap(),
    )
}

fn hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => panic!("invalid fixture hex"),
            };
            digit(pair[0]) << 4 | digit(pair[1])
        })
        .collect()
}

fn stored_zlib(payload: &[u8]) -> Vec<u8> {
    let mut encoded = vec![0x78, 0x01];
    if payload.is_empty() {
        encoded.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    } else {
        let chunks = payload.chunks(u16::MAX as usize);
        let count = chunks.len();
        for (index, chunk) in chunks.enumerate() {
            encoded.push(u8::from(index + 1 == count));
            let length = chunk.len() as u16;
            encoded.extend_from_slice(&length.to_le_bytes());
            encoded.extend_from_slice(&(!length).to_le_bytes());
            encoded.extend_from_slice(chunk);
        }
    }
    encoded.extend_from_slice(&adler32(payload).to_be_bytes());
    encoded
}

#[derive(Default)]
struct TestBitWriter {
    bytes: Vec<u8>,
    bit: usize,
}

impl TestBitWriter {
    fn write_lsb(&mut self, value: u32, count: usize) {
        for shift in 0..count {
            self.write_bit(((value >> shift) & 1) != 0);
        }
    }

    fn write_code(&mut self, bits: &[bool]) {
        for &bit in bits {
            self.write_bit(bit);
        }
    }

    fn write_bit(&mut self, value: bool) {
        if self.bit / 8 == self.bytes.len() {
            self.bytes.push(0);
        }
        if value {
            self.bytes[self.bit / 8] |= 1 << (self.bit % 8);
        }
        self.bit += 1;
    }
}

fn all_literal_dynamic_deflate() -> Vec<u8> {
    let mut writer = TestBitWriter::default();
    writer.write_lsb(1, 1); // final block
    writer.write_lsb(2, 2); // dynamic Huffman block
    writer.write_lsb(0, 5); // 257 literal/length codes
    writer.write_lsb(0, 5); // one distance code
    writer.write_lsb(14, 4); // 18 code-length codes
    for length in [0, 0, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2] {
        writer.write_lsb(length, 3);
    }
    writer.write_code(&[true, true]); // code 18: 65 zero lengths
    writer.write_lsb(54, 7);
    writer.write_code(&[true, false]); // code 1: literal 'A'
    writer.write_code(&[true, true]); // code 18: 138 zero lengths
    writer.write_lsb(127, 7);
    writer.write_code(&[true, true]); // code 18: 52 zero lengths
    writer.write_lsb(41, 7);
    writer.write_code(&[true, false]); // code 1: end-of-block
    writer.write_code(&[false]); // code 0: unused distance entry
    writer.write_code(&[false, true]); // literal 'A', then end-of-block
    writer.bytes
}

fn chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
    bytes.extend_from_slice(kind);
    bytes.extend_from_slice(data);
    let mut checksum = crate::Crc32::new();
    checksum.update(kind);
    checksum.update(data);
    bytes.extend_from_slice(&checksum.finish().to_be_bytes());
    bytes
}

fn png(
    header: (u32, u32, u8, u8, u8),
    palette: Option<&[u8]>,
    transparency: Option<&[u8]>,
    filtered: &[u8],
) -> Vec<u8> {
    let (width, height, depth, color_type, interlace) = header;
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[depth, color_type, 0, 0, interlace]);
    bytes.extend(chunk(b"IHDR", &header));
    if let Some(palette) = palette {
        bytes.extend(chunk(b"PLTE", palette));
    }
    if let Some(transparency) = transparency {
        bytes.extend(chunk(b"tRNS", transparency));
    }
    bytes.extend(chunk(b"IDAT", &stored_zlib(filtered)));
    bytes.extend(chunk(b"IEND", &[]));
    bytes
}

#[test]
fn rfc4648_base64_vectors_are_strict_and_canonical() {
    for (encoded, decoded) in [
        (b"".as_slice(), b"".as_slice()),
        (b"Zg==", b"f"),
        (b"Zm8=", b"fo"),
        (b"Zm9v", b"foo"),
        (b"Zm9vYg==", b"foob"),
        (b"Zm9vYmE=", b"fooba"),
        (b"Zm9vYmFy", b"foobar"),
    ] {
        assert_eq!(
            decode_base64(encoded, base64_limits(64, 1_000)).unwrap(),
            decoded
        );
    }
    assert!(matches!(
        decode_base64(b"Zh==", base64_limits(64, 1_000)),
        Err(CodecError::NonCanonicalBase64 { .. })
    ));
    assert!(matches!(
        decode_base64(b"Zm9v\n", base64_limits(64, 1_000)),
        Err(CodecError::InvalidBase64Padding { .. })
    ));
    assert!(matches!(
        decode_base64(b"Zm=v", base64_limits(64, 1_000)),
        Err(CodecError::InvalidBase64 { .. })
    ));
}

#[test]
fn base64_output_and_work_limits_reject_without_a_prefix() {
    assert!(matches!(
        decode_base64(b"Zm9v", base64_limits(2, 100)),
        Err(CodecError::Limit(LimitError::Exceeded {
            kind: LimitKind::GraphicDecodedBytes,
            ..
        }))
    ));
    assert!(matches!(
        decode_base64(b"Zm9v", base64_limits(64, 3)),
        Err(CodecError::Limit(LimitError::Exceeded {
            kind: LimitKind::ParserWork,
            ..
        }))
    ));
    let clipboard_limits = Base64Limits::clipboard(
        ClipboardBytesLimit::new(2).unwrap(),
        ParserWorkLimit::new(100).unwrap(),
    );
    assert!(matches!(
        decode_base64(b"Zm9v", clipboard_limits),
        Err(CodecError::Limit(LimitError::Exceeded {
            kind: LimitKind::ClipboardBytes,
            requested: 3,
            maximum: 2,
        }))
    ));
}

#[test]
fn checksum_vectors_and_incremental_updates_are_exact() {
    assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    assert_eq!(adler32(b"123456789"), 0x091e_01de);
    let mut crc = crate::Crc32::new();
    let mut adler = crate::Adler32::new();
    for chunk in b"123456789".chunks(2) {
        crc.update(chunk);
        adler.update(chunk);
    }
    assert_eq!(crc.finish(), 0xcbf4_3926);
    assert_eq!(adler.finish(), 0x091e_01de);
}

#[test]
fn zlib_decodes_stored_fixed_and_dynamic_deflate_blocks() {
    let policy = limits(32_768, 1, 1_000, 2_000_000);
    let stored = stored_zlib(b"stored block");
    assert_eq!(decode_zlib(&stored, policy).unwrap(), b"stored block");

    let fixed = hex("78dacb48cdc9c957c84090003a2e067d");
    assert_eq!(decode_zlib(&fixed, policy).unwrap(), b"hello hello hello");

    let dynamic = hex("78daedc1010d000000c2a06cef5fca1e0e28000000e0dd00ffad103d");
    assert_eq!(decode_zlib(&dynamic, policy).unwrap(), vec![b'A'; 4_096]);

    let all_literal = all_literal_dynamic_deflate();
    assert_eq!(decode_deflate(&all_literal, policy).unwrap().bytes, b"A");
}

#[test]
fn zlib_rejects_headers_checksums_trailing_data_bombs_and_truncation() {
    let policy = limits(32_768, 1, 1_000, 2_000_000);
    let mut checksum = stored_zlib(b"payload");
    *checksum.last_mut().unwrap() ^= 1;
    assert!(matches!(
        decode_zlib(&checksum, policy),
        Err(CodecError::ChecksumMismatch {
            kind: ChecksumKind::Adler32,
            ..
        })
    ));

    let mut header = stored_zlib(b"payload");
    header[0] = 0;
    assert!(matches!(
        decode_zlib(&header, policy),
        Err(CodecError::InvalidZlib { .. })
    ));

    let dynamic = hex("78daedc1010d000000c2a06cef5fca1e0e28000000e0dd00ffad103d");
    let bomb_policy = limits(32_768, 1, 10, 2_000_000);
    assert!(matches!(
        decode_zlib(&dynamic, bomb_policy),
        Err(CodecError::Limit(LimitError::Exceeded {
            kind: LimitKind::CompressionRatio,
            ..
        }))
    ));
    for end in 0..dynamic.len() {
        assert!(decode_zlib(&dynamic[..end], policy).is_err());
    }

    assert!(matches!(
        decode_zlib(&[0x78, 0x20], policy),
        Err(CodecError::UnsupportedZlib { .. })
    ));
    assert!(matches!(
        decode_deflate(&[0x07], policy),
        Err(CodecError::InvalidDeflate {
            reason: "reserved block type"
        })
    ));
    assert!(matches!(
        decode_deflate(&[0x01, 0x01, 0x00, 0xff, 0xff, b'x'], policy),
        Err(CodecError::InvalidDeflate {
            reason: "stored block length complement mismatch"
        })
    ));
    assert!(matches!(
        decode_zlib(&stored_zlib(b"work"), limits(32_768, 1, 1_000, 4)),
        Err(CodecError::Limit(LimitError::Exceeded {
            kind: LimitKind::ParserWork,
            ..
        }))
    ));
}

#[test]
fn png_all_filters_reconstruct_truecolor_rows_exactly() {
    let raw_rows: Vec<Vec<u8>> = (0..5)
        .map(|row| {
            (0..9)
                .map(|column| (row * 31 + column * 7 + 3) as u8)
                .collect()
        })
        .collect();
    let mut filtered = Vec::new();
    for (index, raw) in raw_rows.iter().enumerate() {
        let previous = index
            .checked_sub(1)
            .map_or(&[][..], |row| raw_rows[row].as_slice());
        filtered.push(index as u8);
        filtered.extend(encode_filter(index as u8, raw, previous, 3));
    }
    let image = decode_png(
        &png((3, 5, 8, 2, 0), None, None, &filtered),
        limits(1_000_000, 1_000, 100, 1_000_000),
    )
    .unwrap();
    let expected: Vec<Rgba8> = raw_rows
        .iter()
        .flat_map(|row| row.chunks_exact(3))
        .map(|pixel| Rgba8 {
            red: pixel[0],
            green: pixel[1],
            blue: pixel[2],
            alpha: 255,
        })
        .collect();
    assert_eq!(image.pixels, expected);
}

#[test]
fn png_color_types_depths_palette_and_transparency_decode_to_rgba8() {
    let policy = limits(1_000_000, 1_000, 100, 1_000_000);
    let grayscale = decode_png(
        &png((4, 1, 2, 0, 0), None, Some(&[0, 2]), &[0, 0x1b]),
        policy,
    )
    .unwrap();
    assert_eq!(
        grayscale.pixels,
        vec![
            Rgba8 {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 255
            },
            Rgba8 {
                red: 85,
                green: 85,
                blue: 85,
                alpha: 255
            },
            Rgba8 {
                red: 170,
                green: 170,
                blue: 170,
                alpha: 0
            },
            Rgba8 {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255
            },
        ]
    );

    let palette = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
    let indexed = decode_png(
        &png(
            (4, 1, 2, 3, 0),
            Some(&palette),
            Some(&[255, 128, 64]),
            &[0, 0x1b],
        ),
        policy,
    )
    .unwrap();
    assert_eq!(indexed.pixels[1].alpha, 128);
    assert_eq!(
        indexed.pixels[2],
        Rgba8 {
            red: 0,
            green: 0,
            blue: 255,
            alpha: 64
        }
    );

    let grey_alpha = decode_png(&png((1, 1, 8, 4, 0), None, None, &[0, 100, 64]), policy).unwrap();
    assert_eq!(
        grey_alpha.pixels[0],
        Rgba8 {
            red: 100,
            green: 100,
            blue: 100,
            alpha: 64
        }
    );

    let rgba16 = decode_png(
        &png(
            (1, 1, 16, 6, 0),
            None,
            None,
            &[0, 0xff, 0xff, 0, 0, 0x80, 0, 0x40, 0],
        ),
        policy,
    )
    .unwrap();
    assert_eq!(
        rgba16.pixels[0],
        Rgba8 {
            red: 255,
            green: 0,
            blue: 128,
            alpha: 64
        }
    );

    for (depth, sample) in [
        (1, vec![0, 0x80]),
        (2, vec![0, 0xc0]),
        (4, vec![0, 0xf0]),
        (8, vec![0, 0xff]),
        (16, vec![0, 0xff, 0xff]),
    ] {
        let decoded = decode_png(&png((1, 1, depth, 0, 0), None, None, &sample), policy).unwrap();
        assert_eq!(
            decoded.pixels[0],
            Rgba8 {
                red: 255,
                green: 255,
                blue: 255,
                alpha: 255,
            }
        );
    }

    for (depth, sample) in [
        (1, vec![0, 0x80]),
        (2, vec![0, 0x40]),
        (4, vec![0, 0x10]),
        (8, vec![0, 0x01]),
    ] {
        let decoded = decode_png(
            &png(
                (1, 1, depth, 3, 0),
                Some(&[0, 0, 0, 10, 20, 30]),
                None,
                &sample,
            ),
            policy,
        )
        .unwrap();
        assert_eq!(
            decoded.pixels[0],
            Rgba8 {
                red: 10,
                green: 20,
                blue: 30,
                alpha: 255,
            }
        );
    }
}

#[test]
fn adam7_interlace_places_every_pixel_in_reference_order() {
    const PASSES: [(usize, usize, usize, usize); 7] = [
        (0, 0, 8, 8),
        (4, 0, 8, 8),
        (0, 4, 4, 8),
        (2, 0, 4, 4),
        (0, 2, 2, 4),
        (1, 0, 2, 2),
        (0, 1, 1, 2),
    ];
    let pixels: Vec<Rgba8> = (0..64)
        .map(|index| Rgba8 {
            red: index,
            green: 255 - index,
            blue: index.wrapping_mul(3),
            alpha: 255,
        })
        .collect();
    let mut filtered = Vec::new();
    for (x0, y0, dx, dy) in PASSES {
        for y in (y0..8).step_by(dy) {
            filtered.push(0);
            for x in (x0..8).step_by(dx) {
                let pixel = pixels[y * 8 + x];
                filtered.extend_from_slice(&[pixel.red, pixel.green, pixel.blue, pixel.alpha]);
            }
        }
    }
    let image = decode_png(
        &png((8, 8, 8, 6, 1), None, None, &filtered),
        limits(1_000_000, 1_000, 100, 1_000_000),
    )
    .unwrap();
    assert_eq!(image.pixels, pixels);
}

#[test]
fn png_crc_order_palette_filter_and_resource_errors_fail_closed() {
    let policy = limits(1_000_000, 1_000, 100, 1_000_000);
    let mut corrupt = png((1, 1, 8, 6, 0), None, None, &[0, 1, 2, 3, 4]);
    corrupt[29] ^= 1;
    assert!(matches!(
        decode_png(&corrupt, policy),
        Err(CodecError::ChecksumMismatch { .. })
    ));

    let missing_palette = png((1, 1, 8, 3, 0), None, None, &[0, 0]);
    assert!(matches!(
        decode_png(&missing_palette, policy),
        Err(CodecError::InvalidPng { .. })
    ));
    let bad_filter = png((1, 1, 8, 6, 0), None, None, &[5, 1, 2, 3, 4]);
    assert!(matches!(
        decode_png(&bad_filter, policy),
        Err(CodecError::InvalidPng { .. })
    ));
    let pixel_limited = png((2, 2, 8, 6, 0), None, None, &[0; 18]);
    assert!(matches!(
        decode_png(&pixel_limited, limits(1_000_000, 3, 100, 1_000_000)),
        Err(CodecError::Limit(LimitError::Exceeded {
            kind: LimitKind::GraphicPixels,
            ..
        }))
    ));

    let valid = png((1, 1, 8, 6, 0), None, None, &[0, 1, 2, 3, 4]);
    let mut unknown_critical = valid[..33].to_vec();
    unknown_critical.extend(chunk(b"ABCD", &[]));
    unknown_critical.extend_from_slice(&valid[33..]);
    assert!(matches!(
        decode_png(&unknown_critical, policy),
        Err(CodecError::UnsupportedPng { .. })
    ));

    let mut reserved_bit = valid[..33].to_vec();
    reserved_bit.extend(chunk(b"abca", &[]));
    reserved_bit.extend_from_slice(&valid[33..]);
    assert!(matches!(
        decode_png(&reserved_bit, policy),
        Err(CodecError::InvalidPng { .. })
    ));

    let encoded = stored_zlib(&[0, 1, 2, 3, 4]);
    let midpoint = encoded.len() / 2;
    let mut separated_idat = valid[..33].to_vec();
    separated_idat.extend(chunk(b"IDAT", &encoded[..midpoint]));
    separated_idat.extend(chunk(b"tEXt", b"separator"));
    separated_idat.extend(chunk(b"IDAT", &encoded[midpoint..]));
    separated_idat.extend(chunk(b"IEND", &[]));
    assert!(matches!(
        decode_png(&separated_idat, policy),
        Err(CodecError::InvalidPng { .. })
    ));

    let truecolor = png((1, 1, 8, 2, 0), None, None, &[0, 1, 2, 3]);
    let mut palette_after_transparency = truecolor[..33].to_vec();
    palette_after_transparency.extend(chunk(b"tRNS", &[0; 6]));
    palette_after_transparency.extend(chunk(b"PLTE", &[1, 2, 3]));
    palette_after_transparency.extend_from_slice(&truecolor[33..]);
    assert!(matches!(
        decode_png(&palette_after_transparency, policy),
        Err(CodecError::InvalidPng { .. })
    ));

    assert!(matches!(
        decode_png(&valid, limits(1_000_000, 1_000, 100, 8)),
        Err(CodecError::Limit(LimitError::Exceeded {
            kind: LimitKind::ParserWork,
            ..
        }))
    ));
}

#[test]
fn hostile_png_prefixes_and_mutations_never_escape_bounded_errors() {
    let valid = png((1, 1, 8, 6, 0), None, None, &[0, 9, 8, 7, 6]);
    let policy = limits(16_384, 64, 100, 100_000);
    for end in 0..valid.len() {
        assert!(decode_png(&valid[..end], policy).is_err());
    }
    for index in 0..valid.len() {
        let mut mutated = valid.clone();
        mutated[index] ^= 0x5a;
        let _ = decode_png(&mutated, policy);
    }
    assert_eq!(decode_png(&valid, policy).unwrap().pixels[0].alpha, 6);
}

fn encode_filter(filter: u8, raw: &[u8], previous: &[u8], bpp: usize) -> Vec<u8> {
    raw.iter()
        .enumerate()
        .map(|(index, &byte)| {
            let left = index.checked_sub(bpp).map_or(0, |at| raw[at]);
            let above = previous.get(index).copied().unwrap_or(0);
            let upper_left = index
                .checked_sub(bpp)
                .and_then(|at| previous.get(at))
                .copied()
                .unwrap_or(0);
            let predictor = match filter {
                0 => 0,
                1 => left,
                2 => above,
                3 => ((u16::from(left) + u16::from(above)) / 2) as u8,
                4 => test_paeth(left, above, upper_left),
                _ => unreachable!(),
            };
            byte.wrapping_sub(predictor)
        })
        .collect()
}

fn test_paeth(left: u8, above: u8, upper_left: u8) -> u8 {
    let estimate = i32::from(left) + i32::from(above) - i32::from(upper_left);
    let candidates = [left, above, upper_left];
    *candidates
        .iter()
        .min_by_key(|&&value| (estimate - i32::from(value)).abs())
        .unwrap()
}
