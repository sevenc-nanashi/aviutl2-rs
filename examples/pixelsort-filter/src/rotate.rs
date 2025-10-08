pub enum Rotate {
    Ninety,
    OneEighty,
    TwoSeventy,
}

pub fn rotate_image<T: Copy + Send + Sync>(
    pixels: &[T],
    width: usize,
    height: usize,
    rotate: Rotate,
) -> Vec<T> {
    match rotate {
        Rotate::Ninety => {
            if cfg!(feature = "rayon-rotate") {
                use rayon::prelude::*;
                let mut target = vec![pixels[0]; pixels.len()];
                (0..width * height)
                    .into_par_iter()
                    .map(|i| {
                        let new_x = i / height;
                        let new_y = i % height;
                        pixels[(height - new_y - 1) * width + new_x]
                    })
                    .collect_into_vec(&mut target);
                target
            } else {
                let mut rotated = vec![pixels[0]; pixels.len()];
                for y in 0..height {
                    for x in 0..width {
                        rotated[x * height + (height - y - 1)] = pixels[y * width + x];
                    }
                }
                rotated
            }
        }
        Rotate::OneEighty => {
            let mut cloned = pixels.to_vec();
            cloned.reverse();
            cloned
        }
        Rotate::TwoSeventy => {
            if cfg!(feature = "rayon-rotate") {
                use rayon::prelude::*;
                let mut target = vec![pixels[0]; pixels.len()];
                (0..width * height)
                    .into_par_iter()
                    .map(|i| {
                        let new_x = i / height;
                        let new_y = i % height;
                        pixels[new_y * width + (width - new_x - 1)]
                    })
                    .collect_into_vec(&mut target);
                target
            } else {
                let mut rotated = vec![pixels[0]; pixels.len()];
                for y in 0..height {
                    for x in 0..width {
                        rotated[(width - x - 1) * height + y] = pixels[y * width + x];
                    }
                }
                rotated
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotate_image() {
        let width = 3;
        let height = 2;
        let pixels = vec![
            1, 2, 3, // Row 0
            4, 5, 6, // Row 1
        ];
        let pixels = rotate_image(&pixels, width, height, Rotate::Ninety);
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
    fn test_rotate_image_180() {
        let width = 3;
        let height = 2;
        let pixels = vec![
            1, 2, 3, // Row 0
            4, 5, 6, // Row 1
        ];
        let pixels = rotate_image(&pixels, width, height, Rotate::OneEighty);
        assert_eq!(
            pixels,
            vec![
                6, 5, 4, // Row 0
                3, 2, 1, // Row 1
            ]
        );
    }

    #[test]
    fn test_rotate_image_270() {
        let width = 3;
        let height = 2;
        let pixels = vec![
            1, 2, 3, // Row 0
            4, 5, 6, // Row 1
        ];
        let pixels = rotate_image(&pixels, width, height, Rotate::TwoSeventy);
        assert_eq!(
            pixels,
            vec![
                3, 6, // Column 0
                2, 5, // Column 1
                1, 4, // Column 2
            ]
        );
    }

    #[test]
    fn test_rotate_image_reset() {
        let width = 3;
        let height = 2;
        let pixels = vec![
            1, 2, 3, // Row 0
            4, 5, 6, // Row 1
        ];
        let rotated = rotate_image(&pixels, width, height, Rotate::Ninety);
        let restored = rotate_image(&rotated, height, width, Rotate::TwoSeventy);
        assert_eq!(restored, pixels);
    }
}
