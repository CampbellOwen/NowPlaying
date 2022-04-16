use clap::Parser;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, Rgb};

use image_utils::*;
use k_means::cluster;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the image to analyze
    input_image: String,

    /// If specified, will resize the image such that the biggest side will be length <IMAGE_SIZE>
    #[clap(short = 's', long)]
    image_size: Option<u32>,

    /// If specified, will save an image to the path provided with the image clusters annotated on the side
    #[clap(short = 'o', long)]
    annotated_image_path: Option<String>,

    /// If specified, will stop the k-means clustering after <MAX_ITERATIONS> iterations
    #[clap(short, long)]
    max_iterations: Option<u32>,

    /// Cluster the image into <NUM_CLUSTERS> clusters
    #[clap(short, long)]
    num_clusters: u32,
}

fn main() {
    let Args {
        input_image,
        image_size: resized_dimension,
        annotated_image_path: out_path,
        num_clusters,
        max_iterations,
    } = Args::parse();

    let img = image::open(&input_image);
    if let Err(err) = img {
        println!("Error loading image \"{}\": {}", &input_image, err);
        return;
    }
    let original = img.unwrap();
    let img = resized_dimension
        .map(|size| get_resized_image(&original, size).into())
        .unwrap_or_else(|| original.clone());

    let linear: DynamicImage = to_linear(&img).into();

    let clusters = cluster(&linear, num_clusters, max_iterations);

    let colours: Vec<Rgb<u8>> = clusters
        .iter()
        .map(|c| {
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
        })
        .collect();

    clusters.iter().enumerate().for_each(|(i, cluster)| {
        let [r, g, b] = colours[i].0;
        println!(
            "{{ colour: #{:X}{:X}{:X}, average_error: {}, num_pixels: {} }}",
            r,
            g,
            b,
            cluster.score,
            cluster.members.len()
        )
    });

    if out_path.is_some() {
        let rgb = original.to_rgb8();

        let (w, h) = rgb.dimensions();

        let colour_height = original.dimensions().1 as usize / clusters.len();
        let colour_width = 30;

        let mut canvas = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(w + colour_width, h);
        canvas
            .copy_from(&rgb, colour_width, 0)
            .expect("Failed to paste original image");
        colours.iter().copied().enumerate().for_each(|(idx, col)| {
            ((idx * colour_height)..((idx + 1) * colour_height)).for_each(|y| {
                (0..colour_width).for_each(|x| {
                    canvas.put_pixel(x as u32, y as u32, col);
                })
            })
        });
        out_path.iter().for_each(|out_path| {
            canvas
                .save(out_path)
                .unwrap_or_else(|_| panic!(" Failed writing output image to {}", out_path));
        });
    }
}
