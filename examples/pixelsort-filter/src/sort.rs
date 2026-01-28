use crate::FilterConfig;
use aviutl2::filter::RgbaPixel;
use ndarray::{ArrayViewMut2, Axis};
use rayon::prelude::*;

#[cfg_attr(not(test), inline(always))]
pub fn pixelsort(config: &FilterConfig, pixels: &mut [RgbaPixel], width: usize, height: usize) {
    let vertical = config.direction == crate::SortDirection::Vertical
        || config.direction == crate::SortDirection::VerticalInverted;

    let inverted = config.direction == crate::SortDirection::HorizontalInverted
        || config.direction == crate::SortDirection::VerticalInverted;

    let sort_above = config.threshold_type == crate::ThresholdType::Above;

    let threshold = (config.threshold * 255.0) as u8;

    if vertical {
        let mut image_array = ArrayViewMut2::from_shape((height, width), pixels).unwrap();

        image_array
            .axis_iter_mut(Axis(1)) // iterate over columns
            .into_par_iter()
            .for_each_init(
                || Vec::with_capacity(height),
                |col_data, mut col| {
                    col_data.clear();
                    for pixel in col.iter() {
                        let l = ((pixel.r as u16 * 77 + pixel.g as u16 * 151 + pixel.b as u16 * 28)
                            >> 8) as u8;
                        col_data.push((l, *pixel));
                    }

                    let mut start = 0;
                    for i in 0..=height {
                        let is_end_of_line = i == height;
                        let should_sort_current = i < height
                            && if sort_above {
                                col_data[i].0 >= threshold
                            } else {
                                col_data[i].0 < threshold
                            };

                        if is_end_of_line || !should_sort_current {
                            let segment = &mut col_data[start..i];
                            segment.sort_unstable_by_key(|k| k.0);
                            if inverted {
                                segment.reverse();
                            }
                            start = i + 1;
                        }
                    }

                    for i in 0..height {
                        col[i] = col_data[i].1;
                    }
                },
            );
    } else {
        // Horizontal sort can be done in-place by processing rows in parallel.
        pixels
            .par_chunks_mut(width) // Each thread gets a mutable slice of a row.
            .for_each_init(
                || Vec::with_capacity(width),
                |row_with_luminance, row_pixels| {
                    row_with_luminance.clear();
                    // Inline luminance calculation for better locality
                    for pixel in row_pixels.iter() {
                        let l = ((pixel.r as u16 * 77 + pixel.g as u16 * 151 + pixel.b as u16 * 28)
                            >> 8) as u8;
                        row_with_luminance.push((l, *pixel));
                    }

                    let mut start = 0;
                    for i in 0..=width {
                        let is_end_of_line = i == width;
                        let should_sort_current = i < width
                            && if sort_above {
                                row_with_luminance[i].0 >= threshold
                            } else {
                                row_with_luminance[i].0 < threshold
                            };

                        if is_end_of_line || !should_sort_current {
                            let segment = &mut row_with_luminance[start..i];
                            segment.sort_unstable_by_key(|k| k.0);
                            if inverted {
                                segment.reverse();
                            }
                            start = i + 1;
                        }
                    }

                    for i in 0..width {
                        row_pixels[i] = row_with_luminance[i].1;
                    }
                },
            );
    }
}
