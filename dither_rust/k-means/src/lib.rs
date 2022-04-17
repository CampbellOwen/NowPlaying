use std::cmp::Ordering;
use std::fmt::Display;

use image::{DynamicImage, GenericImageView, Pixel};
use oklab::{linear_srgb_to_oklab, Oklab, RGB as OkRGB};

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::Rng;

#[derive(Debug)]
pub struct Cluster {
    pub average_pixel: Oklab,
    pub members: Vec<Oklab>,
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

pub fn average_lab(pixels: &[Oklab]) -> Oklab {
    let (l, a, b) = pixels
        .iter()
        .copied()
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

    return avg;
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

fn lab_equal(c1: &Oklab, c2: &Oklab) -> bool {
    f32_equal(c1.l, c2.l) && f32_equal(c1.a, c2.a) && f32_equal(c1.b, c2.b)
}

fn f32_equal(x: f32, y: f32) -> bool {
    let epsilon = 1e-5;
    f32::abs(x - y) < epsilon
}

fn unique_colours(colours: &[Oklab]) -> Vec<Oklab> {
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

fn compare_f32(x1: f32, x2: f32) -> Ordering {
    if f32_equal(x1, x2) {
        Ordering::Equal
    } else if (x1 - x2) < 0.0 {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

pub fn cluster(img: &DynamicImage, num_clusters: u32, max_iterations: Option<u32>) -> Vec<Cluster> {
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
                    .map(|cluster| oklab_distance(&pixel, &cluster.average_pixel))
                    .map(|score| score * 100.0)
                    .reduce(f32::min)
                    .expect("Should be a min")
            })
            .collect();

        //println!("{:?}", weights);
        let dist = WeightedIndex::new(&weights).expect("Should be able to create a distribution");
        clusters.push(Cluster {
            average_pixel: unique_pixels[dist.sample(&mut rng)],
            members: Vec::new(),
            score: 0.0,
        });
    }

    let max_iterations = max_iterations.unwrap_or(u32::MAX);

    let mut it = 0;
    let mut converged = false;
    while !converged && it < max_iterations {
        clusters
            .iter_mut()
            .for_each(|cluster| cluster.members.clear());

        let prev_scores: Vec<f32> = clusters.iter().map(|cluster| cluster.score).collect();

        lab_pixels.iter().for_each(|pixel| {
            let (best_cluster_idx, _) = clusters
                .iter()
                .enumerate()
                .map(|(idx, cluster)| (idx, oklab_distance(pixel, &cluster.average_pixel)))
                .min_by(|(_, s1), (_, s2)| compare_f32(*s1, *s2))
                .expect("There should always be a best cluster");

            clusters[best_cluster_idx].members.push(*pixel);
        });

        clusters.iter_mut().for_each(|cluster| {
            cluster.average_pixel = average_lab(&cluster.members);
            cluster.score = cluster.members.iter().fold(0.0, |score, pixel| {
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
