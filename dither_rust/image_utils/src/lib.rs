use std::cmp::Ordering;

use image::{imageops, DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixel, Rgb};
use oklab::Oklab;

pub fn average_lab(pixels: &[(u32, u32, Oklab)]) -> Oklab {
    let (l, a, b) = pixels
        .iter()
        .copied()
        .map(|(_, _, p)| p)
        .fold((0.0, 0.0, 0.0), |(l, a, b), pixel| {
            (l + pixel.l as f64, a + pixel.a as f64, b + pixel.b as f64)
        });

    let n = pixels.len() as f64;
    let avg = Oklab {
        l: (l / n) as f32,
        a: (a / n) as f32,
        b: (b / n) as f32,
    };

    if avg.l.is_nan() || avg.a.is_nan() || avg.b.is_nan() {
        return Oklab {
            l: 0.0,
            a: 0.0,
            b: 0.0,
        };
    }

    avg
}

pub fn oklab_distance(col1: &Oklab, col2: &Oklab) -> f32 {
    if lab_equal(col1, col2) {
        return 0.0;
    }

    let l_1 = col1.l as f64;
    let l_2 = col2.l as f64;
    let a_1 = col1.a as f64;
    let a_2 = col2.a as f64;
    let b_1 = col1.b as f64;
    let b_2 = col2.b as f64;

    let delta_l = l_1 - l_2;
    if delta_l.is_nan() {
        return 0.0;
    }

    let c1 = (a_1.powi(2) + b_1.powi(2)).sqrt();
    if c1.is_nan() {
        return 0.0;
    }
    let c2 = (a_2.powi(2) + b_2.powi(2)).sqrt();
    if c2.is_nan() {
        return 0.0;
    }
    let delta_c = c1 - c2;
    let delta_c = if delta_c.abs() < 1e-5 { 0.0 } else { delta_c };

    let delta_a = a_1 - a_2;
    let delta_a = if delta_a.abs() < 1e-5 { 0.0 } else { delta_a };

    let delta_b = b_1 - b_2;
    let delta_b = if delta_b.abs() < 1e-5 { 0.0 } else { delta_b };

    let delta_h = delta_a.powi(2) + delta_b.powi(2) - delta_c.powi(2);
    let delta_h = if delta_h.abs() < 1e-5 { 0.0 } else { delta_h };

    (delta_l.powi(2) + delta_c.powi(2) + delta_h).sqrt() as f32
}

pub fn oklab_distance2(col1: &Oklab, col2: &Oklab) -> f32 {
    ((col1.l - col2.l).powi(2) + (col1.a - col2.a).powi(2) + (col1.b - col2.b).powi(2)).sqrt()
}

pub fn lab_equal(c1: &Oklab, c2: &Oklab) -> bool {
    f32_equal(c1.l, c2.l) && f32_equal(c1.a, c2.a) && f32_equal(c1.b, c2.b)
}

pub fn f32_equal(x: f32, y: f32) -> bool {
    let epsilon = 1e-5;
    f32::abs(x - y) < epsilon
}

pub fn unique_colours(colours: &[Oklab]) -> Vec<Oklab> {
    let mut unique = Vec::new();

    colours.iter().for_each(|c| {
        let mut clash = false;
        for existing in &unique {
            if lab_equal(c, existing) {
                clash = true;
                break;
            }
        }
        if !clash {
            unique.push(*c);
        }
    });

    unique
}

pub fn compare_f32(x1: f32, x2: f32) -> Ordering {
    if f32_equal(x1, x2) {
        Ordering::Equal
    } else if (x1 - x2) < 0.0 {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}
pub fn get_resized_image<I: GenericImageView>(
    img: &I,
    biggest_dimension: u32,
) -> ImageBuffer<I::Pixel, Vec<<I::Pixel as Pixel>::Subpixel>>
where
    I::Pixel: 'static,
    <I::Pixel as Pixel>::Subpixel: 'static,
{
    let (w, h) = img.dimensions();
    let (w, h) = if w > h {
        let aspect_ratio = h as f32 / w as f32;
        (
            biggest_dimension,
            (biggest_dimension as f32 * aspect_ratio) as u32,
        )
    } else {
        let aspect_ratio = w as f32 / h as f32;
        (
            (biggest_dimension as f32 * aspect_ratio) as u32,
            biggest_dimension,
        )
    };

    imageops::resize(img, w, h, imageops::FilterType::CatmullRom)
}

pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        (1.055 * c.powf(1.0 / 2.4)) - 0.055
    }
}

pub fn to_linear(img: &DynamicImage) -> ImageBuffer<Rgb<f32>, Vec<<Rgb<f32> as Pixel>::Subpixel>> {
    let mut converted = img.to_rgb32f();
    converted.pixels_mut().for_each(|pixel| {
        pixel.channels_mut().iter_mut().for_each(|channel| {
            *channel = srgb_to_linear(*channel);
        });
    });

    converted
}

pub fn to_srgb(img: &DynamicImage) -> ImageBuffer<Rgb<f32>, Vec<<Rgb<f32> as Pixel>::Subpixel>> {
    let mut converted = img.to_rgb32f();
    converted.pixels_mut().for_each(|pixel| {
        pixel.channels_mut().iter_mut().for_each(|channel| {
            *channel = linear_to_srgb(*channel);
        });
    });

    converted
}

pub fn colour_bars<I: GenericImageView<Pixel = Rgb<u8>>>(
    img: &I,
    colours: &[Rgb<u8>],
    colour_width: u32,
) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let (w, h) = img.dimensions();
    let colour_height = h as usize / colours.len();

    let mut canvas = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(w + colour_width, h);
    canvas
        .copy_from(img, colour_width, 0)
        .expect("Failed to paste original image");
    colours.iter().copied().enumerate().for_each(|(idx, col)| {
        ((idx * colour_height)..((idx + 1) * colour_height)).for_each(|y| {
            (0..colour_width).for_each(|x| {
                canvas.put_pixel(x as u32, y as u32, col);
            })
        })
    });

    canvas
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
