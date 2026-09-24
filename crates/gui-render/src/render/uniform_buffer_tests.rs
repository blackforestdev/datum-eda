use super::*;

#[cfg(feature = "visual")]
#[test]
#[ignore = "requires local GPU; deferred uniform and lifetime readback"]
fn cancelled_uniform_updates_leave_submitted_bytes_and_retirement_intact() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let read = |buffer: &wgpu::Buffer| {
        let target = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform-readback"),
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(buffer, 0, &target, 0, 16);
        queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        target
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |r| tx.send(r).unwrap());
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        rx.recv().unwrap().unwrap();
        let data = target.slice(..).get_mapped_range().to_vec();
        target.unmap();
        data
    };
    let mut owner = UniformBuffer::new(
        &device,
        "uniform",
        [1_u32; 4],
        &Budget::new(16 * 1024 * 1024),
    )
    .unwrap();
    let id = owner.buffer.id();
    assert_eq!(owner.sync(&queue, [2_u32; 4]), 16);
    assert_eq!(
        read(owner.buffer()),
        bytemuck::cast_slice::<u32, u8>(&[1; 4])
    );
    owner.cancel_uploads();
    assert_eq!(owner.sync(&queue, [1_u32; 4]), 0);
    owner.flush_uploads(&device, &queue);
    assert_eq!(
        read(owner.buffer()),
        bytemuck::cast_slice::<u32, u8>(&[1; 4])
    );
    assert_eq!(owner.sync(&queue, [1, 2, 1, 3]), 8);
    owner.flush_uploads(&device, &queue);
    assert_eq!(
        read(owner.buffer()),
        bytemuck::cast_slice::<u32, u8>(&[1, 2, 1, 3])
    );
    assert_eq!(owner.sync(&queue, [1, 2, 1, 3]), 0);
    let held = owner.submission_ref();
    drop(owner);
    assert!(
        crate::Renderer::gpu_process_allocations()
            .iter()
            .any(|r| r.id == id && r.kind == Kind::Uniform && r.bytes == 16 && r.retiring)
    );
    drop(held);
    assert!(
        !crate::Renderer::gpu_process_allocations()
            .iter()
            .any(|r| r.id == id)
    );
}

#[test]
fn uniform_ranges_transfer_only_dirty_aligned_words() {
    for mask in 0_u32..256 {
        let old = [0_u8; 32];
        let mut new = old;
        for index in 0..8 {
            if mask & (1 << index) != 0 {
                new[4 * index + index % 4] = 1;
            }
        }
        let mut result = old;
        let mut writes = 0;
        let bytes = write_changed_ranges(Some(&old), &new, |offset, data| {
            assert_eq!(offset % 4, 0);
            assert_eq!(data.len() % 4, 0);
            for word in data.as_chunks::<4>().0.iter() {
                assert_ne!(*word, [0; 4]);
            }
            result[offset..offset + data.len()].copy_from_slice(data);
            writes += 1;
        });
        assert_eq!(result, new);
        assert_eq!(bytes, mask.count_ones() as usize * 4);
        assert_eq!(writes, (mask & !(mask << 1)).count_ones());
    }
    let mut writes = 0;
    assert_eq!(
        write_changed_ranges(None, &[0; 64], |offset, data| {
            assert_eq!(offset, 0);
            assert_eq!(data, [0; 64]);
            writes += 1;
        }),
        64
    );
    assert_eq!(writes, 1, "new storage must be initialized in full");
}

#[cfg(feature = "visual")]
#[test]
#[ignore = "requires local GPU; fixed uniform recovery generations"]
fn fixed_uniform_recovery_preserves_submission_generations() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let first = crate::Renderer::new(&device, &queue, format, 1).unwrap();
    let holds = [
        first.uniform_buffer.submission_ref(),
        first.scene_bind_group.buffer.submission_ref(),
        first.schematic_scene_bind_group.buffer.submission_ref(),
    ];
    let budgets = [
        first.uniform_buffer.generation_budget.clone(),
        first.scene_bind_group.buffer.generation_budget.clone(),
        first
            .schematic_scene_bind_group
            .buffer
            .generation_budget
            .clone(),
    ];
    let second = first
        .recreate_for_device(&device, &queue, format, 1)
        .unwrap();
    drop(first);
    assert!(budgets.iter().all(|b| b.used() == 2));
    let screen = second.screen_budget.clone();
    let before = screen.used();
    assert!(
        second
            .recreate_for_device(&device, &queue, format, 1)
            .err()
            .unwrap()
            .to_string()
            .contains("two live GPU allocations")
    );
    assert_eq!(screen.used(), before);
    // Release each earlier allocation separately: failure at later constructors
    // must roll back the earlier reservations from the same recovery attempt.
    for (index, hold) in holds.into_iter().enumerate() {
        drop(hold);
        if budgets.iter().any(|b| b.used() == 2) {
            assert!(
                second
                    .recreate_for_device(&device, &queue, format, 1)
                    .is_err()
            );
            assert_eq!(screen.used(), before - 16 - index as u64 * 64);
        }
    }
    let third = second
        .recreate_for_device(&device, &queue, format, 1)
        .unwrap();
    assert!(budgets.iter().all(|b| b.used() == 2));
    drop(second);
    drop(third);
    assert!(budgets.iter().all(|b| b.used() == 0));
    assert_eq!(screen.used(), 0);
}

#[cfg(feature = "visual")]
#[test]
#[ignore = "requires local GPU; pane uniform close and recovery continuity"]
fn pane_uniform_slots_preserve_retiring_generations_across_reopen_and_recovery() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let state = crate::gpu_surface_pass::board_fixture_state();
    let retained = crate::RetainedScene::from_workspace(&state, 960, 720);
    let prepared = crate::PreparedScene::from_workspace_for_surface(
        &state,
        960,
        720,
        1.0,
        crate::CameraState::fit_to_bounds(&state.scene.bounds),
        &retained,
    );
    let mut first = crate::Renderer::new(&device, &queue, format, 1).unwrap();
    first
        .prepare_surface_uniforms(&device, &queue, &prepared, 960, 720)
        .unwrap();
    assert!(!first.surface_scene_uniforms.is_empty());
    let budgets: Vec<_> = first
        .surface_scene_uniforms
        .iter()
        .map(|b| b.buffer.generation_budget.clone())
        .collect();
    let first_holds: Vec<_> = first
        .surface_scene_uniforms
        .iter()
        .map(|b| b.buffer.submission_ref())
        .collect();
    first.surface_scene_uniforms.clear();
    first
        .prepare_surface_uniforms(&device, &queue, &prepared, 960, 720)
        .unwrap();
    let second_holds: Vec<_> = first
        .surface_scene_uniforms
        .iter()
        .map(|b| b.buffer.submission_ref())
        .collect();
    first.surface_scene_uniforms.clear();
    assert!(budgets.iter().all(|b| b.used() == 2));
    let screen = first.screen_budget.clone();
    let before = screen.used();
    assert!(
        first
            .prepare_surface_uniforms(&device, &queue, &prepared, 960, 720)
            .is_err()
    );
    assert_eq!(screen.used(), before);
    assert!(first.surface_scene_uniforms.is_empty());
    let mut recovered = first
        .recreate_for_device(&device, &queue, format, 1)
        .unwrap();
    drop(first);
    assert!(
        recovered
            .prepare_surface_uniforms(&device, &queue, &prepared, 960, 720)
            .is_err()
    );
    drop(first_holds);
    recovered
        .prepare_surface_uniforms(&device, &queue, &prepared, 960, 720)
        .unwrap();
    for (binding, budget) in recovered.surface_scene_uniforms.iter().zip(&budgets) {
        assert!(Arc::ptr_eq(&binding.buffer.generation_budget, budget));
        assert_eq!(budget.used(), 2);
    }
    drop(second_holds);
    drop(recovered);
    assert!(budgets.iter().all(|b| b.used() == 0));
    assert_eq!(screen.used(), 0);
}
