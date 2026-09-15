use crate::Rgba8;
use crate::kitty_protocol::{KittyAction, KittyGraphicsError};

pub(crate) fn valid_continuation_header(header: &[u8], action: KittyAction) -> bool {
    header.split(|byte| *byte == b',').all(|field| {
        matches!(field, b"m=0" | b"m=1" | b"q=0" | b"q=1" | b"q=2")
            || (action == KittyAction::Frame && field == b"a=f")
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn composite_block(
    canvas: &mut [Rgba8],
    width: u32,
    height: u32,
    block: &[Rgba8],
    block_width: u32,
    block_height: u32,
    x: u32,
    y: u32,
    replace: bool,
) -> Result<(), KittyGraphicsError> {
    if x.saturating_add(block_width) > width || y.saturating_add(block_height) > height {
        return Err(invalid("frame rectangle is out of bounds"));
    }
    for row in 0..block_height {
        for column in 0..block_width {
            let source = block[(row * block_width + column) as usize];
            let destination = &mut canvas[((y + row) * width + x + column) as usize];
            *destination = if replace {
                source
            } else {
                alpha_over(source, *destination)
            };
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn copy_rectangle(
    source: &[Rgba8],
    destination: &mut [Rgba8],
    image_width: u32,
    image_height: u32,
    source_x: u32,
    source_y: u32,
    destination_x: u32,
    destination_y: u32,
    width: u32,
    height: u32,
    replace: bool,
) -> Result<(), KittyGraphicsError> {
    if source_x.saturating_add(width) > image_width
        || source_y.saturating_add(height) > image_height
        || destination_x.saturating_add(width) > image_width
        || destination_y.saturating_add(height) > image_height
    {
        return Err(invalid("composition rectangle is out of bounds"));
    }
    for row in 0..height {
        for column in 0..width {
            let pixel = source[((source_y + row) * image_width + source_x + column) as usize];
            let target = &mut destination
                [((destination_y + row) * image_width + destination_x + column) as usize];
            *target = if replace {
                pixel
            } else {
                alpha_over(pixel, *target)
            };
        }
    }
    Ok(())
}

pub(crate) fn rgba(value: u32) -> Rgba8 {
    let [red, green, blue, alpha] = value.to_be_bytes();
    Rgba8 {
        red,
        green,
        blue,
        alpha,
    }
}

pub(crate) fn rectangles_overlap(
    ax: u32,
    ay: u32,
    bx: u32,
    by: u32,
    width: u32,
    height: u32,
) -> bool {
    ax < bx.saturating_add(width)
        && bx < ax.saturating_add(width)
        && ay < by.saturating_add(height)
        && by < ay.saturating_add(height)
}

fn alpha_over(source: Rgba8, destination: Rgba8) -> Rgba8 {
    let alpha = u32::from(source.alpha);
    let inverse = 255 - alpha;
    Rgba8 {
        red: ((u32::from(source.red) * alpha + u32::from(destination.red) * inverse + 127) / 255)
            as u8,
        green: ((u32::from(source.green) * alpha + u32::from(destination.green) * inverse + 127)
            / 255) as u8,
        blue: ((u32::from(source.blue) * alpha + u32::from(destination.blue) * inverse + 127) / 255)
            as u8,
        alpha: (alpha + (u32::from(destination.alpha) * inverse + 127) / 255).min(255) as u8,
    }
}

fn invalid(reason: &'static str) -> KittyGraphicsError {
    KittyGraphicsError::Malformed { reason }
}
