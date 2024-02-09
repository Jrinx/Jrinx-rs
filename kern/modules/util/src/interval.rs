use alloc::vec::{IntoIter, Vec};

const INFINITY: isize = i16::MAX as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bound {
    start: usize,
    len: usize,
}

pub struct ExclusiveIntervals {
    inner: Vec<(usize, isize)>,
}

pub struct ExclusiveIntervalsIter {
    sum: isize,
    into_iter: IntoIter<(usize, isize)>,
}

impl Bound {
    pub const fn new(start: usize, len: usize) -> Self {
        Self { start, len }
    }
}

impl From<Bound> for (usize, usize) {
    fn from(value: Bound) -> Self {
        (value.start, value.len)
    }
}

impl ExclusiveIntervals {
    pub fn new(iter: impl IntoIterator<Item = Bound>) -> Self {
        let interval = Self {
            inner: iter
                .into_iter()
                .flat_map(|Bound { start, len }| [(start, 1), (start + len, -1)])
                .collect::<Vec<_>>(),
        };
        debug_assert!(interval.inner.len() < INFINITY as _);
        interval
    }
}

impl core::ops::SubAssign<Bound> for ExclusiveIntervals {
    fn sub_assign(&mut self, rhs: Bound) {
        self.inner.push((rhs.start, -INFINITY));
        self.inner.push((rhs.start + rhs.len, INFINITY));
    }
}

impl Iterator for ExclusiveIntervalsIter {
    type Item = Bound;

    fn next(&mut self) -> Option<Self::Item> {
        let mut left = None;
        for (pos, delta) in self.into_iter.by_ref() {
            let old = self.sum;
            self.sum += delta;
            if old <= 0 && self.sum > 0 {
                left = Some(pos);
            } else if old > 0 && self.sum <= 0 {
                return Some(Bound {
                    start: left.unwrap(),
                    len: pos - left.unwrap(),
                });
            }
        }
        None
    }
}

impl IntoIterator for ExclusiveIntervals {
    type Item = Bound;
    type IntoIter = ExclusiveIntervalsIter;

    fn into_iter(mut self) -> Self::IntoIter {
        self.inner.sort_unstable();
        ExclusiveIntervalsIter {
            sum: 0,
            into_iter: self
                .inner
                .chunk_by(|&b1, &b2| b1.0 == b2.0)
                .map(|s| (s[0].0, s.iter().map(|t| t.1).sum::<isize>()))
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }
}
