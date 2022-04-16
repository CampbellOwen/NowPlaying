use clap::Parser;
use image::{imageops, DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgb};

mod kmeans;
use kmeans::cluster;

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

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        (1.055 * c.powf(1.0 / 2.4)) - 0.055
    }
}

fn to_linear(img: &DynamicImage) -> ImageBuffer<Rgb<f32>, Vec<<Rgb<f32> as Pixel>::Subpixel>> {
    let mut converted = img.to_rgb32f();
    converted.pixels_mut().for_each(|pixel| {
        pixel.channels_mut().iter_mut().for_each(|channel| {
            *channel = srgb_to_linear(*channel);
        });
    });

    converted
}

fn to_srgb(img: &DynamicImage) -> ImageBuffer<Rgb<f32>, Vec<<Rgb<f32> as Pixel>::Subpixel>> {
    let mut converted = img.to_rgb32f();
    converted.pixels_mut().for_each(|pixel| {
        pixel.channels_mut().iter_mut().for_each(|channel| {
            *channel = linear_to_srgb(*channel);
        });
    });

    converted
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
