use std::collections::HashSet;

use clap::Parser;
use image::{imageops, DynamicImage, GenericImage, GenericImageView, ImageBuffer, Pixel, Rgb};

mod kmeans;
use kmeans::{cluster, Cluster};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    input_image: String,

    #[clap(short, long)]
    size: Option<u32>,
}

fn get_resized_image<I: GenericImageView>(
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

fn srgb_to_linear(pixel: &mut Rgb<u8>) {
    pixel.0.iter_mut().for_each(|channel| {
        let c = channel.clone() as f32 / 255.0;
        *channel = ((if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }) * 255.0) as u8;
    });
}

fn linear_to_srgb(pixel: &mut Rgb<u8>) {
    pixel.0.iter_mut().for_each(|channel| {
        let c = channel.clone() as f32 / 255.0;
        *channel = ((if c <= 0.0031308 {
            c * 12.92
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }) * 255.0) as u8;
    });
}

fn main() {
    let Args {
        input_image,
        size: resized_dimension,
    } = Args::parse();

    let img = image::open(&input_image);
    if let Err(err) = img {
        println!("Error loading image \"{}\": {}", &input_image, err);
        return;
    }
    let img = img.unwrap();

    let resized_dimension = resized_dimension.unwrap_or(400);
    let resized: DynamicImage = get_resized_image(&img, resized_dimension).into();

    let mut linear = resized.to_rgb8();
    linear.pixels_mut().for_each(srgb_to_linear);
    let dynamic_linear = linear.into();

    let clusters = cluster(&dynamic_linear, 5, 20);
    clusters
        .iter()
        .for_each(|cluster| println!("{:?}", cluster));

    let mut rgb = img.to_rgb8();

    let colour_height = img.dimensions().1 as usize / clusters.len();
    let colour_width = 30;

    clusters.iter().enumerate().for_each(|(idx, cluster)| {
        let col = cluster.average_pixel.to_rgb();
        let mut col = Rgb(col);
        linear_to_srgb(&mut col);
        ((idx * colour_height)..((idx + 1) * colour_height)).for_each(|y| {
            (0..colour_width).for_each(|x| {
                rgb.put_pixel(x as u32, y as u32, col);
            })
        })
    });

    let mut back = dynamic_linear.to_rgb8();
    back.pixels_mut().for_each(linear_to_srgb);

    //let p = img.get_pixel(10, 10);
    resized.save("resized.png");
    dynamic_linear.save("linear.png");
    back.save("srgb.png");
    rgb.save("out.png");
}
