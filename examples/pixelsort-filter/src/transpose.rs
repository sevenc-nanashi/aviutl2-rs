use rayon::prelude::*;

pub enum Transpose {
    Ninety,
    OneEighty,
    TwoSeventy,
}

pub fn transpose_image<T: Copy + Send + Sync>(
    pixels: &mut [T],
    width: usize,
    height: usize,
    transpose: Transpose,
) {
    match transpose {
        Transpose::Ninety => {
            let transposed: Vec<T> = (0..width)
                .into_par_iter()
                .flat_map(|y| {
                    (0..height)
                        .rev()
                        .map(|x| pixels[x * width + y])
                        .collect::<Vec<T>>()
                })
                .collect();
            pixels.copy_from_slice(&transposed);
        }
        Transpose::OneEighty => {
            pixels.reverse();
        }
        Transpose::TwoSeventy => {
            let transposed: Vec<T> = (0..width)
                .into_par_iter()
                .rev()
                .flat_map(|y| {
                    (0..height)
                        .map(|x| pixels[x * width + y])
                        .collect::<Vec<T>>()
                })
                .collect();
            pixels.copy_from_slice(&transposed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transpose_image() {
        let width = 3;
        let height = 2;
        let mut pixels = vec![
            1, 2, 3, // Row 0
            4, 5, 6, // Row 1
        ];
        transpose_image(&mut pixels, width, height, Transpose::Ninety);
        assert_eq!(
            pixels,
            vec![
                4, 1, // Column 0
                5, 2, // Column 1
                6, 3, // Column 2
            ]
        );
    }

    #[test]
    fn test_transpose_image_180() {
        let width = 3;
        let height = 2;
        let mut pixels = vec![
            1, 2, 3, // Row 0
            4, 5, 6, // Row 1
        ];
        transpose_image(&mut pixels, width, height, Transpose::OneEighty);
        assert_eq!(
            pixels,
            vec![
                6, 5, 4, // Row 0
                3, 2, 1, // Row 1
            ]
        );
    }

    #[test]
    fn test_transpose_image_270() {
        let width = 3;
        let height = 2;
        let mut pixels = vec![
            1, 2, 3, // Row 0
            4, 5, 6, // Row 1
        ];
        transpose_image(&mut pixels, width, height, Transpose::TwoSeventy);
        assert_eq!(
            pixels,
            vec![
                3, 6, // Column 0
                2, 5, // Column 1
                1, 4, // Column 2
            ]
        );
    }
}
