//! Bounded-command application of fragmented exact-word updates.
use super::upload::BufferUpload;
use std::sync::OnceLock;

#[derive(Default)]
pub(crate) struct Scatter(OnceLock<wgpu::ComputePipeline>);

pub(super) struct Group<'s, 'a> {
    pub uploads: std::iter::Take<super::upload::plan::Uploads<'s, 'a>>,
    pub first: BufferUpload<'a>,
    pub sparse: bool,
}

pub(super) fn groups<'s, 'a: 's>(
    uploads: impl Into<super::upload::plan::Uploads<'s, 'a>>,
) -> impl Iterator<Item = Group<'s, 'a>> + Clone {
    let mut cursor = uploads.into();
    std::iter::from_fn(move || {
        let start = cursor.clone();
        let first = cursor.next()?;
        let mut count = 1;
        let mut ordered = true;
        let mut previous_end = first.offset.checked_add(first.bytes.len() as u64);
        loop {
            let mut next = cursor.clone();
            let Some(upload) = next.next() else {
                break;
            };
            if upload.buffer != first.buffer {
                break;
            }
            ordered &= previous_end.is_some_and(|end| end <= upload.offset);
            previous_end = upload.offset.checked_add(upload.bytes.len() as u64);
            cursor = next;
            count += 1;
        }
        let sparse =
            count >= 64 && first.buffer.usage().contains(wgpu::BufferUsages::STORAGE) && ordered;
        Some(Group {
            uploads: start.take(count),
            first,
            sparse,
        })
    })
}

impl Group<'_, '_> {
    pub fn packet_bytes(&self) -> u64 {
        self.uploads.clone().map(|u| u.bytes.len() as u64 * 2).sum()
    }

    pub fn fill(&self, packet: &mut [u8]) {
        let mut cursor = 0;
        for upload in self.uploads.clone() {
            for (index, word) in upload.bytes.as_chunks::<4>().0.iter().enumerate() {
                let destination = (upload.offset / 4 + index as u64) as u32;
                packet[cursor..cursor + 4].copy_from_slice(&destination.to_le_bytes());
                packet[cursor + 4..cursor + 8].copy_from_slice(word);
                cursor += 8;
            }
        }
        assert_eq!(cursor, packet.len());
    }
}

impl Scatter {
    pub fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        packet: &wgpu::Buffer,
        destination: &wgpu::Buffer,
    ) {
        let pipeline = self.0.get_or_init(|| {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("datum-exact-word-scatter"),
                source: wgpu::ShaderSource::Wgsl(include_str!("scatter.wgsl").into()),
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("datum-exact-word-scatter"),
                layout: None,
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            })
        });
        let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("datum-exact-word-scatter"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: packet.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: destination.as_entire_binding(),
                },
            ],
        });
        let groups = (packet.size() / 8).div_ceil(64) as u32;
        let x = groups.min(device.limits().max_compute_workgroups_per_dimension);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("datum-exact-word-scatter"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &binding, &[]);
        pass.dispatch_workgroups(x, groups.div_ceil(x), 1);
    }
}

#[cfg(all(test, feature = "visual"))]
#[path = "sparse_upload_tests.rs"]
mod tests;
