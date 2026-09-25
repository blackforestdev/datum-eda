use super::*;

#[test]
#[ignore = "requires local GPU; resource ownership conformance; run serially"]
fn local_observation_preserves_budget_identity_without_retaining_gpu_resources() {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let renderer =
        crate::Renderer::new(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb, 4).unwrap();
    let first = renderer.local_reservation_observer();
    let budget = Arc::downgrade(&renderer.screen_budget);
    let pressure = renderer.atlas.staging_budget.reserve(4096).unwrap();
    let replacement = renderer
        .recreate_for_device(&device, &queue, wgpu::TextureFormat::Rgba8UnormSrgb, 4)
        .unwrap();
    let second = replacement.local_reservation_observer();
    let a = first.usage();
    let b = second.usage();
    assert_ne!(a.renderer_id, b.renderer_id);
    assert_eq!(a.screen, b.screen);
    assert_eq!(a.control_mesh, b.control_mesh);
    assert_eq!(a.atlas, b.atlas);
    assert_eq!(a.staging, b.staging);
    assert!(a.screen.reserved_bytes > 0);
    assert!(a.staging.lifetime_peak_reserved_bytes >= 4096);
    assert!(!a.released());
    drop(renderer);
    drop(replacement);
    assert!(
        !first.usage().released(),
        "a surviving reservation remains visible after renderer close"
    );
    assert_eq!(first.usage().staging.reserved_bytes, 4096);
    drop(pressure);
    assert!(first.usage().released());
    assert!(second.usage().released());
    assert_eq!(
        first.usage().screen.lifetime_peak_reserved_bytes,
        a.screen.lifetime_peak_reserved_bytes
    );
    assert!(
        crate::Renderer::gpu_process_allocations().iter().all(|r| ![
            Some(a.renderer_id),
            Some(b.renderer_id)
        ]
        .contains(&r.renderer_id))
    );
    assert!(
        budget.upgrade().is_some(),
        "only budget metadata remains observed"
    );
    drop(first);
    drop(second);
    assert!(budget.upgrade().is_none());
}

#[test]
fn document_views_reconcile_history_and_shared_gpu_reservations() {
    let budget =
        crate::retained_scene_owner::document_cpu::for_scene("resource-observation-document");
    let id = budget.id();
    let reservation = budget.reserve(2048).unwrap();
    let gpu = crate::Renderer::world_document_gpu_usage()
        .into_iter()
        .find(|row| row.budget_id == id)
        .unwrap();
    let cpu = document_cpu_usage()
        .into_iter()
        .find(|row| row.budget_id == id)
        .unwrap();
    assert_eq!(gpu.scene_id, cpu.scene_id);
    assert_eq!(gpu.reserved_bytes, 2048);
    assert_eq!(gpu.lifetime_peak_reserved_bytes, 2048);
    assert!(
        cpu.retained_bytes > 0,
        "registered document metadata is counted"
    );
    assert_eq!(cpu.history_entries, 0);
    drop(reservation);
    drop(budget);
    assert!(
        crate::Renderer::world_document_gpu_usage()
            .iter()
            .all(|row| row.budget_id != id)
    );
    assert!(document_cpu_usage().iter().all(|row| row.budget_id != id));
}
