//! Fixed row-major source tiles and their disjoint atlas cells.
pub(crate) const TILE: u32 = 32;
pub(crate) const ATLAS: u32 = 512;
const SLOTS: usize = 256;

pub(crate) struct Plan {
    ids: [u32; SLOTS],
    count: usize,
    extent: [u32; 2],
    columns: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Tile {
    pub id: u32,
    pub source: [u32; 2],
    pub atlas: [u32; 2],
    pub extent: [u32; 2],
}
impl Plan {
    pub(crate) fn new(extent: [u32; 2], rectangles: &[[u32; 4]]) -> Option<Self> {
        let [width, height] = extent;
        if width == 0 || height == 0 {
            return None;
        }
        let columns = width.div_ceil(TILE);
        // IDs are also the immutable first-instance argument of restoration.
        columns.checked_mul(height.div_ceil(TILE))?;
        let mut plan = Self {
            ids: [0; SLOTS],
            count: 0,
            extent,
            columns,
        };
        for &[left, top, right, bottom] in rectangles {
            let right = right.min(width);
            let bottom = bottom.min(height);
            if left >= right || top >= bottom {
                continue;
            }
            for y in top / TILE..=(bottom - 1) / TILE {
                for x in left / TILE..=(right - 1) / TILE {
                    let id = y.checked_mul(columns)?.checked_add(x)?;
                    match plan.ids[..plan.count].binary_search(&id) {
                        Ok(_) => (),
                        Err(index) => {
                            if plan.count == SLOTS {
                                return None;
                            }
                            plan.ids.copy_within(index..plan.count, index + 1);
                            plan.ids[index] = id;
                            plan.count += 1;
                        }
                    }
                }
            }
        }
        Some(plan)
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.count == 0
    }
    pub(crate) fn tiles(&self) -> impl Iterator<Item = Tile> + '_ {
        self.ids[..self.count]
            .iter()
            .enumerate()
            .map(|(slot, &id)| {
                let source = [id % self.columns * TILE, id / self.columns * TILE];
                Tile {
                    id,
                    source,
                    atlas: [slot as u32 % 16 * TILE, slot as u32 / 16 * TILE],
                    extent: [
                        (self.extent[0] - source[0]).min(TILE),
                        (self.extent[1] - source[1]).min(TILE),
                    ],
                }
            })
    }
}
impl Tile {
    pub(crate) fn clip(self) -> crate::renderer_state::damage::clip::Clip {
        crate::renderer_state::damage::clip::Clip::translated(
            [
                self.source[0],
                self.source[1],
                self.source[0] + self.extent[0],
                self.source[1] + self.extent[1],
            ],
            self.atlas,
        )
    }
    pub(crate) fn viewport(self, pass: &mut wgpu::RenderPass<'_>, width: u32, height: u32) {
        pass.set_viewport(
            self.atlas[0] as f32 - self.source[0] as f32,
            self.atlas[1] as f32 - self.source[1] as f32,
            width as f32,
            height as f32,
            0.0,
            1.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deduplication_seams_padding_and_capacity_are_exact() {
        let plan = Plan::new(
            [65, 34],
            &[[31, 31, 65, 34], [32, 32, 64, 34], [31, 31, 65, 34]],
        )
        .unwrap();
        let tiles: Vec<_> = plan.tiles().collect();
        assert_eq!(
            tiles.iter().map(|t| t.id).collect::<Vec<_>>(),
            [0, 1, 2, 3, 4, 5]
        );
        assert_eq!(
            tiles[5],
            Tile {
                id: 5,
                source: [64, 32],
                atlas: [160, 0],
                extent: [1, 2]
            }
        );
        assert_eq!(
            tiles[5].clip().intersect([0, 0, 65, 34]),
            Some([160, 0, 1, 2])
        );
        let full = Plan::new([512, 512], &[[0, 0, 512, 512]]).unwrap();
        assert_eq!(full.tiles().count(), 256);
        assert_eq!(full.tiles().last().unwrap().atlas, [480, 480]);
        assert!(Plan::new([513, 512], &[[0, 0, 513, 512]]).is_none());
        assert!(Plan::new([u32::MAX, u32::MAX], &[]).is_none());
        assert!(Plan::new([0, 512], &[]).is_none());
        assert!(
            Plan::new([512, 512], &[[513, 0, 600, 1], [1, 1, 1, 1]])
                .unwrap()
                .is_empty()
        );
        assert!(std::mem::size_of::<Plan>() <= 1056);
    }
}
