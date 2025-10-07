use aviutl2::filter::RgbaPixel;

pub enum Transpose {
    Ninety,
    OneEighty,
    TwoSeventy,
}

pub fn transpose_image(
    pixels: &mut [RgbaPixel],
    width: usize,
    height: usize,
    transpose: Transpose,
) {
    let mut transposed = Vec::with_capacity(pixels.len());
    match transpose {
        Transpose::Ninety => {
            for x in 0..width {
                for y in (0..height).rev() {
                    transposed.push(pixels[y * width + x]);
                }
            }
        }
        Transpose::OneEighty => {
            for y in (0..height).rev() {
                for x in (0..width).rev() {
                    transposed.push(pixels[y * width + x]);
                }
            }
        }
        Transpose::TwoSeventy => {
            for x in (0..width).rev() {
                for y in 0..height {
                    transposed.push(pixels[y * width + x]);
                }
            }
        }
    }

    pixels.copy_from_slice(&transposed);
}
