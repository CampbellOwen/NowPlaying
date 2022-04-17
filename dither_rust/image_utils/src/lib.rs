use image::{imageops, DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixel, Rgb};

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
