use super::*;

fn limits(objects: usize, pixels: usize, frames: usize) -> CoreLimits {
    CoreLimits::try_from(CoreLimitValues {
        parameter_count: 64,
        parameter_digits: 32,
        parameter_value: usize::MAX,
        subparameter_count: 64,
        intermediate_bytes: 64,
        control_string_bytes: 1 << 20,
        cluster_bytes: 4_096,
        title_bytes: 4_096,
        working_directory_bytes: 4_096,
        clipboard_bytes: 1 << 20,
        hyperlink_bytes: 1 << 20,
        input_bytes: 1 << 20,
        keyboard_stack: 64,
        notification_bytes: 4_096,
        reply_bytes: 4_096,
        pending_events: 1_024,
        pending_damage: 1_024,
        history_lines: 64,
        history_bytes: 1 << 20,
        graphic_objects: objects,
        graphic_pixels: pixels,
        graphic_decoded_bytes: pixels * 4,
        graphic_frames: frames,
        compression_ratio: 1_024,
        parser_work: 1 << 20,
        search_work: 1 << 20,
        reflow_work: 1 << 20,
        screen_cells: 1 << 20,
        snapshot_cells: 1 << 20,
    })
    .unwrap()
}

fn core() -> TerminalCore {
    TerminalCore::new(
        limits(128, 1 << 16, 64),
        TerminalSize::new(20, 8, 200, 80).unwrap(),
    )
    .unwrap()
}

fn apc(command: impl AsRef<[u8]>) -> Action {
    Action::ControlString(ControlString {
        kind: ControlStringKind::Apc,
        bytes: command.as_ref().to_vec(),
        terminator: StringTerminator::StringTerminator,
    })
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::new();
    for chunk in bytes.chunks(3) {
        let value = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        encoded.push(TABLE[((value >> 18) & 63) as usize] as char);
        encoded.push(TABLE[((value >> 12) & 63) as usize] as char);
        encoded.push(if chunk.len() > 1 {
            TABLE[((value >> 6) & 63) as usize] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            TABLE[(value & 63) as usize] as char
        } else {
            '='
        });
    }
    encoded
}

fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Rgba8 {
    Rgba8 {
        red,
        green,
        blue,
        alpha,
    }
}

fn reply(update: &CoreUpdate) -> &[u8] {
    update.replies().last().unwrap().bytes()
}

#[test]
fn direct_rgb_rgba_query_and_image_number_replies_are_exact() {
    let mut terminal = core();
    let query = terminal.apply(apc("Ga=q,f=24,s=1,v=1,i=77;AQID")).unwrap();
    assert_eq!(reply(&query), b"\x1b_Gi=77;OK\x1b\\");
    assert_eq!(terminal.state().kitty_images().len(), 0);

    let upload = terminal
        .apply(apc("Ga=t,f=32,s=1,v=1,I=42;/wAA/w=="))
        .unwrap();
    let image = terminal.state().kitty_images().next().unwrap();
    assert_eq!(image.number(), Some(42));
    assert_eq!(image.frames()[0].pixels(), &[rgba(255, 0, 0, 255)]);
    assert_eq!(
        reply(&upload),
        format!("\x1b_Gi={},I=42;OK\x1b\\", image.id().get()).as_bytes()
    );
}

#[test]
fn chunked_transfer_is_atomic_and_uses_final_cursor_position() {
    let mut terminal = core();
    let first = terminal
        .apply(apc("Ga=T,f=32,s=2,v=1,i=9,p=4,m=1;/wAA"))
        .unwrap();
    assert!(first.replies().is_empty());
    assert_eq!(terminal.state().kitty_images().len(), 0);
    terminal
        .reduce(ScreenAction::SetCursor { row: 2, column: 3 })
        .unwrap();
    terminal.apply(apc("Gm=0;/wD/AP8=")).unwrap();
    let placement = terminal.state().graphics().next().unwrap();
    assert_eq!(placement.kitty_image_id().unwrap().get(), 9);
    assert_eq!(placement.kitty_placement_id().unwrap().get(), 4);
    assert_eq!(
        placement.pixels(),
        &[rgba(255, 0, 0, 255), rgba(0, 255, 0, 255)]
    );
    assert_eq!(
        terminal.state().resolve_graphic(placement.id()),
        GraphicAnchorResolution::Screen {
            row: 2,
            column: 3,
            visible_pixel_width: 2,
            visible_pixel_height: 1,
        }
    );
}

#[test]
fn metadata_order_is_irrelevant_and_put_replaces_named_placement() {
    let mut terminal = TerminalCore::new(
        limits(2, 1 << 16, 64),
        TerminalSize::new(20, 8, 200, 80).unwrap(),
    )
    .unwrap();
    terminal
        .apply(apc("GC=1,p=7,c=3,r=2,z=-4,a=T,i=5,f=32,s=1,v=1;/wAA/w=="))
        .unwrap();
    assert_eq!(
        terminal.state().cursor().position,
        CellPoint::new(0, 0, terminal.state().size()).unwrap()
    );
    let first = terminal.state().graphics().next().unwrap();
    assert_eq!(first.z_index(), -4);
    assert_eq!(
        first.cell_extent(),
        GraphicCellExtent {
            columns: 3,
            rows: 2
        }
    );

    terminal
        .reduce(ScreenAction::SetCursor { row: 3, column: 4 })
        .unwrap();
    terminal.apply(apc("Ga=p,i=5,p=7,C=1,X=2,Y=3")).unwrap();
    assert_eq!(terminal.state().graphics().len(), 1);
    let replaced = terminal.state().graphics().next().unwrap();
    assert_eq!(replaced.pixel_offset(), GraphicPixelOffset { x: 2, y: 3 });
    assert_eq!(
        terminal.state().resolve_graphic(replaced.id()),
        GraphicAnchorResolution::Screen {
            row: 3,
            column: 4,
            visible_pixel_width: 1,
            visible_pixel_height: 1,
        }
    );
}

#[test]
fn placement_crop_virtual_relative_cycle_and_parent_lifetime_are_bounded() {
    let mut terminal = core();
    terminal
        .apply(apc("Ga=t,f=32,s=1,v=1,i=1;/wAA/w=="))
        .unwrap();
    terminal
        .apply(apc("Ga=t,f=32,s=1,v=1,i=2;AAD//w=="))
        .unwrap();
    terminal.apply(apc("Ga=p,i=1,p=1,U=1,c=4,r=3")).unwrap();
    terminal
        .apply(apc("Ga=p,i=2,p=2,P=1,Q=1,H=-2,V=3,x=0,y=0,w=1,h=1"))
        .unwrap();
    let relative = terminal.state().graphics().last().unwrap();
    assert_eq!(relative.source().width, 1);
    assert_eq!(relative.parent().unwrap().horizontal_cells, -2);

    let cycle = terminal.apply(apc("Ga=p,i=1,p=1,P=2,Q=2")).unwrap();
    assert!(reply(&cycle).windows(6).any(|part| part == b"ECYCLE"));
    terminal.apply(apc("Ga=d,d=I,i=1,p=1")).unwrap();
    assert_eq!(terminal.state().graphics().len(), 0);
}

#[test]
fn animation_frames_composition_and_deterministic_tick_update_placements() {
    let mut terminal = core();
    terminal
        .apply(apc("Ga=T,f=32,s=2,v=1,i=3,p=1,C=1;/wAA/wD/AP8="))
        .unwrap();
    terminal
        .apply(apc("Ga=f,f=32,s=1,v=1,i=3,x=1,y=0,z=10,X=1;AAD//w=="))
        .unwrap();
    let image = terminal.state().kitty_images().next().unwrap();
    assert_eq!(image.frames().len(), 2);
    assert_eq!(image.frames()[1].pixels()[1], rgba(0, 0, 255, 255));

    terminal.apply(apc("Ga=a,i=3,s=3,c=2,r=1,z=5")).unwrap();
    assert_eq!(
        terminal.state().graphics().next().unwrap().pixels()[1],
        rgba(0, 0, 255, 255)
    );
    terminal
        .apply(apc("Ga=c,i=3,r=2,c=1,x=1,y=0,X=0,Y=0,w=1,h=1,C=1"))
        .unwrap();
    terminal.apply(apc("Ga=a,i=3,c=1")).unwrap();
    assert_eq!(
        terminal.state().graphics().next().unwrap().pixels()[0],
        rgba(0, 0, 255, 255)
    );
    assert!(!terminal.advance_kitty_animations(5).unwrap().recognized());
}

#[test]
fn animation_defaults_missing_frame_gap_to_forty_milliseconds_and_rejects_bad_indices() {
    let mut terminal = core();
    terminal
        .apply(apc("Ga=t,f=32,s=1,v=1,i=30;/wAA/w=="))
        .unwrap();
    terminal
        .apply(apc("Ga=f,f=32,s=1,v=1,i=30;AAD//w=="))
        .unwrap();
    let image = terminal.state().kitty_images().next().unwrap();
    assert_eq!(image.frames()[1].gap_milliseconds(), 40);

    let bad_frame = terminal.apply(apc("Ga=a,i=30,c=99")).unwrap();
    assert!(reply(&bad_frame).windows(6).any(|part| part == b"ENOENT"));
    let bad_gap = terminal.apply(apc("Ga=a,i=30,r=99,z=10")).unwrap();
    assert!(reply(&bad_gap).windows(6).any(|part| part == b"ENOENT"));
}

#[test]
fn soft_and_hard_delete_obey_placement_and_image_lifetimes() {
    let mut terminal = core();
    terminal
        .apply(apc("Ga=T,f=32,s=1,v=1,i=11,p=1,C=1;/wAA/w=="))
        .unwrap();
    terminal.apply(apc("Ga=p,i=11,p=2,C=1,z=8")).unwrap();
    terminal.apply(apc("Ga=d,d=i,i=11,p=1")).unwrap();
    assert_eq!(terminal.state().graphics().len(), 1);
    assert_eq!(terminal.state().kitty_images().len(), 1);
    terminal.apply(apc("Ga=d,d=Z,z=8")).unwrap();
    assert_eq!(terminal.state().graphics().len(), 0);
    assert_eq!(terminal.state().kitty_images().len(), 0);
}

#[test]
fn unsupported_external_transfers_and_quiet_modes_are_safe() {
    let mut terminal = core();
    let error = terminal
        .apply(apc("Ga=q,t=f,f=100,i=7;L3RtcC9pbWFnZS5wbmc="))
        .unwrap();
    assert!(reply(&error).windows(7).any(|part| part == b"ENOTSUP"));
    assert_eq!(terminal.state().kitty_images().len(), 0);
    let quiet = terminal
        .apply(apc("Ga=q,t=s,f=32,s=1,v=1,i=8,q=2;bmFtZQ=="))
        .unwrap();
    assert!(quiet.replies().is_empty());
}

#[test]
fn unicode_placeholders_resolve_color_ids_diacritics_and_inheritance() {
    let mut terminal = core();
    terminal
        .apply(apc("Ga=t,f=32,s=1,v=1,i=42;/wAA/w=="))
        .unwrap();
    terminal.apply(apc("Ga=p,i=42,p=7,U=1,c=3,r=2")).unwrap();
    let style = CellStyle {
        foreground: Color::Indexed(PaletteIndex::new(42)),
        underline_color: Color::Indexed(PaletteIndex::new(7)),
        ..CellStyle::default()
    };
    terminal.reduce(ScreenAction::SetStyle(style)).unwrap();
    terminal
        .reduce(ScreenAction::Print(
            Cluster::new(
                "\u{10eeee}\u{0305}\u{030d}",
                CellWidth::One,
                terminal.limits.cluster_bytes,
            )
            .unwrap(),
        ))
        .unwrap();
    terminal
        .reduce(ScreenAction::Print(
            Cluster::new("\u{10eeee}", CellWidth::One, terminal.limits.cluster_bytes).unwrap(),
        ))
        .unwrap();

    assert_eq!(
        terminal.state().kitty_placeholder(0, 0),
        Some(KittyPlaceholder {
            image_id: KittyImageId::new(42).unwrap(),
            placement_id: KittyPlacementId::new(7),
            row: 0,
            column: 1,
        })
    );
    assert_eq!(terminal.state().kitty_placeholder(0, 1).unwrap().column, 2);
}

#[test]
fn malformed_interleaving_and_limits_leave_prior_state_unchanged() {
    let mut terminal =
        TerminalCore::new(limits(3, 1, 2), TerminalSize::new(8, 4, 80, 40).unwrap()).unwrap();
    terminal
        .apply(apc("Ga=t,f=32,s=1,v=1,i=1;/wAA/w=="))
        .unwrap();
    terminal
        .apply(apc("Ga=t,f=32,s=1,v=1,i=2,m=1;/wAA"))
        .unwrap();
    let interrupted = terminal.apply(apc("Ga=d,d=a")).unwrap();
    assert!(interrupted.recognized());
    let over = terminal
        .apply(apc("Ga=t,f=32,s=2,v=1,i=3;/wAA/wD/AP8="))
        .unwrap();
    assert!(reply(&over).windows(6).any(|part| part == b"ENOSPC"));
    assert_eq!(terminal.state().kitty_images().len(), 1);
    assert_eq!(
        terminal.state().kitty_images().next().unwrap().id().get(),
        1
    );
}

#[test]
fn aggregate_frame_limit_is_atomic_across_sixel_and_kitty_graphics() {
    let mut terminal =
        TerminalCore::new(limits(8, 64, 1), TerminalSize::new(8, 4, 80, 40).unwrap()).unwrap();
    terminal
        .apply(Action::ControlString(ControlString {
            kind: ControlStringKind::Dcs,
            bytes: b"0;1q#2@".to_vec(),
            terminator: StringTerminator::StringTerminator,
        }))
        .unwrap();
    let rejected = terminal
        .apply(apc("Ga=t,f=32,s=1,v=1,i=31;/wAA/w=="))
        .unwrap();
    assert!(reply(&rejected).windows(6).any(|part| part == b"ENOSPC"));
    assert_eq!(terminal.state().graphics().len(), 1);
    assert_eq!(terminal.state().kitty_images().len(), 0);
}

#[test]
fn graphics_survive_history_reflow_and_buffer_teardown() {
    let mut terminal = core();
    terminal
        .apply(apc("Ga=T,f=32,s=1,v=1,i=4,p=1,C=1;/wAA/w=="))
        .unwrap();
    let id = terminal.state().graphics().next().unwrap().id();
    for _ in 0..10 {
        terminal
            .apply(Action::Execute(ControlCode::new(b'\n').unwrap()))
            .unwrap();
    }
    assert!(matches!(
        terminal.state().resolve_graphic(id),
        GraphicAnchorResolution::History { .. }
    ));
    terminal
        .resize(TerminalSize::new(10, 6, 100, 60).unwrap())
        .unwrap();
    assert!(
        terminal
            .state()
            .graphics()
            .any(|placement| placement.id() == id)
    );
    terminal
        .reduce(ScreenAction::SwitchBuffer {
            buffer: ScreenBuffer::Alternate,
            clear: true,
            home: true,
        })
        .unwrap();
    terminal
        .reduce(ScreenAction::SwitchBuffer {
            buffer: ScreenBuffer::Primary,
            clear: false,
            home: true,
        })
        .unwrap();
    assert!(
        terminal
            .state()
            .graphics()
            .any(|placement| placement.id() == id)
    );
}

#[test]
fn zlib_and_png_transfer_use_datum_owned_codecs() {
    let raw = [255, 0, 0, 255];
    let compressed = stored_zlib(&raw);
    let mut terminal = core();
    terminal
        .apply(apc(format!(
            "Ga=t,f=32,s=1,v=1,o=z,i=21;{}",
            encode_base64(&compressed)
        )))
        .unwrap();
    assert_eq!(
        terminal.state().kitty_images().next().unwrap().frames()[0].pixels(),
        &[rgba(255, 0, 0, 255)]
    );

    let png = png_rgba_1x1([0, 255, 0, 255]);
    terminal
        .apply(apc(format!("Ga=t,f=100,i=22;{}", encode_base64(&png))))
        .unwrap();
    let image = terminal
        .state()
        .kitty_images()
        .find(|image| image.id().get() == 22)
        .unwrap();
    assert_eq!(image.frames()[0].pixels(), &[rgba(0, 255, 0, 255)]);
}

fn stored_zlib(payload: &[u8]) -> Vec<u8> {
    let mut encoded = vec![0x78, 0x01, 1];
    let length = payload.len() as u16;
    encoded.extend_from_slice(&length.to_le_bytes());
    encoded.extend_from_slice(&(!length).to_le_bytes());
    encoded.extend_from_slice(payload);
    encoded.extend_from_slice(&adler32(payload).to_be_bytes());
    encoded
}

fn png_rgba_1x1(pixel: [u8; 4]) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::new();
    header.extend_from_slice(&1u32.to_be_bytes());
    header.extend_from_slice(&1u32.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);
    bytes.extend(png_chunk(b"IHDR", &header));
    bytes.extend(png_chunk(
        b"IDAT",
        &stored_zlib(&[0, pixel[0], pixel[1], pixel[2], pixel[3]]),
    ));
    bytes.extend(png_chunk(b"IEND", &[]));
    bytes
}

fn png_chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(data.len() as u32).to_be_bytes());
    bytes.extend_from_slice(kind);
    bytes.extend_from_slice(data);
    let mut checksum = Crc32::new();
    checksum.update(kind);
    checksum.update(data);
    bytes.extend_from_slice(&checksum.finish().to_be_bytes());
    bytes
}
