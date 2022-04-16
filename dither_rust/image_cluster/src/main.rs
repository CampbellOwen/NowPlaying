use clap::Parser;
use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};

use image_utils::*;
use k_means::cluster;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    input_image: String,

    #[clap(short, long)]
    size: Option<u32>,
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

    let linear: DynamicImage = to_linear(&resized).into();

    let clusters = cluster(&linear, 5);
    clusters
        .iter()
        .for_each(|cluster| println!("{:?}", cluster));

    let mut rgb = img.to_rgb8();

    let colour_height = img.dimensions().1 as usize / clusters.len();
    let colour_width = 30;

    let colours = clusters.iter().map(|c| {
        let rgb8 = c.average_pixel.to_rgb();
        let mut srgb_iter = rgb8
            .iter()
            .map(|&x| x as f32 / 255.0)
            .map(linear_to_srgb)
            .map(|f| (f * 255.0) as u8);
        let srgb = [
            srgb_iter.next().unwrap(),
            srgb_iter.next().unwrap(),
            srgb_iter.next().unwrap(),
        ];
        Rgb(srgb)
    });

    colours.enumerate().for_each(|(idx, col)| {
        ((idx * colour_height)..((idx + 1) * colour_height)).for_each(|y| {
            (0..colour_width).for_each(|x| {
                rgb.put_pixel(x as u32, y as u32, col);
            })
        })
    });

    let back = to_srgb(&linear);

    //let p = img.get_pixel(10, 10);
    resized.save("resized.png");
    linear.save("linear.png");
    back.save("srgb.png");
    rgb.save("out.png");
}
