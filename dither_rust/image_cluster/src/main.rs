use std::num::ParseIntError;

use clap::Parser;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
use itertools::Itertools;
use oklab::{oklab_to_srgb, srgb_to_oklab, RGB};

use dither::*;

use image_utils::*;
use k_means::{cluster, filter_matching_pixels};

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

    /// Mask the image if any of the clusters match the colour
    #[clap(short = 'c', long)]
    mask_colour: Option<String>,
}

fn parse_colour(str: &str) -> Option<[u8; 3]> {
    let channels: Vec<Result<u8, ParseIntError>> = str
        .chars()
        .tuples()
        .map(|(first, second)| {
            let mut s = String::new();
            s.push(first);
            s.push(second);
            u8::from_str_radix(&s, 16)
        })
        .collect();
    if channels.len() < 3 || channels.iter().any(|r| r.is_err()) {
        return None;
    }

    let mut channels_it = channels.iter().cloned().map(|r| r.unwrap());
    Some([
        channels_it.next().unwrap(),
        channels_it.next().unwrap(),
        channels_it.next().unwrap(),
    ])
}

fn main() {
    let Args {
        input_image,
        image_size: resized_dimension,
        annotated_image_path: out_path,
        num_clusters,
        max_iterations,
        mask_colour,
    } = Args::parse();

    let img = image::open(&input_image);
    if let Err(err) = img {
        println!("Error loading image \"{}\": {}", &input_image, err);
        return;
    }

    let mask_colour = mask_colour.map(|s| {
        parse_colour(&s).unwrap_or_else(|| {
            panic!(
                "Incorrect colour format: {}\nPlease specify colour as RRGGBB",
                s
            )
        })
    });

    if mask_colour.is_some() && out_path.is_none() {
        panic!("If mask_colour is set, annotated_image_path should also be set");
    }

    let original = img.unwrap();
    let img = resized_dimension
        .map(|size| get_resized_image(&original, size).into())
        .unwrap_or_else(|| original.clone());

    let linear: DynamicImage = to_linear(&img).into();

    let clusters = cluster(&linear, num_clusters, max_iterations);

    let colours: Vec<Rgb<u8>> = clusters
        .iter()
        .enumerate()
        .map(|(_, c)| {
            //if i < (clusters.len() - 1) {
            //    clusters[i + 1..].iter().for_each(|c2| {
            //        let colour1 = c.average_pixel;
            //        let colour2 = c2.average_pixel;
            //        let diff = Lab::squared_distance(&colour1, &colour2);
            //        println!("{:?}\n{:?}\n\t{}", colour1, colour2, diff);
            //    });
            //}
            let srgb8 = oklab_to_srgb(c.average_pixel);
            Rgb([srgb8.r, srgb8.g, srgb8.b])
        })
        .collect();

    clusters.iter().enumerate().for_each(|(i, cluster)| {
        let [r, g, b] = colours[i].0;
        println!(
            "{{ colour: #{:02X}{:02X}{:02X}, average_error: {}, num_pixels: {} }}",
            r,
            g,
            b,
            cluster.score,
            cluster.members.len()
        )
    });

    if let Some(out_path) = out_path {
        let rgb = original.to_rgb8();

        if let Some(mask_rgb) = mask_colour {
            let oklab_mask = srgb_to_oklab(RGB::from(mask_rgb));

            if let Some((matched, not_matched)) =
                filter_matching_pixels(&rgb, &clusters, &oklab_mask)
            {
                matched
                    .save(&out_path)
                    .unwrap_or_else(|_| panic!(" Failed writing output image to {}", out_path));
                not_matched.save("not_matched.png").unwrap_or_else(|_| {
                    panic!(" Failed writing output image to {}", "not_matched.png")
                });

                let red_palette = [
                    srgb_to_oklab(RGB { r: 0, g: 0, b: 0 }),
                    srgb_to_oklab(RGB {
                        r: 255,
                        g: 255,
                        b: 255,
                    }),
                    srgb_to_oklab(RGB { r: 255, g: 0, b: 0 }),
                ];
                let red_dithered = dither(&matched, &red_palette, DitherPattern::FloydSteinberg);

                red_dithered.save("red_dithered.png");

                let bw_palette = [
                    srgb_to_oklab(RGB { r: 0, g: 0, b: 0 }),
                    srgb_to_oklab(RGB {
                        r: 255,
                        g: 255,
                        b: 255,
                    }),
                ];

                let bw = DynamicImage::ImageRgb8(not_matched).to_luma16();
                let bw = DynamicImage::ImageLuma16(bw).to_rgb8();

                let bw_dithered = dither(&bw, &bw_palette, DitherPattern::FloydSteinberg);
                bw_dithered.save("bw_dithered.png");

                let (w, h) = img.dimensions();
                let mut combined = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(w, h);

                let matched_pixels: Vec<(u32, u32)> = clusters
                    .iter()
                    .filter(|cluster| oklab_distance(&cluster.average_pixel, &oklab_mask) < 0.11)
                    .map(|cluster| {
                        cluster
                            .members
                            .iter()
                            .map(|(x, y, _)| (*x, *y))
                            .collect::<Vec<(u32, u32)>>()
                    })
                    .reduce(|mut master, list| {
                        master.extend(list);
                        master
                    })
                    .expect("Should have results");

                for y in 0..h {
                    for x in 0..w {
                        let pixel = if matched_pixels.contains(&(x, y)) {
                            red_dithered.get_pixel(x, y)
                        } else {
                            bw_dithered.get_pixel(x, y)
                        };
                        combined.put_pixel(x, y, pixel.clone());
                    }
                }

                combined.save("combined.png");
            } else {
                let bw = DynamicImage::ImageRgb8(rgb).to_luma16();
                let bw = DynamicImage::ImageLuma16(bw).to_rgb8();
                let bw_palette = [
                    srgb_to_oklab(RGB { r: 0, g: 0, b: 0 }),
                    srgb_to_oklab(RGB {
                        r: 255,
                        g: 255,
                        b: 255,
                    }),
                ];
                let bw_dithered = dither(&bw, &bw_palette, DitherPattern::FloydSteinberg);
                bw_dithered.save("combined.png");
            }
        } else {
            let canvas = colour_bars(&rgb, &colours, 30);
            canvas
                .save(&out_path)
                .unwrap_or_else(|_| panic!(" Failed writing output image to {}", out_path));
        }
    }
}
