//! Borrow decoded RGBA pixels; bound each queue write without an image-sized copy.
use datum_terminal_core::Rgba8;

const MAX_WRITE_BYTES: usize = 256 * 1024;
// The Datum-owned type declares repr(C); freeze every layout fact relied upon here.
const _: () = {
    assert!(std::mem::size_of::<Rgba8>() == 4);
    assert!(std::mem::align_of::<Rgba8>() == 1);
    assert!(std::mem::offset_of!(Rgba8, red) == 0);
    assert!(std::mem::offset_of!(Rgba8, green) == 1);
    assert!(std::mem::offset_of!(Rgba8, blue) == 2);
    assert!(std::mem::offset_of!(Rgba8, alpha) == 3);
};

fn rgba_bytes(pixels: &[Rgba8]) -> &[u8] {
    // SAFETY: repr(C) and the assertions above establish four initialized u8
    // channels with no padding. The immutable byte view has exactly the input
    // slice's extent and lifetime; it neither owns nor mutates the pixel storage.
    unsafe { std::slice::from_raw_parts(pixels.as_ptr().cast(), std::mem::size_of_val(pixels)) }
}

pub(super) fn chunks(
    pixels: &[Rgba8],
    width: u32,
    height: u32,
    mut write: impl FnMut(u32, u32, u32, u32, &[u8]),
) {
    let width = width as usize;
    assert_eq!(pixels.len(), width * height as usize);
    if pixels.is_empty() {
        return;
    }
    let bytes = rgba_bytes(pixels);
    let max_pixels = MAX_WRITE_BYTES / 4;
    if width <= max_pixels {
        let rows = max_pixels / width;
        for (index, part) in bytes.chunks(rows * width * 4).enumerate() {
            write(
                0,
                (index * rows) as u32,
                width as u32,
                (part.len() / (width * 4)) as u32,
                part,
            );
        }
    } else {
        // Even a future device with exceptionally wide textures stays bounded.
        for (y, row) in bytes.chunks(width * 4).enumerate() {
            for (x, part) in row.chunks(MAX_WRITE_BYTES).enumerate() {
                write(
                    (x * max_pixels) as u32,
                    y as u32,
                    (part.len() / 4) as u32,
                    1,
                    part,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_chunks_preserve_channels_and_cover_odd_rows_and_wide_tails() {
        for (width, height) in [(0, 0), (1, 1), (257, 513), (70_001, 3)] {
            let pixels: Vec<_> = (0..width * height)
                .map(|i| Rgba8 {
                    red: i as u8,
                    green: (i / 7) as u8,
                    blue: (i / 257) as u8,
                    alpha: 255 - i as u8,
                })
                .collect();
            let expected: Vec<_> = pixels
                .iter()
                .flat_map(|p| [p.red, p.green, p.blue, p.alpha])
                .collect();
            let mut actual = vec![0; expected.len()];
            let mut coverage = vec![0_u8; pixels.len()];
            chunks(&pixels, width, height, |x, y, w, h, bytes| {
                assert!(bytes.len() <= MAX_WRITE_BYTES);
                let start = (y as usize * width as usize + x as usize) * 4;
                assert_eq!(
                    bytes.as_ptr(),
                    rgba_bytes(&pixels)[start..].as_ptr(),
                    "must borrow source storage"
                );
                for row in 0..h as usize {
                    let dest = ((y as usize + row) * width as usize + x as usize) * 4;
                    let source = row * w as usize * 4;
                    actual[dest..dest + w as usize * 4]
                        .copy_from_slice(&bytes[source..source + w as usize * 4]);
                    for item in &mut coverage[dest / 4..dest / 4 + w as usize] {
                        *item += 1;
                    }
                }
            });
            assert_eq!(actual, expected);
            assert!(coverage.iter().all(|count| *count == 1));
        }
    }
}
