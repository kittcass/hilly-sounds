//! Mapping strategies between time and space.

use hilbert::fast_hilbert::hilbert_axes;
use num_bigint::BigUint;

use crate::Coord;

/// A space strategy which represents a mapping between time and two-dimensional
/// space.
pub trait SpaceStrategy<const N: usize> {
    /// Convert an index to a point in space.
    ///
    /// This should return a value only for all `0 <= index < size`.
    fn index_to_coord(&self, index: usize) -> Option<Coord<N>>;

    /// The effective length of the *n*th dimension.
    ///
    /// If `dimension` exceeds `N`, then this function may panic.
    fn length(&self, dimension: usize) -> u32;

    /// The amount which this space contains.
    ///
    /// By default, this is equal to the product of the lengths of all of the
    /// dimensions.
    fn size(&self) -> usize {
        (0..N).map(|idx| self.length(idx)).product::<u32>() as usize
    }
}

/// Adapt a space strategy of a lower dimension into a higher dimension.
pub struct SpaceStrategyAdapter<const A: usize, const B: usize, S>
where
    S: SpaceStrategy<A>,
{
    inner: S,
}

impl<const A: usize, const B: usize, S> SpaceStrategyAdapter<A, B, S>
where
    S: SpaceStrategy<A>,
{
    pub fn new(inner: S) -> Self {
        assert!(A < B);

        Self {
            inner,
        }
    }
}

impl<const A: usize, const B: usize, S> SpaceStrategy<B>
    for SpaceStrategyAdapter<A, B, S>
where
    S: SpaceStrategy<A>,
    [u32; B]: Default,
{
    fn index_to_coord(&self, index: usize) -> Option<Coord<B>> {
        match self.inner.index_to_coord(index) {
            Some(from_coord) => {
                let mut to_coord: [u32; B] = Default::default();
                to_coord.copy_from_slice(&from_coord);
                Some(to_coord)
            }
            None => None,
        }
    }

    fn length(&self, dimension: usize) -> u32 {
        assert!(dimension < B);

        if dimension < A {
            self.inner.length(dimension)
        } else {
            1
        }
    }
}

pub struct HilbertSpaceStrategy {
    size_exp: u32,
}

impl HilbertSpaceStrategy {
    pub fn from_size(size: u32) -> Self {
        HilbertSpaceStrategy {
            size_exp: size.log2(),
        }
    }
}

impl SpaceStrategy<2> for HilbertSpaceStrategy {
    fn index_to_coord(&self, index: usize) -> Option<Coord<2>> {
        let coords =
            hilbert_axes(&BigUint::from(index), self.size_exp as usize + 2, 2);
        Some([coords[0], coords[1]]) // TODO is there a better way to do this
    }

    fn length(&self, dimension: usize) -> u32 {
        assert!(dimension < 2);

        2u32.pow(self.size_exp)
    }

    fn size(&self) -> usize {
        2u32.pow(2 * self.size_exp) as usize
    }
}

pub struct LineSpaceStrategy {
    length: usize,
}

impl LineSpaceStrategy {
    pub fn new(length: usize) -> LineSpaceStrategy {
        LineSpaceStrategy { length }
    }
}

impl SpaceStrategy<1> for LineSpaceStrategy {
    fn index_to_coord(&self, index: usize) -> Option<Coord<1>> {
        Some([index.try_into().unwrap()])
    }

    fn length(&self, dimension: usize) -> u32 {
        assert!(dimension < 1);

        self.length.try_into().unwrap()
    }

    fn size(&self) -> usize {
        self.length
    }
}
