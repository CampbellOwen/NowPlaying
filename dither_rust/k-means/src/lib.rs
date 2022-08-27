use std::fmt::Display;

use image::{DynamicImage, GenericImage, GenericImageView, Pixel};
use image::{ImageBuffer, Rgb};
use image_utils::{average_lab, compare_f32, f32_equal, oklab_distance, unique_colours};
use oklab::{linear_srgb_to_oklab, Oklab, RGB as OkRGB};

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::Rng;

#[derive(Debug)]
pub struct Cluster {
    pub average_pixel: Oklab,
    pub members: Vec<(u32, u32, Oklab)>,
    pub score: f32,
}

impl Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cluster")
            .field("average_pixel", &self.average_pixel)
            .field("average_error", &self.score)
            .field("members_length", &self.members.len())
            .finish()
    }
}

pub fn cluster(
    img: &DynamicImage,
    num_clusters: u32,
    max_iterations: Option<u32>,
    seed_colour: Option<Oklab>,
) -> Vec<Cluster> {
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

    let mut rng = rand::thread_rng();

    let unique_pixels = unique_colours(&lab_pixels);

    let mut clusters = Vec::new();
    let mut num_clusters = num_clusters;
    if let Some(seed_colour) = seed_colour {
        clusters.push(Cluster {
            average_pixel: seed_colour,
            members: Vec::new(),
            score: 0.0,
        });
        num_clusters -= 1;
    }
    if unique_pixels.len() <= num_clusters as usize {
        for pixel in unique_pixels {
            clusters.push(Cluster {
                average_pixel: pixel,
                members: Vec::new(),
                score: 0.0,
            });
        }
    } else {
        for i in 0..num_clusters {
            if i == 0 {
                let idx = rng.gen_range(0..unique_pixels.len());
                clusters.push(Cluster {
                    average_pixel: unique_pixels[idx],
                    members: Vec::new(),
                    score: 0.0,
                });
                continue;
            }

            let weights: Vec<f32> = unique_pixels
                .iter()
                .map(|pixel| {
                    clusters
                        .iter()
                        .map(|cluster| oklab_distance(pixel, &cluster.average_pixel))
                        .map(|score| score * 100.0)
                        .reduce(f32::min)
                        .expect("Should be a min")
                })
                .collect();

            //println!("{:?}", weights);
            let dist =
                WeightedIndex::new(&weights).expect("Should be able to create a distribution");
            clusters.push(Cluster {
                average_pixel: unique_pixels[dist.sample(&mut rng)],
                members: Vec::new(),
                score: 0.0,
            });
        }
    }

    let max_iterations = max_iterations.unwrap_or(u32::MAX);

    let mut it = 0;
    let mut converged = false;
    while !converged && it < max_iterations {
        clusters
            .iter_mut()
            .for_each(|cluster| cluster.members.clear());

        let prev_scores: Vec<f32> = clusters.iter().map(|cluster| cluster.score).collect();

        lab_pixels.iter().enumerate().for_each(|(i, pixel)| {
            let (best_cluster_idx, _) = clusters
                .iter()
                .enumerate()
                .map(|(idx, cluster)| (idx, oklab_distance(pixel, &cluster.average_pixel)))
                .min_by(|(_, s1), (_, s2)| compare_f32(*s1, *s2))
                .expect("There should always be a best cluster");

            let (x, y) = (i as u32 % w, i as u32 / w);

            clusters[best_cluster_idx].members.push((x, y, *pixel));
        });

        clusters.iter_mut().for_each(|cluster| {
            cluster.average_pixel = average_lab(&cluster.members);
            cluster.score = cluster.members.iter().fold(0.0, |score, (_, _, pixel)| {
                score + oklab_distance(pixel, &cluster.average_pixel)
            }) / cluster.members.len() as f32;
        });

        converged = clusters
            .iter()
            .map(|cluster| cluster.score)
            .zip(prev_scores)
            .all(|(s1, s2)| f32_equal(s1, s2));

        it += 1;
    }

    clusters
}

pub type Rgb8Image = ImageBuffer<Rgb<u8>, Vec<u8>>;
pub fn filter_matching_pixels<I: GenericImageView<Pixel = Rgb<u8>>>(
    img: &I,
    clusters: &[Cluster],
    reference_colour: &Oklab,
    threshold: f32,
) -> Option<(Rgb8Image, Rgb8Image)>
where
    I::Pixel: 'static,
    <I::Pixel as Pixel>::Subpixel: 'static,
{
    let (w, h) = img.dimensions();
    let mut not_matches = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(w, h);
    not_matches
        .copy_from(img, 0, 0)
        .expect("Should be able to copy image");
    let mut matches = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(w, h);
    let matching_clusters: Vec<&Cluster> = clusters
        .iter()
        .filter(|cluster| oklab_distance(&cluster.average_pixel, reference_colour) < threshold)
        .collect();

    if matching_clusters.is_empty() {
        return None;
    }

    matching_clusters.iter().for_each(|&cluster| {
        cluster.members.iter().for_each(|(x, y, _)| {
            let pixel = img.get_pixel(*x, *y);
            let channels = pixel.0;
            let rgb = Rgb::<u8>::from([channels[0], channels[1], channels[2]]);
            matches.put_pixel(*x, *y, rgb);
            not_matches.put_pixel(*x, *y, Rgb::<u8>([0, 0, 0]));
        });
    });

    Some((matches, not_matches))
}
