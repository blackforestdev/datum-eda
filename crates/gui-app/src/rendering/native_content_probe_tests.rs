use super::*;

#[test]
fn disabled_probe_has_no_request_or_output_state() {
    let mut probe = Probe::default();
    assert!(!probe.active());
    probe.wheel(false, false);
    probe.primary(true, false, false);
    #[cfg(feature = "visual")]
    {
        assert!(probe.state.is_none());
        assert!(
            probe
                .observe_armed(|| panic!("disabled subject allocation"))
                .is_none()
        );
    }
}

#[test]
fn subject_observation_runs_only_once_for_each_armed_phase() {
    for phase in [
        Phase::Zoom(0),
        Phase::Zoom(5),
        Phase::BaselinePending,
        Phase::ReleasedReady,
        Phase::ReleasedPending,
        Phase::Done,
        Phase::Failed,
    ] {
        assert!(
            phase
                .observe_armed(|| panic!("unarmed subject allocation: {phase:?}"))
                .is_none()
        );
    }
    for phase in [Phase::BaselineArmed, Phase::ReleasedArmed] {
        let mut calls = 0;
        assert_eq!(
            phase.observe_armed(|| {
                calls += 1;
                "actual subject"
            }),
            Some("actual subject")
        );
        assert_eq!(calls, 1);
    }
}

#[cfg(feature = "visual")]
#[test]
fn map_and_present_own_the_completion_boundary_in_either_order() {
    assert!(!delivery_ready(&Some(Ok(())), None));
    assert!(!delivery_ready(&None, Some(true)));
    assert!(delivery_ready(&Some(Ok(())), Some(true)));
    assert!(delivery_valid(&Some(Ok(())), Some(true), false));
    assert!(!delivery_valid(
        &Some(Err("original map failure".into())),
        Some(true),
        false
    ));
    assert!(!delivery_valid(&Some(Ok(())), Some(false), false));
    assert!(!delivery_valid(&Some(Ok(())), Some(true), true));
}

#[cfg(feature = "visual")]
#[test]
fn native_copy_admission_uses_actual_caps_and_never_adds_unsupported_usage() {
    let mut caps = wgpu::SurfaceCapabilities {
        formats: vec![wgpu::TextureFormat::Bgra8UnormSrgb],
        present_modes: vec![wgpu::PresentMode::Fifo],
        alpha_modes: vec![wgpu::CompositeAlphaMode::Opaque],
        usages: wgpu::TextureUsages::RENDER_ATTACHMENT,
    };
    let mut config = gui_runtime_support::surface_configuration(
        &caps,
        winit::dpi::PhysicalSize::new(1344, 806),
        None,
    );
    assert!(gui_runtime_support::admit_main_content_readback(&caps, &mut config).is_err());
    assert!(!config.usage.contains(wgpu::TextureUsages::COPY_SRC));
    caps.usages |= wgpu::TextureUsages::COPY_SRC;
    gui_runtime_support::admit_main_content_readback(&caps, &mut config).unwrap();
    assert!(config.usage.contains(wgpu::TextureUsages::COPY_SRC));
    config.format = wgpu::TextureFormat::Rgba16Float;
    assert!(gui_runtime_support::admit_main_content_readback(&caps, &mut config).is_err());
    config.format = caps.formats[0];
    config.width = 0;
    assert!(gui_runtime_support::admit_main_content_readback(&caps, &mut config).is_err());
}
