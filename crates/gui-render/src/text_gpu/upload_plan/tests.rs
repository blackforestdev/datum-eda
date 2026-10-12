use super::*;

#[test]
fn exact_dirty_words_replay_without_clean_word_copies() {
    let old = [0_u32; 12];
    let new = [1_u32, 0, 2, 3, 0, 0, 4, 0, 5, 6, 7, 0];
    let old = bytemuck::cast_slice(&old);
    let new = bytemuck::cast_slice(&new);
    let iter = Dirty::new(old, new, 16);
    let expected = vec![
        (0, &new[0..4]),
        (8, &new[8..16]),
        (24, &new[24..28]),
        (32, &new[32..44]),
    ];
    assert_eq!(iter.clone().collect::<Vec<_>>(), expected);
    assert_eq!(iter.collect::<Vec<_>>(), expected);
    assert!(Dirty::new(new, new, 16).next().is_none());
    assert_eq!(Dirty::new(&[], new, 16).collect::<Vec<_>>(), vec![(0, new)]);
}

#[cfg(feature = "visual")]
#[test]
#[ignore = "local GPU exact mixed-batch replay and producer refusal/retry proof"]
fn borrowed_mixed_plan_preserves_flat_groups_atomic_refusal_and_exact_gpu_bytes() {
    use crate::text_gpu::{budget::Budget, lifetime::Owner, sparse_upload, upload};
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        ..Default::default()
    }))
    .unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let storage = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("borrowed sparse target"),
        size: 1024,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let direct = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("borrowed direct target"),
        size: 1024,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let old = [0_u32; 256];
    let mut next = old;
    for i in (0..256).step_by(2) {
        next[i] = i as u32 + 1;
    }
    let old = bytemuck::cast_slice(&old);
    let bytes = bytemuck::cast_slice(&next);
    let ranges = (0..64).map(|i| i * 8..i * 8 + 4).collect::<Vec<_>>();
    let sources = [
        Source {
            target: None,
            buffer: &storage,
            bytes,
            ranges: Ranges::Dirty { old, stride: 4 },
        },
        Source {
            target: None,
            buffer: &direct,
            bytes,
            ranges: Ranges::Prepared(&ranges),
        },
    ];
    let replay = Uploads::sources(&sources);
    let flat = replay.clone().collect::<Vec<_>>();
    let summarize = |uploads: Uploads<'_, '_>| {
        sparse_upload::groups(uploads)
            .map(|group| {
                (
                    group.first.buffer.clone(),
                    group.sparse,
                    group
                        .uploads
                        .map(|u| (u.offset, u.bytes.to_vec()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(summarize(replay.clone()), summarize((&flat).into()));
    assert_eq!(
        summarize(replay.clone())
            .iter()
            .map(|g| g.1)
            .collect::<Vec<_>>(),
        [true, false]
    );
    assert_eq!(
        upload::required_bytes(&[], replay.clone()),
        upload::required_bytes(&[], &flat)
    );
    let owner = Owner::new();
    let scatter = sparse_upload::Scatter::default();
    let refused = Budget::new(1);
    assert!(
        upload::batch_with_scatter(
            &device,
            &owner,
            1,
            &refused,
            &[],
            replay.clone(),
            Some(&scatter)
        )
        .is_err()
    );
    assert_eq!(summarize(replay.clone()), summarize((&flat).into()));
    let budget = Budget::new(16 * 1024 * 1024);
    let mut batch = upload::batch_with_scatter(
        &device,
        &owner,
        1,
        &budget,
        &[],
        replay.clone(),
        Some(&scatter),
    )
    .unwrap()
    .unwrap();
    queue.submit([batch.command()]);
    batch.hold(&queue);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(budget.used(), 0);
    let mut producer =
        crate::gpu_data::screen_buffer::ScreenBuffer::with_budget(Budget::new(16 * 1024 * 1024));
    producer
        .sync(
            &device,
            &queue,
            "submitted producer baseline",
            &[0_u32; 256],
        )
        .unwrap();
    producer.flush_uploads(&device, &queue);
    producer
        .sync(&device, &queue, "pending exact producer", &next)
        .unwrap();
    {
        let mut pending = Vec::new();
        producer.append_sources(&mut pending);
        let before = Uploads::sources(&pending)
            .map(|u| (u.offset, u.bytes.to_vec()))
            .collect::<Vec<_>>();
        assert!(
            upload::batch_with_scatter(
                &device,
                &owner,
                2,
                &refused,
                &[],
                Uploads::sources(&pending),
                Some(&scatter)
            )
            .is_err()
        );
        assert_eq!(
            Uploads::sources(&pending)
                .map(|u| (u.offset, u.bytes.to_vec()))
                .collect::<Vec<_>>(),
            before
        );
        let mut retry = upload::batch_with_scatter(
            &device,
            &owner,
            2,
            &budget,
            &[],
            Uploads::sources(&pending),
            Some(&scatter),
        )
        .unwrap()
        .unwrap();
        queue.submit([retry.command()]);
        retry.hold(&queue);
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
    producer.finish_uploads();
    let mut finished = Vec::new();
    producer.append_sources(&mut finished);
    assert!(Uploads::sources(&finished).is_empty());
    drop(finished);
    assert_eq!(budget.used(), 0);
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("borrowed upload readback"),
        size: 3072,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_buffer_to_buffer(&storage, 0, &readback, 0, 1024);
    encoder.copy_buffer_to_buffer(&direct, 0, &readback, 1024, 1024);
    encoder.copy_buffer_to_buffer(producer.buffer().unwrap(), 0, &readback, 2048, 1024);
    queue.submit([encoder.finish()]);
    let slice = readback.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let actual = slice.get_mapped_range();
    assert_eq!(&actual[..1024], bytes);
    let mut expected_direct = vec![0; 1024];
    for range in &ranges {
        expected_direct[range.clone()].copy_from_slice(&bytes[range.clone()]);
    }
    assert_eq!(&actual[1024..2048], expected_direct);
    assert_eq!(&actual[2048..], bytes);
    drop(actual);
    readback.unmap();
    let invalid = [2000..2004];
    let malformed = [Source {
        target: None,
        buffer: &storage,
        bytes,
        ranges: Ranges::Prepared(&invalid),
    }];
    assert!(
        upload::batch_with_scatter(
            &device,
            &owner,
            2,
            &budget,
            &[],
            Uploads::sources(&malformed),
            Some(&scatter)
        )
        .is_err()
    );
    assert_eq!(budget.used(), 0);
}
