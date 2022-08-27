use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgb};
use image_utils::{compare_f32, linear_to_srgb, oklab_distance, to_linear};
use oklab::{linear_srgb_to_oklab, oklab_to_linear_srgb, Oklab};

pub enum DitherPattern {
    FloydSteinberg,
}

struct DitherMatrix {
    pub weights: Vec<Vec<f32>>,
}

fn get_dither_matrix(pattern: DitherPattern) -> DitherMatrix {
    match pattern {
        DitherPattern::FloydSteinberg => DitherMatrix {
            weights: vec![vec![7.0 / 16.0], vec![3.0 / 16.0, 5.0 / 16.0, 1.0 / 16.0]],
        },
    }
}

type RemainingColour = Oklab;
#[allow(dead_code)]
fn select_colour(colour: &Oklab, palette: &[Oklab]) -> (Oklab, RemainingColour) {
    let (palette_idx, _) = palette
        .iter()
        .enumerate()
        .map(|(i, p)| (i, oklab_distance(colour, p)))
        .min_by(|(_, dist1), (_, dist2)| compare_f32(*dist1, *dist2))
        .expect("Should always be a best choice");

    let quantized = palette[palette_idx];
    (
        quantized,
        Oklab {
            l: colour.l - quantized.l,
            a: colour.a - quantized.a,
            b: colour.b - quantized.b,
        },
    )
}

fn select_colour_rgb(colour: Rgb<f32>, palette: &[Oklab]) -> (Rgb<f32>, Rgb<f32>) {
    let oklab_colour = linear_srgb_to_oklab(oklab::RGB::from(colour.0));
    let (palette_idx, _) = palette
        .iter()
        .enumerate()
        .map(|(i, p)| (i, oklab_distance(&oklab_colour, p)))
        .min_by(|(_, dist1), (_, dist2)| compare_f32(*dist1, *dist2))
        .expect("Should always be a best choice");

    let oklab::RGB { r, g, b } = oklab_to_linear_srgb(palette[palette_idx]);
    let quantized = Rgb([r, g, b]);
    let [r_c, g_c, b_c] = colour.0;
    let diff = Rgb([r_c - r, g_c - g, b_c - b]);

    (quantized, diff)
}

pub fn dither<I: GenericImageView<Pixel = Rgb<u8>>>(
    img: &I,
    palette: &[Oklab],
    pattern: DitherPattern,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (w, h) = img.dimensions();

    let mut linear_img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(w, h);
    linear_img
        .copy_from(img, 0, 0)
        .expect("Copying image should not fail");
    let linear_img = DynamicImage::ImageRgb8(linear_img);

    let mut linear_img = to_linear(&linear_img);

    //let mut lab_pixels: Vec<Oklab> = img
    //    .pixels()
    //    .map(|(_, _, pixel)| {
    //        let channels = pixel.channels();
    //        srgb_to_oklab(OkRGB::from([channels[0], channels[1], channels[2]]))
    //    })
    //    .collect();

    let dither_matrix = get_dither_matrix(pattern);

    for y in 0..h {
        for x in 0..w {
            //let (quantized, diff) = select_colour(&lab_pixels[i], palette);
            let (quantized, diff) = select_colour_rgb(*linear_img.get_pixel(x, y), palette);

            linear_img.put_pixel(x, y, quantized);

            let width_right = dither_matrix.weights[0].len();
            let total_width = dither_matrix.weights[1].len();
            let left_offset = total_width as u32 - (width_right + 1) as u32;
            for (row_i, row) in dither_matrix.weights.iter().enumerate() {
                for (col, &weight) in row.iter().enumerate() {
                    let (x_i, y_i) = (
                        if row_i == 0 {
                            x as i32 + (col + 1) as i32
                        } else {
                            (x as i32 - left_offset as i32) + col as i32
                        },
                        y as i32 + row_i as i32,
                    );

                    if x_i >= 0 && x_i < w as i32 && y_i >= 0 && y_i < h as i32 {
                        let [r, g, b] = linear_img.get_pixel(x_i as u32, y_i as u32).0;

                        let [diff_r, diff_g, diff_b] = diff.0;

                        let new_col = Rgb::<f32>([
                            r + (diff_r * weight),
                            g + (diff_g * weight),
                            b + (diff_b * weight),
                        ]);
                        linear_img.put_pixel(x_i as u32, y_i as u32, new_col);
                    }
                }
            }
        }
    }

    let mut canvas = ImageBuffer::new(w, h);
    linear_img.enumerate_pixels().for_each(|(x, y, pixel)| {
        let [r, g, b] = pixel.0;

        let srgb = Rgb([
            (linear_to_srgb(r) * 255.0) as u8,
            (linear_to_srgb(g) * 255.0) as u8,
            (linear_to_srgb(b) * 255.0) as u8,
        ]);
        canvas.put_pixel(x, y, srgb);
    });

    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use oklab::{srgb_to_oklab, RGB};

    #[test]
    fn quantize() {
        let colour = srgb_to_oklab(RGB {
            r: 128,
            g: 128,
            b: 128,
        });
        let palette = [
            srgb_to_oklab(RGB { r: 0, g: 0, b: 0 }),
            srgb_to_oklab(RGB {
                r: 255,
                g: 255,
                b: 255,
            }),
        ];

        let (colour, _) = select_colour(&colour, &palette);

        assert_eq!(
            colour,
            srgb_to_oklab(RGB {
                r: 255,
                g: 255,
                b: 255
            })
        );
    }

    #[test]
    fn grey_dither() {
        let img = ImageBuffer::from_fn(16, 16, |_, _| Rgb::from([128, 128, 128]));
        let palette = [
            srgb_to_oklab(RGB { r: 0, g: 0, b: 0 }),
            srgb_to_oklab(RGB {
                r: 255,
                g: 255,
                b: 255,
            }),
        ];

        let res = dither(&img, &palette, DitherPattern::FloydSteinberg);
        assert_eq!(res.dimensions(), (16, 16));
        res.save("test.png").unwrap();
    }

    #[test]
    fn test_dither() {
        let img =
            image::open(r"C:\Users\ocamp\source\repos\NowPlaying\dither_rust\out.png").unwrap();
        let img = img.to_rgb8();
        let palette = [
            srgb_to_oklab(RGB { r: 0, g: 0, b: 0 }),
            srgb_to_oklab(RGB {
                r: 255,
                g: 255,
                b: 255,
            }),
            srgb_to_oklab(RGB { r: 255, g: 0, b: 0 }),
        ];

        let res = dither(&img, &palette, DitherPattern::FloydSteinberg);
        assert_eq!(res.dimensions(), (640, 640));
        res.save("test.png").unwrap();
    }
}
