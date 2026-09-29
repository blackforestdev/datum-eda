//! Exact bounded disjoint union; failure retains the full painter graph.
use super::{MAX_RECTS, Pixels};
impl Pixels {
    pub(crate) fn disjoint(&self) -> Option<Self> {
        let mut result = Self::default();
        for &rect in self.rectangles() {
            let mut pending = Self::default();
            pending.push(rect)?;
            for &cut in result.rectangles() {
                let mut i = 0;
                while i < pending.count {
                    let r = pending.rects[i];
                    let overlap = [
                        r[0].max(cut[0]),
                        r[1].max(cut[1]),
                        r[2].min(cut[2]),
                        r[3].min(cut[3]),
                    ];
                    if overlap[0] >= overlap[2] || overlap[1] >= overlap[3] {
                        i += 1;
                        continue;
                    }
                    pending.count -= 1;
                    pending.rects[i] = pending.rects[pending.count];
                    for part in [
                        [r[0], r[1], r[2], overlap[1]],
                        [r[0], overlap[3], r[2], r[3]],
                        [r[0], overlap[1], overlap[0], overlap[3]],
                        [overlap[2], overlap[1], r[2], overlap[3]],
                    ] {
                        pending.push(part)?;
                    }
                    // Appended pieces no longer intersect this cut. The swapped
                    // pending rectangle still must be checked at the same index.
                }
            }
            for &part in pending.rectangles() {
                result.merge_push(part)?;
            }
        }
        Some(result)
    }
    fn merge_push(&mut self, mut r: [u32; 4]) -> Option<()> {
        let mut i = 0;
        while i < self.count {
            let other = self.rects[i];
            if (r[0] == other[0] && r[2] == other[2] && (r[1] == other[3] || r[3] == other[1]))
                || (r[1] == other[1] && r[3] == other[3] && (r[0] == other[2] || r[2] == other[0]))
            {
                r = [
                    r[0].min(other[0]),
                    r[1].min(other[1]),
                    r[2].max(other[2]),
                    r[3].max(other[3]),
                ];
                self.count -= 1;
                self.rects[i] = self.rects[self.count];
                i = 0;
            } else {
                i += 1;
            }
        }
        if self.count == MAX_RECTS {
            return None;
        }
        self.push(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clipped_union_paints_crossings_once_and_bounds_fragmentation() {
        let mut source = Pixels::default();
        for r in [
            [0, 8, 24, 11],
            [8, 0, 11, 24],
            [0, 9, 24, 12],
            [9, 0, 12, 24],
            [4, 4, 18, 6],
            [4, 16, 18, 18],
        ] {
            source.push(r).unwrap();
        }
        for reverse in [false, true] {
            if reverse {
                source.rects[..source.count].reverse();
            }
            let partition = source.disjoint().unwrap();
            for y in 0..25 {
                for x in 0..25 {
                    let inside = |r: &&[u32; 4]| x >= r[0] && y >= r[1] && x < r[2] && y < r[3];
                    let wanted = source.rectangles().iter().any(|r| inside(&r));
                    assert_eq!(
                        partition.rectangles().iter().filter(inside).count(),
                        usize::from(wanted)
                    );
                }
            }
        }
        let mut fragmented = Pixels::default();
        for i in 0..16 {
            fragmented.push([0, i * 2, 32, i * 2 + 1]).unwrap();
        }
        for i in 0..16 {
            fragmented.push([i * 2, 0, i * 2 + 1, 32]).unwrap();
        }
        assert!(
            fragmented.disjoint().is_none(),
            "cap refusal, never overlapping alpha replay"
        );
        let mut exact = Pixels::default();
        for i in 0..MAX_RECTS {
            exact.push([i as u32 * 2, 0, i as u32 * 2 + 1, 1]).unwrap();
        }
        assert_eq!(exact.disjoint().unwrap().count, MAX_RECTS);
    }
}
