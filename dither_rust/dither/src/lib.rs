use image::{GenericImageView, ImageBuffer, Pixel, Rgb};
use oklab::{linear_srgb_to_oklab, Oklab, RGB as OkRGB};

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

fn select_colour(colour: &Oklab, palette: &[Oklab]) -> (Oklab, Oklab) {
    unimplemented!()
}

pub fn dither<I: GenericImageView<Pixel = Rgb<u8>>>(
    img: &I,
    palette: &[Oklab],
    pattern: DitherPattern,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (w, _) = img.dimensions();
    let lab_pixels: Vec<Oklab> = img
        .pixels()
        .map(|(_, _, pixel)| {
            let rgb = pixel.to_rgb();
            let channels = rgb.channels();
            linear_srgb_to_oklab(OkRGB::from([
                channels[0] as f32 / 255.0,
                channels[1] as f32 / 255.0,
                channels[2] as f32 / 255.0,
            ]))
        })
        .collect();

    unimplemented!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
