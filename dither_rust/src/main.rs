use std::collections::HashSet;

use clap::Parser;
use image::{imageops, GenericImage, GenericImageView, ImageBuffer, Pixel};

mod kmeans;
use kmeans::{cluster, Cluster};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
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

    let img = img.to_rgb8();

    let resized_dimension = resized_dimension.unwrap_or(50);
    let resized = get_resized_image(&img, resized_dimension);

    let clusters = cluster(&img, 5);
    println!("{:?}", clusters);

    //let p = img.get_pixel(10, 10);
    resized.save("out.png");
}
