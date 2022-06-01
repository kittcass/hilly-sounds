use hilbert::fast_hilbert::hilbert_axes;
use num_bigint::BigUint;

pub type Coord = [u32; 2];

pub trait SpaceStrategy {
    fn index_to_coord(&self, index: usize) -> Option<Coord>;

    fn coord_to_index(&self, coord: Coord) -> Option<usize>;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn size(&self) -> usize {
        (self.width() * self.height()) as usize
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

impl SpaceStrategy for HilbertSpaceStrategy {
    fn index_to_coord(&self, index: usize) -> Option<Coord> {
        let coords =
            hilbert_axes(&BigUint::from(index), self.size_exp as usize + 2, 2);
        Some([coords[0], coords[1]]) // TODO is there a better way to do this
    }

    fn coord_to_index(&self, _coord: Coord) -> Option<usize> {
        todo!()
    }

    fn width(&self) -> u32 {
        2u32.pow(self.size_exp)
    }

    fn height(&self) -> u32 {
        self.width()
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

impl SpaceStrategy for LineSpaceStrategy {
    fn index_to_coord(&self, index: usize) -> Option<Coord> {
        Some([index.try_into().unwrap(), 0])
    }

    fn coord_to_index(&self, coord: Coord) -> Option<usize> {
        Some(coord[0] as usize)
    }

    fn width(&self) -> u32 {
        self.length as u32
    }

    fn height(&self) -> u32 {
        1
    }

    fn size(&self) -> usize {
        self.length
    }
}
