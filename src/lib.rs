use hilbert::fast_hilbert::hilbert_axes;
use nannou::{
    color::{hsv, Rgba},
    image,
    prelude::{One, ToPrimitive, Zero},
};
use num_bigint::BigUint;

// TODO should we separate pixel data encoding/decoding from the hilbert curve?
// so far, this algorithm operates independently of the curve, so maybe it
// should be decoupled? much to think about...

#[derive(Copy, Clone)]
pub struct Options {
    pub size_exp: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self { size_exp: 9 }
    }
}

pub struct Encoder<I>
where
    I: Iterator<Item = i16>,
{
    iter: I,
    options: Options,
    index: BigUint,
}

impl<I> Encoder<I>
where
    I: Iterator<Item = i16>,
{
    pub fn new(iter: I, options: Options) -> Self {
        Encoder {
            iter,
            options,
            index: BigUint::zero(),
        }
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn side_len(&self) -> u32 {
        2u32.pow(self.options.size_exp as u32)
    }
}

impl<I> Iterator for Encoder<I>
where
    I: Iterator<Item = i16>,
{
    type Item = ([u32; 2], image::Rgba<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO don't do this every iteration?
        let side_len = self.side_len();
        if self.index.to_u32().unwrap() >= side_len * side_len {
            return None;
        }

        if let Some(sample) = self.iter.next() {
            let coords =
                hilbert_axes(&self.index, self.options.size_exp + 2, 2);
            self.index += BigUint::one();

            let hue =
                (sample as f32 + 2u32.pow(15) as f32) / (2u32.pow(16) as f32);

            let rgb: Rgba = hsv(hue, 1.0, 1.0).into();

            Some((
                [coords[0], coords[1]],
                image::Rgba([
                    (255. * rgb.red) as u8,
                    (255. * rgb.green) as u8,
                    (255. * rgb.blue) as u8,
                    255,
                ]),
            ))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // TODO also take into account size from options
        self.iter.size_hint()
    }
}

pub fn encode_image<I>(
    iter: I,
    options: Options,
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>>
where
    I: Iterator<Item = i16>,
{
    let mut encoder = Encoder::new(iter, options);
    let mut image =
        image::ImageBuffer::new(encoder.side_len(), encoder.side_len());

    while let Some(([x, y], color)) = encoder.next() {
        image.put_pixel(x, y, color);
    }

    image
}
