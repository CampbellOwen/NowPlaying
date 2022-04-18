use image::{GenericImageView, ImageBuffer, Pixel, Rgb};
use image_utils::{compare_f32, oklab_distance};
use oklab::{linear_srgb_to_oklab, oklab_to_srgb, Oklab, RGB as OkRGB};

pub enum DitherPattern {
    FloydSteinberg,
}

struct DitherMatrix {
    pub weights: Vec<Vec<f64>>,
}

fn get_dither_matrix(pattern: DitherPattern) -> DitherMatrix {
    match pattern {
        DitherPattern::FloydSteinberg => DitherMatrix {
            weights: vec![vec![7.0 / 16.0], vec![3.0 / 16.0, 5.0 / 16.0, 1.0 / 16.0]],
        },
    }
}

type RemainingColour = Oklab;
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

pub fn dither<I: GenericImageView<Pixel = Rgb<u8>>>(
    img: &I,
    palette: &[Oklab],
    pattern: DitherPattern,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (w, h) = img.dimensions();
    let mut lab_pixels: Vec<Oklab> = img
        .pixels()
        .map(|(_, _, pixel)| {
            let channels = pixel.channels();
            linear_srgb_to_oklab(OkRGB::from([
                channels[0] as f32 / 255.0,
                channels[1] as f32 / 255.0,
                channels[2] as f32 / 255.0,
            ]))
        })
        .collect();

    let dither_matrix = get_dither_matrix(pattern);

    for i in 0..lab_pixels.len() {
        let (x, y) = (i as u32 % w, i as u32 / w);
        let (quantized, diff) = select_colour(&lab_pixels[i], palette);
        lab_pixels[i] = quantized;

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
                    let idx = (y_i as u32 * w) + x_i as u32;

                    let col = lab_pixels[idx as usize];
                    let new_col = Oklab {
                        l: col.l + (diff.l * weight as f32),
                        a: col.a + (diff.a * weight as f32),
                        b: col.b + (diff.b * weight as f32),
                    };
                    lab_pixels[idx as usize] = new_col;
                }
            }
        }
    }

    let mut canvas = ImageBuffer::new(w, h);
    lab_pixels.iter().enumerate().for_each(|(i, pixel)| {
        let i = i as u32;
        let (x, y) = (i % w, i / w);
        let srgb = oklab_to_srgb(*pixel);
        let srgb = Rgb([srgb.r, srgb.g, srgb.b]);
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

        let (colour, diff) = select_colour(&colour, &palette);

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
        res.save("test.png");
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
        res.save("test.png");
    }
}
